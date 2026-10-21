//! 混合求解数据写入、缓存与回读。

use super::*;

impl SparseMassMixedSolveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    pub(super) fn after_matrix(
        &mut self,
        result: Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.mass.write_matrix(values);
        self.after_matrix(result)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let result = self.mass.write_world_matrix(world, values);
        self.after_matrix(result)
    }
    pub(super) fn check_rhs(values: &[f32], expected: usize) -> Result<(), InputError> {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "mixed_solve_rhs",
                expected,
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "mixed_solve_rhs",
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
    pub(super) fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
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
        crate::physics::check_device_values(&values, "mixed_solve_solution")?;
        Ok(values)
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn cached_factor(&self) -> Result<SparseMassMixedFactorOutput, TransferError> {
        if !self.mass.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve_factor",
            });
        }
        let values = self.mass.checked_values()?;
        let input = &self.mass.sparse.input;
        let f = input.fields();
        for world in 0..self.worlds() {
            let first = 4 + self.mass.layout.world_elements(world)?.start + input.qld_block_total();
            for block in input
                .blocks()
                .iter()
                .filter(|b| b.route != SparseMassRoute::SmallCholesky)
            {
                for dof in block.start..block.start + block.size {
                    let value = values[first + (f.m_rowadr[dof] + f.m_rownnz[dof] - 1) as usize];
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof,
                            value: f64::from(value),
                        });
                    }
                }
            }
        }
        Ok(SparseMassMixedFactorOutput {
            input: input.clone(),
            layout: self.mass.layout,
            values,
        })
    }
    pub fn readback(&self) -> Result<SparseMassMixedSolveOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let factor = self.cached_factor()?;
            let mut rhs = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
            self.rhs.read_range_into(0, &mut rhs)?;
            crate::physics::check_device_values(&rhs, "mixed_solve_rhs")?;
            Ok(SparseMassMixedSolveOutput {
                factor,
                layout: self.layout,
                rhs_count: self.rhs_count,
                rhs,
                values: self.checked_solution()?,
            })
        }
    }
}
