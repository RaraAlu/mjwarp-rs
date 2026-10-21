//! 平滑力校验GPU测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceValidateDims;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::SmoothForceMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "M3仅验证类型包装：关闭休眠、真实拒绝和同计划恢复"]
fn gpu_m3_validate_wrapper_sleep_disabled_preserves_output_and_recovers() {
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

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "M3仅验证类型包装：休眠来源、非零awake、ntree零和恢复"]
fn gpu_m3_validate_wrapper_sleep_enabled_covers_sources_and_ntree_zero() {
    let session = TransferSession::new(0).unwrap();
    let input = SmoothForceInput::new(2, vec![-1, 0, 1], vec![1, 2, 0]).unwrap();
    let plan = SmoothForcePlan::new(
        &session,
        input,
        SmoothForceOptions {
            sleep_enabled: true,
            island_disabled: false,
        },
    )
    .unwrap();
    let mut data = plan.create_data(2).unwrap();
    for (field, value) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
        data.write_force(*field, &[value; 6]).unwrap();
    }
    let stale = data.metadata.proof_snapshot_for_test().unwrap();
    data.write_awake(&[-11, 0, 2, i32::MAX]).unwrap();
    let metadata_dims = data.metadata.dims;
    let metadata_span = data.metadata.as_buffer().launcher_span().unwrap();
    assert!(matches!(
        stale.binding(0).validate_for(metadata_span, &metadata_dims),
        Err(LauncherError::StaleMetadata)
    ));
    assert!(data.metadata.proof(metadata_dims).is_ok());
    plan.update(&mut data).unwrap();
    let internal = data.readback().unwrap();
    assert_eq!(internal.world(0).unwrap()[0].to_bits(), 10.0f32.to_bits());
    assert_eq!(internal.world(0).unwrap()[1].to_bits(), 0);
    assert_eq!(internal.world(0).unwrap()[2].to_bits(), 10.0f32.to_bits());
    assert!(internal.world(1).unwrap().iter().all(|&v| v == 10.0));
    let internal_bits = bits(&internal.values);

    let stride = 2 * 2 + 3;
    let mut external = vec![-131072; 2 * stride + 8];
    for world in 0..2 {
        let awake = 4 + world * stride + 2;
        external[awake..awake + 2].copy_from_slice(if world == 0 {
            &[0, -11]
        } else {
            &[2, i32::MAX]
        });
    }
    let mut external = session.upload(&external).unwrap();
    plan.update_with_sleep_source(&mut data, Some(&external))
        .unwrap();
    let external_output = data.readback().unwrap();
    assert_eq!(
        bits(external_output.world(0).unwrap()),
        [0.0f32, 10.0, 10.0].map(f32::to_bits)
    );
    assert_eq!(
        bits(external_output.world(1).unwrap()),
        [10.0f32; 3].map(f32::to_bits)
    );
    let external_good = bits(&external_output.values);
    let output_before = bits(&raw_f32(&data.output));
    external.write_range(0, &[0]).unwrap();
    let error = plan
        .update_with_sleep_source(&mut data, Some(&external))
        .unwrap_err();
    assert_eq!(
        error,
        TransferError::Input(InputError::InvalidTopology {
            field: "smooth_force_source",
            index: 0,
            reason: "nonfinite_guard_or_mapping",
        })
    );
    assert_eq!(raw_f32(&data.status)[4..6], [0.0, 0.0]);
    assert!(data.readback().is_err());
    assert_eq!(bits(&raw_f32(&data.output)), output_before);
    external.write_range(0, &[-131072]).unwrap();
    plan.update_with_sleep_source(&mut data, Some(&external))
        .unwrap();
    assert_eq!(bits(&data.readback().unwrap().values), external_good);

    let output_before_metadata_fault = bits(&raw_f32(&data.output));
    data.metadata
        .corrupt_device_for_validator_test(0, 0)
        .unwrap();
    let error = plan.update(&mut data).unwrap_err();
    assert_eq!(
        error,
        TransferError::Input(InputError::InvalidTopology {
            field: "smooth_force_source",
            index: 0,
            reason: "nonfinite_guard_or_mapping",
        })
    );
    assert_eq!(raw_f32(&data.status)[4..6], [0.0, 0.0]);
    assert!(data.readback().is_err());
    assert_eq!(bits(&raw_f32(&data.output)), output_before_metadata_fault);
    data.metadata.restore_for_test().unwrap();
    plan.update(&mut data).unwrap();
    assert_eq!(bits(&data.readback().unwrap().values), internal_bits);

    let dims = SmoothForceValidateDims {
        abi: [2, 2, 0, 2],
        sleep_enabled: true,
    };
    let metadata = vec![
        -131072, -131072, -131072, -131072, -1, -1, 0, 1, -131072, -131072, -131072, -131072,
    ];
    let metadata = SmoothForceMetadataBuffer::upload(&session, metadata, dims).unwrap();
    let mut forces = vec![-131072.0; 4 * 2 * 2 + 8];
    forces[4..4 + 16].fill(1.0);
    let forces = session.upload(&forces).unwrap();
    let mut status_values = vec![-131072.0; 2 + 8];
    status_values[4..6].fill(f32::NAN);
    let mut status = session.upload(&status_values).unwrap();
    plan.validate
        .launch(
            &metadata,
            &forces,
            metadata.as_buffer(),
            &mut status,
            dims.abi,
        )
        .unwrap();
    let status = raw_f32(&status);
    assert_eq!(status[4..6], [1.0, 1.0]);

    let report = serde_json::json!({
        "variant": "sleep-on",
        "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
        "plans_compiled": 1,
        "internal_awake": true,
        "external_sleep": true,
        "nonzero_awake": [-11, 2, i32::MAX],
        "ntree_zero_reused_validation_kernel": true,
        "validation_rejection_entered_gpu": true,
        "output_bits_unchanged": true,
        "same_plan_recovery": true,
        "task_gpu_memory": "unknown"
    });
    if let Some(path) = std::env::var_os("MJWARP_SMOOTH_VALIDATE_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
