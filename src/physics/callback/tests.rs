//! M10@P7 回调契约测试：冻结上游字段/Stage 数值、时序计划、门控与错误面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/callback/contract.json")).unwrap()
}

fn expected_plan() -> Vec<(CallbackKind, CallbackStage)> {
    vec![
        (CallbackKind::ContactFilter, CallbackStage::Collision),
        (CallbackKind::Sensor, CallbackStage::SensorPos),
        (CallbackKind::Passive, CallbackStage::Passive),
        (CallbackKind::Sensor, CallbackStage::SensorVel),
        (CallbackKind::Control, CallbackStage::Control),
        (CallbackKind::ActDyn, CallbackStage::Actuation),
        (CallbackKind::ActGain, CallbackStage::Actuation),
        (CallbackKind::ActBias, CallbackStage::Actuation),
        (CallbackKind::Sensor, CallbackStage::SensorAcc),
    ]
}

/// 冻结上游 `types.Callback` 字段顺序、默认值与 `types.Stage` 数值。
#[test]
fn contracts_match_frozen_upstream() {
    let fixture = fixture();
    let fields: Vec<&str> = fixture["callback_fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let ours: Vec<&str> = CallbackKind::ALL
        .iter()
        .map(|kind| kind.upstream_field())
        .collect();
    assert_eq!(ours, fields, "回调槽位与上游 dataclass 字段顺序不一致");
    for kind in CallbackKind::ALL {
        assert_eq!(
            fixture["callback_defaults"][kind.upstream_field()],
            Value::Null,
            "上游回调槽位默认值应为 None：{}",
            kind.upstream_field()
        );
    }
    let sensor = fixture["sensor_stage_values"].as_object().unwrap();
    assert_eq!(CallbackStage::SensorPos.sensor_value(), Some(1));
    assert_eq!(CallbackStage::SensorVel.sensor_value(), Some(2));
    assert_eq!(CallbackStage::SensorAcc.sensor_value(), Some(3));
    for (name, stage) in [
        ("POS", CallbackStage::SensorPos),
        ("VEL", CallbackStage::SensorVel),
        ("ACC", CallbackStage::SensorAcc),
    ] {
        assert_eq!(
            stage.sensor_value().unwrap() as i64,
            sensor[name].as_i64().unwrap(),
            "Stage.{name} 数值不一致"
        );
        assert_eq!(
            fixture["stage_enum_values"][name].as_i64().unwrap(),
            sensor[name].as_i64().unwrap(),
            "Stage.{name} 与 mjtStage 不一致"
        );
    }
    for stage in [
        CallbackStage::Collision,
        CallbackStage::SensorPos,
        CallbackStage::Passive,
        CallbackStage::SensorVel,
        CallbackStage::Control,
        CallbackStage::Actuation,
        CallbackStage::SensorAcc,
    ] {
        assert!(!stage.upstream_site().is_empty());
    }
}

/// 全槽位安装时，时序计划必须与上游 forward 调用顺序一致（不执行）。
#[test]
fn plans_follow_frozen_forward_order() {
    let mut callback = Callback::new();
    for kind in CallbackKind::ALL {
        callback.install(kind).unwrap();
    }
    let plan = callback.plan(CallbackGates {
        actuation_enabled: true,
    });
    let expected = expected_plan();
    assert_eq!(plan.len(), expected.len());
    for (invocation, (kind, stage)) in plan.iter().zip(expected) {
        assert_eq!(invocation.kind, kind);
        assert_eq!(invocation.stage, stage);
    }
    // 阶段严格非降序，且同一 act 阶段的固定次序为 act_dyn→act_gain→act_bias。
    for pair in plan.windows(2) {
        assert!(pair[0].stage <= pair[1].stage);
    }
    let actuation: Vec<CallbackKind> = plan
        .iter()
        .filter(|invocation| invocation.stage == CallbackStage::Actuation)
        .map(|invocation| invocation.kind)
        .collect();
    assert_eq!(
        actuation,
        vec![
            CallbackKind::ActDyn,
            CallbackKind::ActGain,
            CallbackKind::ActBias
        ]
    );
}

/// ACTUATION 禁用时 control 回调被跳过（上游 `forward.py:2080` 门控）。
#[test]
fn control_callback_skipped_when_actuation_disabled() {
    let mut callback = Callback::new();
    callback.install(CallbackKind::Control).unwrap();
    callback.install(CallbackKind::Sensor).unwrap();
    let plan = callback.plan(CallbackGates::default());
    let kinds: Vec<CallbackKind> = plan.iter().map(|invocation| invocation.kind).collect();
    assert!(!kinds.contains(&CallbackKind::Control));
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| **kind == CallbackKind::Sensor)
            .count(),
        3
    );
    let enabled = callback.plan(CallbackGates {
        actuation_enabled: true,
    });
    assert!(enabled.iter().any(|invocation| {
        invocation.kind == CallbackKind::Control && invocation.stage == CallbackStage::Control
    }));
}

