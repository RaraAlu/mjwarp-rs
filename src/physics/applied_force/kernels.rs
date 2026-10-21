//! 三类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_meta::applied_force_delta_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::applied_force_publish_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::applied_force_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::AppliedForceDeltaSpec;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::AppliedForceValidateSpec;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::applied_force_publish_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::AppliedForceMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForceValidateKernel {
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl AppliedForceValidateKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "applied_force_validate")?,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &AppliedForceMetadataBuffer,
        geometry: &TransferBuffer<f32>,
        ft: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        let proof = metadata.validate_proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: geometry.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: ft.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<AppliedForceValidateSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(applied_force_validate_launch_error)?;
        // SAFETY: 类型化计划核对元数据、跨度、会话和256线程网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), geometry, ft, status, dims)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForceDeltaKernel {
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl AppliedForceDeltaKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "applied_force_delta")?,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &AppliedForceMetadataBuffer,
        geometry: &TransferBuffer<f32>,
        ft: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        let proof = metadata.delta_proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: geometry.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: ft.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<AppliedForceDeltaSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(applied_force_delta_launch_error)?;
        // SAFETY: 类型化计划核对元数据、跨度、会话和256线程网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), geometry, ft, output, dims)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForcePublishKernel {
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl AppliedForcePublishKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "applied_force_publish")?,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &TransferBuffer<i32>,
        current: &TransferBuffer<f32>,
        delta: &TransferBuffer<f32>,
        staged: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: current.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: delta.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: staged.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        applied_force_publish_plan(&dims, &bindings).map_err(applied_force_publish_launch_error)?;
        // SAFETY: 类型化计划核对四指针、跨度、会话、别名和256线程网格。
        unsafe { self.kernel.launch(metadata, current, delta, staged, dims) }
    }
}
