//! G18@P6 离散积分测试：冻结上游 DISCRETE 单步、原生 f64 一级判定、恒等分支
//! 与拒绝面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../../fixtures/discrete/cases.json")).unwrap()
}

fn f32_list(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect()
}

struct FixtureLayouts {
    jnt_type: Vec<i32>,
    jnt_qposadr: Vec<i32>,
    jnt_dofadr: Vec<i32>,
}

impl FixtureLayouts {
    fn new(fixture: &Value) -> Self {
        Self {
            jnt_type: i32_list(&fixture["joints"]["jnt_type"]),
            jnt_qposadr: i32_list(&fixture["joints"]["jnt_qposadr"]),
            jnt_dofadr: i32_list(&fixture["joints"]["jnt_dofadr"]),
        }
    }

    fn view(&self, nq: usize, nv: usize) -> JointLayout<'_> {
        JointLayout {
            nq,
            nv,
            jnt_type: &self.jnt_type,
            jnt_qposadr: &self.jnt_qposadr,
            jnt_dofadr: &self.jnt_dofadr,
        }
    }
}

fn i32_list(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap() as i32)
        .collect()
}

fn count_bitwise(product: &[f32], expected: &[f32]) -> (usize, f64) {
    let mut bitwise = 0;
    let mut max_abs = 0.0_f64;
    for (left, right) in product.iter().zip(expected.iter()) {
        if left.to_bits() == right.to_bits() {
            bitwise += 1;
        }
        max_abs = max_abs.max((*left as f64 - *right as f64).abs());
    }
    (bitwise, max_abs)
}

#[test]
fn discrete_step_matches_frozen_upstream_and_native() {
    let fixture = fixture();
    assert_eq!(
        fixture["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g18-p6"
    );
    let layouts = FixtureLayouts::new(&fixture);
    let nq = fixture["joints"]["nq"].as_u64().unwrap() as usize;
    let nv = fixture["joints"]["nv"].as_u64().unwrap() as usize;
    let layout = layouts.view(nq, nv);
    let timestep = fixture["joints"]["timestep"].as_f64().unwrap() as f32;
    let upstream_tolerance = fixture["tolerance"]["upstream_abs"].as_f64().unwrap();
    let native_tolerance = fixture["tolerance"]["native_abs"].as_f64().unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    let mut upstream_bitwise = 0;
    let mut upstream_total = 0;
    let mut upstream_max = 0.0_f64;
    let mut native_max = 0.0_f64;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let qpos = f32_list(&case["qpos"]);
        let qvel = f32_list(&case["qvel"]);
        let qacc = f32_list(&case["qacc"]);
        let state = IntegratorState {
            nworld: 1,
            qpos: &qpos,
            qvel: &qvel,
        };
        let step = discrete(&layout, state, &qacc, timestep, 0.0).unwrap();
        let expected_qpos = f32_list(&case["discrete_qpos"]);
        let expected_qvel = f32_list(&case["discrete_qvel"]);
        let (bitwise, max_abs) = count_bitwise(&step.qpos, &expected_qpos);
        upstream_bitwise += bitwise;
        upstream_total += step.qpos.len();
        upstream_max = upstream_max.max(max_abs);
        assert!(
            max_abs <= upstream_tolerance,
            "case {name} qpos upstream delta {max_abs:.3e}"
        );
        let (bitwise, max_abs) = count_bitwise(&step.qvel, &expected_qvel);
        upstream_bitwise += bitwise;
        upstream_total += step.qvel.len();
        upstream_max = upstream_max.max(max_abs);
        assert!(
            max_abs <= upstream_tolerance,
            "case {name} qvel upstream delta {max_abs:.3e}"
        );
        assert_eq!(step.time, case["discrete_time"].as_f64().unwrap() as f32);
        let native_qpos = f32_list(&case["native_qpos"]);
        let native_qvel = f32_list(&case["native_qvel"]);
        for (product, native) in step.qpos.iter().zip(native_qpos.iter()) {
            native_max = native_max.max((*product as f64 - *native as f64).abs());
        }
        for (product, native) in step.qvel.iter().zip(native_qvel.iter()) {
            native_max = native_max.max((*product as f64 - *native as f64).abs());
        }
        assert!(
            native_max <= native_tolerance,
            "case {name} native delta {native_max:.3e}"
        );
        // 与半隐式 Euler 直接分支等价（DISCRETE == _advance(qacc)）。
        let euler_step = crate::physics::euler(&layout, state, &qacc, timestep, 0.0).unwrap();
        assert_eq!(step.qpos, euler_step.qpos);
        assert_eq!(step.qvel, euler_step.qvel);
    }
    println!(
        "discrete bitwise={upstream_bitwise}/{upstream_total} upstream_max={upstream_max:.3e} native_max={native_max:.3e}"
    );
}

#[test]
fn discrete_acc_identity_branch_and_unsupported_damped_branch() {
    let qacc = [0.5_f32, -1.5, 2.0];
    let identity = discrete_acc(DiscreteAccBranch::Identity, &qacc).unwrap();
    assert_eq!(identity, qacc);
    let error = discrete_acc(DiscreteAccBranch::DampedEuler, &qacc).unwrap_err();
    assert_eq!(error, IntegratorError::UnsupportedDiscreteAcc);
}
