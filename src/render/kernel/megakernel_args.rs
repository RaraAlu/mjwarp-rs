//! R06-S1a：`megakernel.rs`（450/450 行死线）的 81 槽实参装配段（零行为拆分）。
//!
//! 本文件由 `megakernel.rs` 的「81 槽实参（顺序 = S7a `MEGAKERNEL_SLOTS`）」段原样迁出：
//! 实参顺序、元素类型、形状（含 1 元素占位与 `mesh_slot_shape` 口径）逐项不变；不参与内核
//! 源码拼接，`.cu` 零改动 ⇒ 内核源与 PTX 不变。调用方：`megakernel.rs::render_impl`。

use crate::render::context::RenderContext;
use crate::render::kernel::megakernel_arrays::{array, to_i32, use_shadows_arg};
use crate::render::kernel::world_arrays::{
    camera_pose_shape, light_type_shape, shared_matrix_shape, shared_shape,
};
use crate::runtime::launcher::{LaunchBounds2, LauncherError, RenderArg, RenderElem, RenderScalar};

use super::flex_channel::{
    flex_edge_shape, flex_geom_shape, flex_group_root_shape, flex_model_shape, flex_vertadr_shape,
    flexvert_xpos_shape,
};
use super::megakernel_uploads::{MegakernelUploads, mesh_slot_shape};

/// 81 槽实参装配的输入视图（原 `render_impl` 中段前已算好的局部量）。
pub(super) struct MegakernelArgsInputs<'a> {
    pub(super) uploads: &'a MegakernelUploads,
    pub(super) context: &'a RenderContext,
    pub(super) use_shadows: bool,
    pub(super) nrender: usize,
    pub(super) ngeom: usize,
    pub(super) nworld: usize,
    pub(super) launch_worlds: usize,
    pub(super) total_rays: usize,
    pub(super) expected_rays: usize,
    pub(super) ncam: usize,
    pub(super) rgb_pixels: usize,
    pub(super) depth_pixels: usize,
    pub(super) seg_slots: usize,
    pub(super) flags_word: u64,
}

