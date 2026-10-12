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

pub fn validate_inventory(value: &Inventory) -> Result<(), String> {
    require(
        value.schema == 1 && value.upstream_revision == UPSTREAM,
        "unfrozen upstream/schema",
    )?;
    require(
        value.source_revision.len() == 40
            && value.source_revision.bytes().all(|c| c.is_ascii_hexdigit()),
        "invalid source revision",
    )?;
    require(
        value.scope == "t4-synchronous-physics-kernels",
        "unexpected precision scope",
    )?;
    require(
        !value.mappings.is_empty() && !value.artifacts.is_empty(),
        "empty precision evidence",
    )?;
    require(
        unique(value.mappings.iter().map(|v| v.entry.as_str())),
        "duplicate kernel mapping",
    )?;
    require(
        unique(value.product_sources.iter().map(|v| v.path.as_str())),
        "duplicate product source",
    )?;
    require(
        unique(value.upstream_sources.iter().map(|v| v.path.as_str())),
        "duplicate upstream source",
    )?;
    for stamp in value.product_sources.iter().chain(&value.upstream_sources) {
        require(valid_hash(&stamp.sha256), "invalid source hash")?;
    }
    let excluded: BTreeSet<_> = value
        .excluded_test_entries
        .iter()
        .map(String::as_str)
        .collect();
    require(
        excluded == BTreeSet::from(["adapter_probe", "fma_probe", "hessian_probe"]),
        "unexpected diagnostic exclusions",
    )?;
    for item in &value.mappings {
        require(
            !excluded.contains(item.entry.as_str()),
            "test-only entry in physical scope",
        )?;
        require(
            value.product_sources.iter().any(|s| s.path == item.product),
            "unbound product source",
        )?;
        require(
            value
                .upstream_sources
                .iter()
                .any(|s| s.path == item.upstream_file),
            "unbound upstream source",
        )?;
        require(
            !item.upstream_symbols.is_empty()
                && unique(item.upstream_symbols.iter().map(String::as_str)),
            "missing upstream symbols",
        )?;
        require(
            !item.fields.is_empty()
                && matches!(item.arithmetic.as_str(), "f32" | "integer-only")
                && !item.boundary.is_empty(),
            "incomplete type mapping",
        )?;
        let integer_only = matches!(
            item.entry.as_str(),
            "flex_invalidate" | "flex_hessian_validate" | "sleep_copy" | "tendon_indices"
        );
        require(
            (item.arithmetic == "integer-only") == integer_only,
            "arithmetic classification changed",
        )?;
    }
    let mut product_sources: BTreeSet<_> =
        value.mappings.iter().map(|v| v.product.as_str()).collect();
    product_sources.extend([
        "src/physics/resident.rs",
        "src/physics/tendon_wrap.rs",
        "src/runtime/transfer/kernel.rs",
        "src/runtime/transfer.rs",
        "fixtures/mjwarp-mass-solve/manifest.json",
    ]);
    require(
        product_sources
            == value
                .product_sources
                .iter()
                .map(|v| v.path.as_str())
                .collect(),
        "product source coverage changed",
    )?;
    let mut upstream_sources: BTreeSet<_> = value
        .mappings
        .iter()
        .map(|v| v.upstream_file.as_str())
        .collect();
    upstream_sources.extend([
        "_src/types.py",
        "_src/math.py",
        "_src/util_misc.py",
        "_src/block_cholesky.py",
    ]);
    require(
        upstream_sources
            == value
                .upstream_sources
                .iter()
                .map(|v| v.path.as_str())
                .collect(),
        "upstream source coverage changed",
    )?;
    let mapped: BTreeSet<_> = value.mappings.iter().map(|v| v.entry.as_str()).collect();
    let captured: BTreeSet<_> = value.artifacts.iter().map(|v| v.entry.as_str()).collect();
    require(mapped == captured, "kernel coverage mismatch")?;
    require(
        mapped == BTreeSet::from(ENTRIES) && value.artifacts.len() == 30,
        "frozen coverage changed",
    )?;
    let mut seen = BTreeSet::new();
    for item in &value.artifacts {
        require(
            seen.insert((&item.entry, &item.ptx_sha256)),
            "duplicate artifact",
        )?;
        require(
            [&item.source_sha256, &item.ptx_sha256, &item.receipt_sha256]
                .into_iter()
                .all(|v| valid_hash(v)),
            "invalid artifact hash",
        )?;
    }
    // 本清单只证明内核来源，不冒充完整逐字段审查。
    require(
        !value.remaining.is_empty(),
        "missing precision boundary declaration",
    )
}

