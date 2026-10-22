//! 编译选项、入口读取与 JSON 视图小工具。

use crate::diff;
use crate::diff::kind_sequence;
use crate::extract;
use crate::nvrtc;
use crate::ptx::{self, Counts, Entry, Program};
use serde_json::{json, Value};
use std::path::Path;

// ---------- 编译选项 ----------

pub fn product_options(lineinfo: bool) -> Vec<String> {
    let mut options = vec![
        "--std=c++17".to_string(),
        "--gpu-architecture=compute_70".to_string(),
    ];
    if lineinfo {
        options.push("--generate-line-info".to_string());
    }
    options
}

/// 复制 Warp 1.15 `wp_cuda_compile_program` 的 release 选项（PCH/Ofast-compile 与
/// NVRTC 12.8 无关，故省略），用于本机 NVRTC 重编译上游生成 CUDA。
pub fn warp_options(warp_include: &Path, lineinfo: bool) -> Vec<String> {
    let mut options = vec![
        "--gpu-architecture=compute_89".to_string(),
        format!("--include-path={}", warp_include.display()),
        "--std=c++17".to_string(),
        "--define-macro=NDEBUG".to_string(),
        "--undefine-macro=WP_VERIFY_FP".to_string(),
        "--define-macro=WP_ENABLE_MATHDX=0".to_string(),
        "--device-as-default-execution-space".to_string(),
        "--extra-device-vectorization".to_string(),
        "--restrict".to_string(),
        "--diag-suppress=177,550".to_string(),
    ];
    if lineinfo {
        options.push("--generate-line-info".to_string());
    }
    options
}

pub fn with_include(options: &[String], include: &Path) -> Vec<String> {
    let mut options = options.to_vec();
    options.push(format!("--include-path={}", include.display()));
    // Warp 头文件在 JIT 模式下要求未标注函数按设备函数处理（Warp 自己也传该开关）。
    options.push("--device-as-default-execution-space".to_string());
    options
}

// ---------- 编译与入口 ----------

pub fn compile(source: &str, options: &[String], program: &str) -> Result<nvrtc::Compiled, String> {
    nvrtc::compile(&nvrtc::CompileSpec {
        name: program.to_string(),
        source: source.to_string(),
        options: options.to_vec(),
        fmad: true,
        fast_math: false,
    })
}

pub fn probe_entry(source: &str, options: &[String], entry: &str) -> Result<Entry, String> {
    let compiled = compile(source, options, "probe.cu")?;
    let program = Program::parse(&compiled.ptx);
    Ok(require_entry(&program, entry)?.clone())
}

pub fn write_file(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("写 {}: {e}", path.display()))
}

pub fn require_entry<'a>(program: &'a Program, name: &str) -> Result<&'a Entry, String> {
    program
        .entry(name)
        .ok_or_else(|| format!("PTX 缺少入口 {name}"))
}

pub fn frozen_entry<'a>(frozen: &'a Program, name: &str) -> Result<&'a Entry, String> {
    require_entry(frozen, name)
}

pub fn instruction_diff(a: &Entry, b: &Entry) -> (bool, usize) {
    if a.instrs.len() != b.instrs.len() {
        return (false, a.instrs.len().abs_diff(b.instrs.len()));
    }
    let mut mismatched = 0usize;
    for (left, right) in a.instrs.iter().zip(&b.instrs) {
        if left.text != right.text {
            mismatched += 1;
        }
    }
    (mismatched == 0, mismatched)
}

pub fn counts(entry: &Entry) -> Counts {
    Counts::of(&entry.arith())
}

pub fn atomic_adds(entry: &Entry) -> usize {
    entry
        .arith()
        .iter()
        .filter(|instr| {
            (instr.opcode.starts_with("atom.") || instr.opcode.starts_with("red."))
                && instr.opcode.contains(".add.f32")
        })
        .count()
}

// ---------- JSON 视图 ----------

pub fn counts_json(counts: &Counts) -> Value {
    json!({
        "total": counts.total,
        "fma": counts.fma,
        "mul": counts.mul,
        "add": counts.add,
        "sub": counts.sub,
        "div": counts.div,
        "sqrt": counts.sqrt,
        "rcp": counts.rcp,
        "neg": counts.neg,
        "unfused_mul": counts.unfused_mul(),
        "unfused_add": counts.unfused_add(),
    })
}

pub fn entry_json(entry: &Entry, compiled: &nvrtc::Compiled) -> Value {
    json!({
        "entry": entry.name,
        "source_sha256": compiled.source_sha256,
        "ptx_sha256": compiled.ptx_sha256,
        "options": compiled.options,
        "counts": counts_json(&counts(entry)),
        "kind_sequence": kind_sequence(&entry.kinds()),
        "instructions": entry.texts(),
    })
}

pub fn compile_json(compiled: &nvrtc::Compiled, options: &[String]) -> Value {
    json!({
        "source_sha256": compiled.source_sha256,
        "ptx_sha256": compiled.ptx_sha256,
        "ptx_bytes": compiled.ptx.len(),
        "options": options,
        "fmad": true,
        "use_fast_math": false,
    })
}

pub fn comparison_json(comparison: &diff::Comparison) -> Value {
    json!({
        "diff_type": comparison.diff_type,
        "upstream_counts": counts_json(&comparison.upstream_counts),
        "product_counts": counts_json(&comparison.product_counts),
        "upstream_kind_sequence": kind_sequence(&comparison.upstream_kinds),
        "product_kind_sequence": kind_sequence(&comparison.product_kinds),
        "upstream_fma_positions": fma_positions(&comparison.upstream_kinds),
        "product_fma_positions": fma_positions(&comparison.product_kinds),
        "notes": comparison.notes,
    })
}

pub fn entry_sequence_json(entry: &Entry) -> Value {
    json!({
        "counts": counts_json(&counts(entry)),
        "kind_sequence": kind_sequence(&entry.kinds()),
        "fma_positions": fma_positions(&entry.kinds()),
    })
}

pub fn fma_positions(kinds: &[ptx::Kind]) -> Vec<usize> {
    kinds
        .iter()
        .enumerate()
        .filter(|(_, kind)| **kind == ptx::Kind::Fma)
        .map(|(index, _)| index)
        .collect()
}

pub fn compiler_line(text: &str) -> String {
    text.lines()
        .find(|line| line.contains("Cuda compilation tools"))
        .map(|line| line.trim().to_string())
        .unwrap_or_default()
}

pub fn dll_identity(cuda_bin: &Path) -> Value {
    let candidates = ["nvrtc64_120_0.dll", "nvrtc64_120_0.alt.dll"];
    for name in candidates {
        let path = cuda_bin.join(name);
        if let Ok((bytes, sha)) = extract::file_identity(&path) {
            return json!({"name": name, "bytes": bytes, "sha256": sha, "directory": cuda_bin.display().to_string()});
        }
    }
    json!({"missing": true, "directory": cuda_bin.display().to_string()})
}

pub fn display_relative(path: &Path, repo: &Path) -> String {
    path.strip_prefix(repo)
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}
