//! 质量求解计划跨度、别名与冻结格式测试。

#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RigidMassSolveDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::rigid_mass_solve_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::proof::RigidMassSolveMetadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::proof::rigid_mass_solve_plan;
#[cfg(any(feature = "cuda-probe", test))]
use super::validate::pack_rigid_mass_solve_blocks;
#[cfg(any(feature = "cuda-probe", test))]
use super::validate::validate_rigid_mass_solve_metadata;
use super::*;

#[test]
fn rigid_mass_solve_plan_checks_spans_aliases_blocks_and_overflow() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, address, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, address, bytes).unwrap(),
        elem,
    };
    let values = vec![2, 0, 2, 2, 2, 1, 0];
    let dims = RigidMassSolveDims {
        abi: [1, 3, 2, 2],
        metadata_elements: values.len() as u64,
    };
    assert_eq!(rigid_mass_solve_lengths(&dims).unwrap(), [7, 12, 46, 50]);
    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(7, 0x1000, 28).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<RigidMassSolveMetadata>::assume_uploaded(
            metadata_span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        bind(0, 7, 0x1000, 28, ElemType::I32),
        bind(1, 7, 0x2000, 48, ElemType::F32),
        bind(2, 7, 0x3000, 184, ElemType::F32),
        bind(3, 7, 0x4000, 200, ElemType::F32),
    ];
    let launch = rigid_mass_solve_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        rigid_mass_solve_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x3000, 0x4000];
    let bytes = [28, 48, 184, 200];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(matches!(
            rigid_mass_solve_plan(&dims, &short, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(matches!(
            rigid_mass_solve_plan(&dims, &long, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
    }

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x3000, 184).unwrap();
    assert!(matches!(
        rigid_mass_solve_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, 0x2000, 200).unwrap();
    assert!(matches!(
        rigid_mass_solve_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));
    revision.invalidate();
    assert!(matches!(
        rigid_mass_solve_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));

    for invalid in [
        vec![-1],
        vec![1, 1, 2, 2],
        vec![1, 0, 0, 2],
        vec![1, 0, 4, 2],
        vec![1, 0, 3, 3],
        vec![1, 0, 2, 2],
    ] {
        let invalid_dims = RigidMassSolveDims {
            metadata_elements: invalid.len() as u64,
            ..dims
        };
        assert!(validate_rigid_mass_solve_metadata(&invalid, invalid_dims).is_err());
    }
    assert!(
        pack_rigid_mass_solve_blocks(
            &[MassFactorBlock {
                start: 0,
                size: 2,
                kind: MassFactorKind::ReverseLdl,
            }],
            2,
        )
        .is_ok()
    );
    assert!(
        pack_rigid_mass_solve_blocks(
            &[MassFactorBlock {
                start: usize::MAX,
                size: 1,
                kind: MassFactorKind::Diagonal,
            }],
            1,
        )
        .is_err()
    );

    let zero = RigidMassSolveDims {
        abi: [1, 0, 3, 2],
        metadata_elements: 1,
    };
    assert_eq!(rigid_mass_solve_lengths(&zero).unwrap(), [1, 1, 28, 14]);
    assert!(validate_rigid_mass_solve_metadata(&[0], zero).is_ok());
    let grid_overflow = RigidMassSolveDims {
        abi: [1, 0, 1, u32::MAX - 254],
        metadata_elements: 1,
    };
    assert!(rigid_mass_solve_lengths(&grid_overflow).is_err());
    let byte_overflow = RigidMassSolveDims {
        abi: [1, 1, i32::MAX as u32 - 5, u32::MAX - 255],
        metadata_elements: 4,
    };
    let byte_bindings = [
        bind(0, 7, 0x1000, 16, ElemType::I32),
        bind(1, 7, 0x2000, 4, ElemType::F32),
        bind(2, 7, 0x3000, 4, ElemType::F32),
        bind(3, 7, 0x4000, 4, ElemType::F32),
    ];
    assert!(rigid_mass_solve_lengths(&byte_overflow).is_ok());
    assert!(matches!(
        rigid_mass_solve_plan(&byte_overflow, &byte_bindings, &[]),
        Err(LauncherError::Overflow("rigid_mass_solve_bytes"))
    ));
}

#[test]
fn rigid_mass_solve_plan_preserves_i32_layout_and_block_routes() {
    let state_limit = RigidMassSolveDims {
        abi: [i32::MAX as u32 / 10, 0, 1, 1],
        metadata_elements: 1,
    };
    assert!(rigid_mass_solve_lengths(&state_limit).is_ok());
    let state_over = RigidMassSolveDims {
        abi: [i32::MAX as u32 / 10 + 1, 0, 1, 1],
        ..state_limit
    };
    assert!(matches!(
        rigid_mass_solve_lengths(&state_over),
        Err(LauncherError::Overflow("rigid_mass_solve_state_width"))
    ));

    let output_limit = RigidMassSolveDims {
        abi: [0, 46_339, 1, 1],
        metadata_elements: 4,
    };
    assert!(rigid_mass_solve_lengths(&output_limit).is_ok());
    let output_over = RigidMassSolveDims {
        abi: [0, 46_340, 1, 1],
        ..output_limit
    };
    assert!(matches!(
        rigid_mass_solve_lengths(&output_over),
        Err(LauncherError::Overflow("rigid_mass_solve_output_width"))
    ));

    let reverse_two = RigidMassSolveDims {
        abi: [1, 2, 1, 1],
        metadata_elements: 4,
    };
    assert!(validate_rigid_mass_solve_metadata(&[1, 0, 2, 2], reverse_two).is_ok());
    assert!(validate_rigid_mass_solve_metadata(&[1, 0, 2, 0], reverse_two).is_err());
    assert!(
        pack_rigid_mass_solve_blocks(
            &[MassFactorBlock {
                start: 0,
                size: 2,
                kind: MassFactorKind::Diagonal,
            }],
            2,
        )
        .is_err()
    );
}
