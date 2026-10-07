#![cfg(feature = "cuda-probe")]
//! G01刚体子集参考测试。
//! 测试只读取静态原生结果。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput},
    physics::{KinematicsOutput, KinematicsWorld, probe_kinematics},
    runtime::TransferSession,
};
use serde_json::Value;

const FIXTURES: [&str; 4] = [
    include_str!("../fixtures/kinematics/mixed-joints.json"),
    include_str!("../fixtures/kinematics/inertial-tree.json"),
    include_str!("../fixtures/kinematics/rotated-tree.json"),
    include_str!("../fixtures/kinematics/zero-dof.json"),
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;

fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect()
}
fn integers(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}
fn model(v: &Value) -> InertialModelInput {
    let m = &v["model"];
    let k = KinematicModelInput::new(
        v["nv"].as_u64().unwrap() as usize,
        KinematicFields {
            qpos0: floats(m, "qpos0"),
            body_parentid: integers(m, "body_parentid"),
            body_jntadr: integers(m, "body_jntadr"),
            body_jntnum: integers(m, "body_jntnum"),
            body_pos: floats(m, "body_pos"),
            body_quat: floats(m, "body_quat"),
            jnt_type: integers(m, "jnt_type"),
            jnt_bodyid: integers(m, "jnt_bodyid"),
            jnt_qposadr: integers(m, "jnt_qposadr"),
            jnt_dofadr: integers(m, "jnt_dofadr"),
            jnt_pos: floats(m, "jnt_pos"),
            jnt_axis: floats(m, "jnt_axis"),
        },
    )
    .unwrap();
    assert_eq!(k.nq(), v["nq"].as_u64().unwrap() as usize);
    assert_eq!(k.nbody(), v["nbody"].as_u64().unwrap() as usize);
    assert_eq!(k.njnt(), v["njnt"].as_u64().unwrap() as usize);
    InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: floats(m, "body_ipos"),
            body_iquat: floats(m, "body_iquat"),
            body_mass: floats(m, "body_mass"),
            body_inertia: floats(m, "body_inertia"),
            dof_bodyid: integers(m, "dof_bodyid"),
            dof_jntid: integers(m, "dof_jntid"),
            dof_parentid: integers(m, "dof_parentid"),
            dof_armature: floats(m, "dof_armature"),
            dof_damping: floats(m, "dof_damping"),
        },
    )
    .unwrap()
}
fn fields(w: KinematicsWorld<'_>) -> [(&'static str, &[f32]); 7] {
    [
        ("xpos", w.xpos),
        ("xquat", w.xquat),
        ("xmat", w.xmat),
        ("xipos", w.xipos),
        ("ximat", w.ximat),
        ("xanchor", w.xanchor),
        ("xaxis", w.xaxis),
    ]
}
fn compare(out: &KinematicsOutput, world: usize, case: &Value) {
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
                "G01 subset {key} world={world} index={index}: actual={a} expected={e} error={error}"
            );
        }
    }
    eprintln!(
        "G01-subset case={} world={world} max_abs_error={maximum:e}",
        case["id"]
    );
}

