//! T4静态证据测试。此目标不加载GPU，也不启动Python。
#![cfg(feature = "cuda-probe")]
#[path = "support/t4_evidence.rs"]
mod evidence;

use evidence::*;
use std::{fs, path::PathBuf};

fn inventory() -> Inventory {
    serde_json::from_str(include_str!("../fixtures/precision/t4-kernels.json")).unwrap()
}

fn performance() -> (PerformancePlan, Vec<PerformanceRow>) {
    // 纯合成真值表；这些值绝不充当实测性能记录。
    let plan = PerformancePlan {
        schema: 1,
        scope: "G01-complete".into(),
        upstream_revision: UPSTREAM.into(),
        product_revision: "d".repeat(40),
        product_artifact_sha256: "e".repeat(64),
        upstream_artifact_sha256: "f".repeat(64),
        model_sha256: "a".repeat(64),
        input_sha256: "b".repeat(64),
        state_sha256: "c".repeat(64),
        worlds: vec![1, 2, 5, 513],
        updates_per_sample: 4,
        warmup: 10,
        repetitions: 3,
        boundary: "resident-g01-complete-no-transfer".into(),
    };
    let mut rows = Vec::new();
    for implementation in ["mjwarp-rs", "frozen-mjwarp"] {
        for &worlds in &plan.worlds {
            rows.push(PerformanceRow {
                implementation: implementation.into(),
                source_revision: if implementation == "mjwarp-rs" {
                    plan.product_revision.clone()
                } else {
                    plan.upstream_revision.clone()
                },
                artifact_sha256: if implementation == "mjwarp-rs" {
                    plan.product_artifact_sha256.clone()
                } else {
                    plan.upstream_artifact_sha256.clone()
                },
                plan_sha256: digest(&serde_json::to_vec(&plan).unwrap()),
                model_sha256: plan.model_sha256.clone(),
                input_sha256: plan.input_sha256.clone(),
                state_sha256: plan.state_sha256.clone(),
                worlds,
                updates_per_sample: 4,
                gpu_uuid: "synthetic-gpu".into(),
                driver: "synthetic-driver".into(),
                precision: "f32".into(),
                fmad: true,
                fast_math: false,
                warmup: plan.warmup,
                boundary: plan.boundary.clone(),
                synchronization: "before-and-after-each-sample".into(),
                timing: "steady_state".into(),
                samples_ms: vec![1.0, 1.1, 0.9],
                memory_scope: "process".into(),
                memory_metric: "peak_device_allocated_bytes".into(),
                memory_method: "allocation_events".into(),
                memory_complete: true,
                baseline_bytes: 1024,
                peak_bytes: 4096,
                correctness_passed: true,
                exclusive_gpu: true,
            });
        }
    }
    (plan, rows)
}

#[test]
fn frozen_inventory_covers_all_captured_physical_entries() {
    validate_inventory(&inventory()).unwrap();
}

#[test]
fn inventory_rejects_missing_duplicate_and_unmapped_kernels() {
    let mut value = inventory();
    value.mappings.pop();
    assert!(validate_inventory(&value).is_err());
    let mut value = inventory();
    value.mappings.push(value.mappings[0].clone());
    assert!(validate_inventory(&value).is_err());
    let mut value = inventory();
    value.artifacts.pop();
    assert!(validate_inventory(&value).is_err());
    let mut value = inventory();
    value.artifacts[1] = value.artifacts[0].clone();
    assert!(validate_inventory(&value).is_err());
    let mut value = inventory();
    let removed = value.mappings.pop().unwrap().entry;
    value.artifacts.retain(|v| v.entry != removed);
    assert!(validate_inventory(&value).is_err());
}

#[test]
fn inventory_rejects_precision_revision_and_boundary_drift() {
    for mutate in [
        |v: &mut Inventory| v.upstream_revision = "main".into(),
        |v: &mut Inventory| v.mappings[0].arithmetic = "f64".into(),
        |v: &mut Inventory| v.mappings[0].arithmetic = "integer-only".into(),
        |v: &mut Inventory| v.mappings[0].upstream_symbols.clear(),
        |v: &mut Inventory| v.mappings[0].product = "untracked.rs".into(),
        |v: &mut Inventory| v.artifacts[0].source_sha256 = "not-a-hash".into(),
        |v: &mut Inventory| v.excluded_test_entries.push("rigid_kinematics".into()),
        |v: &mut Inventory| v.remaining.clear(),
        |v: &mut Inventory| {
            v.product_sources
                .retain(|s| s.path != "src/runtime/transfer/kernel.rs")
        },
        |v: &mut Inventory| v.upstream_sources.retain(|s| s.path != "_src/types.py"),
    ] {
        let mut value = inventory();
        mutate(&mut value);
        assert!(validate_inventory(&value).is_err());
    }
}

