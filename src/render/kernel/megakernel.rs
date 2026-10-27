//! S7b2 全签名渲染执行器：把 S6b context、S5 射线与模型着色字段打成**上游 81 槽签名**
//! 的 `render_megakernel` 实参并发射（渲染/BVH S7b2）。
//!
//! 语义边界：
//! - 签名迁到 S7a 冻结的 `MEGAKERNEL_SLOTS`（1 bounds + 80 形参）；内部 BVH 仍是自有打包，
//!   经 `bvh_id`（u64 标量）以设备指针传入（`MegakernelBlob`，见 `cuda/render_common.cu`），
//!   不消费 Warp BVH 句柄；未用槽用 1 元素占位（渲染 ABI 拒绝零长数组），逐槽表进证据。
//! - 着色子集：headlight 三组、背景色、base color、纹理采样、顶点/面法线、自发光、高光、
//!   背面剔除；阴影/天空盒/splat/flex/hfield/SDF 由本片 host 侧**写前拒绝**。
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
use crate::render::ImageOutput;
use crate::render::camera::CameraModel;
use crate::render::context::RenderContext;
use crate::render::kernel::cast::{CameraPose, MeshDevice, ModelRenderArrays, SceneDevice};
use crate::render::rays::RayBuffer;
use crate::runtime::TransferSession;
use crate::runtime::launcher::{
    BlockRule, LaunchBounds2, LaunchDims, LauncherError, MEGAKERNEL_SLOTS, RenderArg, RenderElem,
    RenderScalar, RenderSignature,
};

