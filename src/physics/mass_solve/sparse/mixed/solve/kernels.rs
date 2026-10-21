//! 混合求解校验与融合同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
#[cfg(feature = "cuda-probe")]
use super::errors::mixed_solve_fused_launch_error;
#[cfg(feature = "cuda-probe")]
use super::errors::mixed_solve_validation_dimension;
#[cfg(feature = "cuda-probe")]
use super::errors::mixed_solve_validation_launch_error;
#[cfg(feature = "cuda-probe")]
use super::fused_meta::MixedSolveFusedDims;
#[cfg(feature = "cuda-probe")]
use super::fused_spec::mixed_solve_fused_lengths;
#[cfg(feature = "cuda-probe")]
use super::fused_spec::mixed_solve_fused_plan;
#[cfg(feature = "cuda-probe")]
use super::validation_buffer::MixedSolveValidationMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::mixed_solve_validation_lengths;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::mixed_solve_validation_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct MixedSolveValidationKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) n: u32,
    pub(super) stride: u32,
}

#[cfg(feature = "cuda-probe")]
impl MixedSolveValidationKernel {
    pub(super) fn compile(
        session: &TransferSession,
        metadata: &MixedSolveValidationMetadataBuffer,
    ) -> Result<Self, TransferError> {
        mixed_solve_validation_lengths(&[metadata.n, metadata.stride, 1, 1])
            .map_err(mixed_solve_validation_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "mixed_solve_validate")?,
            n: metadata.n,
            stride: metadata.stride,
        })
    }

    #[cfg(test)]
    pub(super) fn dimensions(
        &self,
        rhs_count: usize,
        worlds: usize,
    ) -> Result<[u32; 4], TransferError> {
        let nr = mixed_solve_validation_dimension(rhs_count, "mixed_solve_validation_rhs_count")?;
        let worlds = mixed_solve_validation_dimension(worlds, "mixed_solve_validation_worlds")?;
        let dims = [self.n, self.stride, nr, worlds];
        mixed_solve_validation_lengths(&dims).map_err(mixed_solve_validation_launch_error)?;
        Ok(dims)
    }

    pub(super) fn launch(
        &self,
        metadata: &MixedSolveValidationMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: rhs.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: factor.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        mixed_solve_validation_plan(
            [self.n, self.stride],
            metadata.element_count()?,
            &dims,
            &bindings,
            proof.binding(0),
        )
        .map_err(mixed_solve_validation_launch_error)?;
        // SAFETY: 类型化计划核对全部跨度和元数据索引。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), rhs, factor, status, dims)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct MixedSolveFusedKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) n: u32,
    pub(super) stride: u32,
    pub(super) metadata_elements: u64,
}

#[cfg(feature = "cuda-probe")]
impl MixedSolveFusedKernel {
    pub(super) fn compile(
        session: &TransferSession,
        metadata: &MixedSolveValidationMetadataBuffer,
    ) -> Result<Self, TransferError> {
        let dims = MixedSolveFusedDims {
            abi: [metadata.n, metadata.stride, 1, 1],
            metadata_elements: metadata.element_count()?,
        };
        mixed_solve_fused_lengths(&dims).map_err(mixed_solve_fused_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "mixed_solve_fused")?,
            n: metadata.n,
            stride: metadata.stride,
            metadata_elements: metadata.element_count()?,
        })
    }

    pub(super) fn dimensions(
        &self,
        rhs_count: usize,
        worlds: usize,
    ) -> Result<[u32; 4], TransferError> {
        let nr = mixed_solve_validation_dimension(rhs_count, "mixed_solve_fused_rhs_count")?;
        let worlds = mixed_solve_validation_dimension(worlds, "mixed_solve_fused_worlds")?;
        let abi = [self.n, self.stride, nr, worlds];
        mixed_solve_fused_lengths(&MixedSolveFusedDims {
            abi,
            metadata_elements: self.metadata_elements,
        })
        .map_err(mixed_solve_fused_launch_error)?;
        Ok(abi)
    }

    pub(super) fn launch(
        &self,
        metadata: &MixedSolveValidationMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        abi: [u32; 4],
    ) -> Result<(), TransferError> {
        let proof = metadata.fused_proof(abi)?;
        let dims = *proof.dims();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: rhs.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: factor.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        mixed_solve_fused_plan([self.n, self.stride], &dims, &bindings, proof.binding(0))
            .map_err(mixed_solve_fused_launch_error)?;
        // SAFETY: 类型化计划证明完整元数据、跨度和128线程分组。
        unsafe {
            self.kernel
                .launch_mixed_solve(metadata.as_buffer(), rhs, factor, output, abi)
        }
    }
}