#[test]
fn performance_accepts_only_complete_synthetic_pairs() {
    let (plan, rows) = performance();
    verify_performance(&plan, &rows).unwrap();
}

#[test]
fn performance_rejects_coverage_shrinkage_and_duplicates() {
    let (plan, mut rows) = performance();
    assert!(verify_performance(&plan, &[]).is_err());
    assert!(verify_performance(&plan, &rows[..7]).is_err());
    rows[7] = rows[6].clone();
    assert!(verify_performance(&plan, &rows).is_err());
    let (mut plan, rows) = performance();
    plan.worlds.pop();
    assert!(verify_performance(&plan, &rows).is_err());
}

#[test]
fn performance_rejects_unfair_inputs_devices_and_precision() {
    for mutate in [
        |r: &mut PerformanceRow| r.input_sha256 = "e".repeat(64),
        |r: &mut PerformanceRow| r.source_revision = "main".into(),
        |r: &mut PerformanceRow| r.artifact_sha256 = "0".repeat(64),
        |r: &mut PerformanceRow| r.model_sha256 = "e".repeat(64),
        |r: &mut PerformanceRow| r.state_sha256 = "e".repeat(64),
        |r: &mut PerformanceRow| r.plan_sha256 = "e".repeat(64),
        |r: &mut PerformanceRow| r.gpu_uuid = "other-gpu".into(),
        |r: &mut PerformanceRow| r.driver = "other-driver".into(),
        |r: &mut PerformanceRow| r.precision = "f64".into(),
        |r: &mut PerformanceRow| r.precision = "tf32".into(),
        |r: &mut PerformanceRow| r.fmad = false,
        |r: &mut PerformanceRow| r.fast_math = true,
        |r: &mut PerformanceRow| r.correctness_passed = false,
        |r: &mut PerformanceRow| r.exclusive_gpu = false,
        |r: &mut PerformanceRow| r.updates_per_sample = 1,
    ] {
        let (plan, mut rows) = performance();
        mutate(&mut rows[7]);
        assert!(verify_performance(&plan, &rows).is_err());
    }
}

#[test]
fn performance_rejects_cold_unwarmed_unsynchronized_or_invalid_timings() {
    for mutate in [
        |r: &mut PerformanceRow| r.timing = "cold_start".into(),
        |r: &mut PerformanceRow| r.warmup = 0,
        |r: &mut PerformanceRow| r.boundary = "includes_compile_and_transfer".into(),
        |r: &mut PerformanceRow| r.synchronization = "none".into(),
        |r: &mut PerformanceRow| r.samples_ms.clear(),
        |r: &mut PerformanceRow| r.samples_ms[0] = 0.0,
        |r: &mut PerformanceRow| r.samples_ms[0] = -1.0,
        |r: &mut PerformanceRow| r.samples_ms[0] = f64::NAN,
        |r: &mut PerformanceRow| r.samples_ms[0] = f64::INFINITY,
    ] {
        let (plan, mut rows) = performance();
        mutate(&mut rows[0]);
        assert!(verify_performance(&plan, &rows).is_err());
    }
}

#[test]
fn performance_rejects_board_snapshots_and_incomplete_peak_tracking() {
    for mutate in [
        |r: &mut PerformanceRow| r.memory_scope = "board".into(),
        |r: &mut PerformanceRow| r.memory_metric = "before_after_snapshot".into(),
        |r: &mut PerformanceRow| r.memory_method = "polling".into(),
        |r: &mut PerformanceRow| r.memory_complete = false,
        |r: &mut PerformanceRow| r.peak_bytes = 0,
        |r: &mut PerformanceRow| r.peak_bytes = r.baseline_bytes - 1,
    ] {
        let (plan, mut rows) = performance();
        mutate(&mut rows[0]);
        assert!(verify_performance(&plan, &rows).is_err());
    }
}

