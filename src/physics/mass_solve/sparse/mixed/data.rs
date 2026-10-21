//! 混合因子数据实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
use crate::model::BatchLayout;
#[cfg(feature = "cuda-probe")]
use crate::model::SparseMassRoute;

use super::*;

impl SparseMassMixedFactorData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub(super) fn invalidate(&mut self) {
        self.ready = false;
        self.gathered = false;
        self.sparse.ready = false;
        self.small.invalidate();
    }
    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.invalidate();
        }
        result
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        super::super::matrix_values(self.sparse.matrix_layout, values)?;
        self.write(0, values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let range = self.sparse.matrix_layout.world_elements(world)?;
        super::super::matrix_values(BatchLayout::new(1, self.sparse.input.nc(), 4)?, values)?;
        self.write(range.start, values)
    }
    fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        #[cfg(feature = "cuda-probe")]
        let result = self.matrix.write_range(4 + offset, values);
        #[cfg(not(feature = "cuda-probe"))]
        let result = {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        };
        self.after_write(result)
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut values = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut values)?;
        crate::physics::check_device_values(&values, "mixed_factor_output")?;
        let input = &self.sparse.input;
        let prefix = input.qld_block_total();
        for world in 0..self.worlds() {
            let first = 4 + self.layout.world_elements(world)?.start;
            for block in input
                .blocks()
                .iter()
                .filter(|b| b.route == SparseMassRoute::SmallCholesky)
            {
                let address = input.qld_block_adr()[block.start] as usize;
                for i in 0..block.size {
                    let value = values[first + address + i * block.size + i];
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof: block.start + i,
                            value: f64::from(value),
                        });
                    }
                }
            }
            for dof in 0..input.nv() {
                let value = values[first + prefix + input.nc() + dof];
                if value <= 0.0 {
                    return Err(TransferError::InvalidPivot {
                        world,
                        dof,
                        value: f64::from(value),
                    });
                }
            }
        }
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassMixedFactorOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.small.readback()?;
            self.sparse.readback()?;
            Ok(SparseMassMixedFactorOutput {
                input: self.sparse.input.clone(),
                layout: self.layout,
                values: self.checked_values()?,
            })
        }
    }
}