pub fn verify_sources(value: &Inventory, product: &Path, upstream: &Path) -> Result<(), String> {
    validate_inventory(value)?;
    for (root, stamps) in [
        (product, &value.product_sources),
        (upstream, &value.upstream_sources),
    ] {
        for stamp in stamps {
            let bytes = read_under(root, &stamp.path)?;
            let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
            require(
                digest(text.replace("\r\n", "\n").as_bytes()) == stamp.sha256,
                &format!("source drift: {}", stamp.path),
            )?;
        }
    }
    for item in &value.mappings {
        let product =
            String::from_utf8(read_under(product, &item.product)?).map_err(|e| e.to_string())?;
        let upstream = String::from_utf8(read_under(upstream, &item.upstream_file)?)
            .map_err(|e| e.to_string())?;
        require(
            product.contains(&format!("void {}(", item.entry)),
            "missing product entry",
        )?;
        for symbol in &item.upstream_symbols {
            require(
                upstream.contains(&format!("def {symbol}(")),
                &format!("missing upstream symbol: {symbol}"),
            )?;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    entry: String,
    source_sha256: String,
    ptx_sha256: String,
    precision_gate: String,
    cuda_math_reduction: String,
    passed: bool,
    nvrtc_version: [u32; 2],
    fmad: bool,
    use_fast_math: bool,
    options: Vec<String>,
}

/// 验证原回执与原始字节，不信任调用方改写的passed字段。
pub fn verify_capture(value: &Inventory, directory: &Path) -> Result<(), String> {
    validate_inventory(value)?;
    let mut observed = BTreeSet::new();
    for file in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        if file.path().extension().is_some_and(|e| e == "json") {
            let receipt: Receipt =
                serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if !value.excluded_test_entries.contains(&receipt.entry) {
                let expected = format!("{}-{}.json", receipt.entry, receipt.ptx_sha256);
                require(
                    file.file_name().to_str() == Some(&expected),
                    "noncanonical receipt name",
                )?;
                observed.insert((receipt.entry, receipt.ptx_sha256));
            }
        }
    }
    let expected: BTreeSet<_> = value
        .artifacts
        .iter()
        .map(|v| (v.entry.clone(), v.ptx_sha256.clone()))
        .collect();
    require(observed == expected, "capture coverage mismatch")?;
    for item in &value.artifacts {
        let stem = format!("{}-{}", item.entry, item.ptx_sha256);
        let raw = read_under(directory, &format!("{stem}.json"))?;
        require(digest(&raw) == item.receipt_sha256, "receipt hash mismatch")?;
        let receipt: Receipt = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        require(
            receipt.entry == item.entry
                && receipt.source_sha256 == item.source_sha256
                && receipt.ptx_sha256 == item.ptx_sha256,
            "receipt identity mismatch",
        )?;
        require(
            receipt.passed && receipt.precision_gate == "f32-physics",
            "precision gate rejected",
        )?;
        require(
            receipt.cuda_math_reduction == "CUDA-12.8 sinf/cosf integer angle reduction only",
            "unexpected math exception",
        )?;
        require(
            receipt.fmad && !receipt.use_fast_math && receipt.nvrtc_version == [12, 8],
            "compiler precision mismatch",
        )?;
        require(
            receipt.options == ["--std=c++17", "--gpu-architecture=compute_70"],
            "compiler options mismatch",
        )?;
        for (extension, hash) in [("cu", &item.source_sha256), ("ptx", &item.ptx_sha256)] {
            require(
                digest(&read_under(directory, &format!("{stem}.{extension}"))?) == *hash,
                "generated artifact hash mismatch",
            )?;
        }
    }
    Ok(())
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
