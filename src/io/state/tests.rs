//! M08 局部重置切片测试：冻结上游参考与输入校验。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/local-reset/reset-cases.json"
    ))
    .unwrap()
}

fn f32_value(value: &Value) -> f32 {
    value.as_f64().unwrap() as f32
}

fn f32_list(value: &Value) -> Vec<f32> {
    value.as_array().unwrap().iter().map(f32_value).collect()
}

fn f32_matrix(value: &Value) -> Vec<Vec<f32>> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(f32_list)
        .collect::<Vec<_>>()
}

fn flat(matrix: &[Vec<f32>]) -> Vec<f32> {
    matrix.iter().flatten().copied().collect()
}

fn state_from_fixture(value: &Value) -> (WorldState, usize, usize) {
    let nworld = value["nworld"].as_u64().unwrap() as usize;
    let nq = value["model"]["nq"].as_u64().unwrap() as usize;
    let nv = value["model"]["nv"].as_u64().unwrap() as usize;
    let initial = &value["initial"];
    let state = WorldState::from_batches(
        nworld,
        nq,
        nv,
        &f32_list(&initial["time"]),
        &flat(&f32_matrix(&initial["qpos"])),
        &flat(&f32_matrix(&initial["qvel"])),
    )
    .unwrap();
    (state, nq, nv)
}

struct KeyframeArrays {
    nkey: usize,
    time: Vec<f32>,
    qpos: Vec<f32>,
    qvel: Vec<f32>,
}

impl KeyframeArrays {
    fn from_fixture(value: &Value) -> Self {
        let arrays = &value["model_arrays"];
        Self {
            nkey: value["model"]["nkey"].as_u64().unwrap() as usize,
            time: f32_list(&arrays["key_time"]),
            qpos: flat(&f32_matrix(&arrays["key_qpos"])),
            qvel: flat(&f32_matrix(&arrays["key_qvel"])),
        }
    }

    fn keyframes(&self) -> Keyframes<'_> {
        Keyframes::new(self.nkey, &self.time, &self.qpos, &self.qvel).unwrap()
    }
}

fn assert_state_equals(state: &WorldState, expected: &Value) {
    let expected_time = f32_list(&expected["time"]);
    assert_eq!(state.time().len(), expected_time.len());
    for (index, (actual, target)) in state.time().iter().zip(expected_time.iter()).enumerate() {
        assert_eq!(
            actual.to_bits(),
            target.to_bits(),
            "time[{index}] product={actual} expected={target}"
        );
    }
    for (field, actual, expected_matrix) in [
        ("qpos", state.qpos(), f32_matrix(&expected["qpos"])),
        ("qvel", state.qvel(), f32_matrix(&expected["qvel"])),
    ] {
        let expected = flat(&expected_matrix);
        assert_eq!(actual.len(), expected.len());
        for (index, (product, target)) in actual.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                product.to_bits(),
                target.to_bits(),
                "{field}[{index}] product={product} expected={target}"
            );
        }
    }
}

