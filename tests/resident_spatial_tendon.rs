#![cfg(feature = "cuda-probe")]
//! G01空间肌腱site子集验收。
//! 产品测试只读取静态参考。

#[path = "resident_spatial_tendon/periods.rs"]
mod periods;
#[path = "resident_spatial_tendon/reference_cases.rs"]
mod reference_cases;
#[path = "resident_spatial_tendon/resident.rs"]
mod resident;
#[path = "resident_spatial_tendon/subsets.rs"]
mod subsets;

use mjwarp_rs::{
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
        FixedTendonFields, FixedTendonModelInput, KinematicsParameters, MocapModelInput,
        SpatialTendonFields, SpatialTendonModelInput,
    },
    physics::{KinematicsData, KinematicsPlan, KinematicsSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
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
