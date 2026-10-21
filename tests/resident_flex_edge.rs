#![cfg(feature = "cuda-probe")]
//! G01柔体边子集验收。
//! 产品测试只读静态原生参考。
use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexEdgeFields,
    FlexPositionFields, FlexPositionModelInput, MocapModelInput, TendonFields, TendonModelInput,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;
fn base() -> Value {
    serde_json::from_str(include_str!("../fixtures/flex-position/flex-tree.json")).unwrap()
}
fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/flex-edge/edge-state.json")).unwrap()
}
fn ids(v: &Value, k: &str) -> Vec<i32> {
    v[k].as_array()
        .unwrap()
        .iter()
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
fn tendons(v: &Value) -> TendonModelInput {
    let m = &v["model"];
    let a = AttachedModelInput::new(
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
        MocapModelInput::new(a, ids(m, "body_mocapid")).unwrap(),
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
fn positions(v: &Value) -> FlexPositionFields {
    let m = &v["model"];
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
    }
}
fn edges(v: &Value) -> FlexEdgeFields {
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
fn input(b: &Value, e: &Value) -> FlexPositionModelInput {
    FlexPositionModelInput::new(tendons(b), positions(b))
        .unwrap()
        .with_edges(edges(e))
        .unwrap()
}
#[test]
fn reference_hashes_pin_native_edges_and_base_inputs() {
    let m: Value =
        serde_json::from_str(include_str!("../fixtures/flex-edge/manifest.json")).unwrap();
    for f in m["files"].as_array().unwrap() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/flex-edge")
            .join(f["path"].as_str().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(p).unwrap())),
            f["sha256"].as_str().unwrap()
        );
    }
    let b = base();
    let e = fixture();
    let m = input(&b, &e);
    let f = m.edges().unwrap();
    assert_eq!((f.nflexedge(), f.nnz()), (284, 10));
    assert_eq!(&f.flexedge_j_rownnz[..6], &[1, 2, 1, 3, 2, 1]);
    assert!(f.flexedge_j_rownnz[6..].iter().all(|&n| n == 0));
    for c in 0..12 {
        for field in ["qpos", "mocap_pos", "mocap_quat"] {
            assert_eq!(b["cases"][c][field], e["cases"][c][field]);
        }
    }
}
#[test]
fn rejects_invalid_edge_partitions_endpoints_and_sparse_rows() {
    let b = base();
    let e = fixture();
    let f = edges(&e);
    let check = |f| {
        FlexPositionModelInput::new(tendons(&b), positions(&b))
            .unwrap()
            .with_edges(f)
    };
    let mut cases = Vec::new();
    let mut x = f.clone();
    x.flex_edgeadr[1] = 0;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edgenum[0] = -1;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edgenum[3] -= 1;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edge[0] = -1;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edge[0] = 5;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edge.pop();
    cases.push(x);
    let mut x = f.clone();
    x.flex_edgeadr.clear();
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rowadr[1] = 0;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rowadr[6] = 11;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rowadr.pop();
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rownnz[0] = -1;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rownnz[6] = 1;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind[0] = 1;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind[0] = -1;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind[0] = 65;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind[2] = 0;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind.push(65);
    cases.push(x);
    for x in cases {
        assert!(check(x).is_err());
    }
    let mut zero = f;
    zero.flexedge_j_rownnz.fill(0);
    zero.flexedge_j_rowadr.fill(0);
    zero.flexedge_j_colind.clear();
    check(zero).unwrap();
    let mut reserved = edges(&e);
    reserved.flexedge_j_rownnz.fill(0);
    reserved.flexedge_j_rowadr.fill(0);
    check(reserved).unwrap();
    FlexPositionModelInput::new(tendons(&b), FlexPositionFields::default())
        .unwrap()
        .with_edges(FlexEdgeFields::default())
        .unwrap();
}

#[path = "resident_flex_edge/gpu.rs"]
mod gpu;
