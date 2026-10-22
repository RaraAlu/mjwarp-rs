//! 常驻稀疏质量两类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentGatherDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::sparse_resident_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_resident_gather_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_resident_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::SparseResidentMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseResidentValidateKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl SparseResidentValidateKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "resident_sparse_validate")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &SparseResidentMetadataBuffer,
        source: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = SparseResidentValidateDims {
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
                span: source.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        sparse_resident_validate_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(sparse_resident_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), source, source, status, dims.abi())
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseResidentGatherKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl SparseResidentGatherKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "resident_sparse_gather")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &SparseResidentMetadataBuffer,
        source: &TransferBuffer<f32>,
        csr: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = SparseResidentGatherDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.gather_proof(dims)?;
        let abi = dims.abi().map_err(sparse_resident_launch_error)?;
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
                span: source.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: csr.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        sparse_resident_gather_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(sparse_resident_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), source, source, csr, abi)
        }
    }
}
