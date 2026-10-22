//! 柔体边被动力三入口包装的宿主故障测试。

use super::kernels::{plan_clear, plan_compute, plan_validate};
use super::launcher_abi::FlexEdgePassiveDims;
use super::launcher_abi::FlexEdgePassiveMetadata;
use super::launcher_abi::FlexEdgePassiveStaticDims;
use super::launcher_abi::flex_edge_passive_lengths;
use super::launcher_abi::validate_flex_edge_passive_metadata;
use super::*;
use crate::runtime::launcher::{
    Checked, DeviceSpan, ElemType, LauncherError, MetadataRevision, SlotBinding,
};

/// 三个自由度、两条柔体边、两个柔体与三条稀疏行。
fn static_dims() -> FlexEdgePassiveStaticDims {
    FlexEdgePassiveStaticDims::new(3, 2, 2, 3, 7).unwrap()
}

/// 四守卫、行偏移、行数、列索引、柔体号与刚体标记。
fn metadata() -> Vec<i32> {
    vec![
        -131072, -131072, -131072, -131072, // 首部守卫
        0, 2, // 行偏移
        2, 1, // 行数
        0, 2, 1, // 列索引
        0, 1, // 柔体号
        0, 1, // 刚体标记
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

fn dims(output_width: u64) -> FlexEdgePassiveDims {
    FlexEdgePassiveDims {
        static_dims: static_dims(),
        worlds: 2,
        abi: [2, 3, 2, 2],
        output_width,
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
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let base = dims(1);
    assert_eq!(flex_edge_passive_lengths(&base), Ok([19, 14, 22, 10]));
    assert_eq!(
        flex_edge_passive_lengths(&FlexEdgePassiveDims {
            output_width: 6,
            ..base
        }),
        Ok([19, 14, 22, 20])
    );
    assert!(flex_edge_passive_lengths(&FlexEdgePassiveDims { worlds: 0, ..base }).is_err());
    assert!(FlexEdgePassiveStaticDims::new(0, 2, 2, 3, 7).is_err());
    assert!(FlexEdgePassiveStaticDims::new(3, 0, 2, 3, 7).is_err());
    assert!(FlexEdgePassiveStaticDims::new(3, 2, 0, 3, 8).is_err());
    assert!(FlexEdgePassiveStaticDims::new(3, 2, 2, 3, 8).is_err());
}

#[test]
fn metadata_proof_rejects_guards_rows_columns_and_ids() {
    let dims = static_dims();
    assert_eq!(
        validate_flex_edge_passive_metadata(&metadata(), dims),
        Ok(())
    );
    for (index, value, reason) in [
        (0, 0, "首部守卫无效"),
        (4, -1, "行偏移为负"),
        (5, 3, "行区间越界"),
        (6, -1, "行数为负"),
        (8, 3, "列索引越界"),
        (11, 2, "柔体号越界"),
        (13, 2, "刚体标记无效"),
        (18, 0, "尾部守卫无效"),
    ] {
        let mut values = metadata();
        values[index] = value;
        assert!(
            validate_flex_edge_passive_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_flex_edge_passive_metadata(&metadata()[..18], dims).is_err());
}

fn proof(
    bindings: &[SlotBinding; 4],
    dims: FlexEdgePassiveDims,
) -> Result<Checked<FlexEdgePassiveMetadata>, TransferError> {
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<FlexEdgePassiveMetadata>::assume_uploaded(
            bindings[0].span,
            &metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .map_err(super::launcher_abi::flex_edge_passive_launch_error)
}

#[test]
fn validate_plan_rejects_capacity_context_and_missing_proof() {
    let dims = dims(1);
    let lengths = flex_edge_passive_lengths(&dims).unwrap();
    let bindings = bindings(lengths);
    let checked = proof(&bindings, dims).unwrap();
    assert!(plan_validate(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[3].span = span(7, 0x9000, 9 * 4);
    assert_eq!(
        plan_validate(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 3,
            required: 40,
            actual: 36,
        })
    );
    assert_eq!(
        plan_validate(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    );
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x3000, lengths[2] * 4);
    assert!(matches!(
        plan_validate(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}

#[test]
fn clear_and_compute_plans_accept_wide_outputs_and_reject_capacity_and_alias() {
    let dims = dims(6);
    let lengths = flex_edge_passive_lengths(&dims).unwrap();
    let bindings = bindings(lengths);
    let checked = proof(&bindings, dims).unwrap();
    assert!(plan_clear(&dims, &bindings, &[checked.binding(0)]).is_ok());
    assert!(plan_compute(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[2].span = span(7, 0x3000, (lengths[2] - 1) * 4);
    assert_eq!(
        plan_compute(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 2,
            required: 88,
            actual: 84,
        })
    );
    let mut foreign = bindings;
    foreign[0].span = span(8, 0x1000, lengths[0] * 4);
    assert!(matches!(
        plan_compute(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
    let mut alias = bindings;
    alias[3].span = span(7, 0x3000, lengths[3] * 4);
    assert_eq!(
        plan_compute(&dims, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias {
            first: 2,
            second: 3
        })
    );
}
