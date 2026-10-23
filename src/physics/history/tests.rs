//! G24@P7 历史与延迟测试：冻结上游缓冲字、重置结果、读取值与原生 C 参考。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/history/cases.json")).unwrap()
}

fn f32_list(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect()
}

fn f64_list(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap())
        .collect()
}

fn f32_matrix(value: &Value) -> Vec<Vec<f32>> {
    value.as_array().unwrap().iter().map(f32_list).collect()
}

fn flatten(matrix: &[Vec<f32>]) -> Vec<f32> {
    matrix.iter().flatten().copied().collect()
}

fn bits_equal(left: f32, right: f32) -> bool {
    left.to_bits() == right.to_bits()
}

/// 冻结上游 `history._history_*` 回放：缓冲字逐位、读取值按 FMA 末位差容差。
#[test]
fn replays_frozen_upstream_scenarios() {
    let fixture = fixture();
    let mut word_mismatches = 0usize;
    let mut read_mismatches = 0usize;
    let mut read_values = 0usize;
    let mut read_abs = 0.0_f32;
    for scenario in fixture["mechanics"]["scenarios"].as_array().unwrap() {
        let name = scenario["name"].as_str().unwrap();
        let samples = scenario["samples"].as_u64().unwrap() as usize;
        let channels = scenario["channels"].as_u64().unwrap() as usize;
        let initial = f32_matrix(&scenario["initial_words"]);
        let mut buffer =
            HistoryBuffer::from_words(initial.len(), samples, channels, flatten(&initial)).unwrap();
        for op in scenario["ops"].as_array().unwrap() {
            let world = op["world"].as_u64().unwrap() as usize;
            let t = op["t"].as_f64().unwrap() as f32;
            match op["kind"].as_str().unwrap() {
                "insert_scalar" => {
                    let value = op["value"].as_f64().unwrap() as f32;
                    buffer.insert_scalar(world, t, value).unwrap();
                }
                "insert_vector" => {
                    let values = f32_list(&op["values"]);
                    buffer.insert_vector(world, t, &values).unwrap();
                }
                other => panic!("未知插入操作：{other}"),
            }
        }
        let expected = flatten(&f32_matrix(&scenario["expected_words"]));
        assert_eq!(buffer.words().len(), expected.len(), "场景 {name} 字长度");
        for (index, (actual, expected)) in buffer.words().iter().zip(expected.iter()).enumerate() {
            if !bits_equal(*actual, *expected) {
                word_mismatches += 1;
                eprintln!("场景 {name} 字[{index}]：{actual} vs {expected}");
            }
        }
        assert_eq!(word_mismatches, 0, "场景 {name} 缓冲字位型不一致");
        for read in scenario["reads"].as_array().unwrap() {
            let world = read["world"].as_u64().unwrap() as usize;
            let t = read["t"].as_f64().unwrap() as f32;
            let interp = HistoryInterp::from_raw(read["interp"].as_i64().unwrap() as i32).unwrap();
            let expected = f32_list(&read["expected"]);
            let mut out = vec![0.0_f32; expected.len()];
            if read["scalar"].as_bool().unwrap_or(false) {
                out[0] = buffer.read_scalar(world, t, interp).unwrap();
            } else if let Some(delay) = read.get("delay") {
                buffer
                    .read_delayed(world, t, delay.as_f64().unwrap() as f32, interp, &mut out)
                    .unwrap();
            } else {
                buffer.read_vector(world, t, interp, &mut out).unwrap();
            }
            for (actual, expected) in out.iter().zip(expected.iter()) {
                read_values += 1;
                read_abs = read_abs.max((actual - expected).abs());
                if !bits_equal(*actual, *expected) {
                    read_mismatches += 1;
                }
                assert!(
                    (actual - expected).abs() <= 1e-6,
                    "场景 {name} 读取：{actual} vs {expected}"
                );
            }
        }
    }
    println!(
        "冻结上游回放：读取 {read_values} 值，位型不一致 {read_mismatches}，max abs {read_abs:.3e}"
    );
}

