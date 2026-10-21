//! 常驻稀疏求解数据与回读。

#[cfg(feature = "cuda-probe")]
use super::SparseMassFactorOutput;
use super::{SparseMassSolveData, SparseMassSolveOutput};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};

impl SparseMassSolveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    pub fn nv(&self) -> usize {
        self.mass.factor.input.nv()
    }
    fn after_qpos(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.mass.write_qpos(values);
        self.after_qpos(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.mass.write_world_qpos(world, values);
        self.after_qpos(result)
    }
    fn check_rhs(values: &[f32], expected: usize) -> Result<(), InputError> {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "sparse_solve_rhs",
                expected,
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "sparse_solve_rhs",
                    index,
                });
            }
        }
        Ok(())
    }
    pub fn write_rhs(&mut self, values: &[f32]) -> Result<(), TransferError> {
        Self::check_rhs(values, self.layout.total_elements())?;
        self.write(0, values)
    }
    pub fn write_world_rhs(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let r = self.layout.world_elements(world)?;
        Self::check_rhs(values, r.len())?;
        self.write(r.start, values)
    }
    fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.rhs.write_range(4 + offset, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn checked_solution(&self) -> Result<Vec<f32>, TransferError> {
        let mut values = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut values)?;
        crate::physics::check_device_values(&values, "sparse_solve_output")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassSolveOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_mass_solve",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // 只读取缓存因子；旧CSR与CRB不再决定解的寿命。
            // 守卫、全部因子、原主元与逆对角仍须有效。
            if !self.mass.factor.ready {
                return Err(TransferError::StageNotReady {
                    stage: "sparse_mass_factor",
                });
            }
            let f = &self.mass.factor;
            let mut values =
                crate::runtime::host_staging::<f32>(f.output_layout.total_elements() + 8)?;
            f.output.read_range_into(0, &mut values)?;
            crate::physics::check_device_values(&values, "sparse_solve_cached_factor")?;
            let fields = f.input.fields();
            for world in 0..self.worlds() {
                let start = 4 + f.output_layout.world_elements(world)?.start;
                for dof in 0..self.nv() {
                    let value =
                        values[start + (fields.m_rowadr[dof] + fields.m_rownnz[dof] - 1) as usize];
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof,
                            value: f64::from(value),
                        });
                    }
                }
            }
            let factor = SparseMassFactorOutput {
                input: f.input.clone(),
                layout: f.output_layout,
                values,
            };
            for world in 0..self.worlds() {
                for (dof, &value) in factor.world(world)?.diagonal_inverse.iter().enumerate() {
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof,
                            value: f64::from(value),
                        });
                    }
                }
            }
            let mut rhs = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
            self.rhs.read_range_into(0, &mut rhs)?;
            crate::physics::check_device_values(&rhs, "sparse_solve_rhs")?;
            Ok(SparseMassSolveOutput {
                factor,
                layout: self.layout,
                rhs_count: self.rhs_count,
                values: self.checked_solution()?,
            })
        }
    }
}
