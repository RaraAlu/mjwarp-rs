//! 元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::launcher_meta::AppliedForceDeltaMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::AppliedForceValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::StaticAppliedForceMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::applied_force_delta_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::applied_force_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForceMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) dims: [u32; 2],
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticAppliedForceMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl AppliedForceMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        dims: [u32; 2],
    ) -> Result<Self, TransferError> {
        StaticAppliedForceMetadata::validate(&values, &dims)
            .map_err(applied_force_delta_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且设备跨度来自该缓冲。
        let checked = unsafe {
            Checked::<StaticAppliedForceMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(applied_force_delta_launch_error)?;
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

    pub(super) fn validate_proof(
        &self,
        dims: [u32; 4],
    ) -> Result<Checked<AppliedForceValidateMetadata>, TransferError> {
        if self.dims != dims[..2] {
            return Err(InputError::InvalidDimension {
                field: "applied_force_validate_static_dims",
            }
            .into());
        }
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(applied_force_validate_launch_error)?;
        // SAFETY: 私有包装保留上传值、设备跨度及写入代际。
        unsafe {
            Checked::<AppliedForceValidateMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(applied_force_validate_launch_error)
    }

    pub(super) fn delta_proof(
        &self,
        dims: [u32; 4],
    ) -> Result<Checked<AppliedForceDeltaMetadata>, TransferError> {
        if self.dims != dims[..2] {
            return Err(InputError::InvalidDimension {
                field: "applied_force_delta_static_dims",
            }
            .into());
        }
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(applied_force_delta_launch_error)?;
        // SAFETY: 私有包装保留上传值、设备跨度及写入代际。
        unsafe {
            Checked::<AppliedForceDeltaMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(applied_force_delta_launch_error)
    }

    #[cfg(test)]
    pub(super) fn write_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    pub(super) fn corrupt_device_for_validator_test(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        // 仅测试故意绕过代际失效，让设备验证器接收安全的坏值。
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    pub(super) fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入同步完成且跨度未变。
        self.checked = unsafe {
            Checked::<StaticAppliedForceMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(applied_force_delta_launch_error)?;
        Ok(())
    }
}