/// 冻结上游重置内核：全量重置、掩码局部重置与 SO3+QUAT 初值逐位一致；
/// 另以 MuJoCo C `mj_resetData` 的 `d.history`（f64）做原生一级判定（1e-6）。
#[test]
fn resets_frozen_upstream_buffers() {
    let fixture = fixture();
    let reset = &fixture["reset"];
    let timestep = reset["timestep"].as_f64().unwrap() as f32;
    let worlds = reset["nworld"].as_u64().unwrap() as usize;
    let nhistory = reset["nhistory"].as_u64().unwrap() as usize;
    let initial = flatten(&f32_matrix(&reset["initial_words"]));
    let dirty = f32_list(&reset["dirty_words"]);
    assert_eq!(dirty.len(), worlds * nhistory);
    let buffers = reset["buffers"].as_array().unwrap();
    for case in reset["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let mask: Option<Vec<bool>> = case["mask"].as_array().map(|values| {
            values
                .iter()
                .map(|value| value.as_bool().unwrap())
                .collect()
        });
        let mut full = if case["dirty"].as_bool().unwrap_or(false) {
            dirty.clone()
        } else {
            initial.clone()
        };
        for buffer in buffers {
            let kind = buffer["kind"].as_str().unwrap();
            let buf_offset = buffer["buf_offset"].as_u64().unwrap() as usize;
            let samples = buffer["samples"].as_u64().unwrap() as usize;
            let channels = buffer["channels"].as_u64().unwrap() as usize;
            let region = DelayBufferLayout::new(samples, channels)
                .unwrap()
                .region_elements();
            let mut region_words = Vec::with_capacity(worlds * region);
            for world in 0..worlds {
                let start = world * nhistory + buf_offset;
                region_words.extend_from_slice(&full[start..start + region]);
            }
            let mut region_buffer =
                HistoryBuffer::from_words(worlds, samples, channels, region_words).unwrap();
            match kind {
                "actuator" => {
                    let so3_quat = buffer["so3_quat"].as_bool().unwrap();
                    region_buffer
                        .reset_actuator(mask.as_deref(), timestep, so3_quat)
                        .unwrap();
                }
                "sensor" => {
                    let interval = f32_list(&buffer["interval"]);
                    region_buffer
                        .reset_sensor(mask.as_deref(), timestep, [interval[0], interval[1]])
                        .unwrap();
                }
                other => panic!("未知缓冲类型：{other}"),
            }
            for world in 0..worlds {
                let start = world * nhistory + buf_offset;
                full[start..start + region]
                    .copy_from_slice(&region_buffer.words()[world * region..(world + 1) * region]);
            }
        }
        let expected = flatten(&f32_matrix(&case["expected_words"]));
        assert_eq!(full.len(), expected.len(), "重置用例 {name} 字长度");
        for (index, (actual, expected)) in full.iter().zip(expected.iter()).enumerate() {
            assert!(
                bits_equal(*actual, *expected),
                "重置用例 {name} 字[{index}]：{actual} vs {expected}"
            );
        }
    }
    let synthetic = reset["synthetic_actuator"].as_object().unwrap();
    let samples = synthetic["samples"].as_u64().unwrap() as usize;
    let channels = synthetic["channels"].as_u64().unwrap() as usize;
    let initial_words = synthetic["initial_words"].as_array().unwrap();
    let mut buffer =
        HistoryBuffer::from_words(1, samples, channels, f32_list(&initial_words[0])).unwrap();
    buffer
        .reset_actuator(None, synthetic["timestep"].as_f64().unwrap() as f32, true)
        .unwrap();
    let expected = f32_list(&synthetic["expected_words"][0]);
    for (index, (actual, expected)) in buffer.words().iter().zip(expected.iter()).enumerate() {
        assert!(
            bits_equal(*actual, *expected),
            "SO3+QUAT 重置 字[{index}]：{actual} vs {expected}"
        );
    }
    let native = f64_list(&reset["native_words"]);
    let native_abs = initial
        .iter()
        .zip(native.iter())
        .map(|(upstream, native)| (f64::from(*upstream) - native).abs())
        .fold(0.0_f64, f64::max);
    assert!(
        native_abs <= 1e-6,
        "原生 C 一级判定 max abs = {native_abs:.3e}"
    );
    println!(
        "重置用例 {} 个；原生 C mj_resetData max abs {native_abs:.3e}",
        reset["cases"].as_array().unwrap().len()
    );
}

/// 物理索引映射、环形二分与端点语义。
#[test]
fn maps_logical_indices_and_finds_brackets() {
    assert_eq!(HistoryBuffer::physical_index(1, 3, 0), 2);
    assert_eq!(HistoryBuffer::physical_index(1, 3, 1), 0);
    assert_eq!(HistoryBuffer::physical_index(1, 3, 2), 1);
    assert_eq!(HistoryBuffer::physical_index(3, 4, 0), 0);
    assert_eq!(HistoryBuffer::physical_index(3, 4, 3), 3);
    let mut buffer = HistoryBuffer::new(1, 4, 1).unwrap();
    buffer.reset_actuator(None, 0.01, false).unwrap();
    // 初始时间戳为 -0.04,-0.03,-0.02,-0.01，cursor=3。
    assert_eq!(buffer.find_index(0, -0.04).unwrap(), 0);
    assert_eq!(buffer.find_index(0, -0.035).unwrap(), 1);
    assert_eq!(buffer.find_index(0, -0.01).unwrap(), 3);
    assert_eq!(buffer.find_index(0, 0.0).unwrap(), 4);
    assert_eq!(buffer.find_index(0, -1.0).unwrap(), 0);
}

