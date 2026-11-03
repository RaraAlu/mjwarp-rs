//! S7b2 全签名渲染执行器：把 S6b context、S5 射线与模型着色字段打成**上游 81 槽签名**
//! 的 `render_megakernel` 实参并发射（渲染/BVH S7b2）。
//!
//! 语义边界：
//! - 签名迁到 S7a 冻结的 `MEGAKERNEL_SLOTS`（1 bounds + 80 形参）；内部 BVH 仍是自有打包，
//!   经 `bvh_id`（u64 标量）以设备指针传入（`MegakernelBlob`，见 `cuda/render_common.cu`），
//!   不消费 Warp BVH 句柄；未用槽用 1 元素占位（渲染 ABI 拒绝零长数组），逐槽表进证据。
//! - 着色子集：headlight、背景色、base color、纹理、顶点/面法线、自发光、高光、背面剔除；
//!   阴影 R09-S2b 起解锁（天空盒/splat/flex/hfield/SDF 仍写前拒绝）。
//! - S7b3：`samples_per_pixel > 1`（AA）按 sample 顺序发射并把 `store_pixel` 累积到
//!   `aa_accum`，全部 sample 后 `aa_resolve` 平均；depth/seg 只在 `ray_base == 0` 写。
//! - 灯位姿（37/38 槽）：由 `RenderFields` 按上游固定灯公式推导（identity body 旋转），
//!   与 pinned 3.15 `mjwarp.forward` 位型逐位相等由夹具 `light_data_check` 断言。
//! - depth/seg 语义与 S7b1 完全一致：S7b1 的 120,000/120,000 位等是 S7b2 回归门。
//! - 编译一律走 `RenderKernel::compile`（A′ 渲染门禁 `f32-render`，无旁路）。
//! - R06-S2：`render_with_flex` 接受显式宿主 flex 通道（[`FlexHostInputs`]）；`None` 与
//!   `render` 逐字节等价，设备侧 flex 分支与 81 槽 ABI 均不变（`.cu` 零改动）。
use crate::model::render::RenderFields;
use crate::render::camera::CameraModel;
use crate::render::context::RenderContext;
use crate::render::kernel::cast::{CameraPose, ModelRenderArrays, SceneDevice, pack_mesh_devices};
use crate::render::rays::RayBuffer;
use crate::runtime::TransferSession;
use crate::runtime::launcher::{
    BlockRule, LaunchDims, LauncherError, MEGAKERNEL_SLOTS, RenderArg, RenderSignature,
};

use self::megakernel_uploads::{UploadInputs, upload_megakernel_arrays};
use super::launch::RenderKernel;
use super::megakernel_aa::{AAResolve, launch_samples};
use super::megakernel_arrays::{
    channel_flags, floats, geom_data_poses, ints, light_data_poses, reals, shading_words,
};
use super::{MEGAKERNEL_ENTRY, MEGAKERNEL_SOURCE};

// 拆分①：上传段以 `#[path]` 在本文件内声明为子模块（卡片 scope；`lighting.rs` 同款先例）。
#[path = "megakernel_uploads.rs"]
mod megakernel_uploads;

// R06-S2：宿主 flex 通道（`FlexHostInputs` + 校验 + 上传）以 `#[path]` 挂在本文件下
// （同款先例；`megakernel.rs` 450 死线、`megakernel_uploads.rs` 480 行都塞不下）。
#[path = "flex_channel.rs"]
mod flex_channel;
pub use flex_channel::FlexHostInputs;

// R06-S1a：81 槽实参装配段以 `#[path]` 在本文件内声明为子模块（卡片 scope；uploads 同款先例）。
#[path = "megakernel_args.rs"]
mod megakernel_args;

const BLOCK: u32 = 256;

