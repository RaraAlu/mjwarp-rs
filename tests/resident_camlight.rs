#![cfg(feature = "cuda-probe")]
//! G01相机与光源辅助验收。
//! 产品测试只读取静态参考。

#[path = "resident_camlight/cases.rs"]
mod cases;
#[path = "resident_camlight/edge.rs"]
mod edge;
#[path = "resident_camlight/oracle.rs"]
mod oracle;
#[path = "resident_camlight/periods.rs"]
mod periods;
#[path = "resident_camlight/readiness.rs"]
mod readiness;
#[path = "support/rigid_reference.rs"]
mod reference;

use mjwarp_rs::{
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, MocapModelInput,
    },
    physics::KinematicsSnapshot,
};
use reference::floats;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/camlight/mixed-tree.json")).unwrap()
}
fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}
fn model(v: &Value) -> CamLightModelInput {
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
    let base = MocapModelInput::new(attached, ids(m, "body_mocapid")).unwrap();
    CamLightModelInput::new(
        base,
        CamLightFields {
            cam_mode: ids(m, "cam_mode"),
            cam_bodyid: ids(m, "cam_bodyid"),
            cam_targetbodyid: ids(m, "cam_targetbodyid"),
            cam_pos: floats(m, "cam_pos"),
            cam_quat: floats(m, "cam_quat"),
            cam_poscom0: floats(m, "cam_poscom0"),
            cam_pos0: floats(m, "cam_pos0"),
            cam_mat0: floats(m, "cam_mat0"),
            light_mode: ids(m, "light_mode"),
            light_bodyid: ids(m, "light_bodyid"),
            light_targetbodyid: ids(m, "light_targetbodyid"),
            light_pos: floats(m, "light_pos"),
            light_dir: floats(m, "light_dir"),
            light_poscom0: floats(m, "light_poscom0"),
            light_pos0: floats(m, "light_pos0"),
            light_dir0: floats(m, "light_dir0"),
        },
    )
    .unwrap()
}
fn flattened(out: &KinematicsSnapshot, w: usize) -> Vec<f32> {
    let c = out.camlight().world(w).unwrap();
    [c.cam_xpos, c.cam_xmat, c.light_xpos, c.light_xdir].concat()
}
fn compare(actual: &[f32], expected: &[f64]) -> f64 {
    assert_eq!(actual.len(), expected.len());
    let mut max: f64 = 0.0;
    for (index, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        let error = (f64::from(a) - e).abs();
        max = max.max(error);
        assert!(
            a.is_finite() && e.is_finite() && error <= 2e-5 + 2e-5 * e.abs(),
            "G01 camlight index={index} actual={a} expected={e} error={error}"
        );
    }
    max
}
fn native_expected(case: &Value) -> Vec<f64> {
    ["cam_xpos", "cam_xmat", "light_xpos", "light_xdir"]
        .into_iter()
        .flat_map(|f| {
            case[f]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
        })
        .collect()
}
fn write_cases(data: &mut mjwarp_rs::physics::KinematicsData, v: &Value, round: usize) {
    let selected: Vec<_> = (0..data.worlds())
        .map(|w| &v["cases"][(w + round) % 8])
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
}
