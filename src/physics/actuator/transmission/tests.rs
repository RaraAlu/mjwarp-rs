//! G05@P7 关节传动测试：冻结上游参考、端到端执行器链与拒绝面。

use super::*;
use crate::physics::actuator::{ActuationData, ActuatorModel, fwd_actuation};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../../fixtures/actuation/transmission-cases.json"
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

fn assert_bits(field: &str, product: &[f32], expected: &[f32]) {
    assert_eq!(product.len(), expected.len(), "{field} length");
    for (index, (actual, target)) in product.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            actual.to_bits(),
            target.to_bits(),
            "{field}[{index}] product={actual} expected={target}"
        );
    }
}

fn run_transmission(model_arrays: &Value, initial: &Value, model: &Value) -> JointTransmission {
    let nq = model["nq"].as_u64().unwrap() as usize;
    let nv = model["nv"].as_u64().unwrap() as usize;
    transmission(
        JointActuatorModel {
            nv,
            trntype: &i32_list(&model_arrays["actuator_trntype"]),
            trnid: &i32_list(&model_arrays["actuator_trnid"]),
            gear: &f32_list(&model_arrays["actuator_gear"]),
            jnt_type: &i32_list(&model_arrays["jnt_type"]),
            jnt_qposadr: &i32_list(&model_arrays["jnt_qposadr"]),
            jnt_dofadr: &i32_list(&model_arrays["jnt_dofadr"]),
        },
        JointTransmissionData {
            nworld: 2,
            nq,
            qpos: &f32_list(&initial["qpos"]),
            qvel: &f32_list(&initial["qvel"]),
        },
    )
    .unwrap()
}

