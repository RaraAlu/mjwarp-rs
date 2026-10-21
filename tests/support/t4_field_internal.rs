//! 逐字段核对批量打包与质心内部量。
//! 此切片不代表完整精度验收。

use super::face_lineage;

#[path = "t4_field_internal/checks.rs"]
mod checks;
#[path = "t4_field_internal/flex_matrices.rs"]
mod flex_matrices;
#[path = "t4_field_internal/tests.rs"]
mod tests;

use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-field-internal.json");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
const PHYSICS: &str = include_str!("../../src/physics/mod.rs");
const PARAMETERS: &str = include_str!("../../src/model/parameters.rs");
const CAMLIGHT: &str = include_str!("../../src/model/camlight.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/resident.rs", RESIDENT),
    ("src/physics/mod.rs", PHYSICS),
    (
        "src/physics/attached.rs",
        include_str!("../../src/physics/attached.rs"),
    ),
    (
        "src/physics/camlight.rs",
        include_str!("../../src/physics/camlight.rs"),
    ),
    ("src/model/parameters.rs", PARAMETERS),
    ("src/model/camlight.rs", CAMLIGHT),
];
const REMAINING: [&str; 6] = [
    "other-packers-per-field",
    "integer-topology-and-derived-pack-fields",
    "other-kernel-locals-and-accumulators",
    "complete-output-mapping",
    "upstream-generated-kernels",
    "measured-performance",
];
// 人工审查固定对应关系。
// 不把不同顺序冒充逐算子等价。
const STEPS: &[(&str, &str, &str, &str)] = &[
    ("mass-init", "_subtree_com_init", "derived-cache", "float"),
    (
        "weighted-position",
        "_subtree_com_init",
        "same-scalar",
        "wp.vec3",
    ),
    ("mass-accumulate", "_subtree_div", "derived-cache", "float"),
    (
        "position-accumulate",
        "_subtree_com_acc",
        "different-order",
        "wp.vec3",
    ),
    ("normalize", "_subtree_div", "derived-cache", "wp.vec3"),
    ("inertia-offset", "_cinert", "same-scalar", "wp.vec3"),
    ("inertia-rotation", "_cinert", "different-order", "wp.mat33"),
    ("inertia-diagonal", "_cinert", "same-scalar", "vec10"),
    ("inertia-offdiagonal", "_cinert", "same-scalar", "vec10"),
    ("inertia-moments", "_cinert", "same-scalar", "vec10"),
    ("motion-offset", "_cdof", "same-scalar", "wp.vec3"),
    (
        "free-translation",
        "_cdof",
        "same-scalar",
        "wp.spatial_vector",
    ),
    ("motion-axis", "_cdof", "same-scalar", "wp.vec3"),
    ("slide-motion", "_cdof", "same-scalar", "wp.spatial_vector"),
    (
        "rotation-motion",
        "_cdof",
        "same-scalar",
        "wp.spatial_vector",
    ),
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    group: String,
    variant: String,
    name: String,
    slot: usize,
    descriptor_scalar: String,
    parameter_scalar: String,
    width: String,
    reader: String,
    entry: String,
    upstream_declaration: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    symbol: String,
    classification: String,
    scalar: String,
    upstream_type: String,
    product: Vec<String>,
    upstream: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    product_revision: String,
    upstream_revision: String,
    product_sources: Vec<Stamp>,
    upstream_smooth: Stamp,
    fields: Vec<Field>,
    com_steps: Vec<Step>,
    np01_complete: bool,
    remaining: Vec<String>,
}

fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}

fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, s)| (*p == path).then_some(*s))
        .ok_or_else(|| format!("unreviewed source: {path}"))
}

fn com_source() -> &'static str {
    between(PHYSICS, "const COM_POSITION_CUDA: &str = r#\"", "\"#;").unwrap()
}

fn kernel(entry: &str) -> Result<(&'static str, &'static str, &'static str), String> {
    match entry {
        "rigid_kinematics" => Ok((PHYSICS, "model", "desc")),
        "rigid_com_position" => Ok((com_source(), "p", "desc")),
        "attached_kinematics" => Ok((source("src/physics/attached.rs")?, "model", "desc")),
        "camlight_rigid" | "camlight_com" => Ok((source("src/physics/camlight.rs")?, "model", "d")),
        _ => Err("unreviewed reader entry".into()),
    }
}

fn reader_anchor(f: &Field) -> Result<String, String> {
    let (_, parameters, base) = kernel(&f.entry)?;
    let descriptor = if f.slot == 0 {
        base.into()
    } else {
        format!("{base}+{}", 2 * f.slot)
    };
    Ok(format!(
        "const float* {}=field_parameter({parameters},{descriptor},w,{});",
        f.reader, f.width
    ))
}

fn variants(text: &str, start: &str, prefix: &str) -> Result<Vec<String>, String> {
    Ok(between(text, start, "];")?
        .lines()
        .filter_map(|line| line.trim().strip_prefix(prefix))
        .map(|line| line.trim_end_matches(',').to_owned())
        .collect())
}
