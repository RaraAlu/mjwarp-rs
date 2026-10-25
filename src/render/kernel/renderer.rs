//! S7b1 渲染执行器：把 context 拥有的场景/网格 BVH、S5 射线与输出缓冲打成
//! `render_depth_seg` 实参并发射（渲染/BVH S7b1）。
//!
//! 语义边界：
//! - 只发射主射线阶段内核（深度/分割）；RGB/AA/着色属 S7b2/S7b3。
//! - 场景/网格数据每帧从 **context 拥有的 BVH** 重新打包：`set_points` + 显式 `refit`
//!   之后才看得到新几何（S6b 语义：渲染不隐式刷新、不触发物理）。
//! - 输出缓冲来自 S6b `OutputResources`（禁用通道用 1 元素占位缓冲，内核按
//!   `render_depth`/`render_seg` 标志不写）；RGB 缓冲本片不写。
//! - 编译一律走 [`DepthSegKernel::compile`]（A′ 渲染门禁 `f32-render`，无旁路）。

use crate::model::render::RenderFields;
use crate::render::ImageOutput;
use crate::render::camera::CameraModel;
use crate::render::context::RenderContext;
use crate::render::rays::RayBuffer;
use crate::runtime::launcher::{
    LaunchBounds2, LaunchDims, LauncherError, RenderArg, RenderArrayValue, RenderElem, RenderScalar,
};
use crate::runtime::{TransferBuffer, TransferElement, TransferSession};

use super::DEPTH_SEG_SLOTS;
use super::DepthSegKernel;
use super::cast::{CameraDevice, CameraPose, MeshDevice, ModelRenderArrays, SceneDevice};

/// 渲染执行器；持有一个已编译内核（模块缓存）与传输会话。
pub struct DepthSegRenderer {
    session: TransferSession,
    kernel: DepthSegKernel,
}

impl DepthSegRenderer {
    /// 编译 `render_depth_seg`（需要 NVIDIA 驱动与 NVRTC）。
    pub fn new(session: &TransferSession) -> Result<Self, LauncherError> {
        Ok(Self {
            session: session.clone(),
            kernel: DepthSegKernel::compile(session)?,
        })
    }

    /// 已编译内核（证据读取 source/PTX 哈希用）。
    pub fn kernel(&self) -> &super::launch::RenderKernel {
        self.kernel.kernel()
    }

