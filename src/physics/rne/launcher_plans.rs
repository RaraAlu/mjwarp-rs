//! RNE四类发射计划与错误映射。

#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RNE_BACKWARD_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RNE_CACC_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RNE_CFRC_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RNE_QFRC_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneBackwardDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneCaccDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneCfrcDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneQfrcDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::RneStaticDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::RneBackwardSpec;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::RneCaccSpec;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::RneCfrcSpec;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::RneQfrcSpec;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::rne_backward_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::rne_cacc_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::rne_cfrc_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::rne_qfrc_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_meta::validate_rne_metadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticRneMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticRneMetadata {
    type Dims = RneStaticDims;

    const KIND: ProofKind = RNE_QFRC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, *dims)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneCaccMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneCaccMetadata {
    type Dims = RneCaccDims;

    const KIND: ProofKind = RNE_CACC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, dims.static_dims)?;
        rne_cacc_lengths(dims).map(|_| ())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_cacc_plan(
    dims: &RneCaccDims,
    bindings: &[SlotBinding; 6],
    proofs: &[CheckedBinding<'_, RneCaccDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_cacc_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_cacc_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneCaccSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn rne_cacc_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_cacc_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_cacc_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneCfrcMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneCfrcMetadata {
    type Dims = RneCfrcDims;

    const KIND: ProofKind = RNE_CFRC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, dims.static_dims)?;
        rne_cfrc_lengths(dims).map(|_| ())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_cfrc_plan(
    dims: &RneCfrcDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RneCfrcDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_cfrc_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_cfrc_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneCfrcSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn rne_cfrc_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_cfrc_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_cfrc_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneQfrcMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneQfrcMetadata {
    type Dims = RneQfrcDims;

    const KIND: ProofKind = RNE_QFRC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, dims.static_dims)?;
        rne_qfrc_lengths(dims).map(|_| ())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_qfrc_plan(
    dims: &RneQfrcDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RneQfrcDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_qfrc_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_qfrc_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneQfrcSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn rne_qfrc_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_qfrc_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_qfrc_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneBackwardMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneBackwardMetadata {
    type Dims = RneBackwardDims;

    const KIND: ProofKind = RNE_BACKWARD_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, dims.static_dims)?;
        rne_backward_lengths(dims).map(|_| ())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_backward_plan(
    dims: &RneBackwardDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RneBackwardDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_backward_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_backward_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneBackwardSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn rne_backward_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_backward_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_backward_abi",
        }
        .into(),
    }
}
