//! G27@P7 qpos0 依赖常量测试：冻结上游参考、动态重算与校验面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../../fixtures/set-const/qpos-dependent.json"
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

fn run(case: &Value) -> QposDependentConstants {
    let model = &case["model_arrays"];
    let inputs = &case["inputs"];
    set_const_0(QposDependentInput {
        cam_bodyid: &i32_list(&model["cam_bodyid"]),
        cam_targetbodyid: &i32_list(&model["cam_targetbodyid"]),
        cam_xpos: &f32_list(&inputs["cam_xpos"]),
        cam_xmat: &f32_list(&inputs["cam_xmat"]),
        light_bodyid: &i32_list(&model["light_bodyid"]),
        light_targetbodyid: &i32_list(&model["light_targetbodyid"]),
        light_xpos: &f32_list(&inputs["light_xpos"]),
        light_xdir: &f32_list(&inputs["light_xdir"]),
        xpos: &f32_list(&inputs["xpos"]),
        subtree_com: &f32_list(&inputs["subtree_com"]),
        ten_length: &f32_list(&inputs["ten_length"]),
        m_rownnz: &i32_list(&inputs["m_rownnz"]),
        m_rowadr: &i32_list(&inputs["m_rowadr"]),
        m_values: &f32_list(&inputs["m_values"]),
    })
    .unwrap()
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
fn qpos_dependent_constants_match_frozen_upstream_reference() {
    let value = fixture();
    assert_eq!(
        value["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g27-p7"
    );
    let cases = value["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    for case in cases {
        let constants = run(case);
        let expected = &case["expected"];
        assert_bits(
            "tendon_length0",
            constants.tendon_length0(),
            &expected["tendon_length0"],
        );
        assert_bits("cam_pos0", constants.cam_pos0(), &expected["cam_pos0"]);
        assert_bits(
            "cam_poscom0",
            constants.cam_poscom0(),
            &expected["cam_poscom0"],
        );
        assert_bits("cam_mat0", constants.cam_mat0(), &expected["cam_mat0"]);
        assert_bits(
            "light_pos0",
            constants.light_pos0(),
            &expected["light_pos0"],
        );
        assert_bits(
            "light_poscom0",
            constants.light_poscom0(),
            &expected["light_poscom0"],
        );
        assert_bits(
            "light_dir0",
            constants.light_dir0(),
            &expected["light_dir0"],
        );
        let meaninertia = f32_list(&expected["meaninertia"])[0];
        assert_eq!(constants.meaninertia().to_bits(), meaninertia.to_bits());
    }
}

#[test]
fn dynamic_recompute_reflects_qpos0_changes() {
    let value = fixture();
    let cases = value["cases"].as_array().unwrap();
    let default = run(&cases[0]);
    let modified = run(&cases[1]);
    assert_ne!(default.tendon_length0(), modified.tendon_length0());
    assert_ne!(default.cam_pos0(), modified.cam_pos0());
    assert_ne!(default.cam_poscom0(), modified.cam_poscom0());
    assert_ne!(default.light_pos0(), modified.light_pos0());
    assert_eq!(default.meaninertia(), modified.meaninertia());
    // 同一输入的两次调用结果一致（无缓存级联状态）。
    let repeated = run(&cases[1]);
    assert_eq!(modified, repeated);
}

#[test]
fn zero_dof_uses_upstream_default_meaninertia() {
    let empty_f32: [f32; 0] = [];
    let empty_i32: [i32; 0] = [];
    let constants = set_const_0(QposDependentInput {
        cam_bodyid: &empty_i32,
        cam_targetbodyid: &empty_i32,
        cam_xpos: &empty_f32,
        cam_xmat: &empty_f32,
        light_bodyid: &empty_i32,
        light_targetbodyid: &empty_i32,
        light_xpos: &empty_f32,
        light_xdir: &empty_f32,
        xpos: &[0.0, 0.0, 0.0],
        subtree_com: &[0.0, 0.0, 0.0],
        ten_length: &empty_f32,
        m_rownnz: &empty_i32,
        m_rowadr: &empty_i32,
        m_values: &empty_f32,
    })
    .unwrap();
    assert_eq!(constants.meaninertia(), 1.0);
    assert_eq!(constants.tendon_length0(), &[] as &[f32]);
}

#[test]
fn rejects_invalid_targets_and_lengths() {
    let value = fixture();
    let case = &value["cases"].as_array().unwrap()[0];
    let model = &case["model_arrays"];
    let inputs = &case["inputs"];

    let mut cam_target = i32_list(&model["cam_targetbodyid"]);
    cam_target[0] = 99;
    let error = set_const_0(QposDependentInput {
        cam_bodyid: &i32_list(&model["cam_bodyid"]),
        cam_targetbodyid: &cam_target,
        cam_xpos: &f32_list(&inputs["cam_xpos"]),
        cam_xmat: &f32_list(&inputs["cam_xmat"]),
        light_bodyid: &i32_list(&model["light_bodyid"]),
        light_targetbodyid: &i32_list(&model["light_targetbodyid"]),
        light_xpos: &f32_list(&inputs["light_xpos"]),
        light_xdir: &f32_list(&inputs["light_xdir"]),
        xpos: &f32_list(&inputs["xpos"]),
        subtree_com: &f32_list(&inputs["subtree_com"]),
        ten_length: &f32_list(&inputs["ten_length"]),
        m_rownnz: &i32_list(&inputs["m_rownnz"]),
        m_rowadr: &i32_list(&inputs["m_rowadr"]),
        m_values: &f32_list(&inputs["m_values"]),
    })
    .unwrap_err();
    assert!(matches!(
        error,
        InputError::InvalidTopology {
            field: "cam_targetbodyid",
            ..
        }
    ));

    let error = set_const_0(QposDependentInput {
        cam_bodyid: &i32_list(&model["cam_bodyid"]),
        cam_targetbodyid: &i32_list(&model["cam_targetbodyid"]),
        cam_xpos: &f32_list(&inputs["cam_xpos"])[..3],
        cam_xmat: &f32_list(&inputs["cam_xmat"]),
        light_bodyid: &i32_list(&model["light_bodyid"]),
        light_targetbodyid: &i32_list(&model["light_targetbodyid"]),
        light_xpos: &f32_list(&inputs["light_xpos"]),
        light_xdir: &f32_list(&inputs["light_xdir"]),
        xpos: &f32_list(&inputs["xpos"]),
        subtree_com: &f32_list(&inputs["subtree_com"]),
        ten_length: &f32_list(&inputs["ten_length"]),
        m_rownnz: &i32_list(&inputs["m_rownnz"]),
        m_rowadr: &i32_list(&inputs["m_rowadr"]),
        m_values: &f32_list(&inputs["m_values"]),
    })
    .unwrap_err();
    assert!(matches!(
        error,
        InputError::LengthMismatch {
            field: "cam_xpos",
            ..
        }
    ));
}
