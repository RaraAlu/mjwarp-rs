//! RNE元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::RneBackwardDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::RneCaccDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::RneCfrcDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::RneQfrcDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::RneStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_rne_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::RneBackwardMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::RneCaccMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::RneCfrcMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::RneQfrcMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::StaticRneMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_backward_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_cacc_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_cfrc_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_qfrc_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct RneMetadataBuffer {
    pub(super) session: TransferSession,
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) dims: RneStaticDims,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticRneMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl RneMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        nbody: usize,
        nv: usize,
        depth: usize,
    ) -> Result<Self, TransferError> {
        let dims = RneStaticDims {
            nbody: u32::try_from(nbody).map_err(|_| InputError::Overflow {
                field: "rne_metadata_nbody",
            })?,
            nv: u32::try_from(nv).map_err(|_| InputError::Overflow {
                field: "rne_metadata_nv",
            })?,
            depth: u32::try_from(depth).map_err(|_| InputError::Overflow {
                field: "rne_metadata_depth",
            })?,
            metadata_elements: u64::try_from(values.len()).map_err(|_| InputError::Overflow {
                field: "rne_metadata",
            })?,
        };
        validate_rne_metadata(&values, dims).map_err(rne_qfrc_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<StaticRneMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(rne_qfrc_launch_error)?;
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

    pub(super) fn validate_base(
        &self,
    ) -> Result<crate::runtime::launcher::DeviceSpan, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(rne_qfrc_launch_error)?;
        Ok(span)
    }

    pub(super) fn cacc_proof(
        &self,
        flg_acc: bool,
        worlds: u32,
    ) -> Result<Checked<RneCaccMetadata>, TransferError> {
        let span = self.validate_base()?;
        let dims = RneCaccDims {
            flag: u32::from(flg_acc),
            worlds,
            static_dims: self.dims,
        };
        // SAFETY: 基础证明已核对真实缓冲、镜像和写入代际。
        unsafe {
            Checked::<RneCaccMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_cacc_launch_error)
    }

    pub(super) fn cfrc_proof(
        &self,
        worlds: u32,
    ) -> Result<Checked<RneCfrcMetadata>, TransferError> {
        let span = self.validate_base()?;
        let dims = RneCfrcDims {
            abi: [0, 0, 0, worlds],
            static_dims: self.dims,
        };
        // SAFETY: 基础证明已核对真实缓冲、镜像和写入代际。
        unsafe {
            Checked::<RneCfrcMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_cfrc_launch_error)
    }

    pub(super) fn qfrc_proof(
        &self,
        worlds: u32,
    ) -> Result<Checked<RneQfrcMetadata>, TransferError> {
        let span = self.validate_base()?;
        let dims = RneQfrcDims {
            abi: [0, 0, 0, worlds],
            static_dims: self.dims,
        };
        // SAFETY: 私有包装持有实际缓冲、完整镜像和写入代际。
        unsafe {
            Checked::<RneQfrcMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_qfrc_launch_error)
    }

    pub(super) fn backward_proof(
        &self,
        worlds: u32,
        level: u32,
    ) -> Result<Checked<RneBackwardMetadata>, TransferError> {
        let span = self.validate_base()?;
        let work = worlds
            .checked_mul(self.dims.nbody)
            .ok_or(InputError::Overflow {
                field: "rne_backward_work",
            })?;
        let dims = RneBackwardDims {
            abi: [self.dims.nbody, level, 0, work],
            worlds,
            static_dims: self.dims,
        };
        // SAFETY: 基础证明已核对真实缓冲、镜像和写入代际。
        unsafe {
            Checked::<RneBackwardMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_backward_launch_error)
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
            Checked::<StaticRneMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_qfrc_launch_error)?;
        Ok(())
    }
}
