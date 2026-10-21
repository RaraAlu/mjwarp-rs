//! 元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SMOOTH_FORCE_COMPUTE_METADATA_KIND;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SMOOTH_FORCE_VALIDATE_METADATA_KIND;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_compute_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_compute_worlds;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::smooth_force_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn validate_smooth_force_metadata(
    values: &[i32],
    dims: &SmoothForceValidateDims,
) -> Result<(), LauncherError> {
    let expected = smooth_force_validate_lengths(dims)?[0];
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("平滑力元数据长度无效"));
    }
    if values[..4] != [-131072; 4] || values[values.len() - 4..] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("平滑力元数据守卫无效"));
    }
    let [nb, n, nt, _] = dims.abi;
    let nb = usize::try_from(nb).map_err(|_| LauncherError::Overflow("smooth_force_nb"))?;
    let n = usize::try_from(n).map_err(|_| LauncherError::Overflow("smooth_force_n"))?;
    let tree = &values[4..4 + nb];
    let body = &values[4 + nb..4 + nb + n];
    if tree
        .iter()
        .any(|&value| value < -1 || (value >= 0 && value as u32 >= nt))
    {
        return Err(LauncherError::InvalidMetadata("平滑力树索引越界"));
    }
    if body.iter().any(|&value| value < 0 || value as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("平滑力自由度体索引越界"));
    }
    // awake允许任意整数；零表示休眠，其他值表示唤醒。
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SmoothForceValidateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SmoothForceValidateMetadata {
    type Dims = SmoothForceValidateDims;

    const KIND: ProofKind = SMOOTH_FORCE_VALIDATE_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_smooth_force_metadata(values, dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SmoothForceComputeMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SmoothForceComputeMetadata {
    type Dims = SmoothForceComputeDims;

    const KIND: ProofKind = SMOOTH_FORCE_COMPUTE_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        smooth_force_compute_lengths(dims)?;
        let worlds = smooth_force_compute_worlds(dims)?;
        let [nb, n, nt, _] = dims.abi;
        validate_smooth_force_metadata(
            values,
            &SmoothForceValidateDims {
                abi: [nb, n, nt, worlds],
                sleep_enabled: dims.sleep_enabled,
            },
        )
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn smooth_force_validate_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "smooth_force_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_) | LauncherError::MetadataMismatch(_) => {
            InputError::InvalidTopology {
                field: "smooth_force_metadata",
                index: 0,
                reason: "invalid_metadata",
            }
            .into()
        }
        _ => InputError::InvalidDimension {
            field: "smooth_force_validate_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn smooth_force_compute_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "smooth_force_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_) | LauncherError::MetadataMismatch(_) => {
            InputError::InvalidTopology {
                field: "smooth_force_metadata",
                index: 0,
                reason: "invalid_metadata",
            }
            .into()
        }
        _ => InputError::InvalidDimension {
            field: "smooth_force_compute_abi",
        }
        .into(),
    }
}
