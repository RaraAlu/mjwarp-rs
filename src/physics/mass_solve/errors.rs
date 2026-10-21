//! 质量求解发射错误映射。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn rigid_mass_solve_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "mass_solve_blocks",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mass_solve_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mass_solve_abi",
        }
        .into(),
    }
}
