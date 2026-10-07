#![cfg(feature = "cuda-probe")]
//! G01固定肌腱子集验收。
//! 产品测试只读取静态参考。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
        FixedTendonFields, FixedTendonModelInput, KinematicsParameter, KinematicsParameters,
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
    serde_json::from_str(include_str!("../fixtures/fixed-tendon/mixed-tree.json")).unwrap()
}
fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}
fn model(v: &Value) -> FixedTendonModelInput {
    let m = &v["model"];
    let attached = AttachedModelInput::new(
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
    .unwrap();
    let base = CamLightModelInput::new(
        MocapModelInput::new(attached, ids(m, "body_mocapid")).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    FixedTendonModelInput::new(
        base,
        FixedTendonFields {
            tendon_adr: ids(m, "tendon_adr"),
            tendon_num: ids(m, "tendon_num"),
            wrap_type: ids(m, "wrap_type"),
            wrap_objid: ids(m, "wrap_objid"),
            wrap_prm: floats(m, "wrap_prm"),
            ten_j_rowadr: ids(m, "ten_J_rowadr"),
            ten_j_rownnz: ids(m, "ten_J_rownnz"),
            ten_j_colind: ids(m, "ten_J_colind"),
        },
    )
    .unwrap()
}
fn plan(session: &TransferSession, v: &Value) -> KinematicsPlan {
    KinematicsPlan::with_fixed_tendons(
        session,
        model(v),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}
fn flattened(out: &KinematicsSnapshot, world: usize) -> Vec<f32> {
    let row = out.fixed_tendon().world(world).unwrap();
    [row.ten_length, row.ten_jacobian].concat()
}
fn expected(case: &Value) -> Vec<f64> {
    ["ten_length", "ten_J"]
        .into_iter()
        .flat_map(|key| {
            case[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
        })
        .collect()
}
fn compare(actual: &[f32], expected: &[f64]) -> f64 {
    assert_eq!(actual.len(), expected.len());
    let mut max: f64 = 0.0;
    for (index, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        let error = (f64::from(a) - e).abs();
        max = max.max(error);
        assert!(
            a.is_finite() && e.is_finite() && error <= 2e-5 + 2e-5 * e.abs(),
            "G01 fixed tendon index={index} actual={a} expected={e} error={error}"
        );
    }
    max
}

#[test]
fn reference_hashes_and_sparse_scalar_mapping_match() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/fixed-tendon");
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/fixed-tendon/manifest.json")).unwrap();
    for file in manifest["files"].as_array().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(file["path"].as_str().unwrap())).unwrap())
            ),
            file["sha256"].as_str().unwrap()
        );
    }
    assert_eq!(
        manifest["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    let v = fixture();
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 1789);
    assert_eq!(v["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(v["relative_tolerance"].as_f64(), Some(2e-5));
    let m = model(&v);
    assert_eq!((m.rows().ntendon(), m.rows().nnz()), (4, 8));
    assert_eq!(m.rows().row(0).unwrap(), &[9, 10, 11]);
    assert!(m.rows().row(4).is_err());
    let k = m.camlight().mocap().attached().rigid().kinematics();
    assert_eq!((k.nq(), k.nv()), (14, 12));
    assert_eq!(&k.fields().jnt_type[..2], &[0, 1]);
    assert_ne!(k.fields().jnt_qposadr[2], k.fields().jnt_dofadr[2]);
    assert_eq!(v["cases"].as_array().unwrap().len(), 8);
    for case in v["cases"].as_array().unwrap() {
        for q in case["qpos"].as_array().unwrap() {
            let q = q.as_f64().unwrap();
            assert!((q - f64::from(q as f32)).abs() <= f64::EPSILON * q.abs());
        }
    }
}

#[test]
fn rejects_unsupported_wraps_references_duplicates_and_nonfinite_coefficients() {
    let v = fixture();
    let original = model(&v);
    for kind in [0, 2, 3, 4, 5, -1, i32::MAX] {
        let mut f = original.fields().clone();
        f.wrap_type[0] = kind;
        assert!(matches!(
            FixedTendonModelInput::new(original.camlight().clone(), f),
            Err(InputError::InvalidTopology {
                field: "wrap_type",
                ..
            })
        ));
    }
    for joint in [-1, 0, 1, 5, i32::MAX, 2] {
        let mut f = original.fields().clone();
        f.wrap_objid[0] = joint;
        assert!(matches!(
            FixedTendonModelInput::new(original.camlight().clone(), f),
            Err(InputError::InvalidTopology {
                field: "wrap_objid",
                ..
            })
        ));
    }
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut f = original.fields().clone();
        f.wrap_prm[0] = bad;
        assert!(matches!(
            FixedTendonModelInput::new(original.camlight().clone(), f),
            Err(InputError::NonFinite {
                field: "wrap_prm",
                index: 0
            })
        ));
    }
}
#[test]
fn rejects_lengths_partitions_and_sparse_columns_before_gpu_work() {
    let original = model(&fixture());
    for field in 0..8 {
        let mut f = original.fields().clone();
        match field {
            0 => {
                f.tendon_adr.pop();
            }
            1 => {
                f.tendon_num.pop();
            }
            2 => {
                f.wrap_type.pop();
            }
            3 => {
                f.wrap_objid.pop();
            }
            4 => {
                f.wrap_prm.pop();
            }
            5 => {
                f.ten_j_rowadr.pop();
            }
            6 => {
                f.ten_j_rownnz.pop();
            }
            _ => {
                f.ten_j_colind.pop();
            }
        }
        assert!(FixedTendonModelInput::new(original.camlight().clone(), f).is_err());
    }
    for column in [-1, 0, 10, 12, i32::MAX] {
        let mut f = original.fields().clone();
        f.ten_j_colind[0] = column;
        assert!(FixedTendonModelInput::new(original.camlight().clone(), f).is_err());
    }
    for (csr, value) in [(false, -1), (false, 1), (true, -1), (true, 1)] {
        let mut f = original.fields().clone();
        if csr {
            f.ten_j_rowadr[0] = value;
        } else {
            f.tendon_adr[0] = value;
        }
        assert!(FixedTendonModelInput::new(original.camlight().clone(), f).is_err());
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn resident_fixed_tendons_match_native_across_cuda_blocks() {
    reference_hashes_and_sparse_scalar_mapping_match();
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let plan = plan(&session, &v);
    for worlds in [1, 2, 5, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        for round in 0..4 {
            if round != 0 {
                let q: Vec<_> = (0..worlds)
                    .flat_map(|w| floats(&v["cases"][(w + round) % 8], "qpos"))
                    .collect();
                data.write_qpos(&q).unwrap();
            }
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            assert_eq!(out.fixed_tendon().rows(), plan.fixed_tendon_rows());
            for world in 0..worlds {
                let id = if round == 0 { 0 } else { (world + round) % 8 };
                eprintln!(
                    "G01-fixed-tendon kind=native world={world} max_abs_error={:e}",
                    compare(&flattened(&out, world), &expected(&v["cases"][id]))
                );
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn lifecycle_staleness_identity_and_world_writes_preserve_snapshots() {
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let a = plan(&session, &v);
    let b = plan(&session, &v);
    let mut data = a.create_data(513).unwrap();
    let mut second = a.create_data(2).unwrap();
    assert!(matches!(
        b.update_fixed_tendons(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    a.update_fixed_tendons(&mut data).unwrap(); // No FK dependency in this strict subset.
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rigid" })
    ));
    a.update_rigid(&mut data).unwrap();
    a.update_attached(&mut data).unwrap();
    a.update_com(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "fixed_tendon"
        })
    ));
    a.update_fixed_tendons(&mut data).unwrap();
    a.update(&mut second).unwrap();
    let saved = data.readback().unwrap();
    assert!(
        data.write_world_qpos(513, &floats(&v["cases"][1], "qpos"))
            .is_err()
    );
    let mut bad = floats(&v["cases"][1], "qpos");
    bad[11] = f32::NAN;
    assert!(data.write_world_qpos(512, &bad).is_err());
    assert_eq!(
        flattened(&saved, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    for (world, id) in [(0, 1), (256, 4), (512, 7)] {
        data.write_world_qpos(world, &floats(&v["cases"][id], "qpos"))
            .unwrap();
    }
    assert!(data.readback().is_err());
    a.update(&mut data).unwrap();
    let updated = data.readback().unwrap();
    for world in 0..513 {
        let id = match world {
            0 => 1,
            256 => 4,
            512 => 7,
            _ => 0,
        };
        compare(&flattened(&updated, world), &expected(&v["cases"][id]));
    }
    // Unrelated stages leave tendon readiness intact.
    a.update_com(&mut data).unwrap();
    a.update_attached(&mut data).unwrap();
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    drop(b);
    drop(a);
    drop(session);
    compare(
        &flattened(&second.readback().unwrap(), 1),
        &expected(&v["cases"][0]),
    );
    compare(&flattened(&saved, 512), &expected(&v["cases"][0]));
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn raw_qpos_periods_and_sparse_jacobians_match_independent_oracle() {
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let m = model(&v);
    let base = m.camlight().mocap().attached();
    let k = base.rigid().kinematics();
    let f = m.fields().clone();
    let rows = m.rows().clone();
    let kf = k.fields().clone();
    let q: Vec<_> = (1..=3)
        .flat_map(|c| floats(&v["cases"][c], "qpos"))
        .collect();
    let mut parameters = KinematicsParameters::default();
    parameters.set(
        KinematicsParameter::Qpos0,
        ParameterBatch::new(3, k.nq(), q.clone()).unwrap(),
    );
    let a =
        KinematicsPlan::with_fixed_tendons(&session, m, parameters, CamLightParameters::default())
            .unwrap();
    let mut data = a.create_data(513).unwrap();
    a.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    for world in 0..513 {
        let row = &q[(world % 3) * 14..(world % 3 + 1) * 14];
        let mut length = vec![0.0f64; 4];
        let mut jac = vec![0.0f64; 8];
        for (t, len) in length.iter_mut().enumerate() {
            for i in f.tendon_adr[t] as usize..(f.tendon_adr[t] + f.tendon_num[t]) as usize {
                let j = f.wrap_objid[i] as usize;
                let coefficient = f64::from(f.wrap_prm[i]);
                *len += coefficient * f64::from(row[kf.jnt_qposadr[j] as usize]);
                let slot = rows
                    .row(t)
                    .unwrap()
                    .binary_search(&kf.jnt_dofadr[j])
                    .unwrap();
                jac[rows.rowadr()[t] as usize + slot] = coefficient;
            }
        }
        compare(&flattened(&out, world), &[length, jac].concat());
    }
    // Compare a measured length derivative with the sparse DOF moment.
    let saved = out.fixed_tendon().world(512).unwrap();
    let mut perturbed = q[2 * 14..3 * 14].to_vec();
    perturbed[11] += 0.001;
    let delta = f64::from(perturbed[11]) - f64::from(q[2 * 14 + 11]);
    data.write_world_qpos(512, &perturbed).unwrap();
    a.update(&mut data).unwrap();
    let next = data.readback().unwrap();
    let next = next.fixed_tendon().world(512).unwrap();
    let derivative = (f64::from(next.ten_length[0]) - f64::from(saved.ten_length[0])) / delta;
    assert!((derivative - f64::from(saved.ten_jacobian[0])).abs() < 0.001);
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn extra_sparse_columns_zero_each_update_and_empty_tendons_keep_legacy_readback() {
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let m = model(&v);
    let mut fields = m.fields().clone();
    fields.ten_j_colind.insert(0, 0);
    fields.ten_j_rownnz[0] += 1;
    for start in &mut fields.ten_j_rowadr[1..] {
        *start += 1;
    }
    let m = FixedTendonModelInput::new(m.camlight().clone(), fields).unwrap();
    let a = KinematicsPlan::with_fixed_tendons(
        &session,
        m,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = a.create_data(513).unwrap();
    for round in 0..2 {
        data.write_qpos(&floats(&v["cases"][round], "qpos").repeat(513))
            .unwrap();
        a.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        for world in 0..513 {
            let row = out.fixed_tendon().world(world).unwrap();
            assert_eq!(row.ten_jacobian[0], 0.0);
            compare(row.ten_length, &expected(&v["cases"][round])[..4]);
            compare(&row.ten_jacobian[1..], &expected(&v["cases"][round])[4..]);
        }
    }
    // Exercise all five nonempty output groups in the same plan.
    let original = model(&v);
    let identity = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let camlight = CamLightModelInput::new(
        original.camlight().mocap().clone(),
        CamLightFields {
            cam_mode: vec![0],
            cam_bodyid: vec![0],
            cam_targetbodyid: vec![-1],
            cam_pos: vec![1.0, 2.0, 3.0],
            cam_quat: vec![1.0, 0.0, 0.0, 0.0],
            cam_pos0: vec![0.0; 3],
            cam_poscom0: vec![0.0; 3],
            cam_mat0: identity.clone(),
            light_mode: vec![0],
            light_bodyid: vec![0],
            light_targetbodyid: vec![-1],
            light_pos: vec![1.0, 2.0, 3.0],
            light_dir: vec![0.0, 0.0, 1.0],
            light_pos0: vec![0.0; 3],
            light_poscom0: vec![0.0; 3],
            light_dir0: vec![0.0; 3],
        },
    )
    .unwrap();
    let combined = FixedTendonModelInput::new(camlight, original.fields().clone()).unwrap();
    let a = KinematicsPlan::with_fixed_tendons(
        &session,
        combined,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = a.create_data(513).unwrap();
    a.update_rigid(&mut data).unwrap();
    a.update_attached(&mut data).unwrap();
    a.update_com(&mut data).unwrap();
    a.update_fixed_tendons(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "camlight" })
    ));
    a.update_camlight(&mut data).unwrap();
    let out = data.readback().unwrap();
    for world in 0..513 {
        compare(&flattened(&out, world), &expected(&v["cases"][0]));
        let c = out.camlight().world(world).unwrap();
        assert_eq!(c.cam_xpos, &[1.0, 2.0, 3.0]);
        assert_eq!(c.cam_xmat, identity);
        assert_eq!(c.light_xpos, &[1.0, 2.0, 3.0]);
        assert_eq!(c.light_xdir, &[0.0, 0.0, 1.0]);
    }
    let empty =
        FixedTendonModelInput::new(model(&v).camlight().clone(), FixedTendonFields::default())
            .unwrap();
    let a = KinematicsPlan::with_fixed_tendons(
        &session,
        empty,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = a.create_data(513).unwrap();
    a.update_rigid(&mut data).unwrap();
    a.update_attached(&mut data).unwrap();
    a.update_com(&mut data).unwrap();
    let out = data.readback().unwrap();
    assert!(out.fixed_tendon().world(512).unwrap().ten_length.is_empty());
    assert!(out.fixed_tendon().world(513).is_err());
}
