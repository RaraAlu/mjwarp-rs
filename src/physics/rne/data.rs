//! RNE数据、快照与数值检查。

use super::*;

pub(super) fn check_values(
    field: &'static str,
    expected: usize,
    values: &[f32],
) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite { field, index });
    }
    Ok(())
}

impl RneData {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }

    pub(super) fn after_write(
        &mut self,
        result: Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }

    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qpos(values);
        self.after_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qpos(world, values);
        self.after_write(result)
    }
    pub fn write_qvel(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qvel(values);
        self.after_write(result)
    }
    pub fn write_world_qvel(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qvel(world, values);
        self.after_write(result)
    }
    pub fn write_qacc(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write_checked_qacc(0, self.layout.qacc.total_elements(), values)
    }
    pub fn write_world_qacc(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.qacc.world_elements(world)?;
        self.write_checked_qacc(range.start, range.len(), values)
    }
    pub(super) fn write_checked_qacc(
        &mut self,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check_values("rne_qacc", expected, values)?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.qacc.write_range(4 + offset, values)
    }
    pub fn readback(&self) -> Result<RneSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady { stage: "rne" });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.velocity.readback()?;
            let mut qacc =
                crate::runtime::host_staging::<f32>(self.layout.qacc.total_elements() + 8)?;
            self.qacc.read_range_into(0, &mut qacc)?;
            crate::physics::check_device_values(&qacc, "rne_qacc")?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            crate::physics::check_device_values(&values, "rne_output")?;
            Ok(RneSnapshot {
                layout: self.layout,
                flg_acc: self.flg_acc,
                values,
            })
        }
    }
}

impl RneSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn includes_acceleration(&self) -> bool {
        self.flg_acc
    }
    pub fn world(&self, world: usize) -> Result<RneWorld<'_>, InputError> {
        let range = self.layout.output.world_elements(world)?;
        let v = &self.values[4 + range.start..4 + range.end];
        Ok(RneWorld {
            cacc: &v[..6 * self.nbody()],
            cfrc_int: &v[6 * self.nbody()..12 * self.nbody()],
            qfrc_bias: &v[12 * self.nbody()..],
        })
    }
}
