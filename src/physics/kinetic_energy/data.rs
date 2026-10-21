//! 动能数据写入、检查与回读。

use super::checks::check;
use super::checks::check_pair;
use super::*;

impl KineticEnergyData {
    pub fn worlds(&self) -> usize {
        self.layout.energy.worlds()
    }
    pub fn write_pair(
        &mut self,
        q: KineticEnergyVector<'_>,
        m: KineticEnergyVector<'_>,
    ) -> Result<(), TransferError> {
        self.write(0, self.worlds(), q, m)
    }
    pub fn write_world_pair(
        &mut self,
        world: usize,
        q: KineticEnergyVector<'_>,
        m: KineticEnergyVector<'_>,
    ) -> Result<(), TransferError> {
        self.layout.pair.world_elements(world)?;
        self.write(world, 1, q, m)
    }
    pub(super) fn write(
        &mut self,
        first: usize,
        worlds: usize,
        q: KineticEnergyVector<'_>,
        m: KineticEnergyVector<'_>,
    ) -> Result<(), TransferError> {
        check_pair(&self.spec, worlds * self.spec.nv, q, m)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = first;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let n = self.spec.nv;
            let mut packed = crate::runtime::host_staging::<f32>(2 * worlds * n)?;
            for w in 0..worlds {
                packed[2 * w * n..(2 * w + 1) * n].copy_from_slice(&q.values[w * n..(w + 1) * n]);
                packed[(2 * w + 1) * n..2 * (w + 1) * n]
                    .copy_from_slice(&m.values[w * n..(w + 1) * n]);
            }
            // 输入错误保持就绪；驱动错误先使旧快照失效。
            self.ready = false;
            self.pair.write_range(4 + 2 * first * n, &packed)?;
            self.states[first..first + worlds].fill(Some(q.state_id));
            Ok(())
        }
    }
    pub fn write_potential(&mut self, values: &[f32]) -> Result<(), TransferError> {
        check(values, self.worlds(), "energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.ready = false;
            for (w, v) in values.iter().enumerate() {
                self.current.write_range(4 + 2 * w, &[*v])?;
            }
            Ok(())
        }
    }
    pub fn write_world_potential(&mut self, world: usize, value: f32) -> Result<(), TransferError> {
        self.layout.energy.world_elements(world)?;
        check(&[value], 1, "energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.ready = false;
            self.current.write_range(4 + 2 * world, &[value])
        }
    }
    pub fn readback(&self) -> Result<KineticEnergyOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "kinetic_energy",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(KineticEnergyOutput {
                layout: self.layout.energy,
                model_id: self.spec.model_id,
                states: self.states.iter().map(|s| s.unwrap()).collect(),
                values: read(&self.current, self.layout.energy.total_elements(), "energy")?,
            })
        }
    }
}
#[cfg(feature = "cuda-probe")]
pub(super) fn read(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut v)?;
    crate::physics::check_device_values(&v, field)?;
    Ok(v)
}
