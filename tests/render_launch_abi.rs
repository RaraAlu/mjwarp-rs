#![cfg(feature = "cuda-probe")]
//! 渲染/BVH S7a 验收：渲染发射 ABI、W15+NVRTC 编译与拒绝面。
//!
//! 参考：冻结上游 `_render_megakernel` 的真实 `extern "C"` 签名（Warp 1.15 生成源，
//! 见 `docs/development-evidence/render-bvh/s7a-launch-abi/`）与 W15 头树的
//! device C++ 布局（S7a 预检 static_assert）。
//!
//! 主机用例默认执行；NVRTC 用例默认 ignore：
//! cargo test --locked --features cuda-probe --test render_launch_abi -- --ignored --test-threads=1

use mjwarp_rs::render::kernel::launch::{W15_HEADERS_DIR, compile_render_artifact};
use mjwarp_rs::render::kernel::{
    RENDER_PRECISION_GATE, SVD_DOUBLE_CONSTANTS, check_render_ptx, render_recipe_gate,
};
use mjwarp_rs::runtime::launcher::{
    BlockRule, CompileRecipe, DeviceSpan, LaunchDims, LauncherError, MEGAKERNEL_SLOTS,
    PrecisionGate, RecipePurpose, RenderArg, RenderArgKind, RenderArray, RenderArrayValue,
    RenderElem, RenderPlan, RenderScalar, RenderScalarKind, RenderSignature, Texture2d, WarpArray,
};

/// 编译期探针：包含 W15 三头文件并调用硬件纹理采样（证明头树在 NVRTC 下可用）。
const W15_PROBE: &str = r#"
#define WP_NO_CRT
#define WP_NO_BFLOAT16
#include "builtin.h"
#include "bvh.h"
#include "texture.h"

extern "C" __global__ void s7a_w15_probe(
    wp::array_t<wp::texture2d_t> textures,
    wp::array_t<wp::vec2> uvs,
    wp::array_t<wp::vec4> out,
    int count)
{
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) {
        return;
    }
    wp::vec4 sampled = wp::texture_sample<wp::vec4>(textures[i], uvs[i], -1.0f);
    out[i] = sampled;
}
"#;

fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).expect("设备区间")
}

fn warp_grid() -> LaunchDims {
    LaunchDims::new((1, 1, 1), (256, 1, 1), 0).expect("发射维度")
}

#[test]
fn megakernel_signature_covers_upstream_argument_classes() {
    let slots = &MEGAKERNEL_SLOTS;
    assert_eq!(slots.len(), 82, "1 bounds + 73 数组 + 7 标量");
    assert_eq!(slots[0], RenderArgKind::Bounds2D);
    let arrays = slots
        .iter()
        .filter(|slot| matches!(slot, RenderArgKind::Array(_)))
        .count();
    let scalars = slots
        .iter()
        .filter(|slot| matches!(slot, RenderArgKind::Scalar(_)))
        .count();
    assert_eq!((arrays, scalars), (73, 8));
    for elem in RenderElem::ALL {
        assert!(
            slots.contains(&RenderArgKind::Array(elem)),
            "缺少数组元素类 {}",
            elem.name()
        );
    }
    for kind in [
        RenderScalarKind::I32,
        RenderScalarKind::U64,
        RenderScalarKind::Bool,
    ] {
        assert!(
            slots.contains(&RenderArgKind::Scalar(kind)),
            "缺少标量类 {}",
            kind.name()
        );
    }
    let signature = RenderSignature::MEGAKERNEL;
    assert_eq!(signature.name(), "_render_megakernel");
    assert_eq!(signature.len(), 82);
    assert_eq!(signature.block(), BlockRule::WarpGrid);
    assert_eq!(signature.block().threads(), 256);
}

