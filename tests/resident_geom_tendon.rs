#![cfg(feature = "cuda-probe")]
//! G01球柱绕行辅助验收。
//! 产品测试只读取静态参考。
use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
        FixedTendonFields, FixedTendonModelInput, InertialFields, InertialModelInput,
        KinematicFields, KinematicModelInput, KinematicsParameter as F, KinematicsParameters,
        MocapModelInput, ParameterBatch, SpatialTendonFields, SpatialTendonGeometry,
        SpatialTendonModelInput,
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
    serde_json::from_str(include_str!("../fixtures/geom-tendon/wrap-tree.json")).unwrap()
}
fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
fn parts(
    v: &Value,
) -> (
    FixedTendonModelInput,
    SpatialTendonFields,
    SpatialTendonGeometry,
) {
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
        FixedTendonModelInput::new(cam, FixedTendonFields::default()).unwrap(),
        SpatialTendonFields {
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
fn model(v: &Value) -> SpatialTendonModelInput {
    let (b, f, g) = parts(v);
    SpatialTendonModelInput::with_geometry(b, f, g).unwrap()
}
fn plan(s: &TransferSession, v: &Value) -> KinematicsPlan {
    KinematicsPlan::with_spatial_tendons(
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
fn compare(a: &[f32], e: &[f64]) -> f64 {
    assert_eq!(a.len(), e.len());
    let mut max: f64 = 0.0;
    for (i, (&a, &e)) in a.iter().zip(e).enumerate() {
        let error = (f64::from(a) - e).abs();
        max = max.max(error);
        assert!(
            a.is_finite() && error <= 2e-5 + 2e-5 * e.abs(),
            "G01 geom index={i} actual={a} expected={e} error={error}"
        );
    }
    max
}
fn native(out: &KinematicsSnapshot, w: usize, c: &Value) -> f64 {
    let a = out.spatial_tendon().world(w).unwrap();
    assert_eq!(a.ten_wrapadr, ids(c, "ten_wrapadr"));
    assert_eq!(a.ten_wrapnum, ids(c, "ten_wrapnum"));
    assert_eq!(a.wrap_obj, ids(c, "wrap_obj"));
    let e: Vec<_> = ["ten_length", "ten_J", "wrap_xpos"]
        .into_iter()
        .flat_map(|k| c[k].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()))
        .collect();
    compare(&[a.ten_length, a.ten_jacobian, a.wrap_xpos].concat(), &e)
}
#[test]
fn reference_hashes_cover_dynamic_sphere_cylinder_and_inside_paths() {
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/geom-tendon/manifest.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/geom-tendon");
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
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 2099);
    let m = model(&v);
    assert_eq!(
        (m.rows().ntendon(), m.rows().nnz(), m.rows().nwrap()),
        (10, 10, 37)
    );
    assert_eq!(m.geometry().unwrap().geom_size.batches(), 3);
    assert_eq!(v["cases"].as_array().unwrap().len(), 12);
    assert_ne!(v["cases"][0]["ten_wrapnum"], v["cases"][3]["ten_wrapnum"]);
    assert_eq!(v["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(v["relative_tolerance"].as_f64(), Some(2e-5));
}
#[test]
fn rejects_geometry_side_path_dimensions_and_missing_geom_dofs() {
    let v = fixture();
    let (b, f, g) = parts(&v);
    assert!(SpatialTendonModelInput::new(b.clone(), f.clone()).is_err());
    for id in [-1, 4, i32::MAX] {
        let mut bad = f.clone();
        bad.wrap_objid[1] = id;
        assert!(SpatialTendonModelInput::with_geometry(b.clone(), bad, g.clone()).is_err());
    }
    for side in [20.0, f32::MAX, f32::NAN] {
        let mut bad = f.clone();
        bad.wrap_prm[1] = side;
        assert!(SpatialTendonModelInput::with_geometry(b.clone(), bad, g.clone()).is_err());
    }
    for side in [-f32::MAX, -1.0, -0.49, 0.49] {
        let mut good = f.clone();
        good.wrap_prm[1] = side;
        SpatialTendonModelInput::with_geometry(b.clone(), good, g.clone()).unwrap();
    }
    for kind in [-1, 0, 5, 9] {
        let mut bad = g.clone();
        bad.geom_type[0] = kind;
        assert!(SpatialTendonModelInput::with_geometry(b.clone(), f.clone(), bad).is_err());
    }
    let mut bad = g.clone();
    bad.geom_type.pop();
    assert!(SpatialTendonModelInput::with_geometry(b.clone(), f.clone(), bad).is_err());
    let mut bad = g.clone();
    bad.geom_size = ParameterBatch::new(1, 11, vec![0.1; 11]).unwrap();
    assert!(SpatialTendonModelInput::with_geometry(b.clone(), f.clone(), bad).is_err());
    let mut bad = g.clone();
    bad.geom_size = ParameterBatch::new(1, 12, vec![-1.0; 12]).unwrap();
    assert!(SpatialTendonModelInput::with_geometry(b.clone(), f.clone(), bad).is_err());
    for slot in [0, 2] {
        let mut bad = f.clone();
        bad.wrap_type[slot] = 4;
        bad.wrap_objid[slot] = 0;
        bad.wrap_prm[slot] = -1.0;
        assert!(SpatialTendonModelInput::with_geometry(b.clone(), bad, g.clone()).is_err());
    }
    let mut bad = f.clone();
    bad.ten_j_colind.remove(2);
    bad.ten_j_rownnz[3] -= 1;
    for a in &mut bad.ten_j_rowadr[4..] {
        *a -= 1;
    }
    assert!(matches!(
        SpatialTendonModelInput::with_geometry(b, bad, g),
        Err(InputError::InvalidTopology {
            reason: "missing_spatial_dof_column",
            ..
        })
    ));
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn wrapped_and_direct_paths_match_native_across_cuda_blocks() {
    reference_hashes_cover_dynamic_sphere_cylinder_and_inside_paths();
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    for worlds in [1, 2, 5, 513] {
        let mut data = p.create_data(worlds).unwrap();
        for round in 0..4 {
            if round > 0 {
                for w in 0..worlds {
                    write(&mut data, w, &v["cases"][3 * round + w % 3]);
                }
            }
            p.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                eprintln!(
                    "G01-geom-tendon kind=native world={w} max_abs_error={:e}",
                    native(&out, w, &v["cases"][3 * round + w % 3])
                );
            }
        }
    }
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn geometry_dofs_match_length_derivatives_in_wrapped_and_direct_states() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    let mut data = p.create_data(513).unwrap();
    for id in [2, 5, 8, 11] {
        write(&mut data, 512, &v["cases"][id]);
        p.update(&mut data).unwrap();
        let base = data.readback().unwrap();
        native(&base, 512, &v["cases"][id]);
        for dof in 0..2 {
            let mut plus = floats(&v["cases"][id], "qpos");
            let mut minus = plus.clone();
            plus[dof] += 0.001;
            minus[dof] -= 0.001;
            data.write_world_qpos(512, &plus).unwrap();
            p.update(&mut data).unwrap();
            let a = data.readback().unwrap();
            data.write_world_qpos(512, &minus).unwrap();
            p.update(&mut data).unwrap();
            let b = data.readback().unwrap();
            let rows = p.spatial_tendon_rows();
            let original = base.spatial_tendon().world(512).unwrap();
            for t in 0..10 {
                let derivative = (f64::from(a.spatial_tendon().world(512).unwrap().ten_length[t])
                    - f64::from(b.spatial_tendon().world(512).unwrap().ten_length[t]))
                    / f64::from(plus[dof] - minus[dof]);
                let moment = rows
                    .row(t)
                    .unwrap()
                    .binary_search(&(dof as i32))
                    .map(|i| original.ten_jacobian[rows.rowadr()[t] as usize + i])
                    .unwrap_or(0.0);
                assert!(
                    (derivative - f64::from(moment)).abs() < 0.003,
                    "case={id} t={t} dof={dof} derivative={derivative} moment={moment}"
                );
            }
        }
    }
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn shrink_grow_readiness_identity_and_snapshot_lifetime_remain_checked() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    let other = plan(&s, &v);
    let mut data = p.create_data(513).unwrap();
    p.update(&mut data).unwrap();
    let saved = data.readback().unwrap();
    assert!(matches!(
        other.update_spatial_tendons(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(data.write_world_qpos(512, &[f32::NAN; 6]).is_err());
    native(&data.readback().unwrap(), 512, &v["cases"][2]);
    for id in [5, 2, 11, 2] {
        write(&mut data, 512, &v["cases"][id]);
        assert!(data.readback().is_err());
        p.update(&mut data).unwrap();
        native(&data.readback().unwrap(), 512, &v["cases"][id]);
    }
    p.update_attached(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "spatial_tendon"
        })
    ));
    p.update_spatial_tendons(&mut data).unwrap();
    p.update_com(&mut data).unwrap();
    assert!(data.readback().is_err());
    p.update_spatial_tendons(&mut data).unwrap();
    drop(p);
    drop(other);
    drop(s);
    native(&saved, 512, &v["cases"][2]);
    native(&data.readback().unwrap(), 512, &v["cases"][2]);
}
fn static_cylinder() -> SpatialTendonModelInput {
    let k = KinematicModelInput::new(
        0,
        KinematicFields {
            body_parentid: vec![0],
            body_jntadr: vec![-1],
            body_jntnum: vec![0],
            body_pos: vec![0.0; 3],
            body_quat: vec![1.0, 0.0, 0.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap();
    let i = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 3],
            body_iquat: vec![1.0, 0.0, 0.0, 0.0],
            body_mass: vec![0.0],
            body_inertia: vec![0.0; 3],
            ..Default::default()
        },
    )
    .unwrap();
    let a = AttachedModelInput::new(
        i,
        AttachedFields {
            geom_bodyid: vec![0],
            geom_pos: vec![0.0; 3],
            geom_quat: vec![1.0, 0.0, 0.0, 0.0],
            site_bodyid: vec![0; 3],
            site_pos: vec![-1.0, 0.0, -0.5, 1.0, 0.0, 0.5, 0.0, 1.0, 0.0],
            site_quat: [1.0, 0.0, 0.0, 0.0].repeat(3),
        },
    )
    .unwrap();
    let c = CamLightModelInput::new(
        MocapModelInput::new(a, vec![-1]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let b = FixedTendonModelInput::new(c, FixedTendonFields::default()).unwrap();
    SpatialTendonModelInput::with_geometry(
        b,
        SpatialTendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![3],
            wrap_type: vec![3, 5, 3],
            wrap_objid: vec![0, 0, 1],
            wrap_prm: vec![0.0, 2.0, 0.0],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![0],
            ten_j_colind: vec![],
        },
        SpatialTendonGeometry {
            geom_type: vec![5],
            geom_size: ParameterBatch::new(
                3,
                3,
                vec![0.0, 0.8, 0.0, 0.25, 0.8, 0.0, 0.3, 0.8, 0.0],
            )
            .unwrap(),
        },
    )
    .unwrap()
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn zero_dof_cylinder_helix_matches_analytic_independent_parameter_periods() {
    let s = TransferSession::new(0).unwrap();
    let mut params = KinematicsParameters::default();
    params.set(
        F::GeomPos,
        ParameterBatch::new(2, 3, vec![0.0, 0.0, 0.125, 0.0, 0.0, -0.125]).unwrap(),
    );
    params.set(
        F::GeomQuat,
        ParameterBatch::new(
            5,
            4,
            (0..5)
                .flat_map(|i| {
                    let a = i as f32 * 0.12;
                    [a.cos(), 0.0, 0.0, a.sin()]
                })
                .collect(),
        )
        .unwrap(),
    );
    params.set(
        F::SitePos,
        ParameterBatch::new(
            7,
            9,
            (0..7)
                .flat_map(|i| {
                    let r = 1.0 + i as f32 * 0.1;
                    [-r, 0.0, -0.5, r, 0.0, 0.5, 0.0, 1.0, 0.0]
                })
                .collect(),
        )
        .unwrap(),
    );
    let p = KinematicsPlan::with_spatial_tendons(
        &s,
        static_cylinder(),
        params,
        CamLightParameters::default(),
    )
    .unwrap();
    for worlds in [1, 2, 5, 513] {
        let mut data = p.create_data(worlds).unwrap();
        for _ in 0..2 {
            p.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                let r = f64::from([0.0f32, 0.25, 0.3][w % 3]);
                let end = f64::from(1.0 + (w % 7) as f32 * 0.1);
                let a = out.spatial_tendon().world(w).unwrap();
                assert!(a.ten_jacobian.is_empty());
                let length = if r == 0.0 {
                    (4.0 * end * end + 1.0).sqrt()
                } else {
                    let flat = 2.0 * (end * end - r * r).sqrt()
                        + r * (std::f64::consts::PI - 2.0 * (r / end).acos());
                    (flat * flat + 1.0).sqrt()
                };
                let mut points = vec![0.0; 18];
                points[..3].copy_from_slice(&[-end, 0.0, -0.5]);
                if r == 0.0 {
                    points[3..6].copy_from_slice(&[end, 0.0, 0.5]);
                    assert_eq!(a.ten_wrapnum, &[2]);
                    assert_eq!(a.wrap_obj, &[-1, -1, 0, 0, 0, 0]);
                } else {
                    let l = (end * end - r * r).sqrt();
                    let flat = 2.0 * l + r * (std::f64::consts::PI - 2.0 * (r / end).acos());
                    let z = -0.5 + l / flat;
                    let x = r * r / end;
                    let y = r * (1.0 - (r / end).powi(2)).sqrt();
                    points[3..9].copy_from_slice(&[-x, y, z, x, y, -z]);
                    points[9..12].copy_from_slice(&[end, 0.0, 0.5]);
                    assert_eq!(a.ten_wrapnum, &[4]);
                    assert_eq!(a.wrap_obj, &[-1, 0, 0, -1, 0, 0]);
                }
                assert_eq!(a.ten_wrapadr, &[0]);
                let error = compare(
                    &[a.ten_length, a.wrap_xpos].concat(),
                    &[vec![length], points].concat(),
                );
                eprintln!("G01-geom-tendon kind=analytic world={w} max_abs_error={error:e}");
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn sphere_parallel_tangent_contained_coincident_and_tiny_radius_keep_frozen_branches() {
    let m = static_cylinder();
    let mut f = m.fields().clone();
    f.wrap_type[1] = 4;
    let geometry = SpatialTendonGeometry {
        geom_type: vec![2],
        geom_size: ParameterBatch::new(
            7,
            3,
            (0..7)
                .flat_map(|i| [if i == 5 { 1e-16 } else { 0.25 }, 0.0, 0.0])
                .collect(),
        )
        .unwrap(),
    };
    let rows = [
        [-1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [-1.0, 0.25, 0.0, 1.0, 0.25, 0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [0.1, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [-1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    ];
    let mut params = KinematicsParameters::default();
    params.set(
        F::SitePos,
        ParameterBatch::new(7, 9, rows.concat()).unwrap(),
    );
    let s = TransferSession::new(0).unwrap();
    let p = KinematicsPlan::with_spatial_tendons(
        &s,
        SpatialTendonModelInput::with_geometry(m.fixed().clone(), f, geometry).unwrap(),
        params,
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = p.create_data(513).unwrap();
    p.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let circle =
        2.0 * (1.0f64 - 0.25 * 0.25).sqrt() + 0.25 * (std::f64::consts::PI - 2.0 * 0.25f64.acos());
    for w in 0..513 {
        let i = w % 7;
        let row = out.spatial_tendon().world(w).unwrap();
        // 冻结f32叉积选中整圆。
        // 不用理想切线替换此分支。
        let tangent = 2.0 + 0.5 * std::f64::consts::PI;
        compare(
            row.ten_length,
            &[[circle, tangent, 1.0, 0.0, 0.9, 2.0, 2.5][i]],
        );
        let active = matches!(i, 0 | 1 | 6);
        assert_eq!(row.ten_wrapnum, &[if active { 4 } else { 2 }]);
        assert_eq!(
            row.wrap_obj,
            if active {
                &[-1, 0, 0, -1, 0, 0]
            } else {
                &[-1, -1, 0, 0, 0, 0]
            }
        );
        if active {
            for point in [&row.wrap_xpos[3..6], &row.wrap_xpos[6..9]] {
                let squared: f64 = point.iter().map(|&v| f64::from(v).powi(2)).sum();
                assert!((squared.sqrt() - 0.25).abs() < 2e-5);
            }
        }
        assert!(row.ten_jacobian.is_empty());
    }
}
