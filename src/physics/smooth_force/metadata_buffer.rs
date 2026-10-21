//! 元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::SmoothForceComputeMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::SmoothForceValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::smooth_force_compute_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::smooth_force_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_compute_worlds;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SmoothForceMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) dims: SmoothForceValidateDims,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<SmoothForceValidateMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SmoothForceMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        dims: SmoothForceValidateDims,
    ) -> Result<Self, TransferError> {
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；跨度绑定该实际缓冲。
        let checked = unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(smooth_force_validate_launch_error)?;
        Ok(Self {
            buffer,
            values,
            dims,
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.buffer.len()
    }

    pub(super) fn proof(
        &self,
        dims: SmoothForceValidateDims,
    ) -> Result<&Checked<SmoothForceValidateMetadata>, TransferError> {
        if self.dims != dims {
            return Err(InputError::InvalidDimension {
                field: "smooth_force_validate_metadata_dims",
            }
            .into());
        }
        self.checked
            .binding(0)
            .validate_for(self.buffer.launcher_span()?, &dims)
            .map_err(smooth_force_validate_launch_error)?;
        Ok(&self.checked)
    }

    pub(super) fn compute_proof(
        &self,
        dims: SmoothForceComputeDims,
    ) -> Result<Checked<SmoothForceComputeMetadata>, TransferError> {
        let worlds =
            smooth_force_compute_worlds(&dims).map_err(smooth_force_compute_launch_error)?;
        let [nb, n, nt, _] = dims.abi;
        let validation_dims = SmoothForceValidateDims {
            abi: [nb, n, nt, worlds],
            sleep_enabled: dims.sleep_enabled,
        };
        if self.dims != validation_dims {
            return Err(InputError::InvalidDimension {
                field: "smooth_force_compute_metadata_dims",
            }
            .into());
        }
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &validation_dims)
            .map_err(smooth_force_compute_launch_error)?;
        // SAFETY: 私有包装保留实际缓冲、派生维度、宿主镜像和写入代际。
        unsafe {
            Checked::<SmoothForceComputeMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(smooth_force_compute_launch_error)
    }

    pub(super) fn write_range(
        &mut self,
        start: usize,
        values: &[i32],
    ) -> Result<(), TransferError> {
        let end = start
            .checked_add(values.len())
            .filter(|&end| end <= self.values.len())
            .ok_or(InputError::InvalidDimension {
                field: "smooth_force_metadata_write",
            })?;
        self.revision.invalidate();
        self.buffer.write_range(start, values)?;
        self.values[start..end].copy_from_slice(values);
        self.refresh_proof()
    }

    pub(super) fn refresh_proof(&mut self) -> Result<(), TransferError> {
        // SAFETY: 私有包装同步维护设备值、宿主镜像和写入代际。
        self.checked = unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(smooth_force_validate_launch_error)?;
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn corrupt_device_for_validator_test(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        // 测试故意保留有效证明，让设备验证器读取坏值。
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    pub(super) fn proof_snapshot_for_test(
        &self,
    ) -> Result<Checked<SmoothForceValidateMetadata>, TransferError> {
        // SAFETY: 测试快照绑定当前设备值、镜像和代际。
        unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(smooth_force_validate_launch_error)
    }

    #[cfg(test)]
    pub(super) fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        self.refresh_proof()
    }
}
