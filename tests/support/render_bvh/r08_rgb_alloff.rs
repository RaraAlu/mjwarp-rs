//! R08-RGB QA O1：某相机三通道全关（cam1 rgb/depth/seg 均 `[true,false]`）。
//! 覆盖内核 `render_megakernel.cu:131` 的 `!do_depth && !do_seg && !do_rgb` all-off
//! early-return；修复前 host 全局布尔填充会让关闭通道按 `adr=-1` 写（latent 越界）。
#![allow(dead_code)]

use mjwarp_rs::render::CameraModel;
use mjwarp_rs::render::camera::CameraActive;
use mjwarp_rs::render::context::RenderFlagSpec;
use mjwarp_rs::render::kernel::MegakernelRenderer;

use super::r08::{self, OutputBuffers};
use super::s7b2::{self, parse_rgb_fast_math_false_reference, recipe_depth_seg, render_fields};

pub fn cam1_all_channels_off() {
    let session = s7b2::session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let rgb_ref = parse_rgb_fast_math_false_reference();
    let recipe = recipe_depth_seg();
    let renderer = MegakernelRenderer::new(&session).expect("S7b2 内核编译");
    let on0 = [true, false];
    let params = r08::targets_params(
        CameraActive::All,
        Some(RenderFlagSpec::List(&on0)),
        Some(RenderFlagSpec::List(&on0)),
        Some(RenderFlagSpec::List(&on0)),
    );
    let context = s7b2::create_context(&session, &fields, &params);
    let resolved = context.resolved();
    assert_eq!(resolved.output.rgb_adr, vec![0, -1], "cam1 RGB 未启用");
    assert_eq!(resolved.output.depth_adr, vec![0, -1], "cam1 depth 未启用");
    assert_eq!(resolved.output.seg_adr, vec![0, -1], "cam1 seg 未启用");
    let rays = r08::rays_for_plan(&session, &fields, resolved);
    let poses = r08::poses_for_plan(&recipe, &model, resolved);
    let mut hashes = Vec::new();
    for _ in 0..2 {
        renderer
            .render(&context, &fields, &model, &poses, &rays)
            .expect("cam1 三通道全关渲染发射");
        let buffers = OutputBuffers::read_enabled(&context);
        let views = buffers.views(resolved);
        assert_eq!(
            views.rgb_packed(0).expect("cam0 rgb"),
            rgb_ref.cameras[0].packed.as_slice(),
            "cam0 RGB 位级"
        );
        assert_eq!(
            r08::depth_bits(views.depth_raw(0).expect("cam0 depth")),
            recipe.cameras[0].depth_bits,
            "cam0 depth 位等"
        );
        assert_eq!(
            views.segmentation_raw(0).expect("cam0 seg"),
            recipe.cameras[0].seg_pairs.as_slice(),
            "cam0 seg 全等"
        );
        assert!(
            views.rgb_packed(1).is_err()
                && views.depth_raw(1).is_err()
                && views.segmentation_raw(1).is_err(),
            "cam1 三通道全关须全部拒绝"
        );
        hashes.push(r08::sha256_buffers(&buffers));
    }
    assert_eq!(hashes[0], hashes[1], "cam1 三通道全关两跑 SHA-256 相等");
    println!(
        "r08[rgb-hetero] cam1 三通道全关：cam0 三通道对参考、cam1 全拒绝（all-off early-return）"
    );
}
