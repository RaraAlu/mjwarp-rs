//! M10@P7 Rust 回调契约切片：类型、时序、状态与同步契约。
//!
//! 对齐冻结上游 `mujoco_warp._src.types.Callback`（U079；M10；P7；提交
//! `71da24d956…`，Apache-2.0）与以下调用点：
//!
//! - `types.Callback`：七个可选 callable 槽位（`passive`/`control`/`act_dyn`/
//!   `act_gain`/`act_bias`/`sensor`/`contactfilter`）及其写入的 Data 字段。
//! - `forward.forward`：`fwd_position`（内含 collision 的 `contactfilter`）→
//!   `sensor_pos`（sensor POS）→ `fwd_velocity`（passive）→ `sensor_vel`
//!   （sensor VEL）→ `control`（仅 ACTUATION 未禁用）→ `fwd_actuation`
//!   （act_dyn → act_gain → act_bias）→ `solver.solve` → `sensor_acc`（sensor ACC）。
//! - `passive.passive`、`sensor.sensor_pos/_vel/_acc`、`collision_driver` 的
//!   回调调用点与 `types.Stage`（POS/VEL/ACC）数值。
//!
//! 本片只冻结**类型/时序/状态/同步契约**并给出不执行的调用计划；它**不是**
//! 阶段回调，不实现 `forward`/`step`/`step1`/`step2` 调度与回调执行。
//!
//! 未验收边界：回调钩子函数本体与设备内核、状态租约/同步原语接线、
//! `inverse` 路径的回调调用、sensor 延迟/interval 与 contactfilter 全量语义、
//! GPU 内核与图执行。

use std::error::Error;
use std::fmt;

/// 上游 `types.Callback` 的七个回调槽位。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CallbackKind {
    /// 自定义被动力，写入 `Data.qfrc_passive`。
    Passive,
    /// 自定义控制律，写入 `Data.ctrl`。
    Control,
    /// 自定义执行器动力学，写入 `Data.act_dot`。
    ActDyn,
    /// 自定义执行器增益，写入 `Data.actuator_force`。
    ActGain,
    /// 自定义执行器偏置，写入 `Data.actuator_force`。
    ActBias,
    /// 自定义传感器，写入 `Data.sensordata`。
    Sensor,
    /// 自定义接触过滤，写入 `Data.contact`。
    ContactFilter,
}

/// 回调在 `forward`/`step` 管线中的阶段；判别值即上游调用顺序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallbackStage {
    /// `fwd_position` 内的 collision（`collision_driver`）。
    Collision = 0,
    /// `sensor_pos` 之后的 sensor POS 回调。
    SensorPos = 1,
    /// `fwd_velocity` 内 passive 力聚合之后。
    Passive = 2,
    /// `sensor_vel` 之后的 sensor VEL 回调。
    SensorVel = 3,
    /// `forward` 中 ACTUATION 未禁用时、`fwd_actuation` 之前。
    Control = 4,
    /// `fwd_actuation` 的 `_actuator_force` 之内。
    Actuation = 5,
    /// `solver.solve` 之后的 sensor ACC 回调。
    SensorAcc = 6,
}

impl CallbackStage {
    /// 上游调用点（证据 README 的 file:line 映射键）。
    pub fn upstream_site(self) -> &'static str {
        match self {
            Self::Collision => "collision_driver.py:983-984 (fwd_position)",
            Self::SensorPos => "sensor.py:963-964 (sensor_pos, Stage.POS)",
            Self::Passive => "passive.py:3695-3696 (fwd_velocity)",
            Self::SensorVel => "sensor.py:1509-1510 (sensor_vel, Stage.VEL)",
            Self::Control => "forward.py:2080-2082 (forward, ACTUATION gate)",
            Self::Actuation => "forward.py:1916-1921 (_actuator_force)",
            Self::SensorAcc => "sensor.py:2847-2848 (sensor_acc, Stage.ACC)",
        }
    }

    /// sensor 回调的阶段参数（上游 `types.Stage`：POS=0、VEL=1、ACC=2）。
    pub fn sensor_value(self) -> Option<i32> {
        match self {
            Self::SensorPos => Some(1),
            Self::SensorVel => Some(2),
            Self::SensorAcc => Some(3),
            _ => None,
        }
    }
}

