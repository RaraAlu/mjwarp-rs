#![cfg(feature = "cuda-probe")]
//! G03反向LDL与多右端项辅助。
use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::InertialModelInput,
    physics::{MassSolveOutput, probe_mass_solve},
    runtime::TransferSession,
};
use serde_json::Value;
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::{floats, model};

const MODELS: [&str; 6] = [
    include_str!("../fixtures/mass-matrix/mixed-joints.json"),
    include_str!("../fixtures/mass-matrix/inertial-tree.json"),
    include_str!("../fixtures/mass-matrix/rotated-tree.json"),
    include_str!("../fixtures/mass-matrix/zero-dof.json"),
    include_str!("../fixtures/mass-matrix/massless-tree.json"),
    include_str!("../fixtures/mass-matrix/armature-chain.json"),
];
const SOLVES: [&str; 6] = [
    include_str!("../fixtures/mass-solve/mixed-joints.json"),
    include_str!("../fixtures/mass-solve/inertial-tree.json"),
    include_str!("../fixtures/mass-solve/rotated-tree.json"),
    include_str!("../fixtures/mass-solve/zero-dof.json"),
    include_str!("../fixtures/mass-solve/massless-tree.json"),
    include_str!("../fixtures/mass-solve/armature-chain.json"),
];
// Freeze these before the first solve GPU run. Matrix probe tolerances stay
// unchanged; inversion can amplify small float32 matrix differences.
const ABS: f64 = 2e-4;
const REL: f64 = 2e-4;
const RESIDUAL: f64 = 2e-5;
const RECONSTRUCTION: f64 = 2e-6;

fn compare(out: &MassSolveOutput, world: usize, case: &Value) {
    let w = out.world(world).unwrap();
    let n = out.nv();
    let mut maximum: f64 = 0.0;
    for (key, actual) in [("ld", w.ld), ("diagonal_inverse", w.diagonal_inverse)] {
        let expected = case[key].as_array().unwrap();
        assert_eq!(actual.len(), expected.len());
        for (&a, e) in actual.iter().zip(expected) {
            let e = e.as_f64().unwrap();
            let error = (f64::from(a) - e).abs();
            maximum = maximum.max(error);
            assert!(
                a.is_finite() && error <= ABS + REL * e.abs(),
                "G03 {key}: {a} vs {e}"
            );
        }
    }
    let matrix = case["matrix"].as_array().unwrap();
    let rownorm = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| matrix[i * n + j].as_f64().unwrap().abs())
                .sum::<f64>()
        })
        .fold(0.0, f64::max);
    let mut reconstruction: f64 = 0.0;
    for i in 0..n {
        assert!(w.ld[i * n + i] > 0.0 && w.diagonal_inverse[i] > 0.0);
        assert!(
            (f64::from(w.ld[i * n + i]) * f64::from(w.diagonal_inverse[i]) - 1.0).abs() <= 2e-6
        );
        let mut error = 0.0;
        for j in 0..n {
            if j > i {
                assert_eq!(w.ld[i * n + j], 0.0);
            }
            let reconstructed = (i.max(j)..n)
                .map(|k| {
                    let lki = if k == i {
                        1.0
                    } else {
                        f64::from(w.ld[k * n + i])
                    };
                    let lkj = if k == j {
                        1.0
                    } else {
                        f64::from(w.ld[k * n + j])
                    };
                    lki * f64::from(w.ld[k * n + k]) * lkj
                })
                .sum::<f64>();
            error += (reconstructed - matrix[i * n + j].as_f64().unwrap()).abs();
        }
        reconstruction = reconstruction.max(if rownorm == 0.0 {
            error
        } else {
            error / rownorm
        });
    }
    assert!(
        reconstruction <= RECONSTRUCTION,
        "G03 reconstruction={reconstruction:e}"
    );
    let mut residual: f64 = 0.0;
    assert_eq!(w.solution.len(), out.rhs_count() * n);
    for r in 0..out.rhs_count() {
        let x = &w.solution[r * n..(r + 1) * n];
        let expected = &case["solution"].as_array().unwrap()[r % 3 * n..(r % 3 + 1) * n];
        let rhs = &case["rhs"].as_array().unwrap()[r % 3 * n..(r % 3 + 1) * n];
        for (&a, e) in x.iter().zip(expected) {
            let e = e.as_f64().unwrap();
            let error = (f64::from(a) - e).abs();
            maximum = maximum.max(error);
            assert!(
                a.is_finite() && error <= ABS + REL * e.abs(),
                "G03 solve: {a} vs {e}"
            );
        }
        let xnorm = x.iter().map(|&v| f64::from(v).abs()).fold(0.0, f64::max);
        let bnorm = rhs
            .iter()
            .map(|v| v.as_f64().unwrap().abs())
            .fold(0.0, f64::max);
        let denominator = rownorm * xnorm + bnorm;
        for i in 0..n {
            let actual = (0..n)
                .map(|j| matrix[i * n + j].as_f64().unwrap() * f64::from(x[j]))
                .sum::<f64>();
            let error = (actual - rhs[i].as_f64().unwrap()).abs();
            residual = residual.max(if denominator == 0.0 {
                error
            } else {
                error / denominator
            });
        }
        if r % 3 == 0 {
            assert!(x.iter().all(|&v| v == 0.0));
        }
    }
    assert!(residual <= RESIDUAL, "G03 residual={residual:e}");
    eprintln!(
        "G03-solve case={} world={world} max_abs_error={maximum:e} residual={residual:e} reconstruction={reconstruction:e}",
        case["id"]
    );
}
fn rhs(case: &Value, n: usize, count: usize) -> Vec<f32> {
    let base = floats(case, "rhs");
    (0..count)
        .flat_map(|r| base[r % 3 * n..(r % 3 + 1) * n].iter().copied())
        .collect()
}

