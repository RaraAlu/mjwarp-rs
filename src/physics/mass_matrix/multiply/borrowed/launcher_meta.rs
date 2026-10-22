//! 借用质量矩阵元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 校验元数据：守卫、前缀、行偏移、列、下三角地址与跳过标志。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_borrowed_mass_metadata(
    values: &[i32],
    static_dims: &BorrowedMassStaticDims,
    worlds: u32,
) -> Result<(), LauncherError> {
    let expected = static_dims.metadata_elements(worlds)?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("借用质量矩阵元数据长度无效"));
    }
    let n =
        usize::try_from(static_dims.n).map_err(|_| LauncherError::Overflow("borrowed_mass_n"))?;
    let count = usize::try_from(static_dims.count)
        .map_err(|_| LauncherError::Overflow("borrowed_mass_count"))?;
    let so = usize::try_from(static_dims.so()?)
        .map_err(|_| LauncherError::Overflow("borrowed_mass_metadata"))?;
    let worlds =
        usize::try_from(worlds).map_err(|_| LauncherError::Overflow("borrowed_mass_worlds"))?;
    if values[..4] != [-131072; 4] || values[so + worlds..so + worlds + 4] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("借用质量矩阵元数据守卫无效"));
    }
    let count_i32 =
        i32::try_from(count).map_err(|_| LauncherError::Overflow("borrowed_mass_count"))?;
    if values[4] != i32::try_from(n).map_err(|_| LauncherError::Overflow("borrowed_mass_n"))? {
        return Err(LauncherError::InvalidMetadata("借用质量矩阵自由度不符"));
    }
    if values[5] != count_i32 {
        return Err(LauncherError::InvalidMetadata("借用质量矩阵行总数不符"));
    }
    let rows = &values[6..6 + n + 1];
    if rows[0] != 0 || rows[n] != count_i32 {
        return Err(LauncherError::InvalidMetadata("借用质量矩阵行偏移端点无效"));
    }
    for pair in rows.windows(2) {
        let start = usize::try_from(pair[0])
            .map_err(|_| LauncherError::InvalidMetadata("借用质量矩阵行偏移为负"))?;
        let end = usize::try_from(pair[1])
            .map_err(|_| LauncherError::InvalidMetadata("借用质量矩阵行偏移为负"))?;
        if end < start || end > count {
            return Err(LauncherError::InvalidMetadata("借用质量矩阵行偏移越界"));
        }
    }
    let columns = &values[7 + n..7 + n + count];
    if columns
        .iter()
        .any(|&column| column < 0 || column as usize >= n)
    {
        return Err(LauncherError::InvalidMetadata("借用质量矩阵列索引越界"));
    }
    let addresses = &values[7 + n + count..so];
    for (row, pair) in rows.windows(2).enumerate() {
        let start = usize::try_from(pair[0])
            .map_err(|_| LauncherError::InvalidMetadata("借用质量矩阵行偏移为负"))?;
        let end = usize::try_from(pair[1])
            .map_err(|_| LauncherError::InvalidMetadata("借用质量矩阵行偏移为负"))?;
        for (slot, &column) in columns.iter().enumerate().take(end).skip(start) {
            let column = column as usize;
            let (high, low) = (row.max(column), row.min(column));
            let expected = u64::try_from(high)
                .ok()
                .and_then(|value| value.checked_mul(u64::try_from(n).ok()?))
                .and_then(|value| value.checked_add(u64::try_from(low).ok()?))
                .ok_or(LauncherError::Overflow("borrowed_mass_address"))?;
            let actual = addresses[slot];
            if actual < 0 || u64::try_from(actual).ok() != Some(expected) {
                return Err(LauncherError::InvalidMetadata("借用质量矩阵下三角地址不符"));
            }
        }
    }
    if values[so..so + worlds]
        .iter()
        .any(|&skip| skip != 0 && skip != 1)
    {
        return Err(LauncherError::InvalidMetadata("借用质量矩阵跳过标志无效"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct BorrowedMassValidateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for BorrowedMassValidateMetadata {
    type Dims = BorrowedMassValidateDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_borrowed_mass_metadata(values, &dims.static_dims, dims.worlds)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct BorrowedMassComputeMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for BorrowedMassComputeMetadata {
    type Dims = BorrowedMassComputeDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_borrowed_mass_metadata(values, &dims.static_dims, dims.worlds)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn borrowed_mass_validate_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "borrowed_mul_m_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "borrowed_mul_m_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "borrowed_mul_m_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "borrowed_mul_m_validate_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn borrowed_mass_compute_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "borrowed_mul_m_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "borrowed_mul_m_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "borrowed_mul_m_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "borrowed_mul_m_compute_abi",
        }
        .into(),
    }
}
