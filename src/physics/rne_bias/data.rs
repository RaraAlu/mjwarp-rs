//! 偏置力数据写入、检查与回读。

use super::*;

impl RneBiasData {
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

    pub fn write_qpos(&mut self, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qpos(qpos);
        self.after_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qpos(world, qpos);
        self.after_write(result)
    }
    pub fn write_qvel(&mut self, qvel: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qvel(qvel);
        self.after_write(result)
    }
    pub fn write_world_qvel(&mut self, world: usize, qvel: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qvel(world, qvel);
        self.after_write(result)
    }

    /// 显式检查前置字段与全部守卫。
    /// 回读包含内部工作区检查。
    pub fn readback(&self) -> Result<RneBiasSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady { stage: "rne_bias" });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.velocity.readback()?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            crate::physics::check_device_values(&values, "rne_bias_output")?;
            let mut bias = crate::runtime::host_staging::<f32>(self.worlds() * self.layout.nv)?;
            for w in 0..self.worlds() {
                let start =
                    4 + self.layout.output.world_elements(w)?.start + 12 * self.layout.nbody;
                for d in 0..self.layout.nv {
                    bias[w * self.layout.nv + d] = values[start + d];
                }
            }
            if let Some(index) = bias.iter().position(|v| !v.is_finite()) {
                return Err(InputError::NonFinite {
                    field: "rne_bias_output",
                    index,
                }
                .into());
            }
            Ok(RneBiasSnapshot {
                layout: self.layout,
                values: bias,
            })
        }
    }
}
