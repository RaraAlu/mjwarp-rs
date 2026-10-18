//! 真正CSR分层分解；不调用稠密辅助。
//! 子模块提供融合回代；非紧凑混合块仍待实现。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, SparseMassInput, SparseMassRoute};
use crate::runtime::TransferSession;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
#[cfg(any(feature = "cuda-probe", test))]
use std::collections::BTreeSet;
use std::sync::Arc;

mod dense;
mod mixed;
pub use dense::{
    SparseMassDenseData, SparseMassDenseOutput, SparseMassDensePlan, SparseMassSmallData,
    SparseMassSmallOutput, SparseMassSmallPlan,
};
pub use mixed::{
    SparseMassMixedFactorData, SparseMassMixedFactorOutput, SparseMassMixedFactorPlan,
    SparseMassMixedFactorWorld, SparseMassMixedResidentData, SparseMassMixedResidentPlan,
    SparseMassMixedSolveData, SparseMassMixedSolveOutput, SparseMassMixedSolvePlan,
    SparseMassMixedSolveWorld,
};

mod resident;
pub use resident::{
    SparseMassData, SparseMassOutput, SparseMassPlan, SparseMassSolveData, SparseMassSolveOutput,
    SparseMassSolvePlan, SparseMassSolveWorld, SparseMassWorld,
};

#[cfg(any(feature = "cuda-probe", test))]
const SPARSE_MASS_COPY_METADATA_KIND: ProofKind = ProofKind("sparse_mass_copy_indices");

#[cfg(any(feature = "cuda-probe", test))]
const SPARSE_MASS_UPDATE_METADATA_KIND: ProofKind = ProofKind("sparse_mass_update_indices");

#[cfg(any(feature = "cuda-probe", test))]
const SPARSE_MASS_DIAG_METADATA_KIND: ProofKind = ProofKind("sparse_mass_diag_indices");

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SparseMassCopyDims {
    launch: [u32; 4],
    nv: u32,
    nc: u32,
    metadata_elements: u32,
}

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SparseMassUpdateDims {
    launch: [u32; 4],
    nv: u32,
    nc: u32,
    metadata_elements: u32,
    update_count: u32,
    worlds: u32,
}

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SparseMassDiagDims {
    launch: [u32; 4],
    nv: u32,
    nc: u32,
    metadata_elements: u32,
    update_count: u32,
    worlds: u32,
}

