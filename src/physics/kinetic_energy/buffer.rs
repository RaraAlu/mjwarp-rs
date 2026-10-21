//! 动能元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::KineticLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::metadata::StaticKineticMetadata;
#[cfg(feature = "cuda-probe")]
use super::metadata::transfer_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct KineticEnergyMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) nv: u32,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticKineticMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl KineticEnergyMetadataBuffer {
    pub(in crate::physics) fn upload(
        session: &TransferSession,
        nv: u32,
    ) -> Result<Self, TransferError> {
        StaticKineticMetadata::validate(&[nv as i32], &nv).map_err(transfer_launch_error)?;
        let buffer = session.upload(&[nv as i32])?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成；跨度来自同一实际缓冲。
        let checked = unsafe {
            Checked::<StaticKineticMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &[nv as i32],
                nv,
                revision.clone(),
            )
        }
        .map_err(transfer_launch_error)?;
        Ok(Self {
            buffer,
            nv,
            revision,
            checked,
        })
    }

    pub(in crate::physics) fn len(&self) -> usize {
        self.buffer.len()
    }

    pub(in crate::physics) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(in crate::physics) fn nv(&self) -> u32 {
        self.nv
    }

    pub(in crate::physics) fn proof(
        &self,
        dims: [u32; 4],
    ) -> Result<Checked<KineticLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.nv)
            .map_err(transfer_launch_error)?;
        // SAFETY: 独占包装保留上传值；代际检查刚通过。
        unsafe {
            Checked::<KineticLaunchMetadata>::assume_uploaded(
                span,
                &[self.nv as i32],
                dims,
                self.revision.clone(),
            )
        }
        .map_err(transfer_launch_error)
    }

    #[cfg(test)]
    pub(in crate::physics) fn write_for_test(&mut self, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &[value])
    }

    #[cfg(test)]
    pub(in crate::physics) fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &[self.nv as i32])?;
        // SAFETY: 恢复写入同步完成；跨度仍属于原缓冲。
        self.checked = unsafe {
            Checked::<StaticKineticMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &[self.nv as i32],
                self.nv,
                self.revision.clone(),
            )
        }
        .map_err(transfer_launch_error)?;
        Ok(())
    }
}
