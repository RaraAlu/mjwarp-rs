#![cfg(feature = "cuda-probe")]
//! 渲染/BVH R08-S2 验收：多相机选择矩阵 + 异构分辨率/通道端到端（1 验证窗、无新捕获）。
//! 参考复用 S0b/S7b 同配方冻结夹具；host 用例默认执行，GPU 用例默认 ignore（窗口内执行）：
//! cargo test --locked --features cuda-probe --test render_r08_cameras -- --ignored --test-threads=1

#[path = "support/render_bvh/r08.rs"]
mod r08;
#[path = "support/render_bvh/r08_gpu.rs"]
mod r08_gpu;
#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;

use mjwarp_rs::render::camera::CameraActive;
use mjwarp_rs::render::context::RenderFlagSpec;
use mjwarp_rs::render::{CameraModel, CameraViewError, ImageOutput, OutputValues};
use r08::OutputBuffers;
use s7b2::{parse_rgb_fast_math_false_reference, recipe_depth_seg, render_fields};

#[test]
fn selection_matrix_five_forms_resolve_to_layouts() {
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let names = r08::camera_names(&fields);
    let mask = [true, false];
    let indices = [1i32, 0];
    let cases: [(&str, CameraActive<'_>, Vec<usize>); 5] = [
        ("none_all", CameraActive::All, vec![0, 1]),
        ("mask_first", CameraActive::Mask(&mask), vec![0]),
        ("names_all", CameraActive::Names(&names), vec![0, 1]),
        (
            "indices_reversed",
            CameraActive::Indices(&indices),
            vec![1, 0],
        ),
        ("empty", CameraActive::Empty, Vec::new()),
    ];
    for (label, active, order) in cases {
        let resolved = r08::resolved(&fields, &r08::render_params(active));
        assert_eq!(
            resolved.plan.cameras, order,
            "{label} 激活顺序须 = CameraPlan.cameras"
        );
        assert_eq!(resolved.nrender(), order.len(), "{label} nrender");
        let expected = r08::expected_adr(&model, &order);
        assert_eq!(
            (
                resolved.output.rgb_adr.clone(),
                resolved.output.depth_adr.clone(),
                resolved.output.seg_adr.clone()
            ),
            (
                expected[0].clone(),
                expected[1].clone(),
                expected[2].clone()
            ),
            "{label} 布局 adr 顺序"
        );
        let total: usize = order
            .iter()
            .map(|camera| {
                let resolution = model.camera(*camera).expect("相机").resolution;
                resolution[0] as usize * resolution[1] as usize
            })
            .sum();
        assert_eq!(resolved.output.total_rays, total, "{label} total_rays");
        let buffers = OutputBuffers::default();
        let views = buffers.views(&resolved);
        assert_eq!(views.camera_count(), order.len());
        for (position, camera) in order.iter().enumerate() {
            assert_eq!(views.camera_id(position).expect("相机 id"), *camera as i32);
            let resolution = model.camera(*camera).expect("相机").resolution;
            assert_eq!(views.resolution(position).expect("分辨率"), resolution);
        }
        if order.is_empty() {
            assert_eq!(
                (
                    resolved.output.total_rays,
                    resolved.output.rgb_pixels,
                    resolved.output.depth_pixels,
                    resolved.output.seg_pixels
                ),
                (0, 0, 0, 0)
            );
            assert_eq!(
                views.rgb(0).unwrap_err(),
                CameraViewError::CameraIndex {
                    index: 0,
                    cameras: 0
                }
            );
        }
        println!(
            "r08[host] {label}: cameras={order:?} rgb_adr={:?}",
            resolved.output.rgb_adr
        );
    }
}

#[test]
fn heterogeneous_channels_layout_views_and_rejections() {
    let fields = render_fields();
    let rgb_ref = parse_rgb_fast_math_false_reference();
    let recipe = recipe_depth_seg();
    let rgb = [true, false];
    let seg = [true, false];
    let params = r08::targets_params(
        CameraActive::All,
        Some(RenderFlagSpec::List(&rgb)),
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::List(&seg)),
    );
    let resolved = r08::resolved(&fields, &params);
    assert_eq!(resolved.output.rgb_adr, vec![0, -1], "cam1 RGB 未启用");
    assert_eq!(resolved.output.seg_adr, vec![0, -1], "cam1 seg 未启用");
    assert_eq!(
        resolved.output.depth_adr,
        vec![0, 76_800],
        "depth 两相机都启用"
    );
    assert_eq!(
        (
            resolved.output.rgb_pixels,
            resolved.output.depth_pixels,
            resolved.output.seg_pixels
        ),
        (76_800, 120_000, 76_800)
    );
    let buffers = r08::reference_buffers_for_layout(&[0, 1], &resolved.output, &rgb_ref, &recipe);
    let views = buffers.views(&resolved);
    assert_eq!(
        views.rgb_packed(0).expect("cam0 rgb"),
        rgb_ref.cameras[0].packed.as_slice()
    );
    assert_eq!(
        views.rgb_packed(1).unwrap_err(),
        CameraViewError::ChannelDisabled {
            channel: ImageOutput::Rgb,
            camera_index: 1
        }
    );
    assert_eq!(
        r08::depth_bits(views.depth_raw(0).expect("cam0 depth")),
        recipe.cameras[0].depth_bits
    );
    assert_eq!(
        r08::depth_bits(views.depth_raw(1).expect("cam1 depth")),
        recipe.cameras[1].depth_bits
    );
    assert_eq!(
        views.segmentation_raw(0).expect("cam0 seg"),
        recipe.cameras[0].seg_pairs.as_slice()
    );
    assert_eq!(
        views.segmentation(1).unwrap_err(),
        CameraViewError::ChannelDisabled {
            channel: ImageOutput::Segmentation,
            camera_index: 1
        }
    );
    assert!(views.depth(0, 1.0).is_ok() && views.depth(1, 1.0).is_ok());
    println!(
        "r08[host] 异构通道：rgb_adr={:?} depth_adr={:?} seg_adr={:?}；cam1 rgb/seg 读取已拒绝",
        resolved.output.rgb_adr, resolved.output.depth_adr, resolved.output.seg_adr
    );
}

