//! G16@P6 积分器测试：冻结上游逐位对照、原生 f64 一级判定、合成 ODE 收敛阶
//! 与输入校验面。

use super::*;
use serde_json::Value;

/// 上游 `mjtJoint` 的 slide 编号（合成 ODE 用两个标量 slide 关节）。
const SYNTHETIC_SLIDE: i32 = 2;

type Stage = (Vec<f32>, Vec<f32>);

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/integrator/cases.json")).unwrap()
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

fn count_bitwise(product: &[f32], expected: &[f32]) -> (usize, f64) {
    let mut bitwise = 0;
    let mut max_abs = 0.0_f64;
    assert_eq!(product.len(), expected.len());
    for (left, right) in product.iter().zip(expected.iter()) {
        if left.to_bits() == right.to_bits() {
            bitwise += 1;
        }
        max_abs = max_abs.max((*left as f64 - *right as f64).abs());
    }
    (bitwise, max_abs)
}

fn expected_stages(case: &Value) -> [Stage; 4] {
    let stages = case["rk4"]["stages"].as_array().unwrap();
    assert_eq!(stages.len(), 4);
    std::array::from_fn(|index| {
        (
            f32_list(&stages[index]["qvel"]),
            f32_list(&stages[index]["qacc"]),
        )
    })
}

#[test]
fn euler_and_rk4_match_frozen_upstream_reference() {
    let fixture = fixture();
    assert_eq!(
        fixture["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g16-p6"
    );
    let layouts = FixtureLayouts::new(&fixture);
    let nq = fixture["joints"]["nq"].as_u64().unwrap() as usize;
    let nv = fixture["joints"]["nv"].as_u64().unwrap() as usize;
    let layout = layouts.view(nq, nv);
    let timestep = fixture["joints"]["timestep"].as_f64().unwrap() as f32;
    let native_tolerance = fixture["tolerance"]["native_abs"].as_f64().unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 3);
    // 上游 CUDA 内核会把 `pos + timestep*qvel` 收缩为 FMA，宿主侧 f32 逐位一致
    // 会差 1 ulp；冻结容差沿批次 1 口径取 1e-6（FMA 收缩/libm 末位差，非放宽物理容差）。
    let upstream_tolerance = 1.0e-6_f64;
    let mut euler_bitwise = 0;
    let mut euler_total = 0;
    let mut rk4_bitwise = 0;
    let mut rk4_total = 0;
    let mut euler_max = 0.0_f64;
    let mut rk4_max = 0.0_f64;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let qpos = f32_list(&case["qpos"]);
        let qvel = f32_list(&case["qvel"]);
        let state = IntegratorState {
            nworld: 1,
            qpos: &qpos,
            qvel: &qvel,
        };

        let expected = &case["euler"];
        let qacc = f32_list(&expected["qacc"]);
        let step = euler(&layout, state, &qacc, timestep, 0.0).unwrap();
        let (bitwise, max_abs) = count_bitwise(&step.qpos, &f32_list(&expected["qpos"]));
        euler_bitwise += bitwise;
        euler_total += step.qpos.len();
        euler_max = euler_max.max(max_abs);
        assert!(
            max_abs <= upstream_tolerance,
            "euler {name} qpos upstream delta {max_abs:.3e}"
        );
        let (bitwise, max_abs) = count_bitwise(&step.qvel, &f32_list(&expected["qvel"]));
        euler_bitwise += bitwise;
        euler_total += step.qvel.len();
        euler_max = euler_max.max(max_abs);
        assert!(
            max_abs <= upstream_tolerance,
            "euler {name} qvel upstream delta {max_abs:.3e}"
        );
        assert_eq!(step.time, expected["time"].as_f64().unwrap() as f32);
        for (product, native) in step
            .qpos
            .iter()
            .zip(f32_list(&expected["native_qpos"]).iter())
        {
            assert!(
                ((*product as f64) - (*native as f64)).abs() <= native_tolerance,
                "euler {name} qpos vs native"
            );
        }
        for (product, native) in step
            .qvel
            .iter()
            .zip(f32_list(&expected["native_qvel"]).iter())
        {
            assert!(
                ((*product as f64) - (*native as f64)).abs() <= native_tolerance,
                "euler {name} qvel vs native"
            );
        }

        let stages = expected_stages(case);
        let stage_refs: [Rk4Stage<'_>; 4] = [
            Rk4Stage {
                qvel: &stages[0].0,
                qacc: &stages[0].1,
            },
            Rk4Stage {
                qvel: &stages[1].0,
                qacc: &stages[1].1,
            },
            Rk4Stage {
                qvel: &stages[2].0,
                qacc: &stages[2].1,
            },
            Rk4Stage {
                qvel: &stages[3].0,
                qacc: &stages[3].1,
            },
        ];
        let expected = &case["rk4"];
        let step = rungekutta4(&layout, state, stage_refs, timestep, 0.0).unwrap();
        let (bitwise, max_abs) = count_bitwise(&step.qpos, &f32_list(&expected["qpos"]));
        rk4_bitwise += bitwise;
        rk4_total += step.qpos.len();
        rk4_max = rk4_max.max(max_abs);
        assert!(
            max_abs <= upstream_tolerance,
            "rk4 {name} qpos upstream delta {max_abs:.3e}"
        );
        let (bitwise, max_abs) = count_bitwise(&step.qvel, &f32_list(&expected["qvel"]));
        rk4_bitwise += bitwise;
        rk4_total += step.qvel.len();
        rk4_max = rk4_max.max(max_abs);
        assert!(
            max_abs <= upstream_tolerance,
            "rk4 {name} qvel upstream delta {max_abs:.3e}"
        );
        assert_eq!(step.time, expected["time"].as_f64().unwrap() as f32);
        for (product, native) in step
            .qpos
            .iter()
            .zip(f32_list(&expected["native_qpos"]).iter())
        {
            assert!(
                ((*product as f64) - (*native as f64)).abs() <= native_tolerance,
                "rk4 {name} qpos vs native"
            );
        }
        for (product, native) in step
            .qvel
            .iter()
            .zip(f32_list(&expected["native_qvel"]).iter())
        {
            assert!(
                ((*product as f64) - (*native as f64)).abs() <= native_tolerance,
                "rk4 {name} qvel vs native"
            );
        }
        let replay_max = expected["replay_max_abs"].as_f64().unwrap();
        assert_eq!(replay_max, 0.0, "authoring replay must be exact");
    }
    println!(
        "euler bitwise={euler_bitwise}/{euler_total} max_abs={euler_max:.3e}\
         rk4 bitwise={rk4_bitwise}/{rk4_total} max_abs={rk4_max:.3e}"
    );
    assert!(euler_max <= upstream_tolerance);
    assert!(rk4_max <= upstream_tolerance);
}

