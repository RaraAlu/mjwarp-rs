//! 质量矩阵乘法计划实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::SparseMassInput;
use crate::runtime::TransferSession;
use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
use super::*;

impl MassMultiplyPlan {
    pub fn new(
        session: &TransferSession,
        input: SparseMassInput,
        format: MassMultiplyFormat,
    ) -> Result<Self, TransferError> {
        let meta = metadata(&input)?;
        let mc = match format {
            MassMultiplyFormat::Sparse => input.nc(),
            MassMultiplyFormat::Dense => {
                input
                    .nv()
                    .checked_mul(input.nv())
                    .ok_or(InputError::Overflow {
                        field: "mul_m_dense",
                    })?
            }
        };
        layouts(input.nv(), mc, 1)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, meta);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = format!(
                "#define MASS_NV {}\n#define MASS_DENSE {}\n{}",
                input.nv(),
                usize::from(format == MassMultiplyFormat::Dense),
                CUDA
            );
            Ok(Self {
                input: Arc::new(input),
                format,
                metadata: meta,
                session: session.clone(),
                kernels: [
                    SynchronousKernel::compile(session, &source, "mass_multiply_validate")?,
                    SynchronousKernel::compile(session, &source, "mass_multiply")?,
                ],
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<MassMultiplyData, TransferError> {
        let n = self.input.nv();
        let mc = if self.format == MassMultiplyFormat::Dense {
            n * n
        } else {
            self.input.nc()
        };
        let (matrix_layout, vector_layout) = layouts(n, mc, worlds)?;
        let mut meta = self.metadata.clone();
        if meta
            .len()
            .checked_add(worlds)
            .is_none_or(|v| v > i32::MAX as usize)
        {
            return Err(InputError::Overflow {
                field: "mul_m_skip",
            }
            .into());
        }
        meta.resize(meta.len() + worlds, 0);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix_layout, vector_layout, meta);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |len, initial| {
                let mut v = crate::runtime::host_staging::<f32>(len + 8)?;
                v.fill(-131072.0);
                v[4..4 + len].fill(initial);
                self.session.upload(&v)
            };
            Ok(MassMultiplyData {
                input: self.input.clone(),
                format: self.format,
                matrix_layout,
                vector_layout,
                skip_offset: self.metadata.len(),
                ready: false,
                metadata: self.session.upload(&meta)?,
                matrix: allocate(matrix_layout.total_elements(), 0.0)?,
                alternate: allocate(matrix_layout.total_elements(), f32::NAN)?,
                vector: allocate(vector_layout.total_elements(), 0.0)?,
                output: allocate(vector_layout.total_elements(), 0.0)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut MassMultiplyData) -> Result<(), TransferError> {
        self.update_with(data, None, false)
    }
    /// skip保留已有输出位型；alternate选择独立替代矩阵。
    pub fn update_with(
        &self,
        data: &mut MassMultiplyData,
        skip: Option<&[bool]>,
        alternate: bool,
    ) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) || self.format != data.format {
            return Err(TransferError::ModelMismatch);
        }
        if let Some(skip) = skip
            && skip.len() != data.worlds()
        {
            return Err(InputError::LengthMismatch {
                field: "mul_m_skip",
                expected: data.worlds(),
                actual: skip.len(),
            }
            .into());
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = alternate;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let flags = skip.map_or_else(
                || vec![0; data.worlds()],
                |v| v.iter().map(|&s| i32::from(s)).collect::<Vec<_>>(),
            );
            data.metadata.write_range(data.skip_offset, &flags)?;
            let n = self.input.nv() as u32;
            let mc = data.matrix_layout.elements_per_world() as u32;
            let w = data.worlds() as u32;
            let so = data.skip_offset as u32;
            let matrix = if alternate {
                &data.alternate
            } else {
                &data.matrix
            };
            // SAFETY: Same-plan buffers own checked guarded world strides. The
            // validator only reads inputs and writes one disjoint status/world.
            unsafe {
                self.kernels[0].launch(
                    &data.metadata,
                    matrix,
                    &data.vector,
                    &mut data.status,
                    [n, mc, so, w],
                )?;
            }
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "mul_m_status")?;
            if let Some(index) = status[4..4 + data.worlds()].iter().position(|&v| v != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "mul_m_source",
                    index,
                    reason: "nonfinite_or_guard",
                }
                .into());
            }
            // SAFETY: Metadata mirrors validated CSR addresses, or static dense
            // columns fit n. Each thread owns one output; skip never writes it.
            unsafe {
                self.kernels[1].launch(
                    &data.metadata,
                    matrix,
                    &data.vector,
                    &mut data.output,
                    [n, mc, so, w * n],
                )?;
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
