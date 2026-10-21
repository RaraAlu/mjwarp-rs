//! 偏置力元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::metadata::RneBiasDims;
#[cfg(feature = "cuda-probe")]
use super::metadata::validate_rne_bias_metadata;
#[cfg(feature = "cuda-probe")]
use super::spec::RneBiasMetadata;
#[cfg(feature = "cuda-probe")]
use super::spec::rne_bias_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct RneBiasMetadataBuffer {
    pub(super) session: TransferSession,
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) dims: RneBiasDims,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<RneBiasMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl RneBiasMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        nbody: usize,
        nv: usize,
    ) -> Result<Self, TransferError> {
        let dims = RneBiasDims {
            worlds: 1,
            nbody: u32::try_from(nbody).map_err(|_| InputError::Overflow {
                field: "rne_bias_metadata_nbody",
            })?,
            nv: u32::try_from(nv).map_err(|_| InputError::Overflow {
                field: "rne_bias_metadata_nv",
            })?,
            metadata_elements: u64::try_from(values.len()).map_err(|_| InputError::Overflow {
                field: "rne_bias_metadata",
            })?,
        };
        validate_rne_bias_metadata(&values, dims).map_err(rne_bias_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<RneBiasMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(rne_bias_launch_error)?;
        Ok(Self {
            session: session.clone(),
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

    pub(super) fn proof(&self, worlds: u32) -> Result<Checked<RneBiasMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(rne_bias_launch_error)?;
        let dims = RneBiasDims {
            worlds,
            ..self.dims
        };
        // SAFETY: 基础证明已核对真实缓冲、镜像和写入代际。
        unsafe {
            Checked::<RneBiasMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_bias_launch_error)
    }

    #[cfg(test)]
    pub(super) fn overwrite_for_test(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    pub(super) fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入已同步；跨度与镜像重新绑定。
        self.checked = unsafe {
            Checked::<RneBiasMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_bias_launch_error)?;
        Ok(())
    }
}
