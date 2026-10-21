//! 混合求解校验元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::errors::mixed_solve_fused_launch_error;
#[cfg(feature = "cuda-probe")]
use super::errors::mixed_solve_validation_dimension;
#[cfg(feature = "cuda-probe")]
use super::errors::mixed_solve_validation_launch_error;
#[cfg(feature = "cuda-probe")]
use super::fused_meta::MixedSolveFusedDims;
#[cfg(feature = "cuda-probe")]
use super::fused_meta::MixedSolveFusedLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::fused_meta::StaticMixedSolveFusedMetadata;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::MixedSolveValidationLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::StaticMixedSolveValidationMetadata;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct MixedSolveValidationMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) n: u32,
    pub(super) stride: u32,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticMixedSolveValidationMetadata>,
    pub(super) fused_checked: Checked<StaticMixedSolveFusedMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl MixedSolveValidationMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        n: usize,
        stride: usize,
    ) -> Result<Self, TransferError> {
        let n = mixed_solve_validation_dimension(n, "mixed_solve_validation_n")?;
        let stride = mixed_solve_validation_dimension(stride, "mixed_solve_validation_stride")?;
        StaticMixedSolveValidationMetadata::validate(&values, &[n, stride])
            .map_err(mixed_solve_validation_launch_error)?;
        StaticMixedSolveFusedMetadata::validate(&values, &[n, stride])
            .map_err(mixed_solve_fused_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且跨度来自同一缓冲。
        let checked = unsafe {
            Checked::<StaticMixedSolveValidationMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                [n, stride],
                revision.clone(),
            )
        }
        .map_err(mixed_solve_validation_launch_error)?;
        // SAFETY: 同一上传快照覆盖融合入口全部索引。
        let fused_checked = unsafe {
            Checked::<StaticMixedSolveFusedMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                [n, stride],
                revision.clone(),
            )
        }
        .map_err(mixed_solve_fused_launch_error)?;
        Ok(Self {
            buffer,
            values,
            n,
            stride,
            revision,
            checked,
            fused_checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn element_count(&self) -> Result<u64, TransferError> {
        u64::try_from(self.values.len()).map_err(|_| {
            InputError::Overflow {
                field: "mixed_solve_validation_metadata",
            }
            .into()
        })
    }

    pub(super) fn proof(
        &self,
        dims: [u32; 4],
    ) -> Result<Checked<MixedSolveValidationLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &[self.n, self.stride])
            .map_err(mixed_solve_validation_launch_error)?;
        let expected_bytes = self
            .element_count()?
            .checked_mul(4)
            .ok_or(InputError::Overflow {
                field: "mixed_solve_validation_metadata_bytes",
            })?;
        if span.bytes() != expected_bytes {
            return Err(InputError::InvalidDimension {
                field: "mixed_solve_validation_metadata_span",
            }
            .into());
        }
        // SAFETY: 独占包装保留宿主快照和写入代际。
        unsafe {
            Checked::<MixedSolveValidationLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(mixed_solve_validation_launch_error)
    }

    pub(super) fn fused_proof(
        &self,
        abi: [u32; 4],
    ) -> Result<Checked<MixedSolveFusedLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.fused_checked
            .binding(0)
            .validate_for(span, &[self.n, self.stride])
            .map_err(mixed_solve_fused_launch_error)?;
        let metadata_elements = self.element_count()?;
        let expected_bytes = metadata_elements
            .checked_mul(4)
            .ok_or(InputError::Overflow {
                field: "mixed_solve_fused_metadata_bytes",
            })?;
        if span.bytes() != expected_bytes {
            return Err(InputError::InvalidDimension {
                field: "mixed_solve_fused_metadata_span",
            }
            .into());
        }
        let dims = MixedSolveFusedDims {
            abi,
            metadata_elements,
        };
        // SAFETY: 独占包装保留完整快照和写入代际。
        unsafe {
            Checked::<MixedSolveFusedLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(mixed_solve_fused_launch_error)
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
            Checked::<StaticMixedSolveValidationMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.n, self.stride],
                self.revision.clone(),
            )
        }
        .map_err(mixed_solve_validation_launch_error)?;
        // SAFETY: 恢复写入同步完成且融合跨度未变。
        self.fused_checked = unsafe {
            Checked::<StaticMixedSolveFusedMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.n, self.stride],
                self.revision.clone(),
            )
        }
        .map_err(mixed_solve_fused_launch_error)?;
        Ok(())
    }
}
