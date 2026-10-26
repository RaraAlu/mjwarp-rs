//! stage-2（G10@P5 CUDA 默认管线）③c 等价性窗口：**host vs device** 接触数组逐字段对照。
//!
//! 9 个已冻结 case（G2 五 + 3 primitive 输入 + sdf-box）：
//! - 设备侧：产品入口 `CollisionGpuProgram::new(0)` + `run`（NVRTC 编译、sm_89、fmad=true），
//!   每个接触 case 跑 `Contact`（单接触 ccd）与 `ContactMulti`（`use_multiccd=true` 特化），
//!   sdf-box 跑 `Sdf`；每内核连续跑两次并落盘（两次必须逐位一致）。
//! - 宿主侧：`ccd_pair`（`use_multiccd` 与设备内核同口径）/ `sdf_pair_contacts`，输入与设备
//!   逐位相同（G5 冻结 bin 原样上传；`convex-mesh`/`degenerate-mesh` 无设备 bin，复用
//!   `case_data.rs` 的 G2 静态输入合成两侧数组）。
//! - 判据：G5 冻结口径（dist 4 ULP 自身量级、pos 4·ULP(向量尺度)、frame 4 ULP@1.0、
//!   结构字段/计数/overflow 相等）；位级优先。不放宽、不过滤 case。
//!
//! 三个测试（均 `#[ignore]`，默认门禁不跑）：
//! 1. `window_product_equivalence_dump`：**窗口内**运行（唯一需要 GPU 的入口，只跑+落盘，不比对）；
//! 2. `host_vs_product_dump_compare`：窗口后 CPU 比对（读 dump，写矩阵，失败即断言）；
//! 3. `host_vs_frozen_probe_out1`：纯 CPU dry-run，用 G5 已冻结探针输出 `out1/` 验证宿主侧
//!    口径（不开窗即可跑；作为窗口前自检，不替代窗口证据）。
//!
//! 运行：
//! `cargo test --locked --test stage2_equivalence_window -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]
#![allow(clippy::needless_range_loop)]

#[path = "support/stage2_equivalence_cases.rs"]
mod cases;
#[path = "support/stage2_equivalence_cases_synth.rs"]
mod cases_synth;
#[path = "support/stage2_equivalence_compare.rs"]
mod compare;
#[path = "support/stage2_equivalence_host.rs"]
mod host;

use std::path::Path;
use std::time::Instant;

use mjwarp_rs::runtime::collision_gpu::{CollisionGpuOutput, CollisionGpuProgram, CollisionKernel};
use serde_json::{Value, json};

fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("建输出目录");
    }
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("序列化 JSON"))
        .unwrap_or_else(|error| panic!("写 {} 失败：{error}", path.display()));
}

/// 一次回读结果 → JSON（全部字段，便于审计）。
fn output_json(output: &CollisionGpuOutput) -> Value {
    json!({
        "nacon": output.nacon,
        "nccd": output.nccd,
        "overflow": output.overflow,
        "dist": output.dist,
        "pos": output.pos,
        "frame": output.frame,
        "includemargin": output.includemargin,
        "friction": output.friction,
        "solref": output.solref,
        "solreffriction": output.solreffriction,
        "solimp": output.solimp,
        "adhesion": output.adhesion,
        "dim": output.dim,
        "geom": output.geom,
        "efc_address": output.efc_address,
        "worldid": output.worldid,
        "contact_type": output.contact_type,
        "geomcollisionid": output.geomcollisionid,
    })
}

/// 窗口内唯一 GPU 入口：9 case ×（Contact/ContactMulti 或 Sdf）各跑两次并落盘，不做比对。
#[test]
#[ignore = "需要 NVIDIA GPU + NVRTC（窗口内运行；只跑+落盘）"]
fn window_product_equivalence_dump() {
    let out_dir = cases::window_dir().join("product");
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译并加载产品入口");
    println!("product dump 目录：{}", out_dir.display());
    for case in cases::ALL_CASES {
        let input = cases::device_inputs(case);
        let view = input.view();
        for kernel in cases::kernels(case) {
            let start = Instant::now();
            let first = program
                .run(*kernel, &view)
                .unwrap_or_else(|error| panic!("{case} {kernel:?} 第一次运行失败：{error}"));
            let second = program
                .run(*kernel, &view)
                .unwrap_or_else(|error| panic!("{case} {kernel:?} 第二次运行失败：{error}"));
            let elapsed_ms = start.elapsed().as_millis();
            let record = json!({
                "case": case,
                "kernel": format!("{kernel:?}"),
                "input_source": if cases::is_synth(case) { "g2-case-data" } else { "g5-frozen-bins" },
                "nworld": input.meta.nworld,
                "ngeom": input.meta.ngeom,
                "g1": input.meta.g1,
                "g2": input.meta.g2,
                "type1": input.meta.type1,
                "type2": input.meta.type2,
                "run1": output_json(&first),
                "run2": output_json(&second),
                "elapsed_ms": elapsed_ms,
            });
            let path = out_dir.join(format!("{case}.{kernel:?}.json"));
            write_json(&path, &record);
            let deterministic = compare::deterministic(
                &compare::device_rows_from_dump(&record["run1"]),
                &compare::device_rows_from_dump(&record["run2"]),
            )
            .is_ok();
            println!(
                "{case} {kernel:?}: nacon={} nccd={} 两次一致={} {} ms -> {}",
                first.nacon,
                first.nccd,
                deterministic,
                elapsed_ms,
                path.display()
            );
        }
    }
}

