#![cfg(feature = "cuda-probe")]
//! G01质心子集的独立参考。
use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::InertialModelInput,
    physics::{ComPositionOutput, ComPositionWorld, probe_com_position},
    runtime::TransferSession,
};
use serde_json::Value;
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::{floats, model};

const FIXTURES: [&str; 5] = [
    include_str!("../fixtures/com-position/mixed-joints.json"),
    include_str!("../fixtures/com-position/inertial-tree.json"),
    include_str!("../fixtures/com-position/rotated-tree.json"),
    include_str!("../fixtures/com-position/zero-dof.json"),
    include_str!("../fixtures/com-position/massless-tree.json"),
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;
fn fields(w: ComPositionWorld<'_>) -> [(&'static str, &[f32]); 4] {
    [
        ("subtree_mass", w.subtree_mass),
        ("subtree_com", w.subtree_com),
        ("cinert", w.cinert),
        ("cdof", w.cdof),
    ]
}
fn compare(out: &ComPositionOutput, world: usize, case: &Value) {
    let mut maximum: f64 = 0.0;
    for (key, actual) in fields(out.world(world).unwrap()) {
        let expected = case[key].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{key}");
        for (index, (&a, e)) in actual.iter().zip(expected).enumerate() {
            let e = e.as_f64().unwrap();
            let error = (f64::from(a) - e).abs();
            maximum = maximum.max(error);
            assert!(
                a.is_finite() && error <= ABS + REL * e.abs(),
                "G01 COM {key} world={world} index={index}: actual={a} expected={e} error={error}"
            );
        }
    }
    eprintln!(
        "G01-COM case={} world={world} max_abs_error={maximum:e}",
        case["id"]
    );
}
fn compare_zero_mass(out: &ComPositionOutput, world: usize, native: &Value) {
    // Native 3.12 falls back to xipos when mass < mjMINVAL. The frozen Warp
    // kernel instead leaves its zero weighted moment unchanged. Preserve the
    // raw native reference and explicitly test this non-equivalent boundary.
    assert_eq!(native["subtree_com"], native["xipos"]);
    assert!(floats(native, "subtree_com").iter().any(|&v| v != 0.0));
    for (_, actual) in fields(out.world(world).unwrap()) {
        assert!(actual.iter().all(|&v| v == 0.0));
    }
    eprintln!("G01-COM frozen_zero_mass_world={world}");
}

