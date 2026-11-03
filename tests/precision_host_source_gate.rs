//! NP01 门禁第二片：产品 **Rust 宿主源**的精度 token 断言 + 允许清单活性检查。
//!
//! 第一片（`tests/precision_type_gate.rs`）覆盖产品 `.cu` 源面；本片补宿主 Rust 侧：
//!
//! 1. **宿主 Rust 代码区不得出现 C/设备宽度精度 token**（`double` / `__half` / `half2` /
//!    `bfloat16` / `tf32`）。Rust 侧的 `f64` 类型不在本门禁范围——契约允许参考/诊断与原生
//!    ABI 转换使用 f64（`mjtNum`），本门禁只拦「C 风格 / 设备宽度类型混进宿主代码」这类
//!    显然是误写的泄漏。
//! 2. **允许清单必须活着**：唯一放行的是两类站点——精度门禁自身的 token 表
//!    （`src/render/kernel/mod.rs`、`src/runtime/transfer/kernel/ptx.rs`）与刻意注入禁用
//!    token 的负对照测试（`src/runtime/transfer/kernel/tests/`）。每个允许项必须在当前源里
//!    仍然存在至少一处命中，否则清单已腐坏、测试失败，强制随代码一起清理。
//!
//! 不覆盖：`f64` 语义审计、数值容差、GPU 行为；不宣称 NP01 通过。零 GPU、只读文件。

use std::fs;
use std::path::{Path, PathBuf};

/// 允许出现禁用精度 token 的宿主源文件（路径, 最少命中行数）。
const ALLOWLIST: [(&str, usize); 4] = [
    // 渲染精度门禁自身的 token 表：它要逐 token 拒绝 f16/bf16/tf32。
    ("src/render/kernel/mod.rs", 1),
    // 物理 PTX 精度门禁自身的 token 表：同样逐 token 拒绝 f64/f16/bf16/tf32。
    ("src/runtime/transfer/kernel/ptx.rs", 1),
    // 负对照：把 `float` 改写成 `double` 来证明门禁确实拒绝。
    ("src/runtime/transfer/kernel/tests/adapter.rs", 1),
    // 负对照：注入 `cvt.rn.tf32.f32` 与 `double` 源，证明门禁拒绝。
    ("src/runtime/transfer/kernel/tests/precision.rs", 2),
];

const TOKENS: [&str; 5] = ["double", "__half", "half2", "bfloat16", "tf32"];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()));
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rust_sources(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
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

/// 去掉 Rust `//` 行注释与 `/* */` 块注释后返回该行代码区。
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
fn token_hits(path: &Path) -> Vec<(usize, String)> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut hits = Vec::new();
    let mut in_block = false;
    for (index, line) in text.lines().enumerate() {
        let code = code_part(line, &mut in_block);
        let code = code.trim();
        if code.is_empty() {
            continue;
        }
        for token in TOKENS {
            let matched = if token == "double" {
                code.split(|ch: char| !ch.is_alphanumeric() && ch != '_')
                    .any(|word| word == token)
            } else {
                code.contains(token)
            };
            if matched {
                hits.push((index + 1, code.to_string()));
                break;
            }
        }
    }
    hits
}

#[test]
fn host_rust_sources_have_no_c_style_precision_tokens() {
    let root = manifest_dir();
    let mut sources = Vec::new();
    collect_rust_sources(&root.join("src"), &mut sources);
    sources.sort();
    let mut violations = Vec::new();
    for path in &sources {
        let relative_path = relative(&root, path);
        let hits = token_hits(path);
        if hits.is_empty() {
            continue;
        }
        if ALLOWLIST
            .iter()
            .any(|(allowed, _)| *allowed == relative_path)
        {
            continue;
        }
        for (line, code) in hits {
            violations.push(format!("{relative_path}:{line} {code}"));
        }
    }
    assert!(
        violations.is_empty(),
        "产品 Rust 宿主代码区出现 C/设备宽度精度 token（只允许精度门禁自身 token 表与负对照测试）：\n{}",
        violations.join("\n")
    );
}

#[test]
fn allowlist_entries_are_still_live() {
    let root = manifest_dir();
    for (allowed, at_least) in ALLOWLIST {
        let path = root.join(allowed);
        assert!(
            path.is_file(),
            "允许清单条目 {allowed} 已不存在：请从 ALLOWLIST 删除该条目"
        );
        let hits = token_hits(&path).len();
        assert!(
            hits >= at_least,
            "允许清单条目 {allowed} 只剩 {hits} 处命中（登记至少 {at_least}）：清单已腐坏，随代码一起清理"
        );
    }
}

#[test]
fn cuda_gate_snapshot_still_covers_the_primary_surface() {
    // 第二片依赖第一片的 .cu 快照仍在：宿主 Rust 与 CUDA 两片合起来才覆盖产品精度 token 面。
    let root = manifest_dir();
    let snapshot_path = root.join("docs/development-evidence/np01-type-mapping/cuda-surface.json");
    let snapshot: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", snapshot_path.display())),
    )
    .expect("cuda-surface.json 必须存在且是合法 JSON（NP01 第一片）");
    assert_eq!(
        snapshot["totals"]["files"].as_u64(),
        Some(41),
        "CUDA 源面快照文件数应为 41（第一片口径）"
    );
    assert_eq!(
        snapshot["totals"]["globals"].as_u64(),
        Some(22),
        "CUDA 源面快照 __global__ 数应为 22（第一片口径）"
    );
}
