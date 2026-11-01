#![cfg(feature = "cuda-probe")]
//! 渲染/BVH S7b3 验收：flag 变体矩阵（逐变体独立参考）+ AA(spp=2) 图像级参考 +
//! S7b1/S7b2 不回归基线（渲染/BVH S7b3）。
//!
//! 口径：
//! - 每组 flag 变体**各自一份**同配方（`use_fast_math=False`）参考，独立断言
//!   RGB ≤1 LSB（解包后每通道）、depth ≤4 ULP、seg 全等；「开了不崩」不算通过。
//! - 变体语义要具体可判别：texture off 不再采样纹理（差异只出现在有纹理几何）、
//!   culling off 背面命中（seg/depth 变化）、specular/emission off 对应贡献消失
//!   （与关项参考一致且与 base 有差异）；判别力统计来自采集期 `index.json`，
//!   测试再对产品输出做独立断言。
//! - `vertex_normals_off` 在 scene-a 上**参考本身无判别力**（0 像素差异，见 README §），
//!   该分支的判别由合成顶点法线用例（`synthetic_mesh_normal_switches_vertex_normal_path`）
//!   承担：把 `mesh_normal` 换成极端方向，开/关两版输出必须不同。
//! - AA(spp=2)：nsamples=4，子样本顺序 = S5 布局 `[sample][camera][row][column]`，
//!   与上游 `for sample in range(nsamples)` 的 `ray_base = sample * total_rays` 一致。
//! - 不回归：S7b1 depth/seg 120,000/120,000 位等、S7b2 RGB 同配方 120,000/120,000 0 差异。
//!
//! GPU/NVRTC 用例默认 ignore：
//! cargo test --locked --features cuda-probe --test render_kernel_flags -- --ignored --test-threads=1 --nocapture

#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/s7b3.rs"]
mod s7b3;

use mjwarp_rs::render::ImageOutput;
use mjwarp_rs::render::kernel::launch::compile_render_artifact;
use mjwarp_rs::render::kernel::output::{MAX_DEPTH_ULP, diff_seg};
use mjwarp_rs::render::kernel::{
    AA_RESOLVE_ENTRY, AA_RESOLVE_SOURCE, MEGAKERNEL_ENTRY, MEGAKERNEL_SOURCE, MegakernelRenderer,
    check_render_ptx,
};
use s7b2::{diff_rgb, read_rgb, render_fields};
use s7b3::{
    AA_VARIANT, VARIANTS, camera_data_poses, create_variant_context, flags_index, precomputed_rays,
    render_variant, variant_depth_seg, variant_rgb,
};

/// `index.json` 记录的判别力：6 变体齐、两次捕获位等、除 vertex_normals 外均有判别力。
#[test]
fn flag_fixtures_are_complete_and_documented() {
    let index = flags_index();
    let variants = index["variants"]
        .as_array()
        .expect("variants 数组")
        .iter()
        .map(|variant| variant["variant"].as_str().expect("variant 名"))
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        vec![
            "textures_off",
            "vertex_normals_off",
            "specular_off",
            "emission_off",
            "culling_off",
            "aa2"
        ],
        "变体清单"
    );
    for variant in index["variants"].as_array().expect("variants") {
        let name = variant["variant"].as_str().expect("variant");
        let deterministic = variant["determinism"]["bitwise_identical"]
            .as_bool()
            .expect("determinism");
        assert!(deterministic, "{name} 两次捕获必须逐位一致");
        let discriminating = variant["has_discriminating_effect"]
            .as_bool()
            .expect("has_discriminating_effect");
        if name == "vertex_normals_off" {
            // scene-a 夹具下该 flag 无判别力（参考自身开/关逐位相同）；如实记录、不得静默。
            assert!(
                !discriminating,
                "vertex_normals_off 在 scene-a 上应记录为无判别力（见 README）；若参考变了需重评"
            );
        } else {
            assert!(discriminating, "{name} 必须有判别力（差异像素非空）");
        }
    }
    // 每组变体必须有自己的参考文件（逐变体独立可判定，不允许共用）。
    for name in VARIANTS.iter().chain(std::iter::once(&AA_VARIANT)) {
        let _ = variant_rgb(name);
    }
    for name in ["culling_off", AA_VARIANT] {
        assert!(
            variant_depth_seg(name).is_some(),
            "{name} 必须有 depth/seg 参考"
        );
    }
    s7b3::assert_culling_off_sphere_matches_recipe_reference();
    println!("s7b3 fixtures: 6 variants complete, per-variant references present");
}