#[test]
fn joint_transmission_matches_frozen_upstream_reference() {
    let value = fixture();
    assert_eq!(
        value["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g05-p7"
    );
    let model = &value["model"];
    let nact = model["nactuator"].as_u64().unwrap() as usize;
    let nworld = value["nworld"].as_u64().unwrap() as usize;
    let reference = &value["transmission_reference"];
    let result = run_transmission(&value["model_arrays"], &value["initial"], model);
    assert_bits(
        "actuator_length",
        result.actuator_length(),
        &f32_list(&reference["actuator_length"]),
    );
    assert_bits(
        "actuator_velocity",
        result.actuator_velocity(),
        &f32_list(&reference["actuator_velocity"]),
    );
    let upstream_rownnz = i32_list(&reference["moment_rownnz"]);
    let upstream_rowadr = i32_list(&reference["moment_rowadr"]);
    let upstream_colind = i32_list(&reference["moment_colind"]);
    let upstream_moment = f32_list(&reference["actuator_moment"]);
    let upstream_width = reference["moment_width"].as_u64().unwrap() as usize;
    assert_eq!(result.moment_width(), upstream_width);
    for world in 0..nworld {
        for uid in 0..nact {
            let index = world * nact + uid;
            assert_eq!(result.rownnz()[index], 1);
            assert_eq!(upstream_rownnz[index], 1);
            let product_base = world * result.moment_width() + result.rowadr()[index] as usize;
            let upstream_base = world * upstream_width + upstream_rowadr[index] as usize;
            assert_eq!(
                result.colind()[product_base],
                upstream_colind[upstream_base],
                "moment colind world={world} actuator={uid}"
            );
            assert_eq!(
                result.moment()[product_base].to_bits(),
                upstream_moment[upstream_base].to_bits(),
                "moment value world={world} actuator={uid}"
            );
        }
    }
}

#[test]
fn transmission_chain_matches_frozen_actuation_outputs() {
    let value = fixture();
    let model = &value["model"];
    let nv = model["nv"].as_u64().unwrap() as usize;
    let nu = model["nu"].as_u64().unwrap() as usize;
    let na = model["na"].as_u64().unwrap() as usize;
    let nact = model["nactuator"].as_u64().unwrap() as usize;
    let nworld = value["nworld"].as_u64().unwrap() as usize;
    let result = run_transmission(&value["model_arrays"], &value["initial"], model);
    let arrays = &value["model_arrays"];
    let initial = &value["initial"];
    let output = fwd_actuation(
        ActuatorModel {
            timestep: model["timestep"].as_f64().unwrap() as f32,
            disable_clampctrl: false,
            dyntype: &i32_list(&arrays["dyntype"]),
            gaintype: &i32_list(&arrays["gaintype"]),
            biastype: &i32_list(&arrays["biastype"]),
            ctrladr: &i32_list(&arrays["ctrladr"]),
            ctrlnum: &i32_list(&arrays["ctrlnum"]),
            actadr: &i32_list(&arrays["actadr"]),
            actnum: &i32_list(&arrays["actnum"]),
            dynprm: &f32_list(&arrays["dynprm"]),
            gainprm: &f32_list(&arrays["gainprm"]),
            biasprm: &f32_list(&arrays["biasprm"]),
            actlimited: &arrays["actlimited"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item.as_bool().unwrap())
                .collect::<Vec<_>>(),
            actrange: &f32_list(&arrays["actrange"]),
            actearly: &arrays["actearly"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item.as_bool().unwrap())
                .collect::<Vec<_>>(),
            forcelimited: &arrays["forcelimited"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item.as_bool().unwrap())
                .collect::<Vec<_>>(),
            forcerange: &f32_list(&arrays["forcerange"]),
            ctrllimited: &arrays["ctrllimited"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item.as_bool().unwrap())
                .collect::<Vec<_>>(),
            ctrlrange: &f32_list(&arrays["ctrlrange"]),
        },
        ActuationData {
            nworld: 2,
            nv,
            nu,
            na,
            ctrl: &f32_list(&initial["ctrl"]),
            act: &f32_list(&initial["act"]),
            actuator_length: result.actuator_length(),
            actuator_velocity: result.actuator_velocity(),
            moment_width: result.moment_width(),
            moment_rownnz: result.rownnz(),
            moment_rowadr: result.rowadr(),
            moment_colind: result.colind(),
            actuator_moment: result.moment(),
        },
    )
    .unwrap();
    let expected = &value["expected_actuation"];
    assert_bits("act_dot", output.act_dot(), &f32_list(&expected["act_dot"]));
    assert_bits(
        "actuator_force",
        output.actuator_force(),
        &f32_list(&expected["actuator_force"]),
    );
    assert_bits(
        "qfrc_actuator",
        output.qfrc_actuator(),
        &f32_list(&expected["qfrc_actuator"]),
    );
    assert_eq!(output.actuator_force().len(), nworld * nact);
}

#[test]
fn rejects_unsupported_transmission_and_joint_types() {
    let value = fixture();
    let arrays = value["model_arrays"].clone();
    let model = &value["model"];
    let initial = &value["initial"];
    let nq = model["nq"].as_u64().unwrap() as usize;
    let nv = model["nv"].as_u64().unwrap() as usize;

    let mut trntype = i32_list(&arrays["actuator_trntype"]);
    trntype[0] = 1; // JOINTINPARENT
    let error = transmission(
        JointActuatorModel {
            nv,
            trntype: &trntype,
            trnid: &i32_list(&arrays["actuator_trnid"]),
            gear: &f32_list(&arrays["actuator_gear"]),
            jnt_type: &i32_list(&arrays["jnt_type"]),
            jnt_qposadr: &i32_list(&arrays["jnt_qposadr"]),
            jnt_dofadr: &i32_list(&arrays["jnt_dofadr"]),
        },
        JointTransmissionData {
            nworld: 2,
            nq,
            qpos: &f32_list(&initial["qpos"]),
            qvel: &f32_list(&initial["qvel"]),
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ActuationError::Unsupported {
            field: "actuator_trntype",
            ..
        }
    ));

    let mut jnt_type = i32_list(&arrays["jnt_type"]);
    jnt_type[0] = 1; // ball
    let error = transmission(
        JointActuatorModel {
            nv,
            trntype: &i32_list(&arrays["actuator_trntype"]),
            trnid: &i32_list(&arrays["actuator_trnid"]),
            gear: &f32_list(&arrays["actuator_gear"]),
            jnt_type: &jnt_type,
            jnt_qposadr: &i32_list(&arrays["jnt_qposadr"]),
            jnt_dofadr: &i32_list(&arrays["jnt_dofadr"]),
        },
        JointTransmissionData {
            nworld: 2,
            nq,
            qpos: &f32_list(&initial["qpos"]),
            qvel: &f32_list(&initial["qvel"]),
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ActuationError::Unsupported {
            field: "jnt_type",
            ..
        }
    ));
}
