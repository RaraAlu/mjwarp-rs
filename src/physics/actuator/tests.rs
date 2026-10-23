//! G05 执行器切片测试：冻结上游参考与输入校验。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/actuation/actuation-cases.json"
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

fn bool_list(value: &Value) -> Vec<bool> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_bool().unwrap())
        .collect()
}

/// 按 fixture 构造模型与冻结数据并执行一次 `fwd_actuation`。
fn run(value: &Value, arrays: &Value, inputs: &Value) -> Result<ActuationOutput, ActuationError> {
    let model_meta = &value["model"];
    let nworld = value["nworld"].as_u64().unwrap() as usize;
    let nv = model_meta["nv"].as_u64().unwrap() as usize;
    let nu = model_meta["nu"].as_u64().unwrap() as usize;
    let na = model_meta["na"].as_u64().unwrap() as usize;
    let timestep = model_meta["timestep"].as_f64().unwrap() as f32;
    let moment_width = inputs["moment_width"].as_u64().unwrap() as usize;

    let dyntype = i32_list(&arrays["dyntype"]);
    let gaintype = i32_list(&arrays["gaintype"]);
    let biastype = i32_list(&arrays["biastype"]);
    let ctrladr = i32_list(&arrays["ctrladr"]);
    let ctrlnum = i32_list(&arrays["ctrlnum"]);
    let actadr = i32_list(&arrays["actadr"]);
    let actnum = i32_list(&arrays["actnum"]);
    let dynprm = f32_list(&arrays["dynprm"]);
    let gainprm = f32_list(&arrays["gainprm"]);
    let biasprm = f32_list(&arrays["biasprm"]);
    let actlimited = bool_list(&arrays["actlimited"]);
    let actrange = f32_list(&arrays["actrange"]);
    let actearly = bool_list(&arrays["actearly"]);
    let forcelimited = bool_list(&arrays["forcelimited"]);
    let forcerange = f32_list(&arrays["forcerange"]);
    let ctrllimited = bool_list(&arrays["ctrllimited"]);
    let ctrlrange = f32_list(&arrays["ctrlrange"]);
    let ctrl = f32_list(&inputs["ctrl"]);
    let act = f32_list(&inputs["act"]);
    let actuator_length = f32_list(&inputs["actuator_length"]);
    let actuator_velocity = f32_list(&inputs["actuator_velocity"]);
    let moment_rownnz = i32_list(&inputs["moment_rownnz"]);
    let moment_rowadr = i32_list(&inputs["moment_rowadr"]);
    let moment_colind = i32_list(&inputs["moment_colind"]);
    let actuator_moment = f32_list(&inputs["actuator_moment"]);

    let model = ActuatorModel {
        timestep,
        disable_clampctrl: false,
        dyntype: &dyntype,
        gaintype: &gaintype,
        biastype: &biastype,
        ctrladr: &ctrladr,
        ctrlnum: &ctrlnum,
        actadr: &actadr,
        actnum: &actnum,
        dynprm: &dynprm,
        gainprm: &gainprm,
        biasprm: &biasprm,
        actlimited: &actlimited,
        actrange: &actrange,
        actearly: &actearly,
        forcelimited: &forcelimited,
        forcerange: &forcerange,
        ctrllimited: &ctrllimited,
        ctrlrange: &ctrlrange,
    };
    let data = ActuationData {
        nworld,
        nv,
        nu,
        na,
        ctrl: &ctrl,
        act: &act,
        actuator_length: &actuator_length,
        actuator_velocity: &actuator_velocity,
        moment_width,
        moment_rownnz: &moment_rownnz,
        moment_rowadr: &moment_rowadr,
        moment_colind: &moment_colind,
        actuator_moment: &actuator_moment,
    };
    fwd_actuation(model, data)
}

fn fixture_value(value: &Value) -> Value {
    Value::Object(
        [
            ("model", value["model"].clone()),
            ("nworld", value["nworld"].clone()),
            ("frozen_inputs", value["frozen_inputs"].clone()),
        ]
        .into_iter()
        .map(|(key, item)| (key.to_string(), item))
        .collect(),
    )
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
fn actuator_outputs_match_frozen_upstream_reference() {
    let value = fixture();
    assert_eq!(
        value["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g05-p4"
    );
    let meta = fixture_value(&value);
    let output = run(&meta, &value["model_arrays"], &value["frozen_inputs"]).unwrap();
    let expected = &value["expected"];
    assert_bits("act_dot", output.act_dot(), &expected["act_dot"]);
    assert_bits(
        "actuator_force",
        output.actuator_force(),
        &expected["actuator_force"],
    );
    assert_bits(
        "qfrc_actuator",
        output.qfrc_actuator(),
        &expected["qfrc_actuator"],
    );
}

#[test]
fn rejects_unsupported_actuator_types() {
    let value = fixture();
    for (field, unsupported) in [("dyntype", 4_i32), ("gaintype", 2_i32), ("biastype", 2_i32)] {
        let mut arrays = value["model_arrays"].clone();
        let mut values = i32_list(&arrays[field]);
        values[0] = unsupported;
        arrays[field] = Value::Array(values.into_iter().map(Value::from).collect());
        let error = run(&fixture_value(&value), &arrays, &value["frozen_inputs"]).unwrap_err();
        assert!(
            matches!(error, ActuationError::Unsupported { field: rejected, .. } if rejected == field),
            "{field} must be rejected, got {error}"
        );
    }
}

#[test]
fn rejects_invalid_dimensions_and_moment_rows() {
    let value = fixture();
    let meta = fixture_value(&value);
    let empty: [f32; 0] = [];
    let empty_i32: [i32; 0] = [];
    let empty_bool: [bool; 0] = [];
    let model = ActuatorModel {
        timestep: 0.002,
        disable_clampctrl: false,
        dyntype: &empty_i32,
        gaintype: &empty_i32,
        biastype: &empty_i32,
        ctrladr: &empty_i32,
        ctrlnum: &empty_i32,
        actadr: &empty_i32,
        actnum: &empty_i32,
        dynprm: &empty,
        gainprm: &empty,
        biasprm: &empty,
        actlimited: &empty_bool,
        actrange: &empty,
        actearly: &empty_bool,
        forcelimited: &empty_bool,
        forcerange: &empty,
        ctrllimited: &empty_bool,
        ctrlrange: &empty,
    };
    let data = ActuationData {
        nworld: 1,
        nv: 1,
        nu: 0,
        na: 0,
        ctrl: &empty,
        act: &empty,
        actuator_length: &empty,
        actuator_velocity: &empty,
        moment_width: 0,
        moment_rownnz: &empty_i32,
        moment_rowadr: &empty_i32,
        moment_colind: &empty_i32,
        actuator_moment: &empty,
    };
    assert_eq!(
        fwd_actuation(model, data),
        Err(ActuationError::InvalidDimension { field: "nactuator" })
    );

    let mut inputs = value["frozen_inputs"].clone();
    let mut rownnz = i32_list(&inputs["moment_rownnz"]);
    rownnz[0] = 99;
    inputs["moment_rownnz"] = Value::Array(rownnz.into_iter().map(Value::from).collect());
    let error = run(&meta, &value["model_arrays"], &inputs).unwrap_err();
    assert!(matches!(error, ActuationError::InvalidIndex { .. }));
}
