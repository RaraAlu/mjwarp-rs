#![cfg(feature = "cuda-probe")]
//! G01的mocap与静态几何子集。
//! 产品测试只读取静态参考。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{
        AttachedFields, AttachedModelInput, KinematicsParameter as F, KinematicsParameters,
        MocapModelInput, ParameterBatch,
    },
    physics::{KinematicsPlan, KinematicsSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/mocap-kinematics/mixed-tree.json")).unwrap()
}
fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}
fn model(v: &Value) -> MocapModelInput {
    let m = &v["model"];
    MocapModelInput::new(
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
        .unwrap(),
        ids(m, "body_mocapid"),
    )
    .unwrap()
}
fn fields(out: &KinematicsSnapshot, world: usize) -> Vec<(&'static str, &[f32])> {
    let r = out.rigid().world(world).unwrap();
    let c = out.com().world(world).unwrap();
    let a = out.attached().world(world).unwrap();
    vec![
        ("xpos", r.xpos),
        ("xquat", r.xquat),
        ("xmat", r.xmat),
        ("xipos", r.xipos),
        ("ximat", r.ximat),
        ("xanchor", r.xanchor),
        ("xaxis", r.xaxis),
        ("subtree_mass", c.subtree_mass),
        ("subtree_com", c.subtree_com),
        ("cinert", c.cinert),
        ("cdof", c.cdof),
        ("geom_xpos", a.geom_xpos),
        ("geom_xmat", a.geom_xmat),
        ("site_xpos", a.site_xpos),
        ("site_xmat", a.site_xmat),
    ]
}
fn compare(out: &KinematicsSnapshot, world: usize, case: &Value) -> f64 {
    let mut maximum: f64 = 0.0;
    for (key, actual) in fields(out, world) {
        let expected = case[key].as_array().unwrap();
        assert_eq!(actual.len(), expected.len());
        for (index, (&a, e)) in actual.iter().zip(expected).enumerate() {
            let e = e.as_f64().unwrap();
            let error = (f64::from(a) - e).abs();
            maximum = maximum.max(error);
            assert!(
                a.is_finite() && error <= 2e-5 + 2e-5 * e.abs(),
                "G01 mocap {key} world={world} index={index} actual={a} expected={e} error={error}"
            );
        }
    }
    maximum
}

