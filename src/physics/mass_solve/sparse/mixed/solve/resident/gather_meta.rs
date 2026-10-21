//! 常驻混合聚集元数据、规格与错误映射。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) const RESIDENT_MIXED_GATHER_METADATA_KIND: ProofKind =
    ProofKind("resident_mixed_gather_metadata");

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticResidentMixedGatherMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticResidentMixedGatherMetadata {
    type Dims = [u32; 2];

    const KIND: ProofKind = RESIDENT_MIXED_GATHER_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        let [n, nc] = *dims;
        if n == 0 || n > i32::MAX as u32 || nc == 0 || nc > i32::MAX as u32 {
            return Err(LauncherError::InvalidMetadata("常驻混合CSR维度无效"));
        }
        let n =
            usize::try_from(n).map_err(|_| LauncherError::Overflow("resident_mixed_gather_n"))?;
        let nc =
            usize::try_from(nc).map_err(|_| LauncherError::Overflow("resident_mixed_gather_nc"))?;
        let expected = n
            .checked_mul(2)
            .and_then(|value| value.checked_add(nc))
            .ok_or(LauncherError::Overflow("resident_mixed_gather_metadata"))?;
        if values.len() != expected {
            return Err(LauncherError::InvalidMetadata("常驻混合CSR长度无效"));
        }
        let (rownnz, rest) = values.split_at(n);
        let (rowadr, colind) = rest.split_at(n);
        let mut next = 0usize;
        for row in 0..n {
            let count = usize::try_from(rownnz[row])
                .map_err(|_| LauncherError::InvalidMetadata("常驻混合CSR行长度无效"))?;
            if count == 0 || rowadr[row] < 0 || rowadr[row] as usize != next {
                return Err(LauncherError::InvalidMetadata("常驻混合CSR行分区无效"));
            }
            let end = next
                .checked_add(count)
                .filter(|&value| value <= nc)
                .ok_or(LauncherError::InvalidMetadata("常驻混合CSR行越界"))?;
            let columns = &colind[next..end];
            if columns.last() != Some(&(row as i32))
                || columns
                    .windows(2)
                    .any(|pair| pair[0] < 0 || pair[0] >= pair[1])
                || columns
                    .last()
                    .is_some_and(|&column| column < 0 || column as usize >= n)
            {
                return Err(LauncherError::InvalidMetadata("常驻混合CSR列索引无效"));
            }
            next = end;
        }
        if next != nc {
            return Err(LauncherError::InvalidMetadata("常驻混合CSR分区未覆盖"));
        }
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMixedGatherLaunchMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for ResidentMixedGatherLaunchMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = RESIDENT_MIXED_GATHER_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        resident_mixed_gather_lengths(dims)?;
        StaticResidentMixedGatherMetadata::validate(values, &[dims[1], dims[2]])
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) const RESIDENT_MIXED_GATHER_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RESIDENT_MIXED_GATHER_METADATA_KIND,
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
pub(super) struct ResidentMixedGatherSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结常驻混合聚集ABI。
unsafe impl KernelAbi for ResidentMixedGatherSpec {
    const NAME: &'static str = "resident_mixed_gather";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RESIDENT_MIXED_GATHER_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentMixedGatherSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (metadata, source, output) = resident_mixed_gather_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: source,
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
        resident_mixed_gather_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mixed_gather_lengths(
    dims: &[u32; 4],
) -> Result<(u64, u64, u64), LauncherError> {
    let [nb, n, nc, work] = *dims;
    if n == 0
        || n > i32::MAX as u32
        || nc == 0
        || nc > i32::MAX as u32
        || work == 0
        || work > u32::MAX - 255
        || !work.is_multiple_of(n)
    {
        return Err(LauncherError::InvalidArray("常驻混合聚集维度无效"));
    }
    let worlds = work / n;
    let metadata = u64::from(n)
        .checked_mul(2)
        .and_then(|value| value.checked_add(u64::from(nc)))
        .ok_or(LauncherError::Overflow("resident_mixed_gather_metadata"))?;
    let stride = u64::from(nb)
        .checked_mul(10)
        .and_then(|body| {
            u64::from(n)
                .checked_mul(u64::from(n))
                .and_then(|dense| body.checked_add(dense))
        })
        .ok_or(LauncherError::Overflow("resident_mixed_gather_stride"))?;
    let source = stride
        .checked_mul(u64::from(worlds))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_mixed_gather_source"))?;
    let output = u64::from(nc)
        .checked_mul(u64::from(worlds))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_mixed_gather_output"))?;
    Ok((metadata, source, output))
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mixed_gather_dims(
    nb: u32,
    n: u32,
    nc: u32,
    worlds: u32,
) -> Result<[u32; 4], LauncherError> {
    let work = worlds
        .checked_mul(n)
        .ok_or(LauncherError::Overflow("resident_mixed_gather_work"))?;
    let dims = [nb, n, nc, work];
    resident_mixed_gather_lengths(&dims)?;
    Ok(dims)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mixed_gather_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::InvalidMetadata(_) | LauncherError::StaleMetadata => {
            InputError::InvalidTopology {
                field: "resident_mixed_gather_metadata",
                index: 0,
                reason: "invalid_or_stale_proof",
            }
            .into()
        }
        _ => InputError::InvalidDimension {
            field: "resident_mixed_gather_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mixed_gather_plan(
    dims: &[u32; 4],
    bindings: &[SlotBinding; 4],
    proof: CheckedBinding<'_, [u32; 4]>,
) -> Result<(), LauncherError> {
    let (metadata, source, output) = resident_mixed_gather_lengths(dims)?;
    for (slot, elements) in [(0, metadata), (1, source), (3, output)] {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("resident_mixed_gather_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("常驻混合聚集跨度无效"));
        }
    }
    if bindings[2].span.bytes() < 4 {
        return Err(LauncherError::InvalidArray("常驻混合聚集未用输入无效"));
    }
    Kernel::<ResidentMixedGatherSpec>::new()?.plan(dims, bindings, &[proof])?;
    Ok(())
}
