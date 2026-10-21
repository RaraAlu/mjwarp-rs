//! 两类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::smooth_force_compute_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::smooth_force_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_compute_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::SmoothForceMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SmoothForceValidateKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) sleep_enabled: bool,
}

#[cfg(feature = "cuda-probe")]
impl SmoothForceValidateKernel {
    pub(super) fn compile(
        session: &TransferSession,
        source: &str,
        sleep_enabled: bool,
    ) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "smooth_force_validate")?,
            sleep_enabled,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &SmoothForceMetadataBuffer,
        forces: &TransferBuffer<f32>,
        sleep: &TransferBuffer<i32>,
        status: &mut TransferBuffer<f32>,
        abi: [u32; 4],
    ) -> Result<(), TransferError> {
        let dims = SmoothForceValidateDims {
            abi,
            sleep_enabled: self.sleep_enabled,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: forces.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: sleep.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        smooth_force_validate_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(smooth_force_validate_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名和网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), forces, sleep, status, abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SmoothForceComputeKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) sleep_enabled: bool,
}

#[cfg(feature = "cuda-probe")]
impl SmoothForceComputeKernel {
    pub(super) fn compile(
        session: &TransferSession,
        source: &str,
        sleep_enabled: bool,
    ) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "smooth_force")?,
            sleep_enabled,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &SmoothForceMetadataBuffer,
        forces: &TransferBuffer<f32>,
        sleep: &TransferBuffer<i32>,
        output: &mut TransferBuffer<f32>,
        abi: [u32; 4],
    ) -> Result<(), TransferError> {
        let dims = SmoothForceComputeDims {
            abi,
            sleep_enabled: self.sleep_enabled,
        };
        let proof = metadata.compute_proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: forces.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: sleep.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        smooth_force_compute_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(smooth_force_compute_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名和网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), forces, sleep, output, abi)
        }
    }
}
