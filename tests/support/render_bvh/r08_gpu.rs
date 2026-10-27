//! R08-S2 GPU 窗口用例实现（`tests/render_r08_cameras.rs` 的 `#[ignore]` 用例只调用本模块）。
//! 需要 GPU + NVRTC；窗口内以 `--ignored --test-threads=1` 执行。参考一律复用 S0b/S7b 冻结件。
#![allow(dead_code)]

use mjwarp_rs::render::camera::CameraActive;
use mjwarp_rs::render::context::RenderFlagSpec;
use mjwarp_rs::render::kernel::output::{diff_depth, diff_seg};
use mjwarp_rs::render::kernel::{DepthSegRenderer, MegakernelRenderer};
use mjwarp_rs::render::{CameraModel, ContextParams};
use sha2::{Digest, Sha256};

use super::r08::{self, OutputBuffers};
use super::s7b1;
use super::s7b2;
use super::s7b2::{
    default_params, parse_rgb_fast_math_false_reference, recipe_depth_seg, render_fields,
};

/// 渲染矩阵四形态（All / Mask / Names / Indices）逐相机对 S7b 同配方冻结参考。
pub fn matrix_render_matches_reference() {
    let session = s7b2::session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let rgb_ref = parse_rgb_fast_math_false_reference();
    let recipe = recipe_depth_seg();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let names = r08::camera_names(&fields);
    let mask = [true, false];
    let indices = [1i32, 0];
    let cases: [(&str, CameraActive<'_>, Vec<usize>); 4] = [
        ("all", CameraActive::All, vec![0, 1]),
        ("mask_first", CameraActive::Mask(&mask), vec![0]),
        ("names_all", CameraActive::Names(&names), vec![0, 1]),
        (
            "indices_reversed",
            CameraActive::Indices(&indices),
            vec![1, 0],
        ),
    ];
    for (label, active, order) in cases {
        let params = r08::render_params(active);
        let context = s7b2::create_context(&session, &fields, &params);
        let resolved = context.resolved();
        assert_eq!(resolved.plan.cameras, order, "{label} 激活顺序");
        let rays = r08::rays_for_plan(&session, &fields, resolved);
        let poses = r08::poses_for_plan(&recipe, &model, resolved);
        renderer
            .render(&context, &fields, &model, &poses, &rays)
            .expect("渲染发射");
        let buffers = OutputBuffers::read_enabled(&context);
        let views = buffers.views(resolved);
        let stats = r08::assert_views_match_reference(&views, &rgb_ref, &recipe);
        println!(
            "r08[matrix] {label}: cameras={} rgb_adr={:?} depth_adr={:?} seg_adr={:?} stats={stats:?} sha256={}",
            resolved.plan.cameras.len(),
            resolved.output.rgb_adr,
            resolved.output.depth_adr,
            resolved.output.seg_adr,
            r08::sha256_buffers(&buffers)
        );
    }
}

/// 同输入两次运行输出 SHA-256 相同（RGB + depth + seg）。
pub fn two_runs_sha256_identical() {
    let session = s7b2::session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let recipe = recipe_depth_seg();
    let context = s7b2::create_context(&session, &fields, &default_params());
    let resolved = context.resolved();
    let rays = r08::rays_for_plan(&session, &fields, resolved);
    let poses = r08::poses_for_plan(&recipe, &model, resolved);
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let mut hashes = Vec::new();
    for run in 0..2 {
        renderer
            .render(&context, &fields, &model, &poses, &rays)
            .expect("渲染发射");
        let buffers = OutputBuffers::read_enabled(&context);
        let hash = r08::sha256_buffers(&buffers);
        println!("r08[repeat] run{run}: sha256={hash}");
        hashes.push(hash);
    }
    assert_eq!(hashes[0], hashes[1], "同输入两次运行输出 SHA-256 必须相同");
}

fn render_raw(
    session: &mjwarp_rs::runtime::TransferSession,
    fields: &mjwarp_rs::model::render::RenderFields,
    model: &CameraModel,
    recipe: &mjwarp_rs::render::kernel::output::DepthSegReference,
    renderer: &MegakernelRenderer,
    params: &ContextParams<'_>,
) -> Vec<r08::RawCamera> {
    let context = s7b2::create_context(session, fields, params);
    let rays = r08::rays_for_plan(session, fields, context.resolved());
    let poses = r08::poses_for_plan(recipe, model, context.resolved());
    renderer
        .render(&context, fields, model, &poses, &rays)
        .expect("渲染发射");
    let buffers = OutputBuffers::read_enabled(&context);
    let resolved = context.resolved().clone();
    r08::raw_by_model_camera(&buffers.views(&resolved), &[0, 1])
}

/// 隔离：单相机子集 / 乱序选择渲染的逐相机区段与组合渲染**逐位相同**。
pub fn isolation_subset_and_reorder() {
    let session = s7b2::session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let recipe = recipe_depth_seg();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let combined = render_raw(
        &session,
        &fields,
        &model,
        &recipe,
        &renderer,
        &default_params(),
    );
    let mask0 = [true, false];
    let mask1 = [false, true];
    let reversed = [1i32, 0];
    let subsets: [(&str, CameraActive<'_>); 3] = [
        ("subset_cam0", CameraActive::Mask(&mask0)),
        ("subset_cam1", CameraActive::Mask(&mask1)),
        ("reorder", CameraActive::Indices(&reversed)),
    ];
    for (label, active) in subsets {
        let params = r08::render_params(active);
        let raw = render_raw(&session, &fields, &model, &recipe, &renderer, &params);
        for camera in 0..2 {
            if raw[camera].0.is_empty() {
                continue;
            }
            assert_eq!(
                raw[camera].0, combined[camera].0,
                "{label} cam{camera} RGB 逐位相同"
            );
            assert_eq!(
                raw[camera].1, combined[camera].1,
                "{label} cam{camera} depth 逐位相同"
            );
            assert_eq!(
                raw[camera].2, combined[camera].2,
                "{label} cam{camera} seg 全等"
            );
        }
        println!(
            "r08[isolate] {label}: cam0_rgb={} cam1_rgb={}（与组合渲染逐位相同）",
            raw[0].0.len(),
            raw[1].0.len()
        );
    }
}

/// 动态射线（`rays=None`）与预计算射线的 depth/seg **逐位相同**，且两者都对同配方参考位等/全等。
pub fn dynamic_rays_depth_seg() {
    let session = s7b2::session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let recipe = recipe_depth_seg();
    let context = s7b2::create_context(&session, &fields, &default_params());
    let resolved = context.resolved();
    let rays = r08::rays_for_plan(&session, &fields, resolved);
    let poses = r08::poses_for_plan(&recipe, &model, resolved);
    let renderer = DepthSegRenderer::new(&session).expect("S7b1 内核编译");
    let (pre_depth, pre_seg) =
        s7b1::render_and_read(&renderer, &fields, &model, &context, &poses, Some(&rays));
    let (dyn_depth, dyn_seg) =
        s7b1::render_and_read(&renderer, &fields, &model, &context, &poses, None);
    assert_eq!(
        r08::depth_bits(&pre_depth),
        r08::depth_bits(&dyn_depth),
        "动态射线 depth 须逐位相同"
    );
    assert_eq!(pre_seg, dyn_seg, "动态射线 seg 须全等");
    for camera in &recipe.cameras {
        let base = camera.depth_adr as usize;
        let seg_base = camera.seg_adr as usize;
        for (label, depth, seg) in [
            ("precomputed", pre_depth.as_slice(), pre_seg.as_slice()),
            ("dynamic", dyn_depth.as_slice(), dyn_seg.as_slice()),
        ] {
            let depth_diff =
                diff_depth(&depth[base..base + camera.pixels], camera).expect("depth diff");
            let seg_diff =
                diff_seg(&seg[seg_base..seg_base + camera.pixels], camera).expect("seg diff");
            assert_eq!(
                depth_diff.mismatched, 0,
                "cam{} {label} depth 位等",
                camera.index
            );
            assert_eq!(
                seg_diff.mismatched, 0,
                "cam{} {label} seg 全等",
                camera.index
            );
        }
    }
    let mut hasher = Sha256::new();
    for value in &pre_depth {
        hasher.update(value.to_bits().to_le_bytes());
    }
    for pair in &pre_seg {
        for value in pair {
            hasher.update(value.to_le_bytes());
        }
    }
    println!(
        "r08[dynamic] 预计算 vs 动态 depth/seg 逐位相同，sha256={:x}",
        hasher.finalize()
    );
}

/// 异构 depth/seg 通道：cam0 仅 depth、cam1 仅 seg；逐相机对参考，未启用通道读取拒绝。
pub fn heterogeneous_depth_seg_channels() {
    let session = s7b2::session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let recipe = recipe_depth_seg();
    let depth = [true, false];
    let seg = [false, true];
    let params = r08::targets_params(
        CameraActive::All,
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::List(&depth)),
        Some(RenderFlagSpec::List(&seg)),
    );
    let resolved = r08::resolved(&fields, &params);
    assert_eq!(resolved.output.depth_adr, vec![0, -1], "cam1 depth 未启用");
    assert_eq!(resolved.output.seg_adr, vec![-1, 0], "cam0 seg 未启用");
    let context = s7b2::create_context(&session, &fields, &params);
    let rays = r08::rays_for_plan(&session, &fields, context.resolved());
    let poses = r08::poses_for_plan(&recipe, &model, context.resolved());
    let renderer = DepthSegRenderer::new(&session).expect("S7b1 内核编译");
    let _ = s7b1::render_and_read(&renderer, &fields, &model, &context, &poses, Some(&rays));
    let buffers = OutputBuffers::read_enabled(&context);
    let views = buffers.views(context.resolved());
    assert_eq!(
        r08::depth_bits(views.depth_raw(0).expect("cam0 depth")),
        recipe.cameras[0].depth_bits
    );
    assert_eq!(
        views.segmentation_raw(1).expect("cam1 seg"),
        recipe.cameras[1].seg_pairs.as_slice()
    );
    assert!(views.depth_raw(1).is_err(), "cam1 depth 未启用须拒绝");
    assert!(views.segmentation_raw(0).is_err(), "cam0 seg 未启用须拒绝");
    println!(
        "r08[hetero] depth_adr={:?} seg_adr={:?} 逐相机对参考与未启用拒绝均成立",
        resolved.output.depth_adr, resolved.output.seg_adr
    );
}
