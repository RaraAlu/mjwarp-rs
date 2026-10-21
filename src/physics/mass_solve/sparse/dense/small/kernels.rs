//! Small校验与因子内核包装。

#[cfg(feature = "cuda-probe")]
use super::buffer::SmallFactorMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
#[cfg(feature = "cuda-probe")]
use super::factor_meta::SmallFactorSpec;
#[cfg(feature = "cuda-probe")]
use super::factor_meta::small_factor_launch_error;
#[cfg(feature = "cuda-probe")]
use super::factor_meta::small_factor_lengths;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::SmallValidationSpec;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::small_validation_launch_error;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::small_validation_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SmallValidationKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) prefix: u32,
}

#[cfg(feature = "cuda-probe")]
impl SmallValidationKernel {
    pub(super) fn compile(session: &TransferSession, prefix: u32) -> Result<Self, TransferError> {
        small_validation_lengths(&[prefix, 0, 0, 1]).map_err(small_validation_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "small_prefix_validate")?,
            prefix,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &TransferBuffer<i32>,
        matrix: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "small_validation_worlds",
        })?;
        let dims = [self.prefix, 0, 0, worlds];
        let (matrix_len, status_len) =
            small_validation_lengths(&dims).map_err(small_validation_launch_error)?;
        if u64::try_from(matrix.len()).ok() != Some(matrix_len)
            || u64::try_from(status.len()).ok() != Some(status_len)
        {
            return Err(InputError::InvalidDimension {
                field: "small_validation_abi",
            }
            .into());
        }
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: matrix.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<SmallValidationSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[]))
            .map_err(small_validation_launch_error)?;
        // SAFETY: 类型化计划核对实际访问跨度和固定ABI。
        unsafe { self.kernel.launch(metadata, matrix, unused, status, dims) }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SmallFactorKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) size: u32,
    pub(super) prefix: u32,
    pub(super) first: u32,
    pub(super) count: u32,
}

#[cfg(feature = "cuda-probe")]
impl SmallFactorKernel {
    pub(super) fn compile(
        session: &TransferSession,
        metadata: &SmallFactorMetadataBuffer,
        size: u32,
        first: u32,
        count: u32,
    ) -> Result<Self, TransferError> {
        if metadata.group_size(first, count)? != size || !(2..=6).contains(&size) {
            return Err(InputError::InvalidDimension {
                field: "small_factor_static_group",
            }
            .into());
        }
        small_factor_lengths(&[metadata.prefix(), first, count, count])
            .map_err(small_factor_launch_error)?;
        let entry = match size {
            2 => "small_prefix_factor_2",
            3 => "small_prefix_factor_3",
            4 => "small_prefix_factor_4",
            5 => "small_prefix_factor_5",
            _ => "small_prefix_factor_6",
        };
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, entry)?,
            size,
            prefix: metadata.prefix(),
            first,
            count,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &SmallFactorMetadataBuffer,
        matrix: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        if metadata.prefix() != self.prefix
            || metadata.group_size(self.first, self.count)? != self.size
        {
            return Err(InputError::InvalidDimension {
                field: "small_factor_static_group",
            }
            .into());
        }
        let work = worlds.checked_mul(self.count).ok_or(InputError::Overflow {
            field: "small_factor_work",
        })?;
        let dims = [self.prefix, self.first, self.count, work];
        let (metadata_len, buffer_len) =
            small_factor_lengths(&dims).map_err(small_factor_launch_error)?;
        if u64::try_from(metadata.len())
            .ok()
            .is_none_or(|len| len < metadata_len)
            || u64::try_from(matrix.len()).ok() != Some(buffer_len)
            || u64::try_from(unused.len()).ok() != Some(buffer_len)
            || u64::try_from(output.len()).ok() != Some(buffer_len)
        {
            return Err(InputError::InvalidDimension {
                field: "small_factor_abi",
            }
            .into());
        }
        let proof = metadata.proof(dims)?;
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
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<SmallFactorSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(small_factor_launch_error)?;
        // SAFETY: 类型化计划核对全部实际跨度。
        unsafe {
            self.kernel
                .launch_small_factor(metadata.as_buffer(), matrix, unused, output, dims)
        }
    }
}