/// 窗口后 CPU 记录 C（**跨 recipe 包络，不作 pass/fail 判据**）：host vs product device 差异
/// 逐 case/内核记录（dist/pos/frame 差与 ULP@scale）。用户 2026-10-26 拍板 A 后 host 对齐上游
/// CPU 非融合、device 对齐 pinned GPU fused，两套 recipe 本身有已知差（G5：上游 CPU vs CUDA
/// 自身 frame 差 8.19e-7；sdf-box initpoint 分岔与 fmad=false 同类）。本测试只记录事实。
#[test]
#[ignore = "窗口 dump 生成后 CPU 运行（无需 GPU）"]
fn host_vs_product_dump_compare() {
    let dump_dir = cases::window_dir().join("product");
    let mut reports = Vec::new();
    let mut failures = Vec::new();
    for case in cases::ALL_CASES {
        for kernel in cases::kernels(case) {
            let path = dump_dir.join(format!("{case}.{kernel:?}.json"));
            let record: Value = serde_json::from_str(
                &std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("读 dump {} 失败：{error}", path.display())),
            )
            .expect("dump JSON");
            let run1 = &record["run1"];
            let mut device = compare::device_rows_from_dump(run1);
            let (host_rows, host_overflow) = if *kernel == CollisionKernel::Sdf {
                host::host_sdf_rows(case)
            } else {
                host::host_contact_rows(case, cases::host_multiccd(*kernel))
            };
            device.overflow = run1["overflow"]
                .as_array()
                .map(|values| {
                    values
                        .iter()
                        .map(|value| value.as_u64().expect("overflow") as u32)
                        .collect()
                })
                .unwrap_or_default();
            let mut report = compare::compare(
                case,
                &format!("{kernel:?}"),
                &device,
                &host_rows,
                &host_overflow,
            );
            // 两次运行必须逐位一致（数组与标量全字段）。
            if record["run1"] != record["run2"] {
                report
                    .failures
                    .push("两次运行（run1/run2）不逐位一致".to_owned());
            }
            report.print();
            reports.push(report.to_json());
            if !report.failures.is_empty() {
                failures.push(format!("{} {kernel:?}", report.case));
            }
        }
    }
    let result = json!({
        "task_id": "2026-10-25-g10-p5-cuda-default-pipeline-stage2",
        "kind": "stage2-record-c-cross-recipe-envelope",
        "criterion": "跨 recipe 记录：G5 冻结口径仅用于度量 dist/pos/frame 差与 ULP@scale；本记录不作 pass/fail 判据",
        "recipe_note": "host=上游 CPU 非融合 recipe（A 拍板）；device=pinned GPU fused fma.rn；已知上游 CPU/CUDA 自身差 frame 8.19e-7 量级，sdf-box initpoint 梯度下降分岔与 G5 fmad=false 记录同类",
        "determinism_note": "每内核窗口内连跑两次并落盘；按 G5 口径（worldid 分组+稳定排序后行集合）逐位比较，行序不作判据",
        "reports": reports,
        "cases_with_differences": failures,
    });
    let path = cases::window_dir().join("result.json");
    write_json(&path, &result);
    println!("记录 C 写入 {}", path.display());
}

/// 窗口内 device 路径真实执行且落盘 → 记录 B 与 UNC-2 的前置。
fn device_dump_executed() -> bool {
    let dump_dir = cases::window_dir().join("product");
    cases::ALL_CASES.iter().all(|case| {
        cases::kernels(case)
            .iter()
            .all(|kernel| dump_dir.join(format!("{case}.{kernel:?}.json")).is_file())
    })
}