/// 拒绝面：维度、长度、索引、非有限值、插值模式与掩码。
#[test]
fn rejects_invalid_history_inputs() {
    for dimensions in [(0, 1), (1, 0)] {
        assert_eq!(
            DelayBufferLayout::new(dimensions.0, dimensions.1),
            Err(HistoryError::InvalidDimension {
                field: if dimensions.0 == 0 {
                    "samples"
                } else {
                    "channels"
                }
            })
        );
    }
    assert_eq!(
        HistoryBuffer::new(0, 1, 1),
        Err(HistoryError::InvalidDimension { field: "worlds" })
    );
    assert_eq!(
        HistoryBuffer::from_words(1, 2, 1, vec![0.0; 4]),
        Err(HistoryError::LengthMismatch {
            field: "history_words",
            expected: 6,
            actual: 4
        })
    );
    let mut buffer = HistoryBuffer::new(1, 3, 2).unwrap();
    buffer.reset_actuator(None, 0.01, false).unwrap();
    let snapshot = buffer.words().to_vec();
    assert_eq!(
        buffer.insert_vector(0, 0.0, &[1.0]),
        Err(HistoryError::LengthMismatch {
            field: "values",
            expected: 2,
            actual: 1
        })
    );
    assert_eq!(buffer.words(), snapshot.as_slice());
    assert_eq!(
        buffer.insert_scalar(0, f32::NAN, 1.0),
        Err(HistoryError::NonFinite {
            field: "t",
            index: 0
        })
    );
    assert_eq!(
        buffer.insert_vector(0, 0.0, &[1.0, f32::INFINITY]),
        Err(HistoryError::NonFinite {
            field: "values",
            index: 1
        })
    );
    assert_eq!(
        buffer.read_scalar(0, 0.0, HistoryInterp::Linear),
        Err(HistoryError::ScalarReadOnVector { channels: 2 })
    );
    let mut out = [0.0_f32; 2];
    assert_eq!(
        buffer.read_vector(0, f32::NAN, HistoryInterp::Linear, &mut out),
        Err(HistoryError::NonFinite {
            field: "t",
            index: 0
        })
    );
    assert_eq!(
        buffer.read_vector(1, 0.0, HistoryInterp::Linear, &mut out),
        Err(HistoryError::InvalidIndex {
            field: "world",
            index: 1,
            limit: 1
        })
    );
    assert_eq!(
        buffer.read_vector(0, 0.0, HistoryInterp::Linear, &mut out[..1]),
        Err(HistoryError::LengthMismatch {
            field: "out",
            expected: 2,
            actual: 1
        })
    );
    assert_eq!(
        buffer.reset_actuator(Some(&[true, false]), 0.01, false),
        Err(HistoryError::LengthMismatch {
            field: "reset",
            expected: 1,
            actual: 2
        })
    );
    assert_eq!(
        buffer.reset_actuator(None, 0.0, false),
        Err(HistoryError::InvalidTimestep { timestep: 0.0 })
    );
    assert_eq!(
        buffer.reset_sensor(None, 0.01, [0.05, f32::NAN]),
        Err(HistoryError::NonFinite {
            field: "interval",
            index: 1
        })
    );
    assert_eq!(
        HistoryInterp::from_raw(2),
        Err(HistoryError::UnsupportedInterpolation { interp: 2 })
    );
    assert_eq!(HistoryInterp::from_raw(0), Ok(HistoryInterp::ZeroOrderHold));
    assert_eq!(HistoryInterp::from_raw(1), Ok(HistoryInterp::Linear));
    let mut corrupted = HistoryBuffer::new(1, 3, 1).unwrap();
    corrupted.reset_actuator(None, 0.01, false).unwrap();
    let mut words = corrupted.words().to_vec();
    words[1] = 7.0;
    let corrupted = HistoryBuffer::from_words(1, 3, 1, words).unwrap();
    assert_eq!(
        corrupted.cursor(0),
        Err(HistoryError::InvalidCursor {
            cursor: 7.0,
            samples: 3
        })
    );
}
