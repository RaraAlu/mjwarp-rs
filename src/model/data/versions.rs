//! A1-4：常驻状态字段版本与来源代际快照。
//!
//! 每个状态字段独立记录写入版本。
//! 下游模块按字段版本核对来源是否同代。
//! 版本只读快照可跨模块传递，不携带可变借用。

use crate::diagnostics::InputError;
use crate::model::data::lifecycle::DataIdentity;

/// 常驻状态字段；与设备缓冲一一对应。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateField {
    /// 广义位置：`state`缓冲的qpos段。
    Qpos,
    /// 广义速度：常驻边缓冲的qvel段。
    Qvel,
    /// mocap位姿：`state`缓冲的mocap段。
    Mocap,
    /// 休眠树状态：休眠子集的独立状态缓冲。
    Sleep,
}

impl StateField {
    /// 稳定字段名；用于错误与证据记录。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Qpos => "qpos",
            Self::Qvel => "qvel",
            Self::Mocap => "mocap",
            Self::Sleep => "sleep",
        }
    }
}

/// 各状态字段的写入版本。
///
/// 零表示尚未写入；每次成功写入加一。
/// 只读快照不携带可变借用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateFieldVersions {
    qpos: u64,
    qvel: u64,
    mocap: u64,
    sleep: u64,
}

impl StateFieldVersions {
    /// 按字段取版本。
    pub fn get(self, field: StateField) -> u64 {
        match field {
            StateField::Qpos => self.qpos,
            StateField::Qvel => self.qvel,
            StateField::Mocap => self.mocap,
            StateField::Sleep => self.sleep,
        }
    }

    pub fn qpos(self) -> u64 {
        self.qpos
    }

    pub fn qvel(self) -> u64 {
        self.qvel
    }

    pub fn mocap(self) -> u64 {
        self.mocap
    }

    pub fn sleep(self) -> u64 {
        self.sleep
    }

    /// 下一版本；溢出即拒绝。
    pub(crate) fn next(self, field: StateField) -> Result<u64, InputError> {
        self.get(field).checked_add(1).ok_or(InputError::Overflow {
            field: field.as_str(),
        })
    }

    /// 写入已校验的下一版本。
    pub(crate) fn set(&mut self, field: StateField, version: u64) {
        match field {
            StateField::Qpos => self.qpos = version,
            StateField::Qvel => self.qvel = version,
            StateField::Mocap => self.mocap = version,
            StateField::Sleep => self.sleep = version,
        }
    }
}

/// 常驻源状态快照：身份、状态代际与字段版本。
///
/// 由常驻数据在写入或更新时记录。
/// 下游按身份与字段版本核对来源是否同代。
/// 只读快照不携带可变借用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceState {
    identity: DataIdentity,
    state_version: u64,
    fields: StateFieldVersions,
}

impl SourceState {
    pub fn new(identity: DataIdentity, state_version: u64, fields: StateFieldVersions) -> Self {
        Self {
            identity,
            state_version,
            fields,
        }
    }

    pub fn identity(self) -> DataIdentity {
        self.identity
    }

    pub fn state_version(self) -> u64 {
        self.state_version
    }

    pub fn fields(self) -> StateFieldVersions {
        self.fields
    }

    /// 数据身份是否一致；同尺寸不代表同拓扑。
    pub fn same_identity(self, other: Self) -> bool {
        self.identity.matches(&other.identity)
    }

    /// 指定字段版本是否同代；身份必须一致。
    pub fn same_field_generation(self, other: Self, field: StateField) -> bool {
        self.same_identity(other) && self.fields.get(field) == other.fields.get(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::data::lifecycle::{LayoutIdentity, ModelIdentity};

    fn identity() -> DataIdentity {
        DataIdentity::new(
            ModelIdentity::of_topology(&[2, 3, 4]),
            LayoutIdentity::of_parts(&[2, 6, 4]),
        )
    }

    fn source(qpos: u64, qvel: u64) -> SourceState {
        let mut fields = StateFieldVersions::default();
        fields.set(StateField::Qpos, qpos);
        fields.set(StateField::Qvel, qvel);
        SourceState::new(identity(), qpos + qvel, fields)
    }

    #[test]
    fn source_state_compares_identity_and_field_generations() {
        let first = source(2, 3);
        let same = source(2, 3);
        assert!(first.same_identity(same));
        assert!(first.same_field_generation(same, StateField::Qpos));
        assert!(first.same_field_generation(same, StateField::Qvel));
        let advanced_qpos = SourceState::new(identity(), first.state_version() + 1, {
            let mut fields = first.fields();
            fields.set(StateField::Qpos, 3);
            fields
        });
        assert!(first.same_identity(advanced_qpos));
        assert!(!first.same_field_generation(advanced_qpos, StateField::Qpos));
        assert!(first.same_field_generation(advanced_qpos, StateField::Qvel));
        assert_eq!(first.state_version(), 5);
        assert_eq!(advanced_qpos.state_version(), 6);
    }

    #[test]
    fn source_state_rejects_foreign_identity() {
        let foreign = SourceState::new(
            DataIdentity::new(
                ModelIdentity::of_topology(&[9, 9, 9, 9]),
                LayoutIdentity::of_parts(&[9, 9, 9]),
            ),
            0,
            StateFieldVersions::default(),
        );
        assert!(!source(1, 1).same_identity(foreign));
        assert!(!source(1, 1).same_field_generation(foreign, StateField::Qvel));
    }

    #[test]
    fn versions_start_at_zero_and_advance_per_field() {
        let mut versions = StateFieldVersions::default();
        for field in [
            StateField::Qpos,
            StateField::Qvel,
            StateField::Mocap,
            StateField::Sleep,
        ] {
            assert_eq!(versions.get(field), 0);
            let next = versions.next(field).unwrap();
            assert_eq!(next, 1);
            versions.set(field, next);
            assert_eq!(versions.get(field), 1);
        }
        assert_eq!(versions.qpos(), 1);
        assert_eq!(versions.qvel(), 1);
        assert_eq!(versions.mocap(), 1);
        assert_eq!(versions.sleep(), 1);
        assert_eq!(StateField::Qpos.as_str(), "qpos");
    }

    #[test]
    fn rejects_field_version_exhaustion() {
        let mut versions = StateFieldVersions::default();
        versions.set(StateField::Qpos, u64::MAX);
        assert!(versions.next(StateField::Qpos).is_err());
        versions.set(StateField::Qpos, u64::MAX - 1);
        assert_eq!(versions.next(StateField::Qpos).unwrap(), u64::MAX);
    }
}
