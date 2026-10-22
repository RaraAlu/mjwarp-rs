//! 关节被动数据写入、检查与回读。

use super::*;
use crate::physics::KinematicsData;

impl JointPassiveData {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.layout.state.worlds(), self.layout.vector.worlds());
        self.layout.vector.worlds()
    }
    /// A1：最近一次声明的常驻来源；旧写入入口为`None`。
    pub fn source_state(&self) -> Option<SourceState> {
        self.source
    }
    fn check_source(&self, source: &KinematicsData) -> Result<(), TransferError> {
        if source.worlds() == self.worlds() {
            Ok(())
        } else {
            Err(TransferError::ModelMismatch)
        }
    }
    /// 从常驻源写入全部qpos并记录来源代际。
    pub fn write_qpos_from(
        &mut self,
        source: &KinematicsData,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.check_source(source)?;
        self.write(false, None, values)?;
        self.source = Some(source.source_state());
        Ok(())
    }
    /// 从常驻源写入全部qvel并记录来源代际。
    pub fn write_qvel_from(
        &mut self,
        source: &KinematicsData,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.check_source(source)?;
        self.write(true, None, values)?;
        self.source = Some(source.source_state());
        Ok(())
    }
    /// 从常驻源写入指定世界的qpos并记录来源代际。
    pub fn write_world_qpos_from(
        &mut self,
        source: &KinematicsData,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.check_source(source)?;
        self.write(false, Some(world), values)?;
        self.source = Some(source.source_state());
        Ok(())
    }
    /// 从常驻源写入指定世界的qvel并记录来源代际。
    pub fn write_world_qvel_from(
        &mut self,
        source: &KinematicsData,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.check_source(source)?;
        self.write(true, Some(world), values)?;
        self.source = Some(source.source_state());
        Ok(())
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(false, None, values)
    }
    pub fn write_qvel(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(true, None, values)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        self.write(false, Some(world), values)
    }
    pub fn write_world_qvel(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        self.write(true, Some(world), values)
    }
    pub(super) fn write(
        &mut self,
        vel: bool,
        world: Option<usize>,
        values: &[f32],
    ) -> Result<(), TransferError> {
        if let Some(w) = world {
            self.layout.vector.world_elements(w)?;
        }
        let width = if vel {
            self.input.model.nv()
        } else {
            self.input.model.nq()
        };
        let expected = if world.is_some() {
            width
        } else {
            width * self.worlds()
        };
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "joint_passive_state",
                expected,
                actual: values.len(),
            }
            .into());
        }
        if let Some(index) = values.iter().position(|v| !v.is_finite()) {
            return Err(InputError::NonFinite {
                field: "joint_passive_state",
                index,
            }
            .into());
        }
        let offset =
            4 + if vel {
                self.worlds() * self.input.model.nq()
            } else {
                0
            } + world.unwrap_or(0) * width;
        self.ready = false;
        self.smooth.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.state.write_range(offset, values)?;
            // 旧写入入口不声明来源；来源代际快照在此清空。
            self.source = None;
            Ok(())
        }
    }
    /// 调用方显式提供其余三项；不能覆盖本生产者的Passive字段。
    pub fn write_other_force(
        &mut self,
        field: SmoothForceField,
        values: &[f32],
    ) -> Result<(), TransferError> {
        if field == SmoothForceField::Passive {
            return Err(InputError::InvalidDimension {
                field: "joint_only_passive_owned",
            }
            .into());
        }
        self.smooth.write_force(field, values)
    }
    pub fn readback_joint(&self) -> Result<JointPassiveOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "joint_passive",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(JointPassiveOutput {
                layout: self.layout.vector,
                values: read(
                    &self.current,
                    self.layout.parts.total_elements(),
                    "joint_passive",
                )?,
            })
        }
    }
    pub fn readback_joint_only_smooth(&self) -> Result<SmoothForceOutput, TransferError> {
        self.smooth.readback()
    }
}
