#![cfg(feature = "cuda-probe")]
//! RNE偏置力子集静态参考与阶段边界。
//! 产品测试不运行参考生成器。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{AttachedFields, AttachedModelInput},
    physics::{RneBiasPlan, RneBiasSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

const FIXTURES: [&str; 5] = [
    include_str!("../fixtures/rne-bias/mixed-joints.json"),
    include_str!("../fixtures/rne-bias/inertial-tree.json"),
    include_str!("../fixtures/rne-bias/rotated-tree.json"),
    include_str!("../fixtures/rne-bias/zero-dof.json"),
    include_str!("../fixtures/rne-bias/massless-tree.json"),
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;

fn fixture(i: usize) -> Value {
    serde_json::from_str(FIXTURES[i]).unwrap()
}
fn model(v: &Value) -> AttachedModelInput {
    AttachedModelInput::new(reference::model(v), AttachedFields::default()).unwrap()
}
fn gravity(v: &Value, g: usize) -> [f32; 3] {
    v["gravities"][g]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
fn compare(s: &RneBiasSnapshot, w: usize, case: &Value, g: usize) -> (usize, f64) {
    let a = s.world(w).unwrap().qfrc_bias;
    let expected = case["qfrc_bias"][g].as_array().unwrap();
    assert_eq!(a.len(), expected.len());
    let mut maximum = 0.0f64;
    for (i, (&a, e)) in a.iter().zip(expected).enumerate() {
        let e = e.as_f64().unwrap();
        let error = (f64::from(a) - e).abs();
        maximum = maximum.max(error);
        assert!(
            a.is_finite() && error <= ABS + REL * e.abs(),
            "G02-RNE world={w} dof={i} gravity={g} actual={a} expected={e} error={error}"
        );
    }
    (a.len(), maximum)
}
fn same(a: &RneBiasSnapshot, b: &RneBiasSnapshot) {
    assert_eq!(a.worlds(), b.worlds());
    for w in 0..a.worlds() {
        assert_eq!(a.world(w).unwrap().qfrc_bias, b.world(w).unwrap().qfrc_bias);
    }
}

#[test]
fn reference_hashes_freeze_native_bias_inputs_gravities_and_tolerances() {
    let m: Value =
        serde_json::from_str(include_str!("../fixtures/rne-bias/manifest.json")).unwrap();
    assert_eq!(m["files"].as_array().unwrap().len(), 11);
    assert_eq!(m["candidate_version"], "3.12.0");
    assert_eq!(
        m["frozen_warp_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    assert_eq!(m["absolute_tolerance"].as_f64(), Some(ABS));
    assert_eq!(m["relative_tolerance"].as_f64(), Some(REL));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for entry in m["files"].as_array().unwrap() {
        let bytes = std::fs::read(root.join(entry["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            entry["sha256"].as_str().unwrap()
        );
    }
    let mut joint_types = [false; 4];
    for text in FIXTURES {
        let v: Value = serde_json::from_str(text).unwrap();
        let m = model(&v);
        let k = m.rigid().kinematics();
        assert_eq!(v["native_version"], 3012000);
        assert_eq!(v["seed"], 1789);
        assert_eq!(v["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(v["relative_tolerance"].as_f64(), Some(REL));
        assert_eq!(v["gravities"].as_array().unwrap().len(), 3);
        assert_eq!(gravity(&v, 1), [0.0; 3]);
        for &t in &k.fields().jnt_type {
            joint_types[t as usize] = true;
        }
        assert_eq!(v["cases"].as_array().unwrap().len(), 8);
        for c in v["cases"].as_array().unwrap() {
            assert_eq!(floats(c, "qpos").len(), k.nq());
            assert_eq!(floats(c, "qvel").len(), k.nv());
            assert_eq!(c["qfrc_bias"].as_array().unwrap().len(), 3);
            for g in 0..3 {
                let f = c["qfrc_bias"][g].as_array().unwrap();
                assert_eq!(f.len(), k.nv());
                assert!(f.iter().all(|v| v.as_f64().unwrap().is_finite()));
            }
        }
    }
    assert_eq!(joint_types, [true; 4]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn bias_matches_native_gravity_coriolis_and_centrifugal_references() {
    let s = TransferSession::new(0).unwrap();
    let mut count = 0;
    let mut maximum = 0.0f64;
    for text in FIXTURES {
        let v: Value = serde_json::from_str(text).unwrap();
        let cases = v["cases"].as_array().unwrap();
        for g in 0..3 {
            let p = RneBiasPlan::new(&s, model(&v), gravity(&v, g)).unwrap();
            let mut d = p.create_data(cases.len()).unwrap();
            let qpos: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
            let qvel: Vec<_> = cases.iter().flat_map(|c| floats(c, "qvel")).collect();
            d.write_qpos(&qpos).unwrap();
            d.write_qvel(&qvel).unwrap();
            p.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for (w, c) in cases.iter().enumerate() {
                let (n, e) = compare(&out, w, c, g);
                count += n;
                maximum = maximum.max(e);
            }
            assert_eq!(
                qpos,
                cases
                    .iter()
                    .flat_map(|c| floats(c, "qpos"))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                qvel,
                cases
                    .iter()
                    .flat_map(|c| floats(c, "qvel"))
                    .collect::<Vec<_>>()
            );
        }
    }
    eprintln!("G02-RNE states=120 scalars={count} max_abs_error={maximum:e}");
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn bias_world_writes_cross_blocks_and_keep_snapshots_owned() {
    let v = fixture(0);
    let cases = v["cases"].as_array().unwrap();
    let s = TransferSession::new(0).unwrap();
    let p = RneBiasPlan::new(&s, model(&v), gravity(&v, 2)).unwrap();
    let mut d = p.create_data(513).unwrap();
    d.write_qpos(
        &(0..513)
            .flat_map(|w| floats(&cases[w % 8], "qpos"))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    d.write_qvel(
        &(0..513)
            .flat_map(|w| floats(&cases[w % 8], "qvel"))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    p.update(&mut d).unwrap();
    let before = d.readback().unwrap();
    let mut count = 0;
    let mut maximum = 0.0f64;
    for w in 0..513 {
        let (n, e) = compare(&before, w, &cases[w % 8], 2);
        count += n;
        maximum = maximum.max(e);
    }
    d.write_world_qpos(256, &floats(&cases[7], "qpos")).unwrap();
    d.write_world_qvel(256, &floats(&cases[7], "qvel")).unwrap();
    assert!(matches!(
        p.update_bias(&mut d),
        Err(TransferError::StageNotReady { .. })
    ));
    p.update_velocity(&mut d).unwrap();
    p.update_bias(&mut d).unwrap();
    let after = d.readback().unwrap();
    for w in 0..513 {
        compare(&after, w, &cases[if w == 256 { 7 } else { w % 8 }], 2);
    }
    compare(&before, 256, &cases[0], 2);
    eprintln!("G02-RNE cross_block_states=513 scalars={count} max_abs_error={maximum:e}");
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn invalid_bias_inputs_gravity_and_model_identity_preserve_ready_results() {
    let v = fixture(0);
    let s = TransferSession::new(0).unwrap();
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            RneBiasPlan::new(&s, model(&v), [x, 0.0, 0.0]),
            Err(TransferError::Input(InputError::NonFinite { .. }))
        ));
    }
    let p = RneBiasPlan::new(&s, model(&v), gravity(&v, 0)).unwrap();
    let wrong = RneBiasPlan::new(&s, model(&v), gravity(&v, 0)).unwrap();
    let mut d = p.create_data(1).unwrap();
    p.update(&mut d).unwrap();
    let before = d.readback().unwrap();
    compare(&before, 0, &v["cases"][0], 0);
    for result in [
        wrong.update(&mut d),
        wrong.update_velocity(&mut d),
        wrong.update_bias(&mut d),
    ] {
        assert!(matches!(result, Err(TransferError::ModelMismatch)));
    }
    same(&d.readback().unwrap(), &before);
    assert!(d.write_qpos(&[]).is_err());
    assert!(d.write_qvel(&[]).is_err());
    assert!(
        d.write_world_qpos(1, &floats(&v["cases"][0], "qpos"))
            .is_err()
    );
    assert!(
        d.write_world_qvel(1, &floats(&v["cases"][0], "qvel"))
            .is_err()
    );
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut q = floats(&v["cases"][0], "qvel");
        q[0] = x;
        assert!(d.write_qvel(&q).is_err());
        let mut q = floats(&v["cases"][0], "qpos");
        q[0] = x;
        assert!(d.write_qpos(&q).is_err());
        same(&d.readback().unwrap(), &before);
    }
    assert!(p.create_data(0).is_err());
    assert!(p.create_data(usize::MAX).is_err());
    assert!(before.world(1).is_err());
    same(&d.readback().unwrap(), &before);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn bias_only_stage_requires_prepared_velocity_and_never_refreshes_it() {
    let v = fixture(1);
    let s = TransferSession::new(0).unwrap();
    let p = RneBiasPlan::new(&s, model(&v), gravity(&v, 0)).unwrap();
    let mut d = p.create_data(1).unwrap();
    assert!(matches!(
        d.readback(),
        Err(TransferError::StageNotReady { .. })
    ));
    assert!(matches!(
        p.update_bias(&mut d),
        Err(TransferError::StageNotReady {
            stage: "com_velocity"
        })
    ));
    p.update_velocity(&mut d).unwrap();
    assert!(d.readback().is_err());
    p.update_bias(&mut d).unwrap();
    let before = d.readback().unwrap();
    compare(&before, 0, &v["cases"][0], 0);
    p.update_bias(&mut d).unwrap();
    same(&d.readback().unwrap(), &before);
    d.write_qvel(&floats(&v["cases"][5], "qvel")).unwrap();
    assert!(p.update_bias(&mut d).is_err());
    assert!(d.readback().is_err());
    d.write_qpos(&floats(&v["cases"][5], "qpos")).unwrap();
    p.update_velocity(&mut d).unwrap();
    p.update_bias(&mut d).unwrap();
    let split = d.readback().unwrap();
    compare(&split, 0, &v["cases"][5], 0);
    p.update(&mut d).unwrap();
    same(&d.readback().unwrap(), &split);
    d.write_qpos(&floats(&v["cases"][5], "qpos")).unwrap();
    assert!(p.update_bias(&mut d).is_err());
    compare(&before, 0, &v["cases"][0], 0);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn independent_bias_data_and_snapshots_outlive_the_plan() {
    let v = fixture(2);
    let mut d;
    let second;
    {
        let s = TransferSession::new(0).unwrap();
        let p = RneBiasPlan::new(&s, model(&v), gravity(&v, 2)).unwrap();
        d = p.create_data(1).unwrap();
        let mut other = p.create_data(1).unwrap();
        d.write_qpos(&floats(&v["cases"][3], "qpos")).unwrap();
        d.write_qvel(&floats(&v["cases"][3], "qvel")).unwrap();
        other.write_qpos(&floats(&v["cases"][6], "qpos")).unwrap();
        other.write_qvel(&floats(&v["cases"][6], "qvel")).unwrap();
        p.update(&mut d).unwrap();
        p.update(&mut other).unwrap();
        second = other.readback().unwrap();
    }
    compare(&d.readback().unwrap(), 0, &v["cases"][3], 2);
    compare(&second, 0, &v["cases"][6], 2);
    d.write_qvel(&vec![0.0; second.nv()]).unwrap();
    assert!(d.readback().is_err());
    compare(&second, 0, &v["cases"][6], 2);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn bias_overflow_rejects_results_and_recovers_without_session_poisoning() {
    let v = fixture(1);
    let s = TransferSession::new(0).unwrap();
    let p = RneBiasPlan::new(&s, model(&v), gravity(&v, 0)).unwrap();
    let mut d = p.create_data(1).unwrap();
    d.write_qvel(&vec![f32::MAX; v["nv"].as_u64().unwrap() as usize])
        .unwrap();
    p.update(&mut d).unwrap();
    assert!(matches!(
        d.readback(),
        Err(TransferError::Input(InputError::NonFinite { .. }))
    ));
    d.write_qvel(&vec![0.0; v["nv"].as_u64().unwrap() as usize])
        .unwrap();
    p.update(&mut d).unwrap();
    compare(&d.readback().unwrap(), 0, &v["cases"][0], 0);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn zero_dof_bias_worlds_keep_empty_outputs_with_finite_gravity_workspaces() {
    let s = TransferSession::new(0).unwrap();
    for i in [3, 4] {
        let v = fixture(i);
        for g in 0..3 {
            let p = RneBiasPlan::new(&s, model(&v), gravity(&v, g)).unwrap();
            let mut d = p.create_data(257).unwrap();
            d.write_qvel(&[]).unwrap();
            d.write_world_qvel(256, &[]).unwrap();
            assert!(d.write_world_qvel(257, &[]).is_err());
            p.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            assert_eq!(out.worlds(), 257);
            for w in 0..257 {
                assert!(out.world(w).unwrap().qfrc_bias.is_empty());
            }
        }
    }
}
