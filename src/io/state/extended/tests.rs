//! M08@P7 扩展重置测试：冻结上游参考与校验面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../../fixtures/local-reset/extended-cases.json"
    ))
    .unwrap()
}

fn f32_list(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect()
}

fn i32_list(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap() as i32)
        .collect()
}

fn layout(value: &Value) -> ExtendedLayout {
    let model = &value["model"];
    ExtendedLayout {
        nworld: value["nworld"].as_u64().unwrap() as usize,
        nq: model["nq"].as_u64().unwrap() as usize,
        nv: model["nv"].as_u64().unwrap() as usize,
        nu: model["nu"].as_u64().unwrap() as usize,
        na: model["na"].as_u64().unwrap() as usize,
        nmocap: model["nmocap"].as_u64().unwrap() as usize,
        nbody: model["nbody"].as_u64().unwrap() as usize,
    }
}

fn state(value: &Value) -> ExtendedState {
    let initial = &value["initial"];
    ExtendedState::from_batches(
        layout(value),
        ExtendedBatches {
            time: &f32_list(&initial["time"]),
            qpos: &f32_list(&initial["qpos"]),
            qvel: &f32_list(&initial["qvel"]),
            ctrl: &f32_list(&initial["ctrl"]),
            act: &f32_list(&initial["act"]),
            qfrc_applied: &f32_list(&initial["qfrc_applied"]),
            xfrc_applied: &f32_list(&initial["xfrc_applied"]),
            mocap_pos: &f32_list(&initial["mocap_pos"]),
            mocap_quat: &f32_list(&initial["mocap_quat"]),
        },
    )
    .unwrap()
}

fn model_arrays(value: &Value) -> &Value {
    &value["model_arrays"]
}

fn mocap_defaults(value: &Value) -> MocapDefaults<'_> {
    let arrays = model_arrays(value);
    MocapDefaults {
        body_mocapid: Box::leak(i32_list(&arrays["body_mocapid"]).into_boxed_slice()),
        body_pos: Box::leak(f32_list(&arrays["body_pos"]).into_boxed_slice()),
        body_quat: Box::leak(f32_list(&arrays["body_quat"]).into_boxed_slice()),
    }
}

fn keyframes(value: &Value) -> ExtendedKeyframes<'_> {
    let arrays = model_arrays(value);
    ExtendedKeyframes {
        nkey: value["model"]["nkey"].as_u64().unwrap() as usize,
        time: Box::leak(f32_list(&arrays["key_time"]).into_boxed_slice()),
        qpos: Box::leak(f32_list(&arrays["key_qpos"]).into_boxed_slice()),
        qvel: Box::leak(f32_list(&arrays["key_qvel"]).into_boxed_slice()),
        act: Box::leak(f32_list(&arrays["key_act"]).into_boxed_slice()),
        ctrl: Box::leak(f32_list(&arrays["key_ctrl"]).into_boxed_slice()),
        mpos: Box::leak(f32_list(&arrays["key_mpos"]).into_boxed_slice()),
        mquat: Box::leak(f32_list(&arrays["key_mquat"]).into_boxed_slice()),
    }
}

fn assert_bits(field: &str, product: &[f32], expected: &Value) {
    let expected = f32_list(expected);
    assert_eq!(product.len(), expected.len(), "{field} length");
    for (index, (actual, target)) in product.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            actual.to_bits(),
            target.to_bits(),
            "{field}[{index}] product={actual} expected={target}"
        );
    }
}

