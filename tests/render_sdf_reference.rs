#![cfg(feature = "cuda-probe")]
//! U4 SDF 验证窗目标实体（作者侧 + 窗口内执行；本片离线写代码与编译验证）。
//!
//! 参考：`fixtures/render-bvh/scene-sdf/capture-sdf/{sdf-mesh,sdf-occluder}/`
//! 的 rgb/depth/seg npz + capture.json（U3 采集窗 attempt=1 产物，已入库）。
//!
//! 判据（沿用 R09-S3b / R05-S3b，不新增、不放宽；超判据即停手报 Arch，不改容差/参考/fixture）：
//! - RGB 每通道 **≤ 1 LSB**（`beyond_1_lsb=0`、`max_byte_diff<=1`）；
//! - depth **位级优先**（`mismatched==0`）且 `max_ulp<=4`、`beyond_ulp==0`；
//! - seg 逐像素**完全相等**（`mismatched=0`）；
//! - 同一 context 两次渲染的 rgb/depth/seg sha256 相同。
//!
//! 场景口径：SDF（gtype==8）由上游当底层 mesh 渲染 ⇒ host 场景把 mesh 0 交给
//! `RenderContext`，SDF leaf 的 `size` 用 **mesh bounds** 半尺寸（不是 `geom_size`）。
//!
//! 额外两条**一致性**臂（没有上游 shadow-off 参考，不宣称数值参考）：
//! - shadow 开/关：depth/seg 位级相等；rgb `off >= on`（逆序超 1 LSB 的通道=0）且差异像素>0；
//! - DepthSeg 跨路径：`render_depth_seg` 与 megakernel 的 depth/seg 位级相等。
//!
//! 前提（B 类，CR-10/11/12/16）：上述 `<=4 ULP`、位级 depth、RGB `<=1 LSB` 与 DepthSeg 跨路径断言均为
//! 参考-前提型判据：仅在同 recipe、同编译形态、同 GPU+驱动、同 warp 版本下成立；前提不成立即不适用，
//! 不得单独作为通过依据（详见 `docs/development-evidence/criteria-purpose-audit/b-class-premises.md`）。
//!
//! ```text
//! cargo test --locked --features cuda-probe --test render_sdf_reference
//! cargo test --locked --features cuda-probe --test render_sdf_reference -- --ignored --test-threads=1
//! ```

#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/sdf.rs"]
mod sdf;
#[path = "support/render_bvh/sdf_compare.rs"]
mod sdf_compare;
#[path = "support/render_bvh/sdf_scene.rs"]
mod sdf_scene;
#[path = "support/render_bvh/shadow.rs"]
mod shadow;

use mjwarp_rs::render::bvh::{GeomKind, LeafGeometry, mesh_half_extents};
use mjwarp_rs::render::kernel::{DepthSegRenderer, MegakernelRenderer};
use s7b2::session;
use serde_json::Value;

use sdf::{SdfGroup, SdfReference};
use sdf_scene::{SdfScene, read_depth, read_seg};

/// U4a 两份模型 dump 的签字 sha256（`sdf-model-dump/receipt.json`）。
const MODEL_MESH_SHA256: &str = "7de15ff458aa8f6b1c026bbed5d9b1fbaf8f5dd39b385bab0b79d5c289b40d36";
const MODEL_MIXED_SHA256: &str = "60698f6baaeff8dcc11e725e56242bd5d6f947741cb7070b4e1f32ec13765578";