/// 窗口后 CPU 记录 B（**device vs G5 已冻结 device 参考**，既有 device 口径：位级优先、
/// 非位级 4 ULP）。映射：contact case 用 product `Contact`（单接触 ccd）对照 G5 探针 `out1/`；
/// `sdf-box` 用 product `Sdf` 对照 `out1/`。`convex-mesh`/`degenerate-mesh` 无 G5 冻结设备
/// 参考（从未有设备 case 输入），记录为「无参考」，只做窗口内确定性与记录 C。
#[test]
#[ignore = "窗口 dump 生成后 CPU 运行（无需 GPU）"]
fn window_device_vs_frozen_reference() {
    let dump_dir = cases::window_dir().join("product");
    let mut reports = Vec::new();
    let mut failures = Vec::new();
    let mut no_reference = Vec::new();
    for case in cases::ALL_CASES {
        if cases::is_synth(case) {
            no_reference.push(case);
            println!("{case}: 无 G5 冻结 device 参考（G2-only case，从未有设备输入）");
            continue;
        }
        let kernel = if case == "sdf-box" {
            CollisionKernel::Sdf
        } else {
            CollisionKernel::Contact
        };
        let path = dump_dir.join(format!("{case}.{kernel:?}.json"));
        let record: Value = serde_json::from_str(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("读 dump {} 失败：{error}", path.display())),
        )
        .expect("dump JSON");
        let device = compare::device_rows_from_dump(&record["run1"]);
        let frozen = compare::device_rows_from_out1(&cases::bins_dir(case).join("out1"));
        let mut report = compare::compare(
            case,
            &format!("{kernel:?}-vs-out1"),
            &device,
            &frozen.rows,
            &frozen.overflow,
        );
        let second = compare::device_rows_from_dump(&record["run2"]);
        if let Err(reason) = compare::deterministic(&device, &second) {
            report.failures.push(format!(
                "两次运行（run1/run2）不自洽（G5 行序口径）：{reason}"
            ));
        }
        report.print();
        if !report.failures.is_empty() {
            failures.push(format!("{case} {kernel:?}"));
        }
        reports.push(report.to_json());
    }
    // Arch 2026-10-26 批准的分类：这两例 out1 从未作为判据参考（G5 §11.2 上游 dispatch 走
    // primitive_narrowphase），标 reference_not_comparable；其余任何红都算有效参考回归 → 停手。
    const KNOWN_NOT_COMPARABLE: [&str; 2] = ["sphere-box-deep", "capsule-box"];
    let (regressions, not_comparable): (Vec<String>, Vec<String>) =
        failures.into_iter().partition(|item| {
            !KNOWN_NOT_COMPARABLE
                .iter()
                .any(|case| item.starts_with(case))
        });
    let result = json!({
        "task_id": "2026-10-25-g10-p5-cuda-default-pipeline-stage2",
        "kind": "stage2-record-b-device-vs-frozen-device-reference",
        "criterion": "G5 既有 device 口径：位级优先；dist 自身量级 ≤4 ULP（ref==0 用 4 ULP@1.0）；pos 向量尺度 4·ULP(scale)；frame 4 ULP@1.0；结构/计数/overflow 相等；行序不作判据（worldid 分组+稳定排序后行集合逐位比较）",
        "frozen_reference": "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/<case>/out1（G5 窗口冻结探针输出，未重采）",
        "status": if regressions.is_empty() { "pass_valid_subset+reference_not_comparable（不是 B 全绿）" } else { "fail" },
        "device_executed": device_dump_executed(),
        "reference_not_comparable": not_comparable,
        "reference_not_comparable_reason": "G5 README §11.2 方法学修正：sphere-box-shallow/deep、capsule-box 上游 dispatch 走 primitive_narrowphase，不列入设备 ccd 对照；out1 只是探针 ccd 分支产物、从未与上游参考绑定",
        "no_frozen_reference": no_reference,
        "reports": reports,
        "valid_reference_regressions": regressions,
    });
    let path = cases::window_dir().join("record-b.json");
    write_json(&path, &result);
    println!("记录 B 写入 {}", path.display());
    assert!(
        regressions.is_empty(),
        "记录 B 出现有效参考回归：{regressions:?}；明细见 {}",
        path.display()
    );
}

