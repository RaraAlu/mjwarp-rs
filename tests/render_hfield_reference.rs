#![cfg(feature = "cuda-probe")]
//! R05-S3b hfield 目标实体（作者 + 窗口内执行；本轮只做离线 authoring + 编译验证）。
//!
//! 参考：`fixtures/render-bvh/scene-hfield/capture-hfield/{hfield-only,hfield-primitive}/`
//! 的 rgb/depth/seg npz + capture.json（R05-S3a 采集窗 attempt=1 产物，已入库）。
//!
//! 判据（沿用 R09-S3b，不新增、不放宽；超判据即停手报 Arch，不改容差/参考/fixture）：
//! - RGB 每通道 **≤1 LSB**（位级优先）；
//! - depth **位级优先**（`mismatched==0`）且 `max_ulp<=4`（`MAX_DEPTH_ULP=4`）、`beyond_ulp==0`；
//! - seg 逐像素 **完全相等**（mismatch=0）；
//! - 同一 context 两次渲染的 rgb/depth/seg sha256 相同。
//!
//! 场景口径：产品模型字段来自 `tests/support/render_bvh/hfield/model-312.json` 与
//! `model-mixed-312.json`（离线由仓库既有 S0a dumper（MuJoCo 3.12 C API）对两个签字 scene XML 生成）；
//! hfield 注入数据取 R05-S3a 证据 `materials/reverse-calc.txt` [1] 的 16 个 raw f32 位型；
//! `SceneLeaf.size` 用 `build_hfield_mesh(...).half()`（S2 O5，不 host 侧重推）。
//!
//! 本文件**零渲染数值结论**：host 用例只做结构/模型/参考解析核对（无 GPU、无判据数字）；
//! 两个 GPU 用例 `#[ignore]`，只在 S3b 验证窗内对真实捕获产物运行并给出判据数字。
//!
//! ```text
//! cargo test --locked --features cuda-probe --test render_hfield_reference
//! cargo test --locked --features cuda-probe --test render_hfield_reference -- --ignored --test-threads=1
//! ```

#[path = "support/render_bvh/hfield.rs"]
mod hfield;
#[path = "support/render_bvh/hfield_scene.rs"]
mod hfield_scene;
#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/shadow.rs"]
mod shadow;

use mjwarp_rs::model::render::{FieldData, RenderFields};
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::resolve_context;
use s7b2::session;

use hfield::HfieldGroup;
use hfield_scene::{HfieldFixture, HfieldScene};

/// host 自检：S3a 证据 `materials/reverse-calc.txt` [1] 的 16 个 raw f32 位型
/// （`reverse-calc.txt` sha256 `3f404550bbc6acd5d3aaf8720dfa2490699e62c1fd865e2c619a816cbadc7ad5`）
/// 经 `build_hfield_mesh` 后必须得到上游三角化形态 **13 顶点 / 36 索引 / 12 三角形**，
/// 且 `half` 与 `size` 口径一致。**这是结构交叉核对，不是渲染验收结论。**
#[test]
fn hfield_fixture_triangulation_matches_s3a_reverse_calc() {
    let fixture = HfieldFixture::new();
    let mesh = fixture.mesh();
    println!(
        "r05-s3b[host] hfield mesh: vertices={} indices={} triangles={} half={:?}",
        mesh.vertices().len(),
        mesh.indices().len(),
        mesh.bvh().triangle_count(),
        mesh.half()
    );
    assert_eq!(mesh.vertices().len(), 13, "顶点数（S3a 反算记录）");
    assert_eq!(mesh.indices().len(), 36, "索引数（S3a 反算记录）");
    assert_eq!(mesh.bvh().triangle_count(), 12, "三角形数（S3a 反算记录）");
    // size=[1.5,1.0,0.25]：x/y 由网格覆盖得 half=1.5/1.0；z 最大值 0.25、最小 0 ⇒ half=0.125。
    assert_eq!(
        mesh.half(),
        [1.5f32, 1.0, 0.125],
        "half 半尺寸（S2 O5 同源值）"
    );
    assert_eq!(
        fixture.inputs().size,
        Some(&[1.5f32, 1.0, 0.25, 0.1][..]),
        "hfield size 口径（sx,sy,sz,elevation 缩放）"
    );
    assert_eq!(fixture.data().len(), 16, "4x4 高程数据长度");
    // 原始位型抽查：首个/最高点为 S3a 记录的 0x3EBA2E8C / 0x3F800000。
    assert_eq!(fixture.data()[0].to_bits(), 0x3eba_2e8c);
    assert_eq!(fixture.data()[3].to_bits(), 0x3f80_0000);
}