#[test]
fn strict_schema_rejects_missing_or_extra_evidence() {
    let (plan, rows) = performance();
    let mut row = serde_json::to_value(&rows[0]).unwrap();
    row.as_object_mut().unwrap().remove("memory_complete");
    assert!(serde_json::from_value::<PerformanceRow>(row).is_err());
    let mut plan = serde_json::to_value(plan).unwrap();
    plan["accept_missing"] = true.into();
    assert!(serde_json::from_value::<PerformancePlan>(plan).is_err());
}

#[test]
fn source_checks_reject_drift_and_path_escape_before_upstream_access() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let absent = root.join("target/nonexistent-upstream");
    let mut value = inventory();
    value.product_sources[0].sha256 = "0".repeat(64);
    assert!(
        verify_sources(&value, &root, &absent)
            .unwrap_err()
            .contains("source drift")
    );
    for path in [
        "../outside.rs",
        "/outside.rs",
        "C:/outside.rs",
        "..\\outside.rs",
    ] {
        let mut value = inventory();
        let old = value.product_sources[0].path.clone();
        value.product_sources[0].path = path.into();
        for mapping in &mut value.mappings {
            if mapping.product == old {
                mapping.product = path.into();
            }
        }
        assert!(verify_sources(&value, &root, &absent).is_err());
    }
}

#[test]
#[ignore = "requires archived T4 files and frozen upstream sources; host-only"]
fn audit_local_t4_precision_without_gpu_execution() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let capture =
        PathBuf::from(std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("set capture root"));
    let upstream = PathBuf::from(
        std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream source root"),
    );
    let value = inventory();
    verify_sources(&value, &root, &upstream).unwrap();
    for mode in ["debug", "release"] {
        verify_capture(&value, &capture.join(format!("{mode}-ptx"))).unwrap();
    }
    // 用独立副本验证真实文件破坏；不触碰原始验收资料。
    let scratch = root
        .join("target")
        .join(format!("t4-evidence-negative-{}", std::process::id()));
    fs::create_dir(&scratch).unwrap();
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(scratch);
    for artifact in &value.artifacts {
        for extension in ["json", "cu", "ptx"] {
            let file = format!("{}-{}.{extension}", artifact.entry, artifact.ptx_sha256);
            fs::copy(capture.join("debug-ptx").join(&file), scratch.0.join(file)).unwrap();
        }
    }
    verify_capture(&value, &scratch.0).unwrap();
    let first = &value.artifacts[0];
    for extension in ["json", "cu", "ptx"] {
        let file = scratch
            .0
            .join(format!("{}-{}.{extension}", first.entry, first.ptx_sha256));
        let original = fs::read(&file).unwrap();
        fs::write(&file, b"corrupt evidence").unwrap();
        assert!(verify_capture(&value, &scratch.0).is_err());
        fs::write(&file, &original).unwrap();
        fs::remove_file(&file).unwrap();
        assert!(verify_capture(&value, &scratch.0).is_err());
        fs::write(&file, original).unwrap();
    }
    let receipt = scratch
        .0
        .join(format!("{}-{}.json", first.entry, first.ptx_sha256));
    fs::copy(&receipt, scratch.0.join("duplicate.json")).unwrap();
    assert!(verify_capture(&value, &scratch.0).is_err());
    let report = serde_json::json!({
        "schema": 1,
        "scope": value.scope,
        "upstream_revision": value.upstream_revision,
        "source_revision": value.source_revision,
        "inventory_sha256": digest(include_bytes!("../fixtures/precision/t4-kernels.json")),
        "kernel_entries": value.mappings.len(),
        "variants_per_mode": value.artifacts.len(),
        "modes": ["debug", "release"],
        "source_and_capture_integrity_passed": true,
        "product_gpu_executions": 0,
        "np01_complete": false,
        "np02_complete": false,
        "np04_measured": false,
        "full_t4_admitted": false,
        "remaining": value.remaining,
    });
    println!("T4_EVIDENCE {}", serde_json::to_string(&report).unwrap());
    if let Some(path) = std::env::var_os("MJWARP_T4_AUDIT_REPORT") {
        fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
}
