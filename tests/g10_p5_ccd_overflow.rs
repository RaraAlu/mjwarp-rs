//! G10@P5 修正片：设备 CCD 溢出位（EPA `FLOAT_MAX` 时不得多置 `CCD`）与 G5 回归对照。
//!
//! 前置（GPU 窗口内，见证据 `docs/development-evidence/g10-p5-ccd-overflow-bit-fix/`）：
//! `build_probes.ps1` + `run_window.ps1` 产出 `device/`、`device-before/`、
//! `upstream-forced-horizon-run{1,2}.json`、`device-regression/`。
//!
//! 判据（Arch 2026-10-25 任务 `2026-10-25-g10-p5-ccd-overflow-bit-fix`）：
//! - 强制 horizon=2 的 3 例：设备（修正后）`overflow` 与同配方上游 CUDA **位级相等**
//!   （`EPA_HORIZON = 1<<8 = 256`，不含 `CCD = 1<<4 = 16`）；
//! - `dist`/`x1`/`x2` 位级相等、`ncontact`/`multiccd_idx` 相等、两次运行位级一致；
//! - 修正前设备（HEAD 源）在触发 `FLOAT_MAX` 的 world 为 `256|16 = 272`（反向判别力）；
//! - G5 回归：7 case 的设备输出 bin 与冻结窗口输出一致（接触数组排序行 / witness 逐文件）。
//!
//! 运行（GPU 窗口结束后，CPU 读已存盘产物）：
//! `cargo test --locked --test g10_p5_ccd_overflow -- --ignored --test-threads=1`

#[path = "support/ccd_overflow_support.rs"]
mod support;

use std::collections::BTreeMap;

use serde_json::{Value, json};
use support::*;