/// host 自检：两组 3.12 模型 dump 与签字场景语义一致（相机/灯光/geom/纹理计数与类型）。
#[test]
fn model_dumps_match_signed_scene_semantics() {
    for group in [HfieldGroup::Only, HfieldGroup::Primitive] {
        let text = match group {
            HfieldGroup::Only => hfield_scene::MODEL_ONLY,
            HfieldGroup::Primitive => hfield_scene::MODEL_MIXED,
        };
        let model: serde_json::Value = serde_json::from_str(text).expect("model dump JSON");
        assert_eq!(
            model["mujoco_version"].as_str(),
            Some("3.12.0"),
            "{} dump 版本",
            group.label()
        );
        let fields = hfield_scene::fields(group);
        let geom_type = index_values(&fields, "geom_type");
        let geom_dataid = index_values(&fields, "geom_dataid");
        let geom_size = num_values(&fields, "geom_size");
        assert_eq!(
            geom_type,
            match group {
                HfieldGroup::Only => vec![1],
                HfieldGroup::Primitive => vec![1, 6],
            },
            "{} geom_type（1=hfield，6=box）",
            group.label()
        );
        assert_eq!(
            geom_dataid[0],
            0,
            "{} hfield geom 必须引用 hfield id 0",
            group.label()
        );
        assert_eq!(
            [geom_size[0] as f32, geom_size[1] as f32],
            [1.5, 1.0],
            "{} hfield geom_size[0..2]（mjModel geom 半尺寸 x/y）",
            group.label()
        );
        // MuJoCo 对 hfield geom 派生的 geom_size[2] 不等于 hfield_size[2]（本例 0.1125）；
        // 产品 leaf 的 size 用 build_hfield_mesh().half()（S2 O5），这里的派生值只做非零检查/记录。
        assert!(
            geom_size[2] > 0.0,
            "{} hfield geom_size[2] 应为正的 MuJoCo 派生值（实际 {}）",
            group.label(),
            geom_size[2]
        );
        if group == HfieldGroup::Primitive {
            assert_eq!(geom_type[1], 6, "mixed 组第二个 geom 是 box");
            assert_eq!(geom_dataid[1], -1, "box 不引用 hfield/mesh 数据");
            assert_eq!(
                [
                    geom_size[3] as f32,
                    geom_size[4] as f32,
                    geom_size[5] as f32
                ],
                [0.14, 0.14, 0.14],
                "mixed 组 box 半尺寸"
            );
        }
        assert_eq!(
            index_values(&fields, "cam_mode").len(),
            2,
            "{} 相机数",
            group.label()
        );
        assert_eq!(
            index_values(&fields, "light_type").len(),
            2,
            "{} 灯光数",
            group.label()
        );
        assert_eq!(
            flag_values(&fields, "light_castshadow"),
            vec![1, 1],
            "{} 两灯 castshadow",
            group.label()
        );
        assert_eq!(
            flag_values(&fields, "light_active"),
            vec![1, 1],
            "{} 两灯 active",
            group.label()
        );
        assert_eq!(
            byte_values(&fields, "tex_data").len(),
            64 * 64 * 3,
            "{} 内置 checker 纹理数据（64x64 RGB）",
            group.label()
        );
        let resolved = resolve_context(&fields, &hfield_scene::capture_params())
            .expect("context 解析（host）");
        assert_eq!(resolved.nrender(), 2, "{} 激活相机数", group.label());
        assert_eq!(
            resolved.cam_res(),
            &[[320, 240], [240, 180]],
            "{} 相机分辨率",
            group.label()
        );
        assert!(
            resolved.hfield.is_none(),
            "{} resolve_context 不得隐式注入 hfield（注入走 set_hfield）",
            group.label()
        );
        println!(
            "r05-s3b[host] {} model dump: geoms={} cams={} lights={} tex_data={} B",
            group.label(),
            geom_type.len(),
            index_values(&fields, "cam_mode").len(),
            index_values(&fields, "light_type").len(),
            byte_values(&fields, "tex_data").len()
        );
    }
}

/// host 自检：S3a 捕获参考可解析且内部自洽（无 GPU；不比较任何渲染数值）。
/// 捕获产物缺失时打印并跳过（普通 `cargo test` 不因缺参考 panic）。
#[test]
fn capture_references_parse_and_cross_checks_hold() {
    let mut checked = 0usize;
    for group in [HfieldGroup::Only, HfieldGroup::Primitive] {
        let capture = format!("{}/capture.json", group.capture_dir());
        if !std::path::Path::new(&capture).exists() {
            println!(
                "r05-s3b[host] {} 捕获参考缺失，跳过解析（S3a 采集前不产生数值结论）",
                group.label()
            );
            continue;
        }
        let reference = hfield::load(group);
        assert_eq!(reference.cameras(), 2, "{} 参考相机数", group.label());
        assert_eq!(
            reference.total_pixels,
            120_000,
            "{} 参考总像素",
            group.label()
        );
        assert_eq!(
            reference.captured_at,
            hfield::CAPTURE_AT,
            "{} captured_at",
            group.label()
        );
        assert_eq!(
            (reference.rgb[0].width, reference.rgb[0].height),
            (320, 240),
            "{} cam0 分辨率",
            group.label()
        );
        assert_eq!(
            (reference.rgb[1].width, reference.rgb[1].height),
            (240, 180),
            "{} cam1 分辨率",
            group.label()
        );
        for camera in &reference.depth_seg {
            assert_eq!(
                camera.depth_bits.len(),
                camera.pixels,
                "{} cam{} depth 位型长度",
                group.label(),
                camera.index
            );
            assert_eq!(
                camera.seg_pairs.len(),
                camera.pixels,
                "{} cam{} seg 对长度",
                group.label(),
                camera.index
            );
            assert!(
                !camera.seg_unique.is_empty(),
                "{} cam{} seg 参考不得为空",
                group.label(),
                camera.index
            );
        }
        let zero_pixels: usize = reference
            .depth_seg
            .iter()
            .map(|camera| camera.depth_zero_pixels)
            .sum();
        println!(
            "r05-s3b[host] {} reference parsed: pixels={} depth_zero_pixels={} \
             captured_at={}",
            group.label(),
            reference.total_pixels,
            zero_pixels,
            reference.captured_at
        );
        checked += 1;
    }
    if checked == 0 {
        println!("r05-s3b[host] 无任何已入库捕获参考：跳过（不产生数值结论）");
    }
}