/// 回调写入的 Data 字段（上游 docstring 声明）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackTarget {
    /// `qfrc_passive`。
    QfrcPassive,
    /// `ctrl`。
    Ctrl,
    /// `act_dot`。
    ActDot,
    /// `actuator_force`。
    ActuatorForce,
    /// `sensordata`。
    Sensordata,
    /// `contact`。
    Contact,
}

impl CallbackTarget {
    /// Data 字段名（不带 `Data.` 前缀）。
    pub fn data_field(self) -> &'static str {
        match self {
            Self::QfrcPassive => "qfrc_passive",
            Self::Ctrl => "ctrl",
            Self::ActDot => "act_dot",
            Self::ActuatorForce => "actuator_force",
            Self::Sensordata => "sensordata",
            Self::Contact => "contact",
        }
    }
}

/// 回调的调用门控（调用点显式条件）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackGate {
    /// 无附加门控。
    None,
    /// 仅当 `DisableBit.ACTUATION` 未置位（上游 `forward.py:2080`）。
    ActuationEnabled,
}

/// 执行器 USER 类型的字段（`act_*` 回调写入生效的状态前提）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserTypeField {
    /// `dyntype`（`DynType.USER`）。
    DynType,
    /// `gaintype`（`GainType.USER`）。
    GainType,
    /// `biastype`（`BiasType.USER`）。
    BiasType,
}

impl UserTypeField {
    /// 上游字段名。
    pub fn upstream_field(self) -> &'static str {
        match self {
            Self::DynType => "dyntype",
            Self::GainType => "gaintype",
            Self::BiasType => "biastype",
        }
    }
}

/// 单个回调槽位的契约。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackContract {
    /// 槽位。
    pub kind: CallbackKind,
    /// 阶段序列（sensor 为 POS/VEL/ACC 三段，其余一段）。
    pub stages: &'static [CallbackStage],
    /// 写入的 Data 字段。
    pub target: CallbackTarget,
    /// 调用门控。
    pub gate: CallbackGate,
    /// `act_*` 回调的 USER 类型前提。
    pub user_type: Option<UserTypeField>,
}

impl CallbackKind {
    /// 全部七个槽位（上游 dataclass 字段顺序）。
    pub const ALL: [Self; 7] = [
        Self::Passive,
        Self::Control,
        Self::ActDyn,
        Self::ActGain,
        Self::ActBias,
        Self::Sensor,
        Self::ContactFilter,
    ];

    /// 上游 dataclass 字段名。
    pub fn upstream_field(self) -> &'static str {
        match self {
            Self::Passive => "passive",
            Self::Control => "control",
            Self::ActDyn => "act_dyn",
            Self::ActGain => "act_gain",
            Self::ActBias => "act_bias",
            Self::Sensor => "sensor",
            Self::ContactFilter => "contactfilter",
        }
    }

    /// 槽位契约。
    pub fn contract(self) -> CallbackContract {
        let (stages, target, gate, user_type): (
            &'static [CallbackStage],
            CallbackTarget,
            CallbackGate,
            Option<UserTypeField>,
        ) = match self {
            Self::Passive => (
                &[CallbackStage::Passive],
                CallbackTarget::QfrcPassive,
                CallbackGate::None,
                None,
            ),
            Self::Control => (
                &[CallbackStage::Control],
                CallbackTarget::Ctrl,
                CallbackGate::ActuationEnabled,
                None,
            ),
            Self::ActDyn => (
                &[CallbackStage::Actuation],
                CallbackTarget::ActDot,
                CallbackGate::None,
                Some(UserTypeField::DynType),
            ),
            Self::ActGain => (
                &[CallbackStage::Actuation],
                CallbackTarget::ActuatorForce,
                CallbackGate::None,
                Some(UserTypeField::GainType),
            ),
            Self::ActBias => (
                &[CallbackStage::Actuation],
                CallbackTarget::ActuatorForce,
                CallbackGate::None,
                Some(UserTypeField::BiasType),
            ),
            Self::Sensor => (
                &[
                    CallbackStage::SensorPos,
                    CallbackStage::SensorVel,
                    CallbackStage::SensorAcc,
                ],
                CallbackTarget::Sensordata,
                CallbackGate::None,
                None,
            ),
            Self::ContactFilter => (
                &[CallbackStage::Collision],
                CallbackTarget::Contact,
                CallbackGate::None,
                None,
            ),
        };
        CallbackContract {
            kind: self,
            stages,
            target,
            gate,
            user_type,
        }
    }
}

