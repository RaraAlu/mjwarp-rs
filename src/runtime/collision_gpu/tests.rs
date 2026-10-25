//! G10@P5：产品 NVRTC 入口的纯 CPU 单测（**不创建 `CudaContext`、不跑 GPU**）。
//!
//! 覆盖四类：ABI 布局（大小/逐字段偏移）、设备源拼装（顺序/去本地 include/指纹）、
//! 输入校验（`InvalidArgument`）、错误注入 seam（编译/设备构造失败都不 panic）。

use std::error::Error;
use std::mem::{align_of, offset_of, size_of};

use cudarc::driver::DeviceRepr;
use cudarc::nvrtc::Ptx;

use super::abi::{CollisionArgs, CollisionContactOut, validate_inputs};
use super::error::CollisionGpuError;
use super::source::{
    DEVICE_SOURCES, PRODUCT_SOURCE, assemble_device_source, is_local_cu_include, source_sha256,
    strip_local_includes,
};
use super::{
    CollisionGpuInputs, CollisionGpuMeta, compile_options, compile_source_with, program_from_parts,
};

macro_rules! assert_offsets {
    ($structure:ty, $(($field:ident, $offset:expr)),* $(,)?) => {
        $(
            assert_eq!(
                offset_of!($structure, $field),
                $offset,
                "{}::{} 偏移不符",
                stringify!($structure),
                stringify!($field)
            );
        )*
    };
}

