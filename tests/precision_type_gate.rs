//! NP01 精度类型门禁（第一片）：CUDA 源面快照 + 产品 CUDA 精度 token 断言。
//!
//! 口径：`docs/numerical-precision-contract.md` 的 NP01（类型映射率 100%、实际精度偏离 0）
//! 是本门禁的**目标**，不是本测试给出的结论——本测试只做两件机器可判定的事：
//!
//! 1. **CUDA 源面快照棘轮**：`docs/development-evidence/np01-type-mapping/cuda-surface.json`
//!    记录当前产品 CUDA 源文件集合与每个文件的 `__global__` 入口数。任何新增/删除 `.cu`、
//!    或入口数变化都会让本测试失败，必须由人显式更新快照并同步类型清单（不允许新内核
//!    悄悄进产品面而不登记）。
//! 2. **精度 token 断言**：产品 `.cu` 源的符号区（去掉行注释/块注释后）不得出现
//!    `double` / `__half` / `half2` / `bfloat16` / `tf32`；唯一例外是 W15 头的
//!    `#define WP_NO_BFLOAT16` 守卫宏（整行精确匹配）。注释里讨论上游 double 分支是允许的，
//!    代码里出现就是违约。
//!
//! 本测试不覆盖：`.rs` 内联 CUDA 源字符串（下一片）、逐字段类型表、数值容差与任何 GPU 行为；
//! 也不宣称 NP01 通过。零 GPU、零网络、只读文件。

use std::fs;
use std::path::{Path, PathBuf};

const SNAPSHOT: &str = "docs/development-evidence/np01-type-mapping/cuda-surface.json";
const INVENTORY: &str = "docs/development-evidence/np01-type-mapping/inventory.json";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_cuda_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()));
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_cuda_sources(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("cu") {
            out.push(path);
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("source under manifest dir")
        .to_string_lossy()
        .replace('\\', "/")
}

fn global_entry_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .lines()
        .filter(|line| line.contains("__global__"))
        .count()
}

/// 去掉 `//` 行注释与 `/* ... */` 块注释后返回该行代码区。
fn code_part(line: &str, in_block: &mut bool) -> String {
    let mut out = String::new();
    let mut rest = line;
    loop {
        if *in_block {
            match rest.find("*/") {
                Some(end) => {
                    rest = &rest[end + 2..];
                    *in_block = false;
                }
                None => break,
            }
        } else {
            let line_comment = rest.find("//");
            let block_start = rest.find("/*");
            match (line_comment, block_start) {
                (Some(line), Some(block)) if block < line => {
                    out.push_str(&rest[..block]);
                    match rest[block + 2..].find("*/") {
                        Some(end) => rest = &rest[block + 2 + end + 2..],
                        None => {
                            *in_block = true;
                            break;
                        }
                    }
                }
                (None, Some(block)) => {
                    out.push_str(&rest[..block]);
                    match rest[block + 2..].find("*/") {
                        Some(end) => rest = &rest[block + 2 + end + 2..],
                        None => {
                            *in_block = true;
                            break;
                        }
                    }
                }
                (Some(line), _) => {
                    out.push_str(&rest[..line]);
                    break;
                }
                (None, None) => {
                    out.push_str(rest);
                    break;
                }
            }
        }
    }
    out
}

fn precision_hits(path: &Path) -> Vec<(usize, String, String)> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut hits = Vec::new();
    let mut in_block = false;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        let code = code_part(trimmed, &mut in_block);
        let code = code.trim();
        if code.is_empty() {
            continue;
        }
        if code == "#define WP_NO_BFLOAT16" {
            continue;
        }
        for token in ["double", "__half", "half2", "bfloat16", "tf32"] {
            let matched = if token == "double" {
                code.split(|ch: char| !ch.is_alphanumeric() && ch != '_')
                    .any(|word| word == token)
            } else {
                code.contains(token)
            };
            if matched {
                hits.push((index + 1, token.to_string(), code.to_string()));
                break;
            }
        }
    }
    hits
}

#[test]
fn cuda_source_surface_matches_snapshot() {
    let root = manifest_dir();
    let snapshot_path = root.join(SNAPSHOT);
    let snapshot: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", snapshot_path.display())),
    )
    .expect("cuda-surface.json must be valid JSON");

    let mut files = Vec::new();
    collect_cuda_sources(&root.join("src"), &mut files);
    files.sort();
    let actual: Vec<(String, usize)> = files
        .iter()
        .map(|path| (relative(&root, path), global_entry_count(path)))
        .collect();

    let expected: Vec<(String, usize)> = snapshot["files"]
        .as_array()
        .expect("snapshot.files must be an array")
        .iter()
        .map(|entry| {
            (
                entry["path"].as_str().expect("snapshot path").to_string(),
                entry["globals"].as_u64().expect("snapshot globals") as usize,
            )
        })
        .collect();

    assert_eq!(
        actual.len(),
        snapshot["totals"]["files"].as_u64().unwrap_or(0) as usize,
        "CUDA 源文件数与快照 totals.files 不符"
    );
    assert_eq!(
        actual.iter().map(|(_, count)| *count).sum::<usize>(),
        snapshot["totals"]["globals"].as_u64().unwrap_or(0) as usize,
        "CUDA __global__ 入口总数与快照 totals.globals 不符"
    );
    if actual != expected {
        let new_files: Vec<&String> = actual
            .iter()
            .map(|(path, _)| path)
            .filter(|path| {
                !expected
                    .iter()
                    .any(|(expected_path, _)| expected_path == *path)
            })
            .collect();
        let gone_files: Vec<&String> = expected
            .iter()
            .map(|(path, _)| path)
            .filter(|path| !actual.iter().any(|(actual_path, _)| actual_path == *path))
            .collect();
        let changed: Vec<String> = actual
            .iter()
            .filter_map(|(path, count)| {
                expected
                    .iter()
                    .find(|(expected_path, _)| expected_path == path)
                    .filter(|(_, expected_count)| expected_count != count)
                    .map(|(_, expected_count)| format!("{path}: {expected_count} -> {count}"))
            })
            .collect();
        panic!(
            "CUDA 源面与快照不符：新增={new_files:?} 删除={gone_files:?} 入口数变化={changed:?}\n\
             处理：更新 {SNAPSHOT} 并同步 docs/development-evidence/np01-type-mapping/ 的类型登记"
        );
    }
}

