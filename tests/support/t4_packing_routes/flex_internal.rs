use super::super::Stamp;
use super::super::digest;
use serde::Deserialize;
use std::path::Path;

const FIXTURE: &str = include_str!("../../../docs/evidence/t4-g03-retest/flex-internal/map.json");
const FLEX: &str = include_str!("../../../src/physics/flex.rs");
const EDGE: &str = include_str!("../../../src/physics/flex_edge.rs");
use super::RESIDENT;
const MODEL: &str = include_str!("../../../src/model/flex.rs");
const TRANSFER: &str = include_str!("../../../src/runtime/transfer.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/flex.rs", FLEX),
    ("src/physics/flex_edge.rs", EDGE),
    ("src/physics/resident.rs", RESIDENT),
    ("src/model/flex.rs", MODEL),
    ("src/runtime/transfer.rs", TRANSFER),
];
const EXTERNAL: &[&str] = &[
    "upstream/_src/smooth.py",
    "upstream/_src/math.py",
    "upstream/_src/support.py",
    "upstream/_src/types.py",
    "warp/_src/types.py",
];
const STEPS: &[&str] = &[
    "node-transform|same-scalar-order-unverified|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_nodes|wp.vec3|3|3",
    "vertex-direct|same-scalar-order-unverified|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_vertices|wp.vec3|1|2",
    "cell-lookup|f32-to-i32|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_vertices|float|2|2",
    "local-clamp|finite-domain|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_vertices|wp.vec3|2|2",
    "basis|same-scalar-order-unverified|f32|src/physics/flex.rs|upstream/_src/support.py|eval_basis_trilinear|float|1|1",
    "interpolate|same-loop-order|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_vertices|wp.vec3|3|5",
    "invalidate|private-bool-adapter|i32|src/physics/flex.rs|upstream/_src/smooth.py|flex|bool|2|2",
    "edge-difference|same-scalar|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|wp.vec3|1|2",
    "edge-length|same-scalar-order-unverified|f32|src/physics/flex_edge.rs|upstream/_src/math.py|normalize_with_norm|float|1|1",
    "edge-normalize|zero-preserved|f32|src/physics/flex_edge.rs|upstream/_src/math.py|normalize_with_norm|wp.vec3|1|3",
    "edge-offset|same-scalar|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|wp.vec3|2|2",
    "point-velocity|same-scalar-order-unverified|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|wp.vec3|2|2",
    "jacobian|same-endpoint-order|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|float|3|4",
    "edge-velocity|same-row-order|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|float|2|3",
    "sparse-pool|extra-tail-clear|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|float|1|2",
    "edge-early-return|same-velocity-only|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|float|1|3",
];
const OUTPUTS: &[&str] = &[
    "flexnode_xpos|flexnode_xpos|f32|wp.vec3|src/physics/flex.rs|_flex_nodes|array(\"nworld\", \"nflexnode\", wp.vec3)|float* nx=result+4+(unsigned long long)w*(3ull*nn+3ull*nv);|flex_transform(nx+3*i,local,xp,xm,nodebody[i],row[8]!=0||zero);|&v[..3 * self.layout.nn]|flexnode_xpos_out[worldid, nodeid] = xpos",
    "flexvert_xpos|flexvert_xpos|f32|wp.vec3|src/physics/flex.rs|_flex_vertices|array(\"nworld\", \"nflexvert\", wp.vec3)|float* vx=nx+3*nn;|for(int d=0;d<3;++d) vx[3*v+d]=p[d];|&v[3 * self.layout.nn..]|flexvert_xpos_out[worldid, vertid] = result",
    "flexedge_length|flexedge_length|f32|float|src/physics/flex_edge.rs|_flex_edges|array(\"nworld\", \"nflexedge\", float)|float* len=out+4+(unsigned long long)w*(2ull*ne+nnz);|len[e]=norm;|&v[..ne]|flexedge_length_out[worldid, edgeid] = edge_length",
    "flexedge_velocity|flexedge_velocity|f32|float|src/physics/flex_edge.rs|_flex_edges|array(\"nworld\", \"nflexedge\", float)|float* vel=len+ne;|vel[e]+=j*qv[d];|&v[ne..2 * ne]|flexedge_velocity_out[worldid, edgeid] = vel",
    "flexedge_jacobian|flexedge_J|f32|float|src/physics/flex_edge.rs|_flex_edges|array(\"nworld\", \"nJfe\", float)|float* jac=vel+ne;|jac[adr+i]=j;|&v[2 * ne..]|flexedge_J_out[worldid, rowadr + i] = J_val",
    "flex_hessian_valid|flex_hessian_valid|bool|bool|src/physics/flex.rs|flex|array(\"nworld\", \"nflex\", bool)|int* flags=result+4+(unsigned long long)w*nf;|flags[f]=0;|&self.validity[self.layout.validity.world_elements(world)?]|d.flex_hessian_valid.zero_()",
];
const REMAINING: &[&str] = &[
    "generated-operator-semantics",
    "sparse-unused-tail-side-effect",
    "face-hessian-and-other-internals",
    "full-pipeline-current-numerics",
    "measured-performance",
];
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    classification: String,
    scalar: String,
    source: String,
    external: String,
    symbol: String,
    upstream_type: String,
    product: Vec<String>,
    upstream: Vec<String>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    name: String,
    upstream_name: String,
    scalar: String,
    upstream_type: String,
    source: String,
    symbol: String,
    declaration: String,
    pointer: String,
    write: String,
    view: String,
    upstream_write: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    product_revision: String,
    upstream_revision: String,
    product_sources: Vec<Stamp>,
    external_sources: Vec<Stamp>,
    steps: Vec<Step>,
    outputs: Vec<Output>,
    np01_complete: bool,
    np02_complete: bool,
    equivalence_eligible: bool,
    remaining: Vec<String>,
}
#[test]
fn covers_sixteen_groups_and_six_output_routes() {
    validate(&inventory()).unwrap();
    assert_eq!((STEPS.len(), OUTPUTS.len()), (16, 6));
}
#[test]
fn rejects_missing_retyped_and_reclassified_flex_groups() {
    for n in 0..STEPS.len() {
        for mutation in 0..5 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.steps.remove(n);
                }
                1 => i.steps[n].scalar = "f64".into(),
                2 => i.steps[n].classification = "equivalent".into(),
                3 => {
                    i.steps[n].product.pop();
                }
                _ => i.steps[n].product[0] = "missing-product-expression".into(),
            }
            assert!(validate(&i).is_err(), "{n}/{mutation}");
        }
    }
}
#[test]
fn rejects_output_retyping_and_private_abi_confusion() {
    for n in 0..OUTPUTS.len() {
        for mutation in 0..5 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.outputs.remove(n);
                }
                1 => i.outputs[n].scalar = "f64".into(),
                2 => i.outputs[n].view = "&v[..]".into(),
                3 => i.outputs[n].upstream_name = "wrong-upstream-field".into(),
                _ => i.outputs[n].pointer = "wrong-pointer".into(),
            }
            assert!(validate(&i).is_err(), "{n}/{mutation}");
        }
    }
}
#[test]
fn rejects_drift_and_premature_flex_acceptance() {
    for n in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[n].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    for n in 0..5 {
        let mut i = inventory();
        match n {
            0 => i.np01_complete = true,
            1 => i.np02_complete = true,
            2 => i.equivalence_eligible = true,
            3 => i.remaining.clear(),
            _ => {
                i.external_sources.pop();
            }
        }
        assert!(validate(&i).is_err());
    }
}
#[test]
#[ignore = "reads frozen flex sources and existing captures; no Python, GPU or compilation"]
fn audits_frozen_flex_sources_and_retained_artifacts() {
    let i = inventory();
    validate(&i).unwrap();
    let mut texts = Texts::new();
    for s in &i.external_sources {
        let (kind, path) = s.path.split_once('/').unwrap();
        let key = if kind == "upstream" {
            "MJWARP_FROZEN_UPSTREAM_ROOT"
        } else {
            "MJWARP_FROZEN_WARP_ROOT"
        };
        let root = std::env::var_os(key).expect(key);
        texts.insert(
            s.path.clone(),
            std::fs::read_to_string(Path::new(&root).join(path))
                .unwrap()
                .replace("\r\n", "\n"),
        );
    }
    verify_external(&i, &texts).unwrap();
    let mut changed = i.clone();
    changed.steps[0].upstream[0] = "wrong-upstream".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = texts.clone();
    changed
        .get_mut("upstream/_src/support.py")
        .unwrap()
        .push('\n');
    assert!(verify_external(&i, &changed).is_err());
    let mut changed = texts.clone();
    let types = changed.get_mut("upstream/_src/types.py").unwrap();
    *types = types.replace(
        "flexedge_J: array(\"nworld\", \"nJfe\", float)",
        "flexedge_J: array(\"nworld\", \"nJfe\", wp.float64)",
    );
    assert!(verify_external(&i, &changed).is_err());
    let old = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("history");
    let current = std::env::var_os("MJWARP_T4_CURRENT_CAPTURE_ROOT").expect("current");
    let count = verify_captures(Path::new(&old), Path::new(&current)).unwrap();
    let report = serde_json::json!({
        "schema":1,"scope":i.scope,"product_revision":i.product_revision,
        "upstream_revision":i.upstream_revision,"map_sha256":digest(FIXTURE),
        "internal_groups":16,"f32_internal_groups":15,"private_i32_flag_groups":1,
        "output_fields":6,"f32_outputs":5,"bool_outputs":1,
        "historical_capture_variants":6,"retained_current_variants":3,"checked_capture_sets":count,
        "np01_complete":false,"np02_complete":false,"equivalence_eligible":false,
        "np03":"historical-not-rerun","np04":"not-measured","gpu_executions":0,"python_executions":0,
        "recompilations":0,"remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_FLEX_INTERNAL_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}

#[path = "flex_internal/checks.rs"]
mod checks;
use checks::*;