#[cfg(target_pointer_width = "64")]
#[test]
fn abi_contact_out_matches_contact_cu_layout() {
    assert_eq!(align_of::<CollisionContactOut>(), 8);
    assert_eq!(size_of::<CollisionContactOut>(), 136);
    assert_offsets!(
        CollisionContactOut,
        (dist, 0),
        (pos, 8),
        (frame, 16),
        (includemargin, 24),
        (friction, 32),
        (solref, 40),
        (solreffriction, 48),
        (solimp, 56),
        (dim, 64),
        (geom, 72),
        (efc_address, 80),
        (worldid, 88),
        (contact_type, 96),
        (geomcollisionid, 104),
        (adhesion, 112),
        (nacon, 120),
        (efc_address_width, 128),
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn abi_collision_args_matches_probe_args_layout() {
    fn assert_repr<T: DeviceRepr>() {}
    assert_repr::<CollisionArgs>();
    assert_eq!(align_of::<CollisionArgs>(), 8);
    assert_eq!(size_of::<CollisionArgs>(), 496);
    assert_offsets!(
        CollisionArgs,
        (geom_xpos, 0),
        (geom_xmat, 8),
        (geom_size, 16),
        (geom_margin, 24),
        (geom_gap, 32),
        (geom_rbound, 40),
        (geom_aabb, 48),
        (geom_type, 56),
        (geom_dataid, 64),
        (mesh_vertadr, 72),
        (mesh_vertnum, 80),
        (mesh_graphadr, 88),
        (mesh_octadr, 96),
        (mesh_vert, 104),
        (mesh_graph, 112),
        (hfield_size, 120),
        (hfield_nrow, 128),
        (hfield_ncol, 136),
        (hfield_adr, 144),
        (hfield_data, 152),
        (oct_aabb, 160),
        (oct_child, 168),
        (oct_coeff, 176),
        (nworld, 184),
        (ngeom, 188),
        (g1, 192),
        (g2, 196),
        (type1, 200),
        (type2, 204),
        (geomgeomid, 208),
        (naccdmax, 212),
        (hfield_id, 216),
        (oct_root, 220),
        (noct, 224),
        (tolerance, 228),
        (margin, 232),
        (gap, 236),
        (gjk_iterations, 240),
        (epa_iterations, 244),
        (condim, 248),
        (pairid0, 252),
        (pairid1, 256),
        (friction, 260),
        (solref, 280),
        (solreffriction, 288),
        (solimp, 296),
        (adhesion, 316),
        (sdf_initpoints, 320),
        (sdf_iterations, 324),
        (naconmax, 328),
        (efc_width, 332),
        (out, 336),
        (nacon, 472),
        (nccd, 480),
        (overflow, 488),
    );
    assert_eq!(
        offset_of!(CollisionArgs, out) + offset_of!(CollisionContactOut, dist),
        336
    );
    assert_eq!(
        offset_of!(CollisionArgs, out) + offset_of!(CollisionContactOut, efc_address),
        416
    );
}

#[test]
fn assembles_all_device_sources_in_dependency_order() {
    let assembled = assemble_device_source();
    let mut previous = 0usize;
    for (name, text) in DEVICE_SOURCES {
        let stripped = strip_local_includes(text);
        let position = assembled
            .find(&stripped)
            .unwrap_or_else(|| panic!("设备源 {name} 未出现在拼装结果中"));
        assert!(position >= previous, "设备源 {name} 的拼接顺序不符依赖顺序");
        previous = position;
    }
    assert!(
        assembled.ends_with(PRODUCT_SOURCE),
        "product.cu 必须追加在合并源最后"
    );
}

#[test]
fn strips_only_local_cu_includes() {
    assert!(is_local_cu_include("#include \"hfield.cu\"\n"));
    assert!(is_local_cu_include("#include\"common.cu\""));
    assert!(!is_local_cu_include("#include <cuda_runtime.h>\n"));
    assert!(!is_local_cu_include("#include \"some_header.h\"\n"));
    assert!(!is_local_cu_include("// #include \"contact.cu\"\n"));
    for (name, text) in DEVICE_SOURCES {
        let stripped = strip_local_includes(text);
        assert!(
            !stripped
                .lines()
                .any(|line| line.contains("#include \"") && line.contains(".cu\"")),
            "{name} 仍残留本地 .cu include"
        );
        assert_eq!(
            stripped.contains("#pragma once"),
            text.contains("#pragma once"),
            "{name} 的 #pragma once 不应被删除"
        );
    }
    let assembled = assemble_device_source();
    assert!(
        !assembled
            .lines()
            .any(|line| line.contains("#include \"") && line.contains(".cu\"")),
        "拼装结果仍残留本地 .cu include"
    );
}

#[test]
fn assembly_is_deterministic_and_hashed() {
    let first = assemble_device_source();
    let second = assemble_device_source();
    assert_eq!(first, second, "拼装结果必须确定");
    let sha = source_sha256();
    assert_eq!(sha, source_sha256(), "sha256 必须确定");
    assert_eq!(sha.len(), 64, "sha256 须为 64 位小写十六进制");
    assert!(sha.bytes().all(|byte| byte.is_ascii_hexdigit()));
    println!(
        "device_source_len={} device_source_sha256={sha}",
        first.len()
    );
}

#[test]
fn product_source_exposes_three_extern_c_entries() {
    let product = PRODUCT_SOURCE;
    for entry in [super::CONTACT_ENTRY, super::HFIELD_ENTRY, super::SDF_ENTRY] {
        let declaration = format!("extern \"C\" __global__ void {entry}(");
        assert_eq!(
            product.matches(&declaration).count(),
            1,
            "{entry} 须有且仅有一处 extern \"C\" 定义"
        );
    }
    assert_eq!(super::CONTACT_ENTRY, "mjwarp_collision_contact");
    assert_eq!(super::HFIELD_ENTRY, "mjwarp_collision_hfield");
    assert_eq!(super::SDF_ENTRY, "mjwarp_collision_sdf");
    for host_symbol in ["int main(", "load_bin", "CaseMeta", "DeviceArrays"] {
        assert!(
            !product.contains(host_symbol),
            "product.cu 不应含探针 host 代码：{host_symbol}"
        );
    }
    assert_eq!(product.matches("struct mjwarp_collision_args").count(), 1);
    assert_eq!(product.matches("struct ProbeArgs").count(), 0);
    for old_entry in ["kernel_contact", "kernel_hfield", "kernel_sdf"] {
        assert!(
            !product.contains(&format!("void {old_entry}(")),
            "product.cu 不应保留探针旧入口：{old_entry}"
        );
    }
}

fn valid_meta() -> CollisionGpuMeta {
    CollisionGpuMeta {
        nworld: 2,
        ngeom: 2,
        g1: 0,
        g2: 1,
        type1: 2,
        type2: 6,
        geomgeomid: 0,
        naccdmax: 8192,
        hfield_id: -1,
        oct_root: -1,
        noct: 0,
        tolerance: 1.0e-6,
        margin: 0.0,
        gap: 0.0,
        gjk_iterations: 35,
        epa_iterations: 35,
        condim: 3,
        pairid0: -1,
        pairid1: -1,
        friction: [1.0, 1.0, 0.005, 1.0e-4, 1.0e-4],
        solref: [0.02, 1.0],
        solreffriction: [0.0, 0.0],
        solimp: [0.9, 0.95, 0.001, 0.5, 2.0],
        adhesion: 0.0,
        sdf_initpoints: 40,
        sdf_iterations: 10,
        naconmax: 8192,
        efc_width: 4,
    }
}

/// 与 `valid_meta` 匹配的宿主输入（geom 数组按 `nworld*ngeom` 铺满，模型静态数组为空）。
struct SampleInputs {
    geom_xpos: Vec<f32>,
    geom_xmat: Vec<f32>,
    geom_size: Vec<f32>,
    geom_margin: Vec<f32>,
    geom_gap: Vec<f32>,
    geom_rbound: Vec<f32>,
    geom_aabb: Vec<f32>,
    geom_type: Vec<i32>,
    geom_dataid: Vec<i32>,
    empty_f32: Vec<f32>,
    empty_i32: Vec<i32>,
    meta: CollisionGpuMeta,
}

impl SampleInputs {
    fn new(meta: CollisionGpuMeta) -> Self {
        let nworld = meta.nworld as usize;
        let ngeom = meta.ngeom as usize;
        let mut geom_xmat = vec![0.0_f32; nworld * ngeom * 9];
        for rotation in geom_xmat.as_chunks_mut::<9>().0 {
            rotation[0] = 1.0;
            rotation[4] = 1.0;
            rotation[8] = 1.0;
        }
        Self {
            geom_xpos: vec![0.0; nworld * ngeom * 3],
            geom_xmat,
            geom_size: vec![0.1; ngeom * 3],
            geom_margin: vec![0.0; ngeom],
            geom_gap: vec![0.0; ngeom],
            geom_rbound: vec![0.1; ngeom],
            geom_aabb: vec![0.0; ngeom * 6],
            geom_type: vec![2; ngeom],
            geom_dataid: vec![-1; ngeom],
            empty_f32: Vec::new(),
            empty_i32: Vec::new(),
            meta,
        }
    }

    fn inputs(&self) -> CollisionGpuInputs<'_> {
        CollisionGpuInputs {
            geom_xpos: &self.geom_xpos,
            geom_xmat: &self.geom_xmat,
            geom_size: &self.geom_size,
            geom_margin: &self.geom_margin,
            geom_gap: &self.geom_gap,
            geom_rbound: &self.geom_rbound,
            geom_aabb: &self.geom_aabb,
            geom_type: &self.geom_type,
            geom_dataid: &self.geom_dataid,
            mesh_vertadr: &self.empty_i32,
            mesh_vertnum: &self.empty_i32,
            mesh_graphadr: &self.empty_i32,
            mesh_octadr: &self.empty_i32,
            mesh_vert: &self.empty_f32,
            mesh_graph: &self.empty_i32,
            hfield_size: &self.empty_f32,
            hfield_nrow: &self.empty_i32,
            hfield_ncol: &self.empty_i32,
            hfield_adr: &self.empty_i32,
            hfield_data: &self.empty_f32,
            oct_aabb: &self.empty_f32,
            oct_child: &self.empty_i32,
            oct_coeff: &self.empty_f32,
            meta: self.meta,
        }
    }
}

#[test]
fn rejects_invalid_meta_but_accepts_matching_inputs() {
    let sample = SampleInputs::new(valid_meta());
    assert_eq!(validate_inputs(&sample.inputs()), Ok(()));
    let mut sample = sample;
    for (field, message) in [
        ("nworld", "nworld 必须为正"),
        ("ngeom", "ngeom 必须为正"),
        ("naconmax", "naconmax 必须为正"),
        ("efc_width", "efc_width 必须为正"),
        ("g1", "g1 超出 geom 范围"),
        ("g2", "g2 超出 geom 范围"),
    ] {
        let mut meta = valid_meta();
        match field {
            "nworld" => meta.nworld = 0,
            "ngeom" => meta.ngeom = 0,
            "naconmax" => meta.naconmax = 0,
            "efc_width" => meta.efc_width = 0,
            "g1" => meta.g1 = 2,
            _ => meta.g2 = -1,
        }
        sample.meta = meta;
        assert_eq!(
            validate_inputs(&sample.inputs()),
            Err(CollisionGpuError::InvalidArgument(message)),
            "{field} 非法值未被拒绝"
        );
    }
}

#[test]
fn rejects_mismatched_slice_lengths() {
    for (name, message) in [
        ("geom_xpos", "geom_xpos 长度须为 nworld*ngeom*3"),
        ("geom_xmat", "geom_xmat 长度须为 nworld*ngeom*9"),
        ("geom_size", "geom_size 长度须为 ngeom*3"),
        ("geom_margin", "geom_margin 长度须为 ngeom"),
        ("geom_gap", "geom_gap 长度须为 ngeom"),
        ("geom_rbound", "geom_rbound 长度须为 ngeom"),
        ("geom_aabb", "geom_aabb 长度须为 ngeom*6"),
        ("geom_type", "geom_type 长度须为 ngeom"),
        ("geom_dataid", "geom_dataid 长度须为 ngeom"),
    ] {
        let mut broken = SampleInputs::new(valid_meta());
        match name {
            "geom_xpos" => {
                broken.geom_xpos.pop();
            }
            "geom_xmat" => {
                broken.geom_xmat.pop();
            }
            "geom_size" => {
                broken.geom_size.pop();
            }
            "geom_margin" => {
                broken.geom_margin.pop();
            }
            "geom_gap" => {
                broken.geom_gap.pop();
            }
            "geom_rbound" => {
                broken.geom_rbound.pop();
            }
            "geom_aabb" => {
                broken.geom_aabb.pop();
            }
            "geom_type" => {
                broken.geom_type.pop();
            }
            _ => {
                broken.geom_dataid.pop();
            }
        }
        assert_eq!(
            validate_inputs(&broken.inputs()),
            Err(CollisionGpuError::InvalidArgument(message)),
            "{name} 长度不匹配未被拒绝"
        );
    }
}

#[test]
fn error_display_covers_every_variant() {
    let cases = [
        (
            CollisionGpuError::NoDevice("d".to_owned()),
            "CUDA设备不可用：d",
        ),
        (
            CollisionGpuError::Compile("c".to_owned()),
            "NVRTC编译失败：c",
        ),
        (
            CollisionGpuError::Load("l".to_owned()),
            "设备模块加载失败：l",
        ),
        (CollisionGpuError::Launch("r".to_owned()), "内核执行失败：r"),
        (
            CollisionGpuError::InvalidArgument("参数"),
            "输入参数无效：参数",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        let dynamic: Box<dyn Error> = Box::new(error);
        assert!(!dynamic.to_string().is_empty());
    }
}

#[test]
fn compile_options_match_g5_probe_recipe() {
    let options = compile_options();
    assert_eq!(options.fmad, Some(true));
    assert_eq!(options.use_fast_math, Some(false));
    assert_eq!(
        options.options,
        vec![
            "--std=c++17".to_owned(),
            "--gpu-architecture=sm_89".to_owned()
        ]
    );
    assert!(options.include_paths.is_empty());
    assert_eq!(options.name.as_deref(), Some("mjwarp_collision_product.cu"));
}

#[test]
fn compile_failures_are_mapped_without_panicking() {
    let result = compile_source_with(|text, options| {
        assert!(text.contains("mjwarp_collision_contact"));
        assert_eq!(options.fmad, Some(true));
        Err("nvrtc: boom".to_owned())
    });
    assert_eq!(
        result.err(),
        Some(CollisionGpuError::Compile("nvrtc: boom".to_owned()))
    );

    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let panicked = compile_source_with(|_, _| -> Result<Ptx, String> { panic!("nvrtc ffi") });
    std::panic::set_hook(previous);
    assert_eq!(
        panicked.err(),
        Some(CollisionGpuError::Compile("NVRTC编译异常".to_owned()))
    );
}

#[test]
fn constructor_seam_maps_device_and_compile_errors() {
    let no_device = program_from_parts(Err("no cuda driver".into()), Ok(Ptx::from_src("// ptx")));
    assert_eq!(
        no_device.err(),
        Some(CollisionGpuError::NoDevice("no cuda driver".to_owned()))
    );
    let compile_first = program_from_parts(
        Err("unused device error".to_owned()),
        Err(CollisionGpuError::Compile("boom".to_owned())),
    );
    assert_eq!(
        compile_first.err(),
        Some(CollisionGpuError::Compile("boom".to_owned()))
    );
}
