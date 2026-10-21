//! 清单结构与共享读取。

use super::super::{Stamp, between};
use super::{FIXTURE, PHYSICS};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Step {
    pub(super) id: String,
    pub(super) classification: String,
    pub(super) scalar: String,
    pub(super) upstream_source: String,
    pub(super) symbol: String,
    pub(super) upstream_type: String,
    pub(super) product: Vec<String>,
    pub(super) upstream: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Output {
    pub(super) name: String,
    pub(super) scalar: String,
    pub(super) upstream_type: String,
    pub(super) pointer: String,
    pub(super) write: String,
    pub(super) view: String,
    pub(super) symbol: String,
    pub(super) upstream_write: String,
    pub(super) upstream_declaration: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ZeroQuaternion {
    pub(super) status: String,
    pub(super) product_return: [u8; 4],
    pub(super) warp_header_return: [u8; 4],
    pub(super) warp_doc_claim: String,
    pub(super) upstream_generated_verified: bool,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Inventory {
    pub(super) schema: u32,
    pub(super) scope: String,
    pub(super) product_revision: String,
    pub(super) upstream_revision: String,
    pub(super) product_sources: Vec<Stamp>,
    pub(super) external_sources: Vec<Stamp>,
    pub(super) steps: Vec<Step>,
    pub(super) outputs: Vec<Output>,
    pub(super) boundaries: Vec<String>,
    pub(super) zero_quaternion: ZeroQuaternion,
    pub(super) np01_complete: bool,
    pub(super) np02_complete: bool,
    pub(super) equivalence_eligible: bool,
    pub(super) remaining: Vec<String>,
}

pub(super) fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}

pub(super) fn kernel() -> &'static str {
    between(PHYSICS, "const KINEMATICS_CUDA: &str = r#\"", "\"#;").unwrap()
}
