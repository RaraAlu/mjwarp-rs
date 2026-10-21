//! 刚体运动学包装的宿主规则测试。
//! 不需要GPU：只覆盖长度、证明与描述符。

use super::RigidVariant;
use super::metadata::rigid_lengths;
use super::metadata::{
    RigidDims, RigidStaticDims, metadata_elements, probe_parameters, validate_metadata,
};

fn probe_dims(nq: u32, nb: u32, nj: u32) -> RigidStaticDims {
    RigidStaticDims {
        variant: RigidVariant::Probe,
        nq,
        nb,
        nj,
        nmocap: 0,
        metadata_elements: metadata_elements(RigidVariant::Probe, nb, nj).unwrap(),
        parameter_elements: probe_parameters(nq, nb, nj),
    }
}

fn dims(static_dims: RigidStaticDims, worlds: u32) -> RigidDims {
    RigidDims {
        static_dims,
        abi: [static_dims.nq, static_dims.nb, static_dims.nj, worlds],
    }
}

// 两体一铰链：根体0，体1父体0，关节0从属体1、qpos地址0。
const PROBE: [i32; 8] = [0, 0, -1, 0, 0, 1, 3, 0];

#[test]
fn accepts_ordered_tree_and_zero_dof_state() {
    let static_dims = probe_dims(1, 2, 1);
    validate_metadata(&PROBE, &static_dims).unwrap();
    assert_eq!(
        rigid_lengths(&dims(static_dims, 2)).unwrap(),
        [8, 35, 2, 132]
    );
    let zero = probe_dims(0, 2, 0);
    validate_metadata(&[0, 0, -1, -1, 0, 0], &zero).unwrap();
    assert_eq!(rigid_lengths(&dims(zero, 3)).unwrap(), [6, 28, 1, 176]);
    assert!(rigid_lengths(&dims(zero, 0)).is_err());
    assert!(rigid_lengths(&dims(zero, u32::MAX)).is_err());
    assert!(metadata_elements(RigidVariant::Probe, 0, 0).is_err());
    assert!(metadata_elements(RigidVariant::Batch, u32::MAX, 0).is_err());
}

#[test]
fn rejects_bad_parent_joint_types_and_qpos_ranges() {
    let static_dims = probe_dims(1, 2, 1);
    for edited in [
        [1, 0, -1, 0, 0, 1, 3, 0],
        [0, 2, -1, 0, 0, 1, 3, 0],
        [0, 0, 0, 0, 0, 1, 3, 0],
        [0, 0, -1, 1, 0, 1, 3, 0],
        [0, 0, -1, 0, 2, 1, 3, 0],
        [0, 0, -1, 0, 0, 2, 3, 0],
        [0, 0, -1, 0, 0, 1, 4, 0],
        [0, 0, -1, 0, 0, 1, 3, 1],
    ] {
        assert!(validate_metadata(&edited, &static_dims).is_err());
    }
}

#[test]
fn batch_variant_checks_mocap_and_descriptors() {
    // 两体一铰链，nmocap=1，映射[-1,0]，七组描述符。
    let mut values = vec![0, 0, -1, 0, 0, 1, 3, 0, 1, -1, 0];
    let widths = [1u64, 6, 8, 6, 8, 3, 3];
    let periods = [1u32, 2, 1, 1, 1, 1, 1];
    let mut offset = 0u32;
    for (&width, &period) in widths.iter().zip(&periods) {
        values.extend([offset as i32, period as i32]);
        offset += (width * u64::from(period)) as u32;
    }
    let static_dims = RigidStaticDims {
        variant: RigidVariant::Batch,
        nq: 1,
        nb: 2,
        nj: 1,
        nmocap: 1,
        metadata_elements: values.len() as u64,
        parameter_elements: u64::from(offset),
    };
    validate_metadata(&values, &static_dims).unwrap();
    assert_eq!(
        rigid_lengths(&dims(static_dims, 4)).unwrap(),
        [25, 41, 32, 256]
    );
    // G01等价变体与严格批量变体共用布局与证明。
    let mut equivalent = static_dims;
    equivalent.variant = RigidVariant::Equivalent;
    validate_metadata(&values, &equivalent).unwrap();
    assert_eq!(
        rigid_lengths(&dims(equivalent, 4)).unwrap(),
        [25, 41, 32, 256]
    );
    // mocap数量、映射、描述符偏移、周期与总长均须拒绝。
    for (index, value) in [(8, 2), (9, 1), (9, -2), (11, 1), (12, 5), (24, 2)] {
        let mut edited = values.clone();
        edited[index] = value;
        assert!(validate_metadata(&edited, &static_dims).is_err());
    }
}
