//! 常驻稀疏求解两入口包装的宿主故障测试。

use super::kernels::{plan_fused, plan_validate};
use super::launcher_abi::SparseSolveDims;
use super::launcher_abi::SparseSolveEntry;
use super::launcher_abi::SparseSolveMetadata;
use super::launcher_abi::SparseSolveStaticDims;
use super::launcher_abi::sparse_solve_lengths;
use super::launcher_abi::validate_sparse_solve_metadata;
use crate::diagnostics::TransferError;
use crate::runtime::launcher::{
    Checked, DeviceSpan, ElemType, LauncherError, MetadataRevision, SlotBinding,
};

/// 四个自由度、六个非零、两层与两条更新。
fn static_dims() -> SparseSolveStaticDims {
    SparseSolveStaticDims::new(4, 6, 2, 2).unwrap()
}

/// 层级、块地址、对角槽、层偏移与两条更新三元组。
fn metadata() -> Vec<i32> {
    vec![
        2, // 层级
        0, 4, 8, 12, // 块地址
        0, 1, 2, 3, // 对角槽
        0, 1, 2, // 层偏移
        0, 1, 2, // 更新一
        1, 2, 3, // 更新二
    ]
}

fn dims(entry: SparseSolveEntry) -> SparseSolveDims {
    SparseSolveDims {
        static_dims: static_dims(),
        worlds: 2,
        rhs_count: 3,
        entry,
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

fn bindings(lengths: [u64; 4]) -> [SlotBinding; 4] {
    [
        binding(0, 0x1000, lengths[0], ElemType::I32),
        binding(1, 0x2000, lengths[1], ElemType::F32),
        binding(2, 0x3000, lengths[2], ElemType::F32),
        binding(3, 0x9000, lengths[3], ElemType::F32),
    ]
}

#[test]
fn lengths_follow_entry_layout_and_reject_invalid_dimensions() {
    assert_eq!(
        sparse_solve_lengths(&dims(SparseSolveEntry::Validate)),
        Ok([18, 32, 28, 10])
    );
    assert_eq!(
        sparse_solve_lengths(&dims(SparseSolveEntry::Fused)),
        Ok([18, 32, 28, 32])
    );
    let base = dims(SparseSolveEntry::Validate);
    assert!(
        sparse_solve_lengths(&SparseSolveDims {
            rhs_count: 0,
            ..base
        })
        .is_err()
    );
    assert!(sparse_solve_lengths(&SparseSolveDims { worlds: 0, ..base }).is_err());
    assert!(SparseSolveStaticDims::new(0, 6, 2, 2).is_err());
    assert!(SparseSolveStaticDims::new(4, 0, 2, 2).is_err());
}

#[test]
fn metadata_proof_rejects_levels_diagonal_offsets_and_updates() {
    let dims = static_dims();
    assert_eq!(validate_sparse_solve_metadata(&metadata(), dims), Ok(()));
    for (index, value, reason) in [
        (0, 1, "层级计数不符"),
        (5, 10, "对角槽越界"),
        (9, 1, "层偏移起点非零"),
        (11, 0, "层偏移非单调"),
        (11, 3, "层偏移终点越界"),
        (12, 4, "更新目标越界"),
        (17, 10, "更新槽越界"),
    ] {
        let mut values = metadata();
        values[index] = value;
        assert!(
            validate_sparse_solve_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_sparse_solve_metadata(&metadata()[..17], dims).is_err());
}

fn proof(
    bindings: &[SlotBinding; 4],
    dims: SparseSolveDims,
) -> Result<Checked<SparseSolveMetadata>, TransferError> {
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<SparseSolveMetadata>::assume_uploaded(
            bindings[0].span,
            &metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .map_err(super::launcher_abi::sparse_solve_launch_error)
}

#[test]
fn validate_plan_rejects_capacity_context_and_missing_proof() {
    let validate = dims(SparseSolveEntry::Validate);
    let lengths = sparse_solve_lengths(&validate).unwrap();
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
}

#[test]
fn fused_plan_requires_full_solution_width() {
    let fused = dims(SparseSolveEntry::Fused);
    let lengths = sparse_solve_lengths(&fused).unwrap();
    let bounds = bindings(lengths);
    let checked = proof(&bounds, fused).unwrap();
    assert!(plan_fused(&fused, &bounds, &[checked.binding(0)]).is_ok());
    let mut short = bounds;
    short[3].span = span(7, 0x9000, 31 * 4);
    assert_eq!(
        plan_fused(&fused, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity {
            slot: 3,
            required: 128,
            actual: 124,
        })
    );
    let mut alias = bounds;
    alias[3].span = span(7, 0x2000, lengths[3] * 4);
    assert_eq!(
        plan_fused(&fused, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    );
}
