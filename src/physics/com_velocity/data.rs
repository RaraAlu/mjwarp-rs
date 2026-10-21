//! 空间速度数据与快照实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};

use super::*;

impl ComVelocityData {
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn rne_inputs(&self) -> Result<RneInputs<'_>, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "com_velocity",
            });
        }
        Ok(RneInputs {
            qvel: &self.qvel,
            com: &self.com,
            velocity: &self.output,
        })
    }

    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }

    pub fn write_qpos(&mut self, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.position.write_qpos(qpos);
        self.after_position_write(result)
    }

    pub fn write_world_qpos(&mut self, world: usize, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.position.write_world_qpos(world, qpos);
        self.after_position_write(result)
    }

    fn after_position_write(
        &mut self,
        result: Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }

    /// 检查全部速度后再写入设备。
    /// 输入错误保留已有速度结果。
    pub fn write_qvel(&mut self, qvel: &[f32]) -> Result<(), TransferError> {
        self.write_checked_qvel(0, self.layout.qvel.total_elements(), qvel)
    }

    pub fn write_world_qvel(&mut self, world: usize, qvel: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.qvel.world_elements(world)?;
        self.write_checked_qvel(range.start, range.len(), qvel)
    }

    fn write_checked_qvel(
        &mut self,
        offset: usize,
        expected: usize,
        qvel: &[f32],
    ) -> Result<(), TransferError> {
        check_qvel(expected, qvel)?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.qvel.write_range(4 + offset, qvel)
    }

    /// 显式检查速度与输出守卫。
    /// 未就绪或非有限结果不发布。
    pub fn readback(&self) -> Result<ComVelocitySnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "com_velocity",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            for (buffer, length, field) in [
                (&self.rigid, self.layout.guarded_rigid, "com_velocity_rigid"),
                (&self.com, self.layout.guarded_com, "com_velocity_com"),
            ] {
                let mut values = crate::runtime::host_staging::<f32>(length)?;
                buffer.read_range_into(0, &mut values)?;
                super::super::check_device_values(&values, field)?;
            }
            let mut qvel = crate::runtime::host_staging::<f32>(self.layout.guarded_qvel)?;
            self.qvel.read_range_into(0, &mut qvel)?;
            super::super::check_device_values(&qvel, "com_velocity_qvel")?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            super::super::check_device_values(&values, "com_velocity_output")?;
            Ok(ComVelocitySnapshot {
                layout: self.layout,
                values,
            })
        }
    }
}

impl ComVelocitySnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<ComVelocityWorld<'_>, InputError> {
        let range = self.layout.output.world_elements(world)?;
        let values = &self.values[4 + range.start..4 + range.end];
        Ok(ComVelocityWorld {
            cvel: &values[..6 * self.nbody()],
            cdof_dot: &values[6 * self.nbody()..],
        })
    }
}
