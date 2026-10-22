//! 输出：JSON 落盘与终端摘要。

use serde_json::Value;
use std::path::Path;

pub fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|e| format!("序列化 JSON: {e}"))?;
    text.push('\n');
    std::fs::write(path, text).map_err(|e| format!("写 {}: {e}", path.display()))
}

pub fn print_summary(report: &Value) {
    println!("== U038 PTX 逐表达式对照 ==");
    if let Some(compiler) = report["inputs"]["upstream_frozen_ptx"]["compiler_line"].as_str() {
        println!("冻结上游 PTX: {compiler}");
    }
    println!(
        "冻结上游 CUDA sha256: {}",
        report["inputs"]["upstream_generated_cuda"]["sha256"]
            .as_str()
            .unwrap_or("")
    );
    println!(
        "NVRTC: {}",
        report["inputs"]["nvrtc_dll"]["sha256"]
            .as_str()
            .unwrap_or("缺失")
    );
    println!(
        "lineinfo 指令不变: {}",
        report["validation"]["lineinfo_instruction_invariant"]
    );
    let frozen = &report["validation"]["upstream_recompile_vs_frozen"];
    println!(
        "本机重编译 vs 冻结 PTX：{}/{} 个入口指令流完全一致（不一致：{}）",
        frozen["entries_with_identical_instruction_stream"],
        frozen["entries_compared"],
        frozen["mismatched_entries"]
            .as_array()
            .map(|items| items.len())
            .unwrap_or(0)
    );
    let lineinfo = &report["validation"]["upstream_lineinfo_recompile_vs_frozen"];
    println!(
        "lineinfo 重编译 vs 冻结 PTX：{}/{} 个入口指令流完全一致",
        lineinfo["entries_with_identical_instruction_stream"], lineinfo["entries_compared"]
    );
    for row in report["expressions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        let id = row["id"].as_str().unwrap_or("");
        let source_form = &row["source_form_diff"];
        println!(
            "表达式 {id}: 源形态差异={}（产品 fma {} / 上游 fma {}）",
            source_form["diff_type"].as_str().unwrap_or(""),
            source_form["product_counts"]["fma"],
            source_form["upstream_counts"]["fma"]
        );
    }
    let candidate = &report["candidate_transcriptions"];
    if candidate["expression"].as_str().is_some() && candidate["missing"].is_null() {
        println!(
            "rotate 候选转录 vs 上游源形态：{}；候选模式在内核出现（产品 {} 处 / 冻结上游 {} 处）",
            candidate["candidate_source_form"]["diff_type"]
                .as_str()
                .unwrap_or(""),
            candidate["candidate_occurrences_in_product_kernel"]
                .as_array()
                .map(|v| v.len())
                .unwrap_or(0),
            candidate["candidate_occurrences_in_frozen_upstream_kernel"]
                .as_array()
                .map(|v| v.len())
                .unwrap_or(0)
        );
    }
    if let Some(rotate) = report["context_sensitivity"]["rotate"].as_object() {
        println!("上下文敏感性（rotate，q-first / v-first 的 fma 数）：");
        for key in ["product", "upstream", "candidate"] {
            let first = &rotate[format!("{key}_q_first").as_str()]["counts"]["fma"];
            let second = &rotate[format!("{key}_v_first").as_str()]["counts"]["fma"];
            println!("  {key}: {first} / {second}");
        }
        println!(
            "  v-first 上游探针在冻结上游内核命中：{:?}",
            rotate["upstream_v_first_occurrences_in_frozen_kernel"]
        );
        println!(
            "  v-first 产品探针在产品内核命中：{:?}",
            rotate["product_v_first_occurrences_in_product_kernel"]
        );
    }
    for row in report["in_situ"].as_array().cloned().unwrap_or_default() {
        println!(
            "命中 {}: 产品内核 {} 处 / 冻结上游 {} 处 / 重编译上游 {} 处（期望 {}）",
            row["expression"].as_str().unwrap_or(""),
            row["product_probe_occurrences"]
                .as_array()
                .map(|v| v.len())
                .unwrap_or(0),
            row["upstream_probe_warp_options_occurrences"]
                .as_array()
                .map(|v| v.len())
                .unwrap_or(0),
            row["upstream_recompiled_occurrences"]
                .as_array()
                .map(|v| v.len())
                .unwrap_or(0),
            row["expected_instances"]
        );
    }
    for row in report["com_cdof"].as_array().cloned().unwrap_or_default() {
        println!(
            "com/cdof {} {}: fma {} / mul {} / add {} / sub {}",
            row["side"].as_str().unwrap_or(""),
            row["kernel"].as_str().unwrap_or(""),
            row["counts"]["fma"],
            row["counts"]["mul"],
            row["counts"]["add"],
            row["counts"]["sub"]
        );
    }
}