#[test]
fn element_widths_and_layouts_match_w15_device_static_asserts() {
    for (elem, bytes, align) in [
        (RenderElem::I32, 4, 4),
        (RenderElem::U32, 4, 4),
        (RenderElem::U64, 8, 8),
        (RenderElem::Bool, 1, 1),
        (RenderElem::F32, 4, 4),
        (RenderElem::Vec2, 8, 4),
        (RenderElem::Vec3, 12, 4),
        (RenderElem::Vec4, 16, 4),
        (RenderElem::Vec2I, 8, 4),
        (RenderElem::Vec3I, 12, 4),
        (RenderElem::Mat33, 36, 4),
        (RenderElem::Quat, 16, 4),
        (RenderElem::Texture2d, 24, 8),
    ] {
        assert_eq!(elem.bytes(), bytes, "{} 元素宽度", elem.name());
        assert_eq!(elem.align(), align, "{} 元素对齐", elem.name());
    }
    assert_eq!(std::mem::size_of::<Texture2d>(), 24);
    assert_eq!(std::mem::align_of::<Texture2d>(), 8);
    assert_eq!(std::mem::size_of::<RenderArray>(), 56);
    // 与既有 Warp 数组描述符逐项 parity（步长与字节数）。
    for (elem, shape) in [
        (RenderElem::F32, &[3usize, 4][..]),
        (RenderElem::Vec3, &[2, 3][..]),
        (RenderElem::Mat33, &[2][..]),
        (RenderElem::Texture2d, &[5][..]),
        (RenderElem::Bool, &[7][..]),
    ] {
        let render = RenderArray::contiguous(0x1000, elem, shape).expect("渲染描述符");
        let warp =
            WarpArray::<f32>::with_element_bytes(0x1000, shape, elem.bytes()).expect("Warp 描述符");
        assert_eq!(render.strides(), warp.strides(), "{} strides", elem.name());
        // WarpArray::byte_len 按 f32 元素计；渲染侧按元素类宽度计，故对宽度类显式核对。
        let expected_bytes = elem.bytes() * shape.iter().product::<usize>() as u64;
        assert_eq!(render.byte_len(elem).unwrap(), expected_bytes);
        assert_eq!(render.shape(), warp.shape());
    }
    // 纹理按值结构与句柄字段的编码/解码往返。
    let texture = Texture2d {
        tex: 0x0102_0304_0506_0708,
        width: 64,
        height: 32,
        num_channels: 4,
    };
    let mut bytes = [0u8; Texture2d::BYTES];
    texture.encode(&mut bytes);
    assert_eq!(Texture2d::decode(&bytes), texture);
}

#[test]
fn render_recipe_is_frozen_and_gate_is_enforced() {
    let recipe = CompileRecipe::WARP_1_15_RENDER;
    assert_eq!(recipe.purpose(), RecipePurpose::Render);
    assert_eq!(recipe.gate(), PrecisionGate::F32Render);
    assert_eq!(render_recipe_gate(), PrecisionGate::F32Render);
    assert_eq!(RENDER_PRECISION_GATE, "f32-render");
    assert_eq!(
        recipe.nvrtc_options(),
        CompileRecipe::WARP_1_15_PHYSICS.nvrtc_options()
    );
    assert!(recipe.fmad());
    assert!(!recipe.fast_math());
    assert!(recipe.admit(RecipePurpose::Render).is_ok());
    // Diagnostic 不得用于产品 render（下游只能经 WARP_1_15_RENDER 编译）。
    assert!(
        CompileRecipe::WARP_1_15_DIAGNOSTIC
            .admit(RecipePurpose::Render)
            .is_err()
    );
    // 放行清单只有两条精确行（符号 + 字面量 + 行尾）。
    assert_eq!(SVD_DOUBLE_CONSTANTS.len(), 2);
    for line in SVD_DOUBLE_CONSTANTS {
        assert!(line.starts_with(".global .align 8 .f64 _ZN2wp11_svd_configIdE"));
        assert!(line.ends_with("= 0d3D719799812DEA11;"));
    }
}

static TEST_SLOTS: [RenderArgKind; 4] = [
    RenderArgKind::Bounds2D,
    RenderArgKind::Array(RenderElem::Vec3),
    RenderArgKind::Scalar(RenderScalarKind::U64),
    RenderArgKind::Array(RenderElem::I32),
];

fn plan_args(count: usize) -> Vec<RenderArg> {
    let context = 7u64;
    let first = RenderArrayValue::contiguous(span(context, 0x1000, 48), RenderElem::Vec3, &[4])
        .expect("vec3 数组");
    let second = RenderArrayValue::contiguous(span(context, 0x2000, 32), RenderElem::I32, &[8])
        .expect("i32 数组");
    let mut args = vec![
        RenderArg::Bounds2D(
            mjwarp_rs::runtime::launcher::LaunchBounds2::new([2, 2]).expect("边界"),
        ),
        RenderArg::Array(first),
        RenderArg::Scalar(RenderScalar::U64(9)),
        RenderArg::Array(second),
    ];
    args.truncate(count);
    args
}

