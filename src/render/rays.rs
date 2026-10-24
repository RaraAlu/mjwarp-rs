//! 相机空间射线生成（渲染/BVH S5）。
//!
//! 语义对应冻结上游 `create_render_context` 的预计算射线路径与渲染内核的动态射线路径：
//! 两者都按像素/子样本调用 `compute_ray`（`render_util.py`），samples_per_pixel=1 时参数
//! 完全相同，因此 `RayMode::Precomputed` 与 `RayMode::Dynamic` 在同一计划上应产生逐位相同
//! 的结果；本模块只返回相机空间 dir/offset（世界变换 `cam_xmat @ dir` 属 S7b）。
//!
//! 设备内核 `camera/cuda/build_rays.cu` 通过既有 `SynchronousKernel` 发射；输出为每射线
//! 6 个 f32（dir3+offset3 交错），宿主拆分后按 `[sample][camera][row][column]` 布局返回。
//! 空相机列表合法（返回空缓冲，不分配 GPU 缓冲）。

use crate::diagnostics::InputError;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
#[cfg(feature = "cuda-probe")]
use crate::render::camera::CameraModel;
use crate::render::camera::CameraPlan;

/// 射线生成模式；两种模式在 spp=1 时数值相同，动态模式用于 `use_precomputed_rays=false`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RayMode {
    /// 在 context 创建时预计算（`use_precomputed_rays=true`）。
    Precomputed,
    /// 在渲染时逐像素计算（`use_precomputed_rays=false`，只支持 spp=1）。
    Dynamic,
}

/// 相机空间射线集；长度为零表示没有相机/像素。
#[derive(Clone, Debug, PartialEq)]
pub struct RayBuffer {
    directions: Vec<[f32; 3]>,
    offsets: Vec<[f32; 3]>,
}

impl RayBuffer {
    /// 直接构造（host 测试/离线工具用）；长度不符拒绝。
    pub fn new(directions: Vec<[f32; 3]>, offsets: Vec<[f32; 3]>) -> Result<Self, InputError> {
        if directions.len() != offsets.len() {
            return Err(InputError::LengthMismatch {
                field: "ray_offset",
                expected: directions.len(),
                actual: offsets.len(),
            });
        }
        Ok(Self {
            directions,
            offsets,
        })
    }

    /// 射线条数。
    pub fn len(&self) -> usize {
        self.directions.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.directions.is_empty()
    }

    /// 相机空间方向。
    pub fn directions(&self) -> &[[f32; 3]] {
        &self.directions
    }

    /// 相机空间偏移（仅正交相机非零）。
    pub fn offsets(&self) -> &[[f32; 3]] {
        &self.offsets
    }

