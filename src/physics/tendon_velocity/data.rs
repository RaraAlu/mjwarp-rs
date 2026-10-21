//! 肌腱速度数据写入、检查与回读。

use super::*;

impl TendonVelocityData {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.velocity.worlds(), self.output.worlds());
        self.velocity.worlds()
    }
    pub fn write_qvel(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(None, values)
    }
    pub fn write_world_qvel(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        self.write(Some(world), values)
    }
    pub(in crate::physics) fn write(
        &mut self,
        world: Option<usize>,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let (offset, count) = if let Some(w) = world {
            let r = self.velocity.world_elements(w)?;
            (r.start, r.len())
        } else {
            (0, self.velocity.total_elements())
        };
        if values.len() != count {
            return Err(InputError::LengthMismatch {
                field: "tendon_velocity_qvel",
                expected: count,
                actual: values.len(),
            }
            .into());
        }
        if let Some(index) = values.iter().position(|v| !v.is_finite()) {
            return Err(InputError::NonFinite {
                field: "tendon_velocity_qvel",
                index,
            }
            .into());
        }
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.qvel.write_range(4 + offset, values)
        }
    }
}
