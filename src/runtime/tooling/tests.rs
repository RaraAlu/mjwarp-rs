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
    let nanos = measure(3, || counter += 1).unwrap();
    assert_eq!(counter, 3);
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
