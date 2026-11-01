#![cfg(feature = "cuda-probe")]
//! 渲染/BVH S7b1 验收：主射线阶段（平面深度 + 分割）。
//!
//! 参考：`fixtures/render-bvh/scene-a/s7b/depth_seg.json`（authoring 期由
//! `scripts/authoring/render-bvh-s7b/convert_depth_seg.py` 从 S0b `depth.npz`/`seg.npz`
//! 与 pinned 3.15 `mjwarp.forward` 的相机 data 位姿派生）。相机 1 是 TARGETBODY 相机，
//! 渲染使用其 data `cam_xmat`（跟踪 look-at），共享夹具在 `support/render_bvh/s7b1.rs`。
//!
//! 主机用例默认执行；GPU/NVRTC 用例默认 ignore：
//! cargo test --locked --features cuda-probe --test render_kernel_depth_seg -- --ignored --test-threads=1

#[path = "support/render_bvh/s7b1.rs"]
mod support;

use mjwarp_rs::diagnostics::{InputError, TransferError};
use mjwarp_rs::render::context::owner::OwnerError;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::launch::compile_render_artifact;
use mjwarp_rs::render::kernel::output::{
    MAX_DEPTH_ULP, diff_depth, diff_seg, parse_reference, ulp_distance,
};
use mjwarp_rs::render::kernel::{
    DEPTH_SEG_ENTRY, DEPTH_SEG_SOURCE, DepthSegRenderer, check_render_ptx,
};
use mjwarp_rs::render::{CameraModel, ContextParams, ImageOutput, resolve_context};
use serde_json::Value;
use support::outline;
use support::{
    MODEL, RECIPE_REFERENCE, REFERENCE, beyond_ulp_count, bitwise_differences, camera_data_poses,
    camera_modes, check_camera_data, compare_reference, create_context, max_ulp, model_ints,
    output_hash, precomputed_rays, render_and_read, render_fields, rgb_of, scene_fixture, session,
};

#[test]
fn reference_fixture_and_diff_surface() {
    let reference = parse_reference(REFERENCE).expect("参考夹具解析");
    assert_eq!(reference.task_id, "2026-10-25-render-bvh-s7b1-depth-seg");
    assert_eq!(reference.recipe_fast_math, None);
    let recipe = parse_reference(RECIPE_REFERENCE).expect("同配方参考解析");
    assert_eq!(recipe.task_id, "2026-10-25-render-bvh-s7b1-kernel");
    assert_eq!(recipe.recipe_fast_math, Some(false));
    for (left, right) in reference.cameras.iter().zip(&recipe.cameras) {
        assert_eq!(left.depth_bits.len(), right.depth_bits.len());
        assert_eq!(left.seg_pairs, right.seg_pairs, "两份参考 seg 必须相同");
    }
    assert_eq!(reference.total_rays, 120_000);
    assert_eq!(reference.total_pixels(), 120_000);
    assert_eq!(reference.cam_res, vec![[320, 240], [240, 180]]);
    assert_eq!(reference.depth_adr, vec![0, 76_800]);
    assert_eq!(reference.seg_adr, vec![0, 76_800]);
    assert!(reference.depth_encoding.contains("binary32 bits"));
    assert!(reference.seg_encoding.contains("object_id"));
    let expected = [(320usize, 240usize, 16_995usize), (240, 180, 27_498)];
    for (index, (width, height, zeros)) in expected.into_iter().enumerate() {
        let camera = reference.camera(index).expect("相机");
        assert_eq!((camera.width, camera.height), (width, height));
        assert_eq!(
            (camera.pixels, camera.depth_bits.len()),
            (width * height, width * height)
        );
        assert_eq!(camera.seg_pairs.len(), width * height);
        assert_eq!(camera.depth_zero_pixels, zeros);
        assert_eq!(
            camera.depth_bits.iter().filter(|bits| **bits == 0).count(),
            zeros
        );
        // 参考只出现 (geom_id, 5) 与背景 (-1,-1)（mjOBJ_GEOM=5）；深度是正平面米或 0=miss。
        for pair in &camera.seg_unique {
            assert!(pair[1] == 5 || *pair == [-1, -1], "未预期的分割对 {pair:?}");
        }
        assert!(
            camera.seg_unique.contains(&[-1, -1]),
            "参考应包含背景 (-1,-1)"
        );
        for value in camera.depth_values() {
            assert!(
                value == 0.0 || (value.is_finite() && value > 0.0 && value < 125.0),
                "深度 {value} 超出 (0, zfar)"
            );
        }
    }
    // 相机 data 位姿：两相机、模式 fixed/targetbody、复算误差在限内。
    assert_eq!(reference.camera_data.len(), 2);
    assert_eq!(reference.camera_data[0].mode, 0);
    assert_eq!(reference.camera_data[1].mode, 3);
    assert!(reference.camera_data[1].target_bodyid >= 0);
    // depth/seg diff 报告：首个不匹配位置与 ULP 计数。
    let camera = reference.camera(0).expect("相机 0").clone();
    let actual = camera.depth_values();
    let clean = diff_depth(&actual, &camera).expect("diff");
    assert_eq!(
        (clean.mismatched, clean.max_ulp, clean.compared),
        (0, 0, camera.pixels)
    );
    let mut perturbed = actual.clone();
    let mut nonzero = perturbed
        .iter()
        .enumerate()
        .filter(|(_, value)| **value != 0.0);
    let first = nonzero.next().expect("非零深度").0;
    let second = nonzero.next().expect("第二个非零深度").0;
    perturbed[first] = f32::from_bits(perturbed[first].to_bits() + 1);
    perturbed[second] = f32::from_bits(perturbed[second].to_bits() + 8);
    let diff = diff_depth(&perturbed, &camera).expect("diff");
    assert_eq!((diff.mismatched, diff.beyond_ulp, diff.max_ulp), (2, 1, 8));
    assert!(diff.max_ulp > MAX_DEPTH_ULP);
    let mismatch = diff.first.expect("首个不匹配");
    assert_eq!(
        (mismatch.pixel, mismatch.x, mismatch.y),
        (first, first % camera.width, first / camera.width)
    );
    let mut seg_bad = camera.seg_pairs.clone();
    assert_eq!(diff_seg(&seg_bad, &camera).expect("seg diff").mismatched, 0);
    seg_bad[0] = [999, 5];
    let seg_diff = diff_seg(&seg_bad, &camera).expect("seg diff");
    assert_eq!(seg_diff.mismatched, 1);
    assert_eq!(seg_diff.first.expect("首个 seg 不匹配").pixel, 0);
    assert_eq!(ulp_distance(0x3f80_0000, 0x3f80_0001), 1);
    assert_eq!(ulp_distance(0x0000_0000, 0x8000_0000), 0);
}

