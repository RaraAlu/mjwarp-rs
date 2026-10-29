#![cfg(feature = "cuda-probe")]
//! 渲染/BVH S7b2 验收：81 槽全签名着色/RGB 内核（渲染/BVH S7b2）。
//!
//! 口径（任务 JSON）：默认 context、spp=1、`use_precomputed_rays=true`；RGB 与 S0b
//! `reference/rgb.npz` 派生夹具逐像素每通道 ≤1 LSB；depth/seg 相对 S7b1（120,000/120,000
//! 位等）不回归；两次运行输出哈希逐位相同；不支持配置在写前拒绝。
//!
//! 主机用例默认执行；GPU/NVRTC 用例默认 ignore：
//! cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored --test-threads=1

#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;

use mjwarp_rs::render::CameraModel;
use mjwarp_rs::render::ImageOutput;
use mjwarp_rs::render::kernel::launch::compile_render_artifact;
use mjwarp_rs::render::kernel::output::{MAX_DEPTH_ULP, diff_depth, diff_seg};
use mjwarp_rs::render::kernel::{
    MEGAKERNEL_ENTRY, MEGAKERNEL_SOURCE, MegakernelRenderer, check_render_ptx,
};
use mjwarp_rs::runtime::launcher::{LauncherError, MEGAKERNEL_SLOTS};
use s7b1::{camera_data_poses, read_depth, read_seg};
use s7b2::{
    check_light_data, create_context, default_params, diff_rgb,
    parse_rgb_fast_math_false_reference, parse_rgb_reference, precomputed_rays, read_rgb,
    recipe_depth_seg, render, render_fields, session,
};

#[test]
fn reference_fixture_and_light_data_poses() {
    let reference = parse_rgb_reference();
    assert_eq!(reference.total_pixels, 120_000);
    assert_eq!(reference.cameras.len(), 2);
    assert_eq!(reference.cameras[0].pixels, 320 * 240);
    assert_eq!(reference.cameras[1].pixels, 240 * 180);
    for camera in &reference.cameras {
        assert_eq!(camera.packed.len(), camera.pixels, "RGB 位型长度");
    }
    assert_eq!(reference.lights.len(), 3, "scene-a 三盏灯");
    let fields = render_fields();
    check_light_data(&reference, &fields);
    // 同配方参考（`use_fast_math=False`）是 RGB `<=1 LSB` 硬指标依据；长度/总像素须自洽。
    let same_recipe = parse_rgb_fast_math_false_reference();
    assert_eq!(same_recipe.total_pixels, 120_000, "同配方参考总像素");
    assert_eq!(same_recipe.cameras.len(), 2);
    assert_eq!(
        same_recipe.cross_recipe_histograms.len(),
        2,
        "跨配方偏差记录"
    );
    // 81 槽（1 bounds + 73 数组 + 7 标量）与 S7a 冻结签名一致。
    assert_eq!(MEGAKERNEL_SLOTS.len(), 81);
    assert_eq!(
        MEGAKERNEL_SLOTS[0],
        mjwarp_rs::runtime::launcher::RenderArgKind::Bounds2D
    );
}

