//! 平滑力计算GPU测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceComputeDims;
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
#[ignore = "M3仅验证计算包装：发射前拒绝、计算后非有限和同计划恢复"]
fn gpu_m3_compute_wrapper_sleep_disabled_preserves_failure_boundaries() {
    let session = TransferSession::new(0).unwrap();
    let input = SmoothForceInput::new(2, vec![-1, 0, 1], vec![1, 2, 0]).unwrap();
    let plan = SmoothForcePlan::new(&session, input, SmoothForceOptions::default()).unwrap();
    let mut data = plan.create_data(2).unwrap();
    for (field, value) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
        data.write_force(*field, &[value; 6]).unwrap();
    }
    let ignored = session.upload(&[i32::MIN]).unwrap();
    plan.update_with_sleep_source(&mut data, Some(&ignored))
        .unwrap();
    let good_output = data.readback().unwrap();
    for world in 0..2 {
        assert!(
            good_output
                .world(world)
                .unwrap()
                .iter()
                .all(|value| value.to_bits() == 10.0f32.to_bits())
        );
    }
    let good = bits(&good_output.values);

    let before_prelaunch = bits(&raw_f32(&data.output));
    let error = plan
        .compute
        .launch(
            &data.metadata,
            &data.forces,
            &ignored,
            &mut data.output,
            [2, 3, 2, 5],
        )
        .unwrap_err();
    assert_eq!(
        error,
        TransferError::Input(InputError::InvalidDimension {
            field: "smooth_force_compute_abi",
        })
    );
    assert_eq!(bits(&raw_f32(&data.output)), before_prelaunch);

    data.write_world_force(SmoothForceField::Passive, 0, &[f32::MAX, 5.0, 5.0])
        .unwrap();
    data.write_world_force(SmoothForceField::Bias, 0, &[-f32::MAX, 2.0, 2.0])
        .unwrap();
    let before_compute_failure = bits(&raw_f32(&data.output));
    let error = plan
        .update_with_sleep_source(&mut data, Some(&ignored))
        .unwrap_err();
    assert!(matches!(
        error,
        TransferError::Input(InputError::NonFinite {
            field: "qfrc_smooth",
            ..
        })
    ));
    let after_compute_failure = bits(&raw_f32(&data.output));
    assert_ne!(after_compute_failure, before_compute_failure);
    assert_eq!(after_compute_failure[4], f32::INFINITY.to_bits());
    assert!(!data.ready);
    assert!(data.readback().is_err());

    data.write_world_force(SmoothForceField::Passive, 0, &[5.0; 3])
        .unwrap();
    data.write_world_force(SmoothForceField::Bias, 0, &[2.0; 3])
        .unwrap();
    plan.update_with_sleep_source(&mut data, Some(&ignored))
        .unwrap();
    assert_eq!(bits(&data.readback().unwrap().values), good);

    let report = serde_json::json!({
        "variant": "sleep-off",
        "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
        "plans_compiled": 1,
        "compiled_entries": ["smooth_force_validate", "smooth_force"],
        "ignored_sleep_elements": ignored.len(),
        "prelaunch_rejection_output_unchanged": true,
        "post_compute_nonfinite_wrote_output": true,
        "failed_result_unreadable": true,
        "same_plan_recovery": true,
        "task_gpu_memory": "unknown"
    });
    if let Some(path) = std::env::var_os("MJWARP_SMOOTH_COMPUTE_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "M3仅验证计算包装：内部awake、外部sleep、非零awake和ntree零"]
fn gpu_m3_compute_wrapper_sleep_enabled_covers_sources_and_ntree_zero() {
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
    let compute_dims = SmoothForceComputeDims {
        abi: [3, 3, 2, 6],
        sleep_enabled: true,
    };
    let stale = data.metadata.compute_proof(compute_dims).unwrap();
    data.write_awake(&[-11, 0, 2, i32::MAX]).unwrap();
    let metadata_span = data.metadata.as_buffer().launcher_span().unwrap();
    assert!(matches!(
        stale.binding(0).validate_for(metadata_span, &compute_dims),
        Err(LauncherError::StaleMetadata)
    ));
    assert!(data.metadata.compute_proof(compute_dims).is_ok());
    plan.update(&mut data).unwrap();
    let internal = data.readback().unwrap();
    assert_eq!(
        bits(internal.world(0).unwrap()),
        [10.0f32, 0.0, 10.0].map(f32::to_bits)
    );
    assert_eq!(
        bits(internal.world(1).unwrap()),
        [10.0f32; 3].map(f32::to_bits)
    );

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
    let external = session.upload(&external).unwrap();
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

    let before_validation_fault = bits(&raw_f32(&data.output));
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
    assert_eq!(bits(&raw_f32(&data.output)), before_validation_fault);
    assert!(!data.ready);
    assert!(data.readback().is_err());
    data.metadata.restore_for_test().unwrap();
    plan.update(&mut data).unwrap();
    let recovered = data.readback().unwrap();
    assert_eq!(
        bits(recovered.world(0).unwrap()),
        [10.0f32, 0.0, 10.0].map(f32::to_bits)
    );
    assert_eq!(
        bits(recovered.world(1).unwrap()),
        [10.0f32; 3].map(f32::to_bits)
    );

    let validate_dims = SmoothForceValidateDims {
        abi: [2, 2, 0, 2],
        sleep_enabled: true,
    };
    let metadata = vec![
        -131072, -131072, -131072, -131072, -1, -1, 0, 1, -131072, -131072, -131072, -131072,
    ];
    let metadata = SmoothForceMetadataBuffer::upload(&session, metadata, validate_dims).unwrap();
    let mut forces = vec![-131072.0; 4 * 4 + 8];
    for (field, value) in [5.0, 2.0, 3.0, 4.0].into_iter().enumerate() {
        forces[4 + field * 4..4 + (field + 1) * 4].fill(value);
    }
    let forces = session.upload(&forces).unwrap();
    let mut output = vec![-131072.0; 4 + 8];
    output[4..8].fill(f32::NAN);
    let mut output = session.upload(&output).unwrap();
    plan.compute
        .launch(
            &metadata,
            &forces,
            metadata.as_buffer(),
            &mut output,
            [2, 2, 0, 4],
        )
        .unwrap();
    assert_eq!(
        bits(&raw_f32(&output)[4..8]),
        [10.0f32; 4].map(f32::to_bits)
    );

    let report = serde_json::json!({
        "variant": "sleep-on",
        "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
        "plans_compiled": 1,
        "compiled_entries": ["smooth_force_validate", "smooth_force"],
        "internal_awake": true,
        "external_sleep": true,
        "nonzero_awake": [-11, 2, i32::MAX],
        "ntree_zero_reused_compute_kernel": true,
        "validation_precedes_compute": true,
        "same_plan_recovery": true,
        "task_gpu_memory": "unknown"
    });
    if let Some(path) = std::env::var_os("MJWARP_SMOOTH_COMPUTE_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
