//! 肌腱被动力三入口包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::kernels::plan_clear;
#[cfg(feature = "cuda-probe")]
use super::kernels::plan_compute;
#[cfg(feature = "cuda-probe")]
use super::kernels::plan_validate;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonPassiveDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonPassiveMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonPassiveStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::tendon_passive_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::validate_tendon_passive_metadata;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Checked, CheckedBinding, DeviceSpan, ElemType, LauncherError, MetadataRevision, SlotBinding,
};

#[cfg(feature = "cuda-probe")]
fn static_dims() -> TendonPassiveStaticDims {
    TendonPassiveStaticDims::new(2, 3, 3, 5, 10, 2).unwrap()
}

/// 四守卫、行偏移、行数与列索引。
#[cfg(feature = "cuda-probe")]
fn metadata() -> Vec<i32> {
    vec![
        -131072, -131072, -131072, -131072, // 首部守卫
        0, 2, // 行偏移
        1, 1, // 行数
        0, 1, 2, // 列索引
        -131072, -131072, -131072, -131072, // 尾部守卫
    ]
}

#[cfg(feature = "cuda-probe")]
fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).unwrap()
}

#[cfg(feature = "cuda-probe")]
fn binding(slot: usize, context: u64, addr: u64, length: u64, elem: ElemType) -> SlotBinding {
    SlotBinding {
        slot,
        span: span(context, addr, length * 4),
        elem,
    }
}

#[cfg(feature = "cuda-probe")]
fn dims(output_width: u64) -> TendonPassiveDims {
    TendonPassiveDims {
        static_dims: static_dims(),
        worlds: 2,
        abi: [2, 3, 2, 2],
        output_width,
    }
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = dims(1);
    assert_eq!(tendon_passive_lengths(&dims).unwrap(), [15, 18, 18, 12, 10]);
    let compute = dims_with_width(6);
    assert_eq!(
        tendon_passive_lengths(&compute).unwrap(),
        [15, 18, 18, 12, 20]
    );
    assert!(tendon_passive_lengths(&TendonPassiveDims { worlds: 0, ..dims }).is_err());
    assert!(TendonPassiveStaticDims::new(0, 3, 3, 5, 10, 2).is_err());
    assert!(TendonPassiveStaticDims::new(2, 0, 3, 5, 10, 2).is_err());
}

#[cfg(feature = "cuda-probe")]
fn dims_with_width(width: u64) -> TendonPassiveDims {
    TendonPassiveDims {
        output_width: width,
        ..dims(1)
    }
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_guards_rows_and_columns() {
    let dims = static_dims();
    assert!(validate_tendon_passive_metadata(&metadata(), dims).is_ok());
    for (index, value, reason) in [
        (0, 0, "首部守卫无效"),
        (4, -1, "行偏移为负"),
        (5, 3, "行区间越界"),
        (6, -1, "行数为负"),
        (8, 3, "列索引越界"),
        (13, 0, "尾部守卫无效"),
    ] {
        let mut values = metadata();
        values[index] = value;
        assert!(
            validate_tendon_passive_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_tendon_passive_metadata(&metadata()[..14], dims).is_err());
}

#[cfg(feature = "cuda-probe")]
fn proof(
    bindings: &[SlotBinding; 5],
    dims: TendonPassiveDims,
) -> Result<Checked<TendonPassiveMetadata>, TransferError> {
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<TendonPassiveMetadata>::assume_uploaded(
            bindings[0].span,
            &metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .map_err(super::launcher_abi::tendon_passive_launch_error)
}

#[test]
#[cfg(feature = "cuda-probe")]
fn validate_plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = dims(1);
    let lengths = tendon_passive_lengths(&dims).unwrap();
    let bindings = [
        binding(0, 7, 0x1000, lengths[0], ElemType::I32),
        binding(1, 7, 0x2000, lengths[1], ElemType::F32),
        binding(2, 7, 0x3000, lengths[2], ElemType::F32),
        binding(3, 7, 0x4000, lengths[3], ElemType::F32),
        binding(4, 7, 0x9000, lengths[4], ElemType::F32),
    ];
    let checked = proof(&bindings, dims).unwrap();
    assert!(plan_validate(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[4].span = span(7, 0x9000, 9 * 4);
    assert!(matches!(
        plan_validate(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 4, .. })
    ));
    assert!(matches!(
        plan_validate(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 输出槽跨度小于其余输入槽，精确跨度检查先于别名；别名机制由其它入口覆盖。
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x3000, lengths[2] * 4);
    assert!(matches!(
        plan_validate(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn clear_and_compute_plans_accept_wide_outputs_and_reject_capacity() {
    let dims = dims_with_width(6);
    let lengths = tendon_passive_lengths(&dims).unwrap();
    let bindings = [
        binding(0, 7, 0x1000, lengths[0], ElemType::I32),
        binding(1, 7, 0x2000, lengths[1], ElemType::F32),
        binding(2, 7, 0x3000, lengths[2], ElemType::F32),
        binding(3, 7, 0x4000, lengths[3], ElemType::F32),
        binding(4, 7, 0x9000, lengths[4], ElemType::F32),
    ];
    let checked = proof(&bindings, dims).unwrap();
    assert!(plan_clear(&dims, &bindings, &[checked.binding(0)]).is_ok());
    assert!(plan_compute(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[3].span = span(7, 0x4000, (lengths[3] - 1) * 4);
    assert!(matches!(
        plan_compute(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    let mut foreign = bindings;
    foreign[0].span = span(8, 0x1000, lengths[0] * 4);
    assert!(matches!(
        plan_compute(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
#[cfg(feature = "cuda-probe")]
#[allow(dead_code)]
fn _unused(_: CheckedBinding<'static, TendonPassiveDims>) {}