#[test]
fn rejects_invalid_shapes() {
    let fixture = fixture();
    let layouts = FixtureLayouts::new(&fixture);
    let nq = fixture["joints"]["nq"].as_u64().unwrap() as usize;
    let nv = fixture["joints"]["nv"].as_u64().unwrap() as usize;
    let layout = layouts.view(nq, nv);
    let qpos = vec![0.0_f32; nq];
    let qvel = vec![0.0_f32; nv];
    let error = next_position(&layout, 1, &qpos[1..], &qvel, 0.01, 1.0).unwrap_err();
    assert!(matches!(
        error,
        IntegratorError::InvalidLength { field: "qpos", .. }
    ));
    let error = next_velocity(1, nv, &qvel, &qvel[1..], 0.01, 1.0).unwrap_err();
    assert!(matches!(
        error,
        IntegratorError::InvalidLength { field: "qacc", .. }
    ));
    let mut broken = FixtureLayouts::new(&fixture);
    broken.jnt_dofadr[0] = 99;
    let broken_layout = broken.view(nq, nv);
    let state = IntegratorState {
        nworld: 1,
        qpos: &qpos,
        qvel: &qvel,
    };
    let error = euler(&broken_layout, state, &qvel, 0.01, 0.0).unwrap_err();
    assert!(matches!(
        error,
        IntegratorError::InvalidIndex {
            field: "jnt_dofadr",
            ..
        }
    ));
}

fn harmonic_derivative(qpos: [f32; 2]) -> [f32; 2] {
    // 合成 ODE：q' = v, v' = -q；两标量 slide 关节承载 (q, v)。
    [-qpos[0], -qpos[1]]
}

fn exact_harmonic(time: f64, initial: [f64; 2]) -> [f64; 2] {
    [
        initial[0] * time.cos() + initial[1] * time.sin(),
        -initial[0] * time.sin() + initial[1] * time.cos(),
    ]
}

