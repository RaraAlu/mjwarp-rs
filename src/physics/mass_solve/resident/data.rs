//! 常驻分解数据写入与回读。

use super::super::MassSolveOutput;
#[cfg(feature = "cuda-probe")]
use super::super::check_result;
use super::*;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;

impl MassSolveData {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn rhs_count(&self) -> usize {
        self.layout.rhs_count
    }
    pub(super) fn invalidate_factor(&mut self) {
        self.factor_ready = false;
        self.ready = false;
    }
    fn after_position_write(
        &mut self,
        result: Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.invalidate_factor();
        }
        result
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_qpos(values);
        self.after_position_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_world_qpos(world, values);
        self.after_position_write(result)
    }
    pub fn write_rhs(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.layout.check_rhs(values)?;
        self.write_checked_rhs(0, values)
    }
    pub fn write_world_rhs(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.rhs.world_elements(world)?;
        MassSolveLayout::new(1, self.nv(), self.rhs_count())?.check_rhs(values)?;
        self.write_checked_rhs(range.start, values)
    }
    fn write_checked_rhs(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.rhs.write_range(4 + offset, values)
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn check_factor(&self) -> Result<(), TransferError> {
        let mut values = crate::runtime::host_staging::<f32>(self.factor_layout.guarded)?;
        self.factor.read_range_into(0, &mut values)?;
        check_result(self.factor_layout, &values)
    }
    pub fn readback(&self) -> Result<MassSolveOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "resident_mass_solve",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.matrix.readback()?;
            self.check_factor()?;
            let mut rhs =
                crate::runtime::host_staging::<f32>(self.layout.rhs.total_elements() + 8)?;
            self.rhs.read_range_into(0, &mut rhs)?;
            crate::physics::check_device_values(&rhs, "resident_mass_solve_rhs")?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            check_result(self.layout, &values)?;
            Ok(MassSolveOutput {
                layout: self.layout,
                values,
                blocks: self.blocks.clone(),
            })
        }
    }
}