/// 按 `MEGAKERNEL_SLOTS` 顺序 push 81 槽实参（零行为：与拆分前逐项相同）。
pub(super) fn push_megakernel_args(
    args: &mut Vec<RenderArg>,
    inputs: &MegakernelArgsInputs<'_>,
) -> Result<(), LauncherError> {
    let u = inputs.uploads;
    let context = inputs.context;
    // R06-S2：flex 家族计数（空通道全 0 → 各槽 `max(1)` 占位形状；`bvh_nflexgeom` 标量同源）。
    let flex_counts = u.flex.counts;
    let nrender = inputs.nrender;
    let ngeom = inputs.ngeom;
    let nworld = inputs.nworld;
    let launch_worlds = inputs.launch_worlds;
    let total_rays = inputs.total_rays;
    let expected_rays = inputs.expected_rays;
    let ncam = inputs.ncam;
    let rgb_shape = [nworld, inputs.rgb_pixels.max(1)];
    let depth_shape = [nworld, inputs.depth_pixels.max(1)];
    let seg_shape = [nworld, inputs.seg_slots.max(1)];

    // ---- 81 槽实参（顺序 = S7a MEGAKERNEL_SLOTS） ----
    let camera_pose_dims = camera_pose_shape(launch_worlds, ncam);
    let shared_cam = shared_shape(ncam);
    let light_type_dims = light_type_shape(u.nlight);
    let shared_geom = shared_shape(ngeom);
    let shared_light = shared_shape(u.nlight);
    let shared_mat = shared_shape(u.nmat);
    let mat_texid_dims = shared_matrix_shape(u.nmat, 10);
    args.push(RenderArg::Bounds2D(LaunchBounds2::new([
        launch_worlds,
        total_rays,
    ])?));
    args.push(array(&u.geom_type, RenderElem::I32, &shared_geom)?);
    args.push(array(&u.geom_dataid, RenderElem::I32, &shared_geom)?);
    args.push(array(&u.geom_matid, RenderElem::I32, &shared_geom)?);
    args.push(array(&u.geom_size, RenderElem::Vec3, &shared_geom)?);
    args.push(array(&u.geom_rgba, RenderElem::Vec4, &shared_geom)?);
    args.push(array(&u.cam_projection, RenderElem::I32, &shared_cam)?);
    args.push(array(&u.cam_fovy, RenderElem::F32, &shared_cam)?);
    args.push(array(&u.cam_sensorsize, RenderElem::Vec2, &shared_cam)?);
    args.push(array(&u.cam_intrinsic, RenderElem::Vec4, &shared_cam)?);
    args.push(array(&u.light_type, RenderElem::I32, &light_type_dims)?);
    args.push(array(&u.light_castshadow, RenderElem::Bool, &shared_light)?);
    args.push(array(&u.light_active, RenderElem::Bool, &shared_light)?);
    args.push(array(
        &u.light_attenuation,
        RenderElem::Vec3,
        &shared_light,
    )?);
    args.push(array(&u.light_cutoff, RenderElem::F32, &shared_light)?);
    args.push(array(&u.light_exponent, RenderElem::F32, &shared_light)?);
    args.push(array(&u.light_ambient, RenderElem::Vec3, &shared_light)?);
    args.push(array(&u.light_diffuse, RenderElem::Vec3, &shared_light)?);
    args.push(array(&u.light_specular, RenderElem::Vec3, &shared_light)?);
    // R06-S2：flex 家族按上游语义供数；空通道落 1 元素占位 + `max(1)` 形状口径。
    args.push(array(
        &u.flex.flex_vertadr,
        RenderElem::I32,
        &flex_vertadr_shape(flex_counts),
    )?);
    args.push(array(
        &u.flex.flex_edge,
        RenderElem::Vec2I,
        &flex_edge_shape(flex_counts),
    )?);
    args.push(array(
        &u.flex.flex_radius,
        RenderElem::F32,
        &flex_model_shape(flex_counts),
    )?);
    args.push(array(
        &u.mesh_faceadr,
        RenderElem::I32,
        &mesh_slot_shape(u.nmesh),
    )?);
    args.push(array(
        &u.mesh_normaladr,
        RenderElem::I32,
        &mesh_slot_shape(u.nmesh),
    )?);
    args.push(array(
        &u.mesh_normal,
        RenderElem::Vec3,
        &mesh_slot_shape(u.mesh_normal.len() / 3),
    )?);
    args.push(array(&u.mat_texid, RenderElem::I32, &mat_texid_dims)?);
    args.push(array(&u.mat_texuniform, RenderElem::Bool, &shared_mat)?);
    args.push(array(&u.mat_texrepeat, RenderElem::Vec2, &shared_mat)?);
    args.push(array(&u.mat_emission, RenderElem::F32, &shared_mat)?);
    args.push(array(&u.mat_specular, RenderElem::F32, &shared_mat)?);
    args.push(array(&u.mat_shininess, RenderElem::F32, &shared_mat)?);
    args.push(array(&u.mat_rgba, RenderElem::Vec4, &shared_mat)?);
    args.push(array(&u.geom_xpos, RenderElem::Vec3, &shared_geom)?);
    args.push(array(&u.geom_xmat, RenderElem::Mat33, &shared_geom)?);
    args.push(array(&u.cam_xpos, RenderElem::Vec3, &camera_pose_dims)?);
    args.push(array(&u.cam_xmat, RenderElem::Mat33, &camera_pose_dims)?);
    args.push(array(&u.light_xpos, RenderElem::Vec3, &shared_light)?);
    args.push(array(&u.light_xdir, RenderElem::Vec3, &shared_light)?);
    // R06-S2：上游 `flexvert_xpos_in` 是 2-D `[nworld, nflexvert]`（1-D 占位改 2-D 属实参填充）。
    args.push(array(
        &u.flex.flexvert_xpos,
        RenderElem::Vec3,
        &flexvert_xpos_shape(nworld, flex_counts.nflexvert),
    )?);
    args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(nrender)?)));
    // R09-S2b：第 40 槽（0-based `MEGAKERNEL_SLOTS[40]`）阴影总开关，值取 `use_shadows`。
    args.push(use_shadows_arg(inputs.use_shadows));
    args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(ngeom)?)));
    // R06-S2：`bvh_nflexgeom` 标量 = scene BVH flex 图元数（空通道 0，与 S7b2 一致）。
    args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(
        flex_counts.bvh_nflexgeom,
    )?)));
    args.push(array(&u.cam_res, RenderElem::Vec2I, &[nrender])?);
    args.push(array(&u.cam_id_map, RenderElem::I32, &[nrender])?);
    args.push(array(&u.ray, RenderElem::Vec3, &[expected_rays])?);
    args.push(array(&u.ray_offset, RenderElem::Vec3, &[expected_rays])?);
    args.push(RenderArg::Scalar(RenderScalar::I32(0)));
    args.push(array(&u.rgb_adr, RenderElem::I32, &[nrender])?);
    args.push(array(&u.depth_adr, RenderElem::I32, &[nrender])?);
    args.push(array(&u.seg_adr, RenderElem::I32, &[nrender])?);
    args.push(array(&u.render_rgb, RenderElem::Bool, &[nrender])?);
    args.push(array(&u.render_depth, RenderElem::Bool, &[nrender])?);
    args.push(array(&u.render_seg, RenderElem::Bool, &[nrender])?);
    args.push(RenderArg::Scalar(RenderScalar::U64(u.blob_addr)));
    args.push(array(&u.group_root, RenderElem::I32, &[1])?);
    args.push(array(
        &u.flex.flex_bvh_id,
        RenderElem::U64,
        &flex_model_shape(flex_counts),
    )?);
    // R06-S2：上游 `flex_group_root` 是 2-D `[nworld, nflex]`（1-D 占位改 2-D 属实参填充）。
    args.push(array(
        &u.flex.flex_group_root,
        RenderElem::I32,
        &flex_group_root_shape(nworld, flex_counts.nflex),
    )?);
    args.push(array(&u.enabled_geom_ids, RenderElem::I32, &[ngeom])?);
    args.push(array(&u.mesh_bvh_id, RenderElem::U64, &[1])?);
    args.push(array(
        &u.mesh_facetexcoord,
        RenderElem::Vec3I,
        &mesh_slot_shape(u.mesh_faces),
    )?);
    args.push(array(
        &u.mesh_facenormal,
        RenderElem::Vec3I,
        &mesh_slot_shape(u.mesh_faces),
    )?);
    let texcoord_elems = u.mesh_texcoord.len() / 2;
    args.push(array(
        &u.mesh_texcoord,
        RenderElem::Vec2,
        &[texcoord_elems],
    )?);
    args.push(array(
        &u.mesh_texcoord_offsets,
        RenderElem::I32,
        &mesh_slot_shape(u.nmesh),
    )?);
    args.push(array(
        &u.hfield_bvh_id,
        RenderElem::U64,
        &[u.hfield_bvh_count],
    )?);
    args.push(array(
        &u.flex.flex_rgba,
        RenderElem::Vec4,
        &flex_model_shape(flex_counts),
    )?);
    args.push(array(
        &u.flex.flex_geom_flexid,
        RenderElem::I32,
        &flex_geom_shape(flex_counts),
    )?);
    args.push(array(
        &u.flex.flex_geom_edgeid,
        RenderElem::I32,
        &flex_geom_shape(flex_counts),
    )?);
    args.push(array(&u.skybox_tex_id, RenderElem::I32, &[1])?);
    args.push(array(&u.skybox_face_width, RenderElem::I32, &[1])?);
    args.push(match &u.texture_array {
        Some(array) => RenderArg::Array(array.render_arg()?),
        None => array(&u.dummy_texture, RenderElem::Texture2d, &[1])?,
    });
    args.push(array(&u.shading_constants, RenderElem::Vec3, &[4])?);
    args.push(array(&u.splat_rotation, RenderElem::Quat, &[1])?);
    args.push(array(&u.scale_constants, RenderElem::Vec3, &[1])?);
    args.push(array(&u.splat_rgba, RenderElem::Vec4, &[1])?);
    args.push(RenderArg::Scalar(RenderScalar::U64(inputs.flags_word)));
    args.push(array(&u.splat_group_root, RenderElem::I32, &[1])?);
    args.push(match &u.rgb_scratch {
        Some(scratch) => array(scratch, RenderElem::U32, &[1, 1])?,
        None => RenderArg::Array(context.outputs().rgb_render_arg_2d(nworld)?),
    });
    args.push(match &u.aa_scratch {
        Some(scratch) => array(scratch, RenderElem::Vec3, &[1])?,
        None => RenderArg::Array(context.outputs().aa_accum_render_arg_2d(nworld)?),
    });
    args.push(match &u.depth_scratch {
        Some(scratch) => array(scratch, RenderElem::F32, &[1, 1])?,
        None => RenderArg::Array(context.outputs().depth_render_arg_2d(nworld)?),
    });
    args.push(match &u.seg_scratch {
        Some(scratch) => array(scratch, RenderElem::Vec2I, &[1, 1])?,
        None => RenderArg::Array(context.outputs().seg_render_arg_2d(nworld)?),
    });
    let _ = (
        rgb_shape,
        depth_shape,
        seg_shape,
        u.light_castshadow.len(),
        u.light_active.len(),
        u.mat_texid.len(),
    );
    Ok(())
}
