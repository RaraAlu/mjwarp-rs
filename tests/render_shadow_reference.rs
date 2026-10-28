#![cfg(feature = "cuda-probe")]
//! R09-S3b 阴影数值验收（GPU 窗口内执行；本文件只做作者 + 编译验证，不开窗）。
//!
//! 判据（Arch 2026-10-28 S3b start）：
//! - **shadow-on**：产品 `nworld=1` + scene-shadow + `use_shadows=true` 的 rgb/depth/seg 对
//!   `fixtures/render-bvh/scene-shadow/capture-exact/shadow-on/` 判定：RGB 每通道 ≤1 LSB、
//!   depth ≤4 ULP（位级 `mismatched==0` 优先）、seg 全等；**两次运行 sha256 相同**。
//! - **shadow-off**：① 对 `capture-exact/shadow-off/` 同判据；② **逐位回归解锁前既有同配方产品参考**：
//!   depth/seg 对 `s7b1::RECIPE_REFERENCE`（同配方 `depth_seg_fast_math_false.json`）位级
//!   `mismatched==0`、RGB 对 `s7b2::parse_rgb_fast_math_false_reference()` 逐字节
//!   `max_byte_diff==0`（S8 解锁前产品实测口径）；做不到逐位即算失败（不改判据）。
//! - **阴影可见性二值对照**（只作语义交叉校验）：产品 shadow-on vs shadow-off 差异像素数，
//!   与参考层同口径计数并列记录；**不作等价/精度结论**。
//!
//! 场景口径：`scene-shadow/scene.xml` 派生自 scene-a，几何/相机/材质相同，唯一语义差异是
//! 三灯 `castshadow=1`（scene-a 的 lamp 为 0）→ 产品字段复用 `s7b2::fields_from_model()`，
//! 只覆盖 `light_castshadow`（见 `tests/support/render_bvh/shadow_scene.rs`）。
//!
//! 参考配方：`capture-exact/` 是**产品同配方**（`use_fast_math=false`，R09-S3a-capture-exact
//! 窗口）产物 → shadow-on/off 均按既有判据作硬断言；旧 `capture/`（fast-math 组）**不参与
//! 本测试断言**，只在证据中作历史/跨配方诊断记录。同配方的解锁前逐位回归仍由 shadow-off ②
//! 承担（既有 `s7b/*_fast_math_false.json`）。
//!
//! 全部 GPU 用例 `#[ignore]`：
//! cargo test --locked --features cuda-probe --test render_shadow_reference -- --ignored --test-threads=1

#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/shadow.rs"]
mod shadow;
#[path = "support/render_bvh/shadow_scene.rs"]
mod shadow_scene;

use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::output::{MAX_DEPTH_ULP, diff_depth, diff_seg};
use s7b2::{diff_rgb, recipe_depth_seg, session};
use shadow::{shadow_off_reference, shadow_on_reference};
use shadow_scene::{
    CAPTURE_AT, assert_group_criteria, build_scene, compare_group, fire, hash_depth, hash_rgb,
    hash_seg, product_depth_difference, product_rgb_difference, product_seg_difference,
    shadow_fields,
};

/// 主机：参考 npz 解析（冻结 sha 内部断言）+ 形状 + 参考层交叉校验；不需要 GPU。
#[test]
fn shadow_references_parse_and_cross_checks_hold() {
    let on = shadow_on_reference();
    let off = shadow_off_reference();
    for reference in [&on, &off] {
        assert_eq!(reference.rgb.len(), 2, "{} RGB 相机数", reference.group);
        assert_eq!(
            reference.depth_seg.len(),
            2,
            "{} depth/seg 相机数",
            reference.group
        );
        assert_eq!(
            reference.total_pixels, 120_000,
            "{} 总像素",
            reference.group
        );
        assert_eq!(
            (reference.rgb[0].width, reference.rgb[0].height),
            (320, 240),
            "{} cam0",
            reference.group
        );
        assert_eq!(
            (reference.rgb[1].width, reference.rgb[1].height),
            (240, 180),
            "{} cam1",
            reference.group
        );
        assert_eq!(reference.depth_seg[0].depth_bits.len(), 76_800);
        assert_eq!(reference.depth_seg[1].depth_bits.len(), 43_200);
        assert_eq!(reference.depth_seg[0].seg_pairs.len(), 76_800);
        assert_eq!(reference.depth_seg[1].seg_pairs.len(), 43_200);
    }
    // S3a 记录的管线交叉校验信号：两组 depth/seg 参考逐字节相同；RGB 因阴影必须不同。
    let reference_rgb = shadow::rgb_difference_pixels(&on, &off);
    assert_eq!(
        shadow::depth_difference_pixels(&on, &off),
        0,
        "depth 参考差异"
    );
    assert_eq!(shadow::seg_difference_pixels(&on, &off), 0, "seg 参考差异");
    assert!(reference_rgb > 0, "参考层 RGB 必须因阴影不同");
    println!(
        "r09-s3b[host] references frozen: pixels=120000 reference_rgb_diff={reference_rgb} \
         reference_depth_diff=0 reference_seg_diff=0 (capture_at={CAPTURE_AT})"
    );
}

