//! Small因子元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::factor_meta::SmallFactorLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::factor_meta::StaticSmallFactorMetadata;
#[cfg(feature = "cuda-probe")]
use super::factor_meta::small_factor_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SmallFactorMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) prefix: u32,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticSmallFactorMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SmallFactorMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        prefix: u32,
    ) -> Result<Self, TransferError> {
        StaticSmallFactorMetadata::validate(&values, &prefix).map_err(small_factor_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且跨度来自该缓冲。
        let checked = unsafe {
            Checked::<StaticSmallFactorMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                prefix,
                revision.clone(),
            )
        }
        .map_err(small_factor_launch_error)?;
        Ok(Self {
            buffer,
            values,
            prefix,
            revision,
            checked,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.buffer.len()
    }

    pub(super) fn prefix(&self) -> u32 {
        self.prefix
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn group_size(&self, first: u32, count: u32) -> Result<u32, TransferError> {
        let first = usize::try_from(first).map_err(|_| InputError::InvalidDimension {
            field: "small_factor_group",
        })?;
        let count = usize::try_from(count).map_err(|_| InputError::InvalidDimension {
            field: "small_factor_group",
        })?;
        let end = first.checked_add(count).ok_or(InputError::Overflow {
            field: "small_factor_group",
        })?;
        if end > self.values.len() / 3 {
            return Err(InputError::InvalidDimension {
                field: "small_factor_group",
            }
            .into());
        }
        let mut sizes = self
            .values
            .as_chunks::<3>()
            .0
            .iter()
            .skip(first)
            .take(end - first)
            .map(|triple| triple[2]);
        let size = sizes.next().ok_or(InputError::InvalidDimension {
            field: "small_factor_group",
        })?;
        if sizes.any(|value| value != size) {
            return Err(InputError::InvalidTopology {
                field: "small_factor_group",
                index: first,
                reason: "mixed_static_size",
            }
            .into());
        }
        u32::try_from(size).map_err(|_| {
            InputError::InvalidDimension {
                field: "small_factor_group",
            }
            .into()
        })
    }

    pub(super) fn proof(
        &self,
        dims: [u32; 4],
    ) -> Result<Checked<SmallFactorLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.prefix)
            .map_err(small_factor_launch_error)?;
        // SAFETY: 独占包装保留上传值和代际。
        unsafe {
            Checked::<SmallFactorLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(small_factor_launch_error)
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
            Checked::<StaticSmallFactorMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.prefix,
                self.revision.clone(),
            )
        }
        .map_err(small_factor_launch_error)?;
        Ok(())
    }
}
