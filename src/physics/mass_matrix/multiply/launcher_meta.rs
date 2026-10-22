//! 质量矩阵乘法元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 校验元数据：首偏移、行偏移、总数、列与地址范围、跳过标志。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_mass_multiply_metadata(
    values: &[i32],
    static_dims: &MassMultiplyStaticDims,
    worlds: u32,
) -> Result<(), LauncherError> {
    let expected = static_dims.metadata_elements(worlds)?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("质量矩阵乘法元数据长度无效"));
    }
    let n =
        usize::try_from(static_dims.n).map_err(|_| LauncherError::Overflow("mass_multiply_n"))?;
    let mc =
        usize::try_from(static_dims.mc).map_err(|_| LauncherError::Overflow("mass_multiply_mc"))?;
    let count = usize::try_from(static_dims.count)
        .map_err(|_| LauncherError::Overflow("mass_multiply_count"))?;
    let so = usize::try_from(static_dims.so()?)
        .map_err(|_| LauncherError::Overflow("mass_multiply_metadata"))?;
    let worlds =
        usize::try_from(worlds).map_err(|_| LauncherError::Overflow("mass_multiply_worlds"))?;
    if values[0] != 0 {
        return Err(LauncherError::InvalidMetadata("质量矩阵乘法首行偏移无效"));
    }
    for pair in values[..=n].windows(2) {
        let start = usize::try_from(pair[0])
            .map_err(|_| LauncherError::InvalidMetadata("质量矩阵乘法行偏移为负"))?;
        let end = usize::try_from(pair[1])
            .map_err(|_| LauncherError::InvalidMetadata("质量矩阵乘法行偏移为负"))?;
        if end < start {
            return Err(LauncherError::InvalidMetadata("质量矩阵乘法行偏移递减"));
        }
    }
    let total = usize::try_from(values[n])
        .map_err(|_| LauncherError::InvalidMetadata("质量矩阵乘法行总数为负"))?;
    if total != count {
        return Err(LauncherError::InvalidMetadata("质量矩阵乘法行总数不符"));
    }
    let columns = &values[n + 1..n + 1 + count];
    if columns
        .iter()
        .any(|&column| column < 0 || column as usize >= n)
    {
        return Err(LauncherError::InvalidMetadata("质量矩阵乘法列索引越界"));
    }
    let addresses = &values[n + 1 + count..so];
    if addresses
        .iter()
        .any(|&address| address < 0 || address as usize >= mc)
    {
        return Err(LauncherError::InvalidMetadata("质量矩阵乘法地址越界"));
    }
    if values[so..so + worlds]
        .iter()
        .any(|&skip| skip != 0 && skip != 1)
    {
        return Err(LauncherError::InvalidMetadata("质量矩阵乘法跳过标志无效"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct MassMultiplyValidateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for MassMultiplyValidateMetadata {
    type Dims = MassMultiplyValidateDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_mass_multiply_metadata(values, &dims.static_dims, dims.worlds)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct MassMultiplyComputeMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for MassMultiplyComputeMetadata {
    type Dims = MassMultiplyComputeDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_mass_multiply_metadata(values, &dims.static_dims, dims.worlds)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn mass_multiply_validate_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "mass_multiply_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "mass_multiply_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mass_multiply_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mass_multiply_validate_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn mass_multiply_compute_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "mass_multiply_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "mass_multiply_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mass_multiply_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mass_multiply_compute_abi",
        }
        .into(),
    }
}