/// GPU 实体：hfield-only 组（cam0/cam1 × rgb/depth/seg 对 S3a 参考）。
#[test]
#[ignore = "需要 GPU + NVRTC（S3b 验证窗内运行）"]
fn hfield_only_matches_captured_reference() {
    run_group(HfieldGroup::Only);
}

/// GPU 实体：hfield-primitive 组（含 1 个 box 遮挡体）。
#[test]
#[ignore = "需要 GPU + NVRTC（S3b 验证窗内运行）"]
fn hfield_primitive_matches_captured_reference() {
    run_group(HfieldGroup::Primitive);
}

fn run_group(group: HfieldGroup) {
    let session = session();
    let reference = hfield::load(group);
    assert_eq!(reference.cameras(), 2, "{} 参考相机数", group.label());
    assert_eq!(
        reference.total_pixels,
        120_000,
        "{} 参考总像素",
        group.label()
    );

    let scene = HfieldScene::new(&session, group);
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let first = hfield_scene::fire(&session, &renderer, &scene);
    let (rgb_stats, depth_stats, seg_stats) =
        hfield_scene::compare_group(&first, &scene, &reference);

    for (camera, stats) in reference.rgb.iter().zip(&rgb_stats) {
        println!(
            "r05-s3b[{}] cam{} rgb: compared={} mismatched={} max_byte_diff={} beyond_1_lsb={}",
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
            "r05-s3b[{}] {}",
            group.label(),
            hfield_scene::camera_summary(camera, depth, seg)
        );
    }
    hfield_scene::assert_group_criteria(
        &format!("r05-s3b[{}]", group.label()),
        &reference,
        &rgb_stats,
        &depth_stats,
        &seg_stats,
    );

    // 两次渲染 sha256 相同（同一 context 重复发射）。
    let second = hfield_scene::fire(&session, &renderer, &scene);
    assert_eq!(
        hfield_scene::hash_rgb(&first.rgb),
        hfield_scene::hash_rgb(&second.rgb),
        "{} 两次运行 RGB sha256",
        group.label()
    );
    assert_eq!(
        hfield_scene::hash_depth(&first.depth),
        hfield_scene::hash_depth(&second.depth),
        "{} 两次运行 depth sha256",
        group.label()
    );
    assert_eq!(
        hfield_scene::hash_seg(&first.seg),
        hfield_scene::hash_seg(&second.seg),
        "{} 两次运行 seg sha256",
        group.label()
    );
    println!(
        "r05-s3b[{}] run sha256: rgb={} depth={} seg={} (captured_at={})",
        group.label(),
        hfield_scene::hash_rgb(&first.rgb),
        hfield_scene::hash_depth(&first.depth),
        hfield_scene::hash_seg(&first.seg),
        reference.captured_at
    );
}

fn index_values(fields: &RenderFields, name: &str) -> Vec<i32> {
    match fields.field(name) {
        Some(FieldData::Index(values)) => values.clone(),
        _ => panic!("渲染字段 {name} 缺失或类型不是 Index"),
    }
}

fn flag_values(fields: &RenderFields, name: &str) -> Vec<u8> {
    match fields.field(name) {
        Some(FieldData::Flag(values)) => values.clone(),
        _ => panic!("渲染字段 {name} 缺失或类型不是 Flag"),
    }
}

fn num_values(fields: &RenderFields, name: &str) -> Vec<f64> {
    match fields.field(name) {
        Some(FieldData::Num(values)) => values.clone(),
        _ => panic!("渲染字段 {name} 缺失或类型不是 Num"),
    }
}

fn byte_values(fields: &RenderFields, name: &str) -> Vec<u8> {
    match fields.field(name) {
        Some(FieldData::Flag(values)) => values.clone(),
        Some(FieldData::Index(values)) => values.iter().map(|value| *value as u8).collect(),
        _ => panic!("渲染字段 {name} 缺失或类型不可读为字节"),
    }
}
