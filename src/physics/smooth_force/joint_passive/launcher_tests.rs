//! 关节被动力三入口包装的宿主故障测试。

use super::kernels::{plan_compute, plan_sum, plan_validate};
use super::launcher_abi::JointPassiveDims;
use super::launcher_abi::JointPassiveEntry;
use super::launcher_abi::JointPassiveMetadata;
use super::launcher_abi::JointPassiveStaticDims;
use super::launcher_abi::joint_passive_lengths;
use super::launcher_abi::validate_joint_passive_metadata;
use super::*;
use crate::runtime::launcher::{
    Checked, DeviceSpan, ElemType, LauncherError, MetadataRevision, SlotBinding,
};

/// 六个广义坐标、四个自由度、三个关节与二十个参数元素。
fn static_dims() -> JointPassiveStaticDims {
    JointPassiveStaticDims::new(6, 4, 3, 20).unwrap()
}

/// 四守卫、逐自由度关节种类、qpos 偏移、关节号与局部自由度。
fn metadata() -> Vec<i32> {
    vec![
        -131072, -131072, -131072, -131072, // 首部守卫
        2, 3, 2, 3, // 关节种类
        0, 1, 2, 3, // qpos 偏移
        0, 1, 2, 0, // 关节号
        0, 0, 0, 0, // 局部自由度
        -131072, -131072, -131072, -131072, // 尾部守卫
    ]
}

fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).unwrap()
}

fn binding(slot: usize, context: u64, addr: u64, length: u64, elem: ElemType) -> SlotBinding {
    SlotBinding {
        slot,
        span: span(context, addr, length * 4),
        elem,
    }
}

fn dims(entry: JointPassiveEntry) -> JointPassiveDims {
    JointPassiveDims {
        static_dims: static_dims(),
        worlds: 2,
        abi: [6, 4, 2, 8],
        entry,
    }
}

#[test]
fn lengths_follow_entry_layout_and_reject_invalid_dimensions() {
    assert_eq!(
        joint_passive_lengths(&dims(JointPassiveEntry::Validate)),
        Ok([24, 28, 28, 10])
    );
    assert_eq!(
        joint_passive_lengths(&dims(JointPassiveEntry::Compute)),
        Ok([24, 28, 28, 24])
    );
    assert_eq!(
        joint_passive_lengths(&dims(JointPassiveEntry::Sum)),
        Ok([24, 24, 1, 40])
    );
    let base = dims(JointPassiveEntry::Validate);
    assert!(joint_passive_lengths(&JointPassiveDims { worlds: 0, ..base }).is_err());
    assert!(JointPassiveStaticDims::new(0, 4, 3, 20).is_err());
    assert!(JointPassiveStaticDims::new(6, 0, 3, 20).is_err());
    assert!(JointPassiveStaticDims::new(6, 4, 0, 20).is_err());
    assert!(JointPassiveStaticDims::new(6, 4, 3, 0).is_ok());
}

#[test]
fn metadata_proof_rejects_guards_kinds_offsets_and_local_dofs() {
    let dims = static_dims();
    assert_eq!(validate_joint_passive_metadata(&metadata(), dims), Ok(()));
    for (index, value, reason) in [
        (0, 0, "首部守卫无效"),
        (4, 4, "关节种类无效"),
        (8, 6, "qpos区间越界"),
        (12, 3, "关节号越界"),
        (16, 1, "局部自由度越界"),
        (23, 0, "尾部守卫无效"),
    ] {
        let mut values = metadata();
        values[index] = value;
        assert!(
            validate_joint_passive_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    // 自由关节需要七个qpos槽：偏移3将越过边界。
    let mut wide = metadata();
    wide[4] = 0;
    wide[8] = 3;
    assert!(validate_joint_passive_metadata(&wide, dims).is_err());
    assert!(validate_joint_passive_metadata(&metadata()[..23], dims).is_err());
}

fn proof(
    bindings: &[SlotBinding; 4],
    dims: JointPassiveDims,
) -> Result<Checked<JointPassiveMetadata>, TransferError> {
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<JointPassiveMetadata>::assume_uploaded(
            bindings[0].span,
            &metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .map_err(super::launcher_abi::joint_passive_launch_error)
}

fn bindings(lengths: [u64; 4]) -> [SlotBinding; 4] {
    [
        binding(0, 7, 0x1000, lengths[0], ElemType::I32),
        binding(1, 7, 0x2000, lengths[1], ElemType::F32),
        binding(2, 7, 0x3000, lengths[2], ElemType::F32),
        binding(3, 7, 0x9000, lengths[3], ElemType::F32),
    ]
}

#[test]
fn validate_and_compute_plans_reject_capacity_context_and_missing_proof() {
    let validate = dims(JointPassiveEntry::Validate);
    let lengths = joint_passive_lengths(&validate).unwrap();
    let bounds = bindings(lengths);
    let checked = proof(&bounds, validate).unwrap();
    assert!(plan_validate(&validate, &bounds, &[checked.binding(0)]).is_ok());
    let mut short = bounds;
    short[3].span = span(7, 0x9000, 9 * 4);
    assert_eq!(
        plan_validate(&validate, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 3,
            required: 40,
            actual: 36,
        })
    );
    assert_eq!(
        plan_validate(&validate, &bounds, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    );
    let mut foreign = bounds;
    foreign[2].span = span(8, 0x3000, lengths[2] * 4);
    assert!(matches!(
        plan_validate(&validate, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
    let compute = dims(JointPassiveEntry::Compute);
    let compute_lengths = joint_passive_lengths(&compute).unwrap();
    let compute_bounds = bindings(compute_lengths);
    let compute_proof = proof(&compute_bounds, compute).unwrap();
    assert!(plan_compute(&compute, &compute_bounds, &[compute_proof.binding(0)]).is_ok());
    let mut alias = compute_bounds;
    alias[3].span = span(7, 0x3000, compute_lengths[3] * 4);
    assert_eq!(
        plan_compute(&compute, &alias, &[compute_proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 2,
            second: 3
        })
    );
}

#[test]
fn sum_plan_accepts_shared_parts_pointer_and_owned_force_width() {
    let sum = dims(JointPassiveEntry::Sum);
    let lengths = joint_passive_lengths(&sum).unwrap();
    // 槽位2未读：只要求非空，不参与精确跨度校对。
    let bounds = bindings([lengths[0], lengths[1], 40, lengths[3]]);
    let checked = proof(&bounds, sum).unwrap();
    // 未读指针槽只要求非空：与读数分槽共用同一段区是合法读读重叠。
    let mut aliased = bounds;
    aliased[2].span = bounds[1].span;
    assert_eq!(plan_sum(&sum, &aliased, &[checked.binding(0)]), Ok(()));
    assert!(plan_sum(&sum, &bounds, &[checked.binding(0)]).is_ok());
    let mut short = bounds;
    short[3].span = span(7, 0x9000, 39 * 4);
    assert_eq!(
        plan_sum(&sum, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 3,
            required: 160,
            actual: 156,
        })
    );
    let mut empty = bounds;
    empty[2].span = span(7, 0x3000, 3);
    assert_eq!(
        plan_sum(&sum, &empty, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 2,
            required: 4,
            actual: 3,
        })
    );
}
