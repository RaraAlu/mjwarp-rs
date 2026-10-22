//! 重力补偿元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityClearDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravitySumDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验元数据：守卫、根表、祖先表、自由度关节与驱动器标志。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_gravity_metadata(
    values: &[i32],
    dims: &GravityStaticDims,
) -> Result<(), LauncherError> {
    let expected = dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("重力补偿元数据长度无效"));
    }
    let nb =
        usize::try_from(dims.nb).map_err(|_| LauncherError::Overflow("gravity_compensation_nb"))?;
    let nj =
        usize::try_from(dims.nj).map_err(|_| LauncherError::Overflow("gravity_compensation_nj"))?;
    let nv =
        usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("gravity_compensation_nv"))?;
    let tail = values.len() - 4;
    if values[..4] != [-131072; 4] || values[tail..] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("重力补偿元数据守卫无效"));
    }
    let roots = &values[4..4 + nb];
    if roots.iter().any(|&root| root < 0 || root as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("重力补偿根表越界"));
    }
    let ancestry = &values[4 + nb..4 + nb + nb * nv];
    if ancestry.iter().any(|&value| value != 0 && value != 1) {
        return Err(LauncherError::InvalidMetadata("重力补偿祖先表非布尔"));
    }
    let dof_jnt = &values[4 + nb + nb * nv..4 + nb + nb * nv + nv];
    if dof_jnt.iter().any(|&jnt| jnt < 0 || jnt as usize >= nj) {
        return Err(LauncherError::InvalidMetadata("重力补偿关节引用越界"));
    }
    let act = &values[4 + nb + nb * nv + nv..tail];
    if act.iter().any(|&value| value != 0 && value != 1) {
        return Err(LauncherError::InvalidMetadata("重力补偿驱动器标志非布尔"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityClearMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for GravityClearMetadata {
    type Dims = GravityClearDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_gravity_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityValidateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for GravityValidateMetadata {
    type Dims = GravityValidateDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_gravity_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityComputeMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for GravityComputeMetadata {
    type Dims = GravityComputeDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_gravity_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravitySumMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for GravitySumMetadata {
    type Dims = GravitySumDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_gravity_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn gravity_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "gravity_compensation_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "gravity_compensation_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "gravity_compensation_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "gravity_compensation_abi",
        }
        .into(),
    }
}