#[test]
fn frozen_com_references_preserve_layout_tolerances_and_root_topology() {
    let mut types = [false; 4];
    for fixture in FIXTURES {
        let v: Value = serde_json::from_str(fixture).unwrap();
        let m = model(&v);
        assert_eq!(v["native_version"], 3012000);
        assert_eq!(v["seed"], 1789);
        assert_eq!(v["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(v["relative_tolerance"].as_f64(), Some(REL));
        for &ty in &m.kinematics().fields().jnt_type {
            types[ty as usize] = true;
        }
        let roots = v["model"]["body_rootid"].as_array().unwrap();
        for (b, expected) in roots.iter().enumerate() {
            let mut root = b;
            while m.kinematics().fields().body_parentid[root] != 0 {
                root = m.kinematics().fields().body_parentid[root] as usize;
            }
            assert_eq!(expected.as_u64().unwrap() as usize, root);
        }
        let cases = v["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 8);
        for (id, c) in cases.iter().enumerate() {
            assert_eq!(c["id"], id);
            assert_eq!(floats(c, "qpos").len(), m.kinematics().nq());
            for (key, len) in [
                ("subtree_mass", m.kinematics().nbody()),
                ("subtree_com", 3 * m.kinematics().nbody()),
                ("cinert", 10 * m.kinematics().nbody()),
                ("cdof", 6 * m.kinematics().nv()),
            ] {
                let values = c[key].as_array().unwrap();
                assert_eq!(values.len(), len);
                assert!(values.iter().all(|x| x.as_f64().unwrap().is_finite()));
            }
        }
    }
    assert_eq!(types, [true; 4]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn four_joint_com_fields_match_independent_native_references() {
    let session = TransferSession::new(0).unwrap();
    for fixture in &FIXTURES[..4] {
        let v: Value = serde_json::from_str(fixture).unwrap();
        let m = model(&v);
        let cases = v["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let out = probe_com_position(&session, &m, cases.len(), &q).unwrap();
        assert_eq!(out.worlds(), cases.len());
        assert_eq!(out.nbody(), m.kinematics().nbody());
        assert_eq!(out.nv(), m.kinematics().nv());
        assert!(out.world(out.worlds()).is_err());
        for (w, c) in cases.iter().enumerate() {
            compare(&out, w, c);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn com_batches_cross_blocks_without_state_mutation_or_cpu_transforms() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[2]).unwrap();
    let m = model(&v);
    let cases = v["cases"].as_array().unwrap();
    for worlds in [1, 255, 256, 257, 513] {
        let q: Vec<_> = (0..worlds)
            .flat_map(|w| floats(&cases[w % 8], "qpos"))
            .collect();
        let original = q.clone();
        let out = probe_com_position(&session, &m, worlds, &q).unwrap();
        assert_eq!(q, original);
        for w in 0..worlds {
            compare(&out, w, &cases[w % 8]);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn massless_static_subtrees_keep_frozen_warp_zero_com() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[4]).unwrap();
    let m = model(&v);
    assert!(m.fields().body_mass.iter().all(|&v| v == 0.0));
    let out = probe_com_position(&session, &m, 8, &[]).unwrap();
    // Owning readback remains valid after the session and all GPU buffers drop.
    drop(session);
    for (w, c) in v["cases"].as_array().unwrap().iter().enumerate() {
        compare_zero_mass(&out, w, c);
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn tiny_nonzero_mass_divides_without_native_minimum_mass_fallback() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[4]).unwrap();
    let m = model(&v);
    let mut i = m.fields().clone();
    i.body_mass[1] = 1e-20;
    i.body_mass[2] = 1e-20;
    let tiny = InertialModelInput::new(m.kinematics().clone(), i).unwrap();
    let out = probe_com_position(&session, &tiny, 1, &[]).unwrap();
    let w = out.world(0).unwrap();
    // Hand-derived from fixed xipos [2,4,6] and [2,0,7], equal tiny masses.
    // This is analytical evidence, not a fabricated native comparison.
    for (&a, e) in w
        .subtree_com
        .iter()
        .zip([2.0, 2.0, 6.5, 2.0, 2.0, 6.5, 2.0, 0.0, 7.0, 0.0, 0.0, 0.0])
    {
        assert!((a - e).abs() <= 2e-6);
    }
    assert_eq!(w.subtree_mass, &[2e-20, 2e-20, 1e-20, 0.0]);
    eprintln!("G01-COM analytical_tiny_mass_world=0");
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn com_rejects_overflow_without_partial_publication_and_ignores_dof_tuning() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[2]).unwrap();
    let m = model(&v);
    let q = floats(&v["cases"][0], "qpos");
    let mut bad_q = q.clone();
    bad_q[0] = f32::NAN;
    assert!(matches!(
        probe_com_position(&session, &m, 1, &bad_q),
        Err(TransferError::Input(InputError::NonFinite {
            field: "kinematics_qpos",
            ..
        }))
    ));
    let mut i = m.fields().clone();
    i.body_mass[1..].fill(f32::MAX);
    let overflow = InertialModelInput::new(m.kinematics().clone(), i).unwrap();
    assert!(matches!(
        probe_com_position(&session, &overflow, 1, &q),
        Err(TransferError::Input(InputError::NonFinite {
            field: "com_position_output",
            ..
        }))
    ));
    // Match frozen Warp arithmetic even for the zero-mass world body:
    // 0 * overflowing squared COM offset is nonfinite, not silently reset.
    let static_v: Value = serde_json::from_str(FIXTURES[4]).unwrap();
    let static_m = model(&static_v);
    let mut k = static_m.kinematics().fields().clone();
    let mut i = static_m.fields().clone();
    k.body_pos[3] = 3e20;
    k.body_pos[9] = 3e20;
    i.body_mass[1..].fill(1.0);
    let large_offset =
        InertialModelInput::new(mjwarp_rs::model::KinematicModelInput::new(0, k).unwrap(), i)
            .unwrap();
    assert!(matches!(
        probe_com_position(&session, &large_offset, 1, &[]),
        Err(TransferError::Input(InputError::NonFinite {
            field: "com_position_output",
            ..
        }))
    ));
    let out = probe_com_position(&session, &m, 1, &q).unwrap();
    compare(&out, 0, &v["cases"][0]);
    let mut i = m.fields().clone();
    i.dof_armature.fill(100.0);
    i.dof_damping.fill(50.0);
    let tuned = InertialModelInput::new(m.kinematics().clone(), i).unwrap();
    let unchanged = probe_com_position(&session, &tuned, 1, &q).unwrap();
    for ((key, a), (_, b)) in fields(out.world(0).unwrap())
        .into_iter()
        .zip(fields(unchanged.world(0).unwrap()))
    {
        assert_eq!(a, b, "{key}");
    }
}

#[cfg(feature = "native-model-probe")]
#[test]
#[ignore = "requires trusted Windows MuJoCo DLL, NVIDIA driver and NVRTC"]
fn native_model_snapshots_feed_com_without_native_physics_at_runtime() {
    use mjwarp_rs::io::NativeModelProbe;
    let dll = std::path::PathBuf::from(
        std::env::var_os("MJWARP_MUJOCO_DLL").expect("provide trusted DLL"),
    );
    let session = TransferSession::new(0).unwrap();
    let mjbs: [&[u8]; 5] = [
        include_bytes!("../fixtures/native-probe/mixed-joints.mjb"),
        include_bytes!("../fixtures/native-probe/inertial-tree.mjb"),
        include_bytes!("../fixtures/kinematics/rotated-tree.mjb"),
        include_bytes!("../fixtures/native-probe/zero-dof.mjb"),
        include_bytes!("../fixtures/com-position/massless-tree.mjb"),
    ];
    for (index, (fixture, bytes)) in FIXTURES.into_iter().zip(mjbs).enumerate() {
        let v: Value = serde_json::from_str(fixture).unwrap();
        // SAFETY: The caller supplies the hash-verified trusted candidate DLL;
        // each checked-in MJB uses that known compiled-model ABI.
        let native = unsafe { NativeModelProbe::load_trusted(&dll, bytes) }.unwrap();
        let m = native.inertial_snapshot().unwrap().into_model().unwrap();
        assert_eq!(m, model(&v));
        drop(native);
        let cases = v["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let out = probe_com_position(&session, &m, cases.len(), &q).unwrap();
        for (w, c) in cases.iter().enumerate() {
            if index == 4 {
                compare_zero_mass(&out, w, c);
            } else {
                compare(&out, w, c);
            }
        }
    }
}