/// NVRTC 编译 + A′ 渲染门禁（megakernel 与 AA 解析两个入口都要过）。
#[test]
#[ignore = "需要 NVRTC 与 W15 头树（GPU 窗口内统一执行）"]
fn flag_and_aa_sources_compile_under_render_gate() {
    let artifact = compile_render_artifact(MEGAKERNEL_ENTRY, MEGAKERNEL_SOURCE, MEGAKERNEL_ENTRY)
        .expect("megakernel 编译 + A′ 门禁");
    assert_eq!(check_render_ptx(artifact.ptx(), MEGAKERNEL_ENTRY), Ok(()));
    assert!(artifact.ptx().contains(".visible .entry render_megakernel"));
    println!(
        "s7b3 megakernel: source_sha256={} ptx_sha256={} ptx_bytes={}",
        artifact.source_sha256(),
        artifact.ptx_sha256(),
        artifact.ptx().len()
    );
    let resolve = compile_render_artifact(AA_RESOLVE_ENTRY, AA_RESOLVE_SOURCE, AA_RESOLVE_ENTRY)
        .expect("aa_resolve 编译 + A′ 门禁");
    assert_eq!(check_render_ptx(resolve.ptx(), AA_RESOLVE_ENTRY), Ok(()));
    assert!(resolve.ptx().contains(".visible .entry aa_resolve"));
    println!(
        "s7b3 aa_resolve: source_sha256={} ptx_sha256={} ptx_bytes={}",
        resolve.source_sha256(),
        resolve.ptx_sha256(),
        resolve.ptx().len()
    );
}

