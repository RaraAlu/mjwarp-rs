//! 刚体质量矩阵元数据设备缓冲与测试失效入口。

use crate::diagnostics::TransferError;
use crate::runtime::launcher::{Checked, MetadataRevision};
use crate::runtime::{TransferBuffer, TransferSession};

use super::rigid::{
    RigidMassMatrixDims, RigidMassMatrixMetadata, rigid_mass_matrix_launch_error,
    validate_rigid_mass_matrix_metadata,
};

/// 持有设备拓扑、宿主镜像与写入代际。
#[cfg(feature = "cuda-probe")]
pub(super) struct RigidMassMatrixMetadataBuffer {
    pub(super) session: TransferSession,
    buffer: TransferBuffer<i32>,
    #[cfg(test)]
    values: Vec<i32>,
    #[cfg(test)]
    revision: MetadataRevision,
    pub(super) checked: Checked<RigidMassMatrixMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl RigidMassMatrixMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        abi: [u32; 4],
    ) -> Result<Self, TransferError> {
        let dims = RigidMassMatrixDims { abi };
        validate_rigid_mass_matrix_metadata(&values, dims)
            .map_err(rigid_mass_matrix_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<RigidMassMatrixMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(rigid_mass_matrix_launch_error)?;
        #[cfg(not(test))]
        let _ = (values, revision);
        Ok(Self {
            session: session.clone(),
            buffer,
            #[cfg(test)]
            values,
            #[cfg(test)]
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
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
            Checked::<RigidMassMatrixMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                *self.checked.dims(),
                self.revision.clone(),
            )
        }
        .map_err(rigid_mass_matrix_launch_error)?;
        Ok(())
    }
}
