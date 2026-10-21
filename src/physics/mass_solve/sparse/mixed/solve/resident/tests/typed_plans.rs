//! 常驻混合类型化校验与聚集计划测试。

#[cfg(feature = "cuda-probe")]
use super::gather_meta::ResidentMixedGatherLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::ResidentMixedGatherSpec;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::StaticResidentMixedGatherMetadata;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_dims;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_lengths;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_plan;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::ResidentMixedValidationSpec;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_dimension;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_lengths;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_plan;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
#[test]
#[cfg(feature = "cuda-probe")]
fn typed_resident_mixed_validation_plan_freezes_exact_contract() {
    use crate::runtime::launcher::DeviceSpan;

    let expected = [69, 68, 2149];
    let dims = [expected[0], expected[1], expected[2], 257];
    let (source_len, status_len) = resident_mixed_validation_lengths(&dims).unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: DeviceSpan::new(7, 0x1000, 4).unwrap(),
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(7, 0x20_0000, source_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x80_0000, 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x90_0000, status_len * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    resident_mixed_validation_plan(expected, &dims, &bindings).unwrap();
    let launch = ResidentMixedValidationSpec::launch_dims(&dims).unwrap();
    assert_eq!(launch.grid(), (2, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));

    for (slot, bytes) in [(1, source_len * 4 - 4), (3, status_len * 4 - 4)] {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, short[slot].span.addr(), bytes).unwrap();
        assert!(resident_mixed_validation_plan(expected, &dims, &short).is_err());
    }
    for (slot, bytes) in [(1, source_len * 4 + 4), (3, status_len * 4 + 4)] {
        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, long[slot].span.addr(), bytes).unwrap();
        assert!(resident_mixed_validation_plan(expected, &dims, &long).is_err());
    }

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x80_0000, 4).unwrap();
    assert!(matches!(
        resident_mixed_validation_plan(expected, &dims, &foreign),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));

    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, bindings[1].span.addr(), status_len * 4).unwrap();
    assert!(matches!(
        resident_mixed_validation_plan(expected, &dims, &alias),
        Err(LauncherError::Alias {
            first: 1,
            second: 3,
        })
    ));

    assert!(
        resident_mixed_validation_plan(
            expected,
            &[expected[0] + 1, expected[1], expected[2], 257],
            &bindings,
        )
        .is_err()
    );
    assert!(matches!(
        resident_mixed_validation_lengths(&[u32::MAX, u32::MAX, 0, 2]),
        Err(LauncherError::Overflow(_))
    ));
    assert!(ResidentMixedValidationSpec::launch_dims(&[1, 1, 0, u32::MAX - 254,]).is_err());
    if usize::BITS > 32 {
        assert!(
            resident_mixed_validation_dimension(usize::MAX, "resident_mixed_validation_test")
                .is_err()
        );
    }
}

#[test]
#[cfg(feature = "cuda-probe")]
fn typed_resident_mixed_gather_plan_binds_csr_and_exact_spans() {
    use crate::runtime::launcher::DeviceSpan;

    let values = vec![1, 2, 3, 0, 1, 3, 0, 0, 1, 0, 1, 2];
    assert!(StaticResidentMixedGatherMetadata::validate(&values, &[3, 6]).is_ok());
    for invalid in [
        vec![1, 2, 3, 0, 2, 3, 0, 0, 1, 0, 1, 2],
        vec![1, 2, 3, 0, 1, 3, 0, 0, 1, 0, 2, 1],
        vec![1, 2, 3, 0, 1, 3, 0, 0, 1, 0, 1, 3],
    ] {
        assert!(StaticResidentMixedGatherMetadata::validate(&invalid, &[3, 6]).is_err());
    }
    assert!(StaticResidentMixedGatherMetadata::validate(&values[..11], &[3, 6]).is_err());

    let dims = [2, 3, 6, 771];
    let (metadata_len, source_len, output_len) = resident_mixed_gather_lengths(&dims).unwrap();
    assert_eq!((metadata_len, source_len, output_len), (12, 7461, 1550));
    let metadata_span = DeviceSpan::new(7, 0x1000, metadata_len * 4).unwrap();
    let revision = MetadataRevision::new();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<ResidentMixedGatherLaunchMetadata>::assume_uploaded(
            metadata_span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: metadata_span,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(7, 0x20_0000, source_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x80_0000, 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x90_0000, output_len * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    resident_mixed_gather_plan(&dims, &bindings, proof.binding(0)).unwrap();
    let launch = ResidentMixedGatherSpec::launch_dims(&dims).unwrap();
    assert_eq!(launch.grid(), (4, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));

    for (slot, bytes) in [
        (0, metadata_len * 4),
        (1, source_len * 4),
        (3, output_len * 4),
    ] {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, short[slot].span.addr(), bytes - 4).unwrap();
        assert!(resident_mixed_gather_plan(&dims, &short, proof.binding(0)).is_err());
        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, long[slot].span.addr(), bytes + 4).unwrap();
        assert!(resident_mixed_gather_plan(&dims, &long, proof.binding(0)).is_err());
    }

    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, bindings[1].span.addr(), output_len * 4).unwrap();
    assert!(matches!(
        resident_mixed_gather_plan(&dims, &alias, proof.binding(0)),
        Err(LauncherError::Alias {
            first: 1,
            second: 3,
        })
    ));
    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x80_0000, 4).unwrap();
    assert!(matches!(
        resident_mixed_gather_plan(&dims, &foreign, proof.binding(0)),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let replaced = DeviceSpan::new(7, 0xa0_0000, metadata_len * 4).unwrap();
    let mut wrong_binding = bindings;
    wrong_binding[0].span = replaced;
    assert!(matches!(
        resident_mixed_gather_plan(&dims, &wrong_binding, proof.binding(0)),
        Err(LauncherError::MetadataMismatch(_))
    ));
    revision.invalidate();
    assert!(matches!(
        resident_mixed_gather_plan(&dims, &bindings, proof.binding(0)),
        Err(LauncherError::StaleMetadata)
    ));
    assert!(resident_mixed_gather_lengths(&[2, 3, 6, 0]).is_err());
    assert!(resident_mixed_gather_lengths(&[2, 3, 6, 770]).is_err());
    assert!(resident_mixed_gather_lengths(&[2, 3, 6, u32::MAX - 254]).is_err());
    assert!(matches!(
        resident_mixed_gather_dims(2, 3, 6, u32::MAX),
        Err(LauncherError::Overflow("resident_mixed_gather_work"))
    ));
}