/// shadow-on：产品对 S3a 捕获参考的三通道判据 + 两次运行 sha256。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn shadow_on_matches_captured_reference() {
    let session = session();
    let fields = shadow_fields();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let scene = build_scene(&session, &fields, true);
    let reference = shadow_on_reference();
    assert_eq!(reference.total_pixels, 120_000, "参考总像素");
    assert_eq!(reference.rgb.len(), 2, "参考相机数");

    let first = fire(&session, &renderer, &fields, &scene);
    let (rgb_stats, depth_stats, seg_stats) = compare_group(&first, &scene, &reference);
    for camera in &reference.rgb {
        let stats = &rgb_stats[camera.index];
        println!(
            "r09-s3b[shadow-on] cam{} rgb: pixels={} mismatched={} max_byte_diff={} beyond_1_lsb={}",
            camera.index,
            stats.compared,
            stats.mismatched_pixels,
            stats.max_byte_diff,
            stats.beyond_one_lsb_pixels
        );
    }
    for camera in &reference.depth_seg {
        let depth = &depth_stats[camera.index];
        let seg = &seg_stats[camera.index];
        println!(
            "r09-s3b[shadow-on] cam{} depth: compared={} mismatch={} max_ulp={} beyond_ulp={} | \
             seg_mismatch={}",
            camera.index,
            depth.compared,
            depth.mismatched,
            depth.max_ulp,
            depth.beyond_ulp,
            seg.mismatched
        );
    }
    assert_group_criteria(
        "r09-s3b[shadow-on]",
        &reference,
        &rgb_stats,
        &depth_stats,
        &seg_stats,
    );

    // 两次运行 sha256 相同（rgb/depth/seg 各一）。
    let second = fire(&session, &renderer, &fields, &scene);
    assert_eq!(
        hash_rgb(&first.rgb),
        hash_rgb(&second.rgb),
        "两次运行 RGB sha256"
    );
    assert_eq!(
        hash_depth(&first.depth),
        hash_depth(&second.depth),
        "两次运行 depth sha256"
    );
    assert_eq!(
        hash_seg(&first.seg),
        hash_seg(&second.seg),
        "两次运行 seg sha256"
    );
    println!(
        "r09-s3b[shadow-on] run sha256: rgb={} depth={} seg={} (capture_at={CAPTURE_AT})",
        hash_rgb(&first.rgb),
        hash_depth(&first.depth),
        hash_seg(&first.seg)
    );
}

