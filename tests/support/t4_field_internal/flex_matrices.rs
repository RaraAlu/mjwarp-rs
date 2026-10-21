//! 面与矩阵新增切片；不执行旧满额选择。

use super::super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const FIXTURE: &str = include_str!("../../../docs/evidence/t4-g03-retest/flex-matrices/map.json");
const FACE: &str = include_str!("../../../src/physics/flex_face.rs");
const HESSIAN: &str = include_str!("../../../src/physics/flex_hessian.rs");
const RESIDENT: &str =
    include!("../../../docs/development-evidence/t4-resident-anchors/resident.rs");
const MODEL: &str = include_str!("../../../src/model/flex_hessian.rs");
const FLEX: &str = include_str!("../../../src/model/flex.rs");
const TRANSFER: &str = include_str!("../../../src/runtime/transfer.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/flex_face.rs", FACE),
    ("src/physics/flex_hessian.rs", HESSIAN),
    ("src/physics/resident.rs", RESIDENT),
    ("src/model/flex_hessian.rs", MODEL),
    ("src/model/flex.rs", FLEX),
    ("src/runtime/transfer.rs", TRANSFER),
];
const EXTERNAL: &[&str] = &[
    "upstream/_src/smooth.py",
    "upstream/_src/support.py",
    "upstream/_src/passive.py",
    "upstream/_src/types.py",
    "warp/_src/types.py",
    "warp/native/quat.h",
    "warp/native/builtin.h",
];
const GROUPS: &[&str] = &[
    "face-nodes|linear-only|f32|src/physics/flex_face.rs|upstream/_src/smooth.py|_flex_face_kinematics|2|2",
    "face-gradient|linear-only|f32|src/physics/flex_face.rs|upstream/_src/smooth.py|_flex_face_kinematics|1|4",
    "face-tangents|same-loop-order|f32|src/physics/flex_face.rs|upstream/_src/smooth.py|_flex_face_kinematics|3|5",
    "face-columns|column-major-private-layout|f32|src/physics/flex_face.rs|upstream/_src/smooth.py|_flex_face_kinematics|4|4",
    "polar-initial-stop|same-constants|f32|src/physics/flex_face.rs|upstream/_src/support.py|mat33_to_quat_polar|3|3",
    "polar-rotation-matrix|different-unit-quaternion-expression|f32|src/physics/flex_face.rs|warp/native/quat.h||3|4",
    "polar-omega|same-scalar-generated-checked|f32|src/physics/flex_face.rs|upstream/_src/support.py|mat33_to_quat_polar|2|3",
    "polar-axis|same-scalar|f32|src/physics/flex_face.rs|upstream/_src/support.py|mat33_to_quat_polar|2|4",
    "polar-product|different-grouping|f32|src/physics/flex_face.rs|warp/native/quat.h||2|2",
    "polar-normalize|divide-versus-reciprocal|f32|src/physics/flex_face.rs|warp/native/quat.h||2|2",
    "cache-clear|valid-skips-clear|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|_flex_hessian_clear|3|3",
    "edge-vectors|same-endpoint-order|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_edge_vectors|2|2",
    "elongation|same-scalar|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_elongation|1|1",
    "metric|21-coefficient-only|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_metric|2|4",
    "tension|same-row-order|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_tension|2|3",
    "material-block|unscaled-only|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_stiffness_block|2|2",
    "geometric-block|zero-sign-evaluation-differs|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_stiffness_block|3|4",
    "edge-scatter|serial-versus-atomic|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|_flex_hessian_elem|2|3",
    "diagonal-scatter|serial-versus-atomic|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|_flex_hessian_diag|2|3",
    "cache-publish|private-bool-adapter|i32|src/physics/flex_hessian.rs|upstream/_src/passive.py|flex_hessian|2|1",
    "active-gate|host-derived-bool|bool|src/model/flex_hessian.rs|upstream/_src/passive.py|_flex_hessian_active|5|3",
];
const OUTPUTS: &[&str] = &[
    "face_xpos|f32|src/physics/flex_face.rs|array(\"nworld\", \"nflexface\", 9, wp.vec3)|float* positions=result+4+(unsigned long long)w*31ull*nf;|out[k]=nx[3*n+k];|&values[..27 * self.layout.nf]|upstream/_src/smooth.py|_flex_face_kinematics|face_xpos_out[worldid, face_id, local_idx] = node_pos",
    "face_quat|f32|src/physics/flex_face.rs|array(\"nworld\", \"nflexface\", wp.quat)|float* quats=positions+27*nf;|face_polar(quats+4*f,F);|&values[27 * self.layout.nf..]|upstream/_src/smooth.py|_flex_face_kinematics|face_quat_out[worldid, face_id] = support.mat33_to_quat_polar(F)",
    "flexvert_hessian|f32|src/physics/flex_hessian.rs|array(\"nworld\", \"nflexvert\", vec6)|float* diag=result+4+(unsigned long long)w*(6ull*nv+9ull*ne);|d0[k]-=b[upper[k]];d1[k]-=b[lower[k]];|&v[..6 * self.layout.nv]|upstream/_src/passive.py|_flex_hessian_diag|wp.atomic_add(flexvert_hessian_out, worldid, va + vert[0], d0)",
    "flexedge_hessian|f32|src/physics/flex_hessian.rs|array(\"nworld\", \"nflexedge\", wp.mat33)|float* blocks=diag+6*nv;|blocks[9*edge+k]+=block[k];|&v[6 * self.layout.nv..]|upstream/_src/passive.py|_flex_hessian_elem|wp.atomic_add(flexedge_hessian_out, worldid, edge_id, block)",
];
const REMAINING: &[&str] = &[
    "polar-matrix-product-and-normalization-order",
    "serial-versus-atomic-accumulation",
    "zero-sign-nonfinite-intermediates",
    "upstream-hessian-generated-artifacts",
    "higher-order-faces-and-24-coefficient-models",
    "full-pipeline-current-numerics",
    "measured-performance",
];
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    id: String,
    classification: String,
    scalar: String,
    source: String,
    external: String,
    symbol: String,
    product: Vec<String>,
    upstream: Vec<String>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    name: String,
    scalar: String,
    source: String,
    declaration: String,
    pointer: String,
    write: String,
    view: String,
    external: String,
    symbol: String,
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
    groups: Vec<Group>,
    outputs: Vec<Output>,
    remaining: Vec<String>,
    np01_complete: bool,
    np02_complete: bool,
    equivalence_eligible: bool,
}

#[path = "flex_matrices/checks.rs"]
mod checks;
#[path = "flex_matrices/inventory.rs"]
mod inventory;
#[path = "flex_matrices/tests.rs"]
mod tests;
