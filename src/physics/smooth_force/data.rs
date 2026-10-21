//! 数据缓冲、回读与检查。

use super::*;

pub(super) fn check_force(
    values: &[f32],
    expected: usize,
    field: SmoothForceField,
) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field: field.name(),
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite {
            field: field.name(),
            index,
        });
    }
    Ok(())
}
impl SmoothForceData {
    pub fn worlds(&self) -> usize {
        self.layout.vector.worlds()
    }
    pub(in crate::physics) fn invalidate(&mut self) {
        self.ready = false;
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn passive_target(&mut self) -> &mut TransferBuffer<f32> {
        &mut self.forces
    }
    pub fn write_force(
        &mut self,
        field: SmoothForceField,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.write(field, 0, self.layout.vector.total_elements(), values)
    }
    pub fn write_world_force(
        &mut self,
        field: SmoothForceField,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.layout.vector.world_elements(world)?;
        self.write(field, r.start, r.len(), values)
    }
    pub(super) fn write(
        &mut self,
        field: SmoothForceField,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check_force(values, expected, field)?;
        let start = 4 + field.index() * self.layout.vector.total_elements() + offset;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = start;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.forces.write_range(start, values)
        }
    }
    pub fn write_awake(&mut self, values: &[i32]) -> Result<(), TransferError> {
        self.awake(0, self.layout.awake.total_elements(), values)
    }
    pub fn write_world_awake(&mut self, world: usize, values: &[i32]) -> Result<(), TransferError> {
        let r = self.layout.awake.world_elements(world)?;
        self.awake(r.start, r.len(), values)
    }
    pub(super) fn awake(
        &mut self,
        offset: usize,
        expected: usize,
        values: &[i32],
    ) -> Result<(), TransferError> {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "tree_awake",
                expected,
                actual: values.len(),
            }
            .into());
        }
        let start = 4 + self.input.nbody() + self.input.nv() + offset;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = start;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.metadata.write_range(start, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut v = crate::runtime::host_staging::<f32>(self.layout.vector.total_elements() + 8)?;
        self.output.read_range_into(0, &mut v)?;
        crate::physics::check_device_values(&v, "qfrc_smooth")?;
        Ok(v)
    }
    pub fn readback(&self) -> Result<SmoothForceOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "qfrc_smooth",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(SmoothForceOutput {
                layout: self.layout.vector,
                values: self.checked_values()?,
            })
        }
    }
}