/// NVRTC 编译 + A′ 渲染门禁（无 GPU 也可；与 S7a/S7b1 同口径归入 ignored 显式执行）。
#[test]
#[ignore = "需要 NVRTC 与 W15 头树（无 GPU 也可，但按窗口统一执行）"]
fn megakernel_source_compiles_under_render_gate() {
    let artifact = compile_render_artifact(MEGAKERNEL_ENTRY, MEGAKERNEL_SOURCE, MEGAKERNEL_ENTRY)
        .expect("NVRTC 编译 + f32-render 门禁");
    assert_eq!(check_render_ptx(artifact.ptx(), MEGAKERNEL_ENTRY), Ok(()));
    assert!(artifact.ptx().contains(".visible .entry render_megakernel"));
    println!(
        "s7b2 ptx: source_sha256={} ptx_sha256={} ptx_bytes={}",
        artifact.source_sha256(),
        artifact.ptx_sha256(),
        artifact.ptx().len()
    );
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn rgb_matches_reference_and_depth_seg_do_not_regress() {
    let session = session();
    let fields = render_fields();
    let reference = parse_rgb_reference();
    let context = create_context(&session, &fields, &default_params());
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let recipe = recipe_depth_seg();
    let poses = camera_data_poses(&recipe, &model, context.resolved());
    let rays = precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    render(
        &session, &renderer, &context, &fields, &model, &poses, &rays,
    );

    // ① depth/seg 回归门：与 S7b1 的同配方参考逐位相同（120,000/120,000）。
    let depth = read_depth(&context);
    let seg = read_seg(&context);
    for camera in &recipe.cameras {
        let base = camera.depth_adr as usize;
        let seg_base = camera.seg_adr as usize;
        let actual_depth = &depth[base..base + camera.pixels];
        let actual_seg = &seg[seg_base..seg_base + camera.pixels];
        let depth_diff = diff_depth(actual_depth, camera).expect("depth diff");
        let seg_diff = diff_seg(actual_seg, camera).expect("seg diff");
        assert_eq!(
            (depth_diff.mismatched, depth_diff.max_ulp),
            (0, 0),
            "S7b2 后 depth 必须保持 S7b1 位等（cam{} first={:?}）",
            camera.index,
            depth_diff.first
        );
        assert!(depth_diff.max_ulp <= MAX_DEPTH_ULP);
        assert_eq!(
            seg_diff.mismatched, 0,
            "S7b2 后 seg 必须保持 S7b1 全等（cam{}）",
            camera.index
        );
        println!(
            "s7b2[regress] cam{}: depth_mismatch={} max_ulp={} seg_mismatch={}",
            camera.index, depth_diff.mismatched, depth_diff.max_ulp, seg_diff.mismatched
        );
    }

    // ② RGB 硬指标：对**同配方**（`use_fast_math=False`）参考解包后每通道 ≤1 LSB
    //    （byte 差 ≤1）。S0b fast-math 参考是跨配方来源，只作偏差记录（见 ②b）。
    let rgb = read_rgb(&context);
    let layout = &context.resolved().output;
    assert_eq!(rgb.len(), layout.rgb_pixels);
    let same_recipe = parse_rgb_fast_math_false_reference();
    let mut stats_all = Vec::with_capacity(same_recipe.cameras.len());
    for camera in &same_recipe.cameras {
        let start = layout.rgb_adr[camera.index] as usize;
        let actual = &rgb[start..start + camera.pixels];
        let stats = diff_rgb(actual, camera);
        let seg_pairs = &recipe.cameras[camera.index].seg_pairs;
        let (histogram, by_geom) = s7b2::rgb_diff_histogram(actual, camera, seg_pairs);
        println!(
            "s7b2[rgb-same-recipe] cam{}: pixels={} mismatched={} max_byte_diff={} \
             beyond_1_lsb={} first={:?} histogram(0,1,2,3,>3)={:?} beyond_by_geom={:?}",
            camera.index,
            stats.compared,
            stats.mismatched_pixels,
            stats.max_byte_diff,
            stats.beyond_one_lsb_pixels,
            stats.first,
            histogram,
            by_geom
        );
        stats_all.push(stats);
    }
    for (camera, stats) in same_recipe.cameras.iter().zip(&stats_all) {
        assert_eq!(
            stats.beyond_one_lsb_pixels,
            0,
            "cam{} RGB 超 1 LSB（同配方参考）：{} / {} 像素，max_byte_diff={} first={:?}",
            camera.index,
            stats.beyond_one_lsb_pixels,
            stats.compared,
            stats.max_byte_diff,
            stats.first
        );
    }

    // ②b 跨配方偏差记录（**不作通过依据**，也不作为产品缺陷判定）：产品 vs S0b
    //    fast-math 参考的量化直方图，并与采集脚本记录的「同配方参考 vs S0b 参考」直方图并列打印。
    for camera in &reference.cameras {
        let start = layout.rgb_adr[camera.index] as usize;
        let actual = &rgb[start..start + camera.pixels];
        let stats = diff_rgb(actual, camera);
        let (histogram, by_geom) =
            s7b2::rgb_diff_histogram(actual, camera, &recipe.cameras[camera.index].seg_pairs);
        println!(
            "s7b2[rgb-cross-recipe-record] cam{}: beyond_1_lsb={} max_byte_diff={} \
             histogram(0,1,2,3,>3)={:?} beyond_by_geom={:?} fixture_recorded={:?}",
            camera.index,
            stats.beyond_one_lsb_pixels,
            stats.max_byte_diff,
            histogram,
            by_geom,
            same_recipe.cross_recipe_histograms.get(camera.index)
        );
    }

    // ③ 两次运行输出哈希逐位相同（RGB + depth + seg）。
    let first_hash = s7b1::output_hash(&depth, &seg);
    let rgb_word_hash = rgb.iter().fold(0u64, |hash, value| {
        (hash ^ u64::from(*value)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    render(
        &session, &renderer, &context, &fields, &model, &poses, &rays,
    );
    let second_depth = read_depth(&context);
    let second_seg = read_seg(&context);
    let second_rgb = read_rgb(&context);
    assert_eq!(second_rgb, rgb, "两次运行 RGB 必须逐位相同");
    assert_eq!(second_depth, depth, "两次运行 depth 必须逐位相同");
    assert_eq!(second_seg, seg, "两次运行 seg 必须逐位相同");
    println!(
        "s7b2 run: rgb_pixels={} depth_pixels={} seg_pairs={} depth_seg_hash={:016x} rgb_hash={:016x}",
        rgb.len(),
        depth.len(),
        seg.len(),
        first_hash,
        rgb_word_hash
    );
    assert!(context.outputs().is_enabled(ImageOutput::Rgb));
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn unsupported_configurations_are_rejected_before_launch() {
    let session = session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let rays = precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let recipe = recipe_depth_seg();

    // R09-S2b：`use_shadows=true` 已解锁（host 前置拒绝移除）→ 必须被接受并发射成功。
    let shadow_params = mjwarp_rs::render::ContextParams {
        use_shadows: true,
        ..default_params()
    };
    let shadow_context = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        create_context(&session, &fields, &shadow_params)
    })) {
        Ok(context) => context,
        Err(_) => panic!("use_shadows=true 的 context 必须可创建（R09-S2b 解锁）"),
    };
    let shadow_poses = camera_data_poses(&recipe, &model, shadow_context.resolved());
    let shadow_result = renderer.render(&shadow_context, &fields, &model, &shadow_poses, &rays);
    assert!(
        shadow_result.is_ok(),
        "use_shadows=true 必须被接受（实际 {shadow_result:?}）"
    );

    // R02-S1：`render_skybox=true` 已解锁（host 前置拒绝移除）→ 必须被接受并发射成功。
    let skybox_params = mjwarp_rs::render::ContextParams {
        render_skybox: true,
        ..default_params()
    };
    let skybox_context = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        create_context(&session, &fields, &skybox_params)
    })) {
        Ok(context) => context,
        Err(_) => panic!("render_skybox=true 的 context 必须可创建（R02-S1 解锁）"),
    };
    assert!(
        skybox_context.resolved().skybox.is_some(),
        "scene-a 必须解析出天空盒纹理"
    );
    let skybox_poses = camera_data_poses(&recipe, &model, skybox_context.resolved());
    let skybox_result = renderer.render(&skybox_context, &fields, &model, &skybox_poses, &rays);
    assert!(
        skybox_result.is_ok(),
        "render_skybox=true 必须被接受（实际 {skybox_result:?}）"
    );

    // 仍未支持的特性保留写前拒绝：splat（R07 前不实现）。
    let splat_position = [[0.0f32, 0.0, 1.0]];
    let splat_rotation = [[1.0f32, 0.0, 0.0, 0.0]];
    let splat_scale = [[0.1f32, 0.1, 0.1]];
    let splat_rgba = [[1.0f32, 1.0, 1.0, 1.0]];
    let splat_params = mjwarp_rs::render::ContextParams {
        splat: mjwarp_rs::render::context::params::SplatInputs {
            position: Some(&splat_position),
            rotation: Some(&splat_rotation),
            scale: Some(&splat_scale),
            rgba: Some(&splat_rgba),
            adr: None,
            group_id: None,
        },
        ..default_params()
    };
    let splat_context = create_context(&session, &fields, &splat_params);
    let splat_poses = camera_data_poses(&recipe, &model, splat_context.resolved());
    let splat_result = renderer.render(&splat_context, &fields, &model, &splat_poses, &rays);
    assert!(
        matches!(splat_result, Err(LauncherError::Unsupported(_))),
        "splat 必须在写前拒绝（实际 {splat_result:?}）"
    );

    let cases: Vec<(&str, mjwarp_rs::render::ContextParams<'_>)> = vec![
        (
            "use_precomputed_rays=false",
            mjwarp_rs::render::ContextParams {
                use_precomputed_rays: false,
                ..default_params()
            },
        ),
        // 注：`samples_per_pixel=2`（AA）已由 S7b3 实现、`use_shadows` 由 R09-S2b、
        // `render_skybox` 由 R02-S1 解锁，均不再是拒绝项；本用例保留仍在的拒绝面（动态射线/splat）。
    ];
    for (name, params) in cases {
        let context = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            create_context(&session, &fields, &params)
        })) {
            Ok(context) => context,
            Err(_) => continue,
        };
        let poses = camera_data_poses(&recipe, &model, context.resolved());
        let result = renderer.render(&context, &fields, &model, &poses, &rays);
        assert!(
            matches!(result, Err(LauncherError::Unsupported(_))),
            "{name} 必须在写前拒绝（实际 {result:?}）"
        );
    }
    println!(
        "s7b2 rejection surface: shadows+skybox supported; precomputed=false/splat rejected (AA moved to S7b3)"
    );
}