/// 判据：变体 vs 自己参考。
fn check_variant_reference(
    context: &mjwarp_rs::render::context::RenderContext,
    variant: &str,
    with_depth_seg: bool,
) {
    let reference = variant_rgb(variant);
    let rgb = read_rgb(context);
    let layout = &context.resolved().output;
    assert_eq!(rgb.len(), layout.rgb_pixels, "{variant} rgb 像素数");
    let mut beyond = 0usize;
    let mut max_byte = 0u8;
    for camera in &reference.cameras {
        let start = layout.rgb_adr[camera.index] as usize;
        let actual = &rgb[start..start + camera.pixels];
        let stats = diff_rgb(actual, camera);
        beyond += stats.beyond_one_lsb_pixels;
        max_byte = max_byte.max(stats.max_byte_diff);
        println!(
            "s7b3[{variant}] cam{}: mismatched={} max_byte_diff={} beyond_1_lsb={} first={:?}",
            camera.index,
            stats.mismatched_pixels,
            stats.max_byte_diff,
            stats.beyond_one_lsb_pixels,
            stats.first
        );
    }
    assert_eq!(beyond, 0, "{variant} RGB 必须 ≤1 LSB（同配方参考）");
    if !with_depth_seg {
        return;
    }
    let reference = variant_depth_seg(variant).expect("depth/seg 参考");
    let depth = s7b1::read_depth(context);
    let seg = s7b1::read_seg(context);
    for camera in &reference.reference.cameras {
        let depth_base = camera.depth_adr as usize;
        let seg_base = camera.seg_adr as usize;
        let pixels = camera.depth_bits.len();
        let depth_diff = s7b1::outline::variant_depth_diff(
            &depth[depth_base..depth_base + pixels],
            camera,
            variant,
            AA_VARIANT,
        );
        let seg_diff = diff_seg(&seg[seg_base..seg_base + pixels], camera).expect("seg");
        println!(
            "s7b3[{variant}] cam{}: depth_mismatch={} max_ulp={} seg_mismatch={}",
            camera.index, depth_diff.mismatched, depth_diff.max_ulp, seg_diff.mismatched
        );
        assert!(
            depth_diff.max_ulp <= MAX_DEPTH_ULP,
            "{variant} cam{} depth 超 4 ULP：max={} first={:?}",
            camera.index,
            depth_diff.max_ulp,
            depth_diff.first
        );
        assert_eq!(
            seg_diff.mismatched, 0,
            "{variant} cam{} seg 必须全等",
            camera.index
        );
    }
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn flag_variants_match_own_references() {
    let session = s7b1::session();
    let fields = render_fields();
    let renderer = MegakernelRenderer::new(&session).expect("S7b3 内核编译");
    for variant in VARIANTS {
        let context = create_variant_context(&session, &fields, variant);
        let poses = camera_data_poses(&fields, context.resolved());
        let rays = precomputed_rays(&session, &fields);
        render_variant(&session, &renderer, &context, &fields, &poses, &rays);
        let with_depth_seg = matches!(variant, "culling_off");
        check_variant_reference(&context, variant, with_depth_seg);
    }
    println!("s7b3 flag variants: 5/5 matched own same-recipe references");
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn aa2_matches_same_recipe_reference() {
    let session = s7b1::session();
    let fields = render_fields();
    let renderer = MegakernelRenderer::new(&session).expect("S7b3 内核编译");
    let context = create_variant_context(&session, &fields, AA_VARIANT);
    assert_eq!(context.resolved().plan.samples_per_pixel, 2);
    assert_eq!(context.resolved().plan.samples, 4, "spp=2 → 4 子样本");
    let rays = s7b3::aa_rays(&session, &fields);
    assert_eq!(
        rays.len(),
        4 * 120_000,
        "AA 射线条数 = nsamples × total_rays"
    );
    let poses = camera_data_poses(&fields, context.resolved());
    render_variant(&session, &renderer, &context, &fields, &poses, &rays);
    check_variant_reference(&context, AA_VARIANT, true);

    // 两次运行位等（AA 累积缓冲每次重算）。
    let first = read_rgb(&context);
    render_variant(&session, &renderer, &context, &fields, &poses, &rays);
    let second = read_rgb(&context);
    assert_eq!(first, second, "AA 两次运行 RGB 必须逐位相同");
    println!("s7b3 aa2: matched same-recipe reference and repeat runs are bitwise identical");
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn baseline_no_regression_after_flags_and_aa() {
    let session = s7b1::session();
    let fields = render_fields();
    let renderer = MegakernelRenderer::new(&session).expect("S7b3 内核编译");
    let params = s7b3::default_params();
    let context = s7b3::create_context_with_params(&session, &fields, &params);
    let poses = camera_data_poses(&fields, context.resolved());
    let rays = precomputed_rays(&session, &fields);
    render_variant(&session, &renderer, &context, &fields, &poses, &rays);

    // S7b2 RGB 同配方 120,000/120,000 **0 差异**（硬门）。
    let rgb_reference = s7b2::parse_rgb_fast_math_false_reference();
    let rgb = read_rgb(&context);
    let layout = &context.resolved().output;
    let mut rgb_mismatch = 0usize;
    for camera in &rgb_reference.cameras {
        let start = layout.rgb_adr[camera.index] as usize;
        let stats = diff_rgb(&rgb[start..start + camera.pixels], camera);
        rgb_mismatch += stats.mismatched_pixels;
        println!(
            "s7b3[baseline] cam{}: rgb_mismatched={} max_byte_diff={} beyond_1_lsb={}",
            camera.index, stats.mismatched_pixels, stats.max_byte_diff, stats.beyond_one_lsb_pixels
        );
    }
    assert_eq!(rgb_mismatch, 0, "S7b2 RGB 同配方基线不得回归（0 像素差异）");

    // S7b1 depth/seg 120,000/120,000 位等、max 0 ULP。
    let recipe = s7b2::recipe_depth_seg();
    let depth = s7b1::read_depth(&context);
    let seg = s7b1::read_seg(&context);
    for camera in &recipe.cameras {
        let depth_base = camera.depth_adr as usize;
        let seg_base = camera.seg_adr as usize;
        let pixels = camera.depth_bits.len();
        let depth_diff =
            s7b1::outline::recipe_depth_diff(&depth[depth_base..depth_base + pixels], camera);
        let seg_diff = diff_seg(&seg[seg_base..seg_base + pixels], camera).expect("seg");
        println!(
            "s7b3[baseline] cam{}: depth_mismatch={} max_ulp={} seg_mismatch={}",
            camera.index, depth_diff.mismatched, depth_diff.max_ulp, seg_diff.mismatched
        );
        assert_eq!(
            (depth_diff.mismatched, depth_diff.max_ulp),
            (0, 0),
            "S7b1 depth 基线不得回归（cam{}）",
            camera.index
        );
        assert_eq!(
            seg_diff.mismatched, 0,
            "S7b1 seg 基线不得回归（cam{}）",
            camera.index
        );
    }
    assert!(context.outputs().is_enabled(ImageOutput::Rgb));
    println!("s7b3 baseline: S7b1 depth/seg and S7b2 RGB baselines unchanged");
}

/// `vertex_normals` 分支的合成判别：把 `mesh_normal` 换成极端方向后，开/关两版输出
/// `vertex_normals` 分支的判别（pinned 参考对照，补 S7b3 复核缺口）：
/// 合成 `mesh_normal` = [1,0,0] / [0,0,1] 后，产品对 pinned 3.15 同配方参考 ≤1 LSB；
/// 且 `enable_vertex_normals` on/off 输出不同（分支生效）。默认 scene-a 下该 flag
/// 无判别力的结论保留（由 `flag_fixtures_are_complete_and_documented` 断言）。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn synthetic_mesh_normal_matches_pinned_reference_and_discriminates() {
    let session = s7b1::session();
    let renderer = MegakernelRenderer::new(&session).expect("S7b3 内核编译");
    let index = s7b3::synthetic_normal_index();
    let recorded = index["variants"]
        .as_array()
        .expect("variants")
        .iter()
        .map(|variant| {
            (
                variant["variant"].as_str().expect("variant").to_string(),
                variant["effect_vs_default_scene_a"]["changed_pixels"]
                    .as_array()
                    .expect("changed_pixels")
                    .iter()
                    .map(|value| value.as_i64().expect("changed"))
                    .sum::<i64>(),
            )
        })
        .collect::<Vec<_>>();
    for (suffix, direction) in [("x", [1.0f32, 0.0, 0.0]), ("z", [0.0, 0.0, 1.0])] {
        let variant = format!("synthetic_normals_{suffix}");
        let recorded_changed = recorded
            .iter()
            .find(|(name, _)| name == &variant)
            .map(|(_, changed)| *changed)
            .expect("索引记录");
        println!(
            "s7b3[synthetic] {suffix}: pinned override effect vs default scene-a = {recorded_changed} pixels"
        );
        assert!(
            recorded_changed > 0,
            "{variant} 的 pinned 覆盖必须对默认 scene-a 有效（生效断言）"
        );
        let reference = s7b3::synthetic_normal_rgb(suffix);
        let normals = direction.repeat(4).to_vec();
        let fields = s7b3::fields_with_mesh_normal(&normals);
        let mut rendered = Vec::new();
        for enabled in [true, false] {
            let params = mjwarp_rs::render::ContextParams {
                enable_vertex_normals: enabled,
                ..s7b3::default_params()
            };
            let context = s7b3::create_context_with_params(&session, &fields, &params);
            let poses = camera_data_poses(&fields, context.resolved());
            let rays = precomputed_rays(&session, &fields);
            render_variant(&session, &renderer, &context, &fields, &poses, &rays);
            let rgb = read_rgb(&context);
            if enabled {
                let layout = &context.resolved().output;
                let mut beyond = 0usize;
                for camera in &reference.cameras {
                    let start = layout.rgb_adr[camera.index] as usize;
                    let stats = diff_rgb(&rgb[start..start + camera.pixels], camera);
                    beyond += stats.beyond_one_lsb_pixels;
                    println!(
                        "s7b3[synthetic] {suffix} cam{}: mismatched={} max_byte_diff={} beyond_1_lsb={} first={:?}",
                        camera.index,
                        stats.mismatched_pixels,
                        stats.max_byte_diff,
                        stats.beyond_one_lsb_pixels,
                        stats.first
                    );
                }
                assert_eq!(beyond, 0, "{variant} 产品对 pinned 同配方参考必须 ≤1 LSB");
            }
            rendered.push(rgb);
        }
        let changed = rendered[0]
            .iter()
            .zip(&rendered[1])
            .filter(|(left, right)| left != right)
            .count();
        println!("s7b3[synthetic] {suffix}: vertex_on_vs_off_changed_pixels={changed}");
        assert!(
            changed > 0,
            "{variant} 的 vertex_normals on/off 必须可判别（实际 0 像素）"
        );
    }
    println!(
        "s7b3 synthetic normals: product matches pinned same-recipe references (≤1 LSB) and on/off discriminates"
    );
}

/// S7b3 语义升级后的拒绝面：AA 已支持（不再是拒绝项），剩余 3 类仍写前拒绝；
/// AA 射线数不符时写前拒绝（InvalidArray，不发射内核）。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn aa_supported_and_remaining_rejections_pre_launch() {
    let session = s7b1::session();
    let fields = render_fields();
    let model = mjwarp_rs::render::CameraModel::from_fields(&fields).expect("相机模型");
    let renderer = MegakernelRenderer::new(&session).expect("S7b3 内核编译");

    // 1) AA 已支持：spp=2 + nsamples × total_rays 射线 → 成功发射。
    let aa_context = create_variant_context(&session, &fields, AA_VARIANT);
    let aa_poses = camera_data_poses(&fields, aa_context.resolved());
    let aa_rays = s7b3::aa_rays(&session, &fields);
    render_variant(
        &session,
        &renderer,
        &aa_context,
        &fields,
        &aa_poses,
        &aa_rays,
    );

    // 2) AA + 1× 射线（坏配置）→ 写前 InvalidArray，不发射。
    let wrong_rays = precomputed_rays(&session, &fields);
    let result = renderer.render(&aa_context, &fields, &model, &aa_poses, &wrong_rays);
    assert!(
        matches!(
            result,
            Err(mjwarp_rs::runtime::launcher::LauncherError::InvalidArray(
                "ray_count"
            ))
        ),
        "AA 射线数不符必须写前拒绝（实际 {result:?}）"
    );

    // 3) R09-S2b：`use_shadows=true` 已解锁（host 前置拒绝移除）→ 必须被接受并发射成功。
    let shadow_params = mjwarp_rs::render::ContextParams {
        use_shadows: true,
        ..s7b3::default_params()
    };
    let shadow_context = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        s7b3::create_context_with_params(&session, &fields, &shadow_params)
    })) {
        Ok(context) => context,
        Err(_) => panic!("use_shadows=true 的 context 必须可创建（R09-S2b 解锁）"),
    };
    let shadow_poses = camera_data_poses(&fields, shadow_context.resolved());
    let shadow_rays = precomputed_rays(&session, &fields);
    let shadow_result = renderer.render(
        &shadow_context,
        &fields,
        &model,
        &shadow_poses,
        &shadow_rays,
    );
    assert!(
        shadow_result.is_ok(),
        "use_shadows=true 必须被接受（实际 {shadow_result:?}）"
    );

    // 4) R02-S1：`render_skybox=true` 已解锁（host 前置拒绝移除）→ 必须被接受并发射成功。
    let skybox_params = mjwarp_rs::render::ContextParams {
        render_skybox: true,
        ..s7b3::default_params()
    };
    let skybox_context = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        s7b3::create_context_with_params(&session, &fields, &skybox_params)
    })) {
        Ok(context) => context,
        Err(_) => panic!("render_skybox=true 的 context 必须可创建（R02-S1 解锁）"),
    };
    assert!(
        skybox_context.resolved().skybox.is_some(),
        "scene-a 必须解析出天空盒纹理"
    );
    let skybox_poses = camera_data_poses(&fields, skybox_context.resolved());
    let skybox_rays = precomputed_rays(&session, &fields);
    let skybox_result = renderer.render(
        &skybox_context,
        &fields,
        &model,
        &skybox_poses,
        &skybox_rays,
    );
    assert!(
        skybox_result.is_ok(),
        "render_skybox=true 必须被接受（实际 {skybox_result:?}）"
    );

    // 5) 其余仍写前拒绝（AA/阴影/天空盒均已从拒绝面移除；动态射线由本用例守住）。
    let cases = [(
        "use_precomputed_rays=false",
        mjwarp_rs::render::ContextParams {
            use_precomputed_rays: false,
            ..s7b3::default_params()
        },
    )];
    for (name, params) in cases {
        let created = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            s7b3::create_context_with_params(&session, &fields, &params)
        }));
        let Ok(context) = created else {
            continue;
        };
        let poses = camera_data_poses(&fields, context.resolved());
        let rays = precomputed_rays(&session, &fields);
        let result = renderer.render(&context, &fields, &model, &poses, &rays);
        assert!(
            matches!(
                result,
                Err(mjwarp_rs::runtime::launcher::LauncherError::Unsupported(_))
            ),
            "{name} 必须写前拒绝（实际 {result:?}）"
        );
    }
    println!(
        "s7b3 rejection surface: AA + shadows + skybox supported; precomputed=false rejected; bad AA ray count rejected"
    );
}
