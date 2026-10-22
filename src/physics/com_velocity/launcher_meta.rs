//! 空间速度元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComVelocityDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComVelocityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验元数据：体父链、体关节计数与地址、关节类型与自由度地址。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_com_velocity_metadata(
    values: &[i32],
    dims: &ComVelocityStaticDims,
) -> Result<(), LauncherError> {
    let expected = dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("空间速度元数据长度无效"));
    }
    let nbody =
        usize::try_from(dims.nbody).map_err(|_| LauncherError::Overflow("com_velocity_nbody"))?;
    let njnt =
        usize::try_from(dims.njnt).map_err(|_| LauncherError::Overflow("com_velocity_njnt"))?;
    let nv = usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("com_velocity_nv"))?;
    let parents = &values[..nbody];
    let counts = &values[nbody..2 * nbody];
    let addresses = &values[2 * nbody..3 * nbody];
    let types = &values[3 * nbody..3 * nbody + njnt];
    let dofadr = &values[3 * nbody + njnt..];
    if parents[0] != -1 && parents[0] != 0 {
        return Err(LauncherError::InvalidMetadata("空间速度根体父引用无效"));
    }
    for (body, &parent) in parents.iter().enumerate().skip(1) {
        if parent < 0 || parent as usize >= body {
            return Err(LauncherError::InvalidMetadata("空间速度体父链无效"));
        }
    }
    for (count, &address) in counts.iter().zip(addresses) {
        let count = usize::try_from(*count)
            .map_err(|_| LauncherError::InvalidMetadata("空间速度体关节数为负"))?;
        if count == 0 {
            continue;
        }
        let address = usize::try_from(address)
            .map_err(|_| LauncherError::InvalidMetadata("空间速度体关节地址为负"))?;
        if address.checked_add(count).is_none_or(|end| end > njnt) {
            return Err(LauncherError::InvalidMetadata("空间速度体关节地址越界"));
        }
    }
    if types.iter().any(|&kind| !(0..=3).contains(&kind)) {
        return Err(LauncherError::InvalidMetadata("空间速度关节类型越界"));
    }
    for (&address, &kind) in dofadr.iter().zip(types) {
        let width = match kind {
            0 => 6,
            1 => 3,
            _ => 1,
        };
        let address = usize::try_from(address)
            .map_err(|_| LauncherError::InvalidMetadata("空间速度自由度地址为负"))?;
        if address.checked_add(width).is_none_or(|end| end > nv) {
            return Err(LauncherError::InvalidMetadata("空间速度自由度地址越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ComVelocityMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ComVelocityMetadata {
    type Dims = ComVelocityDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_com_velocity_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn com_velocity_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "com_velocity_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "com_velocity_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "com_velocity_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "com_velocity_abi",
        }
        .into(),
    }
}
