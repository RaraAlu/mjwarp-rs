//! Small因子证明、槽位、规格与长度。

#[cfg(feature = "cuda-probe")]
use super::validation_meta::SMALL_FACTOR_METADATA_KIND;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticSmallFactorMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSmallFactorMetadata {
    type Dims = u32;

    const KIND: ProofKind = SMALL_FACTOR_METADATA_KIND;

    fn validate(values: &[i32], prefix: &u32) -> Result<(), LauncherError> {
        if values == [0] {
            return Ok(());
        }
        if *prefix == 0 || *prefix > i32::MAX as u32 {
            return Err(LauncherError::InvalidMetadata("Small前缀无效"));
        }
        if values.is_empty() || !values.len().is_multiple_of(3) {
            return Err(LauncherError::InvalidMetadata("Small元数据长度无效"));
        }
        let mut ranges = Vec::with_capacity(values.len() / 3);
        for triple in values.as_chunks::<3>().0 {
            let _dof = u32::try_from(triple[0])
                .map_err(|_| LauncherError::InvalidMetadata("Small自由度无效"))?;
            let address = u32::try_from(triple[1])
                .map_err(|_| LauncherError::InvalidMetadata("Small地址无效"))?;
            let size = u32::try_from(triple[2])
                .map_err(|_| LauncherError::InvalidMetadata("Small尺寸无效"))?;
            if !(2..=6).contains(&size) {
                return Err(LauncherError::InvalidMetadata("Small尺寸越界"));
            }
            let end = address
                .checked_add(
                    size.checked_mul(size)
                        .ok_or(LauncherError::Overflow("small_factor_block"))?,
                )
                .filter(|&end| end <= *prefix)
                .ok_or(LauncherError::InvalidMetadata("Small块越界"))?;
            if ranges
                .iter()
                .any(|&(start, stop)| address < stop && start < end)
            {
                return Err(LauncherError::InvalidMetadata("Small块重叠"));
            }
            ranges.push((address, end));
        }
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SmallFactorLaunchMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SmallFactorLaunchMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = SMALL_FACTOR_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        small_factor_lengths(dims)?;
        StaticSmallFactorMetadata::validate(values, &dims[0])?;
        let first =
            usize::try_from(dims[1]).map_err(|_| LauncherError::Overflow("small_factor_first"))?;
        let count =
            usize::try_from(dims[2]).map_err(|_| LauncherError::Overflow("small_factor_count"))?;
        let end = first
            .checked_add(count)
            .and_then(|value| value.checked_mul(3))
            .ok_or(LauncherError::Overflow("small_factor_metadata"))?;
        if end > values.len() {
            return Err(LauncherError::InvalidMetadata("Small分组越界"));
        }
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) const SMALL_FACTOR_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SMALL_FACTOR_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Write,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(feature = "cuda-probe")]
pub(super) struct SmallFactorSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结Small因子ABI。
unsafe impl KernelAbi for SmallFactorSpec {
    const NAME: &'static str = "small_prefix_factor";
    const BLOCK: BlockRule = BlockRule::Fixed(64);
    const SLOTS: &'static [SlotDesc] = &SMALL_FACTOR_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SmallFactorSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (metadata, buffers) = small_factor_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: buffers,
            },
            SlotRequirement {
                slot: 2,
                min_elements: buffers,
            },
            SlotRequirement {
                slot: 3,
                min_elements: buffers,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        small_factor_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(64), 1, 1), (64, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn small_factor_lengths(dims: &[u32; 4]) -> Result<(u64, u64), LauncherError> {
    let [prefix, first, count, work] = *dims;
    if prefix == 0
        || prefix > i32::MAX as u32
        || count == 0
        || work == 0
        || work > u32::MAX - 63
        || !work.is_multiple_of(count)
    {
        return Err(LauncherError::InvalidArray("Small因子维度无效"));
    }
    let metadata = u64::from(first)
        .checked_add(u64::from(count))
        .and_then(|value| value.checked_mul(3))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("small_factor_metadata"))?;
    let buffers = u64::from(work / count)
        .checked_mul(u64::from(prefix))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("small_factor_buffers"))?;
    Ok((metadata, buffers))
}

#[cfg(feature = "cuda-probe")]
pub(super) fn small_factor_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "small_factor_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "small_factor_abi",
        }
        .into(),
    }
}