#[cfg(any(feature = "cuda-probe", test))]
fn validate_sparse_mass_copy_metadata(
    values: &[i32],
    nv: u32,
    nc: u32,
    metadata_elements: u32,
) -> Result<(), LauncherError> {
    let expected = usize::try_from(metadata_elements)
        .map_err(|_| LauncherError::Overflow("sparse_mass_copy_metadata"))?;
    if expected < 2 || values.len() != expected {
        return Err(LauncherError::InvalidMetadata("稀疏复制元数据长度无效"));
    }
    let nv = i32::try_from(nv).map_err(|_| LauncherError::InvalidMetadata("稀疏复制自由度越界"))?;
    let nc =
        i32::try_from(nc).map_err(|_| LauncherError::InvalidMetadata("稀疏复制矩阵长度越界"))?;
    if values[..2] != [nv, nc] {
        return Err(LauncherError::InvalidMetadata("稀疏复制元数据维度不匹配"));
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
fn sparse_mass_update_regions(
    nv: u32,
    update_count: u32,
) -> Result<(usize, usize, usize), LauncherError> {
    let nv = usize::try_from(nv).map_err(|_| LauncherError::Overflow("sparse_mass_update_nv"))?;
    let stored_updates = usize::try_from(update_count.max(1))
        .map_err(|_| LauncherError::Overflow("sparse_mass_update_count"))?;
    let rows_end = 2usize
        .checked_add(nv)
        .ok_or(LauncherError::Overflow("sparse_mass_update_rows"))?;
    let updates_start = rows_end
        .checked_add(nv)
        .ok_or(LauncherError::Overflow("sparse_mass_update_rows"))?;
    let metadata_end = stored_updates
        .checked_mul(3)
        .and_then(|count| updates_start.checked_add(count))
        .ok_or(LauncherError::Overflow("sparse_mass_update_metadata"))?;
    Ok((rows_end, updates_start, metadata_end))
}

#[cfg(any(feature = "cuda-probe", test))]
fn validate_sparse_mass_update_metadata(
    values: &[i32],
    nv: u32,
    nc: u32,
    metadata_elements: u32,
    update_count: u32,
) -> Result<(), LauncherError> {
    let expected = usize::try_from(metadata_elements)
        .map_err(|_| LauncherError::Overflow("sparse_mass_update_metadata"))?;
    let (rows_end, updates_start, metadata_end) = sparse_mass_update_regions(nv, update_count)?;
    if expected != metadata_end || values.len() != expected {
        return Err(LauncherError::InvalidMetadata("稀疏更新元数据长度无效"));
    }
    let nv_i32 =
        i32::try_from(nv).map_err(|_| LauncherError::InvalidMetadata("稀疏更新自由度越界"))?;
    let nc_i32 =
        i32::try_from(nc).map_err(|_| LauncherError::InvalidMetadata("稀疏更新矩阵长度越界"))?;
    if values[..2] != [nv_i32, nc_i32] {
        return Err(LauncherError::InvalidMetadata("稀疏更新元数据维度不匹配"));
    }
    let nv = usize::try_from(nv_i32)
        .map_err(|_| LauncherError::InvalidMetadata("稀疏更新自由度无效"))?;
    let nc = usize::try_from(nc_i32)
        .map_err(|_| LauncherError::InvalidMetadata("稀疏更新矩阵长度无效"))?;
    let rownnz = &values[2..rows_end];
    let rowadr = &values[rows_end..updates_start];
    let mut next = 0usize;
    for row in 0..nv {
        let count = usize::try_from(rownnz[row])
            .map_err(|_| LauncherError::InvalidMetadata("稀疏更新行长度无效"))?;
        let address = usize::try_from(rowadr[row])
            .map_err(|_| LauncherError::InvalidMetadata("稀疏更新行地址无效"))?;
        if count == 0 || address != next {
            return Err(LauncherError::InvalidMetadata("稀疏更新CSR分区无效"));
        }
        next = address
            .checked_add(count)
            .filter(|&end| end <= nc)
            .ok_or(LauncherError::InvalidMetadata("稀疏更新CSR越界"))?;
    }
    if next != nc {
        return Err(LauncherError::InvalidMetadata("稀疏更新CSR长度不匹配"));
    }
    let updates = &values[updates_start..metadata_end];
    if update_count == 0 {
        if updates != [0, 0, 0] {
            return Err(LauncherError::InvalidMetadata("稀疏更新空占位无效"));
        }
        return Ok(());
    }
    for update in updates.as_chunks::<3>().0 {
        let i = usize::try_from(update[0])
            .map_err(|_| LauncherError::InvalidMetadata("稀疏更新目标行无效"))?;
        let k = usize::try_from(update[1])
            .map_err(|_| LauncherError::InvalidMetadata("稀疏更新源行无效"))?;
        let ki = usize::try_from(update[2])
            .map_err(|_| LauncherError::InvalidMetadata("稀疏更新乘数地址无效"))?;
        if i >= nv || k >= nv || i >= k || ki >= nc {
            return Err(LauncherError::InvalidMetadata("稀疏更新三元组越界"));
        }
        let target_count = usize::try_from(rownnz[i])
            .map_err(|_| LauncherError::InvalidMetadata("稀疏更新目标行长度无效"))?;
        let source_count = usize::try_from(rownnz[k])
            .map_err(|_| LauncherError::InvalidMetadata("稀疏更新源行长度无效"))?;
        let source_address = usize::try_from(rowadr[k])
            .map_err(|_| LauncherError::InvalidMetadata("稀疏更新源行地址无效"))?;
        let expected_ki = source_address
            .checked_add(target_count)
            .and_then(|value| value.checked_sub(1))
            .ok_or(LauncherError::InvalidMetadata("稀疏更新乘数地址溢出"))?;
        if target_count >= source_count || ki != expected_ki {
            return Err(LauncherError::InvalidMetadata("稀疏更新祖先前缀无效"));
        }
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
fn validate_sparse_mass_update_level(
    values: &[i32],
    dims: &SparseMassUpdateDims,
    level_offsets: &[u32],
    level: usize,
) -> Result<(), LauncherError> {
    validate_sparse_mass_update_metadata(
        values,
        dims.nv,
        dims.nc,
        dims.metadata_elements,
        dims.update_count,
    )?;
    let next_level = level
        .checked_add(1)
        .ok_or(LauncherError::InvalidMetadata("稀疏更新层级越界"))?;
    let Some((&first, &end)) = level_offsets.get(level).zip(level_offsets.get(next_level)) else {
        return Err(LauncherError::InvalidMetadata("稀疏更新层级越界"));
    };
    let count = end
        .checked_sub(first)
        .filter(|&value| value != 0)
        .ok_or(LauncherError::InvalidMetadata("稀疏更新空层级无效"))?;
    if dims.launch[0] != first || dims.launch[1] != count || end > dims.update_count {
        return Err(LauncherError::InvalidMetadata("稀疏更新层级切片不匹配"));
    }
    let (_, updates_start, _) = sparse_mass_update_regions(dims.nv, dims.update_count)?;
    let start =
        usize::try_from(first).map_err(|_| LauncherError::Overflow("sparse_mass_update_first"))?;
    let end =
        usize::try_from(end).map_err(|_| LauncherError::Overflow("sparse_mass_update_end"))?;
    let mut sources = BTreeSet::new();
    let mut multipliers = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for update in values[updates_start + 3 * start..updates_start + 3 * end]
        .as_chunks::<3>()
        .0
    {
        if !sources.insert(update[1]) || !multipliers.insert(update[2]) {
            return Err(LauncherError::InvalidMetadata("稀疏更新同层写入不唯一"));
        }
        targets.insert(update[0]);
    }
    if sources.iter().any(|source| targets.contains(source)) {
        return Err(LauncherError::InvalidMetadata("稀疏更新同层读写相交"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
struct StaticSparseMassCopyMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSparseMassCopyMetadata {
    type Dims = [u32; 3];

    const KIND: ProofKind = SPARSE_MASS_COPY_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_mass_copy_metadata(values, dims[0], dims[1], dims[2])
    }
}

#[cfg(feature = "cuda-probe")]
struct StaticSparseMassUpdateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSparseMassUpdateMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = SPARSE_MASS_UPDATE_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_mass_update_metadata(values, dims[0], dims[1], dims[2], dims[3])
    }
}

#[cfg(feature = "cuda-probe")]
struct StaticSparseMassDiagMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSparseMassDiagMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = SPARSE_MASS_DIAG_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_mass_update_metadata(values, dims[0], dims[1], dims[2], dims[3])
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct SparseMassCopyLaunchMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for SparseMassCopyLaunchMetadata {
    type Dims = SparseMassCopyDims;

    const KIND: ProofKind = SPARSE_MASS_COPY_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        sparse_mass_copy_lengths(dims)?;
        validate_sparse_mass_copy_metadata(values, dims.nv, dims.nc, dims.metadata_elements)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct SparseMassUpdateLaunchMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for SparseMassUpdateLaunchMetadata {
    type Dims = SparseMassUpdateDims;

    const KIND: ProofKind = SPARSE_MASS_UPDATE_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        sparse_mass_update_lengths(dims)?;
        validate_sparse_mass_update_metadata(
            values,
            dims.nv,
            dims.nc,
            dims.metadata_elements,
            dims.update_count,
        )
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct SparseMassDiagLaunchMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for SparseMassDiagLaunchMetadata {
    type Dims = SparseMassDiagDims;

    const KIND: ProofKind = SPARSE_MASS_DIAG_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        sparse_mass_diag_lengths(dims)?;
        validate_sparse_mass_update_metadata(
            values,
            dims.nv,
            dims.nc,
            dims.metadata_elements,
            dims.update_count,
        )
    }
}

#[cfg(any(feature = "cuda-probe", test))]
const SPARSE_MASS_COPY_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SPARSE_MASS_COPY_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Write,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(any(feature = "cuda-probe", test))]
struct SparseMassCopySpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐项匹配冻结稀疏质量复制ABI。
unsafe impl KernelAbi for SparseMassCopySpec {
    const NAME: &'static str = "sparse_mass_copy";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SPARSE_MASS_COPY_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for SparseMassCopySpec {
    type Dims = SparseMassCopyDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(sparse_mass_copy_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sparse_mass_copy_lengths(dims)?;
        LaunchDims::new((dims.launch[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn sparse_mass_copy_lengths(dims: &SparseMassCopyDims) -> Result<[u64; 4], LauncherError> {
    let [first, count, stride, work] = dims.launch;
    if first != 0 || count != 0 {
        return Err(LauncherError::InvalidArray("稀疏复制保留维度必须为零"));
    }
    let expected_stride = dims
        .nc
        .checked_add(dims.nv)
        .filter(|&value| value <= i32::MAX as u32)
        .ok_or(LauncherError::Overflow("sparse_mass_copy_stride"))?;
    if stride != expected_stride || work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("稀疏复制维度无效"));
    }
    if dims.metadata_elements < 2 {
        return Err(LauncherError::InvalidMetadata("稀疏复制元数据长度无效"));
    }
    let guarded = |elements: u64, field| {
        elements
            .checked_add(8)
            .ok_or(LauncherError::Overflow(field))
    };
    let worlds = u64::from(work);
    let matrix = guarded(
        worlds
            .checked_mul(u64::from(dims.nc))
            .ok_or(LauncherError::Overflow("sparse_mass_copy_matrix"))?,
        "sparse_mass_copy_matrix",
    )?;
    let output = guarded(
        worlds
            .checked_mul(u64::from(stride))
            .ok_or(LauncherError::Overflow("sparse_mass_copy_output"))?,
        "sparse_mass_copy_output",
    )?;
    Ok([u64::from(dims.metadata_elements), matrix, matrix, output])
}

#[cfg(any(feature = "cuda-probe", test))]
fn sparse_mass_copy_plan(
    dims: &SparseMassCopyDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, SparseMassCopyDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = sparse_mass_copy_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("sparse_mass_copy_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("稀疏复制跨度不符"));
        }
    }
    let plan = Kernel::<SparseMassCopySpec>::new()?.plan(dims, bindings, proofs)?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
fn sparse_mass_copy_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "sparse_mass_copy_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "sparse_mass_copy_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
const SPARSE_MASS_UPDATE_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SPARSE_MASS_UPDATE_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(any(feature = "cuda-probe", test))]
struct SparseMassUpdateSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐项匹配冻结稀疏质量更新ABI。
unsafe impl KernelAbi for SparseMassUpdateSpec {
    const NAME: &'static str = "sparse_mass_update";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SPARSE_MASS_UPDATE_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for SparseMassUpdateSpec {
    type Dims = SparseMassUpdateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(sparse_mass_update_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sparse_mass_update_lengths(dims)?;
        LaunchDims::new((dims.launch[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn sparse_mass_update_lengths(dims: &SparseMassUpdateDims) -> Result<[u64; 4], LauncherError> {
    let [first, count, stride, work] = dims.launch;
    first
        .checked_add(count)
        .filter(|&value| count != 0 && value <= dims.update_count)
        .ok_or(LauncherError::InvalidArray("稀疏更新区间无效"))?;
    let expected_stride = dims
        .nc
        .checked_add(dims.nv)
        .filter(|&value| value <= i32::MAX as u32)
        .ok_or(LauncherError::Overflow("sparse_mass_update_stride"))?;
    let expected_work = dims
        .worlds
        .checked_mul(count)
        .ok_or(LauncherError::Overflow("sparse_mass_update_work"))?;
    if stride != expected_stride
        || dims.worlds == 0
        || work != expected_work
        || work == 0
        || work > u32::MAX - 255
    {
        return Err(LauncherError::InvalidArray("稀疏更新维度无效"));
    }
    let (_, _, metadata_end) = sparse_mass_update_regions(dims.nv, dims.update_count)?;
    if usize::try_from(dims.metadata_elements).ok() != Some(metadata_end) {
        return Err(LauncherError::InvalidMetadata("稀疏更新元数据长度无效"));
    }
    let guarded = |elements: u64, field| {
        elements
            .checked_add(8)
            .ok_or(LauncherError::Overflow(field))
    };
    let worlds = u64::from(dims.worlds);
    let matrix = guarded(
        worlds
            .checked_mul(u64::from(dims.nc))
            .ok_or(LauncherError::Overflow("sparse_mass_update_matrix"))?,
        "sparse_mass_update_matrix",
    )?;
    let output = guarded(
        worlds
            .checked_mul(u64::from(stride))
            .ok_or(LauncherError::Overflow("sparse_mass_update_output"))?,
        "sparse_mass_update_output",
    )?;
    Ok([u64::from(dims.metadata_elements), matrix, matrix, output])
}

#[cfg(any(feature = "cuda-probe", test))]
fn sparse_mass_update_plan(
    dims: &SparseMassUpdateDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, SparseMassUpdateDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = sparse_mass_update_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("sparse_mass_update_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("稀疏更新跨度不符"));
        }
    }
    let plan = Kernel::<SparseMassUpdateSpec>::new()?.plan(dims, bindings, proofs)?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
fn sparse_mass_update_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "sparse_mass_update_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "sparse_mass_update_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
const SPARSE_MASS_DIAG_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SPARSE_MASS_DIAG_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(any(feature = "cuda-probe", test))]
struct SparseMassDiagSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐项匹配冻结稀疏质量对角ABI。
unsafe impl KernelAbi for SparseMassDiagSpec {
    const NAME: &'static str = "sparse_mass_diag";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SPARSE_MASS_DIAG_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for SparseMassDiagSpec {
    type Dims = SparseMassDiagDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(sparse_mass_diag_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sparse_mass_diag_lengths(dims)?;
        LaunchDims::new((dims.launch[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn sparse_mass_diag_lengths(dims: &SparseMassDiagDims) -> Result<[u64; 4], LauncherError> {
    let [first, count, stride, work] = dims.launch;
    if first != 0 || count != 0 {
        return Err(LauncherError::InvalidArray("稀疏对角保留维度必须为零"));
    }
    let expected_stride = dims
        .nc
        .checked_add(dims.nv)
        .filter(|&value| value <= i32::MAX as u32)
        .ok_or(LauncherError::Overflow("sparse_mass_diag_stride"))?;
    let expected_work = dims
        .worlds
        .checked_mul(dims.nv)
        .ok_or(LauncherError::Overflow("sparse_mass_diag_work"))?;
    if stride != expected_stride
        || dims.worlds == 0
        || dims.nv == 0
        || work != expected_work
        || work > u32::MAX - 255
    {
        return Err(LauncherError::InvalidArray("稀疏对角维度无效"));
    }
    let (_, _, metadata_end) = sparse_mass_update_regions(dims.nv, dims.update_count)?;
    if usize::try_from(dims.metadata_elements).ok() != Some(metadata_end) {
        return Err(LauncherError::InvalidMetadata("稀疏对角元数据长度无效"));
    }
    let guarded = |elements: u64, field| {
        elements
            .checked_add(8)
            .ok_or(LauncherError::Overflow(field))
    };
    let worlds = u64::from(dims.worlds);
    let matrix = guarded(
        worlds
            .checked_mul(u64::from(dims.nc))
            .ok_or(LauncherError::Overflow("sparse_mass_diag_matrix"))?,
        "sparse_mass_diag_matrix",
    )?;
    let output = guarded(
        worlds
            .checked_mul(u64::from(stride))
            .ok_or(LauncherError::Overflow("sparse_mass_diag_output"))?,
        "sparse_mass_diag_output",
    )?;
    Ok([u64::from(dims.metadata_elements), matrix, matrix, output])
}

#[cfg(any(feature = "cuda-probe", test))]
fn sparse_mass_diag_plan(
    dims: &SparseMassDiagDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, SparseMassDiagDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = sparse_mass_diag_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("sparse_mass_diag_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("稀疏对角跨度不符"));
        }
    }
    let plan = Kernel::<SparseMassDiagSpec>::new()?.plan(dims, bindings, proofs)?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
fn sparse_mass_diag_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "sparse_mass_diag_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "sparse_mass_diag_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
struct SparseMassCopyMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    nv: u32,
    nc: u32,
    metadata_elements: u32,
    update_count: u32,
    level_offsets: Vec<u32>,
    revision: MetadataRevision,
    checked: Checked<StaticSparseMassCopyMetadata>,
    checked_update: Checked<StaticSparseMassUpdateMetadata>,
    checked_diag: Checked<StaticSparseMassDiagMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SparseMassCopyMetadataBuffer {
    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        nv: u32,
        nc: u32,
        level_offsets: &[i32],
    ) -> Result<Self, TransferError> {
        let metadata_elements = u32::try_from(values.len()).map_err(|_| InputError::Overflow {
            field: "sparse_mass_copy_metadata",
        })?;
        let level_offsets = level_offsets
            .iter()
            .map(|&value| {
                u32::try_from(value).map_err(|_| InputError::InvalidDimension {
                    field: "sparse_mass_update_levels",
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if level_offsets.first() != Some(&0)
            || level_offsets.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_update_levels",
            }
            .into());
        }
        let update_count = *level_offsets.last().ok_or(InputError::InvalidDimension {
            field: "sparse_mass_update_levels",
        })?;
        let static_dims = [nv, nc, metadata_elements];
        StaticSparseMassCopyMetadata::validate(&values, &static_dims)
            .map_err(sparse_mass_copy_launch_error)?;
        let update_static_dims = [nv, nc, metadata_elements, update_count];
        StaticSparseMassUpdateMetadata::validate(&values, &update_static_dims)
            .map_err(sparse_mass_update_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且跨度来自同一缓冲。
        let checked = unsafe {
            Checked::<StaticSparseMassCopyMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                static_dims,
                revision.clone(),
            )
        }
        .map_err(sparse_mass_copy_launch_error)?;
        // SAFETY: 同一上传保存完整CSR与生成后的更新元组。
        let checked_update = unsafe {
            Checked::<StaticSparseMassUpdateMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                update_static_dims,
                revision.clone(),
            )
        }
        .map_err(sparse_mass_update_launch_error)?;
        // SAFETY: 同一上传保存完整CSR和对角寻址所需行布局。
        let checked_diag = unsafe {
            Checked::<StaticSparseMassDiagMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                update_static_dims,
                revision.clone(),
            )
        }
        .map_err(sparse_mass_diag_launch_error)?;
        Ok(Self {
            buffer,
            values,
            nv,
            nc,
            metadata_elements,
            update_count,
            level_offsets,
            revision,
            checked,
            checked_update,
            checked_diag,
        })
    }

    fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    fn proof(
        &self,
        dims: SparseMassCopyDims,
    ) -> Result<Checked<SparseMassCopyLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &[self.nv, self.nc, self.metadata_elements])
            .map_err(sparse_mass_copy_launch_error)?;
        // SAFETY: 独占包装保留上传值和写入代际。
        unsafe {
            Checked::<SparseMassCopyLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_copy_launch_error)
    }

    fn update_proof(
        &self,
        dims: SparseMassUpdateDims,
        level: usize,
    ) -> Result<Checked<SparseMassUpdateLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked_update
            .binding(0)
            .validate_for(
                span,
                &[self.nv, self.nc, self.metadata_elements, self.update_count],
            )
            .map_err(sparse_mass_update_launch_error)?;
        validate_sparse_mass_update_level(&self.values, &dims, &self.level_offsets, level)
            .map_err(sparse_mass_update_launch_error)?;
        // SAFETY: 私有输入生成元组；层级切片与上传代际均已复核。
        unsafe {
            Checked::<SparseMassUpdateLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_update_launch_error)
    }

    fn diag_proof(
        &self,
        dims: SparseMassDiagDims,
    ) -> Result<Checked<SparseMassDiagLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked_diag
            .binding(0)
            .validate_for(
                span,
                &[self.nv, self.nc, self.metadata_elements, self.update_count],
            )
            .map_err(sparse_mass_diag_launch_error)?;
        // SAFETY: 独占包装保留完整行布局、上传跨度和写入代际。
        unsafe {
            Checked::<SparseMassDiagLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_diag_launch_error)
    }

    #[cfg(test)]
    fn write_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入同步完成且跨度未变。
        self.checked = unsafe {
            Checked::<StaticSparseMassCopyMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.nv, self.nc, self.metadata_elements],
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_copy_launch_error)?;
        // SAFETY: 恢复写入同步完成且完整元数据保持原值。
        self.checked_update = unsafe {
            Checked::<StaticSparseMassUpdateMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.nv, self.nc, self.metadata_elements, self.update_count],
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_update_launch_error)?;
        // SAFETY: 恢复写入同步完成且对角行布局保持原值。
        self.checked_diag = unsafe {
            Checked::<StaticSparseMassDiagMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.nv, self.nc, self.metadata_elements, self.update_count],
                self.revision.clone(),
            )
        }
        .map_err(sparse_mass_diag_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
struct SparseMassCopyKernel {
    kernel: SynchronousKernel,
    nv: u32,
    nc: u32,
    metadata_elements: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseMassCopyKernel {
    fn compile(
        session: &TransferSession,
        metadata: &SparseMassCopyMetadataBuffer,
    ) -> Result<Self, TransferError> {
        let stride = metadata
            .nc
            .checked_add(metadata.nv)
            .ok_or(InputError::Overflow {
                field: "sparse_mass_copy_stride",
            })?;
        sparse_mass_copy_lengths(&SparseMassCopyDims {
            launch: [0, 0, stride, 1],
            nv: metadata.nv,
            nc: metadata.nc,
            metadata_elements: metadata.metadata_elements,
        })
        .map_err(sparse_mass_copy_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_mass_copy")?,
            nv: metadata.nv,
            nc: metadata.nc,
            metadata_elements: metadata.metadata_elements,
        })
    }

    fn launch(
        &self,
        metadata: &SparseMassCopyMetadataBuffer,
        matrix: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if (metadata.nv, metadata.nc, metadata.metadata_elements)
            != (self.nv, self.nc, self.metadata_elements)
        {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_copy_static_dims",
            }
            .into());
        }
        let work = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "sparse_mass_copy_worlds",
        })?;
        let stride = self.nc.checked_add(self.nv).ok_or(InputError::Overflow {
            field: "sparse_mass_copy_stride",
        })?;
        let dims = SparseMassCopyDims {
            launch: [0, 0, stride, work],
            nv: self.nv,
            nc: self.nc,
            metadata_elements: self.metadata_elements,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: matrix.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        sparse_mass_copy_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(sparse_mass_copy_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度、可信索引和写入别名。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), matrix, unused, output, dims.launch)
        }
    }
}

#[cfg(feature = "cuda-probe")]
struct SparseMassUpdateKernel {
    kernel: SynchronousKernel,
    nv: u32,
    nc: u32,
    metadata_elements: u32,
    update_count: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseMassUpdateKernel {
    fn compile(
        session: &TransferSession,
        metadata: &SparseMassCopyMetadataBuffer,
    ) -> Result<Self, TransferError> {
        StaticSparseMassUpdateMetadata::validate(
            &metadata.values,
            &[
                metadata.nv,
                metadata.nc,
                metadata.metadata_elements,
                metadata.update_count,
            ],
        )
        .map_err(sparse_mass_update_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_mass_update")?,
            nv: metadata.nv,
            nc: metadata.nc,
            metadata_elements: metadata.metadata_elements,
            update_count: metadata.update_count,
        })
    }

    fn launch(
        &self,
        metadata: &SparseMassCopyMetadataBuffer,
        unused: &TransferBuffer<f32>,
        unused2: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
        level: usize,
    ) -> Result<(), TransferError> {
        if (
            metadata.nv,
            metadata.nc,
            metadata.metadata_elements,
            metadata.update_count,
        ) != (self.nv, self.nc, self.metadata_elements, self.update_count)
        {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_update_static_dims",
            }
            .into());
        }
        let next_level = level.checked_add(1).ok_or(InputError::InvalidDimension {
            field: "sparse_mass_update_level",
        })?;
        let Some((&first, &end)) = metadata
            .level_offsets
            .get(level)
            .zip(metadata.level_offsets.get(next_level))
        else {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_update_level",
            }
            .into());
        };
        let count = end.checked_sub(first).filter(|&value| value != 0).ok_or(
            InputError::InvalidDimension {
                field: "sparse_mass_update_level",
            },
        )?;
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "sparse_mass_update_worlds",
        })?;
        let stride = self.nc.checked_add(self.nv).ok_or(InputError::Overflow {
            field: "sparse_mass_update_stride",
        })?;
        let work = worlds.checked_mul(count).ok_or(InputError::Overflow {
            field: "sparse_mass_update_work",
        })?;
        let dims = SparseMassUpdateDims {
            launch: [first, count, stride, work],
            nv: self.nv,
            nc: self.nc,
            metadata_elements: self.metadata_elements,
            update_count: self.update_count,
            worlds,
        };
        let proof = metadata.update_proof(dims, level)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused2.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        sparse_mass_update_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(sparse_mass_update_launch_error)?;
        // SAFETY: 类型化计划核对真实层级、跨度、代际与写入隔离。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), unused, unused2, output, dims.launch)
        }
    }
}