/// sensor 回调在 POS/VEL/ACC 三次触发（上游 `sensor.py` 三处调用点）。
#[test]
fn sensor_callback_has_three_stage_invocations() {
    let mut callback = Callback::new();
    callback.install(CallbackKind::Sensor).unwrap();
    let plan = callback.plan(CallbackGates {
        actuation_enabled: true,
    });
    let stages: Vec<CallbackStage> = plan.iter().map(|invocation| invocation.stage).collect();
    assert_eq!(
        stages,
        vec![
            CallbackStage::SensorPos,
            CallbackStage::SensorVel,
            CallbackStage::SensorAcc
        ]
    );
    let values: Vec<i32> = stages
        .iter()
        .map(|stage| stage.sensor_value().unwrap())
        .collect();
    assert_eq!(values, vec![1, 2, 3]);
}

/// 安装/移除/重复/未安装错误面与规范顺序。
#[test]
fn installs_and_removes_slots_once() {
    let mut callback = Callback::new();
    assert!(callback.installed().is_empty());
    callback.install(CallbackKind::Sensor).unwrap();
    callback.install(CallbackKind::Passive).unwrap();
    assert_eq!(
        callback.installed(),
        vec![CallbackKind::Passive, CallbackKind::Sensor]
    );
    assert_eq!(
        callback.install(CallbackKind::Sensor),
        Err(CallbackError::AlreadyInstalled {
            kind: CallbackKind::Sensor
        })
    );
    callback.remove(CallbackKind::Passive).unwrap();
    assert_eq!(
        callback.remove(CallbackKind::Passive),
        Err(CallbackError::NotInstalled {
            kind: CallbackKind::Passive
        })
    );
    assert!(!callback.is_installed(CallbackKind::Passive));
    assert!(callback.is_installed(CallbackKind::Sensor));
    assert_eq!(
        callback.plan(CallbackGates::default()).len(),
        3,
        "sensor 三阶段各一次"
    );
}

/// 状态契约：每个槽位的写入字段、门控与 USER 类型前提。
#[test]
fn write_targets_and_gates_match_upstream_contract_table() {
    let table = [
        (
            CallbackKind::Passive,
            "qfrc_passive",
            CallbackGate::None,
            None,
        ),
        (
            CallbackKind::Control,
            "ctrl",
            CallbackGate::ActuationEnabled,
            None,
        ),
        (
            CallbackKind::ActDyn,
            "act_dot",
            CallbackGate::None,
            Some(UserTypeField::DynType),
        ),
        (
            CallbackKind::ActGain,
            "actuator_force",
            CallbackGate::None,
            Some(UserTypeField::GainType),
        ),
        (
            CallbackKind::ActBias,
            "actuator_force",
            CallbackGate::None,
            Some(UserTypeField::BiasType),
        ),
        (CallbackKind::Sensor, "sensordata", CallbackGate::None, None),
        (
            CallbackKind::ContactFilter,
            "contact",
            CallbackGate::None,
            None,
        ),
    ];
    for (kind, field, gate, user_type) in table {
        let contract = kind.contract();
        assert_eq!(contract.kind, kind);
        assert_eq!(contract.target.data_field(), field);
        assert_eq!(contract.gate, gate);
        assert_eq!(contract.user_type, user_type);
        assert!(!contract.stages.is_empty());
    }
    assert_eq!(UserTypeField::DynType.upstream_field(), "dyntype");
    assert_eq!(UserTypeField::GainType.upstream_field(), "gaintype");
    assert_eq!(UserTypeField::BiasType.upstream_field(), "biastype");
}

/// 同步契约：流序执行、host 读回须显式完成事件、禁止静默回退 CPU。
#[test]
fn sync_contract_forbids_implicit_host_readback() {
    let contract = CALLBACK_SYNC_CONTRACT;
    assert_eq!(contract.before, SyncRequirement::StreamOrder);
    assert_eq!(contract.after, SyncRequirement::StreamOrder);
    assert!(!contract.allows_host_readback);
    assert_ne!(
        SyncRequirement::StreamOrder,
        SyncRequirement::CompletionEvent
    );
}
