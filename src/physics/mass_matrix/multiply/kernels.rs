//! 质量矩阵乘法两类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::mass_multiply_compute_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::mass_multiply_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::mass_multiply_compute_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::mass_multiply_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::MassMultiplyMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct MassMultiplyValidateKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl MassMultiplyValidateKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "mass_multiply_validate")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &MassMultiplyMetadataBuffer,
        matrix: &TransferBuffer<f32>,
        vector: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = MassMultiplyValidateDims {
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
                span: matrix.launcher_span()?,
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
        mass_multiply_validate_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(mass_multiply_validate_launch_error)?;
        let abi = dims.abi().map_err(mass_multiply_validate_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), matrix, vector, status, abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct MassMultiplyComputeKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl MassMultiplyComputeKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "mass_multiply")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &MassMultiplyMetadataBuffer,
        matrix: &TransferBuffer<f32>,
        vector: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = MassMultiplyComputeDims {
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
                span: matrix.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: vector.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        mass_multiply_compute_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(mass_multiply_compute_launch_error)?;
        let abi = dims.abi().map_err(mass_multiply_compute_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), matrix, vector, output, abi)
        }
    }
}