#[cfg(feature = "cuda-probe")]
struct SparseMassDiagKernel {
    kernel: SynchronousKernel,
    nv: u32,
    nc: u32,
    metadata_elements: u32,
    update_count: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseMassDiagKernel {
    fn compile(
        session: &TransferSession,
        metadata: &SparseMassCopyMetadataBuffer,
    ) -> Result<Self, TransferError> {
        StaticSparseMassDiagMetadata::validate(
            &metadata.values,
            &[
                metadata.nv,
                metadata.nc,
                metadata.metadata_elements,
                metadata.update_count,
            ],
        )
        .map_err(sparse_mass_diag_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_mass_diag")?,
            nv: metadata.nv,
            nc: metadata.nc,
            metadata_elements: metadata.metadata_elements,
            update_count: metadata.update_count,
        })
    }

    fn launch(
        &self,
        metadata: &SparseMassCopyMetadataBuffer,
        unused: &TransferBuffer<f32>,
        unused2: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if (
            metadata.nv,
            metadata.nc,
            metadata.metadata_elements,
            metadata.update_count,
        ) != (self.nv, self.nc, self.metadata_elements, self.update_count)
        {
            return Err(InputError::InvalidDimension {
                field: "sparse_mass_diag_static_dims",
            }
            .into());
        }
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "sparse_mass_diag_worlds",
        })?;
        let stride = self.nc.checked_add(self.nv).ok_or(InputError::Overflow {
            field: "sparse_mass_diag_stride",
        })?;
        let work = worlds.checked_mul(self.nv).ok_or(InputError::Overflow {
            field: "sparse_mass_diag_work",
        })?;
        let dims = SparseMassDiagDims {
            launch: [0, 0, stride, work],
            nv: self.nv,
            nc: self.nc,
            metadata_elements: self.metadata_elements,
            update_count: self.update_count,
            worlds,
        };
        let proof = metadata.diag_proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused2.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        sparse_mass_diag_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(sparse_mass_diag_launch_error)?;
        // SAFETY: 类型化计划核对完整行布局、跨度、代际和写入隔离。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), unused, unused2, output, dims.launch)
        }
    }
}