#[test]
fn frozen_native_references_preserve_shapes_and_four_joint_coverage() {
    let mut types = [false; 4];
    for fixture in FIXTURES {
        let v: Value = serde_json::from_str(fixture).unwrap();
        assert_eq!(v["native_version"], 3012000);
        assert_eq!(v["seed"], 1789);
        assert_eq!(v["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(v["relative_tolerance"].as_f64(), Some(REL));
        let m = model(&v);
        for &ty in &m.kinematics().fields().jnt_type {
            types[ty as usize] = true;
        }
        let cases = v["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 8);
        for (id, c) in cases.iter().enumerate() {
            assert_eq!(c["id"], id);
            assert_eq!(floats(c, "qpos").len(), m.kinematics().nq());
            for (key, width, count) in [
                ("xpos", 3, m.kinematics().nbody()),
                ("xquat", 4, m.kinematics().nbody()),
                ("xmat", 9, m.kinematics().nbody()),
                ("xipos", 3, m.kinematics().nbody()),
                ("ximat", 9, m.kinematics().nbody()),
                ("xanchor", 3, m.kinematics().njnt()),
                ("xaxis", 3, m.kinematics().njnt()),
            ] {
                let values = c[key].as_array().unwrap();
                assert_eq!(values.len(), width * count);
                assert!(values.iter().all(|x| x.as_f64().unwrap().is_finite()));
            }
        }
    }
    assert_eq!(types, [true; 4]);
}

#[cfg(feature = "native-model-probe")]
#[test]
#[ignore = "requires trusted Windows MuJoCo DLL, NVIDIA driver and NVRTC"]
fn compiled_native_models_feed_the_gpu_without_native_kinematics_at_runtime() {
    use mjwarp_rs::io::NativeModelProbe;
    let dll = std::path::PathBuf::from(
        std::env::var_os("MJWARP_MUJOCO_DLL").expect("provide trusted DLL"),
    );
    let session = TransferSession::new(0).unwrap();
    let mjbs: [&[u8]; 4] = [
        include_bytes!("../fixtures/native-probe/mixed-joints.mjb"),
        include_bytes!("../fixtures/native-probe/inertial-tree.mjb"),
        include_bytes!("../fixtures/kinematics/rotated-tree.mjb"),
        include_bytes!("../fixtures/native-probe/zero-dof.mjb"),
    ];
    for (fixture, bytes) in FIXTURES.into_iter().zip(mjbs) {
        let v: Value = serde_json::from_str(fixture).unwrap();
        // SAFETY: The caller supplies the trusted, hash-verified candidate DLL.
        // The checked-in MJB uses its known compiled-model ABI.
        let native = unsafe { NativeModelProbe::load_trusted(&dll, bytes) }.unwrap();
        let m = native.inertial_snapshot().unwrap().into_model().unwrap();
        assert_eq!(m, model(&v));
        drop(native);
        let cases = v["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let out = probe_kinematics(&session, &m, cases.len(), &q).unwrap();
        for (world, c) in cases.iter().enumerate() {
            compare(&out, world, c);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn four_joint_gpu_outputs_match_independent_native_references() {
    let session = TransferSession::new(0).unwrap();
    for fixture in FIXTURES {
        let v: Value = serde_json::from_str(fixture).unwrap();
        let m = model(&v);
        let cases = v["cases"].as_array().unwrap();
        let qpos: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let out = probe_kinematics(&session, &m, cases.len(), &qpos).unwrap();
        assert_eq!(out.worlds(), cases.len());
        assert_eq!(out.nbody(), m.kinematics().nbody());
        assert_eq!(out.njnt(), m.kinematics().njnt());
        assert!(out.world(out.worlds()).is_err());
        // Readback owns its values after all device temporaries have dropped.
        for (world, c) in cases.iter().enumerate() {
            compare(&out, world, c);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn batches_cross_blocks_and_repeated_states_remain_independent() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[2]).unwrap();
    let m = model(&v);
    let cases = v["cases"].as_array().unwrap();
    for worlds in [1, 255, 256, 257, 513] {
        let qpos: Vec<_> = (0..worlds)
            .flat_map(|w| floats(&cases[w % 8], "qpos"))
            .collect();
        let original = qpos.clone();
        let out = probe_kinematics(&session, &m, worlds, &qpos).unwrap();
        assert_eq!(qpos, original);
        for w in 0..worlds {
            compare(&out, w, &cases[w % 8]);
        }
    }
    drop(session);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn rejected_state_and_nonfinite_output_do_not_publish_partial_results() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[2]).unwrap();
    let m = model(&v);
    let mut q = floats(&v["cases"][0], "qpos");
    q[0] = f32::NAN;
    assert!(matches!(
        probe_kinematics(&session, &m, 1, &q),
        Err(TransferError::Input(InputError::NonFinite {
            field: "kinematics_qpos",
            ..
        }))
    ));
    let mut k = m.kinematics().fields().clone();
    let mut i = m.fields().clone();
    // Finite input can overflow a transform. The successful kernel/readback
    // still must reject its output, without publishing a partially valid batch.
    k.body_pos[6] = f32::MAX;
    k.body_quat[8..12].copy_from_slice(&[1.0, 0.0, 0.0, 0.0]);
    i.body_ipos[6] = f32::MAX;
    let overflowing =
        InertialModelInput::new(KinematicModelInput::new(m.kinematics().nv(), k).unwrap(), i)
            .unwrap();
    let mut q = floats(&v["cases"][0], "qpos");
    q[3..7].copy_from_slice(&[1.0, 0.0, 0.0, 0.0]);
    assert!(matches!(
        probe_kinematics(&session, &overflowing, 1, &q),
        Err(TransferError::Input(InputError::NonFinite {
            field: "kinematics_output",
            ..
        }))
    ));
    let out = probe_kinematics(&session, &m, 1, &floats(&v["cases"][0], "qpos")).unwrap();
    compare(&out, 0, &v["cases"][0]);
}
