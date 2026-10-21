//! 常驻混合聚集与校验内核包装。

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::ResidentMixedGatherLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::StaticResidentMixedGatherMetadata;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_dims;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_launch_error;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_lengths;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_plan;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_dimension;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_launch_error;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_lengths;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMixedGatherMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) n: u32,
    pub(super) nc: u32,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticResidentMixedGatherMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMixedGatherMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        n: u32,
        nc: u32,
    ) -> Result<Self, TransferError> {
        StaticResidentMixedGatherMetadata::validate(&values, &[n, nc])
            .map_err(resident_mixed_gather_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且跨度来自该缓冲。
        let checked = unsafe {
            Checked::<StaticResidentMixedGatherMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                [n, nc],
                revision.clone(),
            )
        }
        .map_err(resident_mixed_gather_launch_error)?;
        Ok(Self {
            buffer,
            values,
            n,
            nc,
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn proof(
        &self,
        dims: [u32; 4],
    ) -> Result<Checked<ResidentMixedGatherLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &[self.n, self.nc])
            .map_err(resident_mixed_gather_launch_error)?;
        // SAFETY: 独占包装保留上传值和写入代际。
        unsafe {
            Checked::<ResidentMixedGatherLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(resident_mixed_gather_launch_error)
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
            Checked::<StaticResidentMixedGatherMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.n, self.nc],
                self.revision.clone(),
            )
        }
        .map_err(resident_mixed_gather_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMixedGatherKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nb: u32,
    pub(super) n: u32,
    pub(super) nc: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMixedGatherKernel {
    pub(super) fn compile(
        session: &TransferSession,
        metadata: &ResidentMixedGatherMetadataBuffer,
        nb: usize,
    ) -> Result<Self, TransferError> {
        let nb = resident_mixed_validation_dimension(nb, "resident_mixed_gather_nb")?;
        resident_mixed_gather_lengths(&[nb, metadata.n, metadata.nc, metadata.n])
            .map_err(resident_mixed_gather_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "resident_mixed_gather")?,
            nb,
            n: metadata.n,
            nc: metadata.nc,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &ResidentMixedGatherMetadataBuffer,
        source: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if metadata.n != self.n || metadata.nc != self.nc {
            return Err(InputError::InvalidDimension {
                field: "resident_mixed_gather_static_dims",
            }
            .into());
        }
        let worlds = resident_mixed_validation_dimension(worlds, "resident_mixed_gather_worlds")?;
        let dims = resident_mixed_gather_dims(self.nb, self.n, self.nc, worlds)
            .map_err(resident_mixed_gather_launch_error)?;
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: source.launcher_span()?,
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
        resident_mixed_gather_plan(&dims, &bindings, proof.binding(0))
            .map_err(resident_mixed_gather_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度和可信CSR索引。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), source, unused, output, dims)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMixedValidationKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nb: u32,
    pub(super) n: u32,
    pub(super) nc: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMixedValidationKernel {
    pub(super) fn compile(
        session: &TransferSession,
        nb: usize,
        n: usize,
        nc: usize,
    ) -> Result<Self, TransferError> {
        let nb = resident_mixed_validation_dimension(nb, "resident_mixed_validation_nb")?;
        let n = resident_mixed_validation_dimension(n, "resident_mixed_validation_n")?;
        let nc = resident_mixed_validation_dimension(nc, "resident_mixed_validation_nc")?;
        resident_mixed_validation_lengths(&[nb, n, nc, 1])
            .map_err(resident_mixed_validation_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "resident_mixed_validate")?,
            nb,
            n,
            nc,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &TransferBuffer<i32>,
        source: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let worlds =
            resident_mixed_validation_dimension(worlds, "resident_mixed_validation_worlds")?;
        self.launch_dims(
            metadata,
            source,
            unused,
            status,
            [self.nb, self.n, self.nc, worlds],
        )
    }

    pub(super) fn launch_dims(
        &self,
        metadata: &TransferBuffer<i32>,
        source: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: source.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        resident_mixed_validation_plan([self.nb, self.n, self.nc], &dims, &bindings)
            .map_err(resident_mixed_validation_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度和固定ABI。
        unsafe { self.kernel.launch(metadata, source, unused, status, dims) }
    }
}
