//! 流体被动力四入口包装的宿主故障测试。

use super::kernels::{plan_body, plan_clear, plan_project, plan_validate};
use super::launcher_abi::FluidForceDims;
use super::launcher_abi::FluidForceEntry;
use super::launcher_abi::FluidForceMetadata;
use super::launcher_abi::FluidForceStaticDims;
use super::launcher_abi::fluid_force_lengths;
use super::launcher_abi::validate_fluid_force_metadata;
use super::*;
use crate::runtime::launcher::{
    Checked, DeviceSpan, ElemType, LauncherError, MetadataRevision, SlotBinding,
};

/// 三个刚体、一个关节、两个自由度、两个几何与一个 site。
fn static_dims(attached: bool) -> FluidForceStaticDims {
    FluidForceStaticDims::new(3, 1, 2, 2, 1, 20, attached).unwrap()
}

/// 四守卫、父表、根表、自由度体、几何体、几何类型、椭球标记与四守卫。
fn metadata() -> Vec<i32> {
    vec![
        -131072, -131072, -131072, -131072, // 首部守卫
        0, 0, 1, // 父表
        0, 1, 1, // 根表
        1, 2, // 自由度体
        2, 1, // 几何体
        0, 2, // 几何类型
        0, 1, 0, // 椭球标记
        -131072, -131072, -131072, -131072, // 尾部守卫
    ]
}

fn dims(entry: FluidForceEntry, attached: bool) -> FluidForceDims {
    FluidForceDims {
        static_dims: static_dims(attached),
        worlds: 2,
        abi: [3, 2, 2, 4],
        entry,
        clear_work: 44,
    }
}

fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).unwrap()
}

fn binding(slot: usize, addr: u64, length: u64, elem: ElemType) -> SlotBinding {
    SlotBinding {
        slot,
        span: span(7, addr, length * 4),
        elem,
    }
}

fn bindings(lengths: [u64; 7]) -> [SlotBinding; 7] {
    [
        binding(0, 0x1000, lengths[0], ElemType::I32),
        binding(1, 0x2000, lengths[1], ElemType::F32),
        binding(2, 0x3000, lengths[2], ElemType::F32),
        binding(3, 0x4000, lengths[3], ElemType::F32),
        binding(4, 0x5000, lengths[4], ElemType::F32),
        binding(5, 0x6000, lengths[5], ElemType::F32),
        binding(6, 0x9000, lengths[6], ElemType::F32),
    ]
}

#[test]
fn lengths_follow_entry_layout_and_reject_invalid_dimensions() {
    assert_eq!(
        fluid_force_lengths(&dims(FluidForceEntry::Validate, false)),
        Ok([23, 28, 188, 116, 68, 28, 10])
    );
    assert_eq!(
        fluid_force_lengths(&dims(FluidForceEntry::Body, false)),
        Ok([23, 28, 188, 116, 68, 28, 44])
    );
    assert_eq!(
        fluid_force_lengths(&dims(FluidForceEntry::Project, false)),
        Ok([23, 28, 188, 116, 44, 28, 12])
    );
    assert_eq!(
        fluid_force_lengths(&dims(FluidForceEntry::Clear, false)),
        Ok([23, 28, 28, 28, 28, 28, 52])
    );
    // 椭球体存在时第五槽是附着位姿：每世界十二项帧。
    assert_eq!(
        fluid_force_lengths(&dims(FluidForceEntry::Body, true)),
        Ok([23, 28, 188, 116, 68, 80, 44])
    );
    let base = dims(FluidForceEntry::Validate, false);
    assert!(fluid_force_lengths(&FluidForceDims { worlds: 0, ..base }).is_err());
    assert!(FluidForceStaticDims::new(1, 1, 2, 2, 1, 20, false).is_err());
    assert!(FluidForceStaticDims::new(3, 1, 0, 2, 1, 20, false).is_err());
}