#[test]
fn product_cuda_sources_have_no_precision_tokens() {
    let root = manifest_dir();
    let mut files = Vec::new();
    collect_cuda_sources(&root.join("src"), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in &files {
        for (line, token, code) in precision_hits(path) {
            violations.push(format!("{}:{line} [{token}] {code}", relative(&root, path)));
        }
    }
    assert!(
        violations.is_empty(),
        "产品 CUDA 源代码区出现禁用精度 token（注释讨论允许，代码不允许；W15 守卫宏除外）：\n{}",
        violations.join("\n")
    );
}

#[test]
fn type_inventory_keeps_its_registered_shape() {
    let root = manifest_dir();
    let inventory_path = root.join(INVENTORY);
    let inventory: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&inventory_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", inventory_path.display())),
    )
    .expect("inventory.json must be valid JSON");
    let rows = inventory["rows"].as_array().expect("inventory.rows");
    assert!(
        rows.len() >= 30,
        "类型清单行数 {} 少于已登记规模 30（不得静默删行）",
        rows.len()
    );
    for row in rows {
        let id = row["id"].as_str().expect("row.id");
        for key in [
            "input",
            "intermediate",
            "accumulation",
            "output",
            "doubleInProduct",
        ] {
            let value = row["types"][key]
                .as_str()
                .unwrap_or_else(|| panic!("row {id} 缺 types.{key}"));
            assert!(!value.trim().is_empty(), "row {id} 的 types.{key} 为空");
        }
        assert!(
            row["productFiles"]
                .as_array()
                .is_some_and(|files| !files.is_empty()),
            "row {id} 缺 productFiles"
        );
    }
    let host = &inventory["hostSiteRegistration"];
    let host_rows = host["hostCoverage"]
        .as_array()
        .expect("hostCoverage 必须是数组");
    assert!(
        host_rows.len() >= 3,
        "host 站点登记行数 {} 少于已登记规模 3（gjk/ccd/epa host 口径不得被删）",
        host_rows.len()
    );
    let sites = host["siteSemantics"]
        .as_array()
        .expect("siteSemantics 必须是数组");
    assert!(sites.len() >= 3, "host 乘加融合站点语义登记不得被删");
    for row in host_rows {
        let id = row["id"].as_str().expect("hostCoverage.id");
        for key in [
            "input",
            "intermediate",
            "accumulation",
            "output",
            "doubleInProduct",
        ] {
            let value = row["types"][key]
                .as_str()
                .unwrap_or_else(|| panic!("host row {id} 缺 types.{key}"));
            assert!(
                !value.trim().is_empty(),
                "host row {id} 的 types.{key} 为空"
            );
        }
    }
}

// ---- 逐字段表覆盖门禁（NP01 B1 起） ----------------------------------------

const FIELD_TABLES: &str = "docs/development-evidence/np01-type-mapping/field-tables";

/// 尚未产出逐字段表的清单行（按批登记）。每批交付后必须从这里删掉对应行；宁可测试失败，也不许静默缺表。
/// B2 批已收口：30 条产品路径全部有表，清单清空；新增行必须先登记回这里再交付。
const PENDING_ROWS: [(&str, &str); 0] = [];

#[test]
fn field_tables_cover_registered_rows_without_silent_gaps() {
    let root = manifest_dir();
    let inventory: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(INVENTORY)).expect("read inventory.json"),
    )
    .expect("inventory.json must be valid JSON");
    let rows = inventory["rows"].as_array().expect("inventory.rows");
    let row_ids: Vec<String> = rows
        .iter()
        .map(|row| row["id"].as_str().expect("row.id").to_string())
        .collect();

    let tables_dir = root.join(FIELD_TABLES);
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
    assert!(
        !tables.is_empty(),
        "field-tables/ 为空：NP01 逐字段表至少 B1 批必须存在"
    );

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
        let row_id = table["row_id"]
            .as_str()
            .unwrap_or_else(|| panic!("{name} 缺 row_id"))
            .to_string();
        assert!(
            row_ids.iter().any(|id| id == &row_id),
            "{name} 的 row_id={row_id} 不在 inventory.json 行清单里"
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
        covered.push(row_id);
    }

    let mut pending: Vec<String> = PENDING_ROWS.iter().map(|(id, _)| id.to_string()).collect();
    pending.sort();
    covered.sort();
    covered.dedup();
    for id in &pending {
        assert!(
            row_ids.iter().any(|row| row == id),
            "待批清单里的 {id} 不在 inventory 行里（清单已过期）"
        );
        assert!(
            !covered.contains(id),
            "{id} 已有逐字段表，但仍留在待批清单里：请从 PENDING_ROWS 删除"
        );
    }

    let mut missing: Vec<String> = row_ids
        .iter()
        .filter(|id| !covered.contains(*id) && !pending.contains(*id))
        .cloned()
        .collect();
    missing.sort();
    assert!(
        missing.is_empty(),
        "以下清单行既没有逐字段表、也不在待批清单里（不允许静默缺表）：{missing:?}"
    );
}
