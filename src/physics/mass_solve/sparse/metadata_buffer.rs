//! 复制元数据缓冲与写入代际。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassCopyDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassCopyLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassDiagDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassDiagLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassUpdateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassUpdateLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::StaticSparseMassCopyMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::StaticSparseMassDiagMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::StaticSparseMassUpdateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::validate_sparse_mass_update_level;
#[cfg(feature = "cuda-probe")]
use super::launcher_diag::sparse_mass_diag_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_mass_copy_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_mass_update_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseMassCopyMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) nv: u32,
    pub(super) nc: u32,
    pub(super) metadata_elements: u32,
    pub(super) update_count: u32,
    pub(super) level_offsets: Vec<u32>,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticSparseMassCopyMetadata>,
    pub(super) checked_update: Checked<StaticSparseMassUpdateMetadata>,
    pub(super) checked_diag: Checked<StaticSparseMassDiagMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SparseMassCopyMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        nv: u32,
        nc: u32,
        level_offsets: &[i32],
    ) -> Result<Self, TransferError> {
        let metadata_elements = u32::try_from(values.len()).map_err(|_| InputError::Overflow {
            field: "sparse_mass_copy_metadata",
        })?;
        let level_offsets = level_offsets
            .iter()
            .map(|&value| {
                u32::try_from(value).map_err(|_| InputError::InvalidDimension {
                    field: "sparse_mass_update_levels",
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if level_offsets.first() != Some(&0)
            || level_offsets.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_update_levels",
            }
            .into());
        }
        let update_count = *level_offsets.last().ok_or(InputError::InvalidDimension {
            field: "sparse_mass_update_levels",
        })?;
        let static_dims = [nv, nc, metadata_elements];
        StaticSparseMassCopyMetadata::validate(&values, &static_dims)
            .map_err(sparse_mass_copy_launch_error)?;
        let update_static_dims = [nv, nc, metadata_elements, update_count];
        StaticSparseMassUpdateMetadata::validate(&values, &update_static_dims)
            .map_err(sparse_mass_update_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且跨度来自同一缓冲。
        let checked = unsafe {
            Checked::<StaticSparseMassCopyMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                static_dims,
                revision.clone(),
            )
        }
        .map_err(sparse_mass_copy_launch_error)?;
        // SAFETY: 同一上传保存完整CSR与生成后的更新元组。
        let checked_update = unsafe {
            Checked::<StaticSparseMassUpdateMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                update_static_dims,
                revision.clone(),
            )
        }
        .map_err(sparse_mass_update_launch_error)?;
        // SAFETY: 同一上传保存完整CSR和对角寻址所需行布局。
        let checked_diag = unsafe {
            Checked::<StaticSparseMassDiagMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                update_static_dims,
                revision.clone(),
            )
        }
        .map_err(sparse_mass_diag_launch_error)?;
        Ok(Self {
            buffer,
            values,
            nv,
            nc,
            metadata_elements,
            update_count,
            level_offsets,
            revision,
            checked,
            checked_update,
            checked_diag,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn proof(
        &self,
        dims: SparseMassCopyDims,
    ) -> Result<Checked<SparseMassCopyLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &[self.nv, self.nc, self.metadata_elements])
            .map_err(sparse_mass_copy_launch_error)?;
        // SAFETY: 独占包装保留上传值和写入代际。
        unsafe {
            Checked::<SparseMassCopyLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_copy_launch_error)
    }

    pub(super) fn update_proof(
        &self,
        dims: SparseMassUpdateDims,
        level: usize,
    ) -> Result<Checked<SparseMassUpdateLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked_update
            .binding(0)
            .validate_for(
                span,
                &[self.nv, self.nc, self.metadata_elements, self.update_count],
            )
            .map_err(sparse_mass_update_launch_error)?;
        validate_sparse_mass_update_level(&self.values, &dims, &self.level_offsets, level)
            .map_err(sparse_mass_update_launch_error)?;
        // SAFETY: 私有输入生成元组；层级切片与上传代际均已复核。
        unsafe {
            Checked::<SparseMassUpdateLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_update_launch_error)
    }

    pub(super) fn diag_proof(
        &self,
        dims: SparseMassDiagDims,
    ) -> Result<Checked<SparseMassDiagLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked_diag
            .binding(0)
            .validate_for(
                span,
                &[self.nv, self.nc, self.metadata_elements, self.update_count],
            )
            .map_err(sparse_mass_diag_launch_error)?;
        // SAFETY: 独占包装保留完整行布局、上传跨度和写入代际。
        unsafe {
            Checked::<SparseMassDiagLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_diag_launch_error)
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
            Checked::<StaticSparseMassCopyMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.nv, self.nc, self.metadata_elements],
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_copy_launch_error)?;
        // SAFETY: 恢复写入同步完成且完整元数据保持原值。
        self.checked_update = unsafe {
            Checked::<StaticSparseMassUpdateMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.nv, self.nc, self.metadata_elements, self.update_count],
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_update_launch_error)?;
        // SAFETY: 恢复写入同步完成且对角行布局保持原值。
        self.checked_diag = unsafe {
            Checked::<StaticSparseMassDiagMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.nv, self.nc, self.metadata_elements, self.update_count],
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_diag_launch_error)?;
        Ok(())
    }
}
