#![cfg(feature = "cuda-probe")]
//! G01空间肌腱site子集验收。
//! 产品测试只读取静态参考。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
        FixedTendonFields, FixedTendonModelInput, KinematicsParameter as F, KinematicsParameters,
        MocapModelInput, ParameterBatch, SpatialTendonFields, SpatialTendonModelInput,
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
    serde_json::from_str(include_str!("../fixtures/spatial-tendon/mixed-tree.json")).unwrap()
}
fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}
fn model(v: &Value) -> SpatialTendonModelInput {
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
    let fixed = FixedTendonModelInput::new(cam, FixedTendonFields::default()).unwrap();
    SpatialTendonModelInput::new(
        fixed,
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
    )
    .unwrap()
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
fn flattened(out: &KinematicsSnapshot, w: usize) -> Vec<f32> {
    let r = out.spatial_tendon().world(w).unwrap();
    [r.ten_length, r.ten_jacobian, r.wrap_xpos].concat()
}
fn expected(case: &Value) -> Vec<f64> {
    ["ten_length", "ten_J", "wrap_xpos"]
        .into_iter()
        .flat_map(|key| {
            case[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
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
            "G01 spatial tendon index={index} actual={a} expected={e} error={error}"
        );
    }
    max
}
fn compare_native(out: &KinematicsSnapshot, w: usize, case: &Value) -> f64 {
    let r = out.spatial_tendon().world(w).unwrap();
    assert_eq!(r.ten_wrapadr, ids(case, "ten_wrapadr"));
    assert_eq!(r.ten_wrapnum, ids(case, "ten_wrapnum"));
    assert_eq!(r.wrap_obj, ids(case, "wrap_obj"));
    compare(&flattened(out, w), &expected(case))
}
fn write_case(data: &mut KinematicsData, world: usize, case: &Value) {
    data.write_world_qpos(world, &floats(case, "qpos")).unwrap();
    data.write_world_mocap(
        world,
        &floats(case, "mocap_pos"),
        &floats(case, "mocap_quat"),
    )
    .unwrap();
}
#[test]
fn reference_hashes_and_site_pulley_paths_match() {
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/spatial-tendon/manifest.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/spatial-tendon");
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
    assert_eq!(
        (m.rows().ntendon(), m.rows().nnz(), m.rows().nwrap()),
        (5, 45, 19)
    );
    assert!(m.rows().row(4).unwrap().is_empty());
    assert!(m.rows().row(5).is_err());
    assert_eq!(m.fields().wrap_type.iter().filter(|&&t| t == 2).count(), 2);
    assert_eq!(v["cases"].as_array().unwrap().len(), 8);
    for c in v["cases"].as_array().unwrap() {
        for key in ["qpos", "mocap_pos", "mocap_quat"] {
            for value in c[key].as_array().unwrap() {
                let n = value.as_f64().unwrap();
                assert!((n - f64::from(n as f32)).abs() <= f64::EPSILON * n.abs());
            }
        }
    }
}
#[test]
fn rejects_invalid_lengths_wraps_branches_and_pulley_divisors() {
    let m = model(&fixture());
    for which in 0..8 {
        let mut f = m.fields().clone();
        match which {
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
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    for kind in [0, 1, 4, 5, -1, i32::MAX] {
        let mut f = m.fields().clone();
        f.wrap_type[0] = kind;
        assert!(matches!(
            SpatialTendonModelInput::new(m.fixed().clone(), f),
            Err(InputError::InvalidTopology {
                field: "wrap_type",
                ..
            })
        ));
    }
    for id in [-1, 9, i32::MAX] {
        let mut f = m.fields().clone();
        f.wrap_objid[0] = id;
        assert!(matches!(
            SpatialTendonModelInput::new(m.fixed().clone(), f),
            Err(InputError::InvalidTopology {
                field: "wrap_objid",
                ..
            })
        ));
    }
    for value in [0.0, -0.0, -1.0, f32::NAN, f32::INFINITY, f32::from_bits(1)] {
        let mut f = m.fields().clone();
        f.wrap_prm[7] = value;
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    for slot in [1, 4, 8] {
        let mut f = m.fields().clone();
        f.wrap_type[slot] = 2;
        f.wrap_prm[slot] = 2.0;
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    // Pulley object IDs do not refer to a site and never form an address.
    let mut f = m.fields().clone();
    f.wrap_objid[7] = i32::MAX;
    SpatialTendonModelInput::new(m.fixed().clone(), f).unwrap();
}
#[test]
fn rejects_malformed_sparse_rows_and_missing_nonshared_dofs() {
    let m = model(&fixture());
    for column in [-1, 1, 12, i32::MAX] {
        let mut f = m.fields().clone();
        f.ten_j_colind[0] = column;
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    for (csr, value) in [(true, -1), (true, 1), (false, -1), (false, 1)] {
        let mut f = m.fields().clone();
        if csr {
            f.ten_j_rowadr[0] = value;
        } else {
            f.tendon_adr[0] = value;
        }
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    let mut f = m.fields().clone();
    f.ten_j_colind.remove(11);
    f.ten_j_rownnz[0] -= 1;
    for start in &mut f.ten_j_rowadr[1..] {
        *start -= 1;
    }
    assert!(matches!(
        SpatialTendonModelInput::new(m.fixed().clone(), f),
        Err(InputError::InvalidTopology {
            reason: "missing_spatial_dof_column",
            ..
        })
    ));
    // Shared free-body motion cancels from the shared-root path.
    let mut f = m.fields().clone();
    f.ten_j_colind.drain(34..40);
    f.ten_j_rownnz[3] -= 6;
    f.ten_j_rowadr[4] -= 6;
    SpatialTendonModelInput::new(m.fixed().clone(), f).unwrap();
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn resident_spatial_tendons_match_native_across_cuda_blocks() {
    reference_hashes_and_site_pulley_paths_match();
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let a = plan(&s, &v);
    for worlds in [1, 2, 5, 513] {
        let mut data = a.create_data(worlds).unwrap();
        for round in 0..4 {
            if round > 0 {
                let cases: Vec<_> = (0..worlds).map(|w| &v["cases"][(w + round) % 8]).collect();
                data.write_qpos(
                    &cases
                        .iter()
                        .flat_map(|c| floats(c, "qpos"))
                        .collect::<Vec<_>>(),
                )
                .unwrap();
                data.write_mocap(
                    &cases
                        .iter()
                        .flat_map(|c| floats(c, "mocap_pos"))
                        .collect::<Vec<_>>(),
                    &cases
                        .iter()
                        .flat_map(|c| floats(c, "mocap_quat"))
                        .collect::<Vec<_>>(),
                )
                .unwrap();
            }
            a.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            assert_eq!(out.spatial_tendon().rows(), a.spatial_tendon_rows());
            for world in 0..worlds {
                let id = if round == 0 { 0 } else { (world + round) % 8 };
                eprintln!(
                    "G01-spatial-tendon kind=native world={world} max_abs_error={:e}",
                    compare_native(&out, world, &v["cases"][id])
                );
            }
        }
    }
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn staleness_identity_mocap_and_world_writes_preserve_snapshots() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let a = plan(&s, &v);
    let b = plan(&s, &v);
    let mut data = a.create_data(513).unwrap();
    let mut second = a.create_data(2).unwrap();
    assert!(matches!(
        b.update_spatial_tendons(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(matches!(
        a.update_spatial_tendons(&mut data),
        Err(TransferError::StageNotReady { stage: "attached" })
    ));
    a.update_rigid(&mut data).unwrap();
    a.update_attached(&mut data).unwrap();
    assert!(matches!(
        a.update_spatial_tendons(&mut data),
        Err(TransferError::StageNotReady { stage: "com" })
    ));
    a.update_com(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "spatial_tendon"
        })
    ));
    a.update_spatial_tendons(&mut data).unwrap();
    a.update(&mut second).unwrap();
    let saved = data.readback().unwrap();
    assert!(
        data.write_world_mocap(513, &[0.0; 3], &[1.0, 0.0, 0.0, 0.0])
            .is_err()
    );
    assert!(
        data.write_world_mocap(512, &[f32::NAN; 3], &[1.0, 0.0, 0.0, 0.0])
            .is_err()
    );
    assert!(data.write_world_qpos(512, &[]).is_err());
    assert_eq!(
        flattened(&saved, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    for (world, id) in [(0, 1), (256, 4), (512, 7)] {
        write_case(&mut data, world, &v["cases"][id]);
    }
    assert!(data.readback().is_err());
    a.update(&mut data).unwrap();
    let updated = data.readback().unwrap();
    for w in 0..513 {
        compare_native(
            &updated,
            w,
            &v["cases"][match w {
                0 => 1,
                256 => 4,
                512 => 7,
                _ => 0,
            }],
        );
    }
    for com in [false, true] {
        if com {
            a.update_com(&mut data).unwrap();
        } else {
            a.update_attached(&mut data).unwrap();
        }
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady {
                stage: "spatial_tendon"
            })
        ));
        a.update_spatial_tendons(&mut data).unwrap();
    }
    a.update_fixed_tendons(&mut data).unwrap();
    a.update_camlight(&mut data).unwrap();
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    drop(b);
    drop(a);
    drop(s);
    compare_native(&second.readback().unwrap(), 1, &v["cases"][0]);
    compare_native(&saved, 512, &v["cases"][0]);
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
}

fn vec3(v: &[f32]) -> [f64; 3] {
    std::array::from_fn(|i| f64::from(v[i]))
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
// Independent sparse oracle using immutable GPU FK/COM snapshots as inputs.
fn oracle(out: &KinematicsSnapshot, m: &SpatialTendonModelInput, w: usize) -> Vec<f64> {
    let f = m.fields();
    let base = m.fixed().camlight().mocap().attached();
    let k = base.rigid().kinematics().fields();
    let dofs = &base.rigid().fields().dof_bodyid;
    let site = out.attached().world(w).unwrap().site_xpos;
    let com = out.com().world(w).unwrap();
    let mut length = vec![0.0; m.rows().ntendon()];
    let mut jac = vec![0.0; m.rows().nnz()];
    let mut points = vec![0.0; 6 * m.rows().nwrap()];
    for (i, &kind) in f.wrap_type.iter().enumerate() {
        if kind == 3 {
            let id = f.wrap_objid[i] as usize;
            points[3 * i..3 * i + 3].copy_from_slice(&vec3(&site[3 * id..3 * id + 3]));
        }
    }
    for (t, value) in length.iter_mut().enumerate() {
        let start = f.tendon_adr[t] as usize;
        let end = start + f.tendon_num[t] as usize;
        let mut scale = 1.0;
        for i in start..end {
            if f.wrap_type[i] == 2 {
                scale = 1.0 / f64::from(f.wrap_prm[i]);
                continue;
            }
            if i + 1 == end || f.wrap_type[i + 1] != 3 {
                continue;
            }
            let p0: [f64; 3] = points[3 * i..3 * i + 3].try_into().unwrap();
            let p1: [f64; 3] = points[3 * i + 3..3 * i + 6].try_into().unwrap();
            let dif = sub(p1, p0);
            let norm = dif.iter().map(|v| v * v).sum::<f64>().sqrt();
            *value += norm * scale;
            let dir = if norm < 1e-15 {
                [1.0, 0.0, 0.0]
            } else {
                dif.map(|v| v / norm)
            };
            let b0 = base.fields().site_bodyid[f.wrap_objid[i] as usize] as usize;
            let b1 = base.fields().site_bodyid[f.wrap_objid[i + 1] as usize] as usize;
            if b0 == b1 {
                continue;
            }
            for (body, p, sign) in [(b0, p0, -1.0), (b1, p1, 1.0)] {
                let mut root = body;
                while root > 0 && k.body_parentid[root] > 0 {
                    root = k.body_parentid[root] as usize;
                }
                let offset = sub(p, vec3(&com.subtree_com[3 * root..3 * root + 3]));
                for (slot, &dof) in m.rows().row(t).unwrap().iter().enumerate() {
                    let dof = dof as usize;
                    let mut ancestor = body;
                    let mut found = false;
                    while ancestor > 0 {
                        if ancestor == dofs[dof] as usize {
                            found = true;
                            break;
                        }
                        ancestor = k.body_parentid[ancestor] as usize;
                    }
                    if found {
                        let c = &com.cdof[6 * dof..6 * dof + 6];
                        let v = cross(vec3(c), offset);
                        let j: f64 = (0..3)
                            .map(|axis| (f64::from(c[3 + axis]) + v[axis]) * dir[axis])
                            .sum();
                        jac[m.rows().rowadr()[t] as usize + slot] += sign * scale * j;
                    }
                }
            }
        }
    }
    [length, jac, points].concat()
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn independent_field_periods_feed_site_paths_and_sparse_moments() {
    let v = fixture();
    let m = model(&v);
    let s = TransferSession::new(0).unwrap();
    let mut parameters = KinematicsParameters::default();
    let mut site = Vec::new();
    for row in 0..3 {
        let mut values = floats(&v["model"], "site_pos");
        for chunk in values.as_chunks_mut::<3>().0 {
            chunk[0] += row as f32 * 0.125;
        }
        site.extend(values);
    }
    parameters.set(F::SitePos, ParameterBatch::new(3, 27, site).unwrap());
    let mut mass = floats(&v["model"], "body_mass");
    mass.extend(vec![0.0; 7]);
    parameters.set(F::BodyMass, ParameterBatch::new(2, 7, mass).unwrap());
    let mut ipos = Vec::new();
    for row in 0..5 {
        let mut values = floats(&v["model"], "body_ipos");
        for chunk in values[3..].as_chunks_mut::<3>().0 {
            chunk[1] += row as f32 * 0.0625;
        }
        ipos.extend(values);
    }
    parameters.set(F::BodyIpos, ParameterBatch::new(5, 21, ipos).unwrap());
    let a = KinematicsPlan::with_spatial_tendons(
        &s,
        m.clone(),
        parameters,
        CamLightParameters::default(),
    )
    .unwrap();
    for worlds in [1, 2, 5, 513] {
        let mut data = a.create_data(worlds).unwrap();
        for round in 0..2 {
            if round > 0 {
                for w in 0..worlds {
                    write_case(&mut data, w, &v["cases"][(w + 1) % 8]);
                }
            }
            a.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                eprintln!(
                    "G01-spatial-tendon kind=parameters world={w} max_abs_error={:e}",
                    compare(&flattened(&out, w), &oracle(&out, &m, w))
                );
                if w % 2 == 1 {
                    assert_eq!(out.com().world(w).unwrap().subtree_mass, &[0.0; 7]);
                }
            }
        }
    }
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn sparse_moments_match_length_derivatives_and_shared_column_pruning() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let m = model(&v);
    let mut f = m.fields().clone();
    f.ten_j_colind.drain(34..40);
    f.ten_j_rownnz[3] -= 6;
    f.ten_j_rowadr[4] -= 6;
    // Add a legal zero column to the static path.
    f.ten_j_colind.push(0);
    f.ten_j_rownnz[4] = 1;
    let m = SpatialTendonModelInput::new(m.fixed().clone(), f).unwrap();
    let a = KinematicsPlan::with_spatial_tendons(
        &s,
        m.clone(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = a.create_data(513).unwrap();
    write_case(&mut data, 512, &v["cases"][2]);
    a.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    compare(&flattened(&out, 512), &oracle(&out, &m, 512));
    for (qadr, dof) in [(11, 9), (12, 10), (13, 11)] {
        let mut q = floats(&v["cases"][2], "qpos");
        let old = q[qadr];
        q[qadr] += 0.001;
        data.write_world_qpos(512, &q).unwrap();
        a.update(&mut data).unwrap();
        let next = data.readback().unwrap();
        for t in 0..5 {
            let original = out.spatial_tendon().world(512).unwrap();
            let next = next.spatial_tendon().world(512).unwrap();
            let derivative = (f64::from(next.ten_length[t]) - f64::from(original.ten_length[t]))
                / (f64::from(q[qadr]) - f64::from(old));
            let moment = m
                .rows()
                .row(t)
                .unwrap()
                .binary_search(&dof)
                .map(|slot| original.ten_jacobian[m.rows().rowadr()[t] as usize + slot])
                .unwrap_or(0.0);
            assert!(
                (derivative - f64::from(moment)).abs() < 0.003,
                "t={t} dof={dof} derivative={derivative} moment={moment}"
            );
        }
    }
    assert_eq!(
        *out.spatial_tendon()
            .world(512)
            .unwrap()
            .ten_jacobian
            .last()
            .unwrap(),
        0.0
    );
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn all_six_resident_subsets_share_one_update_without_tendon_index_aliasing() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let m = model(&v);
    let cam = CamLightModelInput::new(
        m.fixed().camlight().mocap().clone(),
        CamLightFields {
            cam_mode: vec![0],
            cam_bodyid: vec![0],
            cam_targetbodyid: vec![-1],
            cam_pos: vec![1.0, 2.0, 3.0],
            cam_quat: vec![1.0, 0.0, 0.0, 0.0],
            cam_poscom0: vec![0.0; 3],
            cam_pos0: vec![0.0; 3],
            cam_mat0: vec![0.0; 9],
            light_mode: vec![0],
            light_bodyid: vec![0],
            light_targetbodyid: vec![-1],
            light_pos: vec![3.0, 2.0, 1.0],
            light_dir: vec![0.0, 0.0, 1.0],
            light_poscom0: vec![0.0; 3],
            light_pos0: vec![0.0; 3],
            light_dir0: vec![0.0; 3],
        },
    )
    .unwrap();
    let fixed = FixedTendonModelInput::new(
        cam,
        FixedTendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![1],
            wrap_type: vec![1],
            wrap_objid: vec![2],
            wrap_prm: vec![2.0],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![1],
            ten_j_colind: vec![9],
        },
    )
    .unwrap();
    let plan = KinematicsPlan::with_spatial_tendons(
        &s,
        SpatialTendonModelInput::new(fixed, m.fields().clone()).unwrap(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    for w in [0, 256, 512] {
        write_case(&mut data, w, &v["cases"][w % 8]);
    }
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    for w in [0, 256, 512] {
        let case = &v["cases"][w % 8];
        compare_native(&out, w, case);
        let fixed = out.fixed_tendon().world(w).unwrap();
        assert_eq!(fixed.ten_length, &[2.0 * floats(case, "qpos")[11]]);
        assert_eq!(fixed.ten_jacobian, &[2.0]);
        let cam = out.camlight().world(w).unwrap();
        assert_eq!(cam.cam_xpos, &[1.0, 2.0, 3.0]);
        assert_eq!(cam.light_xpos, &[3.0, 2.0, 1.0]);
        assert_eq!(cam.light_xdir, &[0.0, 0.0, 1.0]);
    }
    assert_eq!(out.fixed_tendon().rows().ntendon(), 1);
    assert_eq!(out.spatial_tendon().rows().ntendon(), 5);
}