    /// 规范字节序（LE f32）：先全部 dir3、再全部 offset3；夹具 SHA-256 口径。
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.directions.len() * 24);
        for direction in &self.directions {
            for value in direction {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        for offset in &self.offsets {
            for value in offset {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes
    }
}

/// 计划对应的射线条数（`samples × total_rays`），溢出时报错。
pub fn planned_ray_count(plan: &CameraPlan) -> Result<usize, InputError> {
    plan.samples
        .checked_mul(plan.total_rays)
        .ok_or(InputError::Overflow { field: "ray_count" })
}

#[cfg(feature = "cuda-probe")]
pub use builder::{RAY_STRUCT_BYTES, RayBuilder};

#[cfg(feature = "cuda-probe")]
mod builder {
    use super::*;
    use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};

    const ENTRY: &str = "build_rays";
    const SOURCE: &str = include_str!("camera/cuda/build_rays.cu");
    /// 输出每射线的字节数（6 个 f32）。
    pub const RAY_STRUCT_BYTES: usize = 24;
    /// 参数每相机 7 个 f32（fovy、sensorsize 2、intrinsic 4），另有 znear 在开头。
    const PARAMS_PER_CAMERA: usize = 7;

    /// 相机空间射线生成器；编译一次可重复使用。
    pub struct RayBuilder {
        session: TransferSession,
        kernel: SynchronousKernel,
    }

    impl RayBuilder {
        /// 在给定传输会话上编译射线下发内核（需要 cuda-probe 与 NVRTC）。
        pub fn new(session: &TransferSession) -> Result<Self, TransferError> {
            Ok(Self {
                session: session.clone(),
                kernel: SynchronousKernel::compile(session, SOURCE, ENTRY)?,
            })
        }

        /// 按计划生成相机空间射线；空相机列表返回空缓冲（合法，不发射内核）。
        pub fn build(
            &self,
            model: &CameraModel,
            plan: &CameraPlan,
            mode: RayMode,
        ) -> Result<RayBuffer, TransferError> {
            self.validate(model, plan, mode)?;
            let ray_count = planned_ray_count(plan).map_err(TransferError::from)?;
            if ray_count == 0 {
                return RayBuffer::new(Vec::new(), Vec::new()).map_err(TransferError::from);
            }
            let cameras = plan.cameras.len();
            let meta_len = 4usize
                .checked_add(4 * cameras)
                .ok_or(InputError::Overflow {
                    field: "build_rays_meta",
                })?;
            let params_len = 1usize
                .checked_add(PARAMS_PER_CAMERA.checked_mul(cameras).ok_or(
                    InputError::Overflow {
                        field: "build_rays_params",
                    },
                )?)
                .ok_or(InputError::Overflow {
                    field: "build_rays_params",
                })?;
            let mut meta = vec![0i32; meta_len];
            meta[0] = i32::try_from(cameras).map_err(|_| InputError::Overflow {
                field: "build_rays_cameras",
            })?;
            meta[1] = i32::try_from(plan.samples).map_err(|_| InputError::Overflow {
                field: "build_rays_samples",
            })?;
            meta[2] = i32::try_from(plan.samples_per_pixel).map_err(|_| InputError::Overflow {
                field: "build_rays_spp",
            })?;
            meta[3] = i32::try_from(plan.total_rays).map_err(|_| InputError::Overflow {
                field: "build_rays_total",
            })?;
            for position in 0..cameras {
                let resolution = plan.resolutions[position];
                meta[4 + position] =
                    i32::try_from(plan.camera_pixel_offset(position)).map_err(|_| {
                        InputError::Overflow {
                            field: "build_rays_offset",
                        }
                    })?;
                meta[4 + cameras + position] =
                    i32::try_from(resolution[0]).map_err(|_| InputError::Overflow {
                        field: "build_rays_width",
                    })?;
                meta[4 + 2 * cameras + position] =
                    i32::try_from(resolution[1]).map_err(|_| InputError::Overflow {
                        field: "build_rays_height",
                    })?;
                let camera =
                    model
                        .camera(plan.cameras[position])
                        .ok_or(InputError::InvalidIndex {
                            field: "camera",
                            index: plan.cameras[position],
                            limit: model.ncam(),
                        })?;
                meta[4 + 3 * cameras + position] = camera.projection.code();
            }
            let mut params = vec![0.0f32; params_len];
            params[0] = plan.znear;
            for position in 0..cameras {
                let camera =
                    model
                        .camera(plan.cameras[position])
                        .ok_or(InputError::InvalidIndex {
                            field: "camera",
                            index: plan.cameras[position],
                            limit: model.ncam(),
                        })?;
                let base = 1 + PARAMS_PER_CAMERA * position;
                params[base] = camera.fovy as f32;
                params[base + 1] = camera.sensorsize[0];
                params[base + 2] = camera.sensorsize[1];
                params[base + 3..base + 7].copy_from_slice(&camera.intrinsic);
            }
            let out_len = ray_count.checked_mul(6).ok_or(InputError::Overflow {
                field: "build_rays_out",
            })?;
            let meta_buffer: TransferBuffer<i32> = self.session.upload(&meta)?;
            let params_buffer: TransferBuffer<f32> = self.session.upload(&params)?;
            let state_buffer: TransferBuffer<f32> = self.session.upload(&[0.0f32])?;
            let mut out = vec![0.0f32; out_len];
            let mut out_buffer: TransferBuffer<f32> = self.session.upload(&out)?;
            let work_items = u32::try_from(ray_count)
                .map_err(|_| InputError::Overflow { field: "ray_count" })?;
            // SAFETY: the kernel ABI and buffer ranges are validated above; the grid
            // covers exactly `ray_count` with a tid guard in build_rays.cu; all buffers
            // belong to this session and the launch is synchronous.
            unsafe {
                self.kernel.launch(
                    &meta_buffer,
                    &params_buffer,
                    &state_buffer,
                    &mut out_buffer,
                    [0, 0, 0, work_items],
                )?;
            }
            out_buffer.read_range_into(0, &mut out)?;
            let mut directions = Vec::with_capacity(ray_count);
            let mut offsets = Vec::with_capacity(ray_count);
            for index in 0..ray_count {
                let base = 6 * index;
                directions.push([out[base], out[base + 1], out[base + 2]]);
                offsets.push([out[base + 3], out[base + 4], out[base + 5]]);
            }
            RayBuffer::new(directions, offsets).map_err(TransferError::from)
        }

        fn validate(
            &self,
            model: &CameraModel,
            plan: &CameraPlan,
            mode: RayMode,
        ) -> Result<(), TransferError> {
            let cameras = plan.cameras.len();
            for (field, actual) in [
                ("resolutions", plan.resolutions.len()),
                ("render_rgb", plan.render_rgb.len()),
                ("render_depth", plan.render_depth.len()),
                ("render_seg", plan.render_seg.len()),
            ] {
                if actual != cameras {
                    return Err(InputError::LengthMismatch {
                        field,
                        expected: cameras,
                        actual,
                    }
                    .into());
                }
            }
            let expected_samples = (plan.samples_per_pixel as usize)
                .checked_mul(plan.samples_per_pixel as usize)
                .ok_or(InputError::Overflow {
                    field: "samples_per_pixel",
                })?;
            if plan.samples != expected_samples {
                return Err(InputError::InvalidDimension {
                    field: "ray_samples",
                }
                .into());
            }
            if mode == RayMode::Dynamic && plan.samples_per_pixel > 1 {
                return Err(InputError::InvalidDimension {
                    field: "dynamic_rays_require_spp1",
                }
                .into());
            }
            for (position, camera) in plan.cameras.iter().enumerate() {
                if *camera >= model.ncam() {
                    return Err(InputError::InvalidIndex {
                        field: "camera",
                        index: *camera,
                        limit: model.ncam(),
                    }
                    .into());
                }
                if plan.resolutions[position] == [0, 0] {
                    return Err(InputError::InvalidDimension {
                        field: "camera_resolution",
                    }
                    .into());
                }
            }
            Ok(())
        }
    }
}
