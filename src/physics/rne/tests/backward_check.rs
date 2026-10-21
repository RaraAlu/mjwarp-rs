//! RNE反向层级计划测试。

use super::launcher_abi::RneBackwardDims;
use super::launcher_abi::RneStaticDims;
use super::launcher_meta::RneBackwardSpec;
use super::launcher_meta::rne_backward_lengths;
use super::launcher_plans::RneBackwardMetadata;
use super::launcher_plans::rne_backward_plan;
use super::*;

#[test]
fn rne_backward_plan_checks_levels_spans_aliases_and_overflow() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, addr, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, addr, bytes).unwrap(),
        elem,
    };
    let static_dims = RneStaticDims {
        nbody: 4,
        nv: 0,
        depth: 3,
        metadata_elements: 21,
    };
    let dims = RneBackwardDims {
        abi: [4, 3, 0, 8],
        worlds: 2,
        static_dims,
    };
    let values = vec![
        4, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3,
    ];
    assert_eq!(rne_backward_lengths(&dims).unwrap(), [21, 120, 56, 104]);
    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(17, 0x1000, 84).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<RneBackwardMetadata>::assume_uploaded(
            metadata_span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        bind(0, 17, 0x1000, 84, ElemType::I32),
        bind(1, 17, 0x2000, 480, ElemType::F32),
        bind(2, 17, 0x3000, 224, ElemType::F32),
        bind(3, 17, 0x4000, 416, ElemType::F32),
    ];
    let launch = rne_backward_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        rne_backward_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x3000, 0x4000];
    let bytes = [84, 480, 224, 416];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(17, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(matches!(
            rne_backward_plan(&dims, &short, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(17, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(matches!(
            rne_backward_plan(&dims, &long, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
    }

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(18, 0x3000, 224).unwrap();
    assert!(matches!(
        rne_backward_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(17, 0x2000, 416).unwrap();
    assert!(matches!(
        rne_backward_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));

    revision.invalidate();
    assert!(matches!(
        rne_backward_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    for invalid in [
        RneBackwardDims {
            abi: [4, 0, 0, 8],
            ..dims
        },
        RneBackwardDims {
            abi: [4, 4, 0, 8],
            ..dims
        },
        RneBackwardDims {
            abi: [3, 3, 0, 8],
            ..dims
        },
        RneBackwardDims {
            abi: [4, 3, 1, 8],
            ..dims
        },
        RneBackwardDims {
            abi: [4, 3, 0, 7],
            ..dims
        },
        RneBackwardDims { worlds: 0, ..dims },
    ] {
        assert!(rne_backward_lengths(&invalid).is_err());
    }
    let overflow = RneBackwardDims {
        abi: [500_000_000, 1, 0, 0],
        worlds: 9,
        static_dims: RneStaticDims {
            nbody: 500_000_000,
            nv: 0,
            depth: 1,
            metadata_elements: 2_000_000_005,
        },
    };
    assert!(matches!(
        rne_backward_lengths(&overflow),
        Err(LauncherError::Overflow("rne_backward_work"))
    ));
    let max_work = u32::MAX - 255;
    let max_grid = RneBackwardDims {
        abi: [2, 1, 0, max_work],
        worlds: max_work / 2,
        static_dims: RneStaticDims {
            nbody: 2,
            nv: 0,
            depth: 1,
            metadata_elements: 13,
        },
    };
    assert!(rne_backward_lengths(&max_grid).is_ok());
    assert_eq!(
        RneBackwardSpec::launch_dims(&max_grid).unwrap().grid(),
        (16_777_215, 1, 1)
    );
    let beyond_grid = RneBackwardDims {
        abi: [2, 1, 0, max_work + 2],
        worlds: max_work / 2 + 1,
        ..max_grid
    };
    assert!(matches!(
        rne_backward_lengths(&beyond_grid),
        Err(LauncherError::Overflow("rne_backward_work"))
    ));
    let byte_overflow = RneBackwardDims {
        abi: [2, 1, 0, max_work],
        worlds: max_work / 2,
        static_dims: RneStaticDims {
            nbody: 2,
            nv: 600_000_000,
            depth: 1,
            metadata_elements: 600_000_013,
        },
    };
    assert!(rne_backward_lengths(&byte_overflow).is_ok());
    let byte_bindings = [
        bind(0, 17, 0x1000, 2_400_000_052, ElemType::I32),
        bind(1, 17, 0x1_0000_0000, 4, ElemType::F32),
        bind(2, 17, 0x2_0000_0000, 4, ElemType::F32),
        bind(3, 17, 0x3_0000_0000, 4, ElemType::F32),
    ];
    assert!(matches!(
        rne_backward_plan(&byte_overflow, &byte_bindings, &[]),
        Err(LauncherError::Overflow("rne_backward_bytes"))
    ));
    let root_only = RneBackwardDims {
        abi: [1, 0, 0, 1],
        worlds: 1,
        static_dims: RneStaticDims {
            nbody: 1,
            nv: 0,
            depth: 0,
            metadata_elements: 9,
        },
    };
    assert!(rne_backward_lengths(&root_only).is_err());
}
