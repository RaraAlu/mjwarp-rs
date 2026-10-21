//! 质量求解元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::errors::rigid_mass_solve_launch_error;
#[cfg(feature = "cuda-probe")]
use super::metadata::RigidMassSolveDims;
#[cfg(feature = "cuda-probe")]
use super::proof::RigidMassSolveBaseMetadata;
#[cfg(feature = "cuda-probe")]
use super::proof::RigidMassSolveMetadata;
#[cfg(feature = "cuda-probe")]
use super::validate::validate_rigid_mass_solve_metadata;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct RigidMassSolveMetadataBuffer {
    pub(super) session: TransferSession,
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) dims: RigidMassSolveDims,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<RigidMassSolveBaseMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl RigidMassSolveMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        abi: [u32; 4],
    ) -> Result<Self, TransferError> {
        let dims = RigidMassSolveDims {
            abi,
            metadata_elements: u64::try_from(values.len()).map_err(|_| InputError::Overflow {
                field: "mass_solve_blocks",
            })?,
        };
        validate_rigid_mass_solve_metadata(&values, dims).map_err(rigid_mass_solve_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<RigidMassSolveBaseMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(rigid_mass_solve_launch_error)?;
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

    pub(super) fn proof(&self) -> Result<Checked<RigidMassSolveMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(rigid_mass_solve_launch_error)?;
        // SAFETY: 私有包装持有实际缓冲、完整镜像和写入代际。
        unsafe {
            Checked::<RigidMassSolveMetadata>::assume_uploaded(
                span,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(rigid_mass_solve_launch_error)
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
            Checked::<RigidMassSolveBaseMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(rigid_mass_solve_launch_error)?;
        Ok(())
    }
}
