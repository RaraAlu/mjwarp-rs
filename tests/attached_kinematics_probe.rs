#![cfg(feature = "cuda-probe")]
//! G01几何与site子集参考测试。
//! 测试只读取静态原生结果。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{AttachedFields, AttachedModelInput, InertialModelInput, KinematicModelInput},
    physics::{AttachedKinematicsOutput, probe_attached_kinematics},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

const FIXTURES: [&str; 5] = [
    include_str!("../fixtures/attached-kinematics/attached-tree.json"),
    include_str!("../fixtures/attached-kinematics/static-frames.json"),
    include_str!("../fixtures/attached-kinematics/sites-only.json"),
    include_str!("../fixtures/attached-kinematics/geoms-only.json"),
    include_str!("../fixtures/attached-kinematics/empty-frames.json"),
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;

fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}

fn model(v: &Value) -> AttachedModelInput {
    let m = &v["model"];
    AttachedModelInput::new(
        reference::model(v),
        AttachedFields {
            geom_bodyid: ids(m, "geom_bodyid"),
            geom_pos: floats(m, "geom_pos"),
            geom_quat: floats(m, "geom_quat"),
            site_bodyid: ids(m, "site_bodyid"),
            site_pos: floats(m, "site_pos"),
            site_quat: floats(m, "site_quat"),
        },
    )
    .unwrap()
}

fn compare(out: &AttachedKinematicsOutput, world: usize, case: &Value) {
    let w = out.world(world).unwrap();
    let mut maximum: f64 = 0.0;
    for (key, actual) in [
        ("geom_xpos", w.geom_xpos),
        ("geom_xmat", w.geom_xmat),
        ("site_xpos", w.site_xpos),
        ("site_xmat", w.site_xmat),
    ] {
        let expected = case[key].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{key}");
        for (index, (&a, e)) in actual.iter().zip(expected).enumerate() {
            let e = e.as_f64().unwrap();
            let error = (f64::from(a) - e).abs();
            maximum = maximum.max(error);
            assert!(
                a.is_finite() && error <= ABS + REL * e.abs(),
                "G01 attached {key} world={world} index={index}: actual={a} expected={e} error={error}"
            );
        }
    }
    eprintln!(
        "G01-attached case={} world={world} max_abs_error={maximum:e}",
        case["id"]
    );
}