/// 强制 horizon=2 三例：设备（修正后）与上游 CUDA 的 overflow/dist/x1/x2/ncontact 位级对照。
/// 同时验证两次运行位级一致，以及修正前设备源确实产生 272（256|16）的反向判别力。
#[test]
#[ignore = "需要 GPU 窗口跑 run_window.ps1（device/ + device-before/ + upstream-forced-horizon-run{1,2}.json）"]
fn forced_horizon_overflow_bits_match_upstream() {
    let upstream = serde_json::from_str::<Value>(
        &std::fs::read_to_string(evidence().join("upstream-forced-horizon-run1.json"))
            .expect("上游捕获"),
    )
    .expect("上游 JSON");
    let upstream_second = serde_json::from_str::<Value>(
        &std::fs::read_to_string(evidence().join("upstream-forced-horizon-run2.json"))
            .expect("上游捕获第二份"),
    )
    .expect("上游 JSON 第二份");
    assert_eq!(
        upstream["provenance"]["authoring_script_sha256"],
        upstream_second["provenance"]["authoring_script_sha256"],
        "两次上游捕获脚本应一致"
    );

    let mut cases = serde_json::Map::new();
    let mut failures = Vec::new();
    let mut float_max_total = 0_usize;
    for case in FORCED_HORIZON_CASES {
        let device_dir = evidence().join("device").join(case);
        let before_dir = evidence().join("device-before").join(case);
        let run_inputs = |run: &str| {
            let directory = device_dir.join(run);
            (
                u32_values(&directory.join("dist.bin")),
                i32_values(&directory.join("ncontact.bin")),
                u32_values(&directory.join("x1.bin")),
                u32_values(&directory.join("x2.bin")),
                i32_values(&directory.join("multiccd_idx.bin")),
                u32_values(&directory.join("overflow.bin")),
            )
        };
        let run1 = run_inputs("run1");
        let run2 = run_inputs("run2");
        assert_eq!(run1.0, run2.0, "{case} dist 两次运行不一致");
        assert_eq!(run1.1, run2.1, "{case} ncontact 两次运行不一致");
        assert_eq!(run1.2, run2.2, "{case} x1 两次运行不一致");
        assert_eq!(run1.3, run2.3, "{case} x2 两次运行不一致");
        assert_eq!(run1.4, run2.4, "{case} multiccd_idx 两次运行不一致");
        assert_eq!(run1.5, run2.5, "{case} overflow 两次运行不一致");

        let before = &before_dir.join("run1");
        let before_overflow = u32_values(&before.join("overflow.bin"));
        let before_dist = u32_values(&before.join("dist.bin"));
        // 修正前设备只在真正触发 EPA FLOAT_MAX 的 world 多置 CCD 位（mesh-mesh world0
        // 的 EPA 正常收敛、本来无溢出，不应被盘成 272）；逐 world 期望值 = 修后位 | CCD。
        let expected_before: Vec<u32> = run1
            .5
            .iter()
            .map(|value| if *value == 0 { 0 } else { *value | CCD })
            .collect();
        for (world, value) in run1.5.iter().enumerate() {
            assert!(
                *value == 0 || *value == EPA_HORIZON,
                "{case} world{world} 修后 overflow={value} 应为 0 或 EPA_HORIZON={EPA_HORIZON}"
            );
        }
        assert_eq!(
            before_overflow, expected_before,
            "{case} 修正前设备源应在触发 FLOAT_MAX 的 world 额外置 CCD（= 修后位 | {CCD}）"
        );
        // 移除该位不改 dist（只改 overflow 位），修正前/后 dist 应位级相同。
        assert_eq!(before_dist, run1.0, "{case} 修正前/后 dist 位型不一致");

        let upstream_case = &upstream["cases"][case];
        let upstream_overflow = json_u32_array(upstream_case, "/overflow");
        let upstream_dist = json_u32_array(upstream_case, "/dist_bits");
        let upstream_x1 = json_u32_array(upstream_case, "/x1_bits");
        let upstream_x2 = json_u32_array(upstream_case, "/x2_bits");
        let upstream_ncontact = json_i32_array(upstream_case, "/ncontact");
        let upstream_idx = json_i32_array(upstream_case, "/multiccd_idx");
        let upstream_second_case = &upstream_second["cases"][case];
        assert_eq!(
            json_u32_array(upstream_second_case, "/overflow"),
            upstream_overflow,
            "{case} 上游两次捕获 overflow 不一致"
        );
        assert_eq!(
            json_u32_array(upstream_second_case, "/dist_bits"),
            upstream_dist,
            "{case} 上游两次捕获 dist 不一致"
        );

        let worlds = run1.0.len();
        let mut per_world = Vec::new();
        let mut float_max_worlds = 0_usize;
        // f32 1e30（`G5_FLOAT_MAX` / 上游 `FLOAT_MAX`）的位型：horizon 容量不足时 EPA 返回它。
        const FLOAT_MAX_BITS: u32 = 0x7149_f2ca;
        for world in 0..worlds {
            let mut world_failures = |message: &str| {
                failures.push(format!("{case} world{world}: {message}"));
            };
            let hit_float_max = upstream_dist[world] == FLOAT_MAX_BITS;
            if hit_float_max {
                float_max_worlds += 1;
            }
            // FLOAT_MAX 分支两侧只置 EPA_HORIZON；正常收敛的 world（本批仅 mesh-mesh world0）
            // 两侧 overflow 都应为 0——不能把「无溢出」也要求成 256。
            let expected_overflow = if hit_float_max { EPA_HORIZON } else { 0 };
            if upstream_overflow[world] != expected_overflow {
                world_failures(&format!(
                    "上游 overflow={}（dist {} FLOAT_MAX）应为 {expected_overflow}",
                    upstream_overflow[world],
                    if hit_float_max { "==" } else { "!=" }
                ));
            }
            if run1.5[world] != upstream_overflow[world] {
                world_failures(&format!(
                    "设备 overflow={} 上游={}",
                    run1.5[world], upstream_overflow[world]
                ));
            }
            if run1.5[world] & CCD != 0 {
                world_failures("设备 overflow 含 CCD(1<<4) 位");
            }
            if run1.0[world] != upstream_dist[world] {
                world_failures(&format!(
                    "dist bits 设备={:#x} 上游={:#x}",
                    run1.0[world], upstream_dist[world]
                ));
            }
            if run1.1[world] != upstream_ncontact[world] {
                world_failures(&format!(
                    "ncontact 设备={} 上游={}",
                    run1.1[world], upstream_ncontact[world]
                ));
            }
            if run1.4[world] != upstream_idx[world] {
                world_failures(&format!(
                    "multiccd_idx 设备={} 上游={}",
                    run1.4[world], upstream_idx[world]
                ));
            }
            for axis in 0..3 {
                let index = world * 3 + axis;
                if run1.2[index] != upstream_x1[index] {
                    world_failures(&format!(
                        "x1[{axis}] bits 设备={:#x} 上游={:#x}",
                        run1.2[index], upstream_x1[index]
                    ));
                }
                if run1.3[index] != upstream_x2[index] {
                    world_failures(&format!(
                        "x2[{axis}] bits 设备={:#x} 上游={:#x}",
                        run1.3[index], upstream_x2[index]
                    ));
                }
            }
            per_world.push(json!({
                "world": world,
                "hit_float_max": hit_float_max,
                "device_overflow": run1.5[world],
                "device_before_overflow": before_overflow[world],
                "upstream_overflow": upstream_overflow[world],
                "dist_bits": run1.0[world],
                "upstream_dist_bits": upstream_dist[world],
                "dist_bits_equal": run1.0[world] == upstream_dist[world],
                "ncontact_equal": run1.1[world] == upstream_ncontact[world],
                "multiccd_idx_equal": run1.4[world] == upstream_idx[world],
            }));
        }
        float_max_total += float_max_worlds;
        cases.insert(
            (*case).to_owned(),
            json!({
                "worlds": worlds,
                "float_max_worlds": float_max_worlds,
                "device_overflow": run1.5,
                "device_before_overflow": before_overflow,
                "upstream_overflow": upstream_overflow,
                "two_runs_bitwise_identical": true,
                "per_world": per_world,
            }),
        );
        println!(
            "{case}: worlds={worlds} float_max_worlds={float_max_worlds} overflow={:?}（修前 +CCD / 上游同值）",
            run1.5
        );
    }
    assert!(
        float_max_total >= 5,
        "FLOAT_MAX（horizon 溢出）world 总数={float_max_total}，少于 3 例的 5 个预期 world"
    );
    let result = json!({
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-ccd-overflow-bit-fix",
        "criterion": "强制 horizon 容量=2：设备 g5_ccd overflow 与同配方上游 f32 GPU 位级相等（EPA_HORIZON=256，不含 CCD=16）；dist/x1/x2 位级、ncontact/multiccd_idx 相等；两次运行一致；修正前设备源在触发 FLOAT_MAX 的 world 为 272（=256|16，反向判别），未触发 world 保持 0",
        "device_bins": "device/<case>/run{1,2}（overflow_probe_after.exe，max_horizon=2）",
        "device_before_bins": "device-before/<case>/run{1,2}（overflow_probe_before.exe，HEAD 源）",
        "upstream_capture": "upstream-forced-horizon-run{1,2}.json（capture_forced_horizon.py --device cuda）",
        "cases": cases,
        "float_max_worlds_total": float_max_total,
        "failures": failures,
    });
    write_json(&evidence().join("overflow-result.json"), &result);
    assert!(
        failures.is_empty(),
        "强制 horizon 溢出位对照失败：{failures:#?}"
    );
}

