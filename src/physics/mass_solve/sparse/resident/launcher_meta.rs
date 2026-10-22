//! 常驻稀疏质量元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentGatherDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentValidateDims;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验元数据：行计数、行地址与列索引。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_sparse_resident_metadata(
    values: &[i32],
    dims: &SparseResidentStaticDims,
) -> Result<(), LauncherError> {
    let expected = dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("常驻稀疏质量元数据长度无效"));
    }
    let n = usize::try_from(dims.n).map_err(|_| LauncherError::Overflow("resident_sparse_n"))?;
    let nc = usize::try_from(dims.nc).map_err(|_| LauncherError::Overflow("resident_sparse_nc"))?;
    if n == 0 {
        if values.len() != 1 || values[0] != 0 {
            return Err(LauncherError::InvalidMetadata(
                "常驻稀疏质量零自由度占位无效",
            ));
        }
        return Ok(());
    }
    let counts = &values[..n];
    let addresses = &values[n..2 * n];
    let columns = &values[2 * n..2 * n + nc];
    for (count, &address) in counts.iter().zip(addresses) {
        let count = usize::try_from(*count)
            .map_err(|_| LauncherError::InvalidMetadata("常驻稀疏质量行计数为负"))?;
        let address = usize::try_from(address)
            .map_err(|_| LauncherError::InvalidMetadata("常驻稀疏质量行地址为负"))?;
        if address.checked_add(count).is_none_or(|end| end > nc) {
            return Err(LauncherError::InvalidMetadata("常驻稀疏质量行区间越界"));
        }
    }
    if columns
        .iter()
        .any(|&column| column < 0 || column as usize >= n)
    {
        return Err(LauncherError::InvalidMetadata("常驻稀疏质量列索引越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseResidentValidateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SparseResidentValidateMetadata {
    type Dims = SparseResidentValidateDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_resident_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseResidentGatherMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SparseResidentGatherMetadata {
    type Dims = SparseResidentGatherDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_resident_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_resident_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_sparse_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_sparse_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_sparse_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_sparse_abi",
        }
        .into(),
    }
}
