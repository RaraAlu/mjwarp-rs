//! G23@P7 传感器测试：冻结上游 sensordata、原生 f64 一级判定、cutoff 截断、
//! 类型/时序与校验面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/sensor/cases.json")).unwrap()
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

struct OwnedLayout {
    nq: usize,
    nv: usize,
    nsensordata: usize,
    sensor_type: Vec<i32>,
    sensor_datatype: Vec<i32>,
    sensor_objid: Vec<i32>,
    sensor_adr: Vec<i32>,
    sensor_cutoff: Vec<f32>,
    jnt_qposadr: Vec<i32>,
    jnt_dofadr: Vec<i32>,
}

impl OwnedLayout {
    fn new(fixture: &Value) -> Self {
        let layout = &fixture["layout"];
        Self {
            nq: layout["nq"].as_u64().unwrap() as usize,
            nv: layout["nv"].as_u64().unwrap() as usize,
            nsensordata: layout["nsensordata"].as_u64().unwrap() as usize,
            sensor_type: i32_list(&layout["sensor_type"]),
            sensor_datatype: i32_list(&layout["sensor_datatype"]),
            sensor_objid: i32_list(&layout["sensor_objid"]),
            sensor_adr: i32_list(&layout["sensor_adr"]),
            sensor_cutoff: f32_list(&layout["sensor_cutoff"]),
            jnt_qposadr: i32_list(&layout["jnt_qposadr"]),
            jnt_dofadr: i32_list(&layout["jnt_dofadr"]),
        }
    }

    fn view(&self) -> SensorLayout<'_> {
        SensorLayout {
            nq: self.nq,
            nv: self.nv,
            nsensordata: self.nsensordata,
            sensor_type: &self.sensor_type,
            sensor_datatype: &self.sensor_datatype,
            sensor_objid: &self.sensor_objid,
            sensor_adr: &self.sensor_adr,
            sensor_cutoff: &self.sensor_cutoff,
            jnt_qposadr: &self.jnt_qposadr,
            jnt_dofadr: &self.jnt_dofadr,
        }
    }
}

fn assert_within(label: &str, product: &[f32], expected: &[f32], tolerance: f64) -> f64 {
    assert_eq!(product.len(), expected.len(), "{label} length");
    let mut max_abs = 0.0_f64;
    let mut bitwise = 0;
    for (left, right) in product.iter().zip(expected.iter()) {
        if left.to_bits() == right.to_bits() {
            bitwise += 1;
        }
        max_abs = max_abs.max((*left as f64 - *right as f64).abs());
    }
    assert!(
        max_abs <= tolerance,
        "{label} max abs {max_abs:.3e} > {tolerance:.3e} (bitwise {bitwise}/{})",
        product.len()
    );
    max_abs
}

#[test]
fn sensors_match_frozen_upstream_and_native_reference() {
    let fixture = fixture();
    assert_eq!(
        fixture["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g23-p7"
    );
    let layout = OwnedLayout::new(&fixture);
    let upstream_tolerance = fixture["tolerance"]["upstream_abs"].as_f64().unwrap();
    let native_tolerance = fixture["tolerance"]["native_abs"].as_f64().unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 3);
    let mut upstream_max = 0.0_f64;
    let mut native_max = 0.0_f64;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let qpos = f32_list(&case["qpos"]);
        let qvel = f32_list(&case["qvel"]);
        let time = [case["time"].as_f64().unwrap() as f32];
        let pos = sensor_pos(&layout.view(), 1, &qpos, &time).unwrap();
        let vel = sensor_vel(&layout.view(), 1, &qvel).unwrap();
        upstream_max = upstream_max.max(assert_within(
            &format!("{name} pos"),
            &pos,
            &f32_list(&case["sensordata_pos"]),
            upstream_tolerance,
        ));
        upstream_max = upstream_max.max(assert_within(
            &format!("{name} vel"),
            &vel,
            &f32_list(&case["sensordata_vel"]),
            upstream_tolerance,
        ));
        native_max = native_max.max(assert_within(
            &format!("{name} pos native"),
            &pos,
            &f32_list(&case["native_sensordata_pos"]),
            native_tolerance,
        ));
        native_max = native_max.max(assert_within(
            &format!("{name} vel native"),
            &vel,
            &f32_list(&case["native_sensordata_vel"]),
            native_tolerance,
        ));
        // 位置阶段不写速度传感器、速度阶段不写位置传感器（时序/单位）。
        for (index, sensor_type) in layout.sensor_type.iter().enumerate() {
            let adr = layout.sensor_adr[index] as usize;
            match *sensor_type {
                SENSOR_JOINTVEL => assert_eq!(pos[adr], 0.0),
                SENSOR_JOINTPOS | SENSOR_BALLQUAT | SENSOR_CLOCK => {
                    assert_eq!(vel[adr], 0.0);
                }
                _ => {}
            }
        }
        // cutoff 截断：slide_pos（adr=1）在 cutoff 用例（qpos=0.8）被截到 0.4。
        if name == "cutoff" {
            let slide_pos_adr = layout.sensor_adr[1] as usize;
            assert_eq!(pos[slide_pos_adr], 0.4);
        }
    }
    println!("sensors upstream_max={upstream_max:.3e} native_max={native_max:.3e}");
}

#[test]
fn rejects_invalid_layout_and_unsupported_type() {
    let fixture = fixture();
    let layout = OwnedLayout::new(&fixture);
    let case = &fixture["cases"][0];
    let qpos = f32_list(&case["qpos"]);
    let qvel = f32_list(&case["qvel"]);
    let time = [0.0_f32];
    let error = sensor_pos(&layout.view(), 1, &qpos[1..], &time).unwrap_err();
    assert!(matches!(
        error,
        SensorError::InvalidLength { field: "qpos", .. }
    ));
    let error = sensor_vel(&layout.view(), 1, &qvel[1..]).unwrap_err();
    assert!(matches!(
        error,
        SensorError::InvalidLength { field: "qvel", .. }
    ));
    let mut broken = OwnedLayout::new(&fixture);
    broken.sensor_adr[0] = 99;
    let error = sensor_pos(&broken.view(), 1, &qpos, &time).unwrap_err();
    assert!(matches!(
        error,
        SensorError::InvalidIndex {
            field: "sensor_adr",
            ..
        }
    ));
    let mut broken = OwnedLayout::new(&fixture);
    broken.jnt_qposadr[0] = 99;
    let error = sensor_pos(&broken.view(), 1, &qpos, &time).unwrap_err();
    assert!(matches!(
        error,
        SensorError::InvalidIndex {
            field: "jnt_qposadr",
            ..
        }
    ));
    let mut broken = OwnedLayout::new(&fixture);
    broken.sensor_cutoff.pop();
    let error = sensor_pos(&broken.view(), 1, &qpos, &time).unwrap_err();
    assert!(matches!(
        error,
        SensorError::InvalidLength {
            field: "sensor_cutoff",
            ..
        }
    ));
}