#[test]
fn reset_cases_match_frozen_upstream_reference() {
    let value = fixture();
    assert_eq!(
        value["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-m08-p3"
    );
    let nworld = value["nworld"].as_u64().unwrap() as usize;
    let qpos0 = f32_list(&value["model_arrays"]["qpos0"]);
    let cases = value["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 7);
    let key_arrays = KeyframeArrays::from_fixture(&value);
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let (mut state, _, _) = state_from_fixture(&value);
        let operation = case["operation"].as_str().unwrap();
        match operation {
            "reset_data" => {
                let mask = match &case["mask"] {
                    Value::Null => ResetMask::All,
                    Value::Array(values) if values.iter().all(Value::is_boolean) => {
                        let mask: Vec<bool> =
                            values.iter().map(|item| item.as_bool().unwrap()).collect();
                        assert_eq!(mask.len(), nworld);
                        ResetMask::Worlds(Box::leak(mask.into_boxed_slice()))
                    }
                    Value::Array(values) => {
                        let flags: Vec<i32> = values
                            .iter()
                            .map(|item| item.as_i64().unwrap() as i32)
                            .collect();
                        ResetMask::Flags(Box::leak(flags.into_boxed_slice()))
                    }
                    other => panic!("unexpected mask {other}"),
                };
                reset_data(&mut state, &qpos0, mask).unwrap();
            }
            "reset_data_keyframe" => {
                let keyframes = key_arrays.keyframes();
                let selection = match &case["key"] {
                    Value::Number(number) => {
                        KeyframeSelection::All(number.as_i64().unwrap() as i32)
                    }
                    Value::Array(values) => {
                        let keys: Vec<i32> = values
                            .iter()
                            .map(|item| item.as_i64().unwrap() as i32)
                            .collect();
                        KeyframeSelection::Worlds(Box::leak(keys.into_boxed_slice()))
                    }
                    other => panic!("unexpected key {other}"),
                };
                reset_data_keyframe(&mut state, keyframes, selection).unwrap();
            }
            other => panic!("unexpected operation {other}"),
        }
        assert_state_equals(&state, &case["expected"]);
        println!("case {id} ok");
    }
}

#[test]
fn mask_flags_treat_nonzero_as_reset() {
    let mut state =
        WorldState::from_batches(2, 1, 1, &[1.0, 2.0], &[10.0, 20.0], &[0.5, 0.5]).unwrap();
    reset_data(&mut state, &[0.0], ResetMask::Flags(&[0, 7])).unwrap();
    assert_eq!(state.time(), &[1.0, 0.0]);
    assert_eq!(state.qpos(), &[10.0, 0.0]);
    assert_eq!(state.qvel(), &[0.5, 0.0]);
}

#[test]
fn non_masked_worlds_keep_their_state() {
    let mut state = WorldState::from_batches(
        3,
        1,
        1,
        &[0.5, 1.0, 2.0],
        &[1.0, 3.0, 5.0],
        &[0.1, 0.3, 0.5],
    )
    .unwrap();
    reset_data(&mut state, &[9.0], ResetMask::Worlds(&[true, false, true])).unwrap();
    assert_eq!(state.time(), &[0.0, 1.0, 0.0]);
    assert_eq!(state.qpos(), &[9.0, 3.0, 9.0]);
    assert_eq!(state.qvel(), &[0.0, 0.3, 0.0]);
}

#[test]
fn rejects_invalid_dimensions_and_lengths() {
    assert_eq!(
        WorldState::new(0, 1, 1),
        Err(ResetError::InvalidDimension { field: "nworld" })
    );
    assert_eq!(
        WorldState::from_batches(2, 1, 1, &[0.0], &[0.0, 0.0], &[0.0, 0.0]),
        Err(ResetError::LengthMismatch {
            field: "time",
            expected: 2,
            actual: 1,
        })
    );
    let mut state = WorldState::new(2, 1, 1).unwrap();
    assert_eq!(
        reset_data(&mut state, &[0.0, 0.0], ResetMask::All),
        Err(ResetError::LengthMismatch {
            field: "qpos0",
            expected: 1,
            actual: 2,
        })
    );
    assert_eq!(
        reset_data(&mut state, &[0.0], ResetMask::Worlds(&[true])),
        Err(ResetError::LengthMismatch {
            field: "reset",
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(
        reset_data(&mut state, &[0.0], ResetMask::Flags(&[1])),
        Err(ResetError::LengthMismatch {
            field: "reset",
            expected: 2,
            actual: 1,
        })
    );
}

#[test]
fn rejects_invalid_keyframes() {
    let key_time = [0.5_f32, 1.5];
    let key_qpos = [0.0_f32, 0.0, 1.0, 1.0];
    let key_qvel = [0.0_f32, 1.0];
    let keyframes = Keyframes::new(2, &key_time, &key_qpos, &key_qvel).unwrap();

    let mut state = WorldState::new(2, 2, 1).unwrap();
    assert_eq!(
        reset_data_keyframe(&mut state, keyframes, KeyframeSelection::All(-1)),
        Err(ResetError::KeyframeOutOfRange { key: -1, nkey: 2 })
    );
    assert_eq!(
        reset_data_keyframe(&mut state, keyframes, KeyframeSelection::All(2)),
        Err(ResetError::KeyframeOutOfRange { key: 2, nkey: 2 })
    );
    assert_eq!(
        reset_data_keyframe(&mut state, keyframes, KeyframeSelection::Worlds(&[0, 1, 0])),
        Err(ResetError::LengthMismatch {
            field: "key",
            expected: 2,
            actual: 3,
        })
    );

    let short_keyframes = Keyframes::new(2, &key_time[..1], &key_qpos, &key_qvel);
    assert_eq!(
        short_keyframes,
        Err(ResetError::LengthMismatch {
            field: "key_time",
            expected: 2,
            actual: 1,
        })
    );
}

#[test]
fn keyframe_selection_skips_invalid_worlds() {
    let key_time = [0.5_f32, 1.5];
    let key_qpos = [0.0_f32, 0.0, 1.0, 1.0];
    let key_qvel = [0.0_f32, 1.0];
    let keyframes = Keyframes::new(2, &key_time, &key_qpos, &key_qvel).unwrap();
    let mut state =
        WorldState::from_batches(2, 2, 1, &[1.0, 2.0], &[7.0, 8.0, 9.0, 10.0], &[3.0, 4.0])
            .unwrap();
    reset_data_keyframe(&mut state, keyframes, KeyframeSelection::Worlds(&[1, -4])).unwrap();
    assert_eq!(state.time(), &[1.5, 2.0]);
    assert_eq!(state.qpos(), &[1.0, 1.0, 9.0, 10.0]);
    assert_eq!(state.qvel(), &[1.0, 4.0]);
}