/// host 自检：两份 3.12 模型 dump 的 sha/字节与 U4a 收口值一致，且字段语义与签字场景一致。
#[test]
fn model_dumps_match_signed_scene_semantics() {
    assert_eq!(
        sdf_scene::sha256_hex(sdf_scene::MODEL_MESH.as_bytes()),
        MODEL_MESH_SHA256,
        "model-312.json 与 U4a 收口 sha 不符"
    );
    assert_eq!(
        sdf_scene::sha256_hex(sdf_scene::MODEL_MIXED.as_bytes()),
        MODEL_MIXED_SHA256,
        "model-mixed-312.json 与 U4a 收口 sha 不符"
    );
    let cases: [(&str, usize, &[i32], &[i32]); 2] = [
        (sdf_scene::MODEL_MESH, 3usize, &[0, 8, 2], &[-1, 0, -1]),
        (
            sdf_scene::MODEL_MIXED,
            4usize,
            &[0, 8, 2, 6],
            &[-1, 0, -1, -1],
        ),
    ];
    for (text, ngeom, geom_type, geom_dataid) in cases {
        let model: Value = serde_json::from_str(text).expect("model dump JSON");
        assert_eq!(
            model["mujoco_version"].as_str(),
            Some("3.12.0"),
            "产品 MuJoCo"
        );
        assert_eq!(model["loaded_mjb"].as_bool(), Some(true), "loaded_mjb");
        let counts = &model["counts"];
        assert_eq!(counts["ngeom"].as_u64(), Some(ngeom as u64), "ngeom");
        assert_eq!(counts["nmesh"].as_u64(), Some(1), "nmesh");
        assert_eq!(counts["nmeshvert"].as_u64(), Some(8), "nmeshvert");
        assert_eq!(counts["nmeshface"].as_u64(), Some(12), "nmeshface");
        assert_eq!(counts["ncam"].as_u64(), Some(2), "ncam");
        assert_eq!(counts["nlight"].as_u64(), Some(2), "nlight");
        assert_eq!(counts["nq"].as_u64(), Some(0), "nq");
        let arrays = &model["arrays"];
        let kinds = ints(&arrays["geom_type"]["values"]);
        assert_eq!(kinds, geom_type, "geom_type（含 SDF=8）");
        let dataids = ints(&arrays["geom_dataid"]["values"]);
        assert_eq!(dataids, geom_dataid, "geom_dataid（SDF 指向 mesh 0）");
        assert!(kinds.contains(&8), "模型必须含 SDF geom（gtype==8）");
    }
}

/// host 自检：两组捕获参考可解析、sha 与 capture.json 一致、相机/像素数一致。
#[test]
fn capture_references_parse_and_cross_checks_hold() {
    for group in [SdfGroup::Mesh, SdfGroup::Occluder] {
        let reference = sdf::load(group);
        assert_eq!(reference.cameras(), 2, "{} 相机数", group.label());
        assert_eq!(reference.total_pixels, 120_000, "{} 总像素", group.label());
        assert_eq!(
            reference.scene_sha256(),
            group.scene_sha256(),
            "{} 场景 sha",
            group.label()
        );
        assert_eq!(reference.rgb[0].pixels, 76_800, "cam-front 像素");
        assert_eq!(reference.depth_seg[1].pixels, 43_200, "cam-top 像素");
        assert!(!reference.depth_seg[0].depth_bits.is_empty(), "depth 位型");
        assert!(!reference.depth_seg[0].seg_pairs.is_empty(), "seg 对");
        // 参考里 SDF 组必须能命中几何（不是全零 depth），否则窗口比对无意义。
        let nonzero = reference.depth_seg[0]
            .depth_bits
            .iter()
            .filter(|bits| **bits != 0)
            .count();
        assert!(
            nonzero > 0,
            "{} cam-front depth 全零，参考异常",
            group.label()
        );
    }
}

/// host 自检：SDF leaf 必须用 mesh bounds（不是 geom_size），且 mesh 数据形状固定。
#[test]
fn sdf_leaf_geometry_uses_mesh_bounds() {
    for group in [SdfGroup::Mesh, SdfGroup::Occluder] {
        let mesh = sdf_scene::mesh_state(group);
        assert_eq!(mesh.vertices.len(), 8, "{} mesh 顶点", group.label());
        assert_eq!(mesh.indices.len(), 12 * 3, "{} mesh 索引", group.label());
        let half = mesh_half_extents(&mesh.vertices).expect("mesh half");
        assert_eq!(
            half,
            [0.35f32, 0.35, 0.35],
            "{} mesh bounds 半尺寸",
            group.label()
        );
        let geometry = LeafGeometry::from_geom_type(8, half).expect("SDF leaf 几何");
        assert_eq!(geometry.kind, GeomKind::Mesh, "SDF 必须映射为 Mesh leaf");
        assert_eq!(geometry.size, half, "SDF leaf size 必须是 mesh bounds");
    }
}

