//! 混合求解发射错误映射与维度转换。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn mixed_solve_validation_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::InvalidMetadata(_)
        | LauncherError::StaleMetadata
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "mixed_solve_validation_metadata",
            index: 0,
            reason: "invalid_or_stale_proof",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mixed_solve_validation_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mixed_solve_validation_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn mixed_solve_fused_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::InvalidMetadata(_)
        | LauncherError::StaleMetadata
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "mixed_solve_fused_metadata",
            index: 0,
            reason: "invalid_or_stale_proof",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mixed_solve_fused_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mixed_solve_fused_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn mixed_solve_validation_dimension(
    value: usize,
    field: &'static str,
) -> Result<u32, TransferError> {
    u32::try_from(value).map_err(|_| InputError::Overflow { field }.into())
}