/// shadow-off：① 对 capture-exact 同配方参考同判据；② 逐位回归解锁前既有同配方产品参考；③ 两次运行 sha256。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn shadow_off_bitwise_regression_to_pre_unlock_reference() {
    let session = session();
    let fields = shadow_fields();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let scene = build_scene(&session, &fields, false);
    let reference = shadow_off_reference();
    assert_eq!(reference.total_pixels, 120_000, "参考总像素");
    let first = fire(&session, &renderer, &fields, &scene);
    let (rgb_stats, depth_stats, seg_stats) = compare_group(&first, &scene, &reference);
    assert_group_criteria(
        "r09-s3b[shadow-off]",
        &reference,
        &rgb_stats,
        &depth_stats,
        &seg_stats,
    );
    for camera in &reference.rgb {
        let stats = &rgb_stats[camera.index];
        println!(
            "r09-s3b[shadow-off] cam{} rgb(exact): mismatched={} max_byte_diff={} beyond_1_lsb={}",
            camera.index, stats.mismatched_pixels, stats.max_byte_diff, stats.beyond_one_lsb_pixels
        );
    }

    // ② 解锁前既有同配方产品参考（S7b1 同配方 depth/seg + S7b2 同配方 RGB，S8 实测 max_byte_diff=0）。
    let recipe = recipe_depth_seg();
    let layout = &scene.context.resolved().output;
    for camera in &recipe.cameras {
        let depth_start = layout.depth_adr[camera.index] as usize;
        let depth_actual = &first.depth[depth_start..depth_start + camera.pixels];
        let depth_diff = diff_depth(depth_actual, camera).expect("同配方 depth diff");
        let seg_start = layout.seg_adr[camera.index] as usize;
        let seg_actual = &first.seg[seg_start..seg_start + camera.pixels];
        let seg_diff = diff_seg(seg_actual, camera).expect("同配方 seg diff");
        println!(
            "r09-s3b[shadow-off] cam{} pre-unlock: depth_mismatch={} max_ulp={} seg_mismatch={}",
            camera.index, depth_diff.mismatched, depth_diff.max_ulp, seg_diff.mismatched
        );
        assert_eq!(
            (depth_diff.mismatched, depth_diff.max_ulp),
            (0, 0),
            "shadow-off 必须逐位等于解锁前同配方 depth 参考（cam{} first={:?}）",
            camera.index,
            depth_diff.first
        );
        assert_eq!(
            seg_diff.mismatched, 0,
            "shadow-off 必须逐位等于解锁前同配方 seg 参考（cam{}）",
            camera.index
        );
    }
    let same_recipe = s7b2::parse_rgb_fast_math_false_reference();
    for camera in &same_recipe.cameras {
        let start = layout.rgb_adr[camera.index] as usize;
        let actual = &first.rgb[start..start + camera.pixels];
        let stats = diff_rgb(actual, camera);
        println!(
            "r09-s3b[shadow-off] cam{} pre-unlock rgb: mismatched={} max_byte_diff={} beyond_1_lsb={}",
            camera.index, stats.mismatched_pixels, stats.max_byte_diff, stats.beyond_one_lsb_pixels
        );
        assert_eq!(
            stats.max_byte_diff, 0,
            "shadow-off RGB 必须逐位等于解锁前同配方参考（S8 实测 max_byte_diff=0；cam{} \
             mismatched={} first={:?}）",
            camera.index, stats.mismatched_pixels, stats.first
        );
    }

    // ③ 两次运行 sha256 相同。
    let second = fire(&session, &renderer, &fields, &scene);
    assert_eq!(
        hash_rgb(&first.rgb),
        hash_rgb(&second.rgb),
        "两次运行 RGB sha256"
    );
    assert_eq!(
        hash_depth(&first.depth),
        hash_depth(&second.depth),
        "两次运行 depth sha256"
    );
    assert_eq!(
        hash_seg(&first.seg),
        hash_seg(&second.seg),
        "两次运行 seg sha256"
    );
    println!(
        "r09-s3b[shadow-off] run sha256: rgb={} depth={} seg={} (max_depth_ulp_allowed={MAX_DEPTH_ULP})",
        hash_rgb(&first.rgb),
        hash_depth(&first.depth),
        hash_seg(&first.seg)
    );
}

/// 阴影可见性二值对照（只作语义交叉校验，**不是**等价/精度结论）。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn shadow_visibility_binary_control() {
    let session = session();
    let fields = shadow_fields();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let on_scene = build_scene(&session, &fields, true);
    let off_scene = build_scene(&session, &fields, false);
    let on = fire(&session, &renderer, &fields, &on_scene);
    let off = fire(&session, &renderer, &fields, &off_scene);

    // 参考层同口径计数（S3a 记录的管线交叉校验信号：两组 depth/seg 参考逐字节相同）。
    let on_reference = shadow_on_reference();
    let off_reference = shadow_off_reference();
    let reference_rgb = shadow::rgb_difference_pixels(&on_reference, &off_reference);
    let reference_depth = shadow::depth_difference_pixels(&on_reference, &off_reference);
    let reference_seg = shadow::seg_difference_pixels(&on_reference, &off_reference);
    assert_eq!(
        reference_depth, 0,
        "参考层两组 depth 应逐位相同（S3a 交叉校验信号）"
    );
    assert_eq!(
        reference_seg, 0,
        "参考层两组 seg 应逐位相同（S3a 交叉校验信号）"
    );
    assert!(
        reference_rgb > 0,
        "阴影必须在参考层可见（rgb 差异像素 {reference_rgb}）"
    );

    let product_rgb = product_rgb_difference(&on, &off);
    let product_depth = product_depth_difference(&on, &off);
    let product_seg = product_seg_difference(&on, &off);
    println!(
        "r09-s3b[visibility] product: rgb_diff={} depth_diff={} seg_diff={} | reference: \
         rgb_diff={} depth_diff={} seg_diff={} (交叉校验信号，非等价/精度结论)",
        product_rgb, product_depth, product_seg, reference_rgb, reference_depth, reference_seg
    );
    // 语义断言：阴影只影响光照；产品层 depth/seg 不应因 use_shadows 变化，RGB 必须变化。
    assert_eq!(product_depth, 0, "shadow-on/off 的 depth 不应不同");
    assert_eq!(product_seg, 0, "shadow-on/off 的 seg 不应不同");
    assert!(product_rgb > 0, "shadow-on/off 的 RGB 必须不同（阴影可见）");
}