fn euler_synthetic(steps: usize, dt: f32, initial: [f64; 2]) -> [f64; 2] {
    let jnt_type = [SYNTHETIC_SLIDE, SYNTHETIC_SLIDE];
    let jnt_qposadr = [0, 1];
    let jnt_dofadr = [0, 1];
    let layout = JointLayout {
        nq: 2,
        nv: 2,
        jnt_type: &jnt_type,
        jnt_qposadr: &jnt_qposadr,
        jnt_dofadr: &jnt_dofadr,
    };
    let mut state = [initial[0] as f32, initial[1] as f32];
    for _ in 0..steps {
        let qvel = [state[1], -state[0]];
        let qacc = harmonic_derivative(state);
        let step = euler(
            &layout,
            IntegratorState {
                nworld: 1,
                qpos: &state,
                qvel: &qvel,
            },
            &qacc,
            dt,
            0.0,
        )
        .unwrap();
        state = [step.qpos[0], step.qpos[1]];
    }
    [state[0] as f64, state[1] as f64]
}

fn rk4_synthetic(steps: usize, dt: f32, initial: [f64; 2]) -> [f64; 2] {
    let jnt_type = [SYNTHETIC_SLIDE, SYNTHETIC_SLIDE];
    let jnt_qposadr = [0, 1];
    let jnt_dofadr = [0, 1];
    let layout = JointLayout {
        nq: 2,
        nv: 2,
        jnt_type: &jnt_type,
        jnt_qposadr: &jnt_qposadr,
        jnt_dofadr: &jnt_dofadr,
    };
    let a_tableau = [0.5_f32, 0.5, 1.0];
    let mut state = [initial[0] as f32, initial[1] as f32];
    for _ in 0..steps {
        let initial_state = state;
        let initial_qvel = [initial_state[1], -initial_state[0]];
        let mut stages: [Stage; 4] = std::array::from_fn(|_| (vec![0.0_f32; 2], vec![0.0_f32; 2]));
        stages[0] = (
            initial_qvel.to_vec(),
            harmonic_derivative(initial_state).to_vec(),
        );
        let mut qvel_prev = initial_qvel;
        let mut qacc_prev = harmonic_derivative(initial_state);
        for (index, scale) in a_tableau.iter().enumerate() {
            let qpos_stage = [
                initial_state[0] + (scale * qvel_prev[0]) * dt,
                initial_state[1] + (scale * qvel_prev[1]) * dt,
            ];
            let qvel_stage = [
                initial_qvel[0] + (scale * qacc_prev[0]) * dt,
                initial_qvel[1] + (scale * qacc_prev[1]) * dt,
            ];
            let qacc_stage = harmonic_derivative(qpos_stage);
            stages[index + 1] = (qvel_stage.to_vec(), qacc_stage.to_vec());
            qvel_prev = qvel_stage;
            qacc_prev = qacc_stage;
        }
        let stage_refs: [Rk4Stage<'_>; 4] = [
            Rk4Stage {
                qvel: &stages[0].0,
                qacc: &stages[0].1,
            },
            Rk4Stage {
                qvel: &stages[1].0,
                qacc: &stages[1].1,
            },
            Rk4Stage {
                qvel: &stages[2].0,
                qacc: &stages[2].1,
            },
            Rk4Stage {
                qvel: &stages[3].0,
                qacc: &stages[3].1,
            },
        ];
        let step = rungekutta4(
            &layout,
            IntegratorState {
                nworld: 1,
                qpos: &initial_state,
                qvel: &initial_qvel,
            },
            stage_refs,
            dt,
            0.0,
        )
        .unwrap();
        state = [step.qpos[0], step.qpos[1]];
    }
    [state[0] as f64, state[1] as f64]
}

#[test]
fn synthetic_harmonic_ode_shows_expected_order() {
    let initial = [1.0_f64, 0.0];
    let exact = exact_harmonic(1.0, initial);
    let mut euler_errors = Vec::new();
    let mut rk4_errors = Vec::new();
    for steps in [2_usize, 4, 8, 16] {
        let dt = 1.0_f32 / steps as f32;
        let euler_state = euler_synthetic(steps, dt, initial);
        let rk4_state = rk4_synthetic(steps, dt, initial);
        let euler_error =
            ((euler_state[0] - exact[0]).powi(2) + (euler_state[1] - exact[1]).powi(2)).sqrt();
        let rk4_error =
            ((rk4_state[0] - exact[0]).powi(2) + (rk4_state[1] - exact[1]).powi(2)).sqrt();
        euler_errors.push(euler_error);
        rk4_errors.push(rk4_error);
    }
    println!("euler errors: {euler_errors:?}");
    println!("rk4 errors: {rk4_errors:?}");
    for window in euler_errors.windows(2) {
        let ratio = window[0] / window[1];
        assert!(
            (1.6..=2.6).contains(&ratio),
            "euler convergence ratio {ratio}"
        );
    }
    for window in rk4_errors.windows(2) {
        let ratio = window[0] / window[1];
        assert!(ratio > 10.0, "rk4 convergence ratio {ratio}");
    }
}
