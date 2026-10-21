//! 柔体Hessian与休眠复制测试。

use super::output::decode_flex_validity;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::SleepCopyDims;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::SleepCopyMetadata;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::SleepCopySpec;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::sleep_copy_buffer_bytes;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::sleep_copy_plan;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::sleep_copy_state_elements;
use super::*;

#[test]
fn flex_hessian_decodes_only_canonical_boolean_slots() {
    assert_eq!(decode_flex_validity(&[]).unwrap(), Vec::<bool>::new());
    assert_eq!(
        decode_flex_validity(&[0, 1, 0, 1]).unwrap(),
        [false, true, false, true]
    );
    for value in [-131072, -1, 2, i32::MIN, i32::MAX] {
        assert!(matches!(
            decode_flex_validity(&[0, value, 1]),
            Err(TransferError::Input(InputError::InvalidTopology {
                field: "resident_flex_hessian_valid",
                index: 1,
                ..
            }))
        ));
    }
}

#[cfg(feature = "cuda-probe")]
#[test]
fn sleep_copy_wrapper_checks_span_session_alias_grid_and_metadata_proof() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = SleepCopyDims {
        abi: [0, 0, 0, 3],
        ntree: 2,
        metadata_elements: 7,
        parameter_elements: 3,
    };
    assert_eq!(sleep_copy_state_elements(&dims).unwrap(), 29);
    assert_eq!(SleepCopySpec::launch_dims(&dims).unwrap().grid(), (1, 1, 1));
    let many_worlds = SleepCopyDims {
        abi: [0, 0, 0, 513],
        ..dims
    };
    assert_eq!(sleep_copy_state_elements(&many_worlds).unwrap(), 3599);
    assert_eq!(
        SleepCopySpec::launch_dims(&many_worlds).unwrap().grid(),
        (3, 1, 1)
    );
    let zero_tree = SleepCopyDims {
        abi: [0, 0, 0, 5],
        ntree: 0,
        ..dims
    };
    assert_eq!(sleep_copy_state_elements(&zero_tree).unwrap(), 23);

    let metadata = DeviceSpan::new(7, 0x1000, 28).unwrap();
    let parameters = DeviceSpan::new(7, 0x2000, 12).unwrap();
    let state = DeviceSpan::new(7, 0x3000, 116).unwrap();
    let output = DeviceSpan::new(7, 0x4000, 116).unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: metadata,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: parameters,
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: state,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 3,
            span: output,
            elem: ElemType::I32,
        },
    ];
    let values = vec![2, 0, 0, 0, 0, 0, 0];
    let revision = MetadataRevision::new();
    // SAFETY: 合成跨度仅参与宿主校验，不访问设备。
    let checked = unsafe {
        Checked::<SleepCopyMetadata>::assume_uploaded(metadata, &values, dims, revision.clone())
    }
    .unwrap();
    assert!(sleep_copy_plan(&dims, &bindings, checked.binding(0)).is_ok());

    let mut read_alias = bindings;
    read_alias[1].span = DeviceSpan::new(7, metadata.addr(), 12).unwrap();
    read_alias[2].span = DeviceSpan::new(7, metadata.addr(), 116).unwrap();
    assert!(sleep_copy_plan(&dims, &read_alias, checked.binding(0)).is_ok());
    let mut output_alias = bindings;
    output_alias[3].span = state;
    assert!(matches!(
        sleep_copy_plan(&dims, &output_alias, checked.binding(0)),
        Err(LauncherError::Alias { .. })
    ));
    let mut partial_output_alias = bindings;
    partial_output_alias[3].span = DeviceSpan::new(7, state.addr() + 4, state.bytes()).unwrap();
    assert!(matches!(
        sleep_copy_plan(&dims, &partial_output_alias, checked.binding(0)),
        Err(LauncherError::Alias {
            first: 2,
            second: 3
        })
    ));

    for (slot, bytes) in [(0, 24), (1, 8), (2, 112), (2, 120), (3, 120)] {
        let mut bad = bindings;
        bad[slot].span = DeviceSpan::new(7, bad[slot].span.addr(), bytes).unwrap();
        let error = sleep_copy_plan(&dims, &bad, checked.binding(0)).unwrap_err();
        assert!(matches!(
            error,
            LauncherError::Capacity { slot: actual, .. } if actual == slot
        ));
    }
    let mut foreign = bindings;
    foreign[1].span = DeviceSpan::new(8, parameters.addr(), parameters.bytes()).unwrap();
    assert!(matches!(
        sleep_copy_plan(&dims, &foreign, checked.binding(0)),
        Err(LauncherError::ContextMismatch { slot: 1 })
    ));
    let wrong_span = DeviceSpan::new(7, 0x5000, metadata.bytes()).unwrap();
    // SAFETY: 合成跨度仅参与宿主校验，不访问设备。
    let wrong_checked = unsafe {
        Checked::<SleepCopyMetadata>::assume_uploaded(
            wrong_span,
            &values,
            dims,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(matches!(
        sleep_copy_plan(&dims, &bindings, wrong_checked.binding(0)),
        Err(LauncherError::MetadataMismatch(_))
    ));
    let mut forged = values.clone();
    forged[0] = 3;
    // SAFETY: 合成跨度仅参与宿主校验，不访问设备。
    assert!(matches!(
        unsafe {
            Checked::<SleepCopyMetadata>::assume_uploaded(
                metadata,
                &forged,
                dims,
                MetadataRevision::new(),
            )
        },
        Err(LauncherError::InvalidMetadata(_))
    ));
    revision.invalidate();
    assert!(matches!(
        sleep_copy_plan(&dims, &bindings, checked.binding(0)),
        Err(LauncherError::StaleMetadata)
    ));

    for bad in [
        SleepCopyDims {
            abi: [1, 0, 0, 3],
            ..dims
        },
        SleepCopyDims {
            abi: [0, 0, 0, 0],
            ..dims
        },
        SleepCopyDims {
            ntree: i32::MAX as u32 + 1,
            ..dims
        },
    ] {
        assert!(matches!(
            sleep_copy_state_elements(&bad),
            Err(LauncherError::InvalidArray(_))
        ));
    }
    let metadata_byte_overflow = SleepCopyDims {
        metadata_elements: u64::MAX,
        ..dims
    };
    assert!(matches!(
        sleep_copy_buffer_bytes(&metadata_byte_overflow, 0),
        Err(LauncherError::Overflow("sleep_copy_metadata_bytes"))
    ));
    let launch_limit = SleepCopyDims {
        abi: [0, 0, 0, u32::MAX - 255],
        ..dims
    };
    assert!(sleep_copy_state_elements(&launch_limit).is_ok());
    assert!(SleepCopySpec::launch_dims(&launch_limit).is_ok());
    assert!(matches!(
        sleep_copy_state_elements(&SleepCopyDims {
            abi: [0, 0, 0, u32::MAX - 254],
            ..dims
        }),
        Err(LauncherError::InvalidArray(_))
    ));
    let state_byte_overflow = SleepCopyDims {
        abi: [0, 0, 0, u32::MAX - 255],
        ntree: i32::MAX as u32,
        ..dims
    };
    let state_upper = sleep_copy_state_elements(&state_byte_overflow).unwrap();
    assert!(state_upper < u64::MAX && state_upper.checked_mul(4).is_none());
    assert!(matches!(
        sleep_copy_buffer_bytes(&state_byte_overflow, 2),
        Err(LauncherError::Overflow("sleep_copy_state_bytes"))
    ));
    assert!(matches!(
        sleep_copy_buffer_bytes(&state_byte_overflow, 3),
        Err(LauncherError::Overflow("sleep_copy_result_bytes"))
    ));
}