#[test]
fn output_values_adapter_builds_whole_buffers() {
    let fields = render_fields();
    let rgb_ref = parse_rgb_fast_math_false_reference();
    let recipe = recipe_depth_seg();
    let resolved = r08::resolved(&fields, &r08::render_params(CameraActive::All));
    let buffers = r08::reference_buffers(&[0, 1], &rgb_ref, &recipe);
    let values = vec![
        OutputValues::Rgb(buffers.rgb.clone().expect("rgb")),
        OutputValues::Depth(buffers.depth.clone().expect("depth")),
        OutputValues::Seg(buffers.seg.clone().expect("seg")),
    ];
    let rebuilt = OutputBuffers::from_values(&values);
    assert_eq!(rebuilt, buffers, "OutputValues → 整缓冲须逐通道保持");
    let views = rebuilt.views(&resolved);
    assert_eq!(
        views.rgb_packed(0).expect("cam0 rgb"),
        rgb_ref.cameras[0].packed.as_slice()
    );
    assert_eq!(
        views.segmentation_raw(1).expect("cam1 seg"),
        recipe.cameras[1].seg_pairs.as_slice()
    );
    assert_eq!(views.camera_count(), 2);
    let partial = OutputBuffers::from_values(&values[..1]);
    let partial_views = partial.views(&resolved);
    assert_eq!(partial_views.rgb_packed(1).expect("cam1 rgb").len(), 43_200);
    assert_eq!(
        partial_views.depth_raw(0).unwrap_err(),
        CameraViewError::BufferMissing {
            channel: ImageOutput::Depth
        }
    );
    assert_eq!(
        partial_views.segmentation(1).unwrap_err(),
        CameraViewError::BufferMissing {
            channel: ImageOutput::Segmentation
        }
    );
}

