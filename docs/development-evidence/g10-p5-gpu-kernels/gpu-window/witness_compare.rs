//! G5 witness 对照辅助（被 `tests/g10_p5_gpu_kernels.rs` 以 `#[path]` 引入，不是独立测试目标）。
//!
//! 背景（Arch 2026-10-25 裁决 1）：`cylinder-box`/`box-box-margin`/`mesh-mesh` 上游走
//! multicontact（设备版未实装），接触数组不做全等；改为 **`ccd()` witness 对照**：
//! - 设备侧：`witness_probe.exe`（同一批 device 源，`--fmad=true`）直调设备 `g5_ccd`；
//! - 上游侧：`capture_witness_ccd.py --device cuda` 按产品路径口径直调 pinned
//!   `collision_gjk.ccd`，取 `dist/ncontact/x1/x2/multiccd_idx/overflow`（同一份 case 输入）。
//!
//! 判据（Arch 2026-10-25 裁决，含裁决 2 的向量尺度补全）：
//! - **正式判据 = 对同配方上游 f32 GPU（`witness-cuda.json`）**：`dist` 按自身量级 ≤4 ULP
//!   （ref==0 用 4 ULP@1.0）；`x1`/`x2`（位置向量）`scale = max_i |ref_i|`（整向量全零回落 1.0），
//!   逐分量 `|dev-ref| ≤ 4·ULP(scale)`，位级优先；`ncontact` 整数相等；`OverflowType` 0；
//!   两次运行位等。判据实现/判别力自证见 `criterion_scale.rs`。
//! - `witness-cpu.json` 仅诊断：上游自身 CPU/CUDA 后端差（真值量化见 `witness-cpu.json`
//!   `sanity_vs_contact_reference`），设备结果对它只记录不判失败。
//! - `multiccd_idx` 只记录（multicontact 设备版未实装，不作为判据）。

// 证据比对辅助：字节切片解析刻意直白。
#![allow(clippy::chunks_exact_to_as_chunks)]

use serde_json::Value;
use std::path::{Path, PathBuf};

pub const WITNESS_CASES: [&str; 3] = ["cylinder-box", "box-box-margin", "mesh-mesh"];

/// 位置向量判据实现：与 `gpu_compare.rs` 共用同一模块（单一来源，不加载第二份副本；
/// 判别力断言只在 `gpu_compare` 侧的 `criterion_test.rs` 注册一次）。
use crate::gpu_compare::criterion_scale::{dist_ok, ulp_at, vector_bound, vector_scale};

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn cases_dir() -> PathBuf {
    root().join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases")
}

fn evidence_dir() -> PathBuf {
    root().join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window")
}

fn read_bin(directory: &Path, name: &str) -> Vec<u8> {
    let path = directory.join(format!("{name}.bin"));
    std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {}: {error}", path.display()))
}

