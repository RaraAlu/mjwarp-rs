//! 元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::APPLIED_FORCE_DELTA_METADATA_KIND;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::APPLIED_FORCE_STATIC_METADATA_KIND;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::APPLIED_FORCE_VALIDATE_METADATA_KIND;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::applied_force_delta_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::applied_force_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn validate_applied_force_metadata(
    values: &[i32],
    nb: u32,
    n: u32,
) -> Result<(), LauncherError> {
    let expected = u64::from(nb)
        .checked_mul(2)
        .and_then(|value| value.checked_add(u64::from(n)))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("applied_force_delta_metadata"))?;
    if expected > i32::MAX as u64 || u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("外力元数据长度无效"));
    }
    if values[..4] != [-131072; 4] || values[values.len() - 4..] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("外力元数据守卫无效"));
    }
    let nb = usize::try_from(nb).map_err(|_| LauncherError::Overflow("applied_force_nb"))?;
    let n = usize::try_from(n).map_err(|_| LauncherError::Overflow("applied_force_n"))?;
    let parent = &values[4..4 + nb];
    let root = &values[4 + nb..4 + 2 * nb];
    let body = &values[4 + 2 * nb..4 + 2 * nb + n];
    for b in 0..nb {
        let p = parent[b];
        if (b == 0 && p != 0) || (b > 0 && (p < 0 || p as usize >= b)) {
            return Err(LauncherError::InvalidMetadata("外力父体索引无效"));
        }
        let r = root[b];
        if r < 0 || r as usize >= nb {
            return Err(LauncherError::InvalidMetadata("外力根索引越界"));
        }
        let expected = if b == 0 {
            0
        } else if p == 0 {
            b as i32
        } else {
            root[p as usize]
        };
        if r != expected {
            return Err(LauncherError::InvalidMetadata("外力根索引无效"));
        }
    }
    if body.iter().any(|&value| value < 0 || value as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("外力自由度体索引越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticAppliedForceMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticAppliedForceMetadata {
    type Dims = [u32; 2];

    const KIND: ProofKind = APPLIED_FORCE_STATIC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_applied_force_metadata(values, dims[0], dims[1])
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForceValidateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for AppliedForceValidateMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = APPLIED_FORCE_VALIDATE_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        applied_force_validate_lengths(dims)?;
        validate_applied_force_metadata(values, dims[0], dims[1])
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForceDeltaMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for AppliedForceDeltaMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = APPLIED_FORCE_DELTA_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        applied_force_delta_lengths(dims)?;
        validate_applied_force_metadata(values, dims[0], dims[1])
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn applied_force_delta_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "applied_force_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "applied_force_delta_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn applied_force_publish_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        _ => InputError::InvalidDimension {
            field: "applied_force_publish_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn applied_force_validate_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "applied_force_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "applied_force_validate_abi",
        }
        .into(),
    }
}
