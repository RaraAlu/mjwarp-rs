//! 常驻附着运动学元数据校验、证明与错误映射。

#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::PROOF;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::ResidentAttachedDims;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::ResidentAttachedStaticDims;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;

/// 校验元数据：刚体跨度、几何/site体索引、静态几何标志与四对描述符。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_resident_attached_metadata(
    values: &[i32],
    dims: &ResidentAttachedStaticDims,
) -> Result<(), LauncherError> {
    let expected = dims.metadata_elements()?;
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("常驻附着元数据长度无效"));
    }
    let nb =
        usize::try_from(dims.nb).map_err(|_| LauncherError::Overflow("resident_attached_nb"))?;
    let ng =
        usize::try_from(dims.ng).map_err(|_| LauncherError::Overflow("resident_attached_ng"))?;
    let ns =
        usize::try_from(dims.ns).map_err(|_| LauncherError::Overflow("resident_attached_ns"))?;
    let parameters = u64::from(dims.parameters_len).max(1);
    let stride = i32::try_from(dims.body_stride()?)
        .map_err(|_| LauncherError::Overflow("resident_attached_body"))?;
    if values[0] != stride {
        return Err(LauncherError::InvalidMetadata("常驻附着刚体跨度不符"));
    }
    let geom = &values[1..1 + ng];
    if geom.iter().any(|&body| body < 0 || body as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("常驻附着几何体越界"));
    }
    let sites = &values[1 + ng..1 + ng + ns];
    if sites.iter().any(|&body| body < 0 || body as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("常驻附着site体越界"));
    }
    let static_geom = &values[1 + ng + ns..1 + 2 * ng + ns];
    if static_geom.iter().any(|&flag| flag != 0 && flag != 1) {
        return Err(LauncherError::InvalidMetadata("常驻附着静态几何标志非布尔"));
    }
    let descriptors = &values[1 + 2 * ng + ns..];
    let widths = [
        3 * u64::from(dims.ng),
        4 * u64::from(dims.ng),
        3 * u64::from(dims.ns),
        4 * u64::from(dims.ns),
    ];
    for (index, width) in widths.into_iter().enumerate() {
        let offset = usize::try_from(descriptors[2 * index])
            .map_err(|_| LauncherError::InvalidMetadata("常驻附着字段偏移为负"))?;
        let batches = usize::try_from(descriptors[2 * index + 1])
            .map_err(|_| LauncherError::InvalidMetadata("常驻附着字段批次数为负"))?;
        if batches == 0 {
            return Err(LauncherError::InvalidMetadata("常驻附着字段批次为零"));
        }
        let end = u64::try_from(offset)
            .ok()
            .and_then(|value| {
                u64::try_from(batches)
                    .ok()?
                    .checked_mul(width)
                    .and_then(|w| value.checked_add(w))
            })
            .ok_or(LauncherError::Overflow("resident_attached_parameters"))?;
        if end > parameters {
            return Err(LauncherError::InvalidMetadata("常驻附着字段区间越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentAttachedMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ResidentAttachedMetadata {
    type Dims = ResidentAttachedDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_resident_attached_metadata(values, &dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_attached_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_attached_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_attached_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_attached_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_attached_abi",
        }
        .into(),
    }
}
