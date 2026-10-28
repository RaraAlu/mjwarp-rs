//! R02-S1 天空盒位级比较实体（窗口实体 `render_skybox_reference`；GPU 用例 `#[ignore]`）。
//!
//! 参考：`fixtures/render-bvh/scene-a/reference-skybox/`，由 stage A 脚本
//! `scripts/authoring/render-bvh-r02/capture_skybox.py` 在 pinned mujoco_warp 3.15 上采集
//! （`render_skybox=true`，两相机 320x240 + 240x180 = 120 000 像素）。
//!
//! 本文件只做「产品路径 vs 参考」的位级比较；判据与 R05 完全一致（RGB ≤1 LSB、depth 位级
//! 优先、seg 全等、两次运行 sha256 相同），不新增/不放宽任何容差、不替换参考。
//!
//! host 用例（默认执行）只校验参考可解析与内部自洽，参考缺失时打印并跳过；
//! GPU 用例默认 ignore，只在 r02-skybox 验证窗内以 `--ignored --test-threads=1` 执行。

#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/shadow.rs"]
mod shadow;
#[path = "support/render_bvh/skybox.rs"]
mod skybox;

use mjwarp_rs::render::kernel::MegakernelRenderer;
use s7b2::session;

/// host 自检：stage A 参考可解析且内部自洽（无 GPU；不比较任何渲染数值）。
#[test]
fn capture_reference_parse_and_cross_checks_hold() {
    if !skybox::reference_exists() {
        println!(
            "r02-s1[host] 天空盒参考缺失（{}），跳过解析：stage A 采集前不产生数值结论",
            skybox::CAPTURE_DIR
        );
        return;
    }
    let reference = skybox::load();
    assert_eq!(reference.cameras(), skybox::CAMERAS, "参考相机数");
    assert_eq!(
        reference.total_pixels,
        skybox::EXPECTED_PIXELS,
        "参考总像素"
    );
    assert_eq!(reference.tex_id, skybox::SKYBOX_TEX_ID, "天空盒纹理 id");
    assert_eq!(
        reference.face_width,
        skybox::SKYBOX_FACE_WIDTH,
        "天空盒 face_width"
    );
    assert_eq!(
        (reference.rgb[0].width, reference.rgb[0].height),
        (320, 240),
        "cam0 分辨率"
    );
    assert_eq!(
        (reference.rgb[1].width, reference.rgb[1].height),
        (240, 180),
        "cam1 分辨率"
    );
    for camera in &reference.depth_seg {
        assert_eq!(
            camera.depth_bits.len(),
            camera.pixels,
            "cam{} depth 位型长度",
            camera.index
        );
        assert_eq!(
            camera.seg_pairs.len(),
            camera.pixels,
            "cam{} seg 对长度",
            camera.index
        );
        assert!(
            !camera.seg_unique.is_empty(),
            "cam{} seg 参考不得为空",
            camera.index
        );
    }
    println!(
        "r02-s1[host] reference parsed: pixels={} captured_at={} tex_id={} face_width={}",
        reference.total_pixels, reference.captured_at, reference.tex_id, reference.face_width
    );
}

/// GPU 实体：scene-a + `render_skybox=true`，两相机 × RGB/depth/seg 对 stage A 参考。
#[test]
#[ignore = "需要 GPU + NVRTC（r02-skybox 验证窗内运行）"]
fn skybox_matches_captured_reference() {
    let session = session();
    let reference = skybox::load();
    assert_eq!(reference.cameras(), skybox::CAMERAS, "参考相机数");
    assert_eq!(
        reference.total_pixels,
        skybox::EXPECTED_PIXELS,
        "参考总像素"
    );

    let scene = skybox::scene(&session, &s7b2::recipe_depth_seg());
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let first = skybox::fire(&session, &renderer, &scene);
    let (rgb_stats, depth_stats, seg_stats) = skybox::compare(&scene, &reference, &first);
    for (camera, stats) in reference.rgb.iter().zip(&rgb_stats) {
        println!(
            "r02-s1 cam{} rgb: compared={} mismatched={} max_byte_diff={} beyond_1_lsb={}",
            camera.index,
            stats.compared,
            stats.mismatched_pixels,
            stats.max_byte_diff,
            stats.beyond_one_lsb_pixels
        );
    }
    for ((camera, depth), seg) in reference.depth_seg.iter().zip(&depth_stats).zip(&seg_stats) {
        println!("r02-s1 {}", skybox::camera_summary(camera, depth, seg));
    }
    skybox::assert_criteria("r02-s1", &reference, &rgb_stats, &depth_stats, &seg_stats);

    let second = skybox::fire(&session, &renderer, &scene);
    assert_eq!(
        skybox::hash_rgb(&first.rgb),
        skybox::hash_rgb(&second.rgb),
        "两次运行 RGB sha256"
    );
    assert_eq!(
        skybox::hash_depth(&first.depth),
        skybox::hash_depth(&second.depth),
        "两次运行 depth sha256"
    );
    assert_eq!(
        skybox::hash_seg(&first.seg),
        skybox::hash_seg(&second.seg),
        "两次运行 seg sha256"
    );
    println!(
        "r02-s1 run sha256: rgb={} depth={} seg={} (captured_at={})",
        skybox::hash_rgb(&first.rgb),
        skybox::hash_depth(&first.depth),
        skybox::hash_seg(&first.seg),
        reference.captured_at
    );
}
