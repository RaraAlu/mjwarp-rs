//! 追踪状态写入与打包存储。
//! 静态证据不替代GPU实测。
use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-state-packing.json");
const RESIDENT: &str = include!("../../docs/development-evidence/t4-resident-anchors/resident.rs");
const PHYSICS: &str = include!("../../docs/development-evidence/t4-resident-anchors/physics.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/resident.rs", RESIDENT),
    ("src/physics/mod.rs", PHYSICS),
    (
        "src/physics/attached.rs",
        include_str!("../../src/physics/attached.rs"),
    ),
    (
        "src/physics/sleep.rs",
        include_str!("../../src/physics/sleep.rs"),
    ),
    (
        "src/physics/flex.rs",
        include_str!("../../src/physics/flex.rs"),
    ),
    (
        "src/physics/flex_hessian.rs",
        include_str!("../../src/physics/flex_hessian.rs"),
    ),
    (
        "src/runtime/transfer.rs",
        concat!(
            include_str!("../../src/runtime/transfer.rs"),
            "\n",
            include_str!("../../src/runtime/transfer/buffer.rs"),
            "\n",
            include_str!("../../src/runtime/transfer/session.rs")
        ),
    ),
];
const REMAINING: [&str; 6] = [
    "per-field-model-packing",
    "physical-locals-and-accumulators",
    "complete-output-mapping",
    "other-physics-state-paths",
    "upstream-generated-kernels",
    "measured-performance",
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Upstream {
    owner: String,
    name: String,
    declaration: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    owner: String,
    name: String,
    host_type: String,
    device_owner: String,
    device_field: String,
    device_type: String,
    classification: String,
    upstream: Vec<Upstream>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packer {
    name: String,
    metadata: String,
    parameters: String,
    boundary: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    upstream_revision: String,
    product_revision: String,
    product_sources: Vec<Stamp>,
    states: Vec<State>,
    packers: Vec<Packer>,
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

fn declaration_fields(owner: &str) -> Result<Vec<(String, String)>, String> {
    let body = between(RESIDENT, &format!("struct {owner} {{"), "}")?;
    body.lines()
        .map(str::trim)
        .filter_map(|s| s.strip_prefix("pub "))
        .map(|s| {
            s.strip_suffix(',')
                .and_then(|s| s.split_once(": "))
                .map(|(name, ty)| (name.into(), ty.into()))
                .ok_or("state declaration changed".into())
        })
        .collect()
}

#[path = "t4_state_packing/captures.rs"]
mod captures;
#[path = "t4_state_packing/external.rs"]
mod external;
#[path = "t4_state_packing/tests.rs"]
mod tests;
#[path = "t4_state_packing/validate.rs"]
mod validate;