fn read_f32(directory: &Path, name: &str) -> Vec<f32> {
    let bytes = read_bin(directory, name);
    assert_eq!(bytes.len() % 4, 0, "{name} 非 f32 对齐");
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn read_i32(directory: &Path, name: &str) -> Vec<i32> {
    let bytes = read_bin(directory, name);
    assert_eq!(bytes.len() % 4, 0, "{name} 非 i32 对齐");
    bytes
        .chunks_exact(4)
        .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn read_u32(directory: &Path, name: &str) -> Vec<u32> {
    let bytes = read_bin(directory, name);
    assert_eq!(bytes.len() % 4, 0, "{name} 非 u32 对齐");
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

/// f32 位型按符号排序后的整数距离（ULP 距离）。
fn ulp_diff(left: f32, right: f32) -> u32 {
    fn key(value: f32) -> i64 {
        let bits = value.to_bits() as i32;
        if bits < 0 {
            0x8000_0000_i64 - i64::from(bits)
        } else {
            i64::from(bits)
        }
    }
    (key(left) - key(right)).unsigned_abs() as u32
}

struct UpstreamWitness {
    dist: Vec<f32>,
    ncontact: Vec<i32>,
    x1: Vec<f32>,
    x2: Vec<f32>,
    multiccd_idx: Vec<i32>,
    overflow: Vec<i32>,
    worlds_per_case: Vec<usize>,
}

fn load_upstream(backend: &str) -> Option<UpstreamWitness> {
    let path = evidence_dir().join(format!("witness-{backend}.json"));
    if !path.is_file() {
        return None;
    }
    let text = std::fs::read_to_string(&path).expect("读取 witness json");
    let value: Value = serde_json::from_str(&text).expect("解析 witness json");
    let mut out = UpstreamWitness {
        dist: Vec::new(),
        ncontact: Vec::new(),
        x1: Vec::new(),
        x2: Vec::new(),
        multiccd_idx: Vec::new(),
        overflow: Vec::new(),
        worlds_per_case: Vec::new(),
    };
    // 三例按库内顺序拼接；每例 worlds 数用于逐 case 切分。
    let cases = value["cases"].as_object().expect("witness cases");
    for case in WITNESS_CASES {
        let entry = &cases[case];
        out.worlds_per_case
            .push(entry["worlds"].as_u64().expect("worlds") as usize);
        for bits in entry["witness"]["dist_bits"].as_array().expect("dist_bits") {
            out.dist
                .push(f32::from_bits(bits.as_u64().expect("bits") as u32));
        }
        for item in entry["witness"]["ncontact"].as_array().expect("ncontact") {
            out.ncontact.push(item.as_i64().expect("int") as i32);
        }
        for row in entry["witness"]["x1_bits"].as_array().expect("x1_bits") {
            for bits in row.as_array().expect("x1 row") {
                out.x1
                    .push(f32::from_bits(bits.as_u64().expect("bits") as u32));
            }
        }
        for row in entry["witness"]["x2_bits"].as_array().expect("x2_bits") {
            for bits in row.as_array().expect("x2 row") {
                out.x2
                    .push(f32::from_bits(bits.as_u64().expect("bits") as u32));
            }
        }
        for item in entry["witness"]["multiccd_idx"].as_array().expect("idx") {
            out.multiccd_idx.push(item.as_i64().expect("int") as i32);
        }
        for item in entry["witness"]["overflow"].as_array().expect("overflow") {
            out.overflow.push(item.as_i64().expect("int") as i32);
        }
    }
    Some(out)
}

struct CaseWitness {
    dist_bit_equal: usize,
    dist_components: usize,
    dist_max_ulp: u32,
    x1_bit_equal: usize,
    x1_components: usize,
    x1_scale_max: f32,
    x1_max_abs: f32,
    x1_max_ulps_at_scale: f64,
    x2_bit_equal: usize,
    x2_components: usize,
    x2_scale_max: f32,
    x2_max_abs: f32,
    x2_max_ulps_at_scale: f64,
    ncontact_equal: bool,
    multiccd_idx_equal: bool,
    overflow: Vec<u32>,
    two_runs_raw_identical: bool,
    /// 逐 world 明细（worldid 配对审计：设备数组按 world 索引写出，上游 capture 按 worldid 显式写）。
    per_world: Vec<serde_json::Value>,
}

fn compare_case(case: &str, upstream: &UpstreamWitness, offset: usize) -> CaseWitness {
    let directory = cases_dir().join(case);
    assert!(
        directory.join("witness1/dist.bin").is_file(),
        "{case}: 缺少 witness 探针输出（先在窗口运行 witness_probe.exe）"
    );
    let run1_dist = read_f32(&directory.join("witness1"), "dist");
    let run1_x1 = read_f32(&directory.join("witness1"), "x1");
    let run1_x2 = read_f32(&directory.join("witness1"), "x2");
    let run1_ncontact = read_i32(&directory.join("witness1"), "ncontact");
    let run1_idx = read_i32(&directory.join("witness1"), "multiccd_idx");
    let run1_overflow = read_u32(&directory.join("witness1"), "overflow");
    let run2_dist = read_f32(&directory.join("witness2"), "dist");
    let run2_x1 = read_f32(&directory.join("witness2"), "x1");
    let run2_x2 = read_f32(&directory.join("witness2"), "x2");
    let run2_ncontact = read_i32(&directory.join("witness2"), "ncontact");
    let run2_idx = read_i32(&directory.join("witness2"), "multiccd_idx");
    let run2_overflow = read_u32(&directory.join("witness2"), "overflow");

    let worlds = run1_dist.len();
    assert_eq!(run1_x1.len(), worlds * 3, "{case}: x1 长度");
    assert_eq!(run1_x2.len(), worlds * 3, "{case}: x2 长度");
    assert_eq!(run1_ncontact.len(), worlds, "{case}: ncontact 长度");
    assert!(
        offset + worlds <= upstream.dist.len(),
        "{case}: 上游 witness 世界数不足"
    );
    let ref_dist = &upstream.dist[offset..offset + worlds];
    let ref_x1 = &upstream.x1[offset * 3..(offset + worlds) * 3];
    let ref_x2 = &upstream.x2[offset * 3..(offset + worlds) * 3];

    let mut report = CaseWitness {
        dist_bit_equal: 0,
        dist_components: worlds,
        dist_max_ulp: 0,
        x1_bit_equal: 0,
        x1_components: worlds * 3,
        x1_scale_max: 0.0,
        x1_max_abs: 0.0,
        x1_max_ulps_at_scale: 0.0,
        x2_bit_equal: 0,
        x2_components: worlds * 3,
        x2_scale_max: 0.0,
        x2_max_abs: 0.0,
        x2_max_ulps_at_scale: 0.0,
        ncontact_equal: run1_ncontact == upstream.ncontact[offset..offset + worlds],
        multiccd_idx_equal: run1_idx == upstream.multiccd_idx[offset..offset + worlds],
        overflow: run1_overflow.clone(),
        two_runs_raw_identical: run1_dist == run2_dist
            && run1_x1 == run2_x1
            && run1_x2 == run2_x2
            && run1_ncontact == run2_ncontact
            && run1_idx == run2_idx
            && run1_overflow == run2_overflow,
        per_world: Vec::new(),
    };

    for world in 0..worlds {
        let (device_dist, ref_value) = (run1_dist[world], ref_dist[world]);
        let equal = device_dist.to_bits() == ref_value.to_bits();
        report.dist_bit_equal += usize::from(equal);
        if ref_value != 0.0 {
            report.dist_max_ulp = report.dist_max_ulp.max(ulp_diff(device_dist, ref_value));
        }
        assert!(
            dist_ok(device_dist, ref_value),
            "{case}: world {world} dist 未过（设备 {device_dist} vs 上游 {ref_value}，ULP={}）",
            ulp_diff(device_dist, ref_value)
        );
    }
    // x1/x2：位置向量，scale = max_i |ref_i|（整向量全零回落 1.0），逐分量 |dev-ref| ≤ 4·ULP(scale)。
    for (name, device, reference) in [("x1", &run1_x1, ref_x1), ("x2", &run1_x2, ref_x2)] {
        for world in 0..worlds {
            let device_slice = &device[world * 3..world * 3 + 3];
            let reference_slice = &reference[world * 3..world * 3 + 3];
            let scale = vector_scale(reference_slice);
            let bound = vector_bound(scale);
            let ulp = ulp_at(scale);
            for (component, (left, right)) in device_slice.iter().zip(reference_slice).enumerate() {
                let equal = left.to_bits() == right.to_bits();
                let difference = (left - right).abs();
                if name == "x1" {
                    report.x1_bit_equal += usize::from(equal);
                    report.x1_scale_max = report.x1_scale_max.max(scale);
                    report.x1_max_abs = report.x1_max_abs.max(difference);
                    report.x1_max_ulps_at_scale = report
                        .x1_max_ulps_at_scale
                        .max(f64::from(difference) / f64::from(ulp));
                } else {
                    report.x2_bit_equal += usize::from(equal);
                    report.x2_scale_max = report.x2_scale_max.max(scale);
                    report.x2_max_abs = report.x2_max_abs.max(difference);
                    report.x2_max_ulps_at_scale = report
                        .x2_max_ulps_at_scale
                        .max(f64::from(difference) / f64::from(ulp));
                }
                assert!(
                    difference <= bound,
                    "{case}: {name} world{world} 分量{component} 绝对差 {difference} 超过 4·ULP(scale={scale})={bound}（设备 {left} vs 上游 {right}）"
                );
            }
        }
    }
    // 逐 world 明细：设备数组 index == worldid（witness_probe 按 blockIdx 写出），
    // 上游 capture 也按 worldid 显式写出；这里把两侧同一个 worldid 逐项列出供审计。
    for world in 0..worlds {
        let x1_scale = vector_scale(&ref_x1[world * 3..world * 3 + 3]);
        let x2_scale = vector_scale(&ref_x2[world * 3..world * 3 + 3]);
        let mut x1_abs = 0.0_f32;
        let mut x2_abs = 0.0_f32;
        let mut x1_bit = 0_usize;
        let mut x2_bit = 0_usize;
        for component in 0..3 {
            let index = world * 3 + component;
            x1_abs = x1_abs.max((run1_x1[index] - ref_x1[index]).abs());
            x2_abs = x2_abs.max((run1_x2[index] - ref_x2[index]).abs());
            x1_bit += usize::from(run1_x1[index].to_bits() == ref_x1[index].to_bits());
            x2_bit += usize::from(run1_x2[index].to_bits() == ref_x2[index].to_bits());
        }
        report.per_world.push(serde_json::json!({
            "worldid": world,
            "dist_device": run1_dist[world],
            "dist_upstream": ref_dist[world],
            "dist_ulp": ulp_diff(run1_dist[world], ref_dist[world]),
            "dist_bit_equal": run1_dist[world].to_bits() == ref_dist[world].to_bits(),
            "x1_bit_equal_components": x1_bit,
            "x1_scale": x1_scale,
            "x1_max_abs": x1_abs,
            "x1_max_ulps_at_scale": f64::from(x1_abs) / f64::from(ulp_at(x1_scale)),
            "x1_bound": vector_bound(x1_scale),
            "x2_bit_equal_components": x2_bit,
            "x2_scale": x2_scale,
            "x2_max_abs": x2_abs,
            "x2_max_ulps_at_scale": f64::from(x2_abs) / f64::from(ulp_at(x2_scale)),
            "x2_bound": vector_bound(x2_scale),
            "ncontact_device": run1_ncontact[world],
            "ncontact_upstream": upstream.ncontact[offset + world],
            "multiccd_idx_device": run1_idx[world],
            "multiccd_idx_upstream": upstream.multiccd_idx[offset + world],
        }));
    }
    assert!(
        report.ncontact_equal,
        "{case}: ncontact 整数不等（设备 {:?} vs 上游 {:?}）",
        run1_ncontact,
        &upstream.ncontact[offset..offset + worlds]
    );
    assert!(
        report.overflow.iter().all(|&value| value == 0),
        "{case}: OverflowType 非 0 {:?}",
        report.overflow
    );
    assert!(
        report.two_runs_raw_identical,
        "{case}: witness 两次运行原始位不等"
    );
    report
}

pub fn run_witness_comparison() -> (Value, Vec<String>) {
    let mut cases = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    let upstream = load_upstream("cuda");
    if upstream.is_none() {
        failures.push(
            "缺少 witness-cuda.json（窗口内需运行 capture_witness_ccd.py --device cuda）".into(),
        );
    }
    if let Some(upstream) = upstream {
        let mut offset = 0_usize;
        for (index, case) in WITNESS_CASES.iter().enumerate() {
            let worlds = upstream.worlds_per_case[index];
            match std::panic::catch_unwind(|| compare_case(case, &upstream, offset)) {
                Ok(report) => {
                    cases.push(serde_json::json!({
                        "case": case,
                        "backend": "cuda",
                        "worlds": worlds,
                        "dist_bit_equal": report.dist_bit_equal,
                        "dist_components": report.dist_components,
                        "dist_max_ulp": report.dist_max_ulp,
                        "x1_bit_equal": report.x1_bit_equal,
                        "x1_components": report.x1_components,
                        "x1_scale_max": report.x1_scale_max,
                        "x1_max_abs": report.x1_max_abs,
                        "x1_max_ulps_at_scale": report.x1_max_ulps_at_scale,
                        "x2_bit_equal": report.x2_bit_equal,
                        "x2_components": report.x2_components,
                        "x2_scale_max": report.x2_scale_max,
                        "x2_max_abs": report.x2_max_abs,
                        "x2_max_ulps_at_scale": report.x2_max_ulps_at_scale,
                        "ncontact_equal": report.ncontact_equal,
                        "multiccd_idx_equal_informational": report.multiccd_idx_equal,
                        "overflow": report.overflow,
                        "two_runs_raw_bitwise_identical": report.two_runs_raw_identical,
                        "per_world": report.per_world,
                    }));
                }
                Err(payload) => {
                    let message = payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| {
                            payload
                                .downcast_ref::<&str>()
                                .map(|text| (*text).to_owned())
                        })
                        .unwrap_or_default();
                    failures.push(format!("{case}: {message}"));
                }
            }
            offset += worlds;
        }
    }
    // 诊断：上游 CPU witness（跨后端，仅记录，不判失败）。
    let cpu_diagnostic = load_upstream("cpu").map(|cpu| {
        let mut entries = Vec::new();
        let mut offset = 0_usize;
        for (index, case) in WITNESS_CASES.iter().enumerate() {
            let worlds = cpu.worlds_per_case[index];
            let directory = cases_dir().join(case);
            let device_dist = read_f32(&directory.join("witness1"), "dist");
            let mut max_ulp = 0_u32;
            for world in 0..worlds.min(device_dist.len()) {
                max_ulp = max_ulp.max(ulp_diff(device_dist[world], cpu.dist[offset + world]));
            }
            entries.push(serde_json::json!({
                "case": case,
                "dist_max_ulp_vs_upstream_cpu_diagnostic": max_ulp,
            }));
            offset += worlds;
        }
        entries
    });

    let unmet = vec![
        "multicontact 设备版未实装：三例仅做 ccd() witness 对照，接触数组不做全等（后续小片）",
        "NVRTC 产品 launcher 未接线：本对照用 nvcc 独立 witness 探针（同一批 device 源、fmad=true）",
        "--fmad=false 仅诊断；本对照只用 fmad=true",
        "上游 CPU witness 只作跨后端诊断（上游自身 CPU/CUDA 差见 witness-cpu.json），不是判据",
    ];
    let json = serde_json::json!({
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-gpu-kernels",
        "kind": "g5-device-ccd-witness-vs-upstream-ccd",
        "criterion": "对同配方上游 f32 GPU 直调 ccd 的 witness：dist 按自身量级 ≤4 ULP（ref==0 用 4 ULP@1.0）；x1/x2（位置向量）scale=max_i|ref_i|（整向量全零回落 1.0），逐分量 |dev-ref| ≤ 4·ULP(scale)，位级优先；ncontact 整数相等；OverflowType=0；两次运行原始位相等；multiccd_idx 只记录。判据实现/判别力自证见 gpu-window/criterion_scale.rs",
        "device_probe": "target/g10-p5-g5/witness_probe.exe（nvcc 12.8.93, fmad=true, -O3, sm_89；见 gpu-window/build-witness-probe-report.txt）",
        "upstream_probe": "gpu-window/capture_witness_ccd.py --device cuda（直调 pinned collision_gjk.ccd；见 gpu-window/witness-cuda.json）",
        "pairing": "按 worldid 配对：设备 witness1/2 数组按 world 索引写出（blockIdx=worldid），上游 capture 按显式 worldid 写出；逐 world 明细见每 case 的 per_world（不是按接触行顺序）",
        "cases": cases,
        "failed_cases": failures.clone(),
        "upstream_cpu_diagnostic": cpu_diagnostic,
        "unmet": unmet,
    });
    (json, failures)
}
