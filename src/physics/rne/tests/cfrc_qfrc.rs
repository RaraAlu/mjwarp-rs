//! RNE复合力与广义力计划测试。

use super::launcher_abi::RneCfrcDims;
use super::launcher_abi::RneQfrcDims;
use super::launcher_abi::RneStaticDims;
use super::launcher_meta::rne_cfrc_lengths;
use super::launcher_meta::rne_metadata_elements;
use super::launcher_meta::rne_qfrc_lengths;
use super::launcher_meta::validate_rne_metadata;
use super::launcher_plans::RneCfrcMetadata;
use super::launcher_plans::RneQfrcMetadata;
use super::launcher_plans::rne_cfrc_plan;
use super::launcher_plans::rne_qfrc_plan;
use super::*;

#[test]
fn rne_cfrc_plan_checks_metadata_spans_aliases_and_overflow() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, addr, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, addr, bytes).unwrap(),
        elem,
    };
    let static_dims = RneStaticDims {
        nbody: 2,
        nv: 1,
        depth: 1,
        metadata_elements: 14,
    };
    let dims = RneCfrcDims {
        abi: [0, 0, 0, 2],
        static_dims,
    };
    let values = vec![2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1];
    assert_eq!(rne_cfrc_lengths(&dims).unwrap(), [14, 76, 44, 58]);
    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(11, 0x1000, 56).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<RneCfrcMetadata>::assume_uploaded(metadata_span, &values, dims, revision.clone())
    }
    .unwrap();
    let bindings = [
        bind(0, 11, 0x1000, 56, ElemType::I32),
        bind(1, 11, 0x2000, 304, ElemType::F32),
        bind(2, 11, 0x2000, 176, ElemType::F32),
        bind(3, 11, 0x4000, 232, ElemType::F32),
    ];
    let launch = rne_cfrc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        rne_cfrc_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x2000, 0x4000];
    let bytes = [56, 304, 176, 232];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(matches!(
            rne_cfrc_plan(&dims, &short, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(matches!(
            rne_cfrc_plan(&dims, &long, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
    }

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(12, 0x2000, 176).unwrap();
    assert!(matches!(
        rne_cfrc_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(11, 0x2000, 232).unwrap();
    assert!(matches!(
        rne_cfrc_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));

    revision.invalidate();
    assert!(matches!(
        rne_cfrc_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    for invalid in [
        RneCfrcDims {
            abi: [1, 0, 0, 2],
            ..dims
        },
        RneCfrcDims {
            abi: [0, 0, 0, 0],
            ..dims
        },
        RneCfrcDims {
            abi: [0, 0, 0, u32::MAX - 254],
            ..dims
        },
    ] {
        assert!(rne_cfrc_lengths(&invalid).is_err());
    }
    let signed = RneCfrcDims {
        abi: [0, 0, 0, 1],
        static_dims: RneStaticDims {
            nbody: i32::MAX as u32 + 1,
            nv: 0,
            depth: 1,
            metadata_elements: 0,
        },
    };
    assert!(rne_cfrc_lengths(&signed).is_err());
    let wide = RneCfrcDims {
        abi: [0, 0, 0, u32::MAX - 255],
        static_dims: RneStaticDims {
            nbody: 400_000_000,
            nv: 0,
            depth: 1,
            metadata_elements: 1_600_000_005,
        },
    };
    assert!(matches!(
        rne_cfrc_lengths(&wide),
        Err(LauncherError::Overflow("rne_cfrc_com"))
    ));

    for (index, value) in [(2, f32::NAN.to_bits() as i32), (6, 1), (11, 2), (12, 2)] {
        let mut invalid = values.clone();
        invalid[index] = value;
        assert!(validate_rne_metadata(&invalid, static_dims).is_err());
    }
    let mut incomplete = values;
    incomplete[8] = 0;
    assert!(validate_rne_metadata(&incomplete, static_dims).is_err());
}

#[test]
fn rne_cfrc_plan_accepts_zero_dof_boundary() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = RneCfrcDims {
        abi: [0, 0, 0, 2],
        static_dims: RneStaticDims {
            nbody: 2,
            nv: 0,
            depth: 1,
            metadata_elements: 13,
        },
    };
    let values = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    let revision = MetadataRevision::new();
    let span = DeviceSpan::new(13, 0x1000, 52).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof =
        unsafe { Checked::<RneCfrcMetadata>::assume_uploaded(span, &values, dims, revision) }
            .unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(13, 0x2000, 256).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(13, 0x2000, 128).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(13, 0x3000, 224).unwrap(),
            elem: ElemType::F32,
        },
    ];
    assert_eq!(rne_cfrc_lengths(&dims).unwrap(), [13, 64, 32, 56]);
    rne_cfrc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
}

#[test]
fn rne_qfrc_plan_checks_metadata_spans_aliases_and_overflow() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, addr, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, addr, bytes).unwrap(),
        elem,
    };
    let static_dims = RneStaticDims {
        nbody: 2,
        nv: 1,
        depth: 1,
        metadata_elements: 14,
    };
    let dims = RneQfrcDims {
        abi: [0, 0, 0, 2],
        static_dims,
    };
    let values = vec![2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1];
    assert_eq!(rne_qfrc_lengths(&dims).unwrap(), [14, 76, 44, 58]);
    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(7, 0x1000, 56).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<RneQfrcMetadata>::assume_uploaded(metadata_span, &values, dims, revision.clone())
    }
    .unwrap();
    let bindings = [
        bind(0, 7, 0x1000, 56, ElemType::I32),
        bind(1, 7, 0x2000, 304, ElemType::F32),
        bind(2, 7, 0x2000, 176, ElemType::F32),
        bind(3, 7, 0x4000, 232, ElemType::F32),
    ];
    let launch = rne_qfrc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        rne_qfrc_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x2000, 0x4000];
    let bytes = [56, 304, 176, 232];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(matches!(
            rne_qfrc_plan(&dims, &short, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(matches!(
            rne_qfrc_plan(&dims, &long, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
    }

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x2000, 176).unwrap();
    assert!(matches!(
        rne_qfrc_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, 0x2000, 232).unwrap();
    assert!(matches!(
        rne_qfrc_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));

    revision.invalidate();
    assert!(matches!(
        rne_qfrc_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    for invalid in [
        RneQfrcDims {
            abi: [1, 0, 0, 2],
            ..dims
        },
        RneQfrcDims {
            abi: [0, 0, 0, 0],
            ..dims
        },
        RneQfrcDims {
            abi: [0, 0, 0, u32::MAX - 255],
            ..dims
        },
    ] {
        assert!(rne_qfrc_lengths(&invalid).is_err());
    }
    assert!(rne_metadata_elements(i32::MAX as u32 + 1, 0).is_err());

    for (index, value) in [(2, f32::NAN.to_bits() as i32), (6, 1), (11, 2), (12, 2)] {
        let mut invalid = values.clone();
        invalid[index] = value;
        assert!(validate_rne_metadata(&invalid, static_dims).is_err());
    }
    let mut incomplete = values;
    incomplete[8] = 0;
    assert!(validate_rne_metadata(&incomplete, static_dims).is_err());
}

#[test]
fn rne_qfrc_plan_accepts_zero_dof_boundary() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = RneQfrcDims {
        abi: [0, 0, 0, 2],
        static_dims: RneStaticDims {
            nbody: 2,
            nv: 0,
            depth: 1,
            metadata_elements: 13,
        },
    };
    let values = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    let revision = MetadataRevision::new();
    let span = DeviceSpan::new(9, 0x1000, 52).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof =
        unsafe { Checked::<RneQfrcMetadata>::assume_uploaded(span, &values, dims, revision) }
            .unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(9, 0x2000, 256).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(9, 0x2000, 128).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(9, 0x3000, 224).unwrap(),
            elem: ElemType::F32,
        },
    ];
    assert_eq!(rne_qfrc_lengths(&dims).unwrap(), [13, 64, 32, 56]);
    rne_qfrc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
}