/// GPU：sdf-mesh 组逐相机参考比对 + 两跑确定性。
#[test]
#[ignore = "需要 GPU + NVRTC（U4 验证窗内运行）"]
fn sdf_mesh_matches_captured_reference() {
    run_group(SdfGroup::Mesh);
}

/// GPU：sdf-occluder 组逐相机参考比对 + 两跑确定性。
#[test]
#[ignore = "需要 GPU + NVRTC（U4 验证窗内运行）"]
fn sdf_occluder_matches_captured_reference() {
    run_group(SdfGroup::Occluder);
}

/// GPU：shadow 开/关一致性（cam-front；无上游 shadow-off 参考，只做一致性）。
#[test]
#[ignore = "需要 GPU + NVRTC（U4 验证窗内运行）"]
fn shadow_toggle_consistency() {
    let session = session();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let on = SdfScene::new(&session, SdfGroup::Mesh, true);
    let off = SdfScene::new(&session, SdfGroup::Mesh, false);
    let rendered_on = sdf_scene::fire(&renderer, &on);
    let rendered_off = sdf_scene::fire(&renderer, &off);
    let (changed, reversed) = sdf_compare::assert_shadow_consistency(
        "u4-sdf[shadow]",
        &rendered_on,
        &rendered_off,
        &on,
        0,
    );
    println!(
        "u4-sdf[shadow] cam0 changed_pixels={changed} reversed_channels={reversed} \
         depth_seg_bitwise_equal=true"
    );
    // 两臂各自两跑确定性。
    let again_on = sdf_scene::fire(&renderer, &on);
    assert_eq!(
        sdf_compare::hash_rgb(&rendered_on.rgb),
        sdf_compare::hash_rgb(&again_on.rgb),
        "shadow-on 两跑 RGB sha"
    );
    assert_eq!(
        sdf_compare::hash_depth(&rendered_on.depth),
        sdf_compare::hash_depth(&again_on.depth),
        "shadow-on 两跑 depth sha"
    );
    assert_eq!(
        sdf_compare::hash_seg(&rendered_on.seg),
        sdf_compare::hash_seg(&again_on.seg),
        "shadow-on 两跑 seg sha"
    );
    let again_off = sdf_scene::fire(&renderer, &off);
    assert_eq!(
        sdf_compare::hash_rgb(&rendered_off.rgb),
        sdf_compare::hash_rgb(&again_off.rgb),
        "shadow-off 两跑 RGB sha"
    );
    assert_eq!(
        sdf_compare::hash_depth(&rendered_off.depth),
        sdf_compare::hash_depth(&again_off.depth),
        "shadow-off 两跑 depth sha"
    );
    assert_eq!(
        sdf_compare::hash_seg(&rendered_off.seg),
        sdf_compare::hash_seg(&again_off.seg),
        "shadow-off 两跑 seg sha"
    );
}

/// GPU：DepthSeg 跨路径一致性（megakernel vs `render_depth_seg`，cam 全量位级）。
#[test]
#[ignore = "需要 GPU + NVRTC（U4 验证窗内运行）"]
fn depth_seg_cross_path_consistency() {
    let session = session();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let scene = SdfScene::new(&session, SdfGroup::Mesh, true);
    let megakernel = sdf_scene::fire(&renderer, &scene);
    let depth_seg = DepthSegRenderer::new(&session).expect("render_depth_seg 编译");
    depth_seg
        .render(
            &scene.context,
            &scene.fields,
            &scene.model,
            &scene.poses,
            Some(&scene.rays),
            true,
        )
        .expect("DepthSeg 发射");
    let depth_seg_rendered = sdf_scene::Rendered {
        rgb: Vec::new(),
        depth: read_depth(&scene.context),
        seg: read_seg(&scene.context),
    };
    sdf_compare::assert_depth_seg_cross_path(
        "u4-sdf[depthseg]",
        &megakernel,
        &depth_seg_rendered,
        &scene,
    );
    println!("u4-sdf[depthseg] cam0..N depth/seg bitwise equal to megakernel");
}

