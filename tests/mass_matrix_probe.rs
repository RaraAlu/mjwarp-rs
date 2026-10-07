#![cfg(feature = "cuda-probe")]
//! G02刚体质量矩阵子集参考。
use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::InertialModelInput,
    physics::{MassMatrixOutput, probe_mass_matrix},
    runtime::TransferSession,
};
use serde_json::Value;
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::{floats, model};
const FIXTURES: [&str; 6] = [
    include_str!("../fixtures/mass-matrix/mixed-joints.json"),
    include_str!("../fixtures/mass-matrix/inertial-tree.json"),
    include_str!("../fixtures/mass-matrix/rotated-tree.json"),
    include_str!("../fixtures/mass-matrix/zero-dof.json"),
    include_str!("../fixtures/mass-matrix/massless-tree.json"),
    include_str!("../fixtures/mass-matrix/armature-chain.json"),
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;
fn compare(out: &MassMatrixOutput, world: usize, case: &Value) {
    let w = out.world(world).unwrap();
    let mut maximum: f64 = 0.0;
    for (key, actual) in [("crb", w.crb), ("matrix", w.matrix)] {
        let expected = case[key].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{key}");
        for (index, (&a, e)) in actual.iter().zip(expected).enumerate() {
            let e = e.as_f64().unwrap();
            let error = (f64::from(a) - e).abs();
            maximum = maximum.max(error);
            assert!(
                a.is_finite() && error <= ABS + REL * e.abs(),
                "G02 {key} world={world} index={index}: actual={a} expected={e} error={error}"
            );
        }
    }
    eprintln!(
        "G02-mass case={} world={world} max_abs_error={maximum:e}",
        case["id"]
    );
}
fn properties(out: &MassMatrixOutput, model: &InertialModelInput, world: usize, positive: bool) {
    let m = out.world(world).unwrap().matrix;
    let nv = out.nv();
    for i in 0..nv {
        for j in 0..i {
            assert_eq!(m[i * nv + j].to_bits(), m[j * nv + i].to_bits());
            let mut ancestor = i as i32;
            while ancestor >= 0 && ancestor != j as i32 {
                ancestor = model.fields().dof_parentid[ancestor as usize];
            }
            if ancestor < 0 {
                assert_eq!(m[i * nv + j], 0.0, "non-ancestor coupling");
            }
        }
    }
    // A test-only f64 Cholesky checks positive-definiteness of these physical
    // fixtures. The public helper neither factors M nor promises invertibility.
    if positive {
        let mut l = vec![0.0f64; nv * nv];
        for i in 0..nv {
            for j in 0..=i {
                let sum = f64::from(m[i * nv + j])
                    - (0..j).map(|k| l[i * nv + k] * l[j * nv + k]).sum::<f64>();
                l[i * nv + j] = if i == j {
                    assert!(sum > 0.0, "non-positive pivot {i}: {sum}");
                    sum.sqrt()
                } else {
                    sum / l[j * nv + j]
                };
            }
        }
    }
}
#[test]
fn mass_references_fix_tolerances_shapes_and_supported_feature_subset() {
    let mut types = [false; 4];
    let mut armature = false;
    for fixture in FIXTURES {
        let v: Value = serde_json::from_str(fixture).unwrap();
        let m = model(&v);
        assert_eq!(v["native_version"], 3012000);
        assert_eq!(v["seed"], 1789);
        assert_eq!(v["ntendon"], 0);
        assert_eq!(v["nu"], 0);
        assert_eq!(v["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(v["relative_tolerance"].as_f64(), Some(REL));
        for &ty in &m.kinematics().fields().jnt_type {
            types[ty as usize] = true;
        }
        armature |= m.fields().dof_armature.iter().any(|&a| a > 0.0);
        let cases = v["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 8);
        for (id, c) in cases.iter().enumerate() {
            assert_eq!(c["id"], id);
            assert_eq!(floats(c, "qpos").len(), m.kinematics().nq());
            for (key, len) in [
                ("crb", 10 * m.kinematics().nbody()),
                ("matrix", m.kinematics().nv().pow(2)),
            ] {
                let values = c[key].as_array().unwrap();
                assert_eq!(values.len(), len);
                assert!(values.iter().all(|e| e.as_f64().unwrap().is_finite()));
            }
        }
    }
    assert_eq!(types, [true; 4]);
    assert!(armature);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn four_joint_mass_matrices_and_crb_match_independent_native_references() {
    let session = TransferSession::new(0).unwrap();
    for fixture in FIXTURES {
        let v: Value = serde_json::from_str(fixture).unwrap();
        let m = model(&v);
        let cases = v["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let out = probe_mass_matrix(&session, &m, cases.len(), &q).unwrap();
        assert_eq!(out.worlds(), 8);
        assert_eq!(out.nbody(), m.kinematics().nbody());
        assert_eq!(out.nv(), m.kinematics().nv());
        assert!(out.world(8).is_err());
        for (w, c) in cases.iter().enumerate() {
            compare(&out, w, c);
            properties(&out, &m, w, true);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn mass_matrix_batches_cross_blocks_and_preserve_input_states() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[2]).unwrap();
    let m = model(&v);
    let cases = v["cases"].as_array().unwrap();
    for worlds in [1, 255, 256, 257, 513] {
        let q: Vec<_> = (0..worlds)
            .flat_map(|w| floats(&cases[w % 8], "qpos"))
            .collect();
        let original = q.clone();
        let out = probe_mass_matrix(&session, &m, worlds, &q).unwrap();
        assert_eq!(q, original);
        for w in 0..worlds {
            compare(&out, w, &cases[w % 8]);
            properties(&out, &m, w, true);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn armature_chain_matches_hand_derived_matrix_and_kinetic_energy() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[5]).unwrap();
    let m = model(&v);
    let q = floats(&v["cases"][0], "qpos");
    let out = probe_mass_matrix(&session, &m, 1, &q).unwrap();
    drop(session);
    let actual = out.world(0).unwrap().matrix;
    let expected = [6.25, 1.75, 0.0, 1.75, 1.6875, 0.0, 0.0, 0.0, 4.75];
    for (&a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() <= 2e-6);
    }
    let velocity = [1.0, 2.0, 3.0];
    let mut energy = 0.0;
    for i in 0..3 {
        for j in 0..3 {
            energy += 0.5 * velocity[i] * f64::from(actual[3 * i + j]) * velocity[j];
        }
    }
    assert!((energy - 31.375).abs() <= 2e-5);
    eprintln!("G02-mass analytical_energy_world=0");
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn zero_inertia_models_return_semidefinite_matrices_without_factorization() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[5]).unwrap();
    let m = model(&v);
    let mut i = m.fields().clone();
    i.body_mass.fill(0.0);
    i.body_inertia.fill(0.0);
    i.dof_armature.fill(0.0);
    let zero = InertialModelInput::new(m.kinematics().clone(), i.clone()).unwrap();
    let q = floats(&v["cases"][0], "qpos");
    let out = probe_mass_matrix(&session, &zero, 1, &q).unwrap();
    assert!(out.world(0).unwrap().matrix.iter().all(|&a| a == 0.0));
    i.dof_armature.copy_from_slice(&[0.25, 0.5, 0.75]);
    let tuned = InertialModelInput::new(m.kinematics().clone(), i).unwrap();
    let out = probe_mass_matrix(&session, &tuned, 1, &q).unwrap();
    assert_eq!(
        out.world(0).unwrap().matrix,
        &[0.25, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.75]
    );
    eprintln!("G02-mass analytical_semidefinite_world=0");
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn mass_matrix_rejects_nonfinite_accumulation_without_poisoning_healthy_sessions() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[5]).unwrap();
    let m = model(&v);
    let q = floats(&v["cases"][0], "qpos");
    let mut bad = q.clone();
    bad[0] = f32::NAN;
    assert!(matches!(
        probe_mass_matrix(&session, &m, 1, &bad),
        Err(TransferError::Input(InputError::NonFinite {
            field: "kinematics_qpos",
            ..
        }))
    ));
    let mut i = m.fields().clone();
    i.body_inertia[3..9].fill(f32::MAX);
    let overflow = InertialModelInput::new(m.kinematics().clone(), i).unwrap();
    assert!(matches!(
        probe_mass_matrix(&session, &overflow, 1, &q),
        Err(TransferError::Input(InputError::NonFinite {
            field: "mass_matrix_output",
            ..
        }))
    ));
    let out = probe_mass_matrix(&session, &m, 1, &q).unwrap();
    compare(&out, 0, &v["cases"][0]);
    let mut i = m.fields().clone();
    i.dof_damping.fill(100.0);
    let damped = InertialModelInput::new(m.kinematics().clone(), i).unwrap();
    let same = probe_mass_matrix(&session, &damped, 1, &q).unwrap();
    assert_eq!(out.world(0).unwrap().matrix, same.world(0).unwrap().matrix);
}

#[cfg(feature = "native-model-probe")]
#[test]
#[ignore = "requires trusted Windows MuJoCo DLL, NVIDIA driver and NVRTC"]
fn native_snapshots_feed_gpu_mass_matrices_without_native_physics_at_runtime() {
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
    for (fixture, bytes) in FIXTURES.into_iter().zip(mjbs) {
        let v: Value = serde_json::from_str(fixture).unwrap();
        // SAFETY: The caller supplies the hash-verified trusted candidate DLL.
        // Static fixtures use that known compiled-model ABI.
        let native = unsafe { NativeModelProbe::load_trusted(&dll, bytes) }.unwrap();
        let m = native.inertial_snapshot().unwrap().into_model().unwrap();
        assert_eq!(m, model(&v));
        drop(native);
        let cases = v["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let out = probe_mass_matrix(&session, &m, cases.len(), &q).unwrap();
        for (w, c) in cases.iter().enumerate() {
            compare(&out, w, c);
        }
    }
}
