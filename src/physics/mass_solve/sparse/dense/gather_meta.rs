//! 稠密聚集元数据证明、槽位与长度。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) const DENSE_GATHER_METADATA_KIND: ProofKind = ProofKind("dense_gather_sources");

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticDenseGatherMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticDenseGatherMetadata {
    type Dims = [u32; 2];

    const KIND: ProofKind = DENSE_GATHER_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        let [nc, prefix] = *dims;
        let sentinel =
            i32::try_from(nc).map_err(|_| LauncherError::InvalidMetadata("稠密源数量越界"))?;
        if prefix == 0 {
            return if values == [0] {
                Ok(())
            } else {
                Err(LauncherError::InvalidMetadata("零前缀占位无效"))
            };
        }
        let prefix =
            usize::try_from(prefix).map_err(|_| LauncherError::Overflow("dense_gather_prefix"))?;
        if values.len() != prefix {
            return Err(LauncherError::InvalidMetadata("稠密源长度无效"));
        }
        if values.iter().any(|&source| source < 0 || source > sentinel) {
            return Err(LauncherError::InvalidMetadata("稠密源索引越界"));
        }
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DenseGatherLaunchMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for DenseGatherLaunchMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = DENSE_GATHER_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        dense_gather_lengths(dims)?;
        StaticDenseGatherMetadata::validate(values, &[dims[0], dims[1]])
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) const DENSE_GATHER_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: DENSE_GATHER_METADATA_KIND,
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
pub(super) struct DenseGatherSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结稠密聚集ABI。
unsafe impl KernelAbi for DenseGatherSpec {
    const NAME: &'static str = "sparse_dense_gather";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &DENSE_GATHER_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for DenseGatherSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (metadata, matrix, output) = dense_gather_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: matrix,
            },
            SlotRequirement {
                slot: 2,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 3,
                min_elements: output,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        dense_gather_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn dense_gather_lengths(dims: &[u32; 4]) -> Result<(u64, u64, u64), LauncherError> {
    let [nc, prefix, reserved, work] = *dims;
    if nc > i32::MAX as u32
        || prefix == 0
        || reserved != 0
        || work == 0
        || work > u32::MAX - 255
        || !work.is_multiple_of(prefix)
    {
        return Err(LauncherError::InvalidArray("稠密聚集维度无效"));
    }
    let worlds = work / prefix;
    let matrix = u64::from(nc)
        .checked_mul(u64::from(worlds))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("dense_gather_matrix"))?;
    let output = u64::from(work)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("dense_gather_output"))?;
    Ok((u64::from(prefix), matrix, output))
}

#[cfg(feature = "cuda-probe")]
pub(super) fn dense_gather_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::InvalidMetadata(_) | LauncherError::StaleMetadata => {
            InputError::InvalidTopology {
                field: "dense_gather_sources",
                index: 0,
                reason: "invalid_or_stale_proof",
            }
            .into()
        }
        _ => InputError::InvalidDimension {
            field: "dense_gather_abi",
        }
        .into(),
    }
}
