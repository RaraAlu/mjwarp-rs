//! 被动合力数据访问与视图回读。

use super::*;

impl PassiveForceData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    pub fn write_other_force(
        &mut self,
        field: SmoothForceField,
        values: &[f32],
    ) -> Result<(), TransferError> {
        if field == SmoothForceField::Passive {
            return Err(InputError::InvalidDimension {
                field: "shared_passive_owned",
            }
            .into());
        }
        self.ready = false;
        self.smooth.write_force(field, values)
    }

    pub(super) fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "passive_force",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(PassiveForceSnapshot {
                layout: self.layout,
                values: read_parts(&self.parts, 2 * self.layout.total_elements())?,
                gravity: read_parts(&self.gravity, self.layout.total_elements())?,
            })
        }
    }

    pub(super) fn smooth_output(&self) -> Result<SmoothForceOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "passive_force",
            });
        }
        self.smooth.readback()
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn corrupt_parts_guard_for_test(
        &mut self,
        index: usize,
        value: f32,
    ) -> Result<(), TransferError> {
        self.parts.write_range(index, &[value])
    }
}

impl PassiveForceView<'_, '_, '_> {
    pub fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
        self.data.snapshot()
    }

    pub fn smooth(&self) -> Result<SmoothForceOutput, TransferError> {
        self.data.smooth_output()
    }
}

impl ClearedPassiveForceView<'_> {
    pub fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
        self.data.snapshot()
    }

    pub fn smooth(&self) -> Result<SmoothForceOutput, TransferError> {
        self.data.smooth_output()
    }
}

impl SleepClearedPassiveForceView<'_> {
    pub fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
        self.data.snapshot()
    }

    pub fn smooth(&self) -> Result<SmoothForceOutput, TransferError> {
        self.data.smooth_output()
    }
}

/// 同代核对：关节副本与肌腱输出必须与常驻源一致。
///
/// 旧写入入口不声明来源（`None`），保持兼容不拒绝。
/// 不一致分阶段返回`StageNotReady`，先于任何设备写入。
pub(in crate::physics) fn check_shared_source_generation(
    joint: Option<SourceState>,
    tendon: Option<SourceState>,
    current: SourceState,
) -> Result<(), TransferError> {
    if let Some(joint) = joint
        && (!joint.same_field_generation(current, StateField::Qpos)
            || !joint.same_field_generation(current, StateField::Qvel))
    {
        return Err(TransferError::StageNotReady {
            stage: "passive_force_joint_generation",
        });
    }
    if let Some(tendon) = tendon
        && !tendon.same_field_generation(current, StateField::Qvel)
    {
        return Err(TransferError::StageNotReady {
            stage: "passive_force_tendon_generation",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::data::{DataIdentity, LayoutIdentity, ModelIdentity, StateFieldVersions};

    fn identity() -> DataIdentity {
        DataIdentity::new(
            ModelIdentity::of_topology(&[2, 3, 4]),
            LayoutIdentity::of_parts(&[2, 6, 4]),
        )
    }

    fn state(qpos: u64, qvel: u64) -> SourceState {
        let mut fields = StateFieldVersions::default();
        fields.set(StateField::Qpos, qpos);
        fields.set(StateField::Qvel, qvel);
        SourceState::new(identity(), qpos + qvel, fields)
    }

    #[test]
    fn legacy_writes_without_source_declaration_stay_compatible() {
        let current = state(2, 3);
        check_shared_source_generation(None, None, current).unwrap();
        check_shared_source_generation(Some(current), Some(current), current).unwrap();
    }

    #[test]
    fn rejects_stale_joint_generation_before_device_writes() {
        let current = state(2, 3);
        let stale = state(1, 3);
        assert_eq!(
            check_shared_source_generation(Some(stale), None, current).unwrap_err(),
            TransferError::StageNotReady {
                stage: "passive_force_joint_generation"
            }
        );
        let stale = state(2, 2);
        assert_eq!(
            check_shared_source_generation(Some(stale), None, current).unwrap_err(),
            TransferError::StageNotReady {
                stage: "passive_force_joint_generation"
            }
        );
    }

    #[test]
    fn rejects_stale_tendon_generation() {
        let current = state(2, 3);
        let stale = state(2, 2);
        assert_eq!(
            check_shared_source_generation(None, Some(stale), current).unwrap_err(),
            TransferError::StageNotReady {
                stage: "passive_force_tendon_generation"
            }
        );
    }

    #[test]
    fn resync_restores_same_generation() {
        let current = state(2, 3);
        let stale = state(1, 2);
        assert!(check_shared_source_generation(Some(stale), Some(stale), current).is_err());
        // 重新同步后两方均取自当前源。
        check_shared_source_generation(Some(current), Some(current), current).unwrap();
    }
}
