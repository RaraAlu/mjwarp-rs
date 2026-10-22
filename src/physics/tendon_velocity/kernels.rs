//! 肌腱速度两类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::tendon_velocity_compute_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::tendon_velocity_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::tendon_velocity_compute_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::tendon_velocity_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::TendonVelocityMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityValidateKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl TendonVelocityValidateKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "tendon_velocity_validate")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &TendonVelocityMetadataBuffer,
        jacobian: &TransferBuffer<f32>,
        qvel: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = TendonVelocityValidateDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.validate_proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: jacobian.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: qvel.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        tendon_velocity_validate_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(tendon_velocity_validate_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), jacobian, qvel, status, dims.abi())
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityComputeKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl TendonVelocityComputeKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "tendon_velocity_compute")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &TendonVelocityMetadataBuffer,
        jacobian: &TransferBuffer<f32>,
        qvel: &TransferBuffer<f32>,
        staged: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = TendonVelocityComputeDims {
            static_dims: metadata.static_dims(),
            worlds,
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
                span: jacobian.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: qvel.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: staged.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        tendon_velocity_compute_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(tendon_velocity_compute_launch_error)?;
        let abi = dims.abi().map_err(tendon_velocity_compute_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), jacobian, qvel, staged, abi)
        }
    }
}
