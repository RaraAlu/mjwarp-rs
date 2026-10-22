//! 借用质量矩阵两类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::borrowed_mass_compute_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::borrowed_mass_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::borrowed_mass_compute_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::borrowed_mass_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::BorrowedMassMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct BorrowedMassValidateKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl BorrowedMassValidateKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "borrowed_mass_validate")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &BorrowedMassMetadataBuffer,
        source: &TransferBuffer<f32>,
        vector: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = BorrowedMassValidateDims {
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
                span: source.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: vector.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        borrowed_mass_validate_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(borrowed_mass_validate_launch_error)?;
        let abi = dims.abi().map_err(borrowed_mass_validate_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), source, vector, status, abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct BorrowedMassComputeKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl BorrowedMassComputeKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "borrowed_mass_multiply")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &BorrowedMassMetadataBuffer,
        source: &TransferBuffer<f32>,
        vector: &TransferBuffer<f32>,
        staged: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = BorrowedMassComputeDims {
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
                span: source.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: vector.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: staged.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        borrowed_mass_compute_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(borrowed_mass_compute_launch_error)?;
        let abi = dims.abi().map_err(borrowed_mass_compute_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), source, vector, staged, abi)
        }
    }
}
