//! 偏置力计划元数据、跨度与零自由度测试。

#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RneBiasDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::rne_bias_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::rne_bias_metadata_elements;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::validate_rne_bias_metadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::spec::RneBiasMetadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::spec::RneBiasSpec;
#[cfg(any(feature = "cuda-probe", test))]
use super::spec::rne_bias_plan;
use super::*;

#[test]
fn rne_bias_plan_checks_metadata_spans_aliases_and_overflow() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, addr, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, addr, bytes).unwrap(),
        elem,
    };
    let dims = RneBiasDims {
        worlds: 2,
        nbody: 2,
        nv: 1,
        metadata_elements: 12,
    };
    let values = vec![2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1];
    assert_eq!(rne_bias_lengths(&dims).unwrap(), [12, 10, 76, 44, 58]);
    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(11, 0x1000, 48).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<RneBiasMetadata>::assume_uploaded(metadata_span, &values, dims, revision.clone())
    }
    .unwrap();
    let bindings = [
        bind(0, 11, 0x1000, 48, ElemType::I32),
        bind(1, 11, 0x2000, 40, ElemType::F32),
        bind(2, 11, 0x3000, 304, ElemType::F32),
        bind(3, 11, 0x4000, 176, ElemType::F32),
        bind(4, 11, 0x5000, 232, ElemType::F32),
    ];
    let launch = rne_bias_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        rne_bias_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x3000, 0x4000, 0x5000];
    let bytes = [48, 40, 304, 176, 232];
    for slot in 0..5 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(matches!(
            rne_bias_plan(&dims, &short, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(matches!(
            rne_bias_plan(&dims, &long, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
    }

    let mut foreign = bindings;
    foreign[3].span = DeviceSpan::new(12, 0x4000, 176).unwrap();
    assert!(matches!(
        rne_bias_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 3 })
    ));
    let mut alias = bindings;
    alias[4].span = DeviceSpan::new(11, 0x2000, 232).unwrap();
    assert!(matches!(
        rne_bias_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 4
        })
    ));
    let mut read_alias = bindings;
    read_alias[2].span = DeviceSpan::new(11, 0x2000, 304).unwrap();
    rne_bias_plan(&dims, &read_alias, &[proof.binding(0)]).unwrap();

    revision.invalidate();
    assert!(matches!(
        rne_bias_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    for invalid in [
        RneBiasDims { worlds: 0, ..dims },
        RneBiasDims {
            worlds: u32::MAX - 254,
            ..dims
        },
        RneBiasDims {
            metadata_elements: 11,
            ..dims
        },
    ] {
        assert!(rne_bias_lengths(&invalid).is_err());
    }
    assert!(rne_bias_metadata_elements(i32::MAX as u32 + 1, 0).is_err());
    assert!(rne_bias_metadata_elements(715_827_881, 0).is_err());
    let layout_wide = RneBiasDims {
        worlds: 1,
        nbody: i32::MAX as u32 / 12 + 1,
        nv: 0,
        metadata_elements: rne_bias_metadata_elements(i32::MAX as u32 / 12 + 1, 0).unwrap(),
    };
    assert!(matches!(
        rne_bias_lengths(&layout_wide),
        Err(LauncherError::Overflow("rne_bias_output_width"))
    ));
    let wide = RneBiasDims {
        worlds: u32::MAX - 255,
        nbody: 400_000_000,
        nv: 0,
        metadata_elements: 1_200_000_005,
    };
    assert!(matches!(
        rne_bias_lengths(&wide),
        Err(LauncherError::Overflow("rne_bias_com"))
    ));
    let byte_overflow = RneBiasDims {
        worlds: u32::MAX - 255,
        nbody: 100_000_000,
        nv: 0,
        metadata_elements: 300_000_005,
    };
    let byte_overflow_bindings = [
        bind(0, 11, 0x1000, 1_200_000_020, ElemType::I32),
        bind(1, 11, 0x1_0000_0000, 32, ElemType::F32),
        bind(2, 11, 0x2_0000_0000, 4, ElemType::F32),
        bind(3, 11, 0x3_0000_0000, 4, ElemType::F32),
        bind(4, 11, 0x4_0000_0000, 4, ElemType::F32),
    ];
    assert!(matches!(
        rne_bias_plan(&byte_overflow, &byte_overflow_bindings, &[]),
        Err(LauncherError::Overflow("rne_bias_bytes"))
    ));
    let max_width = RneBiasDims {
        worlds: 1,
        nbody: 1,
        nv: i32::MAX as u32 - 12,
        metadata_elements: i32::MAX as u64 - 4,
    };
    assert!(rne_bias_lengths(&max_width).is_ok());
    let too_wide = RneBiasDims {
        nv: i32::MAX as u32 - 11,
        metadata_elements: i32::MAX as u64 - 3,
        ..max_width
    };
    assert!(matches!(
        rne_bias_lengths(&too_wide),
        Err(LauncherError::Overflow("rne_bias_output_width"))
    ));

    for (index, value) in [
        (2, f32::NAN.to_bits() as i32),
        (5, 1),
        (6, 1),
        (7, 1),
        (8, 2),
        (10, 1),
        (11, 0),
        (11, 2),
    ] {
        let mut invalid = values.clone();
        invalid[index] = value;
        assert!(validate_rne_bias_metadata(&invalid, dims).is_err());
    }
    let mut incomplete = values;
    incomplete[8] = 0;
    assert!(validate_rne_bias_metadata(&incomplete, dims).is_err());

    let tail = RneBiasDims {
        worlds: 513,
        ..dims
    };
    assert_eq!(RneBiasSpec::launch_dims(&tail).unwrap().grid(), (3, 1, 1));
}

#[test]
fn rne_bias_plan_accepts_zero_dof_boundary() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = RneBiasDims {
        worlds: 2,
        nbody: 2,
        nv: 0,
        metadata_elements: 11,
    };
    let values = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let revision = MetadataRevision::new();
    let span = DeviceSpan::new(13, 0x1000, 44).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof =
        unsafe { Checked::<RneBiasMetadata>::assume_uploaded(span, &values, dims, revision) }
            .unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(13, 0x2000, 32).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(13, 0x3000, 256).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(13, 0x4000, 128).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 4,
            span: DeviceSpan::new(13, 0x5000, 224).unwrap(),
            elem: ElemType::F32,
        },
    ];
    assert_eq!(rne_bias_lengths(&dims).unwrap(), [11, 8, 64, 32, 56]);
    rne_bias_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
}
