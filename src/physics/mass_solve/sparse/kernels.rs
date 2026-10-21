//! 三类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassCopyDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassDiagDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseMassUpdateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::StaticSparseMassDiagMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::StaticSparseMassUpdateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_diag::sparse_mass_diag_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_diag::sparse_mass_diag_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_mass_copy_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_mass_copy_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_mass_copy_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_mass_update_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_mass_update_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::SparseMassCopyMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseMassCopyKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nv: u32,
    pub(super) nc: u32,
    pub(super) metadata_elements: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseMassCopyKernel {
    pub(super) fn compile(
        session: &TransferSession,
        metadata: &SparseMassCopyMetadataBuffer,
    ) -> Result<Self, TransferError> {
        let stride = metadata
            .nc
            .checked_add(metadata.nv)
            .ok_or(InputError::Overflow {
                field: "sparse_mass_copy_stride",
            })?;
        sparse_mass_copy_lengths(&SparseMassCopyDims {
            launch: [0, 0, stride, 1],
            nv: metadata.nv,
            nc: metadata.nc,
            metadata_elements: metadata.metadata_elements,
        })
        .map_err(sparse_mass_copy_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_mass_copy")?,
            nv: metadata.nv,
            nc: metadata.nc,
            metadata_elements: metadata.metadata_elements,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &SparseMassCopyMetadataBuffer,
        matrix: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if (metadata.nv, metadata.nc, metadata.metadata_elements)
            != (self.nv, self.nc, self.metadata_elements)
        {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_copy_static_dims",
            }
            .into());
        }
        let work = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "sparse_mass_copy_worlds",
        })?;
        let stride = self.nc.checked_add(self.nv).ok_or(InputError::Overflow {
            field: "sparse_mass_copy_stride",
        })?;
        let dims = SparseMassCopyDims {
            launch: [0, 0, stride, work],
            nv: self.nv,
            nc: self.nc,
            metadata_elements: self.metadata_elements,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: matrix.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        sparse_mass_copy_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(sparse_mass_copy_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度、可信索引和写入别名。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), matrix, unused, output, dims.launch)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseMassUpdateKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nv: u32,
    pub(super) nc: u32,
    pub(super) metadata_elements: u32,
    pub(super) update_count: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseMassUpdateKernel {
    pub(super) fn compile(
        session: &TransferSession,
        metadata: &SparseMassCopyMetadataBuffer,
    ) -> Result<Self, TransferError> {
        StaticSparseMassUpdateMetadata::validate(
            &metadata.values,
            &[
                metadata.nv,
                metadata.nc,
                metadata.metadata_elements,
                metadata.update_count,
            ],
        )
        .map_err(sparse_mass_update_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_mass_update")?,
            nv: metadata.nv,
            nc: metadata.nc,
            metadata_elements: metadata.metadata_elements,
            update_count: metadata.update_count,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &SparseMassCopyMetadataBuffer,
        unused: &TransferBuffer<f32>,
        unused2: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
        level: usize,
    ) -> Result<(), TransferError> {
        if (
            metadata.nv,
            metadata.nc,
            metadata.metadata_elements,
            metadata.update_count,
        ) != (self.nv, self.nc, self.metadata_elements, self.update_count)
        {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_update_static_dims",
            }
            .into());
        }
        let next_level = level.checked_add(1).ok_or(InputError::InvalidDimension {
            field: "sparse_mass_update_level",
        })?;
        let Some((&first, &end)) = metadata
            .level_offsets
            .get(level)
            .zip(metadata.level_offsets.get(next_level))
        else {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_update_level",
            }
            .into());
        };
        let count = end.checked_sub(first).filter(|&value| value != 0).ok_or(
            InputError::InvalidDimension {
                field: "sparse_mass_update_level",
            },
        )?;
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "sparse_mass_update_worlds",
        })?;
        let stride = self.nc.checked_add(self.nv).ok_or(InputError::Overflow {
            field: "sparse_mass_update_stride",
        })?;
        let work = worlds.checked_mul(count).ok_or(InputError::Overflow {
            field: "sparse_mass_update_work",
        })?;
        let dims = SparseMassUpdateDims {
            launch: [first, count, stride, work],
            nv: self.nv,
            nc: self.nc,
            metadata_elements: self.metadata_elements,
            update_count: self.update_count,
            worlds,
        };
        let proof = metadata.update_proof(dims, level)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused2.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        sparse_mass_update_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(sparse_mass_update_launch_error)?;
        // SAFETY: 类型化计划核对真实层级、跨度、代际与写入隔离。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), unused, unused2, output, dims.launch)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseMassDiagKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nv: u32,
    pub(super) nc: u32,
    pub(super) metadata_elements: u32,
    pub(super) update_count: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseMassDiagKernel {
    pub(super) fn compile(
        session: &TransferSession,
        metadata: &SparseMassCopyMetadataBuffer,
    ) -> Result<Self, TransferError> {
        StaticSparseMassDiagMetadata::validate(
            &metadata.values,
            &[
                metadata.nv,
                metadata.nc,
                metadata.metadata_elements,
                metadata.update_count,
            ],
        )
        .map_err(sparse_mass_diag_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_mass_diag")?,
            nv: metadata.nv,
            nc: metadata.nc,
            metadata_elements: metadata.metadata_elements,
            update_count: metadata.update_count,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &SparseMassCopyMetadataBuffer,
        unused: &TransferBuffer<f32>,
        unused2: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if (
            metadata.nv,
            metadata.nc,
            metadata.metadata_elements,
            metadata.update_count,
        ) != (self.nv, self.nc, self.metadata_elements, self.update_count)
        {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_diag_static_dims",
            }
            .into());
        }
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "sparse_mass_diag_worlds",
        })?;
        let stride = self.nc.checked_add(self.nv).ok_or(InputError::Overflow {
            field: "sparse_mass_diag_stride",
        })?;
        let work = worlds.checked_mul(self.nv).ok_or(InputError::Overflow {
            field: "sparse_mass_diag_work",
        })?;
        let dims = SparseMassDiagDims {
            launch: [0, 0, stride, work],
            nv: self.nv,
            nc: self.nc,
            metadata_elements: self.metadata_elements,
            update_count: self.update_count,
            worlds,
        };
        let proof = metadata.diag_proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused2.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        sparse_mass_diag_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(sparse_mass_diag_launch_error)?;
        // SAFETY: 类型化计划核对完整行布局、跨度、代际和写入隔离。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), unused, unused2, output, dims.launch)
        }
    }
}