#[test]
fn metadata_proof_rejects_guards_parents_roots_and_flags() {
    let dims = static_dims(false);
    assert_eq!(validate_fluid_force_metadata(&metadata(), dims), Ok(()));
    for (index, value, reason) in [
        (0, 0, "首部守卫无效"),
        (4, 1, "根体父引用无效"),
        (6, 2, "体父链无效"),
        (7, 2, "根表无效"),
        (9, 0, "根表值与父链不符"),
        (10, 3, "自由度体越界"),
        (12, 3, "几何体越界"),
        (14, -1, "几何类型为负"),
        (16, 2, "椭球标记无效"),
        (22, 0, "尾部守卫无效"),
    ] {
        let mut values = metadata();
        values[index] = value;
        assert!(
            validate_fluid_force_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_fluid_force_metadata(&metadata()[..22], dims).is_err());
}

fn proof(
    bindings: &[SlotBinding; 7],
    dims: FluidForceDims,
) -> Result<Checked<FluidForceMetadata>, TransferError> {
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<FluidForceMetadata>::assume_uploaded(
            bindings[0].span,
            &metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .map_err(super::launcher_abi::fluid_force_launch_error)
}

#[test]
fn validate_and_body_plans_reject_capacity_context_and_missing_proof() {
    let validate = dims(FluidForceEntry::Validate, false);
    let lengths = fluid_force_lengths(&validate).unwrap();
    let bounds = bindings(lengths);
    let checked = proof(&bounds, validate).unwrap();
    assert!(plan_validate(&validate, &bounds, &[checked.binding(0)]).is_ok());
    let mut short = bounds;
    short[6].span = span(7, 0x9000, 9 * 4);
    assert_eq!(
        plan_validate(&validate, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 6,
            required: 40,
            actual: 36,
        })
    );
    assert_eq!(
        plan_validate(&validate, &bounds, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    );
    let mut foreign = bounds;
    foreign[4].span = span(8, 0x5000, lengths[4] * 4);
    assert!(matches!(
        plan_validate(&validate, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
    let body = dims(FluidForceEntry::Body, true);
    let body_lengths = fluid_force_lengths(&body).unwrap();
    let body_bounds = bindings(body_lengths);
    let body_proof = proof(&body_bounds, body).unwrap();
    assert!(plan_body(&body, &body_bounds, &[body_proof.binding(0)]).is_ok());
    let mut short_velocity = body_bounds;
    short_velocity[4].span = span(7, 0x5000, (body_lengths[4] - 1) * 4);
    assert!(matches!(
        plan_body(&body, &short_velocity, &[body_proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 4, .. })
    ));
}

#[test]
fn project_and_clear_plans_follow_body_and_clear_widths() {
    let project = dims(FluidForceEntry::Project, false);
    let lengths = fluid_force_lengths(&project).unwrap();
    let bounds = bindings(lengths);
    let checked = proof(&bounds, project).unwrap();
    assert!(plan_project(&project, &bounds, &[checked.binding(0)]).is_ok());
    // 投影入口第四槽是刚体空间力，不是速度。
    let mut narrow = bounds;
    narrow[4].span = span(7, 0x5000, 67 * 4);
    assert_eq!(
        plan_project(&project, &narrow, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 4,
            required: 176,
            actual: 268,
        })
    );
    let clear = dims(FluidForceEntry::Clear, false);
    let clear_lengths = fluid_force_lengths(&clear).unwrap();
    let clear_bounds = bindings(clear_lengths);
    let clear_proof = proof(&clear_bounds, clear).unwrap();
    assert!(plan_clear(&clear, &clear_bounds, &[clear_proof.binding(0)]).is_ok());
    let mut wide = clear_bounds;
    wide[6].span = span(7, 0x9000, 53 * 4);
    assert_eq!(
        plan_clear(&clear, &wide, &[clear_proof.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 6,
            required: 208,
            actual: 212,
        })
    );
}
