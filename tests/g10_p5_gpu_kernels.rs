//! G10@P5 G5：设备内核 GPU 对照入口（独立 nvcc 探针输出 vs 上游 CUDA 参考）。
//!
//! 前置（GPU 窗口内，见证据 `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/`）：
//! 1. `prepare_cases.py --device cuda` → `cases/<case>/`（输入 + 本窗口参考绑定）；
//! 2. `build_probe.ps1` → `target/g10-p5-g5/probe.exe`（同一批 device 源、fmad=true）；
//! 3. 探针每 case 跑两次：`probe.exe cases/<case> 2` → `out1|out2/`。
//!
//! 判据与全部比对逻辑见 `tests/common/gpu_compare.rs`（Arch 2026-10-25 裁决：
//! frame 按 scale=1.0 的 4 ULP 绝对差，dist/pos 按各自量级 ULP，参数/整数位级）。
//!
//! 运行（GPU 窗口结束后，CPU 侧）：
//! `cargo test --locked --features cuda-probe --test g10_p5_gpu_kernels -- --ignored --test-threads=1`

#![cfg(feature = "cuda-probe")]

#[path = "../docs/development-evidence/g10-p5-gpu-kernels/gpu-window/gpu_compare.rs"]
mod gpu_compare;

use serde_json::Value;

#[test]
#[ignore = "需要先在本机 GPU 窗口跑 prepare_cases.py --device cuda + build_probe.ps1 + probe.exe"]
fn device_contact_arrays_match_upstream_cuda_reference() {
    let (result, failures) = gpu_compare::run_contact_comparison();
    let path = gpu_compare::root()
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/result.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&result).expect("json"))
        .expect("写 result.json");
    println!("wrote {}", path.display());
    assert!(
        failures.is_empty(),
        "设备对照未全部通过（未通过 case: {failures:?}）；明细见 {}",
        path.display()
    );
}

/// 结构自检（不需要 GPU）：窗口产物齐全且语义正确。
#[test]
#[ignore = "GPU 窗口产物（build-probe-report.txt / prepare-cases-report.json / result.json）生成后运行"]
fn gpu_window_evidence_is_complete() {
    let base = gpu_compare::root().join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window");
    let build = std::fs::read_to_string(base.join("build-probe-report.txt")).expect("build 报告");
    for needle in [
        "fmad=true",
        "arch=sm_89",
        "probe_binary_sha256=",
        "source_sha256 contact.cu",
        "source_sha256 hfield.cu",
        "source_sha256 sdf_narrowphase.cu",
    ] {
        assert!(build.contains(needle), "build 报告缺少 {needle}");
    }
    let prepare =
        std::fs::read_to_string(base.join("prepare-cases-report.json")).expect("准备报告");
    let prepared: Value = serde_json::from_str(&prepare).expect("解析准备报告");
    assert_eq!(
        prepared["device"], "cuda",
        "输入必须用 CUDA 后端准备（绑定窗口）"
    );
    assert_eq!(prepared["all_matches_reference_sorted_bitwise"], true);
    assert_eq!(
        prepared["cases"].as_object().expect("cases").len(),
        10,
        "10 个 case"
    );
    for case in gpu_compare::CONTACT_CASES {
        assert_eq!(
            prepared["cases"][case]["contact_run"]["matches_reference_sorted_bitwise"], true,
            "{case}: 本窗口 CUDA 重跑必须与交付参考排序位级相等"
        );
    }
    let result: Value =
        serde_json::from_str(&std::fs::read_to_string(base.join("result.json")).expect("result"))
            .expect("解析 result");
    assert_eq!(
        result["cases"].as_array().expect("cases").len()
            + result["failed_cases"].as_array().expect("failed").len(),
        gpu_compare::CONTACT_CASES.len(),
        "对照 case 总数"
    );
    assert!(
        result["criterion"]
            .as_str()
            .expect("criterion")
            .contains("4 ULP@1.0"),
        "判据必须写明 frame 的 4 ULP@1.0 口径"
    );
}