#[test]
fn camera_data_poses_match_reference_and_fixed_quat_path() {
    let reference = parse_reference(REFERENCE).expect("参考夹具解析");
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let resolved = resolve_context(&fields, &ContextParams::default()).expect("context 解析");
    let poses = camera_data_poses(&reference, &model, &resolved);
    assert_eq!(poses.len(), model.ncam());
    for (position, camera_id) in resolved.plan.cameras.iter().enumerate() {
        let data = &reference.camera_data[position];
        assert_eq!(poses[*camera_id].xpos.map(f32::to_bits), data.origin_bits);
        assert_eq!(poses[*camera_id].xmat.map(f32::to_bits), data.xmat_bits);
    }
    // fixed 相机（mode 0）与 f32 `quat_to_mat(cam_quat)` 路径的差；TARGETBODY 只认 data。
    let modes = camera_modes(&fields);
    assert_eq!(modes.first(), Some(&0));
    let (fixed_max, _) = check_camera_data(&reference, &fields);
    assert!(
        fixed_max <= 1.0e-6,
        "fixed 相机 data xmat 与 quat 路径差 {fixed_max}"
    );
    // S0b 射线原点（camera_pixels）与 data cam_xpos 逐位一致由夹具转换脚本保证，
    // 这里再用 f32 解码核对一次。
    let quat_poses = CameraPose::from_fields(&fields).expect("f32 quat 位姿");
    for (index, pose) in quat_poses.iter().enumerate().take(2) {
        assert_eq!(
            pose.xpos.map(f32::to_bits),
            reference.camera_data[index].origin_bits
        );
    }
}

