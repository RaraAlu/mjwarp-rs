//! 偏置力规格、证明与发射错误映射。

#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RNE_BIAS_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RNE_BIAS_SLOTS;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RneBiasDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::rne_bias_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::validate_rne_bias_metadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneBiasSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_bias私有ABI。
unsafe impl KernelAbi for RneBiasSpec {
    const NAME: &'static str = "rne_bias";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_BIAS_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneBiasSpec {
    type Dims = RneBiasDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_bias_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_bias_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneBiasMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneBiasMetadata {
    type Dims = RneBiasDims;

    const KIND: ProofKind = RNE_BIAS_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_bias_metadata(values, *dims)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_bias_plan(
    dims: &RneBiasDims,
    bindings: &[SlotBinding; 5],
    proofs: &[CheckedBinding<'_, RneBiasDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_bias_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_bias_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneBiasSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn rne_bias_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_bias_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_bias_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_bias_abi",
        }
        .into(),
    }
}
