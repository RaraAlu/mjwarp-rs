//! 常驻CRB元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbAccumulateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbInitDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbMatrixDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbStaticDims;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验拓扑元数据：体父链、关节体、关节父链与深度表。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_resident_crb_metadata(
    values: &[i32],
    static_dims: &ResidentCrbStaticDims,
) -> Result<(), LauncherError> {
    let expected = static_dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("常驻CRB元数据长度无效"));
    }
    let nbody = usize::try_from(static_dims.nbody)
        .map_err(|_| LauncherError::Overflow("resident_crb_nbody"))?;
    let nv =
        usize::try_from(static_dims.nv).map_err(|_| LauncherError::Overflow("resident_crb_nv"))?;
    let depth_limit = i32::try_from(static_dims.depth)
        .map_err(|_| LauncherError::Overflow("resident_crb_depth"))?;
    if values[0]
        != i32::try_from(nbody).map_err(|_| LauncherError::Overflow("resident_crb_nbody"))?
    {
        return Err(LauncherError::InvalidMetadata("常驻CRB刚体数不符"));
    }
    if values[1] != i32::try_from(nv).map_err(|_| LauncherError::Overflow("resident_crb_nv"))? {
        return Err(LauncherError::InvalidMetadata("常驻CRB自由度不符"));
    }
    let parents = &values[2..2 + nbody];
    if parents[0] != -1 && parents[0] != 0 {
        return Err(LauncherError::InvalidMetadata("常驻CRB根体父引用无效"));
    }
    for (body, &parent) in parents.iter().enumerate().skip(1) {
        if parent < 0 || parent as usize >= body {
            return Err(LauncherError::InvalidMetadata("常驻CRB体父链无效"));
        }
    }
    let dof_body = &values[2 + nbody..2 + nbody + nv];
    if dof_body
        .iter()
        .any(|&body| body < 0 || body as usize >= nbody)
    {
        return Err(LauncherError::InvalidMetadata("常驻CRB关节体越界"));
    }
    let dof_parent = &values[2 + nbody + nv..2 + nbody + 2 * nv];
    for (dof, &parent) in dof_parent.iter().enumerate() {
        let valid = if dof == 0 {
            parent == -1
        } else {
            parent >= -1 && (parent < 0 || (parent as usize) < dof)
        };
        if !valid {
            return Err(LauncherError::InvalidMetadata("常驻CRB关节父链无效"));
        }
    }
    let depth = &values[2 + nbody + 2 * nv..2 + 2 * nbody + 2 * nv];
    if depth[0] != 0 {
        return Err(LauncherError::InvalidMetadata("常驻CRB根深度非零"));
    }
    for (body, &value) in depth.iter().enumerate().skip(1) {
        let parent = parents[body] as usize;
        if value != depth[parent] + 1 {
            return Err(LauncherError::InvalidMetadata("常驻CRB深度表不符"));
        }
    }
    if depth.iter().any(|&value| value < 0) || depth.iter().max() != Some(&depth_limit) {
        return Err(LauncherError::InvalidMetadata("常驻CRB深度上界不符"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbInitMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ResidentCrbInitMetadata {
    type Dims = ResidentCrbInitDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_resident_crb_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbAccumulateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ResidentCrbAccumulateMetadata {
    type Dims = ResidentCrbAccumulateDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_resident_crb_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbMatrixMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ResidentCrbMatrixMetadata {
    type Dims = ResidentCrbMatrixDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_resident_crb_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_crb_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_crb_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_crb_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_crb_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_crb_abi",
        }
        .into(),
    }
}