#[test]
fn render_plan_rejects_wrong_count_kind_alignment_capacity_context_and_block() {
    let signature =
        RenderSignature::new("abi_probe", BlockRule::WarpGrid, &TEST_SLOTS).expect("签名");
    let args = plan_args(4);
    let plan = RenderPlan::plan(signature, &args, warp_grid()).expect("计划");
    assert_eq!(plan.dims().block(), (256, 1, 1));

    // 数量不符。
    assert_eq!(
        RenderPlan::plan(signature, &args[..3], warp_grid()).unwrap_err(),
        LauncherError::RenderArgCount {
            expected: 4,
            actual: 3,
        }
    );
    // 种类不符（把 u64 标量换成 i32）。
    let mut wrong_kind = args.clone();
    wrong_kind[2] = RenderArg::Scalar(RenderScalar::I32(9));
    assert!(matches!(
        RenderPlan::plan(signature, &wrong_kind, warp_grid()),
        Err(LauncherError::RenderArgKind { slot: 2, .. })
    ));
    // 首槽位不是边界。
    assert!(matches!(
        RenderSignature::new("abi_probe", BlockRule::WarpGrid, &TEST_SLOTS[1..]),
        Err(LauncherError::InvalidSignature { .. })
    ));
    // 线程块与签名规则不符。
    let other_block = LaunchDims::new((1, 1, 1), (128, 1, 1), 0).unwrap();
    assert_eq!(
        RenderPlan::plan(signature, &args, other_block).unwrap_err(),
        LauncherError::RenderBounds { slot: 0 }
    );
    // 地址未按元素对齐（vec3 align 4；texture2d align 8）。
    assert!(matches!(
        RenderArrayValue::contiguous(span(7, 0x1002, 48), RenderElem::Vec3, &[4]),
        Err(LauncherError::RenderArrayAlign { slot: 0 })
    ));
    assert!(matches!(
        RenderArrayValue::contiguous(span(7, 0x1004, 24), RenderElem::Texture2d, &[1]),
        Err(LauncherError::RenderArrayAlign { slot: 0 })
    ));
    // 容量不足。
    assert!(matches!(
        RenderArrayValue::contiguous(span(7, 0x1000, 24), RenderElem::Vec3, &[4]),
        Err(LauncherError::Capacity { slot: 0, .. })
    ));
    // 跨上下文。
    let stray =
        RenderArrayValue::contiguous(span(8, 0x3000, 32), RenderElem::I32, &[8]).expect("i32 数组");
    let mut mixed = args.clone();
    mixed[3] = RenderArg::Array(stray);
    assert!(matches!(
        RenderPlan::plan(signature, &mixed, warp_grid()),
        Err(LauncherError::ContextMismatch { slot: 3 })
    ));
    // 形状非法。
    assert!(RenderArray::contiguous(0x1000, RenderElem::F32, &[]).is_err());
    assert!(RenderArray::contiguous(0x1000, RenderElem::F32, &[2, 0]).is_err());
}

#[test]
#[ignore = "需要 NVRTC 与 W15 头树（无 GPU 也可，但按窗口统一执行）"]
fn nvrtc_compiles_w15_headers_under_render_recipe() {
    let artifact = compile_render_artifact("s7a_w15_probe", W15_PROBE, "s7a_w15_probe")
        .expect("W15 头树 + 渲染门禁编译");
    let ptx = artifact.ptx();
    assert!(ptx.contains(".visible .entry s7a_w15_probe"));
    assert!(ptx.contains("tex.2d"), "硬件纹理取指路径");
    let svd_lines = ptx
        .lines()
        .filter(|line| line.trim().ends_with(';') && line.contains(".global .align 8 .f64"))
        .map(str::trim)
        .count();
    assert_eq!(svd_lines, 2, "只允许 W15 的两条 f64 常量声明");
    for allowed in SVD_DOUBLE_CONSTANTS {
        assert!(
            ptx.lines().any(|line| line.trim() == allowed),
            "缺少放行常量 {allowed}"
        );
    }
    assert_eq!(check_render_ptx(ptx, "s7a_w15_probe"), Ok(()));
    println!(
        "s7a w15: headers={} source_sha256={} ptx_bytes={} ptx_sha256={} gate={}",
        W15_HEADERS_DIR,
        artifact.source_sha256(),
        ptx.len(),
        artifact.ptx_sha256(),
        RENDER_PRECISION_GATE
    );
}

#[test]
#[ignore = "需要 NVRTC 与 W15 头树（无 GPU 也可，但按窗口统一执行）"]
fn render_gate_rejects_ptx_without_allowlisted_constants_or_with_extra_f64() {
    // 真实 W15 产物通过；人为插入第三条 f64 全局声明必须被拒。
    let artifact =
        compile_render_artifact("s7a_w15_probe", W15_PROBE, "s7a_w15_probe").expect("W15 编译");
    let mut tampered = String::from(artifact.ptx());
    tampered.push_str(
        ".global .align 8 .f64 _ZN2wp11_rogue_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;\n",
    );
    assert!(check_render_ptx(&tampered, "s7a_w15_probe").is_err());
    // 缺少入口名。
    assert!(check_render_ptx(artifact.ptx(), "missing_entry").is_err());
}
