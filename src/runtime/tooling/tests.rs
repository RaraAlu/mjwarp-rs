//! A10@P10 工具测试：无头测速记录/报告、状态导出 JSON、错误面，以及批次 2
//! 此前切片的 API 复核。

use super::*;
use serde_json::Value;

#[test]
fn recorder_aggregates_phases_in_insertion_order() {
    let mut recorder = BenchmarkRecorder::new();
    recorder.record("host", 100).unwrap();
    recorder.record("host", 300).unwrap();
    recorder.record("export", 50).unwrap();
    recorder.record("host", 200).unwrap();
    let report = recorder.report();
    assert_eq!(report.phases.len(), 2);
    assert_eq!(report.phases[0].name, "host");
    assert_eq!(report.phases[0].count, 3);
    assert_eq!(report.phases[0].total_ns, 600);
    assert_eq!(report.phases[0].min_ns, 100);
    assert_eq!(report.phases[0].max_ns, 300);
    assert_eq!(report.phases[0].mean_ns, 200.0);
    assert_eq!(report.phases[0].median_ns, 200.0);
    assert_eq!(report.phases[1].name, "export");
    assert_eq!(report.phases[1].median_ns, 50.0);
    assert_eq!(report.total_ns, 650);
    assert_eq!(report.total_samples, 4);
    assert_eq!(
        recorder.record("", 1).unwrap_err(),
        ToolingError::EmptyPhase { name: "" }
    );
}

#[test]
fn measure_is_headless_and_rejects_zero_iterations() {
    assert_eq!(measure(0, || {}).unwrap_err(), ToolingError::ZeroIterations);
    let mut counter = 0_u64;
    // release 优化下极短负载的耗时可能落在 Instant 计时粒度内（0 ns 合法），
    // 因此每次迭代做可测的 black_box 自增，保留 nanos > 0 的严格断言。
    let nanos = measure(3, || {
        for _ in 0..10_000 {
            counter = std::hint::black_box(counter + 1);
        }
    })
    .unwrap();
    assert_eq!(counter, 30_000);
    assert!(nanos > 0);
}

#[test]
fn state_export_round_trips_through_json() {
    let qpos = [0.1_f32, -0.2, 0.3];
    let qvel = [1.0_f32, -2.5, 0.0];
    let state = StateExport {
        time: 0.002_f32,
        nworld: 2,
        qpos: &qpos,
        qvel: &qvel,
    };
    let json = export_state_json(&state).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["time"].as_f64().unwrap() as f32, 0.002_f32);
    assert_eq!(value["nworld"].as_u64().unwrap(), 2);
    let decoded_qpos: Vec<f32> = value["qpos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect();
    let decoded_qvel: Vec<f32> = value["qvel"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect();
    assert_eq!(decoded_qpos, qpos);
    assert_eq!(decoded_qvel, qvel);

    let error = export_state_json(&StateExport {
        time: f32::NAN,
        nworld: 1,
        qpos: &qpos,
        qvel: &qvel,
    })
    .unwrap_err();
    assert_eq!(
        error,
        ToolingError::NonFinite {
            field: "time",
            index: 0
        }
    );
    let error = export_state_json(&StateExport {
        time: 0.0,
        nworld: 0,
        qpos: &qpos,
        qvel: &qvel,
    })
    .unwrap_err();
    assert!(matches!(error, ToolingError::InvalidState { .. }));
    let error = export_state_json(&StateExport {
        time: 0.0,
        nworld: 1,
        qpos: &qpos,
        qvel: &qvel[..2],
    })
    .unwrap_err();
    assert!(matches!(error, ToolingError::InvalidState { .. }));
}

#[test]
fn report_export_is_valid_json() {
    let mut recorder = BenchmarkRecorder::new();
    for sample in [10_u64, 20, 30] {
        recorder.record("physics", sample).unwrap();
    }
    recorder.record("io", 5).unwrap();
    let report = recorder.report();
    let json = export_report_json(&report);
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["total_samples"].as_u64().unwrap(), 4);
    assert_eq!(value["total_ns"].as_u64().unwrap(), 65);
    let phases = value["phases"].as_array().unwrap();
    assert_eq!(phases.len(), 2);
    assert_eq!(phases[0]["name"].as_str().unwrap(), "physics");
    assert_eq!(phases[0]["count"].as_u64().unwrap(), 3);
    assert_eq!(phases[0]["min_ns"].as_u64().unwrap(), 10);
    assert_eq!(phases[0]["max_ns"].as_u64().unwrap(), 30);
    assert_eq!(phases[1]["name"].as_str().unwrap(), "io");
}

