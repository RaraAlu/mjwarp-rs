//! 常驻质心位置元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::com_launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::com_launcher_abi::ResidentComDims;
#[cfg(feature = "cuda-probe")]
use super::com_launcher_abi::ResidentComStaticDims;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验元数据：体父链、关节表、自由度地址与两对字段描述符。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_resident_com_metadata(
    values: &[i32],
    dims: &ResidentComStaticDims,
) -> Result<(), LauncherError> {
    let expected = dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("常驻质心元数据长度无效"));
    }
    let nb = usize::try_from(dims.nb).map_err(|_| LauncherError::Overflow("resident_com_nb"))?;
    let nj = usize::try_from(dims.nj).map_err(|_| LauncherError::Overflow("resident_com_nj"))?;
    let nv = usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("resident_com_nv"))?;
    let parameters = u64::from(dims.parameters_len).max(1);
    let parents = &values[..nb];
    if parents[0] != -1 && parents[0] != 0 {
        return Err(LauncherError::InvalidMetadata("常驻质心根体父引用无效"));
    }
    for (body, &parent) in parents.iter().enumerate().skip(1) {
        if parent < 0 || parent as usize >= body {
            return Err(LauncherError::InvalidMetadata("常驻质心体父链无效"));
        }
    }
    let types = &values[nb..nb + nj];
    if types.iter().any(|&kind| !(0..=3).contains(&kind)) {
        return Err(LauncherError::InvalidMetadata("常驻质心关节类型越界"));
    }
    let bodies = &values[nb + nj..nb + 2 * nj];
    if bodies.iter().any(|&body| body < 0 || body as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("常驻质心关节体越界"));
    }
    let addresses = &values[nb + 2 * nj..nb + 3 * nj];
    for (&address, &kind) in addresses.iter().zip(types) {
        let width = match kind {
            0 => 6,
            1 => 3,
            _ => 1,
        };
        let address = usize::try_from(address)
            .map_err(|_| LauncherError::InvalidMetadata("常驻质心关节自由度为负"))?;
        if address.checked_add(width).is_none_or(|end| end > nv) {
            return Err(LauncherError::InvalidMetadata("常驻质心关节自由度越界"));
        }
    }
    let descriptors = &values[nb + 3 * nj..];
    for (index, width) in [u64::from(dims.nb), 3 * u64::from(dims.nb)]
        .into_iter()
        .enumerate()
    {
        let offset = usize::try_from(descriptors[2 * index])
            .map_err(|_| LauncherError::InvalidMetadata("常驻质心字段偏移为负"))?;
        let batches = usize::try_from(descriptors[2 * index + 1])
            .map_err(|_| LauncherError::InvalidMetadata("常驻质心字段批次数为负"))?;
        if batches == 0 {
            return Err(LauncherError::InvalidMetadata("常驻质心字段批次为零"));
        }
        let end = u64::try_from(offset)
            .ok()
            .and_then(|value| {
                u64::try_from(batches)
                    .ok()?
                    .checked_mul(width)
                    .and_then(|w| value.checked_add(w))
            })
            .ok_or(LauncherError::Overflow("resident_com_parameters"))?;
        if end > parameters {
            return Err(LauncherError::InvalidMetadata("常驻质心字段区间越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentComMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ResidentComMetadata {
    type Dims = ResidentComDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_resident_com_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_com_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_com_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_com_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_com_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_com_abi",
        }
        .into(),
    }
}