/// GPU：同一 context 两跑确定性（rgb/depth/seg sha256 相等）。
#[test]
#[ignore = "需要 GPU + NVRTC（U4 验证窗内运行）"]
fn two_run_determinism() {
    let session = session();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let scene = SdfScene::new(&session, SdfGroup::Mesh, true);
    let first = sdf_scene::fire(&renderer, &scene);
    let second = sdf_scene::fire(&renderer, &scene);
    assert_eq!(
        sdf_compare::hash_rgb(&first.rgb),
        sdf_compare::hash_rgb(&second.rgb),
        "两跑 RGB sha256"
    );
    assert_eq!(
        sdf_compare::hash_depth(&first.depth),
        sdf_compare::hash_depth(&second.depth),
        "两跑 depth sha256"
    );
    assert_eq!(
        sdf_compare::hash_seg(&first.seg),
        sdf_compare::hash_seg(&second.seg),
        "两跑 seg sha256"
    );
    println!(
        "u4-sdf[determinism] run sha256: rgb={} depth={} seg={}",
        sdf_compare::hash_rgb(&first.rgb),
        sdf_compare::hash_depth(&first.depth),
        sdf_compare::hash_seg(&first.seg)
    );
}

fn run_group(group: SdfGroup) {
    let session = session();
    let reference = sdf::load(group);
    assert_eq!(reference.cameras(), 2, "{} 参考相机数", group.label());
    assert_eq!(
        reference.total_pixels,
        120_000,
        "{} 参考总像素",
        group.label()
    );

    let scene = SdfScene::new(&session, group, true);
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let first = sdf_scene::fire(&renderer, &scene);
    let (rgb_stats, depth_stats, seg_stats) =
        sdf_compare::compare_group(&first, &scene, &reference);
    // 只读诊断（QA 前置）：只打印 depth mismatch 明细，不参与断言/判定。
    sdf_compare::report_depth_mismatches(group.label(), &first, &scene, &reference, 40);

    for (camera, stats) in reference.rgb.iter().zip(&rgb_stats) {
        println!(
            "u4-sdf[{}] cam{} rgb: compared={} mismatched={} max_byte_diff={} beyond_1_lsb={}",
            group.label(),
            camera.index,
            stats.compared,
            stats.mismatched_pixels,
            stats.max_byte_diff,
            stats.beyond_one_lsb_pixels
        );
    }
    for ((camera, depth), seg) in reference.depth_seg.iter().zip(&depth_stats).zip(&seg_stats) {
        println!(
            "u4-sdf[{}] {}",
            group.label(),
            sdf_compare::camera_summary(camera, depth, seg)
        );
    }
    sdf_compare::assert_group_criteria(
        &format!("u4-sdf[{}]", group.label()),
        &reference,
        &rgb_stats,
        &depth_stats,
        &seg_stats,
    );

    // 同一 context 两次渲染 sha256 相同。
    let second = sdf_scene::fire(&renderer, &scene);
    assert_eq!(
        sdf_compare::hash_rgb(&first.rgb),
        sdf_compare::hash_rgb(&second.rgb),
        "{} 两次运行 RGB sha256",
        group.label()
    );
    assert_eq!(
        sdf_compare::hash_depth(&first.depth),
        sdf_compare::hash_depth(&second.depth),
        "{} 两次运行 depth sha256",
        group.label()
    );
    assert_eq!(
        sdf_compare::hash_seg(&first.seg),
        sdf_compare::hash_seg(&second.seg),
        "{} 两次运行 seg sha256",
        group.label()
    );
    println!(
        "u4-sdf[{}] run sha256: rgb={} depth={} seg={} (captured_at={})",
        group.label(),
        sdf_compare::hash_rgb(&first.rgb),
        sdf_compare::hash_depth(&first.depth),
        sdf_compare::hash_seg(&first.seg),
        reference.captured_at
    );
}

fn ints(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .expect("数组")
        .iter()
        .map(|item| item.as_i64().expect("i32") as i32)
        .collect()
}

#[allow(dead_code)]
fn _reference_type_anchor(_reference: SdfReference) {}