/// 末片复核：批次 2 此前交付的公开 API 仍可用（冒烟级复核，不重跑物理参考）。
#[test]
fn batch_two_public_apis_still_work() {
    // G07：宽相 API 的公开错误类型可构造/匹配。
    let filter = crate::physics::BroadphaseFilter::ALL;
    assert_eq!(filter.bits(), 15);
    assert_eq!(crate::physics::BroadphaseType::Nxn.as_i32(), 0);

    // G16：四元数积分保持单位范数；速度推进与解析式一致。
    let quat = crate::physics::quat_integrate([1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.01);
    let norm =
        (quat[0] * quat[0] + quat[1] * quat[1] + quat[2] * quat[2] + quat[3] * quat[3]).sqrt();
    assert!((norm - 1.0).abs() < 1.0e-6, "quat norm {norm}");
    let velocities = crate::physics::next_velocity(1, 1, &[1.0], &[2.0], 0.5, 1.0).unwrap();
    assert_eq!(velocities, vec![2.0]);

    // G20：单 DOF 质量阵乘法。
    let mass = crate::physics::SparseMass {
        nv: 1,
        nnz: 1,
        mulm_nnz: 1,
        mulm_rowadr: &[0, 1],
        mulm_col: &[0],
        mulm_madr: &[0],
        values: &[3.0],
    };
    let ma = crate::physics::mul_m(&mass, 1, &[2.0]).unwrap();
    assert_eq!(ma, vec![6.0]);

    // G21：多项式阻尼导数解析式。
    let derivative = crate::physics::poly_force_deriv(0.5, [0.2, 0.1], 2.0);
    assert_eq!(derivative, 2.5);

    // G23：单传感器位置写出（JOINTPOS=9、REAL=0 为 mjtSensor/mjtDataType 常量值）。
    let sensor_layout = crate::physics::SensorLayout {
        nq: 1,
        nv: 1,
        nsensordata: 1,
        sensor_type: &[9],
        sensor_datatype: &[0],
        sensor_objid: &[0],
        sensor_adr: &[0],
        sensor_cutoff: &[0.0],
        jnt_qposadr: &[0],
        jnt_dofadr: &[0],
    };
    let sensordata = crate::physics::sensor_pos(&sensor_layout, 1, &[0.75], &[0.0]).unwrap();
    assert_eq!(sensordata, vec![0.75]);

    // A08：借用视图指针不变。
    let mirror = crate::runtime::external::view::MirrorBuffer::new(
        "recheck",
        0,
        0,
        0,
        3,
        vec![1.0, 2.0, 3.0, 4.0],
    );
    let view = crate::runtime::external::view::ZeroCopyView::new(
        &mirror,
        crate::runtime::external::view::FieldLayout::contiguous(1, 2, 1),
        crate::runtime::external::view::StreamFence::completed(3, 0),
    )
    .unwrap();
    let field = view.field().unwrap();
    assert_eq!(field, &[2.0, 3.0]);
    assert_eq!(
        field.as_ptr() as usize,
        mirror.data().as_ptr() as usize + std::mem::size_of::<f32>()
    );
}

// ---- A10@P11 性能记录（宿主侧数据面，不需要 GPU） ----

fn test_environment() -> EnvironmentFacts {
    EnvironmentFacts::capture("NVIDIA Test GPU", (8, 9), 12080)
}

fn test_memory(baseline: u64, peak: u64, end: u64) -> MemoryStats {
    MemoryStats {
        total_bytes: 16 * 1024 * 1024 * 1024,
        baseline_used_bytes: baseline,
        peak_used_bytes: peak,
        end_used_bytes: end,
        samples: 12,
    }
}

fn test_report() -> PerformanceReport {
    let plan = BenchmarkPlan::new(2, 5).unwrap();
    let mut report = PerformanceReport::new(TASK_ID, "probe/8", test_environment(), plan);
    report.push_record(
        PerformanceRecord::new(
            "phase",
            vec![100, 300, 200, 400, 250],
            test_memory(10, 20, 15),
        )
        .unwrap(),
    );
    report.push_checkpoint(MemoryCheckpoint {
        label: "idle".into(),
        used_bytes: 100,
        total_bytes: 200,
    });
    report.push_note("固定负载为探针微基准，不是物理阶段");
    report
}

#[test]
fn plan_requires_warmup_and_repetitions() {
    assert_eq!(
        BenchmarkPlan::new(0, 1).unwrap_err(),
        ToolingError::ZeroWarmup
    );
    assert_eq!(
        BenchmarkPlan::new(1, 0).unwrap_err(),
        ToolingError::ZeroIterations
    );
    let plan = BenchmarkPlan::new(3, 7).unwrap();
    assert_eq!(plan.warmup, 3);
    assert_eq!(plan.repetitions, 7);
}

#[test]
fn timing_stats_aggregate_matches_p10_recorder() {
    let samples = [100_u64, 300, 200];
    let stats = TimingStats::from_samples(&samples).unwrap();
    assert_eq!(stats.count, 3);
    assert_eq!(stats.total_ns, 600);
    assert_eq!(stats.min_ns, 100);
    assert_eq!(stats.max_ns, 300);
    assert_eq!(stats.mean_ns, 200.0);
    assert_eq!(stats.median_ns, 200.0);

    // 与 P10 `BenchmarkRecorder` 的报告口径一致（同一输入同一统计）。
    let mut recorder = BenchmarkRecorder::new();
    for sample in samples {
        recorder.record("phase", sample).unwrap();
    }
    let summary = recorder.report().phases[0].clone();
    assert_eq!(summary.count, stats.count);
    assert_eq!(summary.total_ns, stats.total_ns);
    assert_eq!(summary.min_ns, stats.min_ns);
    assert_eq!(summary.max_ns, stats.max_ns);
    assert_eq!(summary.mean_ns, stats.mean_ns);
    assert_eq!(summary.median_ns, stats.median_ns);

    // 偶数样本取中间两值均值。
    let even = TimingStats::from_samples(&[100, 200, 300, 400]).unwrap();
    assert_eq!(even.median_ns, 250.0);
    assert_eq!(
        TimingStats::from_samples(&[]).unwrap_err(),
        ToolingError::ZeroIterations
    );
}

#[test]
fn report_json_round_trips_and_escapes_text() {
    let mut report = test_report();
    report.environment = EnvironmentFacts::capture("GPU \"quoted\"\nline", (8, 9), 12080);
    report.push_note("含引号\"与反斜杠\\与换行\n的说明");
    let json = report.to_json().unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["schema"].as_u64().unwrap(), u64::from(RECORD_SCHEMA));
    assert_eq!(value["task"].as_str().unwrap(), TASK_ID);
    assert_eq!(value["workload"].as_str().unwrap(), "probe/8");
    assert_eq!(
        value["environment"]["gpu"].as_str().unwrap(),
        "GPU \"quoted\"\nline"
    );
    assert_eq!(
        value["environment"]["compute_capability"][1].as_i64(),
        Some(9)
    );
    assert_eq!(value["plan"]["warmup"].as_u64(), Some(2));
    assert_eq!(value["records"][0]["timing"]["count"].as_u64(), Some(5));
    assert_eq!(
        value["records"][0]["timing"]["median_ns"].as_f64(),
        Some(250.0)
    );
    assert_eq!(
        value["records"][0]["timing"]["samples_ns"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    let checkpoints = value["checkpoints"].as_array().unwrap();
    assert_eq!(checkpoints[0]["label"].as_str().unwrap(), "idle");
    assert_eq!(checkpoints[0]["used_bytes"].as_u64(), Some(100));
    let notes = value["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 2);
    // 同输入两次导出字节一致（确定性）。
    assert_eq!(json, report.to_json().unwrap());
}

#[test]
fn report_validation_rejects_inconsistent_records() {
    assert!(test_report().to_json().is_ok());

    let mut empty = test_report();
    empty.records.clear();
    assert!(matches!(
        empty.to_json().unwrap_err(),
        ToolingError::InvalidRecord {
            field: "report.records",
            ..
        }
    ));

    let mut no_notes = test_report();
    no_notes.notes.clear();
    assert!(matches!(
        no_notes.to_json().unwrap_err(),
        ToolingError::InvalidRecord {
            field: "report.notes",
            ..
        }
    ));

    let mut no_checkpoints = test_report();
    no_checkpoints.checkpoints.clear();
    assert!(matches!(
        no_checkpoints.to_json().unwrap_err(),
        ToolingError::InvalidRecord {
            field: "report.checkpoints",
            ..
        }
    ));

    // 样本数与固定计划不一致。
    let mut short = PerformanceReport::new(
        TASK_ID,
        "probe/8",
        test_environment(),
        BenchmarkPlan::new(2, 6).unwrap(),
    );
    short.push_record(
        PerformanceRecord::new(
            "phase",
            vec![100, 200, 300, 400, 250],
            test_memory(10, 20, 15),
        )
        .unwrap(),
    );
    short.push_checkpoint(MemoryCheckpoint {
        label: "idle".into(),
        used_bytes: 100,
        total_bytes: 200,
    });
    short.push_note("边界");
    assert!(matches!(
        short.to_json().unwrap_err(),
        ToolingError::InvalidRecord {
            field: "timing.samples_ns",
            ..
        }
    ));

    // 峰值低于基线。
    let mut bad_memory = PerformanceReport::new(
        TASK_ID,
        "probe/8",
        test_environment(),
        BenchmarkPlan::new(2, 5).unwrap(),
    );
    bad_memory.push_record(
        PerformanceRecord::new("phase", vec![1, 2, 3, 4, 5], test_memory(30, 20, 15)).unwrap(),
    );
    bad_memory.push_checkpoint(MemoryCheckpoint {
        label: "idle".into(),
        used_bytes: 100,
        total_bytes: 200,
    });
    bad_memory.push_note("边界");
    assert!(matches!(
        bad_memory.to_json().unwrap_err(),
        ToolingError::InvalidRecord {
            field: "memory.peak_used_bytes",
            ..
        }
    ));

    // 统计与样本被拼接篡改。
    let mut tampered = test_report();
    tampered.records[0].timing.stats.total_ns += 1;
    assert!(matches!(
        tampered.to_json().unwrap_err(),
        ToolingError::InvalidRecord {
            field: "timing.stats",
            ..
        }
    ));

    // 档位不在 debug/release 之内。
    let mut bad_profile = test_report();
    bad_profile.environment.profile = "bench";
    assert!(matches!(
        bad_profile.to_json().unwrap_err(),
        ToolingError::InvalidRecord {
            field: "environment.profile",
            ..
        }
    ));
}

#[test]
fn build_profile_tracks_build_mode() {
    let profile = build_profile();
    if cfg!(debug_assertions) {
        assert_eq!(profile, "debug");
    } else {
        assert_eq!(profile, "release");
    }
}
