//! 借用矩阵计划实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::SparseMassInput;
use std::sync::Arc;

use super::super::{MassMultiplyFormat, layouts};
#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
use super::*;

impl<'plan> MassMatrixMultiplyPlan<'plan> {
    pub fn new(
        matrix: &'plan MassMatrixPlan,
        input: SparseMassInput,
        format: MassMultiplyFormat,
    ) -> Result<Self, TransferError> {
        let (nb, n, session, parents) = matrix.multiply_context()?;
        let s = packed(nb, &parents, &input)?;
        if n != s.nv {
            return Err(TransferError::ModelMismatch);
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (s, session, format);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let code = format!(
                "#define BORROW_NV {n}\n#define BORROW_NB {nb}\n#define BORROW_COUNT {}\n#define BORROW_DENSE {}\n{CUDA}",
                s.meta[1],
                usize::from(format == MassMultiplyFormat::Dense)
            );
            Ok(Self {
                matrix,
                spec: Arc::new(s),
                session: session.clone(),
                kernels: [
                    SynchronousKernel::compile(&session, &code, "borrowed_mass_validate")?,
                    SynchronousKernel::compile(&session, &code, "borrowed_mass_multiply")?,
                ],
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<MassMatrixMultiplyData, TransferError> {
        let s = &self.spec;
        let (_, vl) = layouts(s.nv, 10 * s.nbody + s.nv * s.nv, worlds)?;
        self.spec
            .meta
            .len()
            .checked_add(worlds)
            .and_then(|v| v.checked_add(8))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "borrowed_mul_m_metadata",
            })?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = vl;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut meta = vec![-131072; 4];
            meta.extend(&s.meta);
            meta.resize(meta.len() + worlds, 0);
            meta.extend([-131072; 4]);
            let alloc = |count: usize, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            Ok(MassMatrixMultiplyData {
                spec: self.spec.clone(),
                vector_layout: vl,
                metadata: self.session.upload(&meta)?,
                vector: alloc(vl.total_elements(), 0.0)?,
                current: alloc(vl.total_elements(), 0.0)?,
                staged: alloc(vl.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update<'borrow>(
        &self,
        data: &'borrow mut MassMatrixMultiplyData,
        source: &'borrow MassMatrixData,
        skip: Option<&[bool]>,
    ) -> Result<MassMatrixMultiplyView<'borrow>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec) {
            return Err(TransferError::ModelMismatch);
        }
        self.matrix.check_data(source)?;
        if source.worlds() != data.worlds() {
            return Err(InputError::LengthMismatch {
                field: "borrowed_mul_m_worlds",
                expected: data.worlds(),
                actual: source.worlds(),
            }
            .into());
        }
        if let Some(v) = skip
            && v.len() != data.worlds()
        {
            return Err(InputError::LengthMismatch {
                field: "borrowed_mul_m_skip",
                expected: data.worlds(),
                actual: v.len(),
            }
            .into());
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // This borrow is checked afresh on every call. No cached pointer,
            // exported integer tag, or solver-factor lifetime substitutes it.
            let m = source.matrix_input()?;
            let stride = 10 * self.spec.nbody + self.spec.nv * self.spec.nv;
            if m.len() != stride * data.worlds() + 8 {
                return Err(TransferError::ModelMismatch);
            }
            let so = 4 + self.spec.meta.len();
            let flags = skip.map_or_else(
                || vec![0; data.worlds()],
                |v| v.iter().map(|&v| i32::from(v)).collect(),
            );
            data.metadata.write_range(so, &flags)?;
            read(
                &data.current,
                data.vector_layout.total_elements(),
                "borrowed_mul_m_current",
            )?;
            let n = self.spec.nv as u32;
            let nb = self.spec.nbody as u32;
            let w = data.worlds() as u32;
            // SAFETY: Actual same-plan source owns its guarded CRB-prefix plus
            // dense-M stride. Constant metadata and vector capacities match it.
            unsafe {
                self.kernels[0].launch(
                    &data.metadata,
                    m,
                    &data.vector,
                    &mut data.status,
                    [nb, n, so as u32, w],
                )?;
            }
            let status = read(&data.status, data.worlds(), "borrowed_mul_m_status")?;
            if let Some(index) = status[4..4 + data.worlds()].iter().position(|&v| v != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "borrowed_mul_m_source",
                    index,
                    reason: "nonfinite_guard_or_mapping",
                }
                .into());
            }
            // Only result payload is copied device-to-device to preserve skip.
            // M stays in the original producer allocation; guards stay local.
            data.staged.copy_range_from(
                4,
                &data.current,
                4,
                data.vector_layout.total_elements(),
            )?;
            // SAFETY: Validated metadata bounds every lower-M address and
            // column. Each thread owns one staged result; skipped worlds return.
            unsafe {
                self.kernels[1].launch(
                    &data.metadata,
                    m,
                    &data.vector,
                    &mut data.staged,
                    [nb, n, so as u32, w * n],
                )?;
            }
            read(
                &data.staged,
                data.vector_layout.total_elements(),
                "borrowed_Mqvel_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            let computed = flags.iter().map(|&v| v == 0).collect();
            Ok(MassMatrixMultiplyView {
                source,
                data,
                computed,
            })
        }
    }
}
