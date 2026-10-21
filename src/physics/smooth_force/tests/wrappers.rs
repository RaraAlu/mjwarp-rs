//! 平滑力包装故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::SmoothForceComputeMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::SmoothForceValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_compute_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_compute_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
fn validate_wrapper_rejects_capacity_context_alias_span_dimension_and_proof_faults() {
    let dims = SmoothForceValidateDims {
        abi: [2, 3, 2, 2],
        sleep_enabled: true,
    };
    let lengths = smooth_force_validate_lengths(&dims).unwrap();
    assert_eq!(lengths, [17, 32, 22, 10]);
    for abi in [[0, 3, 2, 2], [2, 0, 2, 2], [2, 3, 2, 0]] {
        assert!(matches!(
            smooth_force_validate_lengths(&SmoothForceValidateDims {
                abi,
                sleep_enabled: true,
            }),
            Err(LauncherError::InvalidArray("平滑力验证维度无效"))
        ));
    }
    let max_grid_worlds = u32::MAX - 255;
    assert!(
        smooth_force_validate_lengths(&SmoothForceValidateDims {
            abi: [1, 1, 0, max_grid_worlds],
            sleep_enabled: false,
        })
        .is_ok()
    );
    assert!(matches!(
        smooth_force_validate_lengths(&SmoothForceValidateDims {
            abi: [1, 1, 0, max_grid_worlds + 1],
            sleep_enabled: false,
        }),
        Err(LauncherError::InvalidArray("平滑力验证维度无效"))
    ));
    let metadata_limit = SmoothForceValidateDims {
        abi: [1, 1, i32::MAX as u32 - 10, 1],
        sleep_enabled: true,
    };
    assert_eq!(
        smooth_force_validate_lengths(&metadata_limit).unwrap()[0],
        i32::MAX as u64
    );
    let metadata_over = SmoothForceValidateDims {
        abi: [1, 1, i32::MAX as u32 - 9, 1],
        sleep_enabled: true,
    };
    assert!(matches!(
        smooth_force_validate_lengths(&metadata_over),
        Err(LauncherError::Overflow("smooth_force_validate_metadata"))
    ));
    let forces_over = SmoothForceValidateDims {
        abi: [1, i32::MAX as u32 - 9, 0, max_grid_worlds],
        sleep_enabled: false,
    };
    assert!(matches!(
        smooth_force_validate_lengths(&forces_over),
        Err(LauncherError::Overflow("smooth_force_validate_forces"))
    ));
    // status由u32加8构成，静态上界不会触及u64溢出。
    assert!(u64::from(u32::MAX).checked_add(8).is_some());
    // 合法meta约束worlds*nt不超过i32上界。
    // 因此外部sleep的静态上界也远低于u64上界。
    let sleep_static_upper = 2u64 * i32::MAX as u64 + 3u64 * u64::from(max_grid_worlds) + 8;
    assert!(sleep_static_upper < u64::MAX);
    let spans = [
        DeviceSpan::new(7, 0x1000, lengths[0] * 4).unwrap(),
        DeviceSpan::new(7, 0x2000, lengths[1] * 4).unwrap(),
        DeviceSpan::new(7, 0x3000, lengths[2] * 4).unwrap(),
        DeviceSpan::new(7, 0x4000, lengths[3] * 4).unwrap(),
    ];
    let bindings = [
        SlotBinding {
            slot: 0,
            span: spans[0],
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: spans[1],
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: spans[2],
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 3,
            span: spans[3],
            elem: ElemType::F32,
        },
    ];
    let values = vec![
        -131072,
        -131072,
        -131072,
        -131072,
        -1,
        1,
        0,
        1,
        1,
        0,
        -11,
        2,
        i32::MAX,
        -131072,
        -131072,
        -131072,
        -131072,
    ];
    let revision = MetadataRevision::new();
    // SAFETY: 合成跨度与宿主快照长度一致；本测试不访问设备。
    let checked = unsafe {
        Checked::<SmoothForceValidateMetadata>::assume_uploaded(
            spans[0],
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    assert!(smooth_force_validate_plan(&dims, &bindings, &[checked.binding(0)]).is_ok());
    // SAFETY: 短快照与短跨度一致，只验证宿主内容。
    assert!(matches!(
        unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                DeviceSpan::new(7, 0x5000, (lengths[0] - 1) * 4).unwrap(),
                &values[..values.len() - 1],
                dims,
                MetadataRevision::new(),
            )
        },
        Err(LauncherError::InvalidMetadata(_))
    ));

    let mut wrong_metadata_span = bindings;
    wrong_metadata_span[0].span = DeviceSpan::new(7, spans[0].addr(), lengths[0] * 4 + 4).unwrap();
    assert!(matches!(
        smooth_force_validate_plan(&dims, &wrong_metadata_span, &[checked.binding(0)]),
        Err(LauncherError::MetadataMismatch(_))
    ));
    for (slot, bytes) in [
        (1, lengths[1] * 4 - 4),
        (3, lengths[3] * 4 - 4),
        (3, lengths[3] * 4 + 4),
    ] {
        let mut bad = bindings;
        bad[slot].span = DeviceSpan::new(7, bad[slot].span.addr(), bytes).unwrap();
        assert!(matches!(
            smooth_force_validate_plan(&dims, &bad, &[checked.binding(0)]),
            Err(LauncherError::Capacity { slot: actual, .. }) if actual == slot
        ));
    }
    let mut bad_sleep = bindings;
    for bytes in [lengths[2] * 4 - 4, lengths[2] * 4 + 4] {
        bad_sleep[2].span = DeviceSpan::new(7, 0x3000, bytes).unwrap();
        assert!(matches!(
            smooth_force_validate_plan(&dims, &bad_sleep, &[checked.binding(0)]),
            Err(LauncherError::Capacity { slot: 2, .. })
        ));
    }
    let mut wrong_context = bindings;
    wrong_context[2].span = DeviceSpan::new(8, 0x3000, lengths[2] * 4).unwrap();
    assert!(matches!(
        smooth_force_validate_plan(&dims, &wrong_context, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut partial_alias = bindings;
    partial_alias[2].span = DeviceSpan::new(7, spans[0].addr() + 4, lengths[2] * 4).unwrap();
    assert!(matches!(
        smooth_force_validate_plan(&dims, &partial_alias, &[checked.binding(0)]),
        Err(LauncherError::Alias {
            first: 0,
            second: 2
        })
    ));
    let mut read_alias = bindings;
    read_alias[2].span = DeviceSpan::new(7, spans[1].addr(), lengths[2] * 4).unwrap();
    assert!(matches!(
        smooth_force_validate_plan(&dims, &read_alias, &[checked.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 2
        })
    ));
    let mut status_alias = bindings;
    status_alias[3].span = spans[1];
    assert!(matches!(
        smooth_force_validate_plan(&dims, &status_alias, &[checked.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));
    let mut exact_alias = bindings;
    exact_alias[2].span = spans[0];
    assert!(smooth_force_validate_plan(&dims, &exact_alias, &[checked.binding(0)]).is_ok());

    let disabled = SmoothForceValidateDims {
        sleep_enabled: false,
        ..dims
    };
    // SAFETY: 合成跨度与同一有效宿主快照绑定。
    let disabled_checked = unsafe {
        Checked::<SmoothForceValidateMetadata>::assume_uploaded(
            spans[0],
            &values,
            disabled,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    let mut ignored_sleep = bindings;
    ignored_sleep[2].span = DeviceSpan::new(7, 0x3000, 4).unwrap();
    assert!(
        smooth_force_validate_plan(&disabled, &ignored_sleep, &[disabled_checked.binding(0)])
            .is_ok()
    );
    assert!(matches!(
        DeviceSpan::new(7, 0x3000, 0),
        Err(LauncherError::InvalidSpan("区间长度为零"))
    ));

    revision.invalidate();
    assert!(matches!(
        smooth_force_validate_plan(&dims, &bindings, &[checked.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    let mut invalid_values = values.clone();
    invalid_values[4] = -2;
    // SAFETY: 合成跨度只用于宿主校验，不触发设备访问。
    assert!(matches!(
        unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                spans[0],
                &invalid_values,
                dims,
                MetadataRevision::new(),
            )
        },
        Err(LauncherError::InvalidMetadata(_))
    ));
}
#[test]
#[cfg(feature = "cuda-probe")]
fn compute_wrapper_rejects_capacity_context_alias_span_dimension_and_proof_faults() {
    let dims = SmoothForceComputeDims {
        abi: [2, 3, 2, 6],
        sleep_enabled: true,
    };
    let lengths = smooth_force_compute_lengths(&dims).unwrap();
    assert_eq!(lengths, [17, 32, 22, 14]);
    for abi in [[0, 3, 2, 6], [2, 0, 2, 6], [2, 3, 2, 0]] {
        assert!(matches!(
            smooth_force_compute_lengths(&SmoothForceComputeDims {
                abi,
                sleep_enabled: true,
            }),
            Err(LauncherError::InvalidArray("平滑力计算维度无效"))
        ));
    }
    assert!(matches!(
        smooth_force_compute_lengths(&SmoothForceComputeDims {
            abi: [2, 3, 2, 5],
            sleep_enabled: true,
        }),
        Err(LauncherError::InvalidArray("平滑力计算工作量不匹配"))
    ));
    let max_grid_work = u32::MAX - 255;
    assert!(
        smooth_force_compute_lengths(&SmoothForceComputeDims {
            abi: [1, 1, 0, max_grid_work],
            sleep_enabled: false,
        })
        .is_ok()
    );
    assert!(matches!(
        smooth_force_compute_lengths(&SmoothForceComputeDims {
            abi: [1, 1, 0, max_grid_work + 1],
            sleep_enabled: false,
        }),
        Err(LauncherError::InvalidArray("平滑力计算维度无效"))
    ));
    let metadata_limit = SmoothForceComputeDims {
        abi: [1, 1, i32::MAX as u32 - 10, 1],
        sleep_enabled: true,
    };
    assert_eq!(
        smooth_force_compute_lengths(&metadata_limit).unwrap()[0],
        i32::MAX as u64
    );
    let metadata_over = SmoothForceComputeDims {
        abi: [1, 1, i32::MAX as u32 - 9, 1],
        sleep_enabled: true,
    };
    assert!(matches!(
        smooth_force_compute_lengths(&metadata_over),
        Err(LauncherError::Overflow("smooth_force_compute_metadata"))
    ));
    // work受u32网格上界约束，力与输出不会触及u64上界。
    assert!(u64::from(max_grid_work).checked_mul(4).is_some());
    assert!(u64::from(max_grid_work).checked_add(8).is_some());
    // 合法meta约束worlds*nt不超过i32上界。
    // 因此外部sleep的静态上界也远低于u64上界。
    let sleep_static_upper = 2u64 * i32::MAX as u64 + 3u64 * u64::from(max_grid_work) + 8;
    assert!(sleep_static_upper < u64::MAX);

    let spans = [
        DeviceSpan::new(7, 0x1000, lengths[0] * 4).unwrap(),
        DeviceSpan::new(7, 0x2000, lengths[1] * 4).unwrap(),
        DeviceSpan::new(7, 0x3000, lengths[2] * 4).unwrap(),
        DeviceSpan::new(7, 0x4000, lengths[3] * 4).unwrap(),
    ];
    let bindings = [
        SlotBinding {
            slot: 0,
            span: spans[0],
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: spans[1],
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: spans[2],
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 3,
            span: spans[3],
            elem: ElemType::F32,
        },
    ];
    let values = vec![
        -131072,
        -131072,
        -131072,
        -131072,
        -1,
        1,
        0,
        1,
        1,
        0,
        -11,
        2,
        i32::MAX,
        -131072,
        -131072,
        -131072,
        -131072,
    ];
    let revision = MetadataRevision::new();
    // SAFETY: 合成跨度与宿主快照长度一致；本测试不访问设备。
    let checked = unsafe {
        Checked::<SmoothForceComputeMetadata>::assume_uploaded(
            spans[0],
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    assert!(smooth_force_compute_plan(&dims, &bindings, &[checked.binding(0)]).is_ok());

    let mut wrong_metadata_span = bindings;
    wrong_metadata_span[0].span = DeviceSpan::new(7, spans[0].addr(), lengths[0] * 4 + 4).unwrap();
    assert!(matches!(
        smooth_force_compute_plan(&dims, &wrong_metadata_span, &[checked.binding(0)]),
        Err(LauncherError::MetadataMismatch(_))
    ));
    for (slot, bytes) in [
        (1, lengths[1] * 4 - 4),
        (3, lengths[3] * 4 - 4),
        (3, lengths[3] * 4 + 4),
    ] {
        let mut bad = bindings;
        bad[slot].span = DeviceSpan::new(7, bad[slot].span.addr(), bytes).unwrap();
        assert!(matches!(
            smooth_force_compute_plan(&dims, &bad, &[checked.binding(0)]),
            Err(LauncherError::Capacity { slot: actual, .. }) if actual == slot
        ));
    }
    for bytes in [lengths[2] * 4 - 4, lengths[2] * 4 + 4] {
        let mut bad = bindings;
        bad[2].span = DeviceSpan::new(7, 0x3000, bytes).unwrap();
        assert!(matches!(
            smooth_force_compute_plan(&dims, &bad, &[checked.binding(0)]),
            Err(LauncherError::Capacity { slot: 2, .. })
        ));
    }
    let mut wrong_context = bindings;
    wrong_context[2].span = DeviceSpan::new(8, 0x3000, lengths[2] * 4).unwrap();
    assert!(matches!(
        smooth_force_compute_plan(&dims, &wrong_context, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    for (slot, address) in [(2, spans[0].addr() + 4), (2, spans[1].addr())] {
        let mut bad = bindings;
        bad[slot].span = DeviceSpan::new(7, address, lengths[2] * 4).unwrap();
        assert!(matches!(
            smooth_force_compute_plan(&dims, &bad, &[checked.binding(0)]),
            Err(LauncherError::Alias { .. })
        ));
    }
    let mut output_alias = bindings;
    output_alias[3].span = spans[1];
    assert!(matches!(
        smooth_force_compute_plan(&dims, &output_alias, &[checked.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut exact_alias = bindings;
    exact_alias[2].span = spans[0];
    assert!(smooth_force_compute_plan(&dims, &exact_alias, &[checked.binding(0)]).is_ok());

    let disabled = SmoothForceComputeDims {
        sleep_enabled: false,
        ..dims
    };
    // SAFETY: 合成跨度与同一有效宿主快照绑定。
    let disabled_checked = unsafe {
        Checked::<SmoothForceComputeMetadata>::assume_uploaded(
            spans[0],
            &values,
            disabled,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    let mut ignored_sleep = bindings;
    ignored_sleep[2].span = DeviceSpan::new(7, 0x3000, 4).unwrap();
    assert!(
        smooth_force_compute_plan(&disabled, &ignored_sleep, &[disabled_checked.binding(0)])
            .is_ok()
    );
    assert!(matches!(
        DeviceSpan::new(7, 0x3000, 0),
        Err(LauncherError::InvalidSpan("区间长度为零"))
    ));

    let smaller_worlds = SmoothForceComputeDims {
        abi: [2, 3, 2, 3],
        ..dims
    };
    assert!(matches!(
        smooth_force_compute_plan(&smaller_worlds, &bindings, &[checked.binding(0)]),
        Err(LauncherError::MetadataMismatch(_))
    ));
    revision.invalidate();
    assert!(matches!(
        smooth_force_compute_plan(&dims, &bindings, &[checked.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
}
