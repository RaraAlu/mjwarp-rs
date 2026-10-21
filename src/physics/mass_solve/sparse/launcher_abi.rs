//! 稀疏质量发射器证明与维度。

#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_diag::sparse_mass_diag_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_specs::sparse_mass_copy_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_specs::sparse_mass_update_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;
#[cfg(any(feature = "cuda-probe", test))]
use std::collections::BTreeSet;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const SPARSE_MASS_COPY_METADATA_KIND: ProofKind = ProofKind("sparse_mass_copy_indices");

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const SPARSE_MASS_UPDATE_METADATA_KIND: ProofKind =
    ProofKind("sparse_mass_update_indices");

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const SPARSE_MASS_DIAG_METADATA_KIND: ProofKind = ProofKind("sparse_mass_diag_indices");

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SparseMassCopyDims {
    pub(super) launch: [u32; 4],
    pub(super) nv: u32,
    pub(super) nc: u32,
    pub(super) metadata_elements: u32,
}

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SparseMassUpdateDims {
    pub(super) launch: [u32; 4],
    pub(super) nv: u32,
    pub(super) nc: u32,
    pub(super) metadata_elements: u32,
    pub(super) update_count: u32,
    pub(super) worlds: u32,
}

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SparseMassDiagDims {
    pub(super) launch: [u32; 4],
    pub(super) nv: u32,
    pub(super) nc: u32,
    pub(super) metadata_elements: u32,
    pub(super) update_count: u32,
    pub(super) worlds: u32,
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn validate_sparse_mass_copy_metadata(
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
pub(super) fn sparse_mass_update_regions(
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
pub(super) fn validate_sparse_mass_update_metadata(
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
pub(super) fn validate_sparse_mass_update_level(
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
pub(super) struct StaticSparseMassCopyMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSparseMassCopyMetadata {
    type Dims = [u32; 3];

    const KIND: ProofKind = SPARSE_MASS_COPY_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_mass_copy_metadata(values, dims[0], dims[1], dims[2])
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticSparseMassUpdateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSparseMassUpdateMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = SPARSE_MASS_UPDATE_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_mass_update_metadata(values, dims[0], dims[1], dims[2], dims[3])
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticSparseMassDiagMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSparseMassDiagMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = SPARSE_MASS_DIAG_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_mass_update_metadata(values, dims[0], dims[1], dims[2], dims[3])
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct SparseMassCopyLaunchMetadata;

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
pub(super) struct SparseMassUpdateLaunchMetadata;

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
pub(super) struct SparseMassDiagLaunchMetadata;

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