#[test]
fn frozen_references_preserve_hashes_shapes_and_attachment_branches() {
    let manifest: Value = serde_json::from_str(include_str!(
        "../fixtures/attached-kinematics/manifest.json"
    ))
    .unwrap();
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/attached-kinematics");
    for file in manifest["files"].as_array().unwrap() {
        let path = file["path"].as_str().unwrap();
        let bytes = std::fs::read(root.join(path)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            file["sha256"].as_str().unwrap(),
            "{path}"
        );
    }
    let mut types = [false; 4];
    let mut branches = [false; 4];
    for fixture in FIXTURES {
        let v: Value = serde_json::from_str(fixture).unwrap();
        assert_eq!(v["native_version"], 3012000);
        assert_eq!(v["seed"], 1789);
        assert_eq!(v["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(v["relative_tolerance"].as_f64(), Some(REL));
        let m = model(&v);
        assert_eq!(m.ngeom(), v["ngeom"].as_u64().unwrap() as usize);
        assert_eq!(m.nsite(), v["nsite"].as_u64().unwrap() as usize);
        for &ty in &m.rigid().kinematics().fields().jnt_type {
            types[ty as usize] = true;
        }
        branches[0] |= m.ngeom() == 0;
        branches[1] |= m.nsite() == 0;
        branches[2] |= m.rigid().kinematics().nq() == 0;
        branches[3] |= m.fields().geom_bodyid.contains(&0) && m.fields().site_bodyid.contains(&0);
        let cases = v["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 8);
        for (id, c) in cases.iter().enumerate() {
            assert_eq!(c["id"], id);
            assert_eq!(floats(c, "qpos").len(), m.rigid().kinematics().nq());
            for (key, width, count) in [
                ("geom_xpos", 3, m.ngeom()),
                ("geom_xmat", 9, m.ngeom()),
                ("site_xpos", 3, m.nsite()),
                ("site_xmat", 9, m.nsite()),
            ] {
                let values = c[key].as_array().unwrap();
                assert_eq!(values.len(), width * count);
                assert!(values.iter().all(|x| x.as_f64().unwrap().is_finite()));
            }
        }
    }
    assert_eq!(types, [true; 4]);
    assert_eq!(branches, [true; 4]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn attached_frames_match_independent_native_references() {
    let session = TransferSession::new(0).unwrap();
    for fixture in FIXTURES {
        let v: Value = serde_json::from_str(fixture).unwrap();
        let m = model(&v);
        let cases = v["cases"].as_array().unwrap();
        let qpos: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let out = probe_attached_kinematics(&session, &m, cases.len(), &qpos).unwrap();
        assert_eq!(
            (out.worlds(), out.ngeom(), out.nsite()),
            (cases.len(), m.ngeom(), m.nsite())
        );
        assert_eq!(out.rigid().nbody(), m.rigid().kinematics().nbody());
        assert!(out.world(out.worlds()).is_err());
        for (world, c) in cases.iter().enumerate() {
            compare(&out, world, c);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn batches_cross_blocks_without_mutating_inputs_or_aliasing_worlds() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[0]).unwrap();
    let m = model(&v);
    let original_model = m.clone();
    let cases = v["cases"].as_array().unwrap();
    for worlds in [1, 255, 256, 257, 513] {
        let qpos: Vec<_> = (0..worlds)
            .flat_map(|w| floats(&cases[w % 8], "qpos"))
            .collect();
        let original = qpos.clone();
        let out = probe_attached_kinematics(&session, &m, worlds, &qpos).unwrap();
        assert_eq!(qpos, original);
        assert_eq!(m, original_model);
        for world in 0..worlds {
            compare(&out, world, &cases[world % 8]);
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn analytic_world_and_offcenter_rotations_survive_owner_drop() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[2]).unwrap();
    let m = model(&v);
    let mut f = m.fields().clone();
    f.geom_bodyid = vec![0];
    f.geom_pos = vec![1.0, 2.0, 3.0];
    f.geom_quat = vec![0.0, 0.0, 0.0, 1.0];
    let m = AttachedModelInput::new(m.rigid().clone(), f).unwrap();
    let out = probe_attached_kinematics(&session, &m, 1, &[0.0, 0.0, 1.0, 0.0]).unwrap();
    drop(m);
    drop(session);
    let w = out.world(0).unwrap();
    assert_eq!(w.geom_xpos, &[1.0, 2.0, 3.0]);
    assert_eq!(
        w.geom_xmat,
        &[-1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0]
    );
    assert_eq!(w.site_xpos, &[0.5, -0.25, 1.0, 1.0, 0.25, -0.875]);
    assert_eq!(
        &w.site_xmat[..9],
        &[0.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0]
    );
    assert_eq!(
        &w.site_xmat[9..],
        &[1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0]
    );
    assert_eq!(out.rigid().world(0).unwrap().xpos.len(), 6);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn rejected_state_and_overflow_do_not_publish_partial_results() {
    let session = TransferSession::new(0).unwrap();
    let v: Value = serde_json::from_str(FIXTURES[0]).unwrap();
    let m = model(&v);
    let q = floats(&v["cases"][0], "qpos");
    for (worlds, state) in [(0, q.clone()), (1, vec![]), (2, q.clone())] {
        assert!(matches!(
            probe_attached_kinematics(&session, &m, worlds, &state),
            Err(TransferError::Input(_))
        ));
    }
    let mut bad = q.clone();
    bad[0] = f32::NAN;
    assert!(matches!(
        probe_attached_kinematics(&session, &m, 1, &bad),
        Err(TransferError::Input(InputError::NonFinite {
            field: "kinematics_qpos",
            ..
        }))
    ));
    let mut bad = q.clone();
    bad[3..7].fill(0.0);
    assert!(matches!(
        probe_attached_kinematics(&session, &m, 1, &bad),
        Err(TransferError::Input(InputError::InvalidTopology {
            reason: "unusable_state_quaternion",
            ..
        }))
    ));
    let s: Value = serde_json::from_str(FIXTURES[1]).unwrap();
    let static_model = model(&s);
    let mut k = static_model.rigid().kinematics().fields().clone();
    k.body_pos[3] = f32::MAX;
    k.body_quat[4..8].copy_from_slice(&[1.0, 0.0, 0.0, 0.0]);
    let rigid = InertialModelInput::new(
        KinematicModelInput::new(0, k).unwrap(),
        static_model.rigid().fields().clone(),
    )
    .unwrap();
    let mut f = static_model.fields().clone();
    f.geom_pos[3] = f32::MAX;
    let overflowing = AttachedModelInput::new(rigid, f).unwrap();
    assert!(matches!(
        probe_attached_kinematics(&session, &overflowing, 2, &[]),
        Err(TransferError::Input(InputError::NonFinite {
            field: "attached_kinematics_output",
            ..
        }))
    ));
    // A healthy first world must not leak through a failed second world.
    let mut f = m.fields().clone();
    f.geom_pos[3..6].fill(f32::MAX);
    let large = AttachedModelInput::new(m.rigid().clone(), f).unwrap();
    let mut safe_q = q.clone();
    safe_q[..3].fill(0.0);
    safe_q[3..7].copy_from_slice(&[1.0, 0.0, 0.0, 0.0]);
    let safe = probe_attached_kinematics(&session, &large, 1, &safe_q).unwrap();
    assert_eq!(&safe.world(0).unwrap().geom_xpos[3..6], &[f32::MAX; 3]);
    let mut bad_q = safe_q.clone();
    bad_q[3..7].fill(0.5);
    let stride = 12 * (large.ngeom() + large.nsite());
    assert!(matches!(
        probe_attached_kinematics(&session, &large, 2, &[safe_q, bad_q].concat()),
        Err(TransferError::Input(InputError::NonFinite {
            field: "attached_kinematics_output",
            index,
        })) if index >= stride
    ));
    let out = probe_attached_kinematics(&session, &m, 1, &q).unwrap();
    compare(&out, 0, &v["cases"][0]);
}
