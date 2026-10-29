//! S7b2 全签名渲染执行器：把 S6b context、S5 射线与模型着色字段打成**上游 81 槽签名**
//! 的 `render_megakernel` 实参并发射（渲染/BVH S7b2）。
//!
//! 语义边界：
//! - 签名迁到 S7a 冻结的 `MEGAKERNEL_SLOTS`（1 bounds + 80 形参）；内部 BVH 仍是自有打包，
//!   经 `bvh_id`（u64 标量）以设备指针传入（`MegakernelBlob`，见 `cuda/render_common.cu`），
//!   不消费 Warp BVH 句柄；未用槽用 1 元素占位（渲染 ABI 拒绝零长数组），逐槽表进证据。
//! - 着色子集：headlight 三组、背景色、base color、纹理采样、顶点/面法线、自发光、高光、
//!   背面剔除；阴影产品路径 R09-S2b 起已解锁（天空盒/splat/flex/hfield/SDF 仍写前拒绝）。
//! - S7b3：`samples_per_pixel > 1`（AA）时按 sample 顺序发射 `render_megakernel`
//!   （`ray_base = sample * total_rays`，与上游 `for sample in range(nsamples)` 一致），
//!   `store_pixel` 累积到 `aa_accum`，全部 sample 之后发射 `aa_resolve` 求平均并打包。
//! - depth/seg 是**单样本输出**：只在 `ray_base == 0` 的第一个 pass 写（上游 render.py：
//!   「Depth and seg are single-sample outputs: only the first pass writes them」；
//!   见 `cuda/render_megakernel.cu` 的 `first_sample`）。AA 时后续子样本不覆盖 depth/seg；
//!   spp=1 时 `ray_base` 恒为 0，行为与 S7b1 相同。这是 AA 首轮 depth 78% 不匹配的根因修复。
//! - 灯位姿（37/38 槽）：由 `RenderFields` 按上游固定灯公式推导（identity body 旋转），
//!   与 pinned 3.15 `mjwarp.forward` 位型逐位相等由夹具 `light_data_check` 断言。
//! - depth/seg 语义与 S7b1 完全一致：S7b1 的 120,000/120,000 位等是 S7b2 回归门。
//! - 编译一律走 `RenderKernel::compile`（A′ 渲染门禁 `f32-render`，无旁路）。

use crate::model::render::RenderFields;
use crate::render::camera::CameraModel;
use crate::render::context::RenderContext;
use crate::render::kernel::cast::{CameraPose, ModelRenderArrays, SceneDevice, pack_mesh_devices};
use crate::render::rays::RayBuffer;
use crate::runtime::TransferSession;
use crate::runtime::launcher::{
    BlockRule, LaunchBounds2, LaunchDims, LauncherError, MEGAKERNEL_SLOTS, RenderArg, RenderElem,
    RenderScalar, RenderSignature,
};

use self::megakernel_uploads::{UploadInputs, mesh_slot_shape, upload_megakernel_arrays};
use super::launch::RenderKernel;
use super::megakernel_aa::{AAResolve, launch_samples};
use super::megakernel_arrays::{
    array, channel_flags, floats, geom_data_poses, ints, light_data_poses, reals, shading_words,
    to_i32, use_shadows_arg,
};
use super::world_arrays::{camera_pose_shape, light_type_shape, shared_matrix_shape, shared_shape};
use super::{MEGAKERNEL_ENTRY, MEGAKERNEL_SOURCE};

// 拆分①：上传段以 `#[path]` 在**本文件内**声明为子模块，父模块 `kernel/mod.rs` 零改动
// （卡片 scope 只含 `megakernel.rs` 与新文件；`lighting.rs` 同款先例）。
#[path = "megakernel_uploads.rs"]
mod megakernel_uploads;

const BLOCK: u32 = 256;

/// S7b3 渲染执行器；持有 81 槽 megakernel 与 AA 解析内核（模块缓存）与传输会话。
pub struct MegakernelRenderer {
    session: TransferSession,
    kernel: RenderKernel,
    aa: AAResolve,
}

impl MegakernelRenderer {
    /// 编译 `render_megakernel` 与 `aa_resolve`（需要 NVIDIA 驱动与 NVRTC）。
    pub fn new(session: &TransferSession) -> Result<Self, LauncherError> {
        let signature =
            RenderSignature::new(MEGAKERNEL_ENTRY, BlockRule::WarpGrid, &MEGAKERNEL_SLOTS)?;
        let kernel = RenderKernel::compile(
            session,
            MEGAKERNEL_ENTRY,
            MEGAKERNEL_SOURCE,
            signature,
            MEGAKERNEL_ENTRY,
        )?;
        let aa = AAResolve::compile(session)?;
        Ok(Self {
            session: session.clone(),
            kernel,
            aa,
        })
    }

    /// 已编译内核（PTX/哈希证据）。
    pub fn kernel(&self) -> &RenderKernel {
        &self.kernel
    }

