//! 只校验证据，不执行物理或性能工作负载。
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

pub const UPSTREAM: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
pub const ENTRIES: [&str; 24] = [
    "attached_kinematics",
    "camlight_com",
    "camlight_rigid",
    "com_velocity",
    "fixed_tendon",
    "flex_edges",
    "flex_faces",
    "flex_hessian",
    "flex_hessian_validate",
    "flex_invalidate",
    "flex_positions",
    "initialize_attached",
    "rigid_com_position",
    "rigid_kinematics",
    "rigid_mass_matrix",
    "rigid_mass_solve",
    "rne_bias",
    "sleep_copy",
    "spatial_moment",
    "spatial_site",
    "spatial_wrap",
    "tendon_indices",
    "tendon_values",
    "tendon_wake",
];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub entry: String,
    pub product: String,
    pub upstream_file: String,
    pub upstream_symbols: Vec<String>,
    pub fields: Vec<String>,
    pub arithmetic: String,
    pub boundary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub entry: String,
    pub source_sha256: String,
    pub ptx_sha256: String,
    pub receipt_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    pub schema: u32,
    pub upstream_revision: String,
    pub source_revision: String,
    pub scope: String,
    pub product_sources: Vec<Stamp>,
    pub upstream_sources: Vec<Stamp>,
    pub mappings: Vec<Mapping>,
    pub artifacts: Vec<Artifact>,
    pub excluded_test_entries: Vec<String>,
    pub remaining: Vec<String>,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn require(condition: bool, reason: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(reason.into())
    }
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn unique<'a>(values: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .all(|value| !value.is_empty() && seen.insert(value))
}

