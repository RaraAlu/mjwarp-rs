//! 常驻质量分解元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassFactorDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassSolveDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassStaticDims;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验块表：块数、连续分区、尺寸上限与因子格式。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_resident_mass_metadata(
    values: &[i32],
    static_dims: &ResidentMassStaticDims,
) -> Result<(), LauncherError> {
    let expected = static_dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("常驻质量分解元数据长度无效"));
    }
    let n =
        usize::try_from(static_dims.n).map_err(|_| LauncherError::Overflow("resident_mass_n"))?;
    let count = usize::try_from(static_dims.count)
        .map_err(|_| LauncherError::Overflow("resident_mass_count"))?;
    let declared = usize::try_from(values[0])
        .map_err(|_| LauncherError::InvalidMetadata("常驻质量分解块数为负"))?;
    if declared != count {
        return Err(LauncherError::InvalidMetadata("常驻质量分解块数不符"));
    }
    if count == 0 {
        if n != 0 {
            return Err(LauncherError::InvalidMetadata(
                "常驻质量分解零块覆盖非零维度",
            ));
        }
        return Ok(());
    }
    let mut cursor = 0usize;
    for block in 0..count {
        let start = usize::try_from(values[1 + 3 * block])
            .map_err(|_| LauncherError::InvalidMetadata("常驻质量分解块起点为负"))?;
        let size = usize::try_from(values[2 + 3 * block])
            .map_err(|_| LauncherError::InvalidMetadata("常驻质量分解块尺寸为负"))?;
        let kind = values[3 + 3 * block];
        if size == 0 || start != cursor {
            return Err(LauncherError::InvalidMetadata("常驻质量分解块分界无效"));
        }
        cursor = cursor
            .checked_add(size)
            .filter(|&end| end <= n)
            .ok_or(LauncherError::InvalidMetadata("常驻质量分解块越界"))?;
        let expected = match size {
            1 => 0,
            2..=64 => 1,
            _ => 2,
        };
        if kind != expected {
            return Err(LauncherError::InvalidMetadata("常驻质量分解因子格式不符"));
        }
    }
    if cursor != n {
        return Err(LauncherError::InvalidMetadata(
            "常驻质量分解块未覆盖全部自由度",
        ));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMassFactorMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ResidentMassFactorMetadata {
    type Dims = ResidentMassFactorDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_resident_mass_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMassSolveMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ResidentMassSolveMetadata {
    type Dims = ResidentMassSolveDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_resident_mass_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mass_factor_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidDimension {
            field: "mass_factor_metadata",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidDimension {
            field: "mass_factor_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mass_factor_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mass_factor_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mass_solve_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidDimension {
            field: "mass_solve_metadata",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidDimension {
            field: "mass_solve_metadata",
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
