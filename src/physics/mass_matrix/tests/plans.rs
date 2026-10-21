//! 刚体质量矩阵冻结ABI与发射计划测试。

use super::rigid::{
    RigidMassMatrixDims, RigidMassMatrixMetadata, rigid_mass_matrix_lengths,
    rigid_mass_matrix_plan, validate_rigid_mass_matrix_metadata,
};
use crate::runtime::launcher::{Checked, ElemType, LauncherError, MetadataRevision, SlotBinding};

#[test]
fn rigid_mass_matrix_plan_checks_spans_aliases_topology_and_overflow() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, address, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, address, bytes).unwrap(),
        elem,
    };
    let values = vec![0, 0, 1, 1, 2, -1, 0];
    let dims = RigidMassMatrixDims { abi: [3, 2, 0, 2] };
    assert_eq!(rigid_mass_matrix_lengths(&dims).unwrap(), [7, 2, 116, 76]);
    let revision = MetadataRevision::new();
    let metadata_span = DeviceSpan::new(7, 0x1000, 28).unwrap();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<RigidMassMatrixMetadata>::assume_uploaded(
            metadata_span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        bind(0, 7, 0x1000, 28, ElemType::I32),
        bind(1, 7, 0x2000, 8, ElemType::F32),
        bind(2, 7, 0x3000, 464, ElemType::F32),
        bind(3, 7, 0x4000, 304, ElemType::F32),
    ];
    let launch = rigid_mass_matrix_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(matches!(
        rigid_mass_matrix_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let addresses = [0x1000, 0x2000, 0x3000, 0x4000];
    let bytes = [28, 8, 464, 304];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] - 4).unwrap();
        assert!(matches!(
            rigid_mass_matrix_plan(&dims, &short, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(matches!(
            rigid_mass_matrix_plan(&dims, &long, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
        ));
    }

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x3000, 464).unwrap();
    assert!(matches!(
        rigid_mass_matrix_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, 0x3000, 304).unwrap();
    assert!(matches!(
        rigid_mass_matrix_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 2,
            second: 3
        })
    ));
    // 只读槽之间允许重叠。
    let mut shared = bindings;
    shared[1].span = DeviceSpan::new(7, 0x3000, 8).unwrap();
    assert!(rigid_mass_matrix_plan(&dims, &shared, &[proof.binding(0)]).is_ok());
    let other = RigidMassMatrixDims { abi: [3, 2, 0, 1] };
    assert!(matches!(
        rigid_mass_matrix_plan(
            &other,
            &[
                bindings[0],
                bindings[1],
                bind(2, 7, 0x3000, 248, ElemType::F32),
                bind(3, 7, 0x4000, 168, ElemType::F32),
            ],
            &[proof.binding(0)],
        ),
        Err(LauncherError::MetadataMismatch(_))
    ));
    revision.invalidate();
    assert!(matches!(
        rigid_mass_matrix_plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));

    for invalid in [
        vec![0, 0, 1, 1, 2, -1],
        vec![0, 0, 1, 1, 2, -1, 0, 0],
        vec![1, 0, 1, 1, 2, -1, 0],
        vec![0, -1, 1, 1, 2, -1, 0],
        vec![0, 0, 2, 1, 2, -1, 0],
        vec![0, 0, 1, 3, 2, -1, 0],
        vec![0, 0, 1, -1, 2, -1, 0],
        vec![0, 0, 1, 1, 2, 0, 0],
        vec![0, 0, 1, 1, 2, -1, 1],
        vec![0, 0, 1, 1, 2, -1, -2],
    ] {
        assert!(validate_rigid_mass_matrix_metadata(&invalid, dims).is_err());
    }
    assert!(validate_rigid_mass_matrix_metadata(&[0, 0, 1, 1, 2, -1, -1], dims).is_ok());
    let zero = RigidMassMatrixDims { abi: [2, 0, 0, 2] };
    assert_eq!(rigid_mass_matrix_lengths(&zero).unwrap(), [2, 1, 64, 48]);
    assert!(validate_rigid_mass_matrix_metadata(&[0, 0], zero).is_ok());

    for abi in [
        [0, 0, 0, 1],
        [1, 0, 1, 1],
        [1, 0, 0, 0],
        [1, 0, 0, u32::MAX - 254],
    ] {
        assert!(matches!(
            rigid_mass_matrix_lengths(&RigidMassMatrixDims { abi }),
            Err(LauncherError::InvalidArray(_))
        ));
    }
    assert!(
        rigid_mass_matrix_lengths(&RigidMassMatrixDims {
            abi: [1, 0, 0, u32::MAX - 255]
        })
        .is_ok()
    );
}

#[test]
fn rigid_mass_matrix_plan_preserves_u32_strides_and_byte_bounds() {
    let state_limit = RigidMassMatrixDims {
        abi: [i32::MAX as u32 / 14, 0, 0, 1],
    };
    assert!(rigid_mass_matrix_lengths(&state_limit).is_ok());
    assert!(matches!(
        rigid_mass_matrix_lengths(&RigidMassMatrixDims {
            abi: [i32::MAX as u32 / 14 + 1, 0, 0, 1],
        }),
        Err(LauncherError::Overflow("rigid_mass_matrix_state_width"))
    ));
    assert!(
        rigid_mass_matrix_lengths(&RigidMassMatrixDims {
            abi: [1, 46_340, 0, 1]
        })
        .is_ok()
    );
    assert!(matches!(
        rigid_mass_matrix_lengths(&RigidMassMatrixDims {
            abi: [1, 46_341, 0, 1],
        }),
        Err(LauncherError::Overflow("rigid_mass_matrix_output_width"))
    ));
    assert!(matches!(
        rigid_mass_matrix_lengths(&RigidMassMatrixDims {
            abi: [1, i32::MAX as u32, 0, 1],
        }),
        Err(LauncherError::Overflow("rigid_mass_matrix_metadata"))
    ));

    let byte_overflow = RigidMassMatrixDims {
        abi: [i32::MAX as u32 / 14, 0, 0, u32::MAX - 255],
    };
    assert!(rigid_mass_matrix_lengths(&byte_overflow).is_ok());
    let bind = |slot, address, bytes, elem| SlotBinding {
        slot,
        span: crate::runtime::launcher::DeviceSpan::new(7, address, bytes).unwrap(),
        elem,
    };
    // 前两槽精确匹配，state字节数溢出u64。
    assert!(matches!(
        rigid_mass_matrix_plan(
            &byte_overflow,
            &[
                bind(0, 0x1000, 4 * (i32::MAX as u64 / 14), ElemType::I32),
                bind(1, 0x4000_0000, 4, ElemType::F32),
                bind(2, 0x5000_0000, 4, ElemType::F32),
                bind(3, 0x6000_0000, 4, ElemType::F32),
            ],
            &[],
        ),
        Err(LauncherError::Overflow("rigid_mass_matrix_bytes"))
    ));
}
