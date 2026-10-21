use super::compare::compare;
use super::*;

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_factors_and_multiple_rhs_match_independent_native_solves() {
    let mut audit = Audit::new("G03-factors-and-rhs");
    let session = TransferSession::new(0).unwrap();
    for (m, source) in MODELS.into_iter().zip(SOLVES) {
        let m = model(&serde_json::from_str(m).unwrap());
        let s: Value = serde_json::from_str(source).unwrap();
        let cases = s["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let b: Vec<_> = cases.iter().flat_map(|c| floats(c, "rhs")).collect();
        let out = probe_mass_solve(&session, &m, 8, &q, 3, &b).unwrap();
        assert_eq!(out.worlds(), 8);
        assert_eq!(out.nv(), m.kinematics().nv());
        assert_eq!(out.rhs_count(), 3);
        assert!(out.world(8).is_err());
        for (w, c) in cases.iter().enumerate() {
            compare(&out, w, c, source, &mut audit);
        }
    }
    audit.finish();
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn mass_solve_batches_cross_blocks_without_mutating_inputs() {
    let mut audit = Audit::new("G03-batches");
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
        let started = std::time::Instant::now();
        let out = probe_mass_solve(&session, &m, worlds, &q, 3, &b).unwrap();
        eprintln!(
            "G03-batch worlds={worlds} host_wall_ms={} includes_compile_and_transfer=true",
            started.elapsed().as_secs_f64() * 1000.0
        );
        assert_eq!((q, b), original);
        for w in 0..worlds {
            compare(&out, w, &cases[w % 8], SOLVES[2], &mut audit);
        }
    }
    audit.finish();
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn rhs_counts_one_two_three_and_five_reuse_each_world_factor() {
    let mut audit = Audit::new("G03-rhs-counts");
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
            compare(&out, w, c, SOLVES[0], &mut audit);
        }
    }
    audit.finish();
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
    let u00 = 6.25_f64.sqrt();
    for (&a, e) in w.ld.iter().zip([
        u00,
        1.75 / u00,
        0.0,
        0.0,
        (1.6875_f64 - 1.75 * 1.75 / 6.25).sqrt(),
        0.0,
        0.0,
        0.0,
        0.0,
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
            dof: 0,
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
    let mut audit = Audit::new("G03-recovery");
    compare(&out, 0, &s["cases"][0], SOLVES[5], &mut audit);
    audit.finish();
}