/// NVRTC 编译 + A′ 渲染门禁（无 GPU 也可；与 S7a 用例同口径归入 ignored 显式执行）。
#[test]
#[ignore = "需要 NVRTC 与 W15 头树（无 GPU 也可，但按窗口统一执行）"]
fn depth_seg_source_compiles_under_render_gate() {
    let artifact = compile_render_artifact(DEPTH_SEG_ENTRY, DEPTH_SEG_SOURCE, DEPTH_SEG_ENTRY)
        .expect("NVRTC 编译 + f32-render 门禁");
    assert_eq!(check_render_ptx(artifact.ptx(), DEPTH_SEG_ENTRY), Ok(()));
    assert!(artifact.ptx().contains(".visible .entry render_depth_seg"));
    // 源码是 S2b scene.cu 只读复用 + render_math.cu + render.cu 的拼接，因此 PTX 里同时
    // 存在 S2b 的 scene_bvh_ray 与本片 render_depth_seg 两个 entry（均不被额外发射）。
    assert!(artifact.ptx().contains(".visible .entry scene_bvh_ray"));
    assert_eq!(
        artifact.ptx().matches(".global .align 8 .f64").count(),
        2,
        "只允许 W15 两条死数据常量"
    );
    println!(
        "s7b1 ptx: source_sha256={} ptx_sha256={} ptx_bytes={}",
        artifact.source_sha256(),
        artifact.ptx_sha256(),
        artifact.ptx().len()
    );
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn depth_and_seg_match_frozen_reference_for_both_cameras() {
    let session = session();
    let fields = render_fields();
    let s0b = parse_reference(REFERENCE).expect("S0b fast-math 参考夹具");
    let recipe = parse_reference(RECIPE_REFERENCE).expect("同配方（fast_math=false）参考夹具");
    assert_eq!(recipe.task_id, "2026-10-25-render-bvh-s7b1-kernel");
    assert_eq!(recipe.recipe_fast_math, Some(false));
    // 两份参考的 seg 必须逐像素相同（Arch 裁决：seg 以 S0b 为源，且同配方参考不得改变几何判定）。
    for (left, right) in s0b.cameras.iter().zip(&recipe.cameras) {
        assert_eq!(left.seg_pairs, right.seg_pairs, "两份参考 seg 必须相同");
    }
    let context = create_context(&session, &fields, 1, true, true);
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = camera_data_poses(&s0b, &model, context.resolved());
    let rays = precomputed_rays(&session, &fields);
    assert_eq!(
        context.resolved().output.depth_adr,
        s0b.depth_adr,
        "输出偏移"
    );
    assert_eq!(context.resolved().output.seg_adr, s0b.seg_adr);
    let renderer = DepthSegRenderer::new(&session).expect("渲染内核编译");
    let (depth, seg) = render_and_read(&renderer, &fields, &model, &context, &poses, Some(&rays));
    // 验收口径：depth ≤4 ULP 对同配方参考；seg 逐像素全等（compare_reference 内断言）。
    let stats = compare_reference("recipe", &depth, &seg, &recipe);
    let beyond = beyond_ulp_count(&stats, &[0, 1, 2, 3, 4, 5, 6]);
    assert_eq!(
        beyond,
        0,
        "depth 超 {MAX_DEPTH_ULP} ULP 的像素必须为 0（max={}）",
        max_ulp(&stats)
    );
    let recipe_max_ulp = max_ulp(&stats);
    // criterion v2.2 clause 2: the S0b fast-math reference is a DIFFERENT compilation form, so the
    // B-class premise "same compilation form" does not hold after the a-order refix. This
    // cross-recipe comparison is RECORD-ONLY (observation; never an acceptance basis). The one
    // fail-closed rule kept: every >4 ULP pixel must belong to the frozen recipe list
    // (outside-list bitwise equality is already guaranteed by the recipe arm above).
    let known = compare_reference("s0b-fast-math", &depth, &seg, &s0b);
    let known_beyond = beyond_ulp_count(&known, &[0, 1, 2, 3, 4, 5, 6]);
    let known_histograms: Vec<[usize; 6]> = known.iter().map(|camera| camera.histogram).collect();
    println!(
        "s7b1[s0b-fast-math] record-only (B-class premise not met): cam0 hist={:?} cam1 hist={:?} \
         beyond4={} max_ulp={}",
        known_histograms[0],
        known_histograms[1],
        known_beyond,
        max_ulp(&known)
    );
    if outline::enabled() {
        let frozen = outline::list(outline::Arm::Recipe);
        let mut beyond_total = 0usize;
        let mut beyond_outside = 0usize;
        for camera in &s0b.cameras {
            let base = camera.depth_adr as usize;
            for pixel in 0..camera.pixels {
                let ulp = ulp_distance(depth[base + pixel].to_bits(), camera.depth_bits[pixel]);
                if ulp <= MAX_DEPTH_ULP {
                    continue;
                }
                beyond_total += 1;
                if camera.index != 1 || !frozen.entries.contains_key(&pixel) {
                    beyond_outside += 1;
                }
            }
        }
        assert_eq!(
            beyond_outside, 0,
            "criterion v2.2: S0b cross-recipe >4 ULP pixels must all be inside the frozen recipe \
             list (outside={beyond_outside} of {beyond_total})"
        );
        println!(
            "s7b1[s0b-fast-math] beyond4_subset_of_frozen_list=true beyond4={beyond_total} \
             outside={beyond_outside} list_rows={}",
            frozen.rows
        );
    }
    // Reference-to-reference cross-recipe delta stays a fixture assertion (it does not involve the
    // product): recipe (form A) vs S0b over both cameras.
    let reference_histograms: Vec<[usize; 6]> = recipe
        .cameras
        .iter()
        .zip(&s0b.cameras)
        .map(|(left, right)| {
            let mut histogram = [0usize; 6];
            for (a, b) in left.depth_bits.iter().zip(&right.depth_bits) {
                let ulp = ulp_distance(*a, *b);
                histogram[if ulp > 4 { 5 } else { ulp as usize }] += 1;
            }
            histogram
        })
        .collect();
    assert_eq!(
        reference_histograms,
        vec![[57726, 17681, 1393, 0, 0, 0], [39136, 3677, 382, 3, 2, 0]],
        "参考↔参考跨配方已知偏差直方图必须与夹具 cross_recipe_vs_s0b 一致"
    );
    let geom_type = model_ints(
        &serde_json::from_str::<Value>(MODEL).expect("模型"),
        "geom_type",
    );
    let (mut mesh_pixels, mut primitive_pixels) = (0usize, 0usize);
    for pair in &seg {
        if pair[1] != 5 {
            continue;
        }
        if matches!(geom_type.get(pair[0] as usize), Some(7 | 8)) {
            mesh_pixels += 1;
        } else {
            primitive_pixels += 1;
        }
    }
    assert!(
        mesh_pixels > 0 && primitive_pixels > 0,
        "primitive+mesh 同池"
    );
    assert!(
        rgb_of(&context).iter().all(|value| *value == 0),
        "RGB 不应被写入"
    );
    println!(
        "s7b1 reference: rays={} recipe_max_ulp={recipe_max_ulp} known_cross_recipe_beyond4={known_beyond} \
         mesh_pixels={mesh_pixels} primitive_pixels={primitive_pixels} source_sha256={} ptx_sha256={}",
        rays.len(),
        renderer.kernel().artifact().source_sha256(),
        renderer.kernel().artifact().ptx_sha256()
    );
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn dynamic_rays_render_bitwise_identical_to_precomputed() {
    let session = session();
    let fields = render_fields();
    let reference = parse_reference(REFERENCE).expect("参考夹具");
    let context = create_context(&session, &fields, 1, true, true);
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = camera_data_poses(&reference, &model, context.resolved());
    let rays = precomputed_rays(&session, &fields);
    let renderer = DepthSegRenderer::new(&session).expect("渲染内核编译");
    let (depth_a, seg_a) =
        render_and_read(&renderer, &fields, &model, &context, &poses, Some(&rays));
    let (depth_b, seg_b) = render_and_read(&renderer, &fields, &model, &context, &poses, None);
    assert_eq!(
        bitwise_differences(&depth_a, &depth_b),
        0,
        "dynamic 与预计算深度位型必须相同"
    );
    assert_eq!(seg_a, seg_b, "dynamic 与预计算分割必须相同");
    println!(
        "s7b1 dynamic: rays={} seg_equal=true hash={:016x}",
        rays.len(),
        output_hash(&depth_a, &seg_a)
    );
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn two_worlds_do_not_cross_hit() {
    let session = session();
    let fields = render_fields();
    let reference = parse_reference(REFERENCE).expect("参考夹具");
    let context = create_context(&session, &fields, 2, true, true);
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = camera_data_poses(&reference, &model, context.resolved());
    let rays = precomputed_rays(&session, &fields);
    let renderer = DepthSegRenderer::new(&session).expect("渲染内核编译");
    let (depth, seg) = render_and_read(&renderer, &fields, &model, &context, &poses, Some(&rays));
    let layout = &context.resolved().output;
    assert_eq!(
        (depth.len(), seg.len()),
        (2 * layout.depth_pixels, 2 * layout.seg_slots)
    );
    let world0_depth = &depth[..layout.depth_pixels];
    assert!(
        depth[layout.depth_pixels..]
            .iter()
            .all(|value| *value == 0.0),
        "world 1 必须全 miss"
    );
    assert!(
        seg[layout.seg_slots..].iter().all(|pair| *pair == [-1, -1]),
        "world 1 分割必须全背景"
    );
    // 双 world 不跨：world 0 与单 world 渲染逐位一致（比对上参考更严格的自一致断言）。
    let single = create_context(&session, &fields, 1, true, true);
    let single_poses = camera_data_poses(&reference, &model, single.resolved());
    let (single_depth, single_seg) = render_and_read(
        &renderer,
        &fields,
        &model,
        &single,
        &single_poses,
        Some(&rays),
    );
    assert_eq!(
        bitwise_differences(world0_depth, &single_depth),
        0,
        "world 0 必须与单 world 渲染逐位一致"
    );
    assert_eq!(
        &seg[..layout.seg_slots],
        single_seg.as_slice(),
        "world 0 分割一致"
    );
    println!(
        "s7b1 two-world: world0_is_bitwise_single=true world1_all_miss=true depth_pixels={} seg_slots={}",
        layout.depth_pixels, layout.seg_slots
    );
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn mesh_refit_changes_pixels_and_repeat_runs_are_bitwise_identical() {
    let session = session();
    let fields = render_fields();
    let reference = parse_reference(REFERENCE).expect("参考夹具");
    let mut context = create_context(&session, &fields, 1, true, true);
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = camera_data_poses(&reference, &model, context.resolved());
    let rays = precomputed_rays(&session, &fields);
    let renderer = DepthSegRenderer::new(&session).expect("渲染内核编译");
    let (baseline_depth, baseline_seg) =
        render_and_read(&renderer, &fields, &model, &context, &poses, Some(&rays));
    // 网格顶点沿 +y 平移；`set_points` 不自动 refit（S3 语义），显式 refit 后重渲染。
    for (index, mesh) in scene_fixture().meshes.iter().enumerate() {
        let moved = mesh
            .vertices
            .iter()
            .map(|vertex| [vertex[0], vertex[1] + 0.5, vertex[2]])
            .collect::<Vec<_>>();
        context.set_mesh_points(index, &moved).expect("set_points");
        context.refit_mesh(index).expect("显式 refit");
    }
    let (depth_a, seg_a) =
        render_and_read(&renderer, &fields, &model, &context, &poses, Some(&rays));
    let (depth_b, seg_b) =
        render_and_read(&renderer, &fields, &model, &context, &poses, Some(&rays));
    let seg_changed = seg_a
        .iter()
        .zip(&baseline_seg)
        .filter(|(a, b)| a != b)
        .count();
    let changed = bitwise_differences(&depth_a, &baseline_depth) + seg_changed;
    assert!(changed > 0, "set_points+refit 后必须有像素变化");
    assert_eq!(
        bitwise_differences(&depth_a, &depth_b),
        0,
        "两次渲染必须逐位一致"
    );
    assert_eq!(seg_a, seg_b);
    println!(
        "s7b1 refit: changed_pixels={changed} deterministic=true hash={:016x}",
        output_hash(&depth_a, &seg_a)
    );
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn disabled_depth_and_seg_keep_s6b_rejections() {
    let session = session();
    let fields = render_fields();
    let reference = parse_reference(REFERENCE).expect("参考夹具");
    let context = create_context(&session, &fields, 1, false, false);
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = camera_data_poses(&reference, &model, context.resolved());
    let rays = precomputed_rays(&session, &fields);
    let renderer = DepthSegRenderer::new(&session).expect("渲染内核编译");
    renderer
        .render(&context, &fields, &model, &poses, Some(&rays), true)
        .expect("禁用 depth/seg 仍可发射（占位缓冲，不写）");
    let rejected = |result: Result<(), OwnerError>| {
        matches!(
            result,
            Err(OwnerError::Transfer(TransferError::Input(
                InputError::DisabledOutput
            )))
        )
    };
    for output in [ImageOutput::Depth, ImageOutput::Segmentation] {
        assert!(
            rejected(context.outputs().byte_len(output).map(|_| ())),
            "{output:?} 字节数必须拒绝"
        );
        assert!(
            rejected(context.outputs().read(output).map(|_| ())),
            "{output:?} 读取必须拒绝"
        );
    }
    assert!(
        rgb_of(&context).iter().all(|value| *value == 0),
        "RGB 本片不写"
    );
    println!("s7b1 disabled: depth/seg 保持 DisabledOutput，RGB 本片不写");
}