    /// 发射一帧：`rays=Some` 走 S5 预计算缓冲，`None` 走内核 dynamic 重建。
    ///
    /// `camera_poses` 按模型相机 id 索引，必须是上游 data 语义的 `d.cam_xpos/cam_xmat`
    /// （f32 位型）：固定相机可由 `CameraPose::from_fields` 派生，TARGETBODY 跟踪相机
    /// 无法从 `cam_quat` 重建，需调用方提供（见 `output::CameraData`）。
    pub fn render(
        &self,
        context: &RenderContext,
        fields: &RenderFields,
        model: &CameraModel,
        camera_poses: &[CameraPose],
        rays: Option<&RayBuffer>,
        cull_backfaces: bool,
    ) -> Result<(), LauncherError> {
        let resolved = context.resolved();
        let layout = &resolved.output;
        if layout.nsamples != 1 {
            return Err(LauncherError::Unsupported(
                "S7b1 只支持 nsamples=1（AA 属 S7b3）",
            ));
        }
        let nworld = resolved.nworld;
        let total_rays = layout.total_rays;
        let nrender = resolved.nrender();
        if nrender == 0 || total_rays == 0 {
            return Err(LauncherError::InvalidArray("render_cameras"));
        }
        let model_arrays = ModelRenderArrays::from_fields(fields)?;
        let camera = CameraDevice::build(model, resolved, camera_poses)?;
        let scene = SceneDevice::pack(
            context.scene_bvh(),
            context.scene(),
            &model_arrays,
            context.mesh_count(),
        )?;
        let mut meshes = Vec::with_capacity(context.mesh_count());
        for index in 0..context.mesh_count() {
            meshes.push(
                context
                    .mesh_bvh(index)
                    .ok_or(LauncherError::InvalidArray("mesh_index"))?,
            );
        }
        let mesh = MeshDevice::pack(&meshes)?;

        let (ray_dirs, ray_offsets, precomputed) = match rays {
            Some(rays) => {
                if rays.len() != total_rays {
                    return Err(LauncherError::InvalidArray("ray_count"));
                }
                let mut directions = Vec::with_capacity(3 * total_rays);
                let mut offsets = Vec::with_capacity(3 * total_rays);
                for (direction, offset) in rays.directions().iter().zip(rays.offsets()) {
                    directions.extend_from_slice(direction);
                    offsets.extend_from_slice(offset);
                }
                (directions, offsets, true)
            }
            None => (vec![0.0f32; 3], vec![0.0f32; 3], false),
        };
        let ray_len = if precomputed { total_rays } else { 1 };

        let depth_enabled = context.outputs().is_enabled(ImageOutput::Depth);
        let seg_enabled = context.outputs().is_enabled(ImageOutput::Segmentation);
        let depth_scratch = match depth_enabled {
            true => None,
            false => Some(upload(&self.session, &[0.0f32])?),
        };
        let seg_scratch = match seg_enabled {
            true => None,
            false => Some(upload(&self.session, &[0i32; 2])?),
        };

        let scene_meta = upload(&self.session, &scene.meta)?;
        let scene_params = upload(&self.session, &scene.params)?;
        let mesh_meta = upload(&self.session, &mesh.meta)?;
        let mesh_params = upload(&self.session, &mesh.params)?;
        let mesh_offsets = upload(&self.session, &mesh.flat_offsets())?;
        let geom_type = upload(&self.session, &model_arrays.geom_type)?;
        let geom_rgba = upload(&self.session, &model_arrays.geom_rgba)?;
        let geom_matid = upload(&self.session, &model_arrays.geom_matid)?;
        let mat_rgba = upload(&self.session, &model_arrays.mat_rgba)?;
        let cam_projection = upload(&self.session, &camera.projection)?;
        let cam_fovy = upload(&self.session, &camera.fovy)?;
        let cam_sensorsize = upload(&self.session, &camera.sensorsize)?;
        let cam_intrinsic = upload(&self.session, &camera.intrinsic)?;
        let cam_xpos = upload(&self.session, &camera.xpos)?;
        let cam_xmat = upload(&self.session, &camera.xmat)?;
        let cam_res = upload(&self.session, &flat_resolutions(&camera.res))?;
        let depth_adr = upload(&self.session, &camera.depth_adr)?;
        let seg_adr = upload(&self.session, &camera.seg_adr)?;
        let render_depth = upload(&self.session, &bool_words(&camera.render_depth))?;
        let render_seg = upload(&self.session, &bool_words(&camera.render_seg))?;
        let ray = upload(&self.session, &ray_dirs)?;
        let ray_offset = upload(&self.session, &ray_offsets)?;

        let ngeom = model_arrays.geom_type.len();
        let mut args = Vec::with_capacity(DEPTH_SEG_SLOTS.len());
        args.push(RenderArg::Bounds2D(LaunchBounds2::new([
            nworld, total_rays,
        ])?));
        args.push(array_arg(
            &scene_meta,
            RenderElem::I32,
            &[scene.meta.len()],
        )?);
        args.push(array_arg(
            &scene_params,
            RenderElem::F32,
            &[scene.params.len()],
        )?);
        args.push(array_arg(&mesh_meta, RenderElem::I32, &[mesh.meta.len()])?);
        args.push(array_arg(
            &mesh_params,
            RenderElem::F32,
            &[mesh.params.len()],
        )?);
        args.push(array_arg(&mesh_offsets, RenderElem::Vec2I, &[mesh.len()])?);
        args.push(array_arg(&geom_type, RenderElem::I32, &[ngeom])?);
        args.push(array_arg(&geom_rgba, RenderElem::Vec4, &[ngeom])?);
        args.push(array_arg(&geom_matid, RenderElem::I32, &[ngeom])?);
        args.push(array_arg(
            &mat_rgba,
            RenderElem::Vec4,
            &[model_arrays.mat_rgba.len() / 4],
        )?);
        args.push(array_arg(&cam_projection, RenderElem::I32, &[nrender])?);
        args.push(array_arg(&cam_fovy, RenderElem::F32, &[nrender])?);
        args.push(array_arg(&cam_sensorsize, RenderElem::Vec2, &[nrender])?);
        args.push(array_arg(&cam_intrinsic, RenderElem::Vec4, &[nrender])?);
        args.push(array_arg(&cam_xpos, RenderElem::Vec3, &[nrender])?);
        args.push(array_arg(&cam_xmat, RenderElem::Mat33, &[nrender])?);
        args.push(array_arg(&cam_res, RenderElem::Vec2I, &[nrender])?);
        args.push(array_arg(&depth_adr, RenderElem::I32, &[nrender])?);
        args.push(array_arg(&seg_adr, RenderElem::I32, &[nrender])?);
        args.push(array_arg(&render_depth, RenderElem::Bool, &[nrender])?);
        args.push(array_arg(&render_seg, RenderElem::Bool, &[nrender])?);
        args.push(array_arg(&ray, RenderElem::Vec3, &[ray_len])?);
        args.push(array_arg(&ray_offset, RenderElem::Vec3, &[ray_len])?);
        args.push(match &depth_scratch {
            Some(scratch) => array_arg(scratch, RenderElem::F32, &[1])?,
            None => RenderArg::Array(context.outputs().depth_render_arg()?),
        });
        args.push(match &seg_scratch {
            Some(scratch) => array_arg(scratch, RenderElem::Vec2I, &[1])?,
            None => RenderArg::Array(context.outputs().seg_render_arg()?),
        });

        let znear_bits = i32::from_ne_bytes((resolved.znear as f32).to_ne_bytes());
        let zfar_bits = i32::from_ne_bytes((resolved.zfar as f32).to_ne_bytes());
        let nrender_i32 = to_i32(nrender, "render_cameras")?;
        args.push(RenderArg::Scalar(RenderScalar::I32(nrender_i32)));
        args.push(RenderArg::Scalar(RenderScalar::I32(znear_bits)));
        args.push(RenderArg::Scalar(RenderScalar::I32(zfar_bits)));
        args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(
            layout.depth_pixels,
            "depth_pixels",
        )?)));
        args.push(RenderArg::Scalar(RenderScalar::I32(to_i32(
            layout.seg_slots,
            "seg_slots",
        )?)));
        args.push(RenderArg::Scalar(RenderScalar::Bool(precomputed)));
        args.push(RenderArg::Scalar(RenderScalar::Bool(cull_backfaces)));

        let work_items = nworld
            .checked_mul(total_rays)
            .ok_or(LauncherError::Overflow("render_work_items"))?;
        let grid = u32::try_from(work_items.div_ceil(256))
            .map_err(|_| LauncherError::Overflow("render_grid"))?;
        let dims = LaunchDims::new((grid, 1, 1), (256, 1, 1), 0)?;
        self.kernel.launch(&self.session, &args, dims)
    }
}

