//! 共享被动力四入口包装的宿主故障测试。

use super::kernels::{plan_add_fluid, plan_clear, plan_clear_gravity, plan_sum};
use super::launcher_abi::PassiveForceDims;
use super::launcher_abi::PassiveForceEntry;
use super::launcher_abi::PassiveForceMetadata;
use super::launcher_abi::PassiveForceStaticDims;
use super::launcher_abi::passive_force_lengths;
use super::launcher_abi::validate_passive_force_metadata;
use super::*;
use crate::runtime::launcher::{
    Checked, DeviceSpan, ElemType, LauncherError, MetadataRevision, SlotBinding,
};

fn static_dims() -> PassiveForceStaticDims {
    PassiveForceStaticDims::new(3).unwrap()
}

fn dims(entry: PassiveForceEntry) -> PassiveForceDims {
    PassiveForceDims {
        static_dims: static_dims(),
        worlds: 2,
        abi: [0, 3, 2, 6],
        entry,
    }
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

fn bindings(lengths: [u64; 4]) -> [SlotBinding; 4] {
    [
        binding(0, 7, 0x1000, lengths[0], ElemType::I32),
        binding(1, 7, 0x2000, lengths[1], ElemType::F32),
        binding(2, 7, 0x3000, lengths[2], ElemType::F32),
        binding(3, 7, 0x9000, lengths[3], ElemType::F32),
    ]
}

#[test]
fn lengths_follow_entry_layout_and_reject_invalid_dimensions() {
    assert_eq!(
        passive_force_lengths(&dims(PassiveForceEntry::Clear)),
        Ok([8, 8, 8, 20])
    );
    assert_eq!(
        passive_force_lengths(&dims(PassiveForceEntry::Sum)),
        Ok([8, 8, 20, 14])
    );
    assert_eq!(
        passive_force_lengths(&dims(PassiveForceEntry::ClearGravity)),
        Ok([8, 8, 8, 14])
    );
    assert_eq!(
        passive_force_lengths(&dims(PassiveForceEntry::AddFluid)),
        Ok([8, 8, 14, 14])
    );
    let base = dims(PassiveForceEntry::Clear);
    assert!(passive_force_lengths(&PassiveForceDims { worlds: 0, ..base }).is_err());
    assert!(
        passive_force_lengths(&PassiveForceDims {
            static_dims: PassiveForceStaticDims { nv: 0 },
            ..base
        })
        .is_err()
    );
    assert!(PassiveForceStaticDims::new(0).is_err());
}

#[test]
fn metadata_proof_requires_the_frozen_eight_guards() {
    let dims = static_dims();
    assert_eq!(validate_passive_force_metadata(&[-131072; 8], dims), Ok(()));
    for index in [0, 3, 4, 7] {
        let mut values = [-131072; 8];
        values[index] = 0;
        assert!(
            validate_passive_force_metadata(&values, dims).is_err(),
            "守卫 {index} 应被拒绝"
        );
    }
    assert!(validate_passive_force_metadata(&[-131072; 7], dims).is_err());
}

fn proof(
    bindings: &[SlotBinding; 4],
    dims: PassiveForceDims,
) -> Result<Checked<PassiveForceMetadata>, TransferError> {
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<PassiveForceMetadata>::assume_uploaded(
            bindings[0].span,
            &[-131072; 8],
            dims,
            MetadataRevision::new(),
        )
    }
    .map_err(super::launcher_abi::passive_force_launch_error)
}

#[test]
fn clear_plan_rejects_capacity_context_alias_and_missing_proof() {
    let clear = dims(PassiveForceEntry::Clear);
    let lengths = passive_force_lengths(&clear).unwrap();
    let bounds = bindings(lengths);
    let checked = proof(&bounds, clear).unwrap();
    assert!(plan_clear(&clear, &bounds, &[checked.binding(0)]).is_ok());
    let mut short = bounds;
    short[3].span = span(7, 0x9000, 19 * 4);
    assert_eq!(
        plan_clear(&clear, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 3,
            required: 80,
            actual: 76,
        })
    );
    assert_eq!(
        plan_clear(&clear, &bounds, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    );
    let mut foreign = bounds;
    foreign[2].span = span(8, 0x3000, lengths[2] * 4);
    assert!(matches!(
        plan_clear(&clear, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
    let mut alias = bounds;
    alias[3].span = span(7, 0x2000, lengths[3] * 4);
    assert_eq!(
        plan_clear(&clear, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    );
}

#[test]
fn sum_and_fluid_accept_wide_smooth_forces_and_reject_narrow_slots() {
    let sum = dims(PassiveForceEntry::Sum);
    let lengths = passive_force_lengths(&sum).unwrap();
    let mut bounds = bindings(lengths);
    // 平滑力缓冲为四字段；内核只写首个字段。
    bounds[3].span = span(7, 0x9000, (8 + 4 * 2 * 3) * 4);
    let checked = proof(&bounds, sum).unwrap();
    assert!(plan_sum(&sum, &bounds, &[checked.binding(0)]).is_ok());
    let mut short_parts = bounds;
    short_parts[2].span = span(7, 0x3000, 19 * 4);
    assert_eq!(
        plan_sum(&sum, &short_parts, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 2,
            required: 80,
            actual: 76,
        })
    );
    let mut narrow_forces = bounds;
    narrow_forces[3].span = span(7, 0x9000, 13 * 4);
    assert_eq!(
        plan_sum(&sum, &narrow_forces, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 3,
            required: 56,
            actual: 52,
        })
    );
    let fluid = dims(PassiveForceEntry::AddFluid);
    let fluid_lengths = passive_force_lengths(&fluid).unwrap();
    let mut fluid_bounds = bindings(fluid_lengths);
    fluid_bounds[3].span = span(7, 0x9000, (8 + 4 * 2 * 3) * 4);
    let fluid_proof = proof(&fluid_bounds, fluid).unwrap();
    assert!(plan_add_fluid(&fluid, &fluid_bounds, &[fluid_proof.binding(0)]).is_ok());
    let gravity = dims(PassiveForceEntry::ClearGravity);
    let gravity_lengths = passive_force_lengths(&gravity).unwrap();
    let gravity_bounds = bindings(gravity_lengths);
    let gravity_proof = proof(&gravity_bounds, gravity).unwrap();
    assert!(plan_clear_gravity(&gravity, &gravity_bounds, &[gravity_proof.binding(0)]).is_ok());
}
