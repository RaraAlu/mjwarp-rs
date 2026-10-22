//! 肌腱速度元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::tendon_velocity_metadata_elements;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 校验元数据：守卫、行偏移、行数与列索引。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_tendon_velocity_metadata(
    values: &[i32],
    dims: &TendonVelocityStaticDims,
) -> Result<(), LauncherError> {
    let expected = tendon_velocity_metadata_elements(dims)?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("肌腱速度元数据长度无效"));
    }
    if values[..4] != [-131072; 4] || values[values.len() - 4..] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("肌腱速度元数据守卫无效"));
    }
    let nt = usize::try_from(dims.nt).map_err(|_| LauncherError::Overflow("tendon_velocity_nt"))?;
    let nnz =
        usize::try_from(dims.nnz).map_err(|_| LauncherError::Overflow("tendon_velocity_nnz"))?;
    let rowadr = &values[4..4 + nt];
    let rownnz = &values[4 + nt..4 + 2 * nt];
    let colind = &values[4 + 2 * nt..4 + 2 * nt + nnz];
    for (start, count) in rowadr.iter().zip(rownnz) {
        let start = usize::try_from(*start)
            .map_err(|_| LauncherError::InvalidMetadata("肌腱速度行偏移为负"))?;
        let count = usize::try_from(*count)
            .map_err(|_| LauncherError::InvalidMetadata("肌腱速度行数为负"))?;
        if start > nnz || count > nnz - start {
            return Err(LauncherError::InvalidMetadata("肌腱速度行区间越界"));
        }
    }
    if colind
        .iter()
        .any(|&column| column < 0 || column as usize >= dims.nv as usize)
    {
        return Err(LauncherError::InvalidMetadata("肌腱速度列索引越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityValidateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for TendonVelocityValidateMetadata {
    type Dims = TendonVelocityValidateDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_tendon_velocity_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityComputeMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for TendonVelocityComputeMetadata {
    type Dims = TendonVelocityComputeDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_tendon_velocity_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_velocity_validate_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "tendon_velocity_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "tendon_velocity_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "tendon_velocity_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "tendon_velocity_validate_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_velocity_compute_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "tendon_velocity_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "tendon_velocity_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "tendon_velocity_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "tendon_velocity_compute_abi",
        }
        .into(),
    }
}