pub struct SparseMassFactorPlan {
    input: Arc<SparseMassInput>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: SparseMassCopyMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    copy: SparseMassCopyKernel,
    #[cfg(feature = "cuda-probe")]
    update: SparseMassUpdateKernel,
    #[cfg(feature = "cuda-probe")]
    diag: SparseMassDiagKernel,
}
pub struct SparseMassFactorData {
    input: Arc<SparseMassInput>,
    matrix_layout: BatchLayout,
    output_layout: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    matrix: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassFactorOutput {
    input: Arc<SparseMassInput>,
    layout: BatchLayout,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassFactorWorld<'a> {
    /// 对应上游qLD[qLD_block_total..]。
    /// 它不包含尚未实现的dense前缀。
    pub sparse_factors: &'a [f32],
    pub diagonal_inverse: &'a [f32],
}
impl SparseMassFactorOutput {
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<SparseMassFactorWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        let v = &self.values[4 + r.start..4 + r.end];
        Ok(SparseMassFactorWorld {
            sparse_factors: &v[..self.input.nc()],
            diagonal_inverse: &v[self.input.nc()..],
        })
    }
}
fn layouts(
    input: &SparseMassInput,
    worlds: usize,
) -> Result<(BatchLayout, BatchLayout), InputError> {
    let stride = input
        .nc()
        .checked_add(input.nv())
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "sparse_factor_stride",
        })?;
    // 两种并行内核均采用单一线性工作域。
    let max_updates = input
        .qld_level_offsets()
        .windows(2)
        .map(|p| (p[1] - p[0]) as usize)
        .max()
        .unwrap_or(1);
    if worlds == 0
        || worlds
            .checked_mul(max_updates.max(input.nv()).max(1))
            .is_none_or(|v| v > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "sparse_factor_work_items",
        });
    }
    let m = BatchLayout::new(worlds, input.nc(), 4)?;
    let o = BatchLayout::new(worlds, stride, 4)?;
    for count in [m.total_elements(), o.total_elements()] {
        BatchLayout::new(
            1,
            count.checked_add(8).ok_or(InputError::Overflow {
                field: "sparse_factor_guards",
            })?,
            4,
        )?;
    }
    Ok((m, o))
}
fn matrix_values(layout: BatchLayout, values: &[f32]) -> Result<(), InputError> {
    if values.len() != layout.total_elements() {
        return Err(InputError::LengthMismatch {
            field: "sparse_mass_M",
            expected: layout.total_elements(),
            actual: values.len(),
        });
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: "sparse_mass_M",
                index,
            });
        }
    }
    Ok(())
}
impl SparseMassFactorPlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        // 无稀疏块时，上游不分配CSR因子尾区。
        // 不能用本工作区冒充纯紧凑qLD布局。
        if input.nv() != 0
            && !input
                .blocks()
                .iter()
                .any(|b| b.route == SparseMassRoute::SparseLdl)
        {
            return Err(InputError::InvalidTopology {
                field: "qLD_block_adr",
                index: 0,
                reason: "sparse_factor_region_required",
            }
            .into());
        }
        for block in input.blocks() {
            if !matches!(
                block.route,
                SparseMassRoute::SparseLdl | SparseMassRoute::Compact
            ) {
                return Err(InputError::InvalidTopology {
                    field: "qLD_block_adr",
                    index: block.start,
                    reason: "noncompact_mixed_factor_stage_not_implemented",
                }
                .into());
            }
        }
        Self::from_input(session, input)
    }
    // The mixed caller validates supported routes before using this factory.
    // Keep the public constructor's noncompact rejection above unchanged.
    fn from_input(
        session: &TransferSession,
        input: SparseMassInput,
    ) -> Result<Self, TransferError> {
        let input = Arc::new(input);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, input);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let nv = u32::try_from(input.nv()).map_err(|_| InputError::Overflow {
                field: "sparse_factor_nv",
            })?;
            let nc = u32::try_from(input.nc()).map_err(|_| InputError::Overflow {
                field: "sparse_factor_nc",
            })?;
            let nv_i32 = i32::try_from(nv).map_err(|_| InputError::Overflow {
                field: "sparse_factor_nv",
            })?;
            let nc_i32 = i32::try_from(nc).map_err(|_| InputError::Overflow {
                field: "sparse_factor_nc",
            })?;
            let mut packed = vec![nv_i32, nc_i32];
            packed.extend(&input.fields().m_rownnz);
            packed.extend(&input.fields().m_rowadr);
            packed.extend(input.qld_all_updates().iter().flatten());
            if packed.len() > i32::MAX as usize {
                return Err(InputError::Overflow {
                    field: "sparse_factor_metadata",
                }
                .into());
            }
            let metadata = SparseMassCopyMetadataBuffer::upload(
                session,
                packed,
                nv,
                nc,
                input.qld_level_offsets(),
            )?;
            let copy = SparseMassCopyKernel::compile(session, &metadata)?;
            let update = SparseMassUpdateKernel::compile(session, &metadata)?;
            let diag = SparseMassDiagKernel::compile(session, &metadata)?;
            Ok(Self {
                input,
                session: session.clone(),
                metadata,
                copy,
                update,
                diag,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassFactorData, TransferError> {
        let (matrix_layout, output_layout) = layouts(&self.input, worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix_layout, output_layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |layout: BatchLayout, initial| {
                let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
                values.fill(-131072.0);
                values[4..4 + layout.total_elements()].fill(initial);
                self.session.upload(&values)
            };
            Ok(SparseMassFactorData {
                input: self.input.clone(),
                matrix_layout,
                output_layout,
                ready: false,
                matrix: allocate(matrix_layout, 0.0)?,
                output: allocate(output_layout, f32::NAN)?,
            })
        }
    }
    pub fn update_factor(&self, data: &mut SparseMassFactorData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds_usize = data.worlds();
            self.copy.launch(
                &self.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.output,
                worlds_usize,
            )?;
            for level in (0..self.input.qld_level_offsets().len() - 1).rev() {
                self.update.launch(
                    &self.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.output,
                    worlds_usize,
                    level,
                )?;
            }
            if self.input.nv() != 0 {
                self.diag.launch(
                    &self.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.output,
                    worlds_usize,
                )?;
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
impl SparseMassFactorData {
    pub fn worlds(&self) -> usize {
        self.output_layout.worlds()
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        matrix_values(self.matrix_layout, values)?;
        self.write(0, values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let range = self.matrix_layout.world_elements(world)?;
        matrix_values(BatchLayout::new(1, self.input.nc(), 4)?, values)?;
        self.write(range.start, values)
    }
    fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.matrix.write_range(4 + offset, values)
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut matrix =
            crate::runtime::host_staging::<f32>(self.matrix_layout.total_elements() + 8)?;
        self.matrix.read_range_into(0, &mut matrix)?;
        crate::physics::check_device_values(&matrix, "sparse_mass_M")?;
        let mut values =
            crate::runtime::host_staging::<f32>(self.output_layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut values)?;
        // 先定位主元，再统一检查有限值与守卫。
        for world in 0..self.worlds() {
            let start = 4 + self.output_layout.world_elements(world)?.start;
            for dof in 0..self.input.nv() {
                let f = self.input.fields();
                let diagonal = values[start + (f.m_rowadr[dof] + f.m_rownnz[dof] - 1) as usize];
                if !diagonal.is_finite() || diagonal <= 0.0 {
                    return Err(TransferError::InvalidPivot {
                        world,
                        dof,
                        value: f64::from(diagonal),
                    });
                }
            }
        }
        crate::physics::check_device_values(&values, "sparse_mass_factor")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassFactorOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_mass_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        Ok(SparseMassFactorOutput {
            input: self.input.clone(),
            layout: self.output_layout,
            values: self.checked_values()?,
        })
    }
}

// Frozen smooth.py _qLD_acc/_qLDiag_div (71da24d), Apache-2.0.
// Copyright 2025 The Newton Developers; licenses/Apache-2.0.txt.
// Private ABI replaces array_t descriptors, not CSR or arithmetic semantics.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void sparse_mass_copy(const int* meta,const float* matrix,
 const float* unused,float* out,unsigned first,unsigned count,unsigned stride,unsigned work) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 unsigned nc=meta[1],n=meta[0];float* L=out+4+(unsigned long long)w*stride;
 for(unsigned j=0;j<nc;++j)L[j]=matrix[4+(unsigned long long)w*nc+j];
 for(unsigned i=0;i<n;++i)L[nc+i]=0.0f;
}
extern "C" __global__ void sparse_mass_update(const int* meta,const float* unused,
 const float* unused2,float* out,unsigned first,unsigned count,unsigned stride,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/count,u=first+tid%count,n=meta[0];
 const int* rownnz=meta+2;const int* rowadr=rownnz+n;const int* updates=rowadr+n;
 int i=updates[3*u],k=updates[3*u+1],ki=updates[3*u+2];
 float* L=out+4+(unsigned long long)w*stride;
 float tmp=__fdiv_rn(L[ki],L[rowadr[k]+rownnz[k]-1]);
 for(int j=0;j<rownnz[i];++j) {
   float value=__fmul_rn(L[rowadr[k]+j],tmp);
   float ignored;
   asm volatile("atom.global.add.f32 %0, [%1], %2;" : "=f"(ignored) : "l"(L+rowadr[i]+j),"f"(-value) : "memory");
 }
 L[ki]=tmp;
}
extern "C" __global__ void sparse_mass_diag(const int* meta,const float* unused,
 const float* unused2,float* out,unsigned first,unsigned count,unsigned stride,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned n=meta[0],nc=meta[1],w=tid/n,i=tid%n;
 const int* rownnz=meta+2;const int* rowadr=rownnz+n;
 float* L=out+4+(unsigned long long)w*stride;
 L[nc+i]=__fdiv_rn(1.0f,L[rowadr[i]+rownnz[i]-1]);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cuda-probe")]
    use crate::model::SparseMassFields;

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

    #[cfg(feature = "cuda-probe")]
    fn assert_copy_matches_matrix(data: &SparseMassFactorData) {
        let mut matrix = vec![0.0; data.matrix.len()];
        let mut output = vec![0.0; data.output.len()];
        data.matrix.read_range_into(0, &mut matrix).unwrap();
        data.output.read_range_into(0, &mut output).unwrap();
        for guard in output[..4].iter().chain(&output[output.len() - 4..]) {
            assert_eq!(guard.to_bits(), (-131072.0f32).to_bits());
        }
        let nc = data.input.nc();
        let stride = nc + data.input.nv();
        for world in 0..data.worlds() {
            let matrix_start = 4 + world * nc;
            let output_start = 4 + world * stride;
            for (actual, expected) in output[output_start..output_start + nc]
                .iter()
                .zip(&matrix[matrix_start..matrix_start + nc])
            {
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
            for value in &output[output_start + nc..output_start + stride] {
                assert_eq!(value.to_bits(), 0.0f32.to_bits());
            }
        }
    }

    #[cfg(feature = "cuda-probe")]
    fn assert_close(actual: &[f32], expected: &serde_json::Value) {
        let expected = expected.as_array().unwrap();
        assert_eq!(actual.len(), expected.len());
        for (&actual, expected) in actual.iter().zip(expected) {
            let expected = expected.as_f64().unwrap();
            let error = (f64::from(actual) - expected).abs();
            assert!(
                error <= 2e-4 + 2e-4 * expected.abs(),
                "actual={actual} expected={expected} error={error}"
            );
        }
    }

    #[cfg(feature = "cuda-probe")]
    fn assert_reconstruction(input: &SparseMassInput, factor: &[f32], matrix: &[f32]) {
        let n = input.nv();
        let fields = input.fields();
        let mut lower = vec![0.0f64; n * n];
        let mut source = vec![0.0f64; n * n];
        let mut diagonal = vec![0.0f64; n];
        for row in 0..n {
            for item in 0..fields.m_rownnz[row] as usize {
                let address = fields.m_rowadr[row] as usize + item;
                let column = fields.m_colind[address] as usize;
                source[row * n + column] = f64::from(matrix[address]);
                source[column * n + row] = f64::from(matrix[address]);
                if column == row {
                    diagonal[row] = f64::from(factor[address]);
                    lower[row * n + row] = 1.0;
                } else {
                    lower[row * n + column] = f64::from(factor[address]);
                }
            }
        }
        let norm = source
            .chunks(n)
            .map(|row| row.iter().map(|value| value.abs()).sum::<f64>())
            .fold(0.0, f64::max);
        let mut error = 0.0f64;
        for row in 0..n {
            let mut row_error = 0.0;
            for column in 0..n {
                let reconstructed = (0..n)
                    .map(|k| lower[k * n + row] * diagonal[k] * lower[k * n + column])
                    .sum::<f64>();
                row_error += (reconstructed - source[row * n + column]).abs();
            }
            error = error.max(row_error);
        }
        let relative = error / norm;
        assert!(relative <= 2e-6, "reconstruction={relative}");
    }

    #[cfg(feature = "cuda-probe")]
    fn assert_factor_matches_native(data: &SparseMassFactorData, model: &serde_json::Value) {
        let output = data.readback().unwrap();
        let cases = model["cases"].as_array().unwrap();
        for world in 0..output.worlds() {
            let case = &cases[world % cases.len()];
            let view = output.world(world).unwrap();
            assert_close(view.sparse_factors, &case["factor"]);
            assert_close(view.diagonal_inverse, &case["diagonal_inverse"]);
            let matrix = case["matrix"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_f64().unwrap() as f32)
                .collect::<Vec<_>>();
            assert_reconstruction(output.input(), view.sparse_factors, &matrix);
        }
    }

    #[cfg(feature = "cuda-probe")]
    fn assert_diag_matches_factors_bitwise(
        data: &SparseMassFactorData,
        before: &[f32],
        after: &[f32],
    ) {
        assert_eq!(before.len(), after.len());
        for index in (0..4).chain(after.len() - 4..after.len()) {
            assert_eq!(before[index].to_bits(), after[index].to_bits());
        }
        let fields = data.input.fields();
        let nc = data.input.nc();
        let nv = data.input.nv();
        let stride = nc + nv;
        for world in 0..data.worlds() {
            let start = 4 + world * stride;
            for index in 0..nc {
                assert_eq!(
                    before[start + index].to_bits(),
                    after[start + index].to_bits()
                );
            }
            for row in 0..nv {
                let address = fields.m_rowadr[row] as usize;
                let count = fields.m_rownnz[row] as usize;
                let expected = 1.0f32 / before[start + address + count - 1];
                assert_eq!(expected.to_bits(), after[start + nc + row].to_bits());
            }
        }
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC; new sparse buffer fault purpose"]
    fn matrix_write_is_bit_isolated_and_both_guards_reject_corruption() {
        let refs: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sparse-mass-factor/native.json"
        )))
        .unwrap();
        let m = &refs["models"][2];
        let ints = |name: &str| {
            m[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect()
        };
        let input = SparseMassInput::new(SparseMassFields {
            dof_parentid: ints("dof_parentid"),
            m_rownnz: ints("m_rownnz"),
            m_rowadr: ints("m_rowadr"),
            m_colind: ints("m_colind"),
            tree_dofadr: ints("tree_dofadr"),
            tree_dofnum: ints("tree_dofnum"),
        })
        .unwrap();
        let nc = input.nc();
        let values = |case: usize| {
            m["cases"][case]["matrix"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect::<Vec<_>>()
        };
        let session = TransferSession::new(0).unwrap();
        let compact = SparseMassInput::new(SparseMassFields {
            dof_parentid: vec![-1],
            m_rownnz: vec![1],
            m_rowadr: vec![0],
            m_colind: vec![0],
            tree_dofadr: vec![0],
            tree_dofnum: vec![1],
        })
        .unwrap();
        assert!(SparseMassFactorPlan::new(&session, compact).is_err());
        let mut plan = SparseMassFactorPlan::new(&session, input).unwrap();
        let mut data = plan.create_data(3).unwrap();
        data.write_matrix(&values(0).repeat(3)).unwrap();
        let mut before = vec![0.0; 3 * nc + 8];
        data.matrix.read_range_into(0, &mut before).unwrap();
        data.write_world_matrix(1, &values(1)).unwrap();
        let mut after = vec![0.0; before.len()];
        data.matrix.read_range_into(0, &mut after).unwrap();
        for index in (0..4 + nc).chain(4 + 2 * nc..after.len()) {
            assert_eq!(before[index].to_bits(), after[index].to_bits());
        }
        assert!(
            before[4 + nc..4 + 2 * nc]
                .iter()
                .zip(&after[4 + nc..4 + 2 * nc])
                .any(|(a, b)| a.to_bits() != b.to_bits())
        );
        let worlds = data.worlds();
        plan.copy
            .launch(
                &plan.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.output,
                worlds,
            )
            .unwrap();
        assert_copy_matches_matrix(&data);
        assert!(!data.ready);
        let foreign = TransferSession::new(0).unwrap();
        let foreign_matrix = foreign.upload(&vec![0.0f32; data.matrix.len()]).unwrap();
        assert!(matches!(
            plan.copy.launch(
                &plan.metadata,
                &foreign_matrix,
                &foreign_matrix,
                &mut data.output,
                worlds,
            ),
            Err(TransferError::SessionMismatch)
        ));
        assert_copy_matches_matrix(&data);
        assert!(!data.ready);
        let mut before_update_reject = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut before_update_reject)
            .unwrap();
        assert!(matches!(
            plan.update.launch(
                &plan.metadata,
                &foreign_matrix,
                &foreign_matrix,
                &mut data.output,
                worlds,
                0,
            ),
            Err(TransferError::SessionMismatch)
        ));
        let mut after_update_reject = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut after_update_reject)
            .unwrap();
        assert_eq!(
            before_update_reject
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            after_update_reject
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        let first = plan.metadata.values[0];
        plan.metadata.write_for_test(0, first + 1).unwrap();
        assert!(
            plan.update
                .launch(
                    &plan.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.output,
                    worlds,
                    0,
                )
                .is_err()
        );
        let mut after_stale_reject = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut after_stale_reject)
            .unwrap();
        assert_eq!(
            after_update_reject
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            after_stale_reject
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.metadata.restore_for_test().unwrap();
        plan.copy
            .launch(
                &plan.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.output,
                worlds,
            )
            .unwrap();
        for level in (0..plan.input.qld_level_offsets().len() - 1).rev() {
            plan.update
                .launch(
                    &plan.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.output,
                    worlds,
                    level,
                )
                .unwrap();
        }
        let mut before_diag_reject = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut before_diag_reject)
            .unwrap();
        assert!(matches!(
            plan.diag.launch(
                &plan.metadata,
                &foreign_matrix,
                &foreign_matrix,
                &mut data.output,
                worlds,
            ),
            Err(TransferError::SessionMismatch)
        ));
        let mut after_diag_foreign = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut after_diag_foreign)
            .unwrap();
        assert_eq!(
            before_diag_reject
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            after_diag_foreign
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        let first = plan.metadata.values[0];
        plan.metadata.write_for_test(0, first + 1).unwrap();
        assert!(
            plan.diag
                .launch(
                    &plan.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.output,
                    worlds,
                )
                .is_err()
        );
        let mut after_diag_stale = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut after_diag_stale)
            .unwrap();
        assert_eq!(
            after_diag_foreign
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            after_diag_stale
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.metadata.restore_for_test().unwrap();
        plan.diag
            .launch(
                &plan.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.output,
                worlds,
            )
            .unwrap();
        let mut after_diag = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut after_diag).unwrap();
        assert_diag_matches_factors_bitwise(&data, &before_diag_reject, &after_diag);
        assert!(!data.ready);

        plan.update_factor(&mut data).unwrap();
        assert_factor_matches_native(&data, m);
        let first = plan.metadata.values[0];
        plan.metadata.write_for_test(0, first + 1).unwrap();
        assert!(plan.update_factor(&mut data).is_err());
        assert!(!data.ready);
        assert!(data.readback().is_err());
        plan.metadata.restore_for_test().unwrap();
        plan.update_factor(&mut data).unwrap();
        assert_factor_matches_native(&data, m);
        data.output.write_range(0, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        data.output.write_range(0, &[-131072.0]).unwrap();
        data.matrix.write_range(0, &[0.0]).unwrap();
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.matrix.write_range(0, &[-131072.0]).unwrap();
        plan.update_factor(&mut data).unwrap();
        assert_factor_matches_native(&data, m);
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC; zero-stride sparse copy purpose"]
    fn empty_sparse_mass_copy_preserves_zero_stride_and_guards() {
        let session = TransferSession::new(0).unwrap();
        let input = SparseMassInput::new(SparseMassFields::default()).unwrap();
        let plan = SparseMassFactorPlan::new(&session, input).unwrap();
        let mut data = plan.create_data(2).unwrap();
        plan.update_factor(&mut data).unwrap();
        let output = data.readback().unwrap();
        assert_eq!(output.worlds(), 2);
        assert!(output.world(0).unwrap().sparse_factors.is_empty());
        assert!(output.world(0).unwrap().diagonal_inverse.is_empty());
        assert!(output.world(1).unwrap().sparse_factors.is_empty());
        assert!(output.world(1).unwrap().diagonal_inverse.is_empty());
        let mut raw = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut raw).unwrap();
        assert_eq!(raw.len(), 8);
        for value in raw {
            assert_eq!(value.to_bits(), (-131072.0f32).to_bits());
        }
    }
}