#[test]
fn extended_reset_cases_match_frozen_upstream_reference() {
    let value = fixture();
    assert_eq!(
        value["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-m08-p7"
    );
    let qpos0 = f32_list(&model_arrays(&value)["qpos0"]);
    let mocap = mocap_defaults(&value);
    let keyframes = keyframes(&value);
    let cases = value["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 6);
    for case in cases {
        let mut state = state(&value);
        let operation = case["operation"].as_str().unwrap();
        match operation {
            "reset_data" => {
                let mask = match &case["mask"] {
                    Value::Null => ResetMask::All,
                    Value::Array(values) if values.iter().all(Value::is_boolean) => {
                        ResetMask::Worlds(Box::leak(
                            values
                                .iter()
                                .map(|item| item.as_bool().unwrap())
                                .collect::<Vec<_>>()
                                .into_boxed_slice(),
                        ))
                    }
                    Value::Array(values) => ResetMask::Flags(Box::leak(
                        values
                            .iter()
                            .map(|item| item.as_i64().unwrap() as i32)
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    )),
                    other => panic!("unexpected mask {other}"),
                };
                reset_data_extended(&mut state, &qpos0, mocap, mask).unwrap();
            }
            "reset_data_keyframe" => {
                let selection = match &case["key"] {
                    Value::Number(number) => {
                        KeyframeSelection::All(number.as_i64().unwrap() as i32)
                    }
                    Value::Array(values) => KeyframeSelection::Worlds(Box::leak(
                        values
                            .iter()
                            .map(|item| item.as_i64().unwrap() as i32)
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    )),
                    other => panic!("unexpected key {other}"),
                };
                reset_data_keyframe_extended(&mut state, keyframes, selection).unwrap();
            }
            other => panic!("unexpected operation {other}"),
        }
        let expected = &case["expected"];
        assert_bits("time", state.time(), &expected["time"]);
        assert_bits("qpos", state.qpos(), &expected["qpos"]);
        assert_bits("qvel", state.qvel(), &expected["qvel"]);
        assert_bits("ctrl", state.ctrl(), &expected["ctrl"]);
        assert_bits("act", state.act(), &expected["act"]);
        assert_bits(
            "qfrc_applied",
            state.qfrc_applied(),
            &expected["qfrc_applied"],
        );
        assert_bits(
            "xfrc_applied",
            state.xfrc_applied(),
            &expected["xfrc_applied"],
        );
        assert_bits("mocap_pos", state.mocap_pos(), &expected["mocap_pos"]);
        assert_bits("mocap_quat", state.mocap_quat(), &expected["mocap_quat"]);
    }
}

#[test]
fn non_masked_worlds_keep_extended_state() {
    let value = fixture();
    let qpos0 = f32_list(&model_arrays(&value)["qpos0"]);
    let mocap = mocap_defaults(&value);
    let mut state = state(&value);
    reset_data_extended(&mut state, &qpos0, mocap, ResetMask::Worlds(&[false, true])).unwrap();
    assert_eq!(state.time(), &[0.5, 0.0]);
    assert_eq!(&state.ctrl()[0..1], &[0.6]);
    assert_eq!(&state.ctrl()[1..2], &[0.0]);
    assert_eq!(&state.mocap_pos()[0..3], &[0.2, 0.1, -0.3]);
    assert_eq!(&state.mocap_pos()[3..6], &[0.5, -0.5, 0.2]);
}

#[test]
fn rejects_invalid_keyframes_and_masks() {
    let value = fixture();
    let qpos0 = f32_list(&model_arrays(&value)["qpos0"]);
    let mocap = mocap_defaults(&value);
    let keyframes = keyframes(&value);
    let mut state = state(&value);
    assert_eq!(
        reset_data_extended(&mut state, &qpos0, mocap, ResetMask::Worlds(&[true])),
        Err(ResetError::LengthMismatch {
            field: "reset",
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(
        reset_data_keyframe_extended(&mut state, keyframes, KeyframeSelection::All(2)),
        Err(ResetError::KeyframeOutOfRange { key: 2, nkey: 2 })
    );
    assert_eq!(
        reset_data_keyframe_extended(&mut state, keyframes, KeyframeSelection::Worlds(&[0, 1, 0])),
        Err(ResetError::LengthMismatch {
            field: "key",
            expected: 2,
            actual: 3,
        })
    );
    let short = ExtendedKeyframes {
        nkey: 2,
        time: &keyframes.time[..1],
        ..keyframes
    };
    assert_eq!(
        reset_data_keyframe_extended(&mut state, short, KeyframeSelection::All(0)),
        Err(ResetError::LengthMismatch {
            field: "key_time",
            expected: 2,
            actual: 1,
        })
    );
}
