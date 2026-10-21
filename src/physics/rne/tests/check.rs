//! RNE布局与输入边界测试。

use super::data::check_values;
use super::launcher_abi::RneCaccDims;
use super::launcher_abi::RneStaticDims;
use super::launcher_meta::RneCaccSpec;
use super::launcher_meta::rne_cacc_lengths;
use super::launcher_meta::validate_rne_metadata;
use super::launcher_plans::RneCaccMetadata;
use super::launcher_plans::rne_cacc_plan;
use super::*;

#[test]
fn checks_acceleration_layout_and_input_boundaries() {
    for (w, b, v) in [
        (0, 1, 0),
        (1, 0, 0),
        (usize::MAX, 1, 0),
        (1, usize::MAX, 0),
        (1, 1, usize::MAX),
        (u32::MAX as usize, 2, 0),
    ] {
        assert!(RneLayout::new(w, b, v).is_err());
    }
    let layout = RneLayout::new(513, 1, 0).unwrap();
    assert_eq!(layout.output.total_elements(), 513 * 12);
    assert_eq!(layout.qacc.total_elements(), 0);
    assert!(check_values("test", 2, &[0.0]).is_err());
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(check_values("test", 1, &[value]).is_err());
    }
    assert!(check_values("test", 0, &[]).is_ok());
}

#[test]
fn rne_cacc_plan_checks_metadata_spans_aliases_and_overflow() {
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
    let dims = RneCaccDims {
        flag: 1,
        worlds: 2,
        static_dims,
    };
    let values = vec![2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1];
    assert_eq!(rne_cacc_lengths(&dims).unwrap(), [14, 10, 10, 76, 44, 58]);
    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(11, 0x1000, 56).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<RneCaccMetadata>::assume_uploaded(metadata_span, &values, dims, revision.clone())
    }
    .unwrap();
    let bindings = [
        bind(0, 11, 0x1000, 56, ElemType::I32),
        bind(1, 11, 0x2000, 40, ElemType::F32),
        bind(2, 11, 0x2000, 40, ElemType::F32),
        bind(3, 11, 0x3000, 304, ElemType::F32),
        bind(4, 11, 0x4000, 176, ElemType::F32),
        bind(5, 11, 0x5000, 232, ElemType::F32),
    ];
    let launch = rne_cacc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        rne_cacc_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x2000, 0x3000, 0x4000, 0x5000];
    let bytes = [56, 40, 40, 304, 176, 232];
    for slot in 0..6 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(matches!(
            rne_cacc_plan(&dims, &short, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(matches!(
            rne_cacc_plan(&dims, &long, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
    }

    let mut foreign = bindings;
    foreign[4].span = DeviceSpan::new(12, 0x4000, 176).unwrap();
    assert!(matches!(
        rne_cacc_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 4 })
    ));
    let mut alias = bindings;
    alias[5].span = DeviceSpan::new(11, 0x3000, 232).unwrap();
    assert!(matches!(
        rne_cacc_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 3,
            second: 5
        })
    ));

    revision.invalidate();
    assert!(matches!(
        rne_cacc_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    for invalid in [
        RneCaccDims { flag: 2, ..dims },
        RneCaccDims { worlds: 0, ..dims },
        RneCaccDims {
            worlds: u32::MAX - 254,
            ..dims
        },
    ] {
        assert!(rne_cacc_lengths(&invalid).is_err());
    }
    let signed = RneCaccDims {
        flag: 0,
        worlds: 1,
        static_dims: RneStaticDims {
            nbody: i32::MAX as u32 + 1,
            nv: 0,
            depth: 1,
            metadata_elements: 0,
        },
    };
    assert!(rne_cacc_lengths(&signed).is_err());
    let wide = RneCaccDims {
        flag: 0,
        worlds: u32::MAX - 255,
        static_dims: RneStaticDims {
            nbody: 400_000_000,
            nv: 0,
            depth: 1,
            metadata_elements: 1_600_000_005,
        },
    };
    assert!(matches!(
        rne_cacc_lengths(&wide),
        Err(LauncherError::Overflow("rne_cacc_com"))
    ));

    for (index, value) in [(2, f32::NAN.to_bits() as i32), (6, 1), (11, 2), (12, 2)] {
        let mut invalid = values.clone();
        invalid[index] = value;
        assert!(validate_rne_metadata(&invalid, static_dims).is_err());
    }
    let mut incomplete = values;
    incomplete[8] = 0;
    assert!(validate_rne_metadata(&incomplete, static_dims).is_err());

    let tail = RneCaccDims {
        worlds: 513,
        ..dims
    };
    let tail_launch = RneCaccSpec::launch_dims(&tail).unwrap();
    assert_eq!(tail_launch.grid(), (3, 1, 1));
}

#[test]
fn rne_cacc_plan_accepts_zero_dof_boundary() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = RneCaccDims {
        flag: 0,
        worlds: 2,
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
        unsafe { Checked::<RneCaccMetadata>::assume_uploaded(span, &values, dims, revision) }
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
            span: DeviceSpan::new(13, 0x2000, 32).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(13, 0x3000, 256).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 4,
            span: DeviceSpan::new(13, 0x4000, 128).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 5,
            span: DeviceSpan::new(13, 0x5000, 224).unwrap(),
            elem: ElemType::F32,
        },
    ];
    assert_eq!(rne_cacc_lengths(&dims).unwrap(), [13, 8, 8, 64, 32, 56]);
    rne_cacc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
}