#[test]
fn activation_order_is_position_not_model_id() {
    let fields = render_fields();
    let rgb_ref = parse_rgb_fast_math_false_reference();
    let recipe = recipe_depth_seg();
    let indices = [1i32, 0];
    let resolved = r08::resolved(
        &fields,
        &r08::render_params(CameraActive::Indices(&indices)),
    );
    assert_eq!(resolved.plan.cameras, vec![1, 0]);
    assert_eq!(resolved.output.rgb_adr, vec![0, 43_200]);
    let buffers = r08::reference_buffers(&[1, 0], &rgb_ref, &recipe);
    let views = buffers.views(&resolved);
    assert_eq!(
        (
            views.camera_id(0).expect("id"),
            views.camera_id(1).expect("id")
        ),
        (1, 0)
    );
    assert_eq!(views.resolution(0).expect("分辨率"), [240, 180]);
    assert_eq!(
        views.rgb_packed(0).expect("cam1 rgb"),
        rgb_ref.cameras[1].packed.as_slice()
    );
    assert_eq!(
        views.rgb_packed(1).expect("cam0 rgb"),
        rgb_ref.cameras[0].packed.as_slice()
    );
    assert_eq!(
        views.segmentation_raw(0).expect("cam1 seg"),
        recipe.cameras[1].seg_pairs.as_slice()
    );
    println!(
        "r08[host] 激活顺序 [1,0]：camera_id=(1,0)，rgb_adr={:?}",
        resolved.output.rgb_adr
    );
}

#[test]
fn empty_selection_context_and_view_semantics() {
    let fields = render_fields();
    let resolved = r08::resolved(&fields, &r08::render_params(CameraActive::Empty));
    assert_eq!(resolved.nrender(), 0);
    assert_eq!(
        (
            resolved.output.total_rays,
            resolved.output.rgb_pixels,
            resolved.output.depth_pixels
        ),
        (0, 0, 0)
    );
    assert!(resolved.output.rgb_adr.is_empty());
    assert_eq!(
        (resolved.output.seg_slots, resolved.output.aa_accum_width),
        (1, 1)
    );
    assert_eq!(resolved.output.rgb_buffer_bytes(1).expect("字节"), 0);
    let buffers = OutputBuffers::default();
    let views = buffers.views(&resolved);
    assert_eq!(views.camera_count(), 0);
    assert_eq!(
        views.rgb(0).unwrap_err(),
        CameraViewError::CameraIndex {
            index: 0,
            cameras: 0
        }
    );
    // 上游 `cam_active=[]` 的渲染发射是 `wp.launch(dim=0)` no-op；产品 renderer 目前对
    // `nrender==0` 返回 `InvalidArray`（S7b3 未实现项），本片不覆盖发射路径，待 Arch 裁定。
    println!(
        "r08[host] 空选择：0 相机/0 射线/无缓冲，视图越界读取已拒绝；发射路径未覆盖（产品当前拒绝 nrender=0）"
    );
}

#[test]
#[ignore = "需要 GPU + NVRTC（窗口内执行）"]
fn gpu_matrix_render_matches_frozen_reference() {
    r08_gpu::matrix_render_matches_reference();
}

#[test]
#[ignore = "需要 GPU + NVRTC（窗口内执行）"]
fn gpu_two_runs_sha256_identical() {
    r08_gpu::two_runs_sha256_identical();
}

#[test]
#[ignore = "需要 GPU + NVRTC（窗口内执行）"]
fn gpu_isolation_subset_and_reorder_bitwise() {
    r08_gpu::isolation_subset_and_reorder();
}

#[test]
#[ignore = "需要 GPU + NVRTC（窗口内执行）"]
fn gpu_dynamic_rays_depth_seg_bitwise_matches_precomputed() {
    r08_gpu::dynamic_rays_depth_seg();
}

#[test]
#[ignore = "需要 GPU + NVRTC（窗口内执行）"]
fn gpu_heterogeneous_depth_seg_channels() {
    r08_gpu::heterogeneous_depth_seg_channels();
}
