//! 质心位置元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComPositionDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComPositionStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验元数据：体父链、关节类型、关节体与关节自由度地址。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_com_position_metadata(
    values: &[i32],
    dims: &ComPositionStaticDims,
) -> Result<(), LauncherError> {
    let expected = dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("质心位置元数据长度无效"));
    }
    let nbody =
        usize::try_from(dims.nbody).map_err(|_| LauncherError::Overflow("com_position_nbody"))?;
    let njnt =
        usize::try_from(dims.njnt).map_err(|_| LauncherError::Overflow("com_position_njnt"))?;
    let nv = usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("com_position_nv"))?;
    let parents = &values[..nbody];
    if parents[0] != -1 && parents[0] != 0 {
        return Err(LauncherError::InvalidMetadata("质心位置根体父引用无效"));
    }
    for (body, &parent) in parents.iter().enumerate().skip(1) {
        if parent < 0 || parent as usize >= body {
            return Err(LauncherError::InvalidMetadata("质心位置体父链无效"));
        }
    }
    let types = &values[nbody..nbody + njnt];
    if types.iter().any(|&kind| !(0..=3).contains(&kind)) {
        return Err(LauncherError::InvalidMetadata("质心位置关节类型越界"));
    }
    let bodies = &values[nbody + njnt..nbody + 2 * njnt];
    if bodies
        .iter()
        .any(|&body| body < 0 || body as usize >= nbody)
    {
        return Err(LauncherError::InvalidMetadata("质心位置关节体越界"));
    }
    let addresses = &values[nbody + 2 * njnt..];
    for (&address, &kind) in addresses.iter().zip(types) {
        let width = match kind {
            0 => 6,
            1 => 3,
            _ => 1,
        };
        if address < 0 || (address as usize).saturating_add(width) > nv {
            return Err(LauncherError::InvalidMetadata("质心位置关节自由度越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ComPositionMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ComPositionMetadata {
    type Dims = ComPositionDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_com_position_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn com_position_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "com_position_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "com_position_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "com_position_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "com_position_abi",
        }
        .into(),
    }
}
