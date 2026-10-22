//! 肌腱速度数据写入、检查与回读。

use super::*;
use crate::physics::KinematicsData;

impl TendonVelocityData {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.velocity.worlds(), self.output.worlds());
        self.velocity.worlds()
    }
    /// A1：qvel历史副本的常驻来源声明；旧写入入口为`None`。
    pub fn source_state(&self) -> Option<SourceState> {
        self.source
    }
    /// A1：最近一次成功更新消费的常驻来源代际。
    pub fn published_state(&self) -> Option<SourceState> {
        self.published
    }
    fn check_source(&self, source: &KinematicsData) -> Result<(), TransferError> {
        if source.worlds() == self.worlds() {
            Ok(())
        } else {
            Err(TransferError::ModelMismatch)
        }
    }
    /// 从常驻源写入全部历史qvel并记录来源代际。
    pub fn write_qvel_from(
        &mut self,
        source: &KinematicsData,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.check_source(source)?;
        self.write(None, values)?;
        self.source = Some(source.source_state());
        Ok(())
    }
    /// 从常驻源写入指定世界的历史qvel并记录来源代际。
    pub fn write_world_qvel_from(
        &mut self,
        source: &KinematicsData,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.check_source(source)?;
        self.write(Some(world), values)?;
        self.source = Some(source.source_state());
        Ok(())
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
            self.qvel.write_range(4 + offset, values)?;
            // 旧写入入口不声明来源；输入声明与已发布代际在此清空。
            self.source = None;
            self.published = None;
            Ok(())
        }
    }
}