/// 同步要求。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncRequirement {
    /// 同一流顺序即可（前序内核先入队、后序内核随后入队）。
    StreamOrder,
    /// 需要显式完成事件（host 读取设备结果前）。
    CompletionEvent,
}

/// 回调执行的同步契约。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackSyncContract {
    /// 回调前的同步要求。
    pub before: SyncRequirement,
    /// 回调后的同步要求。
    pub after: SyncRequirement,
    /// 是否允许回调内隐式 host 读回；本片恒为 false（禁止静默回退 CPU）。
    pub allows_host_readback: bool,
}

/// 冻结的上游回调同步契约：回调在 GPU 阶段之间按流序执行；host 读回必须
/// 由调用方显式提供完成事件（本片不接线租约/事件原语）。
pub const CALLBACK_SYNC_CONTRACT: CallbackSyncContract = CallbackSyncContract {
    before: SyncRequirement::StreamOrder,
    after: SyncRequirement::StreamOrder,
    allows_host_readback: false,
};

/// `forward`/`step` 中的门控位（本片只冻结 ACTUATION 位）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CallbackGates {
    /// `DisableBit.ACTUATION` 未置位。
    pub actuation_enabled: bool,
}

/// 计划中的一次回调调用（不执行，只描述时序）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackInvocation {
    /// 槽位。
    pub kind: CallbackKind,
    /// 阶段。
    pub stage: CallbackStage,
}

/// 回调槽位登记：对应上游 `Model.callback` 的七个可选 callable。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Callback {
    installed: [bool; CallbackKind::ALL.len()],
}

impl Callback {
    /// 空登记。
    pub fn new() -> Self {
        Self::default()
    }

    fn slot(kind: CallbackKind) -> usize {
        CallbackKind::ALL
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("ALL 覆盖全部槽位")
    }

    /// 安装槽位；重复安装拒绝。
    pub fn install(&mut self, kind: CallbackKind) -> Result<(), CallbackError> {
        let slot = Self::slot(kind);
        if self.installed[slot] {
            return Err(CallbackError::AlreadyInstalled { kind });
        }
        self.installed[slot] = true;
        Ok(())
    }

    /// 移除槽位；未安装拒绝。
    pub fn remove(&mut self, kind: CallbackKind) -> Result<(), CallbackError> {
        let slot = Self::slot(kind);
        if !self.installed[slot] {
            return Err(CallbackError::NotInstalled { kind });
        }
        self.installed[slot] = false;
        Ok(())
    }

    /// 槽位是否已安装。
    pub fn is_installed(self, kind: CallbackKind) -> bool {
        self.installed[Self::slot(kind)]
    }

    /// 已安装槽位（按上游 dataclass 字段顺序）。
    pub fn installed(self) -> Vec<CallbackKind> {
        CallbackKind::ALL
            .iter()
            .copied()
            .filter(|kind| self.is_installed(*kind))
            .collect()
    }

    /// 按上游 `forward` 时序生成**不执行**的调用计划。
    ///
    /// 顺序固定为：collision(contactfilter) → sensor POS → passive →
    /// sensor VEL → control（ACTUATION 未禁用时）→ act_dyn → act_gain →
    /// act_bias → sensor ACC。未安装的槽位不产生调用。
    pub fn plan(self, gates: CallbackGates) -> Vec<CallbackInvocation> {
        let mut plan = Vec::new();
        for kind in CallbackKind::ALL {
            if !self.is_installed(kind) {
                continue;
            }
            let contract = kind.contract();
            if contract.gate == CallbackGate::ActuationEnabled && !gates.actuation_enabled {
                continue;
            }
            for stage in contract.stages {
                plan.push(CallbackInvocation {
                    kind,
                    stage: *stage,
                });
            }
        }
        plan.sort_by_key(|invocation| invocation.stage);
        plan
    }
}

/// 回调契约错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallbackError {
    /// 槽位已安装。
    AlreadyInstalled { kind: CallbackKind },
    /// 槽位未安装。
    NotInstalled { kind: CallbackKind },
}

impl fmt::Display for CallbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyInstalled { kind } => {
                write!(formatter, "回调槽位已安装：{}", kind.upstream_field())
            }
            Self::NotInstalled { kind } => {
                write!(formatter, "回调槽位未安装：{}", kind.upstream_field())
            }
        }
    }
}

impl Error for CallbackError {}

#[cfg(test)]
mod tests;