use super::launch::{RenderKernel, Texture2dArray};
use super::megakernel_aa::{AAResolve, launch_samples};
use super::megakernel_arrays::{
    array, bool_words, flags, floats, geom_data_poses, ints, light_data_poses, pack_blob, reals,
    shading_words, span_addr, to_i32, upload,
};
use super::{MEGAKERNEL_ENTRY, MEGAKERNEL_SOURCE};

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

    /// 发射一帧（默认 context 口径：spp=1、预计算射线、无阴影/天空盒/splat/AA）。
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
        if resolved.use_shadows {
            return Err(LauncherError::Unsupported("S7b3 不含阴影"));
        }
        if resolved.render_skybox {
            return Err(LauncherError::Unsupported("S7b3 不含天空盒"));
        }
        if resolved.splat_count != 0 {
            return Err(LauncherError::Unsupported("S7b3 不含 splat"));
        }
        if ints(fields, "geom_type")?
            .iter()
            .any(|kind| matches!(kind, 1 | 8))
        {
            return Err(LauncherError::Unsupported("S7b3 不含 hfield/SDF 求交"));
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
        let scene = SceneDevice::pack(
            context.scene_bvh(),
            context.scene(),
            &model_arrays,
            context.mesh_count(),
        )?;
        let mut mesh_bvhs = Vec::with_capacity(context.mesh_count());
        for index in 0..context.mesh_count() {
            mesh_bvhs.push(
                context
                    .mesh_bvh(index)
                    .ok_or(LauncherError::InvalidArray("mesh_index"))?,
            );
        }
        let mesh = MeshDevice::pack(&mesh_bvhs)?;

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
        let rgb_enabled = context.outputs().is_enabled(ImageOutput::Rgb);
        let depth_enabled = context.outputs().is_enabled(ImageOutput::Depth);
        let seg_enabled = context.outputs().is_enabled(ImageOutput::Segmentation);
        let render_rgb = vec![i32::from(rgb_enabled); nrender];
        let render_depth = vec![i32::from(depth_enabled); nrender];
        let render_seg = vec![i32::from(seg_enabled); nrender];
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

        // ---- 纹理数组（S4 只读句柄 → S7a `texture2d_t` 数组）----
        let texture_array = match context.textures() {
            Some(resources) if resolved.use_textures => {
                let handles = resources.handles();
                if handles.is_empty() {
                    return Err(LauncherError::InvalidArray("textures"));
                }
                Some(Texture2dArray::upload(&self.session, &handles)?)
            }
            _ => None,
        };

        // ---- 上传所有设备缓冲 ----
        let blob = pack_blob(&scene, &mesh);
        let blob_buffer = upload(&self.session, &blob)?;
        let blob_addr = span_addr(&blob_buffer)?;

        let geom_type = upload(&self.session, &model_arrays.geom_type)?;
        let geom_dataid = ints(fields, "geom_dataid")?;
        let geom_dataid = upload(&self.session, &geom_dataid)?;
        let geom_matid = upload(&self.session, &model_arrays.geom_matid)?;
        let geom_size = reals(fields, "geom_size")?;
        let geom_size = upload(&self.session, &geom_size)?;
        let geom_rgba = upload(&self.session, &model_arrays.geom_rgba)?;
        let cam_projection = upload(&self.session, &cam_projection)?;
        let cam_fovy = upload(&self.session, &cam_fovy)?;
        let cam_sensorsize = upload(&self.session, &cam_sensorsize)?;
        let cam_intrinsic = upload(&self.session, &cam_intrinsic)?;

        let light_type = ints(fields, "light_type")?;
        let nlight = light_type.len();
        let light_type = upload(&self.session, &light_type)?;
        let light_castshadow = bool_words(&flags(fields, "light_castshadow")?);
        let light_castshadow = upload(&self.session, &light_castshadow)?;
        let light_active = bool_words(&flags(fields, "light_active")?);
        let light_active = upload(&self.session, &light_active)?;
        let light_attenuation = upload(&self.session, &floats(fields, "light_attenuation")?)?;
        let light_cutoff = upload(&self.session, &floats(fields, "light_cutoff")?)?;
        let light_exponent = upload(&self.session, &floats(fields, "light_exponent")?)?;
        let light_ambient = upload(&self.session, &floats(fields, "light_ambient")?)?;
        let light_diffuse = upload(&self.session, &floats(fields, "light_diffuse")?)?;
        let light_specular = upload(&self.session, &floats(fields, "light_specular")?)?;
        let light_xpos = upload(&self.session, &light_xpos)?;
        let light_xdir = upload(&self.session, &light_xdir)?;

        let flex_vertadr = upload(&self.session, &[0i32])?;
        let flex_edge = upload(&self.session, &[0i32, 0])?;
        let flex_radius = upload(&self.session, &[0.0f32])?;
        let flexvert_xpos = upload(&self.session, &[0.0f32; 3])?;
        let flex_bvh_id = upload(&self.session, &[0u32, 0])?;
        let flex_group_root = upload(&self.session, &[0i32])?;
        let flex_rgba = upload(&self.session, &[0.0f32; 4])?;
        let flex_geom_flexid = upload(&self.session, &[0i32])?;
        let flex_geom_edgeid = upload(&self.session, &[0i32])?;
        let hfield_bvh_id = upload(&self.session, &[0u32, 0])?;
        let mesh_bvh_id = upload(&self.session, &[0u32, 0])?;
        let group_root = upload(&self.session, &[0i32])?;
        let skybox_tex_id = upload(&self.session, &[0i32])?;
        let skybox_face_width = upload(&self.session, &[0i32])?;
        let splat_rotation = upload(&self.session, &[0.0f32; 4])?;
        let splat_rgba = upload(&self.session, &[0.0f32; 4])?;
        let splat_group_root = upload(&self.session, &[0i32])?;
        let dummy_texture = upload(&self.session, &[0u32; 6])?;

        let mesh_faceadr = ints(fields, "mesh_faceadr")?;
        let nmesh = mesh_faceadr.len();
        let mesh_faceadr = upload(&self.session, &mesh_faceadr)?;
        let mesh_normaladr = ints(fields, "mesh_normaladr")?;
        let mesh_normaladr = upload(&self.session, &mesh_normaladr)?;
        let mesh_normal = floats(fields, "mesh_normal")?;
        let mesh_normal = upload(&self.session, &mesh_normal)?;
        let mesh_facetexcoord = ints(fields, "mesh_facetexcoord")?;
        let mesh_faces = mesh_facetexcoord.len() / 3;
        let mesh_facetexcoord = upload(&self.session, &mesh_facetexcoord)?;
        let mesh_facenormal = ints(fields, "mesh_facenormal")?;
        let mesh_facenormal = upload(&self.session, &mesh_facenormal)?;
        // scene-a 的网格没有 texcoord（`mesh_texcoordadr=-1`）：上传 2 个 0 占位（ABI 拒绝零长数组）。
        let mesh_texcoord = {
            let values = floats(fields, "mesh_texcoord")?;
            if values.is_empty() {
                vec![0.0f32; 2]
            } else {
                values
            }
        };
        let mesh_texcoord = upload(&self.session, &mesh_texcoord)?;
        let mesh_texcoord_offsets = ints(fields, "mesh_texcoordadr")?;
        let mesh_texcoord_offsets = upload(&self.session, &mesh_texcoord_offsets)?;

        let mat_texid = ints(fields, "mat_texid")?;
        let nmat = mat_texid.len() / 10;
        let mat_texid = upload(&self.session, &mat_texid)?;
        let mat_texuniform = bool_words(&flags(fields, "mat_texuniform")?);
        let mat_texuniform = upload(&self.session, &mat_texuniform)?;
        let mat_texrepeat = upload(&self.session, &floats(fields, "mat_texrepeat")?)?;
        let mat_emission = upload(&self.session, &floats(fields, "mat_emission")?)?;
        let mat_specular = upload(&self.session, &floats(fields, "mat_specular")?)?;
        let mat_shininess = upload(&self.session, &floats(fields, "mat_shininess")?)?;
        let mat_rgba = upload(&self.session, &model_arrays.mat_rgba)?;

        let geom_xpos = upload(&self.session, &geom_xpos)?;
        let geom_xmat = upload(&self.session, &geom_xmat)?;
        let cam_xpos = upload(&self.session, &cam_xpos)?;
        let cam_xmat = upload(&self.session, &cam_xmat)?;

        let cam_res = upload(&self.session, &cam_res_flat)?;
        let cam_id_map = upload(&self.session, &cam_id_map)?;
        let ray = upload(&self.session, &ray_dirs)?;
        let ray_offset = upload(&self.session, &ray_offsets)?;
        let rgb_adr = upload(&self.session, &layout.rgb_adr)?;
        let depth_adr = upload(&self.session, &layout.depth_adr)?;
        let seg_adr = upload(&self.session, &layout.seg_adr)?;
        let render_rgb = upload(&self.session, &bool_words(&render_rgb))?;
        let render_depth = upload(&self.session, &bool_words(&render_depth))?;
        let render_seg = upload(&self.session, &bool_words(&render_seg))?;
        let enabled_geom_ids = match resolved.enabled_geom_ids.is_empty() {
            true => (0..ngeom as i32).collect::<Vec<i32>>(),
            false => resolved.enabled_geom_ids.clone(),
        };
        let enabled_geom_ids = upload(&self.session, &enabled_geom_ids)?;
        let shading_constants = upload(&self.session, &shading_constants)?;
        let scale_constants = upload(&self.session, &scale_constants)?;
        // AA（nsamples>1）时 aa_accum 由 OutputResources 分配（world × rgb_pixels vec3），
        // 否则 1 元素占位（内核按 AA 标志不访问）。
        let aa_scratch = match nsamples > 1 {
            true => None,
            false => Some(upload(&self.session, &[0.0f32; 3])?),
        };

        // 输出缓冲（S6b OutputResources；未启用时 1 元素占位、内核按标志不写）。
        let rgb_scratch = match rgb_enabled {
            true => None,
            false => Some(upload(&self.session, &[0u32])?),
        };
        let depth_scratch = match depth_enabled {
            true => None,
            false => Some(upload(&self.session, &[0.0f32])?),
        };
        let seg_scratch = match seg_enabled {
            true => None,
            false => Some(upload(&self.session, &[0i32; 2])?),
        };
        let rgb_shape = [nworld, layout.rgb_pixels.max(1)];
        let depth_shape = [nworld, layout.depth_pixels.max(1)];
        let seg_shape = [nworld, layout.seg_slots.max(1)];

        // ---- 81 槽实参（顺序 = S7a MEGAKERNEL_SLOTS） ----
        let mut args: Vec<RenderArg> = Vec::with_capacity(MEGAKERNEL_SLOTS.len());
        args.push(RenderArg::Bounds2D(LaunchBounds2::new([
            nworld, total_rays,
        ])?));
        args.push(array(&geom_type, RenderElem::I32, &[nworld, ngeom])?);
        args.push(array(&geom_dataid, RenderElem::I32, &[nworld, ngeom])?);
        args.push(array(&geom_matid, RenderElem::I32, &[nworld, ngeom])?);
        args.push(array(&geom_size, RenderElem::Vec3, &[nworld, ngeom])?);
        args.push(array(&geom_rgba, RenderElem::Vec4, &[nworld, ngeom])?);
        args.push(array(&cam_projection, RenderElem::I32, &[nworld, ncam])?);
        args.push(array(&cam_fovy, RenderElem::F32, &[nworld, ncam])?);
        args.push(array(&cam_sensorsize, RenderElem::Vec2, &[nworld, ncam])?);
        args.push(array(&cam_intrinsic, RenderElem::Vec4, &[nworld, ncam])?);
        args.push(array(&light_type, RenderElem::I32, &[nworld, nlight])?);
        args.push(array(
            &light_castshadow,
            RenderElem::Bool,
            &[nworld, nlight],
        )?);
        args.push(array(&light_active, RenderElem::Bool, &[nworld, nlight])?);
        args.push(array(
            &light_attenuation,
            RenderElem::Vec3,
            &[nworld, nlight],
        )?);
        args.push(array(&light_cutoff, RenderElem::F32, &[nworld, nlight])?);
        args.push(array(&light_exponent, RenderElem::F32, &[nworld, nlight])?);
        args.push(array(&light_ambient, RenderElem::Vec3, &[nworld, nlight])?);
        args.push(array(&light_diffuse, RenderElem::Vec3, &[nworld, nlight])?);
        args.push(array(&light_specular, RenderElem::Vec3, &[nworld, nlight])?);
        args.push(array(&flex_vertadr, RenderElem::I32, &[1])?);
        args.push(array(&flex_edge, RenderElem::Vec2I, &[1])?);
        args.push(array(&flex_radius, RenderElem::F32, &[1])?);
        args.push(array(&mesh_faceadr, RenderElem::I32, &[nmesh])?);
        args.push(array(&mesh_normaladr, RenderElem::I32, &[nmesh])?);
        args.push(array(
            &mesh_normal,
            RenderElem::Vec3,
            &[mesh_normal.len() / 3],
        )?);
        args.push(array(&mat_texid, RenderElem::I32, &[nworld, nmat, 10])?);
        args.push(array(&mat_texuniform, RenderElem::Bool, &[nworld, nmat])?);
        args.push(array(&mat_texrepeat, RenderElem::Vec2, &[nworld, nmat])?);
        args.push(array(&mat_emission, RenderElem::F32, &[nworld, nmat])?);
        args.push(array(&mat_specular, RenderElem::F32, &[nworld, nmat])?);
        args.push(array(&mat_shininess, RenderElem::F32, &[nworld, nmat])?);
        args.push(array(&mat_rgba, RenderElem::Vec4, &[nworld, nmat])?);
        args.push(array(&geom_xpos, RenderElem::Vec3, &[nworld, ngeom])?);
        args.push(array(&geom_xmat, RenderElem::Mat33, &[nworld, ngeom])?);
        args.push(array(&cam_xpos, RenderElem::Vec3, &[nworld, ncam])?);
        args.push(array(&cam_xmat, RenderElem::Mat33, &[nworld, ncam])?);
        args.push(array(&light_xpos, RenderElem::Vec3, &[nworld, nlight])?);
        args.push(array(&light_xdir, RenderElem::Vec3, &[nworld, nlight])?);
        args.push(array(&flexvert_xpos, RenderElem::Vec3, &[1])?);
        args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(nrender)?)));
        args.push(RenderArg::Scalar(RenderScalar::Bool(false)));
        args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(ngeom)?)));
        args.push(RenderArg::Scalar(RenderScalar::I32(0)));
        args.push(array(&cam_res, RenderElem::Vec2I, &[nrender])?);
        args.push(array(&cam_id_map, RenderElem::I32, &[nrender])?);
        args.push(array(&ray, RenderElem::Vec3, &[expected_rays])?);
        args.push(array(&ray_offset, RenderElem::Vec3, &[expected_rays])?);
        args.push(RenderArg::Scalar(RenderScalar::I32(0)));
        args.push(array(&rgb_adr, RenderElem::I32, &[nrender])?);
        args.push(array(&depth_adr, RenderElem::I32, &[nrender])?);
        args.push(array(&seg_adr, RenderElem::I32, &[nrender])?);
        args.push(array(&render_rgb, RenderElem::Bool, &[nrender])?);
        args.push(array(&render_depth, RenderElem::Bool, &[nrender])?);
        args.push(array(&render_seg, RenderElem::Bool, &[nrender])?);
        args.push(RenderArg::Scalar(RenderScalar::U64(blob_addr)));
        args.push(array(&group_root, RenderElem::I32, &[1])?);
        args.push(array(&flex_bvh_id, RenderElem::U64, &[1])?);
        args.push(array(&flex_group_root, RenderElem::I32, &[1])?);
        args.push(array(&enabled_geom_ids, RenderElem::I32, &[ngeom])?);
        args.push(array(&mesh_bvh_id, RenderElem::U64, &[1])?);
        args.push(array(&mesh_facetexcoord, RenderElem::Vec3I, &[mesh_faces])?);
        args.push(array(&mesh_facenormal, RenderElem::Vec3I, &[mesh_faces])?);
        let texcoord_elems = mesh_texcoord.len() / 2;
        args.push(array(&mesh_texcoord, RenderElem::Vec2, &[texcoord_elems])?);
        args.push(array(&mesh_texcoord_offsets, RenderElem::I32, &[nmesh])?);
        args.push(array(&hfield_bvh_id, RenderElem::U64, &[1])?);
        args.push(array(&flex_rgba, RenderElem::Vec4, &[1])?);
        args.push(array(&flex_geom_flexid, RenderElem::I32, &[1])?);
        args.push(array(&flex_geom_edgeid, RenderElem::I32, &[1])?);
        args.push(array(&skybox_tex_id, RenderElem::I32, &[1])?);
        args.push(array(&skybox_face_width, RenderElem::I32, &[1])?);
        args.push(match &texture_array {
            Some(array) => RenderArg::Array(array.render_arg()?),
            None => array(&dummy_texture, RenderElem::Texture2d, &[1])?,
        });
        args.push(array(&shading_constants, RenderElem::Vec3, &[4])?);
        args.push(array(&splat_rotation, RenderElem::Quat, &[1])?);
        args.push(array(&scale_constants, RenderElem::Vec3, &[1])?);
        args.push(array(&splat_rgba, RenderElem::Vec4, &[1])?);
        args.push(RenderArg::Scalar(RenderScalar::U64(flags_word)));
        args.push(array(&splat_group_root, RenderElem::I32, &[1])?);
        args.push(match &rgb_scratch {
            Some(scratch) => array(scratch, RenderElem::U32, &[1, 1])?,
            None => RenderArg::Array(context.outputs().rgb_render_arg_2d(nworld)?),
        });
        args.push(match &aa_scratch {
            Some(scratch) => array(scratch, RenderElem::Vec3, &[1])?,
            None => RenderArg::Array(context.outputs().aa_accum_render_arg_2d(nworld)?),
        });
        args.push(match &depth_scratch {
            Some(scratch) => array(scratch, RenderElem::F32, &[1, 1])?,
            None => RenderArg::Array(context.outputs().depth_render_arg_2d(nworld)?),
        });
        args.push(match &seg_scratch {
            Some(scratch) => array(scratch, RenderElem::Vec2I, &[1, 1])?,
            None => RenderArg::Array(context.outputs().seg_render_arg_2d(nworld)?),
        });
        let _ = (
            rgb_shape,
            depth_shape,
            seg_shape,
            light_castshadow.len(),
            light_active.len(),
            mat_texid.len(),
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
