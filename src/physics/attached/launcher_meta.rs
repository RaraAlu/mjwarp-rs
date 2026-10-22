//! 附着运动学元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::AttachedDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::AttachedStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验元数据：刚体跨度首项、几何体索引与site体索引。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_attached_metadata(
    values: &[i32],
    dims: &AttachedStaticDims,
) -> Result<(), LauncherError> {
    let expected = dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("附着运动学元数据长度无效"));
    }
    let nbody =
        usize::try_from(dims.nbody).map_err(|_| LauncherError::Overflow("attached_nbody"))?;
    let ngeom =
        usize::try_from(dims.ngeom).map_err(|_| LauncherError::Overflow("attached_ngeom"))?;
    let nsite =
        usize::try_from(dims.nsite).map_err(|_| LauncherError::Overflow("attached_nsite"))?;
    let stride =
        i32::try_from(dims.body_stride()?).map_err(|_| LauncherError::Overflow("attached_body"))?;
    if values[0] != stride {
        return Err(LauncherError::InvalidMetadata("附着运动学刚体跨度不符"));
    }
    let bodies = &values[1..1 + ngeom + nsite];
    if bodies
        .iter()
        .any(|&body| body < 0 || body as usize >= nbody)
    {
        return Err(LauncherError::InvalidMetadata("附着运动学体索引越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct AttachedMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for AttachedMetadata {
    type Dims = AttachedDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_attached_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn attached_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "attached_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "attached_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "attached_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "attached_abi",
        }
        .into(),
    }
}