fn flat_resolutions(resolutions: &[[i32; 2]]) -> Vec<i32> {
    let mut flat = Vec::with_capacity(2 * resolutions.len());
    for resolution in resolutions {
        flat.extend_from_slice(resolution);
    }
    flat
}

/// bool 数组按 Warp `array_t<bool>` 的 1 字节元素打包（会话缓冲只有 4 字节类，
/// 故按小端把 4 个标志塞进一个 u32；设备侧按字节索引与数据逐位一致）。
fn bool_words(flags: &[i32]) -> Vec<u32> {
    let mut words = vec![0u32; flags.len().div_ceil(4)];
    for (index, flag) in flags.iter().enumerate() {
        if *flag != 0 {
            words[index / 4] |= 1u32 << (8 * (index % 4));
        }
    }
    words
}

fn upload<T: TransferElement>(
    session: &TransferSession,
    values: &[T],
) -> Result<TransferBuffer<T>, LauncherError> {
    session
        .upload(values)
        .map_err(|error| LauncherError::Driver(format!("depth-seg-upload: {error}")))
}

fn array_arg<T: TransferElement>(
    buffer: &TransferBuffer<T>,
    elem: RenderElem,
    shape: &[usize],
) -> Result<RenderArg, LauncherError> {
    let span = buffer
        .launcher_span()
        .map_err(|error| LauncherError::Driver(format!("depth-seg-span: {error}")))?;
    Ok(RenderArg::Array(RenderArrayValue::contiguous(
        span, elem, shape,
    )?))
}

fn to_i32(value: usize, field: &'static str) -> Result<i32, LauncherError> {
    i32::try_from(value).map_err(|_| LauncherError::Overflow(field))
}
