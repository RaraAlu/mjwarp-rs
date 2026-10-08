use super::engine;
use engine::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexEdgeFields,
    FlexHessianFields, FlexPositionFields, FlexPositionModelInput, MocapModelInput, TendonFields,
    TendonModelInput,
};
use serde_json::Value;
#[path = "rigid_reference.rs"]
mod rigid;
pub use rigid::floats;
pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../../fixtures/flex-stretch/stretch.json")).unwrap()
}
pub fn ids(v: &Value, k: &str) -> Vec<i32> {
    v[k].as_array()
        .unwrap()
        .iter()
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
pub fn input(v: &Value) -> FlexPositionModelInput {
    let m = &v["model"];
    let a = AttachedModelInput::new(
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
    let t = TendonModelInput::new(
        CamLightModelInput::new(
            MocapModelInput::new(a, ids(m, "body_mocapid")).unwrap(),
            CamLightFields::default(),
        )
        .unwrap(),
        TendonFields::default(),
    )
    .unwrap();
    FlexPositionModelInput::new(
        t,
        FlexPositionFields {
            flex_interp: ids(m, "flex_interp"),
            flex_cellnum: ids(m, "flex_cellnum"),
            flex_nodeadr: ids(m, "flex_nodeadr"),
            flex_nodenum: ids(m, "flex_nodenum"),
            flex_vertadr: ids(m, "flex_vertadr"),
            flex_vertnum: ids(m, "flex_vertnum"),
            flex_centered: ids(m, "flex_centered").iter().map(|&b| b != 0).collect(),
            flex_nodebodyid: ids(m, "flex_nodebodyid"),
            flex_vertbodyid: ids(m, "flex_vertbodyid"),
            flex_node: floats(m, "flex_node"),
            flex_vert: floats(m, "flex_vert"),
            flex_vert0: floats(m, "flex_vert0"),
        },
    )
    .unwrap()
    .with_edges(edges(v))
    .unwrap()
}
pub fn edges(v: &Value) -> FlexEdgeFields {
    let m = &v["model"];
    FlexEdgeFields {
        flex_edgeadr: ids(m, "flex_edgeadr"),
        flex_edgenum: ids(m, "flex_edgenum"),
        flex_edge: ids(m, "flex_edge"),
        flexedge_j_rowadr: ids(m, "flexedge_J_rowadr"),
        flexedge_j_rownnz: ids(m, "flexedge_J_rownnz"),
        flexedge_j_colind: ids(m, "flexedge_J_colind"),
    }
}
pub fn fields(v: &Value) -> FlexHessianFields {
    let m = &v["model"];
    FlexHessianFields {
        flex_dim: ids(m, "flex_dim"),
        flex_rigid: ids(m, "flex_rigid").iter().map(|&b| b != 0).collect(),
        flex_elemadr: ids(m, "flex_elemadr"),
        flex_elemnum: ids(m, "flex_elemnum"),
        flex_elemdataadr: ids(m, "flex_elemdataadr"),
        flex_elemedgeadr: ids(m, "flex_elemedgeadr"),
        flex_stiffnessadr: ids(m, "flex_stiffnessadr"),
        flex_elem: ids(m, "flex_elem"),
        flex_elemedge: ids(m, "flex_elemedge"),
        flexedge_length0: floats(m, "flexedge_length0"),
        flex_stiffness: floats(m, "flex_stiffness"),
    }
}
pub fn compare(a: &[f32], e: &Value) -> f64 {
    let e = e.as_array().unwrap();
    assert_eq!(a.len(), e.len());
    let mut max = 0.0f64;
    for (i, (&a, e)) in a.iter().zip(e).enumerate() {
        let e = e.as_f64().unwrap();
        let error = (f64::from(a) - e).abs();
        max = max.max(error);
        assert!(
            a.is_finite() && error <= 2e-5 + 2e-5 * e.abs(),
            "stretch index={i} actual={a} expected={e} error={error}"
        );
    }
    max
}
