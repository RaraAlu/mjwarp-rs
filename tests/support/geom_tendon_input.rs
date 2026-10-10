//! 两类参考共享完全相同的输入。
use mjwarp_rs::{
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
        FixedTendonFields, FixedTendonModelInput, KinematicsParameters, MocapModelInput,
        ParameterBatch, SpatialTendonFields, SpatialTendonGeometry, SpatialTendonModelInput,
    },
    physics::{KinematicsData, KinematicsPlan},
    runtime::TransferSession,
};
use serde_json::Value;
#[path = "rigid_reference.rs"]
mod reference;
pub use reference::floats;

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../../fixtures/geom-tendon/wrap-tree.json")).unwrap()
}
pub fn motion_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/geom-tendon/inside-cylinder-motion.json"
    ))
    .unwrap()
}
pub fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
pub fn parts(
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
pub fn model(v: &Value) -> SpatialTendonModelInput {
    let (b, f, g) = parts(v);
    SpatialTendonModelInput::with_geometry(b, f, g).unwrap()
}
pub fn plan(s: &TransferSession, v: &Value) -> KinematicsPlan {
    KinematicsPlan::with_spatial_tendons(
        s,
        model(v),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}
pub fn write(data: &mut KinematicsData, w: usize, c: &Value) {
    data.write_world_qpos(w, &floats(c, "qpos")).unwrap();
    data.write_world_mocap(w, &floats(c, "mocap_pos"), &floats(c, "mocap_quat"))
        .unwrap();
}
