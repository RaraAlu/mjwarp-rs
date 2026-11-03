//! NP01 host 站点逐字段表覆盖门禁（B3 起；2026-11-04 门禁硬化从 precision_type_gate 拆出）。

use std::fs;
use std::path::PathBuf;

const INVENTORY: &str = "docs/development-evidence/np01-type-mapping/inventory.json";
const HOST_TABLES: &str = "docs/development-evidence/np01-type-mapping/host-tables";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 逐字段表公共形状断言：必备段非空、kernels 逐项含 params/outputs 与 intermediates/accumulations 段、
/// 文件名（去 `.json`）必须等于表内 `row_id`。
fn assert_table_shape(
    name: &str,
    table: &serde_json::Value,
    allowed: &[String],
    kind: &str,
) -> String {
    let row_id = table["row_id"]
        .as_str()
        .unwrap_or_else(|| panic!("{name} 缺 row_id"))
        .to_string();
    assert_eq!(
        name.strip_suffix(".json").unwrap_or(name),
        row_id,
        "{name} 文件名≠row_id"
    );
    assert!(
        allowed.iter().any(|id| id == &row_id),
        "{name} 的 row_id={row_id} 不在 {kind} 里"
    );
    for key in [
        "batch",
        "layer",
        "audit_level",
        "status",
        "source",
        "upstream",
        "evidence",
    ] {
        let empty = match &table[key] {
            serde_json::Value::String(text) => text.trim().is_empty(),
            serde_json::Value::Array(items) => items.is_empty(),
            serde_json::Value::Object(map) => map.is_empty(),
            serde_json::Value::Null => true,
            _ => false,
        };
        assert!(!empty, "{name} 的 {key} 为空");
    }
    let kernels = table["kernels"]
        .as_array()
        .unwrap_or_else(|| panic!("{name} 缺 kernels"));
    assert!(!kernels.is_empty(), "{name} 的 kernels 为空");
    for kernel in kernels {
        let kernel_name = kernel["name"].as_str().unwrap_or("<unnamed>");
        for key in ["params", "outputs"] {
            let items = kernel[key]
                .as_array()
                .unwrap_or_else(|| panic!("{name}/{kernel_name} 缺 {key}"));
            assert!(!items.is_empty(), "{name}/{kernel_name} 的 {key} 为空");
        }
        assert!(
            kernel["intermediates"].is_array() && kernel["accumulations"].is_array(),
            "{name}/{kernel_name} 缺 intermediates/accumulations 段"
        );
    }
    row_id
}

#[test]
fn host_tables_cover_registered_host_sites_without_silent_gaps() {
    let root = manifest_dir();
    let inventory: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(INVENTORY)).expect("read inventory.json"),
    )
    .expect("inventory.json must be valid JSON");
    let host_sites: Vec<String> = inventory["hostSiteRegistration"]["hostCoverage"]
        .as_array()
        .expect("hostSiteRegistration.hostCoverage")
        .iter()
        .map(|row| row["id"].as_str().expect("hostCoverage.id").to_string())
        .collect();
    assert!(!host_sites.is_empty(), "hostCoverage 为空");

    let tables_dir = root.join(HOST_TABLES);
    let mut tables = Vec::new();
    for entry in fs::read_dir(&tables_dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", tables_dir.display()))
    {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            tables.push(path);
        }
    }
    tables.sort();
    assert!(!tables.is_empty(), "host-tables/ 为空：host 站点表不得缺失");

    let mut covered = Vec::new();
    for path in &tables {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let table: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(path).unwrap_or_else(|error| panic!("read {name}: {error}")),
        )
        .unwrap_or_else(|error| panic!("{name} 不是合法 JSON: {error}"));
        covered.push(assert_table_shape(
            &name,
            &table,
            &host_sites,
            "inventory.hostSiteRegistration.hostCoverage",
        ));
    }

    assert_eq!(
        covered.len(),
        host_sites.len(),
        "host-tables/ 表数与 hostCoverage 站点数不一致（重复或多表）"
    );

    let mut missing: Vec<String> = host_sites
        .iter()
        .filter(|id| !covered.contains(*id))
        .cloned()
        .collect();
    missing.sort();
    assert!(
        missing.is_empty(),
        "以下 host 站点没有逐字段表（不允许静默缺表）：{missing:?}"
    );
}
