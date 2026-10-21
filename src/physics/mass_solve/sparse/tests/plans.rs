//! 稀疏质量计划边界测试。

#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassCopyDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassCopyLaunchMetadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassDiagDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassDiagLaunchMetadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassUpdateDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassUpdateLaunchMetadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::validate_sparse_mass_update_level;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::validate_sparse_mass_update_metadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_diag::sparse_mass_diag_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_diag::sparse_mass_diag_plan;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_specs::sparse_mass_copy_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_specs::sparse_mass_copy_plan;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_specs::sparse_mass_update_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_specs::sparse_mass_update_plan;
use super::*;

#[test]
fn sparse_mass_copy_plan_checks_metadata_spans_empty_and_aliases() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, addr, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, addr, bytes).unwrap(),
        elem,
    };
    let dims = SparseMassCopyDims {
        launch: [0, 0, 7, 2],
        nv: 4,
        nc: 3,
        metadata_elements: 9,
    };
    assert_eq!(sparse_mass_copy_lengths(&dims).unwrap(), [9, 14, 14, 22]);
    let values = vec![4, 3, 0, 0, 0, 0, 0, 0, 0];
    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(7, 0x1000, 36).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<SparseMassCopyLaunchMetadata>::assume_uploaded(
            metadata_span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        bind(0, 7, 0x1000, 36, ElemType::I32),
        bind(1, 7, 0x2000, 56, ElemType::F32),
        bind(2, 7, 0x2000, 56, ElemType::F32),
        bind(3, 7, 0x3000, 88, ElemType::F32),
    ];
    let launch = sparse_mass_copy_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        sparse_mass_copy_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x2000, 0x3000];
    let bytes = [36, 56, 56, 88];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(sparse_mass_copy_plan(&dims, &short, &[proof.binding(0)]).is_err());

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(sparse_mass_copy_plan(&dims, &long, &[proof.binding(0)]).is_err());
    }

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x2000, 56).unwrap();
    assert!(matches!(
        sparse_mass_copy_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));

    let alias_dims = SparseMassCopyDims {
        launch: [0, 0, 3, 2],
        nv: 0,
        nc: 3,
        metadata_elements: 9,
    };
    let alias_values = vec![0, 3, 0, 0, 0, 0, 0, 0, 0];
    let alias_revision = MetadataRevision::new();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let alias_proof = unsafe {
        Checked::<SparseMassCopyLaunchMetadata>::assume_uploaded(
            metadata_span,
            &alias_values,
            alias_dims,
            alias_revision,
        )
    }
    .unwrap();
    let mut alias = [
        bind(0, 7, 0x1000, 36, ElemType::I32),
        bind(1, 7, 0x2000, 56, ElemType::F32),
        bind(2, 7, 0x2000, 56, ElemType::F32),
        bind(3, 7, 0x2000, 56, ElemType::F32),
    ];
    assert!(matches!(
        sparse_mass_copy_plan(&alias_dims, &alias, &[alias_proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));
    alias[3].span = DeviceSpan::new(7, 0x3000, 56).unwrap();
    sparse_mass_copy_plan(&alias_dims, &alias, &[alias_proof.binding(0)]).unwrap();

    let empty_dims = SparseMassCopyDims {
        launch: [0, 0, 0, 2],
        nv: 0,
        nc: 0,
        metadata_elements: 5,
    };
    let empty_values = vec![0; 5];
    let empty_revision = MetadataRevision::new();
    let empty_span = DeviceSpan::new(7, 0x4000, 20).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let empty_proof = unsafe {
        Checked::<SparseMassCopyLaunchMetadata>::assume_uploaded(
            empty_span,
            &empty_values,
            empty_dims,
            empty_revision,
        )
    }
    .unwrap();
    let empty = [
        bind(0, 7, 0x4000, 20, ElemType::I32),
        bind(1, 7, 0x5000, 32, ElemType::F32),
        bind(2, 7, 0x5000, 32, ElemType::F32),
        bind(3, 7, 0x6000, 32, ElemType::F32),
    ];
    sparse_mass_copy_plan(&empty_dims, &empty, &[empty_proof.binding(0)]).unwrap();

    revision.invalidate();
    assert!(matches!(
        sparse_mass_copy_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    assert!(SparseMassCopyLaunchMetadata::validate(&values[1..], &dims).is_err());
    let mut bad_values = values;
    bad_values[0] = 3;
    assert!(SparseMassCopyLaunchMetadata::validate(&bad_values, &dims).is_err());

    for invalid in [
        SparseMassCopyDims {
            launch: [1, 0, 7, 2],
            ..dims
        },
        SparseMassCopyDims {
            launch: [0, 1, 7, 2],
            ..dims
        },
        SparseMassCopyDims {
            launch: [0, 0, 6, 2],
            ..dims
        },
        SparseMassCopyDims {
            launch: [0, 0, 7, 0],
            ..dims
        },
        SparseMassCopyDims {
            launch: [0, 0, 7, u32::MAX - 254],
            ..dims
        },
        SparseMassCopyDims {
            launch: [0, 0, 0, 1],
            nv: 1,
            nc: u32::MAX,
            metadata_elements: 9,
        },
        SparseMassCopyDims {
            metadata_elements: 1,
            ..dims
        },
    ] {
        assert!(sparse_mass_copy_lengths(&invalid).is_err());
    }
}

#[test]
fn sparse_mass_update_plan_checks_real_levels_spans_and_aliases() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, addr, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, addr, bytes).unwrap(),
        elem,
    };
    let values = vec![
        3, 6, // nv, nc
        1, 2, 3, // rownnz
        0, 1, 3, // rowadr
        0, 1, 1, // 共享根目标一
        0, 2, 3, // 共享根目标二
        1, 2, 4, // 下一层
    ];
    let dims = SparseMassUpdateDims {
        launch: [0, 2, 9, 4],
        nv: 3,
        nc: 6,
        metadata_elements: 17,
        update_count: 3,
        worlds: 2,
    };
    let levels = [0, 2, 3];
    validate_sparse_mass_update_level(&values, &dims, &levels, 0).unwrap();
    assert_eq!(sparse_mass_update_lengths(&dims).unwrap(), [17, 20, 20, 26]);

    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(7, 0x1000, 68).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<SparseMassUpdateLaunchMetadata>::assume_uploaded(
            metadata_span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        bind(0, 7, 0x1000, 68, ElemType::I32),
        bind(1, 7, 0x2000, 80, ElemType::F32),
        bind(2, 7, 0x2000, 80, ElemType::F32),
        bind(3, 7, 0x3000, 104, ElemType::F32),
    ];
    let launch = sparse_mass_update_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        sparse_mass_update_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x2000, 0x3000];
    let bytes = [68, 80, 80, 104];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(sparse_mass_update_plan(&dims, &short, &[proof.binding(0)]).is_err());

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(sparse_mass_update_plan(&dims, &long, &[proof.binding(0)]).is_err());
    }

    let mut foreign = bindings;
    foreign[1].span = DeviceSpan::new(8, 0x2000, 80).unwrap();
    assert!(matches!(
        sparse_mass_update_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 1 })
    ));

    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, 0x2000, 104).unwrap();
    assert!(matches!(
        sparse_mass_update_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));

    let mut wrong_level = dims;
    wrong_level.launch = [1, 2, 9, 4];
    assert!(validate_sparse_mass_update_level(&values, &wrong_level, &levels, 0).is_err());
    assert!(validate_sparse_mass_update_level(&values, &dims, &levels, usize::MAX).is_err());
    let mut duplicate_source = values.clone();
    duplicate_source[11..14].copy_from_slice(&[0, 1, 1]);
    validate_sparse_mass_update_metadata(&duplicate_source, 3, 6, 17, 3).unwrap();
    assert!(validate_sparse_mass_update_level(&duplicate_source, &dims, &levels, 0).is_err());

    let empty = [0, 0, 0, 0, 0];
    validate_sparse_mass_update_metadata(&empty, 0, 0, 5, 0).unwrap();
    for invalid in [
        SparseMassUpdateDims {
            launch: [0, 0, 9, 0],
            ..dims
        },
        SparseMassUpdateDims {
            launch: [2, 2, 9, 4],
            ..dims
        },
        SparseMassUpdateDims {
            launch: [0, 2, 8, 4],
            ..dims
        },
        SparseMassUpdateDims {
            launch: [0, 2, 9, 5],
            ..dims
        },
        SparseMassUpdateDims {
            launch: [0, 2, 9, 0],
            worlds: 0,
            ..dims
        },
    ] {
        assert!(sparse_mass_update_lengths(&invalid).is_err());
    }

    revision.invalidate();
    assert!(matches!(
        sparse_mass_update_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    let mut bad_prefix = values;
    bad_prefix[10] = 0;
    assert!(validate_sparse_mass_update_metadata(&bad_prefix, 3, 6, 17, 3).is_err());
}

#[test]
fn sparse_mass_diag_plan_checks_spans_aliases_and_zero_nv() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, addr, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, addr, bytes).unwrap(),
        elem,
    };
    let values = vec![
        3, 6, // nv, nc
        1, 2, 3, // rownnz
        0, 1, 3, // rowadr
        0, 1, 1, // 共享根目标一
        0, 2, 3, // 共享根目标二
        1, 2, 4, // 下一层
    ];
    let dims = SparseMassDiagDims {
        launch: [0, 0, 9, 6],
        nv: 3,
        nc: 6,
        metadata_elements: 17,
        update_count: 3,
        worlds: 2,
    };
    assert_eq!(sparse_mass_diag_lengths(&dims).unwrap(), [17, 20, 20, 26]);

    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(7, 0x1000, 68).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<SparseMassDiagLaunchMetadata>::assume_uploaded(
            metadata_span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        bind(0, 7, 0x1000, 68, ElemType::I32),
        bind(1, 7, 0x2000, 80, ElemType::F32),
        bind(2, 7, 0x2000, 80, ElemType::F32),
        bind(3, 7, 0x3000, 104, ElemType::F32),
    ];
    let launch = sparse_mass_diag_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        sparse_mass_diag_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x2000, 0x3000];
    let bytes = [68, 80, 80, 104];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(sparse_mass_diag_plan(&dims, &short, &[proof.binding(0)]).is_err());

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(sparse_mass_diag_plan(&dims, &long, &[proof.binding(0)]).is_err());
    }

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x2000, 80).unwrap();
    assert!(matches!(
        sparse_mass_diag_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));

    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, 0x2000, 104).unwrap();
    assert!(matches!(
        sparse_mass_diag_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));

    for invalid in [
        SparseMassDiagDims {
            launch: [1, 0, 9, 6],
            ..dims
        },
        SparseMassDiagDims {
            launch: [0, 1, 9, 6],
            ..dims
        },
        SparseMassDiagDims {
            launch: [0, 0, 8, 6],
            ..dims
        },
        SparseMassDiagDims {
            launch: [0, 0, 9, 5],
            ..dims
        },
        SparseMassDiagDims {
            launch: [0, 0, 9, 0],
            worlds: 0,
            ..dims
        },
        SparseMassDiagDims {
            launch: [0, 0, 9, 1],
            worlds: u32::MAX,
            ..dims
        },
    ] {
        assert!(sparse_mass_diag_lengths(&invalid).is_err());
    }

    let empty_values = [0, 0, 0, 0, 0];
    validate_sparse_mass_update_metadata(&empty_values, 0, 0, 5, 0).unwrap();
    let empty_dims = SparseMassDiagDims {
        launch: [0, 0, 0, 0],
        nv: 0,
        nc: 0,
        metadata_elements: 5,
        update_count: 0,
        worlds: 2,
    };
    assert!(sparse_mass_diag_lengths(&empty_dims).is_err());

    revision.invalidate();
    assert!(matches!(
        sparse_mass_diag_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
}