/// 窗口后汇总：三条记录（A 引用既有 verified 证据 / B device-vs-device 参考 / C 跨 recipe 包络）
/// + UNC-2 判定（只有 device 真实执行 且 记录 B 全绿才 closed）。
#[test]
#[ignore = "窗口 dump 生成后 CPU 运行（无需 GPU）"]
fn three_record_summary() {
    let result_c: Value = serde_json::from_str(
        &std::fs::read_to_string(cases::window_dir().join("result.json"))
            .expect("先跑 host_vs_product_dump_compare 生成 result.json"),
    )
    .expect("result.json");
    let record_b: Value = serde_json::from_str(
        &std::fs::read_to_string(cases::window_dir().join("record-b.json"))
            .expect("先跑 window_device_vs_frozen_reference 生成 record-b.json"),
    )
    .expect("record-b.json");
    let b_ok = record_b["valid_reference_regressions"]
        .as_array()
        .is_some_and(|list| list.is_empty());
    let executed = record_b["device_executed"].as_bool().unwrap_or(false);
    // Arch 2026-10-26：UNC-2 本片不自行 closed，统一记 open (pending Arch/Chief confirmation)。
    let unc2_closed = false;
    let summary = json!({
        "task_id": "2026-10-25-g10-p5-cuda-default-pipeline-stage2",
        "kind": "stage2-three-records",
        "record_a_host_vs_upstream_cpu_reference": {
            "status": "cited-existing-verified-evidence（不重做）",
            "g2_five_cases": "2026-10-25-g10-p5-faithful-fma-fix-r2（50a0acd1）：5 case 与上游 CPU 参考位级 0 差异",
            "sdf_box": "G3：host sdf_pair_contacts vs 上游 f32 CPU 参考（fixtures/g10-p5/sdf-f32/sdf-box-cpu.json）≤4 ULP",
            "primitive_three_cases": "host 无 primitive 实现（ccd_pair 为 host 默认路径），不与上游 primitive 参考比较；如实记录不适用",
        },
        "record_b_device_vs_frozen_device_reference": {
            "status": if b_ok { "pass_valid_subset+reference_not_comparable（不是 B 全绿）" } else { "fail" },
            "valid_reference_regressions": record_b["valid_reference_regressions"],
            "reference_not_comparable": record_b["reference_not_comparable"],
            "no_frozen_reference": record_b["no_frozen_reference"],
        },
        "record_c_cross_recipe_envelope": {
            "status": "recorded-envelope（不作通过/失败判据）",
            "cases_with_differences": result_c["cases_with_differences"],
        },
        "device_path_really_executed": executed,
        "unc2": if unc2_closed { "closed（device 真实执行 + device vs 冻结 device 参考全绿）" } else { "open (pending Arch/Chief confirmation：device 真实执行；记录 B 有效参考子集全绿，3 红已分类 reference_not_comparable/行序 bug 已修正)" },
        "red_line": "不放宽 4 ULP、不过滤 case、不改判据/期望；record C 不写成『通过』",
    });
    let path = cases::window_dir().join("three-records.json");
    write_json(&path, &summary);
    println!("三条记录汇总写入 {}", path.display());
    println!(
        "record B valid-subset={} / device_executed={executed} / UNC-2=open (pending Arch/Chief confirmation)",
        if b_ok { "pass" } else { "fail" }
    );
    assert!(
        executed,
        "device 路径未真实执行（dump 缺失）：UNC-2 保持 open"
    );
}

/// 纯 CPU dry-run：宿主复算 vs G5 已冻结探针输出（`out1/`，不开窗即可跑）。
///
/// 用途：窗口前验证宿主侧输入/口径装配正确；**不替代**窗口内 product 入口证据。
#[test]
#[ignore = "CPU dry-run（读 G5 冻结 out1/，无需 GPU）"]
fn host_vs_frozen_probe_out1() {
    let mut reports = Vec::new();
    let mut failures = Vec::new();
    for case in cases::ALL_CASES {
        if cases::is_synth(case) {
            println!("{case}: 无 G5 冻结设备输入，跳过 dry-run（窗口内由合成输入对照）");
            continue;
        }
        let out1 = cases::bins_dir(case).join("out1");
        let device = compare::device_rows_from_out1(&out1);
        let (host_rows, host_overflow) = if case == "sdf-box" {
            host::host_sdf_rows(case)
        } else {
            host::host_contact_rows(case, false)
        };
        let report = compare::compare(
            case,
            "out1-contact/sdf",
            &device,
            &host_rows,
            &host_overflow,
        );
        report.print();
        reports.push(report.to_json());
        if !report.failures.is_empty() {
            failures.push(report.case.clone());
        }
    }
    let result = json!({
        "task_id": "2026-10-25-g10-p5-cuda-default-pipeline-stage2",
        "kind": "stage2-host-vs-frozen-probe-out1-dryrun",
        "criterion": "同 result.json.criterion（G5 冻结口径）",
        "note": "CPU dry-run，用 G5 冻结探针 out1/ 验证宿主侧装配；不替代窗口 product 入口证据",
        "reports": reports,
        "failures": failures,
    });
    let path = cases::window_dir().join("dryrun-host-vs-out1.json");
    write_json(&path, &result);
    println!("写入 {}", path.display());
    println!(
        "dry-run（仅装配自检、不作证据）：差异 case={failures:?}；明细见 {}",
        path.display()
    );
}
