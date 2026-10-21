//! 稠密校验内核与聚集元数据缓冲。

#[cfg(feature = "cuda-probe")]
use super::gather_meta::DenseGatherLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::DenseGatherSpec;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::StaticDenseGatherMetadata;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::dense_gather_launch_error;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::dense_gather_lengths;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::DenseValidationSpec;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::dense_validation_launch_error;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::dense_validation_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct DenseValidationKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nc: u32,
    pub(super) prefix: u32,
}

#[cfg(feature = "cuda-probe")]
impl DenseValidationKernel {
    pub(super) fn compile(
        session: &TransferSession,
        nc: u32,
        prefix: u32,
    ) -> Result<Self, TransferError> {
        dense_validation_lengths(&[nc, prefix, 0, 1]).map_err(dense_validation_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_dense_validate")?,
            nc,
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
            field: "dense_validation_worlds",
        })?;
        let dims = [self.nc, self.prefix, 0, worlds];
        let (matrix_len, status_len) =
            dense_validation_lengths(&dims).map_err(dense_validation_launch_error)?;
        if u64::try_from(matrix.len()).ok() != Some(matrix_len)
            || u64::try_from(status.len()).ok() != Some(status_len)
        {
            return Err(InputError::InvalidDimension {
                field: "dense_validation_abi",
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
        Kernel::<DenseValidationSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[]))
            .map_err(dense_validation_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度和固定ABI。
        unsafe { self.kernel.launch(metadata, matrix, unused, status, dims) }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DenseGatherMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) nc: u32,
    pub(super) prefix: u32,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticDenseGatherMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl DenseGatherMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        nc: u32,
        prefix: u32,
    ) -> Result<Self, TransferError> {
        let dims = [nc, prefix];
        StaticDenseGatherMetadata::validate(&values, &dims).map_err(dense_gather_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且跨度来自该缓冲。
        let checked = unsafe {
            Checked::<StaticDenseGatherMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(dense_gather_launch_error)?;
        Ok(Self {
            buffer,
            values,
            nc,
            prefix,
            revision,
            checked,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.buffer.len()
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn proof(
        &self,
        dims: [u32; 4],
    ) -> Result<Checked<DenseGatherLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &[self.nc, self.prefix])
            .map_err(dense_gather_launch_error)?;
        // SAFETY: 独占包装保留上传值和写入代际。
        unsafe {
            Checked::<DenseGatherLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(dense_gather_launch_error)
    }

    #[cfg(test)]
    pub(super) fn write_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    pub(super) fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入同步完成且跨度未变。
        self.checked = unsafe {
            Checked::<StaticDenseGatherMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.nc, self.prefix],
                self.revision.clone(),
            )
        }
        .map_err(dense_gather_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DenseGatherKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nc: u32,
    pub(super) prefix: u32,
}

#[cfg(feature = "cuda-probe")]
impl DenseGatherKernel {
    pub(super) fn compile(
        session: &TransferSession,
        metadata: &DenseGatherMetadataBuffer,
    ) -> Result<Self, TransferError> {
        if metadata.nc > i32::MAX as u32 {
            return Err(InputError::InvalidDimension {
                field: "dense_gather_nc",
            }
            .into());
        }
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_dense_gather")?,
            nc: metadata.nc,
            prefix: metadata.prefix,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &DenseGatherMetadataBuffer,
        matrix: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if metadata.nc != self.nc || metadata.prefix != self.prefix {
            return Err(InputError::InvalidDimension {
                field: "dense_gather_static_dims",
            }
            .into());
        }
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "dense_gather_worlds",
        })?;
        let work = worlds
            .checked_mul(self.prefix)
            .ok_or(InputError::Overflow {
                field: "dense_gather_work",
            })?;
        let dims = [self.nc, self.prefix, 0, work];
        let (metadata_len, matrix_len, output_len) =
            dense_gather_lengths(&dims).map_err(dense_gather_launch_error)?;
        if u64::try_from(metadata.len()).ok() != Some(metadata_len)
            || u64::try_from(matrix.len()).ok() != Some(matrix_len)
            || unused.is_empty()
            || u64::try_from(output.len()).ok() != Some(output_len)
        {
            return Err(InputError::InvalidDimension {
                field: "dense_gather_abi",
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
        Kernel::<DenseGatherSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(dense_gather_launch_error)?;
        // SAFETY: 类型化计划核对实际跨度和可信索引。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), matrix, unused, output, dims)
        }
    }
}
