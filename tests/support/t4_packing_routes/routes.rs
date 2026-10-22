use super::super::Stamp;
use super::super::digest;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

const FIXTURE: &str = include_str!("../../../fixtures/precision/g01-packing-routes.json");
const RESIDENT: &str = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/evidence/t4-g03-retest/resident-source-anchor.rs"
));
macro_rules! sources {
    ($($path:literal),+ $(,)?) => {
        &[$(($path, include_str!(concat!("../../../", $path)))),+]
    };
}
const SOURCES: &[(&str, &str)] = sources![
    "src/physics/resident.rs",
    "src/physics/fixed_tendon.rs",
    "src/physics/spatial_tendon.rs",
    "src/physics/tendon.rs",
    "src/physics/flex.rs",
    "src/physics/flex_edge.rs",
    "src/physics/flex_face.rs",
    "src/physics/flex_hessian.rs",
    "src/physics/sleep.rs",
    "src/model/flex.rs",
    "src/model/flex_hessian.rs",
    "src/model/sleep.rs",
    "src/model/fixed_tendon.rs",
    "src/model/spatial_tendon.rs",
    "src/model/tendon.rs",
];
// 独立冻结分母和表示类别。
// 新增路由须重新审查此表。
const PACKERS: &[(&str, &str, &str, &str)] = &[
    (
        "pack_flex_hessian",
        "flex_hessian",
        "src/physics/flex_hessian.rs",
        "header:header:,flex_dim:copy-i32:flex_dim,flex_vertadr:copy-i32:flex_vertadr,flex_vertnum:copy-i32:flex_vertnum,flex_edgeadr:copy-i32:flex_edgeadr,flex_edgenum:copy-i32:flex_edgenum,flex_elemadr:unused-i32:flex_elemadr,flex_elemnum:copy-i32:flex_elemnum,flex_elemdataadr:copy-i32:flex_elemdataadr,flex_elemedgeadr:copy-i32:flex_elemedgeadr,flex_stiffnessadr:copy-i32:flex_stiffnessadr,active:index-derived:flex_interp+flex_rigid+flex_dim+flex_stiffnessadr+flex_stiffness,flex_elem:copy-i32:flex_elem,flex_elemedge:copy-i32:flex_elemedge,flex_edge:copy-i32:flex_edge,flexedge_length0:copy-f32:flexedge_length0,flex_stiffness:copy-f32:flex_stiffness",
    ),
    (
        "pack_flex_faces",
        "flex_faces",
        "src/physics/flex_face.rs",
        "header:header:,normal-axis:index-derived:flex_face_map+flex_cellnum,flex_face:copy-i32:flex_face,unused:sentinel:",
    ),
    (
        "pack_sleep",
        "tendon_wake",
        "src/physics/sleep.rs",
        "header:header:,wake-gate:index-derived:,tendon_adr:copy-i32:tendon_adr,tendon_num:copy-i32:tendon_num,tendon_limited:bool-i32:tendon_limited,wrap_treeid:index-derived:body_treeid+wrap_type+wrap_objid,tendon_range:copy-f32:tendon_range,tendon_margin:copy-f32:tendon_margin,empty-padding:sentinel:",
    ),
    (
        "pack_flex_edges",
        "flex_edges",
        "src/physics/flex_edge.rs",
        "header:header:,vertex0:index-derived:flex_vertadr+flex_edge,vertex1:index-derived:flex_vertadr+flex_edge,flexedge_J_rowadr:copy-i32:flexedge_J_rowadr,flexedge_J_rownnz:copy-i32:flexedge_J_rownnz,flexedge_J_colind:copy-i32:flexedge_J_colind,flex_vertbodyid:copy-i32:flex_vertbodyid,body_parentid:copy-i32:body_parentid,body_rootid:index-derived:body_parentid+body_rootid,dof_bodyid:copy-i32:dof_bodyid",
    ),
    (
        "pack_flex",
        "flex_positions",
        "src/physics/flex.rs",
        "header:header:,flex_interp:copy-i32:flex_interp,flex_cellnum:copy-i32:flex_cellnum,flex_nodeadr:copy-i32:flex_nodeadr,flex_nodenum:copy-i32:flex_nodenum,flex_vertadr:copy-i32:flex_vertadr,flex_vertnum:copy-i32:flex_vertnum,flex_centered:bool-i32:flex_centered,flex_nodebodyid:copy-i32:flex_nodebodyid,flex_vertbodyid:copy-i32:flex_vertbodyid,flex_vert:copy-f32:flex_vert,flex_vert0:copy-f32:flex_vert0,flex_node:copy-f32:flex_node",
    ),
    (
        "pack_tendon",
        "tendon_values",
        "src/physics/tendon.rs",
        "header:header:,subset-kind:index-derived:,subset-local:index-derived:,subset-offset:index-derived:ten_J_rowadr,ten_J_rowadr:copy-i32:ten_J_rowadr,ten_J_rownnz:copy-i32:ten_J_rownnz,unused:sentinel:",
    ),
    (
        "pack_fixed_tendon",
        "fixed_tendon",
        "src/physics/fixed_tendon.rs",
        "header:header:,tendon_adr:copy-i32:tendon_adr,tendon_num:copy-i32:tendon_num,wrap_prm:copy-f32:wrap_prm,qpos-address:index-derived:wrap_objid+jnt_qposadr,jacobian-slot:index-derived:jnt_dofadr+ten_J_rowadr+ten_J_rownnz+ten_J_colind",
    ),
    (
        "pack_spatial_tendon",
        "spatial_site",
        "src/physics/spatial_tendon.rs",
        "header:header:,body_parentid:copy-i32:body_parentid,body_rootid:index-derived:body_parentid+body_rootid,body_dofadr:index-derived:body_jntadr+jnt_dofadr+body_dofadr,body_dofnum:index-derived:body_jntadr+body_jntnum+jnt_type+body_dofnum,tendon_adr:copy-i32:tendon_adr,tendon_num:copy-i32:tendon_num,ten_J_rowadr:copy-i32:ten_J_rowadr,ten_J_rownnz:copy-i32:ten_J_rownnz,wrap_type:copy-i32:wrap_type,wrap_objid:copy-i32:wrap_objid,wrap-body:index-derived:wrap_type+wrap_objid+site_bodyid+geom_bodyid,side-site:index-derived:wrap_type+wrap_prm,ten_J_colind:copy-i32:ten_J_colind,pulley-scale:f32-precompute:wrap_type+wrap_prm,geom_size:copy-f32:geom_size",
    ),
];
const REMAINING: [&str; 4] = [
    "rigid-camlight-integer-topology",
    "other-physics-internals-and-outputs",
    "upstream-generated-kernels",
    "measured-performance",
];
// 本库按肌腱串行累加。
// 上游按包裹对象原子累加。
const FIXED_INTERNAL: &[(&str, &str)] = &[
    ("float sum=0.0f;", "ten_length_out: wp.array2d[float],"),
    (
        "sum+=coefficients[i]*qpos[qadr[i]];",
        "L = prm * qpos_in[worldid, jnt_qposadr[wrapobjid]]",
    ),
    (
        "jac[jslot[i]]=coefficients[i];",
        "ten_J_out[worldid, rowadr + k] = prm",
    ),
    (
        "length[t]=sum;",
        "wp.atomic_add(ten_length_out[worldid], tenid, L)",
    ),
];
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Upstream {
    name: String,
    declaration: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    id: String,
    kind: String,
    scalar: String,
    upstream: Vec<Upstream>,
    write: String,
    read: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packer {
    name: String,
    entry: String,
    file: String,
    routes: Vec<Route>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    product_revision: String,
    upstream_revision: String,
    product_sources: Vec<Stamp>,
    packers: Vec<Packer>,
    np01_complete: bool,
    remaining: Vec<String>,
}

#[path = "routes/checks.rs"]
mod checks;
use checks::*;

#[test]
fn covers_eight_packers_and_all_82_routes() {
    validate(&inventory()).unwrap();
    assert_eq!(
        inventory()
            .packers
            .iter()
            .map(|p| p.routes.len())
            .sum::<usize>(),
        82
    );
}
#[test]
fn rejects_every_missing_duplicated_or_mistyped_route() {
    for (p, &(_, _, _, expected)) in inventory().packers.iter().zip(PACKERS) {
        for index in 0..p.routes.len() {
            for mutation in 0..8 {
                let mut changed = p.clone();
                match mutation {
                    0 => {
                        changed.routes.remove(index);
                    }
                    1 => changed.routes[index] = p.routes[(index + 1) % p.routes.len()].clone(),
                    2 => changed.routes[index].scalar = "f64".into(),
                    3 => changed.routes[index].kind = "equivalent".into(),
                    4 => changed.routes[index].write = "wrong_writer".into(),
                    5 => changed.routes[index].read = "wrong_reader".into(),
                    6 if !changed.routes[index].upstream.is_empty() => {
                        changed.routes[index].upstream.pop();
                    }
                    _ => changed.routes[index].upstream.push(Upstream {
                        name: "bad".into(),
                        declaration: "f64".into(),
                    }),
                }
                assert!(
                    validate_routes(
                        &changed,
                        expected,
                        &RESIDENT.replace("\r\n", "\n"),
                        cuda(&p.file).unwrap()
                    )
                    .is_err(),
                    "{} route {index} mutation {mutation}",
                    p.name
                );
            }
        }
    }
}
#[test]
fn rejects_source_coverage_and_false_completion() {
    for index in 0..8 {
        let mut i = inventory();
        i.packers.remove(index);
        assert!(validate(&i).is_err());
    }
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    let mut i = inventory();
    i.np01_complete = true;
    assert!(validate(&i).is_err());
    i.np01_complete = false;
    i.remaining.clear();
    assert!(validate(&i).is_err());
}
#[test]
fn rejects_fixed_accumulator_and_unused_slot_drift() {
    let fixed = cuda("src/physics/fixed_tendon.rs").unwrap();
    for &(anchor, _) in FIXED_INTERNAL {
        assert!(verify_fixed(&fixed.replace(anchor, "changed_expression")).is_err());
    }
    assert!(verify_fixed(&fixed.replace("float sum", "double sum")).is_err());
    let i = inventory();
    let route = i.packers[0]
        .routes
        .iter()
        .find(|r| r.kind == "unused-i32")
        .unwrap();
    assert!(validate_reader(route, "int changed=r[6];").is_err());
}
#[test]
#[ignore = "reads frozen sources and existing GPU captures without executing them"]
fn audit_remaining_packing_sources_and_captures() {
    let i = inventory();
    validate(&i).unwrap();
    let root = std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream root");
    let types = std::fs::read_to_string(Path::new(&root).join("_src/types.py")).unwrap();
    let smooth = std::fs::read_to_string(Path::new(&root).join("_src/smooth.py"))
        .unwrap()
        .replace("\r\n", "\n");
    let warp_root = std::env::var_os("MJWARP_FROZEN_WARP_ROOT").expect("set Warp root");
    let warp = std::fs::read_to_string(Path::new(&warp_root).join("_src/types.py")).unwrap();
    verify_upstream(&i, &types, &warp, &smooth).unwrap();
    let mut changed = i.clone();
    changed.packers[0].routes[1].upstream[0].declaration = "array(\"nflex\", float)".into();
    assert!(verify_upstream(&changed, &types, &warp, &smooth).is_err());
    assert!(verify_upstream(&i, &types, &warp, &(smooth.clone() + "\n")).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("set evidence root");
    let captures = verify_captures(&i, Path::new(&root)).unwrap();
    let fields: BTreeSet<_> = i
        .packers
        .iter()
        .flat_map(|p| p.routes.iter())
        .flat_map(|r| r.upstream.iter().map(|u| u.name.as_str()))
        .collect();
    let report = serde_json::json!({
        "schema":1, "scope":i.scope, "inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision, "upstream_revision":i.upstream_revision,
        "packing_functions":8, "packing_routes":82, "upstream_fields":fields.len(),
        "fixed_internal_groups":4, "captured_variants":captures,
        "gpu_executions":0, "python_executions":0, "np01_complete":false, "np02_complete":false,
        "np03":"historical-unchanged", "np04":"not-measured", "remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_PACKING_ROUTES_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