/// 一次发射的可选通道（R06-S2 抽参以守 clippy 参数上限）：方案 L 的逐 world 选择与宿主 flex 通道。
///
/// 两者都为 `None` 时与 S7b2/S7b3 的全 world 单次发射逐字节等价。
struct RenderPass<'a> {
    /// `Some(world)` = 逐 world 发射（方案 L v1）；`None` = 全 world 一次发射。
    world: Option<usize>,
    /// 宿主 flex 通道；`None` 与 S7b2 等价（S7b2 的 1 元素占位）。
    flex: Option<&'a FlexHostInputs<'a>>,
}

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
        self.render_impl(
            context,
            fields,
            model,
            camera_poses,
            rays,
            RenderPass {
                world: None,
                flex: None,
            },
        )
    }

    /// R06-S2：带显式宿主 flex 通道发射（`None` 与 [`Self::render`] 逐字节等价）。
    ///
    /// 通道只做宿主校验 + 上传 + 实参形状口径；设备侧 flex 分支留 S3（`.cu` 零改动），
    /// model/context 的真实数据源归 R06-S2b。逐 world 入口（[`Self::render_world`]）本片
    /// 不支持 flex 通道（留 L v2/K）。
    pub fn render_with_flex(
        &self,
        context: &RenderContext,
        fields: &RenderFields,
        model: &CameraModel,
        camera_poses: &[CameraPose],
        rays: &RayBuffer,
        flex: Option<&FlexHostInputs<'_>>,
    ) -> Result<(), LauncherError> {
        self.render_impl(
            context,
            fields,
            model,
            camera_poses,
            rays,
            RenderPass { world: None, flex },
        )
    }

    /// 单 world 发射（per-world 方案 L）：发射维 `launch_worlds=1`，相机只喂该 world 行，
    /// 输出定位由 `adr + world×pitch` 位移完成（不做 sub-buffer 视图）。
    pub(crate) fn render_world(
        &self,
        context: &RenderContext,
        fields: &RenderFields,
        model: &CameraModel,
        camera_poses: &[CameraPose],
        rays: &RayBuffer,
        world: usize,
    ) -> Result<(), LauncherError> {
        self.render_impl(
            context,
            fields,
            model,
            camera_poses,
            rays,
            RenderPass {
                world: Some(world),
                flex: None,
            },
        )
    }

    fn render_impl(
        &self,
        context: &RenderContext,
        fields: &RenderFields,
        model: &CameraModel,
        camera_poses: &[CameraPose],
        rays: &RayBuffer,
        pass: RenderPass<'_>,
    ) -> Result<(), LauncherError> {
        let RenderPass { world, flex } = pass;
        let resolved = context.resolved();
        let layout = &resolved.output;
        let nsamples = layout.nsamples;
        if nsamples == 0 {
            return Err(LauncherError::InvalidArray("nsamples"));
        }
        if world.is_some() && nsamples != 1 {
            return Err(LauncherError::Unsupported(
                "逐 world 只支持 nsamples=1（AA+per-world 留 L v2/K）",
            ));
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
        if ints(fields, "light_mode")?.iter().any(|mode| *mode != 0) {
            return Err(LauncherError::Unsupported(
                "S7b3 只支持 light_mode=0（固定灯）的 data 位姿推导",
            ));
        }
        let nworld = resolved.nworld;
        let launch_worlds = if world.is_some() { 1 } else { nworld };
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
        let (cam_xpos, cam_xmat) = match world {
            Some(world) => {
                let row = if camera_poses.len() == ncam {
                    camera_poses
                } else {
                    let start = world
                        .checked_mul(ncam)
                        .ok_or(LauncherError::Overflow("camera_pose_count"))?;
                    let end = start
                        .checked_add(ncam)
                        .ok_or(LauncherError::Overflow("camera_pose_count"))?;
                    camera_poses
                        .get(start..end)
                        .ok_or(LauncherError::InvalidArray("camera_pose_count"))?
                };
                super::world_camera::world_camera_pose_arrays(row, 1, ncam)?
            }
            None => super::world_camera::world_camera_pose_arrays(camera_poses, nworld, ncam)?,
        };

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
            // R06-S2：宿主 flex 通道（`None` = S7b2 等价占位；数据源接线归 S2b）。
            flex,
            // 方案 L：单 world 发射时把输出 adr 平移到该 world 的区段（`world × pitch`）。
            adr_world: world,
        })?;
        let u = &uploads;
        // ---- 81 槽实参（顺序 = S7a MEGAKERNEL_SLOTS）：装配段拆分至 `megakernel_args.rs`（R06-S1a） ----
        let mut args: Vec<RenderArg> = Vec::with_capacity(MEGAKERNEL_SLOTS.len());
        megakernel_args::push_megakernel_args(
            &mut args,
            &megakernel_args::MegakernelArgsInputs {
                uploads: u,
                context,
                use_shadows: resolved.use_shadows,
                nrender,
                ngeom,
                nworld,
                launch_worlds,
                total_rays,
                expected_rays,
                ncam,
                rgb_pixels: layout.rgb_pixels,
                depth_pixels: layout.depth_pixels,
                seg_slots: layout.seg_slots,
                flags_word,
            },
        )?;

        let grid = u32::try_from((launch_worlds * total_rays).div_ceil(BLOCK as usize))
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
