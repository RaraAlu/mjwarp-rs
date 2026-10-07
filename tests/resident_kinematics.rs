#![cfg(feature = "cuda-probe")]
//! G01常驻子集的独立参考与生命周期测试。
//! 产品测试不生成参考或启动Python。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{AttachedFields, AttachedModelInput},
    physics::{KinematicsPlan, KinematicsSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

const RIGID: [&str; 4] = [
    include_str!("../fixtures/kinematics/mixed-joints.json"),
    include_str!("../fixtures/kinematics/rotated-tree.json"),
    include_str!("../fixtures/kinematics/zero-dof.json"),
    include_str!("../fixtures/kinematics/inertial-tree.json"),
];
const COM: [&str; 5] = [
    include_str!("../fixtures/com-position/mixed-joints.json"),
    include_str!("../fixtures/com-position/inertial-tree.json"),
    include_str!("../fixtures/com-position/rotated-tree.json"),
    include_str!("../fixtures/com-position/zero-dof.json"),
    include_str!("../fixtures/com-position/massless-tree.json"),
];
const ATTACHED: [&str; 5] = [
    include_str!("../fixtures/attached-kinematics/attached-tree.json"),
    include_str!("../fixtures/attached-kinematics/static-frames.json"),
    include_str!("../fixtures/attached-kinematics/sites-only.json"),
    include_str!("../fixtures/attached-kinematics/geoms-only.json"),
    include_str!("../fixtures/attached-kinematics/empty-frames.json"),
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;

fn model(v: &Value) -> AttachedModelInput {
    let m = &v["model"];
    let fields = if m.get("geom_bodyid").is_some() {
        let ids = |key: &str| {
            m[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
                .collect()
        };
        AttachedFields {
            geom_bodyid: ids("geom_bodyid"),
            geom_pos: floats(m, "geom_pos"),
            geom_quat: floats(m, "geom_quat"),
            site_bodyid: ids("site_bodyid"),
            site_pos: floats(m, "site_pos"),
            site_quat: floats(m, "site_quat"),
        }
    } else {
        AttachedFields::default()
    };
    AttachedModelInput::new(reference::model(v), fields).unwrap()
}

fn compare(out: &KinematicsSnapshot, world: usize, case: &Value, kind: &str, zero_mass: bool) {
    let rigid = out.rigid().world(world).unwrap();
    let com = out.com().world(world).unwrap();
    let attached = out.attached().world(world).unwrap();
    let fields: Vec<_> = match kind {
        "rigid" => vec![
            ("xpos", rigid.xpos),
            ("xquat", rigid.xquat),
            ("xmat", rigid.xmat),
            ("xipos", rigid.xipos),
            ("ximat", rigid.ximat),
            ("xanchor", rigid.xanchor),
            ("xaxis", rigid.xaxis),
        ],
        "com" => vec![
            ("subtree_mass", com.subtree_mass),
            ("subtree_com", com.subtree_com),
            ("cinert", com.cinert),
            ("cdof", com.cdof),
        ],
        "attached" => vec![
            ("geom_xpos", attached.geom_xpos),
            ("geom_xmat", attached.geom_xmat),
            ("site_xpos", attached.site_xpos),
            ("site_xmat", attached.site_xmat),
        ],
        _ => unreachable!(),
    };
    let mut maximum: f64 = 0.0;
    for (key, actual) in fields {
        let expected = case[key].as_array().unwrap();
        assert_eq!(actual.len(), expected.len());
        for (index, (&a, e)) in actual.iter().zip(expected).enumerate() {
            // Frozen Warp leaves zero-mass COM at zero. Native 3.12 instead
            // falls back to xipos; do not use this CPU boundary as equivalence.
            let e = if zero_mass { 0.0 } else { e.as_f64().unwrap() };
            let error = (f64::from(a) - e).abs();
            maximum = maximum.max(error);
            assert!(
                a.is_finite() && error <= ABS + REL * e.abs(),
                "G01 resident {kind} {key} world={world} index={index}: actual={a} expected={e} error={error}"
            );
        }
    }
    eprintln!("G01-resident kind={kind} world={world} max_abs_error={maximum:e}");
}

#[test]
fn reference_manifests_preserve_hashes_and_tolerances() {
    for directory in ["kinematics", "com-position", "attached-kinematics"] {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(directory);
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
        // COM清单使用仓库相对路径。
        let base = if directory == "com-position" {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        } else {
            &root
        };
        for file in manifest["files"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            assert_eq!(
                format!(
                    "{:x}",
                    Sha256::digest(std::fs::read(base.join(path)).unwrap())
                ),
                file["sha256"].as_str().unwrap(),
                "{directory}/{path}"
            );
        }
    }
    let mut types = [false; 4];
    for fixture in RIGID.into_iter().chain(COM).chain(ATTACHED) {
        let v: Value = serde_json::from_str(fixture).unwrap();
        assert_eq!(v["native_version"], 3012000);
        assert_eq!(v["seed"], 1789);
        assert_eq!(v["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(v["relative_tolerance"].as_f64(), Some(REL));
        let m = model(&v);
        for &ty in &m.rigid().kinematics().fields().jnt_type {
            types[ty as usize] = true;
        }
        for c in v["cases"].as_array().unwrap() {
            assert_eq!(floats(c, "qpos").len(), m.rigid().kinematics().nq());
        }
    }
    assert_eq!(types, [true; 4]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn repeated_updates_match_independent_native_references() {
    let session = TransferSession::new(0).unwrap();
    for (kind, fixtures) in [
        ("rigid", &RIGID[..]),
        ("com", &COM[..]),
        ("attached", &ATTACHED[..]),
    ] {
        for fixture in fixtures {
            let v: Value = serde_json::from_str(fixture).unwrap();
            let m = model(&v);
            let zero_mass = kind == "com" && m.rigid().fields().body_mass.iter().all(|&x| x == 0.0);
            let cases = v["cases"].as_array().unwrap();
            let plan = KinematicsPlan::new(&session, m).unwrap();
            let mut data = plan.create_data(cases.len()).unwrap();
            plan.update(&mut data).unwrap();
            let initial = data.readback().unwrap();
            for world in 0..cases.len() {
                compare(&initial, world, &cases[0], kind, zero_mass);
            }
            for round in 0..3 {
                let qpos: Vec<_> = (0..cases.len())
                    .flat_map(|w| floats(&cases[(w + round) % cases.len()], "qpos"))
                    .collect();
                data.write_qpos(&qpos).unwrap();
                plan.update(&mut data).unwrap();
                let out = data.readback().unwrap();
                for world in 0..cases.len() {
                    compare(
                        &out,
                        world,
                        &cases[(world + round) % cases.len()],
                        kind,
                        zero_mass,
                    );
                }
            }
            // Completed host snapshots never borrow the resident state buffers.
            compare(&initial, 0, &cases[0], kind, zero_mass);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn partial_world_writes_and_multiple_data_sets_remain_isolated() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(ATTACHED[0]).unwrap();
    let cases = v["cases"].as_array().unwrap();
    let plan = KinematicsPlan::new(&session, model(&v)).unwrap();
    assert_eq!(plan.device(), 0);
    let mut other = plan.create_data(1).unwrap();
    plan.update(&mut other).unwrap();
    for worlds in [1, 257, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        plan.update(&mut data).unwrap();
        let initial = data.readback().unwrap();
        data.write_world_qpos(worlds - 1, &floats(&cases[1], "qpos"))
            .unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        assert_eq!(data.worlds(), worlds);
        for world in 0..worlds {
            compare(
                &out,
                world,
                &cases[usize::from(world == worlds - 1)],
                "attached",
                false,
            );
        }
        compare(&initial, worlds - 1, &cases[0], "attached", false);
        compare(&other.readback().unwrap(), 0, &cases[0], "attached", false);
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn stage_readiness_and_input_errors_preserve_completed_results() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(ATTACHED[0]).unwrap();
    let plan = KinematicsPlan::new(&session, model(&v)).unwrap();
    assert!(matches!(
        plan.create_data(0),
        Err(TransferError::Input(InputError::InvalidDimension { .. }))
    ));
    let mut data = plan.create_data(2).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rigid" })
    ));
    assert_eq!(
        plan.update_com(&mut data),
        Err(TransferError::StageNotReady { stage: "rigid" })
    );
    assert_eq!(
        plan.update_attached(&mut data),
        Err(TransferError::StageNotReady { stage: "rigid" })
    );
    plan.update_rigid(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "attached" })
    ));
    plan.update_attached(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "com" })
    ));
    plan.update_com(&mut data).unwrap();
    let cases = v["cases"].as_array().unwrap();
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut qpos = floats(&cases[0], "qpos");
        qpos[0] = bad;
        assert!(matches!(
            data.write_world_qpos(0, &qpos),
            Err(TransferError::Input(InputError::NonFinite { .. }))
        ));
    }
    let mut bad = floats(&cases[0], "qpos");
    bad[3..7].fill(0.0);
    assert!(matches!(
        data.write_world_qpos(0, &bad),
        Err(TransferError::Input(InputError::InvalidTopology { .. }))
    ));
    assert!(matches!(
        data.write_qpos(&[]),
        Err(TransferError::Input(InputError::LengthMismatch { .. }))
    ));
    assert!(matches!(
        data.write_world_qpos(2, &floats(&cases[0], "qpos")),
        Err(TransferError::Input(InputError::InvalidIndex { .. }))
    ));
    compare(&data.readback().unwrap(), 0, &cases[0], "attached", false);
    data.write_world_qpos(0, &floats(&cases[1], "qpos"))
        .unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rigid" })
    ));
    plan.update(&mut data).unwrap();
    plan.update_rigid(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "attached" })
    ));
    plan.update_com(&mut data).unwrap();
    plan.update_attached(&mut data).unwrap();
    compare(&data.readback().unwrap(), 0, &cases[1], "attached", false);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn identical_models_and_other_sessions_cannot_exchange_state() {
    let session = TransferSession::new(0).unwrap();
    let other_session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(ATTACHED[0]).unwrap();
    let m = model(&v);
    let plan = KinematicsPlan::new(&session, m.clone()).unwrap();
    let same_shape = KinematicsPlan::new(&session, m.clone()).unwrap();
    let other_context = KinematicsPlan::new(&other_session, m).unwrap();
    let mut data = plan.create_data(1).unwrap();
    plan.update(&mut data).unwrap();
    for wrong in [&same_shape, &other_context] {
        assert_eq!(wrong.update(&mut data), Err(TransferError::ModelMismatch));
        assert_eq!(
            wrong.update_com(&mut data),
            Err(TransferError::ModelMismatch)
        );
        assert_eq!(
            wrong.update_attached(&mut data),
            Err(TransferError::ModelMismatch)
        );
        compare(
            &data.readback().unwrap(),
            0,
            &v["cases"][0],
            "attached",
            false,
        );
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn completed_buffers_outlive_session_and_plan_handles() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(ATTACHED[0]).unwrap();
    let plan = KinematicsPlan::new(&session, model(&v)).unwrap();
    let mut data = plan.create_data(1).unwrap();
    drop(session);
    plan.update(&mut data).unwrap();
    drop(plan);
    compare(
        &data.readback().unwrap(),
        0,
        &v["cases"][0],
        "attached",
        false,
    );
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn nonfinite_outputs_fail_at_readback_and_allow_healthy_reuse() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(ATTACHED[0]).unwrap();
    let plan = KinematicsPlan::new(&session, model(&v)).unwrap();
    let mut data = plan.create_data(1).unwrap();
    let healthy = floats(&v["cases"][0], "qpos");
    let mut extreme = healthy.clone();
    extreme[0] = f32::MAX;
    data.write_qpos(&extreme).unwrap();
    plan.update(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::Input(InputError::NonFinite { .. }))
    ));
    data.write_qpos(&healthy).unwrap();
    plan.update(&mut data).unwrap();
    compare(
        &data.readback().unwrap(),
        0,
        &v["cases"][0],
        "attached",
        false,
    );
}