#[test]
fn solve_references_freeze_shapes_tolerances_and_matching_model_states() {
    for (m, s) in MODELS.into_iter().zip(SOLVES) {
        let m: Value = serde_json::from_str(m).unwrap();
        let s: Value = serde_json::from_str(s).unwrap();
        let model = model(&m);
        let n = model.kinematics().nv();
        assert_eq!(s["native_version"], 3012000);
        assert_eq!(s["seed"], 1789);
        assert_eq!(s["rhs_seed"], 9701);
        assert_eq!(s["ntendon"], 0);
        assert_eq!(s["nu"], 0);
        assert_eq!(s["rhs_count"], 3);
        assert_eq!(s["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(s["relative_tolerance"].as_f64(), Some(REL));
        assert_eq!(s["residual_tolerance"].as_f64(), Some(RESIDUAL));
        assert_eq!(s["reconstruction_tolerance"].as_f64(), Some(RECONSTRUCTION));
        for key in ["nq", "nv", "nbody"] {
            assert_eq!(m[key], s[key]);
        }
        let cases = s["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 8);
        for (i, c) in cases.iter().enumerate() {
            assert_eq!(c["id"], i);
            assert_eq!(c["qpos"], m["cases"][i]["qpos"]);
            assert_eq!(c["matrix"], m["cases"][i]["matrix"]);
            for (key, len) in [
                ("ld", n * n),
                ("diagonal_inverse", n),
                ("rhs", 3 * n),
                ("solution", 3 * n),
            ] {
                let values = c[key].as_array().unwrap();
                assert_eq!(values.len(), len);
                assert!(values.iter().all(|v| v.as_f64().unwrap().is_finite()));
            }
        }
    }
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_factors_and_multiple_rhs_match_independent_native_solves() {
    let session = TransferSession::new(0).unwrap();
    for (m, s) in MODELS.into_iter().zip(SOLVES) {
        let m = model(&serde_json::from_str(m).unwrap());
        let s: Value = serde_json::from_str(s).unwrap();
        let cases = s["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let b: Vec<_> = cases.iter().flat_map(|c| floats(c, "rhs")).collect();
        let out = probe_mass_solve(&session, &m, 8, &q, 3, &b).unwrap();
        assert_eq!(out.worlds(), 8);
        assert_eq!(out.nv(), m.kinematics().nv());
        assert_eq!(out.rhs_count(), 3);
        assert!(out.world(8).is_err());
        for (w, c) in cases.iter().enumerate() {
            compare(&out, w, c);
        }
    }
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn mass_solve_batches_cross_blocks_without_mutating_inputs() {
    let session = TransferSession::new(0).unwrap();
    let m = model(&serde_json::from_str(MODELS[2]).unwrap());
    let s: Value = serde_json::from_str(SOLVES[2]).unwrap();
    let cases = s["cases"].as_array().unwrap();
    for worlds in [1, 255, 256, 257, 513] {
        let q: Vec<_> = (0..worlds)
            .flat_map(|w| floats(&cases[w % 8], "qpos"))
            .collect();
        let b: Vec<_> = (0..worlds)
            .flat_map(|w| floats(&cases[w % 8], "rhs"))
            .collect();
        let original = (q.clone(), b.clone());
        let out = probe_mass_solve(&session, &m, worlds, &q, 3, &b).unwrap();
        assert_eq!((q, b), original);
        for w in 0..worlds {
            compare(&out, w, &cases[w % 8]);
        }
    }
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn rhs_counts_one_two_three_and_five_reuse_each_world_factor() {
    let session = TransferSession::new(0).unwrap();
    let m = model(&serde_json::from_str(MODELS[0]).unwrap());
    let s: Value = serde_json::from_str(SOLVES[0]).unwrap();
    let cases = s["cases"].as_array().unwrap();
    let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
    for count in [1, 2, 3, 5] {
        let b: Vec<_> = cases
            .iter()
            .flat_map(|c| rhs(c, m.kinematics().nv(), count))
            .collect();
        let out = probe_mass_solve(&session, &m, 8, &q, count, &b).unwrap();
        for (w, c) in cases.iter().enumerate() {
            compare(&out, w, c);
        }
    }
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn analytical_chain_recovers_known_vector_and_owns_results_after_session_drop() {
    let session = TransferSession::new(0).unwrap();
    let m = model(&serde_json::from_str(MODELS[5]).unwrap());
    let s: Value = serde_json::from_str(SOLVES[5]).unwrap();
    let q = floats(&s["cases"][0], "qpos");
    let out =
        probe_mass_solve(&session, &m, 1, &q, 2, &[0.0, 0.0, 0.0, 9.75, 5.125, 14.25]).unwrap();
    drop(session);
    let w = out.world(0).unwrap();
    for (&a, e) in w.solution.iter().zip([0.0, 0.0, 0.0, 1.0, 2.0, 3.0]) {
        assert!((a - e).abs() <= 2e-6);
    }
    for (&a, e) in w.ld.iter().zip([
        479.0 / 108.0,
        0.0,
        0.0,
        28.0 / 27.0,
        27.0 / 16.0,
        0.0,
        0.0,
        0.0,
        19.0 / 4.0,
    ]) {
        assert!((f64::from(a) - e).abs() <= 2e-6);
    }
    eprintln!("G03-solve analytical_world=0");
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn singular_models_reject_without_regularization_and_healthy_sessions_recover() {
    let session = TransferSession::new(0).unwrap();
    let m = model(&serde_json::from_str(MODELS[5]).unwrap());
    let s: Value = serde_json::from_str(SOLVES[5]).unwrap();
    let q = floats(&s["cases"][0], "qpos");
    assert!(matches!(
        probe_mass_solve(&session, &m, 1, &q, 1, &[1.0]),
        Err(TransferError::Input(InputError::LengthMismatch {
            field: "mass_solve_rhs",
            ..
        }))
    ));
    assert!(matches!(
        probe_mass_solve(&session, &m, 1, &q, 1, &[1.0, f32::NAN, 1.0]),
        Err(TransferError::Input(InputError::NonFinite {
            field: "mass_solve_rhs",
            index: 1
        }))
    ));
    let mut invalid = q.clone();
    invalid[0] = f32::NAN;
    assert!(matches!(
        probe_mass_solve(&session, &m, 1, &invalid, 1, &[1.0; 3]),
        Err(TransferError::Input(InputError::NonFinite {
            field: "kinematics_qpos",
            ..
        }))
    ));
    let mut i = m.fields().clone();
    i.body_mass.fill(0.0);
    i.body_inertia.fill(0.0);
    i.dof_armature.fill(0.0);
    let singular = InertialModelInput::new(m.kinematics().clone(), i.clone()).unwrap();
    assert_eq!(
        probe_mass_solve(&session, &singular, 1, &q, 1, &[1.0; 3]).unwrap_err(),
        TransferError::InvalidPivot {
            world: 0,
            dof: 2,
            value: 0.0
        }
    );
    eprintln!("G03-solve singular_rejected_world=0");
    i.dof_armature.copy_from_slice(&[0.25, 0.5, 0.75]);
    let tuned = InertialModelInput::new(m.kinematics().clone(), i).unwrap();
    let out = probe_mass_solve(&session, &tuned, 1, &q, 1, &[1.0; 3]).unwrap();
    for (&a, e) in out
        .world(0)
        .unwrap()
        .solution
        .iter()
        .zip([4.0, 2.0, 4.0 / 3.0])
    {
        assert!((a - e).abs() <= 2e-6);
    }
    let out = probe_mass_solve(&session, &m, 1, &q, 3, &floats(&s["cases"][0], "rhs")).unwrap();
    compare(&out, 0, &s["cases"][0]);
}
#[cfg(feature = "native-model-probe")]
#[test]
#[ignore = "requires trusted Windows MuJoCo DLL, NVIDIA driver and NVRTC"]
fn native_snapshots_feed_gpu_solves_without_native_physics_at_runtime() {
    use mjwarp_rs::io::NativeModelProbe;
    let dll = std::path::PathBuf::from(
        std::env::var_os("MJWARP_MUJOCO_DLL").expect("provide trusted DLL"),
    );
    let session = TransferSession::new(0).unwrap();
    let mjbs: [&[u8]; 6] = [
        include_bytes!("../fixtures/native-probe/mixed-joints.mjb"),
        include_bytes!("../fixtures/native-probe/inertial-tree.mjb"),
        include_bytes!("../fixtures/kinematics/rotated-tree.mjb"),
        include_bytes!("../fixtures/native-probe/zero-dof.mjb"),
        include_bytes!("../fixtures/com-position/massless-tree.mjb"),
        include_bytes!("../fixtures/mass-matrix/armature-chain.mjb"),
    ];
    for ((m, s), mjb) in MODELS.into_iter().zip(SOLVES).zip(mjbs) {
        // SAFETY: The caller supplies the hash-verified trusted candidate DLL.
        // These static compiled models use that exact known ABI.
        let native = unsafe { NativeModelProbe::load_trusted(&dll, mjb) }.unwrap();
        let m_native = native.inertial_snapshot().unwrap().into_model().unwrap();
        assert_eq!(m_native, model(&serde_json::from_str(m).unwrap()));
        drop(native);
        let s: Value = serde_json::from_str(s).unwrap();
        let cases = s["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let b: Vec<_> = cases.iter().flat_map(|c| floats(c, "rhs")).collect();
        let out = probe_mass_solve(&session, &m_native, 8, &q, 3, &b).unwrap();
        for (w, c) in cases.iter().enumerate() {
            compare(&out, w, c);
        }
    }
}