#[test]
fn reference_hashes_and_static_masks_match_native_topology() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/mocap-kinematics");
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/mocap-kinematics/manifest.json")).unwrap();
    for file in manifest["files"].as_array().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(file["path"].as_str().unwrap())).unwrap())
            ),
            file["sha256"].as_str().unwrap()
        );
    }
    let v = fixture();
    let model = model(&v);
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 1789);
    assert_eq!(v["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(v["relative_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(model.nmocap(), 2);
    assert_eq!(model.body_mocapid()[3], 1);
    assert_eq!(model.body_mocapid()[6], 0);
    let weld = ids(&v["model"], "body_weldid");
    let roots = ids(&v["model"], "body_rootid");
    for (geom, &body) in model.attached().fields().geom_bodyid.iter().enumerate() {
        assert_eq!(
            model.static_geom()[geom],
            weld[body as usize] == 0 && model.body_mocapid()[roots[body as usize] as usize] == -1
        );
    }
    assert_eq!(
        model.static_geom(),
        [true, true, true, false, false, false, false, false]
    );
    assert_eq!(v["cases"].as_array().unwrap().len(), 8);
    for case in v["cases"].as_array().unwrap() {
        for field in ["qpos", "mocap_pos", "mocap_quat"] {
            for value in case[field].as_array().unwrap() {
                let value = value.as_f64().unwrap();
                // serde_json's default f64 parser can shift the final binary
                // digit. Only permit f64 roundoff, never f32 input drift.
                assert!(
                    (value - f64::from(value as f32)).abs() <= f64::EPSILON * value.abs(),
                    "{field} input must preserve f32 precision"
                );
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn mocap_updates_match_native_references_across_cuda_blocks() {
    reference_hashes_and_static_masks_match_native_topology();
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let plan =
        KinematicsPlan::with_mocap(&session, model(&v), KinematicsParameters::default()).unwrap();
    for worlds in [1, 2, 5, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        assert_eq!(data.nmocap(), 2);
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rigid" })
        ));
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        for world in 0..worlds {
            eprintln!(
                "G01-mocap kind=native world={world} max_abs_error={:e}",
                compare(&out, world, &v["cases"][0])
            );
        }
        for round in 0..3 {
            let selected: Vec<_> = (0..worlds)
                .map(|w| &v["cases"][(w + round + 1) % 8])
                .collect();
            data.write_qpos(
                &selected
                    .iter()
                    .flat_map(|c| floats(c, "qpos"))
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            data.write_mocap(
                &selected
                    .iter()
                    .flat_map(|c| floats(c, "mocap_pos"))
                    .collect::<Vec<_>>(),
                &selected
                    .iter()
                    .flat_map(|c| floats(c, "mocap_quat"))
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for (world, case) in selected.iter().enumerate() {
                eprintln!(
                    "G01-mocap kind=native world={world} max_abs_error={:e}",
                    compare(&out, world, case)
                );
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn mocap_world_writes_validate_before_invalidation_and_remain_isolated() {
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let plan =
        KinematicsPlan::with_mocap(&session, model(&v), KinematicsParameters::default()).unwrap();
    let other =
        KinematicsPlan::with_mocap(&session, model(&v), KinematicsParameters::default()).unwrap();
    let mut data = plan.create_data(513).unwrap();
    let mut second = plan.create_data(2).unwrap();
    plan.update(&mut data).unwrap();
    plan.update(&mut second).unwrap();
    let snapshot = data.readback().unwrap();
    let p = floats(&v["cases"][0], "mocap_pos");
    let q = floats(&v["cases"][0], "mocap_quat");
    assert!(matches!(
        data.write_world_mocap(513, &p, &q),
        Err(TransferError::Input(InputError::InvalidIndex { .. }))
    ));
    assert!(data.write_world_mocap(usize::MAX, &p, &q).is_err());
    assert!(data.write_mocap(&p, &q).is_err());
    assert!(data.write_world_mocap(0, &p, &q[..4]).is_err());
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut bad_p = p.clone();
        bad_p[5] = bad;
        assert!(data.write_world_mocap(512, &bad_p, &q).is_err());
        let mut bad_q = q.clone();
        bad_q[7] = bad;
        assert!(data.write_world_mocap(512, &p, &bad_q).is_err());
    }
    for bad in [0.0, 1e-7, 1e7] {
        let mut bad_q = q.clone();
        bad_q[4..].copy_from_slice(&[bad, 0.0, 0.0, 0.0]);
        assert!(data.write_world_mocap(512, &p, &bad_q).is_err());
    }
    assert_eq!(
        other.update_rigid(&mut data),
        Err(TransferError::ModelMismatch)
    );
    compare(&data.readback().unwrap(), 512, &v["cases"][0]);
    for (world, id) in [(0, 1), (256, 4), (512, 7)] {
        let case = &v["cases"][id];
        data.write_world_qpos(world, &floats(case, "qpos")).unwrap();
        data.write_world_mocap(
            world,
            &floats(case, "mocap_pos"),
            &floats(case, "mocap_quat"),
        )
        .unwrap();
    }
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rigid" })
    ));
    assert!(plan.update_attached(&mut data).is_err());
    assert!(plan.update_com(&mut data).is_err());
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    for world in 0..513 {
        let id = match world {
            0 => 1,
            256 => 4,
            512 => 7,
            _ => 0,
        };
        compare(&out, world, &v["cases"][id]);
    }
    compare(&snapshot, 0, &v["cases"][0]);
    compare(&second.readback().unwrap(), 1, &v["cases"][0]);
    drop(plan);
    drop(other);
    drop(session);
    compare(&data.readback().unwrap(), 512, &v["cases"][7]);
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn periodic_defaults_initialize_static_geoms_and_mocap_independently() {
    let v = fixture();
    let m = model(&v);
    let nb = m.attached().rigid().kinematics().nbody();
    let mut parameters = KinematicsParameters::default();
    let mut bp = floats(&v["model"], "body_pos").repeat(3);
    for row in 0..3 {
        for body in [1, 3, 6] {
            bp[row * 3 * nb + 3 * body] += row as f32;
        }
    }
    let mut bq = floats(&v["model"], "body_quat").repeat(5);
    for row in 0..5 {
        for body in [3, 6] {
            bq[row * 4 * nb + 4 * body..row * 4 * nb + 4 * body + 4].copy_from_slice(
                if row % 2 == 0 {
                    &[1.0, 0.0, 0.0, 0.0]
                } else {
                    &[0.0, 1.0, 0.0, 0.0]
                },
            );
        }
    }
    let mut gp = floats(&v["model"], "geom_pos").repeat(7);
    let ng = m.attached().ngeom();
    for row in 0..7 {
        gp[row * 3 * ng] += row as f32 * 0.25;
    }
    parameters.set(
        F::BodyPos,
        ParameterBatch::new(3, 3 * nb, bp.clone()).unwrap(),
    );
    parameters.set(
        F::BodyQuat,
        ParameterBatch::new(5, 4 * nb, bq.clone()).unwrap(),
    );
    parameters.set(
        F::GeomPos,
        ParameterBatch::new(7, 3 * ng, gp.clone()).unwrap(),
    );
    let session = TransferSession::new(0).unwrap();
    let plan = KinematicsPlan::with_mocap(&session, m, parameters).unwrap();
    for worlds in [1, 2, 5, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        for world in 0..worlds {
            let r = out.rigid().world(world).unwrap();
            let a = out.attached().world(world).unwrap();
            for body in [3, 6] {
                assert_eq!(
                    &r.xpos[3 * body..3 * body + 3],
                    &bp[(world % 3) * 3 * nb + 3 * body..(world % 3) * 3 * nb + 3 * body + 3]
                );
                assert_eq!(
                    &r.xquat[4 * body..4 * body + 4],
                    &bq[(world % 5) * 4 * nb + 4 * body..(world % 5) * 4 * nb + 4 * body + 4]
                );
            }
            assert_eq!(
                &a.geom_xpos[..3],
                &gp[(world % 7) * 3 * ng..(world % 7) * 3 * ng + 3]
            );
            let expected = floats(&v["cases"][0], "geom_xpos");
            assert!((a.geom_xpos[3] - expected[3] - (world % 3) as f32).abs() < 2e-6);
        }
        data.write_world_mocap(worlds - 1, &[8.0; 6], &[1.0, 0.0, 0.0, 0.0].repeat(2))
            .unwrap();
        plan.update(&mut data).unwrap();
        let updated = data.readback().unwrap();
        for world in 0..worlds {
            let old = out.attached().world(world).unwrap();
            let new = updated.attached().world(world).unwrap();
            assert_eq!(&old.geom_xpos[..9], &new.geom_xpos[..9]);
            assert_eq!(&old.geom_xmat[..27], &new.geom_xmat[..27]);
            if world == worlds - 1 {
                assert_eq!(
                    &updated.rigid().world(world).unwrap().xpos[9..12],
                    &[8.0; 3]
                );
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn zero_dof_mocap_state_preserves_empty_qpos_and_attachment_sets() {
    use mjwarp_rs::model::{
        InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
    };
    let session = TransferSession::new(0).unwrap();
    for (ng, ns) in [(0, 0), (2, 0), (0, 2), (2, 2)] {
        let k = KinematicModelInput::new(
            0,
            KinematicFields {
                body_parentid: vec![0, 0, 0],
                body_jntadr: vec![-1; 3],
                body_jntnum: vec![0; 3],
                body_pos: vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(3),
                ..Default::default()
            },
        )
        .unwrap();
        let rigid = InertialModelInput::new(
            k,
            InertialFields {
                body_ipos: vec![0.0; 9],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(3),
                body_mass: vec![0.0, 1.0, 1.0],
                body_inertia: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
                ..Default::default()
            },
        )
        .unwrap();
        let m = AttachedModelInput::new(
            rigid,
            AttachedFields {
                geom_bodyid: vec![1, 2][..ng].to_vec(),
                geom_pos: vec![0.0; 3 * ng],
                geom_quat: [1.0, 0.0, 0.0, 0.0].repeat(ng),
                site_bodyid: vec![1, 2][..ns].to_vec(),
                site_pos: vec![0.0; 3 * ns],
                site_quat: [1.0, 0.0, 0.0, 0.0].repeat(ns),
            },
        )
        .unwrap();
        let plan = KinematicsPlan::with_mocap(
            &session,
            MocapModelInput::new(m, vec![-1, 1, 0]).unwrap(),
            KinematicsParameters::default(),
        )
        .unwrap();
        for worlds in [2, 513] {
            let mut data = plan.create_data(worlds).unwrap();
            data.write_qpos(&[]).unwrap();
            data.write_world_qpos(worlds - 1, &[]).unwrap();
            let positions: Vec<_> = (0..worlds)
                .flat_map(|w| [w as f32, 2.0, 3.0, -(w as f32), 5.0, 6.0])
                .collect();
            data.write_mocap(&positions, &[-2.0, 0.0, 0.0, 0.0].repeat(2 * worlds))
                .unwrap();
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for world in 0..worlds {
                let r = out.rigid().world(world).unwrap();
                let a = out.attached().world(world).unwrap();
                assert_eq!(&r.xpos[3..6], &positions[6 * world + 3..6 * world + 6]);
                assert_eq!(&r.xpos[6..9], &positions[6 * world..6 * world + 3]);
                if ng != 0 {
                    assert_eq!(a.geom_xpos, &r.xpos[3..]);
                }
                if ns != 0 {
                    assert_eq!(a.site_xpos, &r.xpos[3..]);
                }
                assert!(r.xanchor.is_empty() && r.xaxis.is_empty());
                assert!(out.com().world(world).unwrap().cdof.is_empty());
                assert_eq!(
                    &out.com().world(world).unwrap().subtree_com[..3],
                    &[0.0, 3.5, 4.5]
                );
            }
        }
    }
}
