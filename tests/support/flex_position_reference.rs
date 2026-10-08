//! G01柔体位置共用静态DTO。
use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexPositionFields,
    MocapModelInput, TendonFields, TendonModelInput,
};
use serde_json::Value;
#[path = "rigid_reference.rs"]
mod rigid;
pub use rigid::floats;

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../../fixtures/flex-position/flex-tree.json")).unwrap()
}
pub fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
pub fn tendons(v: &Value) -> TendonModelInput {
    let m = &v["model"];
    let base = AttachedModelInput::new(
        rigid::model(v),
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
    TendonModelInput::new(
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
    )
    .unwrap()
}
pub fn fields(v: &Value) -> FlexPositionFields {
    let m = &v["model"];
    FlexPositionFields {
        flex_interp: ids(m, "flex_interp"),
        flex_cellnum: ids(m, "flex_cellnum"),
        flex_nodeadr: ids(m, "flex_nodeadr"),
        flex_nodenum: ids(m, "flex_nodenum"),
        flex_vertadr: ids(m, "flex_vertadr"),
        flex_vertnum: ids(m, "flex_vertnum"),
        flex_centered: ids(m, "flex_centered").iter().map(|&i| i != 0).collect(),
        flex_nodebodyid: ids(m, "flex_nodebodyid"),
        flex_vertbodyid: ids(m, "flex_vertbodyid"),
        flex_node: floats(m, "flex_node"),
        flex_vert: floats(m, "flex_vert"),
        flex_vert0: floats(m, "flex_vert0"),
    }
}