/// 禁止绝对路径、父目录以及越过根目录的链接。
fn read_under(root: &Path, path: &str) -> Result<Vec<u8>, String> {
    use std::path::Component;
    let relative = Path::new(path);
    require(
        !path.contains('\\') && !path.contains(':'),
        "non-portable evidence path",
    )?;
    require(
        relative
            .components()
            .all(|v| matches!(v, Component::Normal(_))),
        "unsafe evidence path",
    )?;
    let base = root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", root.display()))?;
    let file = base
        .join(relative)
        .canonicalize()
        .map_err(|e| format!("{path}: {e}"))?;
    require(file.starts_with(&base), "evidence escapes root")?;
    fs::read(file).map_err(|e| format!("{path}: {e}"))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PerformancePlan {
    pub schema: u32,
    pub scope: String,
    pub upstream_revision: String,
    pub product_revision: String,
    pub product_artifact_sha256: String,
    pub upstream_artifact_sha256: String,
    pub model_sha256: String,
    pub input_sha256: String,
    pub state_sha256: String,
    pub worlds: Vec<u64>,
    pub updates_per_sample: u64,
    pub warmup: u64,
    pub repetitions: usize,
    pub boundary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceRow {
    pub implementation: String,
    pub source_revision: String,
    pub artifact_sha256: String,
    pub plan_sha256: String,
    pub model_sha256: String,
    pub input_sha256: String,
    pub state_sha256: String,
    pub worlds: u64,
    pub updates_per_sample: u64,
    pub gpu_uuid: String,
    pub driver: String,
    pub precision: String,
    pub fmad: bool,
    pub fast_math: bool,
    pub warmup: u64,
    pub boundary: String,
    pub synchronization: String,
    pub timing: String,
    pub samples_ms: Vec<f64>,
    pub memory_scope: String,
    pub memory_metric: String,
    pub memory_method: String,
    pub memory_complete: bool,
    pub baseline_bytes: u64,
    pub peak_bytes: u64,
    pub correctness_passed: bool,
    pub exclusive_gpu: bool,
}

/// 返回条件校验结果，不证明数据真实性或性能达标。
pub fn verify_performance(plan: &PerformancePlan, rows: &[PerformanceRow]) -> Result<(), String> {
    require(
        plan.schema == 1 && plan.scope == "G01-complete" && plan.upstream_revision == UPSTREAM,
        "invalid performance plan",
    )?;
    require(
        plan.product_revision.len() == 40
            && plan.product_revision.bytes().all(|c| c.is_ascii_hexdigit()),
        "missing product revision",
    )?;
    require(
        valid_hash(&plan.product_artifact_sha256) && valid_hash(&plan.upstream_artifact_sha256),
        "missing benchmark artifact identity",
    )?;
    require(
        [&plan.model_sha256, &plan.input_sha256, &plan.state_sha256]
            .into_iter()
            .all(|s| valid_hash(s)),
        "invalid effective input hashes",
    )?;
    require(
        plan.worlds == [1, 2, 5, 513] && plan.updates_per_sample == 4,
        "frozen G01 coverage changed",
    )?;
    require(
        plan.worlds.iter().collect::<BTreeSet<_>>().len() == plan.worlds.len(),
        "duplicate planned batch",
    )?;
    require(
        plan.warmup > 0
            && plan.repetitions > 1
            && plan.boundary == "resident-g01-complete-no-transfer",
        "unfrozen steady-state boundary",
    )?;
    require(
        rows.len() == plan.worlds.len() * 2,
        "missing paired batches",
    )?;
    let plan_hash = digest(&serde_json::to_vec(plan).map_err(|e| e.to_string())?);
    let mut seen = BTreeSet::new();
    let first = rows.first().ok_or("empty performance evidence")?;
    require(
        !first.gpu_uuid.is_empty() && !first.driver.is_empty(),
        "missing device identity",
    )?;
    for row in rows {
        require(
            row.updates_per_sample == plan.updates_per_sample,
            "state update coverage changed",
        )?;
        require(
            matches!(row.implementation.as_str(), "mjwarp-rs" | "frozen-mjwarp"),
            "unknown implementation",
        )?;
        let (revision, artifact) = if row.implementation == "mjwarp-rs" {
            (&plan.product_revision, &plan.product_artifact_sha256)
        } else {
            (&plan.upstream_revision, &plan.upstream_artifact_sha256)
        };
        require(
            &row.source_revision == revision && &row.artifact_sha256 == artifact,
            "benchmark source/artifact mismatch",
        )?;
        require(
            plan.worlds.contains(&row.worlds) && seen.insert((&row.implementation, row.worlds)),
            "duplicate or extra batch",
        )?;
        require(
            row.plan_sha256 == plan_hash
                && row.model_sha256 == plan.model_sha256
                && row.input_sha256 == plan.input_sha256
                && row.state_sha256 == plan.state_sha256,
            "effective inputs differ",
        )?;
        require(
            row.gpu_uuid == first.gpu_uuid && row.driver == first.driver,
            "device mismatch",
        )?;
        require(
            row.precision == "f32" && row.fmad && !row.fast_math,
            "unfair precision",
        )?;
        require(
            row.warmup == plan.warmup
                && row.boundary == plan.boundary
                && row.timing == "steady_state",
            "unfair timing boundary",
        )?;
        require(
            row.synchronization == "before-and-after-each-sample",
            "unsynchronized timing",
        )?;
        require(
            row.samples_ms.len() == plan.repetitions
                && row.samples_ms.iter().all(|x| x.is_finite() && *x > 0.0),
            "invalid timing samples",
        )?;
        require(
            row.memory_scope == "process"
                && row.memory_metric == "peak_device_allocated_bytes"
                && row.memory_method == "allocation_events"
                && row.memory_complete,
            "incomplete process memory evidence",
        )?;
        require(
            row.peak_bytes > 0 && row.peak_bytes >= row.baseline_bytes,
            "invalid process peak",
        )?;
        require(
            row.correctness_passed && row.exclusive_gpu,
            "unverified or contended benchmark",
        )?;
    }
    Ok(())
}

#[path = "t4_evidence/capture.rs"]
mod capture;
#[path = "t4_evidence/inventory.rs"]
mod inventory;
#[path = "t4_evidence/sources.rs"]
mod sources;

pub use capture::verify_capture;
pub use inventory::validate_inventory;
pub use sources::verify_sources;