    /// 发射一帧（默认 context 口径：spp=1、预计算射线、无天空盒/splat/AA；阴影由 `use_shadows` 控制）。
    pub fn render(
        &self,
        context: &RenderContext,
        fields: &RenderFields,
        model: &CameraModel,
        camera_poses: &[CameraPose],
        rays: &RayBuffer,
    ) -> Result<(), LauncherError> {
        let resolved = context.resolved();
        let layout = &resolved.output;
        let nsamples = layout.nsamples;
        if nsamples == 0 {
            return Err(LauncherError::InvalidArray("nsamples"));
        }
        if !resolved.use_precomputed_rays {
            return Err(LauncherError::Unsupported(
                "S7b3 只支持 use_precomputed_rays=true（动态射线未实现）",
            ));
        }
        // R09-S2b：`use_shadows` 产品路径已解锁（S7b3 前置拒绝已移除；数值验收在 S3b 窗）。
        // R02-S1：天空盒产品路径已解锁（flag bit9 + `sample_skybox`，数值验收在 S2/S3 窗）。
        if resolved.splat_count != 0 {
            return Err(LauncherError::Unsupported("S7b3 不含 splat"));
        }
        // R05-S2：hfield 求交已接入（`gtype==1` 由设备侧分派到 mesh BVH）；
        // SDF（`gtype==8`）仍拒绝。
        if ints(fields, "geom_type")?.contains(&8) {
            return Err(LauncherError::Unsupported("S7b3 不含 SDF 求交"));
        }
        if ints(fields, "light_mode")?.iter().any(|mode| *mode != 0) {
            return Err(LauncherError::Unsupported(
                "S7b3 只支持 light_mode=0（固定灯）的 data 位姿推导",
            ));
        }
        let nworld = resolved.nworld;
        let nrender = resolved.nrender();
        let total_rays = layout.total_rays;
        if nrender == 0 || total_rays == 0 {
            return Ok(()); // 上游 `wp.launch(dim=0)` 为 no-op：空选择不发射、不写输出
        }
        let expected_rays = total_rays
            .checked_mul(nsamples)
            .ok_or(LauncherError::Overflow("ray_count"))?;
        if rays.len() != expected_rays {
            return Err(LauncherError::InvalidArray("ray_count"));
        }
        let ncam = model.ncam();
        let (cam_xpos, cam_xmat) =
            super::world_camera::world_camera_pose_arrays(camera_poses, nworld, ncam)?;

        let model_arrays = ModelRenderArrays::from_fields(fields)?;
        let ngeom = model_arrays.geom_type.len();
        if !resolved.enabled_geom_ids.is_empty() && resolved.enabled_geom_ids.len() != ngeom {
            return Err(LauncherError::Unsupported(
                "S7b2 要求 enabled_geom_ids 覆盖全部 geom（自有打包按 geom id 索引）",
            ));
        }

        // 场景/网格设备打包（与 S7b1 同源；每帧从 context 拥有的 BVH 重打包）。
        // R05-S2：hfield mesh 追加在既有 mesh 末尾（既有索引不变），打包逻辑在 `cast_mesh.rs`。
        let scene = SceneDevice::pack(
            context.scene_bvh(),
            context.scene(),
            &model_arrays,
            context.mesh_count(),
        )?;
        let (mesh, hfield) = pack_mesh_devices(context, resolved, &model_arrays)?;

        // ---- 相机参数（按模型相机 id；上游 `cam_*_in[worldid, mujoco_cam_id]`） ----
        let cam_projection = ints(fields, "cam_projection")?;
        let cam_fovy = reals(fields, "cam_fovy")?;
        let cam_sensorsize = floats(fields, "cam_sensorsize")?;
        let cam_intrinsic = floats(fields, "cam_intrinsic")?;

        // ---- 场景 geom data 位姿（从打包后的 scene 布局反解 leaf→geom） ----
        let (geom_xpos, geom_xmat) = geom_data_poses(&scene, ngeom)?;

        // ---- 灯 data 位姿（固定灯：identity body 旋转；见文件头） ----
        let light_poses = light_data_poses(fields)
            .map_err(|error| LauncherError::Driver(format!("light-poses: {error}")))?;
        let mut light_xpos = Vec::with_capacity(3 * light_poses.len());
        let mut light_xdir = Vec::with_capacity(3 * light_poses.len());
        for pose in &light_poses {
            light_xpos.extend_from_slice(&pose.xpos);
            light_xdir.extend_from_slice(&pose.xdir);
        }

        // ---- 输出布局/相机布局/射线 ----
        // R08-RGB：通道开关按激活相机逐项填充（81 槽 51/52/53 本就是 per-render bool 数组）。
        // 关闭通道的相机在输出布局里 `*_adr = -1`，内核按 `do_rgb/do_depth/do_seg` 跳过写入；
        // 不得再用全局布尔填充（否则关闭通道的相机会按负偏移写，即 latent 越界）。
        let render_rgb = channel_flags(&resolved.plan.render_rgb);
        let render_depth = channel_flags(&resolved.plan.render_depth);
        let render_seg = channel_flags(&resolved.plan.render_seg);
        let cam_res_flat: Vec<i32> = resolved
            .cam_res()
            .iter()
            .flat_map(|resolution| {
                [
                    i32::try_from(resolution[0]).unwrap_or(i32::MAX),
                    i32::try_from(resolution[1]).unwrap_or(i32::MAX),
                ]
            })
            .collect();
        let cam_id_map = resolved.cam_id_map();
        let mut ray_dirs = Vec::with_capacity(3 * total_rays);
        let mut ray_offsets = Vec::with_capacity(3 * total_rays);
        for (direction, offset) in rays.directions().iter().zip(rays.offsets()) {
            ray_dirs.extend_from_slice(direction);
            ray_offsets.extend_from_slice(offset);
        }

        // ---- 着色常量/符号位（本实现自有槽，见内核注释；拆到 arrays 保 500 行棘轮）----
        let (shading_constants, scale_constants, flags_word) = shading_words(resolved, nsamples);

        // ---- 设备上传（拆分①：纯移动至 `megakernel_uploads.rs`；逻辑/顺序/错误路径/占位零变化）----
        let uploads = upload_megakernel_arrays(&UploadInputs {
            session: &self.session,
            fields,
            model_arrays: &model_arrays,
            resolved,
            context,
            scene: &scene,
            mesh: &mesh,
            ngeom,
            nsamples,
            cam_projection: &cam_projection,
            cam_fovy: &cam_fovy,
            cam_sensorsize: &cam_sensorsize,
            cam_intrinsic: &cam_intrinsic,
            light_xpos: &light_xpos,
            light_xdir: &light_xdir,
            geom_xpos: &geom_xpos,
            geom_xmat: &geom_xmat,
            cam_xpos: &cam_xpos,
            cam_xmat: &cam_xmat,
            cam_res_flat: &cam_res_flat,
            cam_id_map: &cam_id_map,
            ray_dirs: &ray_dirs,
            ray_offsets: &ray_offsets,
            render_rgb: &render_rgb,
            render_depth: &render_depth,
            render_seg: &render_seg,
            shading_constants: &shading_constants,
            scale_constants: &scale_constants,
            // R05-S2：无 hfield geom 时为空切片（上传侧落 1 个 `u64::MAX` 哨兵占位）。
            hfield_bvh_ids: hfield.as_ref().map_or(&[], |device| device.bvh_ids()),
        })?;
        let u = &uploads;
        let rgb_shape = [nworld, layout.rgb_pixels.max(1)];
        let depth_shape = [nworld, layout.depth_pixels.max(1)];
        let seg_shape = [nworld, layout.seg_slots.max(1)];

        // ---- 81 槽实参（顺序 = S7a MEGAKERNEL_SLOTS） ----
        let mut args: Vec<RenderArg> = Vec::with_capacity(MEGAKERNEL_SLOTS.len());
        let camera_pose_dims = camera_pose_shape(nworld, ncam);
        let shared_cam = shared_shape(ncam);
        let light_type_dims = light_type_shape(u.nlight);
        let shared_geom = shared_shape(ngeom);
        let shared_light = shared_shape(u.nlight);
        let shared_mat = shared_shape(u.nmat);
        let mat_texid_dims = shared_matrix_shape(u.nmat, 10);
        args.push(RenderArg::Bounds2D(LaunchBounds2::new([
            nworld, total_rays,
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
        args.push(array(&u.flex_vertadr, RenderElem::I32, &[1])?);
        args.push(array(&u.flex_edge, RenderElem::Vec2I, &[1])?);
        args.push(array(&u.flex_radius, RenderElem::F32, &[1])?);
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
        args.push(array(&u.flexvert_xpos, RenderElem::Vec3, &[1])?);
        args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(nrender)?)));
        // R09-S2b：第 40 槽（0-based `MEGAKERNEL_SLOTS[40]`）阴影总开关，值取 `resolved.use_shadows`。
        args.push(use_shadows_arg(resolved.use_shadows));
        args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(ngeom)?)));
        args.push(RenderArg::Scalar(RenderScalar::I32(0)));
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
        args.push(array(&u.flex_bvh_id, RenderElem::U64, &[1])?);
        args.push(array(&u.flex_group_root, RenderElem::I32, &[1])?);
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
        args.push(array(&u.flex_rgba, RenderElem::Vec4, &[1])?);
        args.push(array(&u.flex_geom_flexid, RenderElem::I32, &[1])?);
        args.push(array(&u.flex_geom_edgeid, RenderElem::I32, &[1])?);
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
        args.push(RenderArg::Scalar(RenderScalar::U64(flags_word)));
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

        let grid = u32::try_from((nworld * total_rays).div_ceil(BLOCK as usize))
            .map_err(|_| LauncherError::Overflow("render_grid"))?;
        let dims = LaunchDims::new((grid, 1, 1), (BLOCK, 1, 1), 0)?;

        // S7b3 AA：按 sample 顺序发射（只改 ray_base 槽）；全部 sample 之后 resolve。
        launch_samples(
            &self.kernel,
            &self.session,
            &mut args,
            dims,
            nsamples,
            total_rays,
        )?;
        if nsamples == 1 {
            return Ok(());
        }
        self.aa
            .resolve(&self.session, context, nworld, layout.rgb_pixels, nsamples)
    }
}