/// G5 回归：修正后设备源重跑交付 case 与 witness case，输出与 G5 冻结窗口逐文件位级一致。
#[test]
#[ignore = "需要 GPU 窗口跑 run_window.ps1（device-regression/）"]
fn g5_regression_device_outputs_unchanged() {
    let mut cases = serde_json::Map::new();
    let mut failures = Vec::new();
    let mut total_files = 0_usize;
    let mut checked = 0_usize;

    fn check_dirs(
        case: &str,
        dirs: &[&str],
        cases: &mut serde_json::Map<String, Value>,
        failures: &mut Vec<String>,
        total_files: &mut usize,
        checked: &mut usize,
    ) {
        let fresh_root = evidence().join("device-regression").join(case);
        let frozen_root = frozen_cases().join(case);
        let mut per_case = BTreeMap::new();
        for dir in dirs {
            let fresh_dir = fresh_root.join(dir);
            let frozen_dir = frozen_root.join(dir);
            let fresh = bin_names(&fresh_dir);
            let frozen = bin_names(&frozen_dir);
            *total_files += frozen.len();
            if fresh != frozen {
                failures.push(format!(
                    "{case}/{dir}: 文件集合不一致 fresh={fresh:?} frozen={frozen:?}"
                ));
                continue;
            }
            *checked += frozen.len();
            if frozen_dir.join("worldid.bin").is_file() {
                // contact 输出：行序不稳定 → 排序后按行内全字段位级比较；摘要计数直接比。
                let (fresh_rows, fresh_summary) = canonical_contact_rows(&fresh_dir);
                let (frozen_rows, frozen_summary) = canonical_contact_rows(&frozen_dir);
                if fresh_rows.len() != frozen_rows.len() {
                    failures.push(format!(
                        "{case}/{dir}: 接触行数不一致 fresh={} frozen={}",
                        fresh_rows.len(),
                        frozen_rows.len()
                    ));
                } else if fresh_rows != frozen_rows {
                    let first = fresh_rows
                        .iter()
                        .zip(frozen_rows.iter())
                        .position(|(left, right)| left != right)
                        .unwrap_or(0);
                    failures.push(format!(
                        "{case}/{dir}: 排序后接触行不一致（首差 row {first}）"
                    ));
                }
                if fresh_summary.len() != frozen_summary.len() {
                    failures.push(format!("{case}/{dir}: 摘要文件数不一致"));
                } else {
                    for ((name, fresh_bytes), (frozen_name, frozen_bytes)) in
                        fresh_summary.iter().zip(frozen_summary.iter())
                    {
                        if name != frozen_name || fresh_bytes != frozen_bytes {
                            failures.push(format!("{case}/{dir}/{name}: 摘要不一致"));
                        }
                    }
                }
                per_case.insert(
                    (*dir).to_owned(),
                    json!({ "mode": "canonical_rows", "rows": frozen_rows.len() }),
                );
            } else {
                // witness 输出：每个 world 一行、无原子行序，直接逐文件位级比较。
                for name in &frozen {
                    let fresh_bytes = read_bytes(&fresh_dir.join(name));
                    let frozen_bytes = read_bytes(&frozen_dir.join(name));
                    if fresh_bytes != frozen_bytes {
                        failures.push(format!("{case}/{dir}/{name}: 字节不一致"));
                    }
                }
                per_case.insert(
                    (*dir).to_owned(),
                    json!({ "mode": "bitwise", "files": frozen.len() }),
                );
            }
        }
        cases.insert((*case).to_owned(), json!(per_case));
    }

    for case in REGRESSION_CONTACT_CASES {
        check_dirs(
            case,
            &["out1", "out2"],
            &mut cases,
            &mut failures,
            &mut total_files,
            &mut checked,
        );
    }
    for case in REGRESSION_WITNESS_CASES {
        check_dirs(
            case,
            &["witness1", "witness2"],
            &mut cases,
            &mut failures,
            &mut total_files,
            &mut checked,
        );
    }

    let result = json!({
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-ccd-overflow-bit-fix",
        "criterion": "修正后设备源重跑 G5 交付 3 例 + ellipsoid-box + witness 3 例；contact 输出按 worldid/全字段稳定排序后逐行位级一致（G5 同口径，行序由原子计数决定不作判据），并直接比较 nacon/nccd/overflow 摘要计数；witness 输出逐文件位级一致",
        "contact_cases": REGRESSION_CONTACT_CASES,
        "witness_cases": REGRESSION_WITNESS_CASES,
        "files_compared": checked,
        "files_expected": total_files,
        "cases": cases,
        "failures": failures,
    });
    write_json(&evidence().join("regression-result.json"), &result);
    assert_eq!(checked, total_files, "回归对比文件数不足");
    assert!(failures.is_empty(), "G5 回归对照失败：{failures:#?}");
}
