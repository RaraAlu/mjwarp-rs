#![cfg(feature = "cuda-probe")]
//! G01混合肌腱全局编号验收。
//! 产品测试只读取静态原生参考。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
        KinematicsParameters, MocapModelInput, ParameterBatch, SpatialTendonGeometry, TendonFields,
        TendonModelInput, TendonSubset,
    },
    physics::{KinematicsData, KinematicsPlan, KinematicsSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/mixed-tendon/mixed-tree.json")).unwrap()
}
fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
fn parts(v: &Value) -> (CamLightModelInput, TendonFields, SpatialTendonGeometry) {
    let m = &v["model"];
    let base = AttachedModelInput::new(
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
    let cam = CamLightModelInput::new(
        MocapModelInput::new(base, ids(m, "body_mocapid")).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    (
        cam,
        TendonFields {
            tendon_adr: ids(m, "tendon_adr"),
            tendon_num: ids(m, "tendon_num"),
            wrap_type: ids(m, "wrap_type"),
            wrap_objid: ids(m, "wrap_objid"),
            wrap_prm: floats(m, "wrap_prm"),
            ten_j_rowadr: ids(m, "ten_J_rowadr"),
            ten_j_rownnz: ids(m, "ten_J_rownnz"),
            ten_j_colind: ids(m, "ten_J_colind"),
        },
        SpatialTendonGeometry {
            geom_type: ids(m, "geom_type"),
            geom_size: ParameterBatch::new(
                3,
                12,
                (0..3)
                    .flat_map(|c| floats(&v["cases"][c], "geom_size"))
                    .collect(),
            )
            .unwrap(),
        },
    )
}
fn model(v: &Value) -> TendonModelInput {
    let (c, f, g) = parts(v);
    TendonModelInput::with_geometry(c, f, g).unwrap()
}
fn plan(s: &TransferSession, v: &Value) -> KinematicsPlan {
    KinematicsPlan::with_tendons(
        s,
        model(v),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}
fn write(data: &mut KinematicsData, w: usize, c: &Value) {
    data.write_world_qpos(w, &floats(c, "qpos")).unwrap();
    data.write_world_mocap(w, &floats(c, "mocap_pos"), &floats(c, "mocap_quat"))
        .unwrap();
}
fn compare(actual: &[f32], expected: &Value) -> f64 {
    let expected = expected.as_array().unwrap();
    assert_eq!(actual.len(), expected.len());
    let mut max: f64 = 0.0;
    for (i, (&a, e)) in actual.iter().zip(expected).enumerate() {
        let e = e.as_f64().unwrap();
        let error = (f64::from(a) - e).abs();
        max = max.max(error);
        assert!(
            a.is_finite() && error <= 2e-5 + 2e-5 * e.abs(),
            "G01 mixed index={i} actual={a} expected={e} error={error}"
        );
    }
    max
}
fn native(out: &KinematicsSnapshot, world: usize, c: &Value) -> f64 {
    let global = out.tendon().unwrap();
    let a = global.world(world).unwrap();
    assert_eq!(a.ten_wrapadr, ids(c, "ten_wrapadr"));
    assert_eq!(a.ten_wrapnum, ids(c, "ten_wrapnum"));
    assert_eq!(a.wrap_obj, ids(c, "wrap_obj"));
    let mut max = compare(a.ten_length, &c["ten_length"]);
    max = max.max(compare(a.ten_jacobian, &c["ten_J"]));
    max = max.max(compare(a.wrap_xpos, &c["wrap_xpos"]));
    let local_points = out.spatial_tendon().world(world).unwrap().wrap_xpos;
    assert_eq!(
        a.wrap_xpos[..local_points.len()]
            .iter()
            .map(|x| x.to_bits())
            .collect::<Vec<_>>(),
        local_points.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
    );
    assert!(
        a.wrap_xpos[local_points.len()..]
            .iter()
            .all(|&x| x.to_bits() == 0)
    );
    // 合并只重排GPU结果。
    // 它保持每个f32原始位模式。
    for t in 0..global.rows().ntendon() {
        let start = global.rows().rowadr()[t] as usize;
        let count = global.rows().rownnz()[t] as usize;
        let (length, jac) = match global.rows().subset(t).unwrap() {
            TendonSubset::Fixed(local) => {
                let o = out.fixed_tendon();
                let row = o.world(world).unwrap();
                let adr = o.rows().rowadr()[local] as usize;
                (row.ten_length[local], &row.ten_jacobian[adr..adr + count])
            }
            TendonSubset::Spatial(local) => {
                let o = out.spatial_tendon();
                let row = o.world(world).unwrap();
                let adr = o.rows().rowadr()[local] as usize;
                (row.ten_length[local], &row.ten_jacobian[adr..adr + count])
            }
        };
        assert_eq!(a.ten_length[t].to_bits(), length.to_bits());
        assert_eq!(
            a.ten_jacobian[start..start + count]
                .iter()
                .map(|x| x.to_bits())
                .collect::<Vec<_>>(),
            jac.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
    }
    max
}

#[test]
fn reference_hashes_preserve_interleaved_native_rows_and_zero_coefficient() {
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/mixed-tendon/manifest.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/mixed-tendon");
    for f in manifest["files"].as_array().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(f["path"].as_str().unwrap())).unwrap())
            ),
            f["sha256"].as_str().unwrap()
        );
    }
    assert_eq!(
        manifest["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    let v = fixture();
    let m = model(&v);
    let r = m.rows();
    assert_eq!((r.ntendon(), r.nnz(), r.nwrap()), (14, 16, 43));
    assert_eq!(r.rowadr(), ids(&v["model"], "ten_J_rowadr"));
    assert_eq!(r.rownnz(), ids(&v["model"], "ten_J_rownnz"));
    assert_eq!(r.colind(), ids(&v["model"], "ten_J_colind"));
    for (i, t) in [0, 3, 7, 13].into_iter().enumerate() {
        assert_eq!(r.subset(t).unwrap(), TendonSubset::Fixed(i));
    }
    assert_eq!(r.subset(1).unwrap(), TendonSubset::Spatial(0));
    assert!(r.subset(14).is_err());
    assert!(r.row(usize::MAX).is_err());
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 2101);
    assert_eq!(v["cases"].as_array().unwrap().len(), 12);
    assert_eq!(v["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(v["relative_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(
        v["cases"][0]["ten_J"][r.rowadr()[3] as usize].as_f64(),
        Some(0.0)
    );
}

#[test]
fn rejects_global_lengths_partitions_csr_and_mixed_paths() {
    let v = fixture();
    let (c, f, g) = parts(&v);
    let check = |f| TendonModelInput::with_geometry(c.clone(), f, g.clone());
    for kind in 0..5 {
        let mut bad = f.clone();
        match kind {
            0 => {
                bad.tendon_num.pop();
            }
            1 => {
                bad.wrap_objid.pop();
            }
            2 => {
                bad.wrap_prm.pop();
            }
            3 => {
                bad.ten_j_rowadr.pop();
            }
            _ => {
                bad.ten_j_rownnz.pop();
            }
        }
        assert!(matches!(check(bad), Err(InputError::LengthMismatch { .. })));
    }
    for kind in 0..9 {
        let mut bad = f.clone();
        match kind {
            0 => bad.tendon_adr[1] += 1,
            1 => bad.tendon_num[0] = 0,
            2 => bad.ten_j_rowadr[1] += 1,
            3 => bad.ten_j_colind[0] = -1,
            4 => bad.ten_j_colind[1] = bad.ten_j_colind[0],
            5 => bad.wrap_type[1] = 3,
            6 => bad.wrap_type[2] = 1,
            7 => bad.wrap_objid[0] = 2,
            _ => bad.wrap_prm[0] = f32::NAN,
        }
        assert!(check(bad).is_err(), "boundary {kind}");
    }
    assert!(TendonModelInput::new(c.clone(), f.clone()).is_err());
    let empty = TendonModelInput::new(c, TendonFields::default()).unwrap();
    assert_eq!(empty.rows().ntendon(), 0);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn matches_native_global_fields_for_uneven_worlds_and_repeated_updates() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    assert_eq!(p.tendon_rows().unwrap().ntendon(), 14);
    for worlds in [1, 2, 5, 513] {
        let mut d = p.create_data(worlds).unwrap();
        assert_eq!(d.tendon_rows(), p.tendon_rows());
        assert!(d.readback().is_err());
        for round in 0..4 {
            for w in 0..worlds {
                write(&mut d, w, &v["cases"][3 * round + w % 3]);
            }
            p.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            assert_eq!(out.tendon().unwrap().worlds(), worlds);
            assert!(out.tendon().unwrap().world(worlds).is_err());
            let mut max: f64 = 0.0;
            for w in 0..worlds {
                max = max.max(native(&out, w, &v["cases"][3 * round + w % 3]));
            }
            println!(
                "G01-global-tendon worlds={worlds} round={round} comparisons={worlds} max_abs_error={max:e}"
            );
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn invalidates_global_output_and_preserves_snapshot_and_model_identity() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    let other = plan(&s, &v);
    let mut d = p.create_data(5).unwrap();
    for w in 0..5 {
        write(&mut d, w, &v["cases"][w % 3]);
    }
    p.update(&mut d).unwrap();
    let old = d.readback().unwrap();
    let keep = old.tendon().unwrap().world(2).unwrap().ten_length.to_vec();
    assert!(matches!(
        other.update(&mut d),
        Err(TransferError::ModelMismatch)
    ));
    assert!(
        d.write_world_qpos(5, &floats(&v["cases"][0], "qpos"))
            .is_err()
    );
    assert_eq!(
        d.readback()
            .unwrap()
            .tendon()
            .unwrap()
            .world(2)
            .unwrap()
            .ten_length,
        keep
    );
    p.update_fixed_tendons(&mut d).unwrap();
    assert!(matches!(
        d.readback(),
        Err(TransferError::StageNotReady { stage: "tendon" })
    ));
    p.update_tendons(&mut d).unwrap();
    p.update_spatial_tendons(&mut d).unwrap();
    assert!(matches!(
        d.readback(),
        Err(TransferError::StageNotReady { stage: "tendon" })
    ));
    write(&mut d, 2, &v["cases"][5]);
    assert!(p.update_tendons(&mut d).is_err());
    p.update(&mut d).unwrap();
    assert_eq!(old.tendon().unwrap().world(2).unwrap().ten_length, keep);
    native(&d.readback().unwrap(), 2, &v["cases"][5]);
    drop(p);
    drop(s);
    native(&d.readback().unwrap(), 2, &v["cases"][5]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn handles_empty_and_single_kind_global_models_without_changing_local_apis() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let (cam, fields, geometry) = parts(&v);
    for kind in [None, Some(false), Some(true)] {
        let mut f = TendonFields::default();
        let selected: Vec<_> = (0..14)
            .filter(|&t| {
                kind.is_some_and(|fixed| {
                    (fields.wrap_type[fields.tendon_adr[t] as usize] == 1) == fixed
                })
            })
            .collect();
        for &t in &selected {
            let start = fields.tendon_adr[t] as usize;
            let end = start + fields.tendon_num[t] as usize;
            let row = fields.ten_j_rowadr[t] as usize;
            let count = fields.ten_j_rownnz[t] as usize;
            f.tendon_adr.push(f.wrap_type.len() as i32);
            f.tendon_num.push(fields.tendon_num[t]);
            f.wrap_type.extend_from_slice(&fields.wrap_type[start..end]);
            f.wrap_objid
                .extend_from_slice(&fields.wrap_objid[start..end]);
            f.wrap_prm.extend_from_slice(&fields.wrap_prm[start..end]);
            f.ten_j_rowadr.push(f.ten_j_colind.len() as i32);
            f.ten_j_rownnz.push(count as i32);
            f.ten_j_colind
                .extend_from_slice(&fields.ten_j_colind[row..row + count]);
        }
        let nw = f.wrap_type.len();
        let m = TendonModelInput::with_geometry(cam.clone(), f, geometry.clone()).unwrap();
        let local = KinematicsPlan::with_spatial_tendons(
            &s,
            m.spatial().clone(),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let p = KinematicsPlan::with_tendons(
            &s,
            m,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        for worlds in [1, 513] {
            let mut data = p.create_data(worlds).unwrap();
            let mut old_data = local.create_data(worlds).unwrap();
            for w in 0..worlds {
                write(&mut data, w, &v["cases"][3 + w % 3]);
                write(&mut old_data, w, &v["cases"][3 + w % 3]);
            }
            p.update(&mut data).unwrap();
            local.update(&mut old_data).unwrap();
            assert!(old_data.readback().unwrap().tendon().is_none());
            let out = data.readback().unwrap();
            for w in 0..worlds {
                let c = &v["cases"][3 + w % 3];
                let a = out.tendon().unwrap().world(w).unwrap();
                let length: Vec<_> = selected
                    .iter()
                    .map(|&t| c["ten_length"][t].clone())
                    .collect();
                let jac: Vec<_> = selected
                    .iter()
                    .flat_map(|&t| {
                        let start = fields.ten_j_rowadr[t] as usize;
                        let count = fields.ten_j_rownnz[t] as usize;
                        c["ten_J"].as_array().unwrap()[start..start + count]
                            .iter()
                            .cloned()
                    })
                    .collect();
                compare(a.ten_length, &Value::Array(length));
                compare(a.ten_jacobian, &Value::Array(jac));
                if kind == Some(false) {
                    assert_eq!(
                        a.ten_wrapadr,
                        selected
                            .iter()
                            .map(|&t| ids(c, "ten_wrapadr")[t])
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(
                        a.ten_wrapnum,
                        selected
                            .iter()
                            .map(|&t| ids(c, "ten_wrapnum")[t])
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(a.wrap_obj, &ids(c, "wrap_obj")[..2 * nw]);
                    compare(
                        a.wrap_xpos,
                        &Value::Array(c["wrap_xpos"].as_array().unwrap()[..6 * nw].to_vec()),
                    );
                } else {
                    assert!(
                        a.ten_wrapadr
                            .iter()
                            .chain(a.ten_wrapnum)
                            .chain(a.wrap_obj)
                            .all(|&x| x == 0)
                    );
                    assert!(a.wrap_xpos.iter().all(|&x| x == 0.0));
                }
            }
        }
    }
}
