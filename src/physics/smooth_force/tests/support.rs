//! 平滑力测试共用参考与工具。

#[cfg(feature = "cuda-probe")]
use super::*;

pub(super) fn reference() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/smooth-force/reference.json"
    )))
    .unwrap()
}
pub(super) fn row(v: &serde_json::Value, index: usize) -> ([f32; 4], u32) {
    let r = &v["rows"][index];
    let mut x = [0.0; 4];
    for (j, value) in x.iter_mut().enumerate() {
        *value =
            f32::from_bits(u32::from_str_radix(r["input_bits"][j].as_str().unwrap(), 16).unwrap());
    }
    let bits = u32::from_str_radix(r["output_bits"].as_str().unwrap(), 16).unwrap();
    (x, bits)
}

#[cfg(feature = "cuda-probe")]
pub(super) const FIELDS: [SmoothForceField; 4] = [
    SmoothForceField::Passive,
    SmoothForceField::Bias,
    SmoothForceField::Actuator,
    SmoothForceField::Applied,
];
#[cfg(feature = "cuda-probe")]
pub(super) fn model(n: usize) -> SmoothForceInput {
    SmoothForceInput::new(
        129,
        (0..273)
            .map(|b| if b % 11 == 0 { -1 } else { b * 7 % 129 })
            .collect(),
        (0..n).map(|i| ((i * 17 + 1) % 273) as i32).collect(),
    )
    .unwrap()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn raw_f32(buffer: &TransferBuffer<f32>) -> Vec<f32> {
    let mut v = vec![0.0; buffer.len()];
    buffer.read_range_into(0, &mut v).unwrap();
    v
}
#[cfg(feature = "cuda-probe")]
pub(super) fn raw_i32(buffer: &TransferBuffer<i32>) -> Vec<i32> {
    let mut v = vec![0; buffer.len()];
    buffer.read_range_into(0, &mut v).unwrap();
    v
}
#[cfg(feature = "cuda-probe")]
pub(super) fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|v| v.to_bits()).collect()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn tokens(d: &SmoothForceData) -> (Vec<u32>, Vec<i32>, Vec<u32>) {
    (
        bits(&raw_f32(&d.forces)),
        raw_i32(d.metadata.as_buffer()),
        bits(&raw_f32(&d.output)),
    )
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "M3仅验证类型包装：关闭休眠、真实拒绝和同计划恢复"]
pub(super) fn gpu_m3_validate_wrapper_sleep_disabled_preserves_output_and_recovers() {
    let session = TransferSession::new(0).unwrap();
    let input = SmoothForceInput::new(2, vec![-1, 0, 1], vec![1, 2, 0]).unwrap();
    let plan = SmoothForcePlan::new(&session, input, SmoothForceOptions::default()).unwrap();
    let mut data = plan.create_data(2).unwrap();
    for (field, value) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
        data.write_force(*field, &[value; 6]).unwrap();
    }
    data.write_awake(&[0, 0, 0, 0]).unwrap();
    let ignored = session.upload(&[i32::MIN]).unwrap();
    plan.update_with_sleep_source(&mut data, Some(&ignored))
        .unwrap();
    let output = data.readback().unwrap();
    for world in 0..2 {
        assert!(
            output
                .world(world)
                .unwrap()
                .iter()
                .all(|value| value.to_bits() == 10.0f32.to_bits())
        );
    }
    let good = bits(&output.values);
    let output_before = bits(&raw_f32(&data.output));

    data.forces.write_range(4, &[f32::NAN]).unwrap();
    let error = plan
        .update_with_sleep_source(&mut data, Some(&ignored))
        .unwrap_err();
    assert_eq!(
        error,
        TransferError::Input(InputError::InvalidTopology {
            field: "smooth_force_source",
            index: 0,
            reason: "nonfinite_guard_or_mapping",
        })
    );
    assert_eq!(raw_f32(&data.status)[4..6], [0.0, 1.0]);
    assert!(data.readback().is_err());
    assert_eq!(bits(&raw_f32(&data.output)), output_before);
    data.forces.write_range(4, &[5.0]).unwrap();
    plan.update_with_sleep_source(&mut data, Some(&ignored))
        .unwrap();
    assert_eq!(bits(&data.readback().unwrap().values), good);

    let report = serde_json::json!({
        "variant": "sleep-off",
        "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
        "plans_compiled": 1,
        "validation_rejection_entered_gpu": true,
        "output_bits_unchanged": true,
        "same_plan_recovery": true,
        "ignored_sleep_elements": ignored.len(),
        "task_gpu_memory": "unknown"
    });
    if let Some(path) = std::env::var_os("MJWARP_SMOOTH_VALIDATE_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
