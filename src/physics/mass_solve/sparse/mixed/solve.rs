//! 消费同代合并因子，求解Sparse、Small与Compact。
//! RHS独立批量；Tile与CRB桥接仍不支持。
use super::{
    SparseMassMixedFactorData, SparseMassMixedFactorOutput, SparseMassMixedFactorPlan,
    SparseMassMixedFactorWorld,
};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
#[cfg(any(feature = "cuda-probe", test))]
use crate::model::SparseMassRoute;
use crate::model::{BatchLayout, SparseMassInput};
use crate::runtime::TransferSession;
mod resident;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
pub use resident::{SparseMassMixedResidentData, SparseMassMixedResidentPlan};

#[cfg(any(feature = "cuda-probe", test))]
const MIXED_SOLVE_VALIDATION_METADATA_KIND: ProofKind =
    ProofKind("mixed_solve_validation_metadata");

#[cfg(any(feature = "cuda-probe", test))]
struct StaticMixedSolveValidationMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for StaticMixedSolveValidationMetadata {
    type Dims = [u32; 2];

    const KIND: ProofKind = MIXED_SOLVE_VALIDATION_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        let [n, stride] = *dims;
        if n == 0 || stride == 0 || n > i32::MAX as u32 {
            return Err(LauncherError::InvalidMetadata("混合求解校验维度无效"));
        }
        let n =
            usize::try_from(n).map_err(|_| LauncherError::Overflow("mixed_solve_validation_n"))?;
        let minimum = n
            .checked_mul(2)
            .and_then(|value| value.checked_add(4))
            .ok_or(LauncherError::Overflow("mixed_solve_validation_metadata"))?;
        if values.len() < minimum {
            return Err(LauncherError::InvalidMetadata("混合求解元数据过短"));
        }
        let inverse_base = values[1]
            .checked_add(values[2])
            .ok_or(LauncherError::InvalidMetadata("混合求解D基址溢出"))?;
        let inverse_base = u64::try_from(inverse_base)
            .map_err(|_| LauncherError::InvalidMetadata("混合求解D基址为负"))?;
        let inverse_end = inverse_base
            .checked_add(n as u64)
            .ok_or(LauncherError::Overflow("mixed_solve_validation_inverse"))?;
        if inverse_end > u64::from(stride) {
            return Err(LauncherError::InvalidMetadata("混合求解D区越界"));
        }
        if values[4 + n..4 + 2 * n]
            .iter()
            .any(|&index| index < 0 || index as u64 >= u64::from(stride))
        {
            return Err(LauncherError::InvalidMetadata("混合求解对角索引越界"));
        }
        Ok(())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct MixedSolveValidationLaunchMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for MixedSolveValidationLaunchMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = MIXED_SOLVE_VALIDATION_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        mixed_solve_validation_lengths(dims)?;
        StaticMixedSolveValidationMetadata::validate(values, &[dims[0], dims[1]])
    }
}

#[cfg(any(feature = "cuda-probe", test))]
const MIXED_SOLVE_VALIDATION_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: MIXED_SOLVE_VALIDATION_METADATA_KIND,
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

#[cfg(any(feature = "cuda-probe", test))]
struct MixedSolveValidationSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明匹配冻结混合求解校验ABI。
unsafe impl KernelAbi for MixedSolveValidationSpec {
    const NAME: &'static str = "mixed_solve_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &MIXED_SOLVE_VALIDATION_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for MixedSolveValidationSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (metadata, rhs, factor, status) = mixed_solve_validation_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: rhs,
            },
            SlotRequirement {
                slot: 2,
                min_elements: factor,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        mixed_solve_validation_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn mixed_solve_validation_lengths(dims: &[u32; 4]) -> Result<(u64, u64, u64, u64), LauncherError> {
    let [n, stride, nr, worlds] = *dims;
    if n == 0 || stride == 0 || nr == 0 || worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("混合求解校验维度无效"));
    }
    let metadata = u64::from(n)
        .checked_mul(2)
        .and_then(|value| value.checked_add(4))
        .ok_or(LauncherError::Overflow("mixed_solve_validation_metadata"))?;
    let rhs = u64::from(n)
        .checked_mul(u64::from(nr))
        .and_then(|value| value.checked_mul(u64::from(worlds)))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("mixed_solve_validation_rhs"))?;
    let factor = u64::from(stride)
        .checked_mul(u64::from(worlds))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("mixed_solve_validation_factor"))?;
    let status = u64::from(worlds)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("mixed_solve_validation_status"))?;
    Ok((metadata, rhs, factor, status))
}

#[cfg(any(feature = "cuda-probe", test))]
fn mixed_solve_validation_plan(
    expected: [u32; 2],
    metadata_elements: u64,
    dims: &[u32; 4],
    bindings: &[SlotBinding; 4],
    proof: CheckedBinding<'_, [u32; 4]>,
) -> Result<LaunchDims, LauncherError> {
    if dims[..2] != expected {
        return Err(LauncherError::InvalidArray("混合求解校验静态维度不匹配"));
    }
    let (metadata_min, rhs, factor, status) = mixed_solve_validation_lengths(dims)?;
    if metadata_elements < metadata_min {
        return Err(LauncherError::InvalidArray("混合求解元数据跨度无效"));
    }
    for (slot, elements) in [(0, metadata_elements), (1, rhs), (2, factor), (3, status)] {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("mixed_solve_validation_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("混合求解校验跨度无效"));
        }
    }
    let plan = Kernel::<MixedSolveValidationSpec>::new()?.plan(dims, bindings, &[proof])?;
    Ok(plan.dims())
}

#[cfg(any(feature = "cuda-probe", test))]
const MIXED_SOLVE_FUSED_METADATA_KIND: ProofKind = ProofKind("mixed_solve_fused_metadata");

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MixedSolveFusedDims {
    abi: [u32; 4],
    metadata_elements: u64,
}

#[cfg(any(feature = "cuda-probe", test))]
struct StaticMixedSolveFusedMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for StaticMixedSolveFusedMetadata {
    type Dims = [u32; 2];

    const KIND: ProofKind = MIXED_SOLVE_FUSED_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        let [n, stride] = *dims;
        if n == 0
            || stride == 0
            || n > i32::MAX as u32
            || stride > i32::MAX as u32
            || values.len() > i32::MAX as usize
        {
            return Err(LauncherError::InvalidMetadata("混合融合维度无效"));
        }
        let n = usize::try_from(n).map_err(|_| LauncherError::Overflow("mixed_solve_fused_n"))?;
        let stride = usize::try_from(stride)
            .map_err(|_| LauncherError::Overflow("mixed_solve_fused_stride"))?;
        let header_end = 4usize;
        let flags_end = header_end
            .checked_add(n)
            .ok_or(LauncherError::Overflow("mixed_solve_fused_flags"))?;
        let diagonal_end = flags_end
            .checked_add(n)
            .ok_or(LauncherError::Overflow("mixed_solve_fused_diagonal"))?;
        if values.len() <= diagonal_end {
            return Err(LauncherError::InvalidMetadata("混合融合元数据过短"));
        }
        let levels = usize::try_from(values[0])
            .map_err(|_| LauncherError::InvalidMetadata("混合融合层数为负"))?;
        let prefix = usize::try_from(values[1])
            .map_err(|_| LauncherError::InvalidMetadata("混合融合前缀为负"))?;
        let nc = usize::try_from(values[2])
            .map_err(|_| LauncherError::InvalidMetadata("混合融合稀疏跨度为负"))?;
        let small_count = usize::try_from(values[3])
            .map_err(|_| LauncherError::InvalidMetadata("混合融合Small数量为负"))?;
        let expected_stride = prefix
            .checked_add(nc)
            .and_then(|value| value.checked_add(n))
            .ok_or(LauncherError::Overflow("mixed_solve_fused_stride"))?;
        if expected_stride != stride {
            return Err(LauncherError::InvalidMetadata("混合融合因子分区不符"));
        }
        let offsets_end = diagonal_end
            .checked_add(levels)
            .and_then(|value| value.checked_add(1))
            .ok_or(LauncherError::Overflow("mixed_solve_fused_offsets"))?;
        if values.len() < offsets_end {
            return Err(LauncherError::InvalidMetadata("混合融合层偏移越界"));
        }
        let offsets = &values[diagonal_end..offsets_end];
        if offsets.first() != Some(&0)
            || offsets
                .windows(2)
                .any(|pair| pair[0] < 0 || pair[0] >= pair[1])
        {
            return Err(LauncherError::InvalidMetadata("混合融合层偏移无效"));
        }
        let update_count = usize::try_from(*offsets.last().unwrap_or(&0))
            .map_err(|_| LauncherError::InvalidMetadata("混合融合更新数量为负"))?;
        if (levels == 0) != (update_count == 0) || update_count > i32::MAX as usize / 3 {
            return Err(LauncherError::InvalidMetadata("混合融合更新数量无效"));
        }
        let stored_updates = update_count.max(1);
        let updates_end = offsets_end
            .checked_add(
                stored_updates
                    .checked_mul(3)
                    .ok_or(LauncherError::Overflow("mixed_solve_fused_updates"))?,
            )
            .ok_or(LauncherError::Overflow("mixed_solve_fused_updates"))?;
        let expected_len = updates_end
            .checked_add(
                small_count
                    .checked_mul(3)
                    .ok_or(LauncherError::Overflow("mixed_solve_fused_small"))?,
            )
            .ok_or(LauncherError::Overflow("mixed_solve_fused_metadata"))?;
        if values.len() != expected_len {
            return Err(LauncherError::InvalidMetadata("混合融合元数据长度不符"));
        }
        if update_count == 0 && small_count != 0 {
            return Err(LauncherError::InvalidMetadata("混合融合占位更新偏移Small"));
        }

        let flags = &values[header_end..flags_end];
        for level in 0..levels {
            let first = usize::try_from(offsets[level])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合层起点为负"))?;
            let end = usize::try_from(offsets[level + 1])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合层终点为负"))?;
            if end > update_count {
                return Err(LauncherError::InvalidMetadata("混合融合层终点越界"));
            }
            let mut writes = vec![false; n];
            let mut reads = vec![false; n];
            for update in first..end {
                let base = offsets_end
                    .checked_add(
                        update
                            .checked_mul(3)
                            .ok_or(LauncherError::Overflow("mixed_solve_fused_update_index"))?,
                    )
                    .ok_or(LauncherError::Overflow("mixed_solve_fused_update_index"))?;
                let i = usize::try_from(values[base])
                    .map_err(|_| LauncherError::InvalidMetadata("混合融合写索引为负"))?;
                let k = usize::try_from(values[base + 1])
                    .map_err(|_| LauncherError::InvalidMetadata("混合融合读索引为负"))?;
                let address = usize::try_from(values[base + 2])
                    .map_err(|_| LauncherError::InvalidMetadata("混合融合L索引为负"))?;
                if i >= n || k >= n || address >= nc {
                    return Err(LauncherError::InvalidMetadata("混合融合更新索引越界"));
                }
                if i >= k || flags[i] != -1 || flags[k] != -1 {
                    return Err(LauncherError::InvalidMetadata("混合融合更新路由无效"));
                }
                writes[i] = true;
                reads[k] = true;
            }
            if writes.iter().zip(reads).any(|(write, read)| *write && read) {
                return Err(LauncherError::InvalidMetadata("混合融合同层读写交叉"));
            }
        }
        if !flags.contains(&-1) {
            return Err(LauncherError::InvalidMetadata("混合融合缺少Sparse尾部"));
        }
        let mut small_owned = vec![false; n];
        let mut prefix_ranges = Vec::with_capacity(small_count);
        for block in 0..small_count {
            let base = updates_end
                .checked_add(
                    block
                        .checked_mul(3)
                        .ok_or(LauncherError::Overflow("mixed_solve_fused_small_index"))?,
                )
                .ok_or(LauncherError::Overflow("mixed_solve_fused_small_index"))?;
            let start = usize::try_from(values[base])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合Small起点为负"))?;
            let size = usize::try_from(values[base + 1])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合Small大小为负"))?;
            let address = usize::try_from(values[base + 2])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合Small地址为负"))?;
            if !(2..=6).contains(&size) {
                return Err(LauncherError::InvalidMetadata("混合融合Small大小无效"));
            }
            let end = start
                .checked_add(size)
                .filter(|&value| value <= n)
                .ok_or(LauncherError::InvalidMetadata("混合融合Small区越界"))?;
            let address_end = address
                .checked_add(
                    size.checked_mul(size)
                        .ok_or(LauncherError::Overflow("mixed_solve_fused_small_square"))?,
                )
                .filter(|&value| value <= prefix)
                .ok_or(LauncherError::InvalidMetadata("混合融合Small因子越界"))?;
            for dof in start..end {
                if small_owned[dof] || flags[dof] != values[base + 2] {
                    return Err(LauncherError::InvalidMetadata("混合融合Small分区冲突"));
                }
                small_owned[dof] = true;
            }
            prefix_ranges.push((address, address_end));
        }
        prefix_ranges.sort_unstable();
        let mut next_address = 0usize;
        for (start, end) in prefix_ranges {
            if start != next_address {
                return Err(LauncherError::InvalidMetadata("混合融合Small前缀不连续"));
            }
            next_address = end;
        }
        if next_address != prefix {
            return Err(LauncherError::InvalidMetadata("混合融合Small前缀未覆盖"));
        }
        for (dof, &flag) in flags.iter().enumerate() {
            if small_owned[dof] {
                if flag < 0 {
                    return Err(LauncherError::InvalidMetadata("混合融合Small标记无效"));
                }
            } else if flag != -1 && flag != -2 {
                return Err(LauncherError::InvalidMetadata("混合融合路由标记无效"));
            }
        }
        Ok(())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct MixedSolveFusedLaunchMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for MixedSolveFusedLaunchMetadata {
    type Dims = MixedSolveFusedDims;

    const KIND: ProofKind = MIXED_SOLVE_FUSED_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        mixed_solve_fused_lengths(dims)?;
        if u64::try_from(values.len()).ok() != Some(dims.metadata_elements) {
            return Err(LauncherError::InvalidMetadata("混合融合快照长度不符"));
        }
        StaticMixedSolveFusedMetadata::validate(values, &[dims.abi[0], dims.abi[1]])
    }
}

#[cfg(any(feature = "cuda-probe", test))]
const MIXED_SOLVE_FUSED_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: MIXED_SOLVE_FUSED_METADATA_KIND,
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

#[cfg(any(feature = "cuda-probe", test))]
struct MixedSolveFusedSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明匹配冻结混合融合求解ABI。
unsafe impl KernelAbi for MixedSolveFusedSpec {
    const NAME: &'static str = "mixed_solve_fused";
    const BLOCK: BlockRule = BlockRule::Fixed(128);
    const SLOTS: &'static [SlotDesc] = &MIXED_SOLVE_FUSED_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for MixedSolveFusedSpec {
    type Dims = MixedSolveFusedDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (metadata, rhs, factor, output) = mixed_solve_fused_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: rhs,
            },
            SlotRequirement {
                slot: 2,
                min_elements: factor,
            },
            SlotRequirement {
                slot: 3,
                min_elements: output,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        let groups = mixed_solve_fused_groups(dims)?;
        LaunchDims::new((groups, 1, 1), (128, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn mixed_solve_fused_groups(dims: &MixedSolveFusedDims) -> Result<u32, LauncherError> {
    let [n, stride, nr, worlds] = dims.abi;
    if [n, stride, nr, worlds].contains(&0)
        || [n, stride, nr, worlds]
            .iter()
            .any(|&value| value > i32::MAX as u32)
        || stride
            < n.checked_mul(2)
                .ok_or(LauncherError::Overflow("mixed_solve_fused_stride"))?
    {
        return Err(LauncherError::InvalidArray("混合融合维度无效"));
    }
    let groups = worlds
        .checked_mul(nr)
        .filter(|&value| value <= i32::MAX as u32)
        .ok_or(LauncherError::Overflow("mixed_solve_fused_groups"))?;
    Ok(groups)
}

#[cfg(any(feature = "cuda-probe", test))]
fn mixed_solve_fused_lengths(
    dims: &MixedSolveFusedDims,
) -> Result<(u64, u64, u64, u64), LauncherError> {
    let [n, stride, _, worlds] = dims.abi;
    let groups = mixed_solve_fused_groups(dims)?;
    if dims.metadata_elements == 0 || dims.metadata_elements > i32::MAX as u64 {
        return Err(LauncherError::InvalidArray("混合融合元数据跨度无效"));
    }
    let rhs = u64::from(groups)
        .checked_mul(u64::from(n))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("mixed_solve_fused_rhs"))?;
    let factor = u64::from(worlds)
        .checked_mul(u64::from(stride))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("mixed_solve_fused_factor"))?;
    Ok((dims.metadata_elements, rhs, factor, rhs))
}

#[cfg(any(feature = "cuda-probe", test))]
fn mixed_solve_fused_plan(
    expected: [u32; 2],
    dims: &MixedSolveFusedDims,
    bindings: &[SlotBinding; 4],
    proof: CheckedBinding<'_, MixedSolveFusedDims>,
) -> Result<LaunchDims, LauncherError> {
    if dims.abi[..2] != expected {
        return Err(LauncherError::InvalidArray("混合融合静态维度不匹配"));
    }
    let lengths = mixed_solve_fused_lengths(dims)?;
    for (slot, elements) in [lengths.0, lengths.1, lengths.2, lengths.3]
        .into_iter()
        .enumerate()
    {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("mixed_solve_fused_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("混合融合跨度不符"));
        }
    }
    let plan = Kernel::<MixedSolveFusedSpec>::new()?.plan(dims, bindings, &[proof])?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
fn mixed_solve_validation_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::InvalidMetadata(_)
        | LauncherError::StaleMetadata
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "mixed_solve_validation_metadata",
            index: 0,
            reason: "invalid_or_stale_proof",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mixed_solve_validation_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mixed_solve_validation_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
fn mixed_solve_fused_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::InvalidMetadata(_)
        | LauncherError::StaleMetadata
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "mixed_solve_fused_metadata",
            index: 0,
            reason: "invalid_or_stale_proof",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mixed_solve_fused_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mixed_solve_fused_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
fn mixed_solve_validation_dimension(
    value: usize,
    field: &'static str,
) -> Result<u32, TransferError> {
    u32::try_from(value).map_err(|_| InputError::Overflow { field }.into())
}

#[cfg(feature = "cuda-probe")]
struct MixedSolveValidationMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    n: u32,
    stride: u32,
    revision: MetadataRevision,
    checked: Checked<StaticMixedSolveValidationMetadata>,
    fused_checked: Checked<StaticMixedSolveFusedMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl MixedSolveValidationMetadataBuffer {
    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        n: usize,
        stride: usize,
    ) -> Result<Self, TransferError> {
        let n = mixed_solve_validation_dimension(n, "mixed_solve_validation_n")?;
        let stride = mixed_solve_validation_dimension(stride, "mixed_solve_validation_stride")?;
        StaticMixedSolveValidationMetadata::validate(&values, &[n, stride])
            .map_err(mixed_solve_validation_launch_error)?;
        StaticMixedSolveFusedMetadata::validate(&values, &[n, stride])
            .map_err(mixed_solve_fused_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且跨度来自同一缓冲。
        let checked = unsafe {
            Checked::<StaticMixedSolveValidationMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                [n, stride],
                revision.clone(),
            )
        }
        .map_err(mixed_solve_validation_launch_error)?;
        // SAFETY: 同一上传快照覆盖融合入口全部索引。
        let fused_checked = unsafe {
            Checked::<StaticMixedSolveFusedMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                [n, stride],
                revision.clone(),
            )
        }
        .map_err(mixed_solve_fused_launch_error)?;
        Ok(Self {
            buffer,
            values,
            n,
            stride,
            revision,
            checked,
            fused_checked,
        })
    }

    fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    fn element_count(&self) -> Result<u64, TransferError> {
        u64::try_from(self.values.len()).map_err(|_| {
            InputError::Overflow {
                field: "mixed_solve_validation_metadata",
            }
            .into()
        })
    }

    fn proof(
        &self,
        dims: [u32; 4],
    ) -> Result<Checked<MixedSolveValidationLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &[self.n, self.stride])
            .map_err(mixed_solve_validation_launch_error)?;
        let expected_bytes = self
            .element_count()?
            .checked_mul(4)
            .ok_or(InputError::Overflow {
                field: "mixed_solve_validation_metadata_bytes",
            })?;
        if span.bytes() != expected_bytes {
            return Err(InputError::InvalidDimension {
                field: "mixed_solve_validation_metadata_span",
            }
            .into());
        }
        // SAFETY: 独占包装保留宿主快照和写入代际。
        unsafe {
            Checked::<MixedSolveValidationLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(mixed_solve_validation_launch_error)
    }

    fn fused_proof(
        &self,
        abi: [u32; 4],
    ) -> Result<Checked<MixedSolveFusedLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.fused_checked
            .binding(0)
            .validate_for(span, &[self.n, self.stride])
            .map_err(mixed_solve_fused_launch_error)?;
        let metadata_elements = self.element_count()?;
        let expected_bytes = metadata_elements
            .checked_mul(4)
            .ok_or(InputError::Overflow {
                field: "mixed_solve_fused_metadata_bytes",
            })?;
        if span.bytes() != expected_bytes {
            return Err(InputError::InvalidDimension {
                field: "mixed_solve_fused_metadata_span",
            }
            .into());
        }
        let dims = MixedSolveFusedDims {
            abi,
            metadata_elements,
        };
        // SAFETY: 独占包装保留完整快照和写入代际。
        unsafe {
            Checked::<MixedSolveFusedLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(mixed_solve_fused_launch_error)
    }

    #[cfg(test)]
    fn write_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入同步完成且跨度未变。
        self.checked = unsafe {
            Checked::<StaticMixedSolveValidationMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.n, self.stride],
                self.revision.clone(),
            )
        }
        .map_err(mixed_solve_validation_launch_error)?;
        // SAFETY: 恢复写入同步完成且融合跨度未变。
        self.fused_checked = unsafe {
            Checked::<StaticMixedSolveFusedMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                [self.n, self.stride],
                self.revision.clone(),
            )
        }
        .map_err(mixed_solve_fused_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
struct MixedSolveValidationKernel {
    kernel: SynchronousKernel,
    n: u32,
    stride: u32,
}

#[cfg(feature = "cuda-probe")]
impl MixedSolveValidationKernel {
    fn compile(
        session: &TransferSession,
        metadata: &MixedSolveValidationMetadataBuffer,
    ) -> Result<Self, TransferError> {
        mixed_solve_validation_lengths(&[metadata.n, metadata.stride, 1, 1])
            .map_err(mixed_solve_validation_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "mixed_solve_validate")?,
            n: metadata.n,
            stride: metadata.stride,
        })
    }

    #[cfg(test)]
    fn dimensions(&self, rhs_count: usize, worlds: usize) -> Result<[u32; 4], TransferError> {
        let nr = mixed_solve_validation_dimension(rhs_count, "mixed_solve_validation_rhs_count")?;
        let worlds = mixed_solve_validation_dimension(worlds, "mixed_solve_validation_worlds")?;
        let dims = [self.n, self.stride, nr, worlds];
        mixed_solve_validation_lengths(&dims).map_err(mixed_solve_validation_launch_error)?;
        Ok(dims)
    }

    fn launch(
        &self,
        metadata: &MixedSolveValidationMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: rhs.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: factor.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        mixed_solve_validation_plan(
            [self.n, self.stride],
            metadata.element_count()?,
            &dims,
            &bindings,
            proof.binding(0),
        )
        .map_err(mixed_solve_validation_launch_error)?;
        // SAFETY: 类型化计划核对全部跨度和元数据索引。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), rhs, factor, status, dims)
        }
    }
}

#[cfg(feature = "cuda-probe")]
struct MixedSolveFusedKernel {
    kernel: SynchronousKernel,
    n: u32,
    stride: u32,
    metadata_elements: u64,
}

#[cfg(feature = "cuda-probe")]
impl MixedSolveFusedKernel {
    fn compile(
        session: &TransferSession,
        metadata: &MixedSolveValidationMetadataBuffer,
    ) -> Result<Self, TransferError> {
        let dims = MixedSolveFusedDims {
            abi: [metadata.n, metadata.stride, 1, 1],
            metadata_elements: metadata.element_count()?,
        };
        mixed_solve_fused_lengths(&dims).map_err(mixed_solve_fused_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "mixed_solve_fused")?,
            n: metadata.n,
            stride: metadata.stride,
            metadata_elements: metadata.element_count()?,
        })
    }

    fn dimensions(&self, rhs_count: usize, worlds: usize) -> Result<[u32; 4], TransferError> {
        let nr = mixed_solve_validation_dimension(rhs_count, "mixed_solve_fused_rhs_count")?;
        let worlds = mixed_solve_validation_dimension(worlds, "mixed_solve_fused_worlds")?;
        let abi = [self.n, self.stride, nr, worlds];
        mixed_solve_fused_lengths(&MixedSolveFusedDims {
            abi,
            metadata_elements: self.metadata_elements,
        })
        .map_err(mixed_solve_fused_launch_error)?;
        Ok(abi)
    }

    fn launch(
        &self,
        metadata: &MixedSolveValidationMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        abi: [u32; 4],
    ) -> Result<(), TransferError> {
        let proof = metadata.fused_proof(abi)?;
        let dims = *proof.dims();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: rhs.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: factor.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        mixed_solve_fused_plan([self.n, self.stride], &dims, &bindings, proof.binding(0))
            .map_err(mixed_solve_fused_launch_error)?;
        // SAFETY: 类型化计划证明完整元数据、跨度和128线程分组。
        unsafe {
            self.kernel
                .launch_mixed_solve(metadata.as_buffer(), rhs, factor, output, abi)
        }
    }
}

pub struct SparseMassMixedSolvePlan {
    mass: SparseMassMixedFactorPlan,
    #[cfg(feature = "cuda-probe")]
    metadata: MixedSolveValidationMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validation: MixedSolveValidationKernel,
    #[cfg(feature = "cuda-probe")]
    solve: MixedSolveFusedKernel,
}
pub struct SparseMassMixedSolveData {
    mass: SparseMassMixedFactorData,
    layout: BatchLayout,
    rhs_count: usize,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    rhs: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassMixedSolveOutput {
    factor: SparseMassMixedFactorOutput,
    layout: BatchLayout,
    rhs_count: usize,
    rhs: Vec<f32>,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassMixedSolveWorld<'a> {
    pub factor: SparseMassMixedFactorWorld<'a>,
    /// 每世界按RHS、DOF连续排列。
    pub rhs: &'a [f32],
    pub solution: &'a [f32],
}
impl SparseMassMixedSolveOutput {
    pub fn input(&self) -> &SparseMassInput {
        self.factor.input()
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    pub fn world(&self, world: usize) -> Result<SparseMassMixedSolveWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(SparseMassMixedSolveWorld {
            factor: self.factor.world(world)?,
            rhs: &self.rhs[4 + r.start..4 + r.end],
            solution: &self.values[4 + r.start..4 + r.end],
        })
    }
}
fn layout(worlds: usize, nv: usize, nr: usize) -> Result<BatchLayout, InputError> {
    let invalid = || InputError::InvalidDimension {
        field: "mixed_solve_layout",
    };
    if nv == 0
        || nr == 0
        || nr > i32::MAX as usize
        || worlds == 0
        || worlds.checked_mul(nr).is_none_or(|v| v > i32::MAX as usize)
    {
        return Err(invalid());
    }
    let layout = BatchLayout::new(worlds, nv.checked_mul(nr).ok_or_else(invalid)?, 4)?;
    BatchLayout::new(
        1,
        layout.total_elements().checked_add(8).ok_or_else(invalid)?,
        4,
    )?;
    Ok(layout)
}
#[cfg(any(feature = "cuda-probe", test))]
fn metadata(input: &SparseMassInput) -> Result<Vec<i32>, TransferError> {
    let n = input.nv();
    let offsets = input.qld_level_offsets();
    let small: Vec<_> = input
        .blocks()
        .iter()
        .filter(|b| b.route == SparseMassRoute::SmallCholesky)
        .collect();
    let length = 4usize
        .checked_add(2 * n)
        .and_then(|v| v.checked_add(offsets.len()))
        .and_then(|v| v.checked_add(3 * input.qld_all_updates().len()))
        .and_then(|v| v.checked_add(3 * small.len()))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "mixed_solve_metadata",
        })?;
    let mut packed = crate::runtime::host_staging::<i32>(length)?;
    packed.clear();
    packed.extend([
        (offsets.len() - 1) as i32,
        input.qld_block_total() as i32,
        input.nc() as i32,
        small.len() as i32,
    ]);
    packed.extend(input.qld_block_adr());
    let f = input.fields();
    let mut diagonal = vec![0i32; n];
    for block in input.blocks() {
        for i in 0..block.size {
            let dof = block.start + i;
            diagonal[dof] = if block.route == SparseMassRoute::SmallCholesky {
                input.qld_block_adr()[dof] + (i * block.size + i) as i32
            } else {
                (input.qld_block_total() + f.m_rowadr[dof] as usize + f.m_rownnz[dof] as usize - 1)
                    as i32
            };
        }
    }
    packed.extend(diagonal);
    packed.extend(offsets);
    packed.extend(input.qld_all_updates().iter().flatten());
    for block in small {
        packed.extend([
            block.start as i32,
            block.size as i32,
            input.qld_block_adr()[block.start],
        ]);
    }
    Ok(packed)
}
impl SparseMassMixedSolvePlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        let mass = SparseMassMixedFactorPlan::new(session, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = mass;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let stride = mass
                .input()
                .qld_block_total()
                .checked_add(mass.input().nc())
                .and_then(|value| value.checked_add(mass.input().nv()))
                .ok_or(InputError::Overflow {
                    field: "mixed_solve_validation_stride",
                })?;
            let metadata = MixedSolveValidationMetadataBuffer::upload(
                session,
                metadata(mass.input())?,
                mass.input().nv(),
                stride,
            )?;
            let validation = MixedSolveValidationKernel::compile(session, &metadata)?;
            let solve = MixedSolveFusedKernel::compile(session, &metadata)?;
            Ok(Self {
                mass,
                metadata,
                validation,
                solve,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.mass.input()
    }
    pub fn create_data(
        &self,
        worlds: usize,
        rhs_count: usize,
    ) -> Result<SparseMassMixedSolveData, TransferError> {
        let layout = layout(worlds, self.input().nv(), rhs_count)?;
        let mass = self.mass.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (mass, layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, initial: f32| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(initial);
                self.mass.session.upload(&values)
            };
            Ok(SparseMassMixedSolveData {
                mass,
                layout,
                rhs_count,
                ready: false,
                rhs: allocate(layout.total_elements(), 0.0)?,
                output: allocate(layout.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.update_gather(data)?;
        self.update_factor(data)?;
        self.update_solve(data)
    }
    pub fn update_gather(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_gather(&mut data.mass)
    }
    pub fn update_factor(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_factor(&mut data.mass)
    }
    pub fn update_solve(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        if !data.mass.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let dimensions = self.solve.dimensions(data.rhs_count, data.worlds())?;
            self.validation.launch(
                &self.metadata,
                &data.rhs,
                &data.mass.output,
                &mut data.status,
                dimensions,
            )?;
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "mixed_solve_status")?;
            for (world, &value) in status[4..4 + data.worlds()].iter().enumerate() {
                if value != 0.0 {
                    if value == 1.0 || value == 3.0 {
                        data.mass.ready = false;
                    }
                    return Err(InputError::InvalidTopology {
                        field: "mixed_solve_sources",
                        index: world,
                        reason: "invalid_cached_factor_or_rhs",
                    }
                    .into());
                }
            }
            self.solve.launch(
                &self.metadata,
                &data.rhs,
                &data.mass.output,
                &mut data.output,
                dimensions,
            )?;
            data.checked_solution()?;
            data.ready = true;
            Ok(())
        }
    }
}
impl SparseMassMixedSolveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    fn after_matrix(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.mass.write_matrix(values);
        self.after_matrix(result)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let result = self.mass.write_world_matrix(world, values);
        self.after_matrix(result)
    }
    fn check_rhs(values: &[f32], expected: usize) -> Result<(), InputError> {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "mixed_solve_rhs",
                expected,
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "mixed_solve_rhs",
                    index,
                });
            }
        }
        Ok(())
    }
    pub fn write_rhs(&mut self, values: &[f32]) -> Result<(), TransferError> {
        Self::check_rhs(values, self.layout.total_elements())?;
        self.write(0, values)
    }
    pub fn write_world_rhs(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let r = self.layout.world_elements(world)?;
        Self::check_rhs(values, r.len())?;
        self.write(r.start, values)
    }
    fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.rhs.write_range(4 + offset, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_solution(&self) -> Result<Vec<f32>, TransferError> {
        let mut values = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut values)?;
        crate::physics::check_device_values(&values, "mixed_solve_solution")?;
        Ok(values)
    }
    #[cfg(feature = "cuda-probe")]
    fn cached_factor(&self) -> Result<SparseMassMixedFactorOutput, TransferError> {
        if !self.mass.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve_factor",
            });
        }
        let values = self.mass.checked_values()?;
        let input = &self.mass.sparse.input;
        let f = input.fields();
        for world in 0..self.worlds() {
            let first = 4 + self.mass.layout.world_elements(world)?.start + input.qld_block_total();
            for block in input
                .blocks()
                .iter()
                .filter(|b| b.route != SparseMassRoute::SmallCholesky)
            {
                for dof in block.start..block.start + block.size {
                    let value = values[first + (f.m_rowadr[dof] + f.m_rownnz[dof] - 1) as usize];
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof,
                            value: f64::from(value),
                        });
                    }
                }
            }
        }
        Ok(SparseMassMixedFactorOutput {
            input: input.clone(),
            layout: self.mass.layout,
            values,
        })
    }
    pub fn readback(&self) -> Result<SparseMassMixedSolveOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let factor = self.cached_factor()?;
            let mut rhs = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
            self.rhs.read_range_into(0, &mut rhs)?;
            crate::physics::check_device_values(&rhs, "mixed_solve_rhs")?;
            Ok(SparseMassMixedSolveOutput {
                factor,
                layout: self.layout,
                rhs_count: self.rhs_count,
                rhs,
                values: self.checked_solution()?,
            })
        }
    }
}

// Frozen smooth.py3314–3553; Apache-2.0, Copyright2025 The Newton Developers.
// Private ABI adds RHS grouping, not a different scalar or sparse algorithm.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
__device__ float mixed_schur(float value,float a,float b) {
 float result;asm("{ .reg .f32 product; mul.f32 product,%2,%3; sub.f32 %0,%1,product; }"
 : "=f"(result):"f"(value),"f"(a),"f"(b));return result;
}
template<int N> __device__ void mixed_small_solve(const float* F,const float* b,float* x,int start,int address) {
 for(int i=0;i<N;++i) {
  float value=b[start+i];for(int k=0;k<i;++k)value=mixed_schur(value,F[address+k*N+i],x[start+k]);
  x[start+i]=__fdiv_rn(value,F[address+i*N+i]);
 }
 for(int i=N-1;i>=0;--i) {
  float value=x[start+i];for(int k=i+1;k<N;++k)value=mixed_schur(value,F[address+i*N+k],x[start+k]);
  x[start+i]=__fdiv_rn(value,F[address+i*N+i]);
 }
}
extern "C" __global__ void mixed_solve_validate(const int* meta,const float* rhs,const float* factor,
 float* status,unsigned n,unsigned stride,unsigned nr,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long rs=1ull*n*nr;bool f=true,r=true;
 for(unsigned j=0;j<4;++j) {
  f=f && factor[j]==-131072.0f && factor[4+1ull*stride*worlds+j]==-131072.0f;
  r=r && rhs[j]==-131072.0f && rhs[4+rs*worlds+j]==-131072.0f;
 }
 const float* F=factor+4+1ull*w*stride;const int* diag=meta+4+n;
 for(unsigned j=0;j<stride;++j)f=f && isfinite(F[j]);
 for(unsigned i=0;i<n;++i)f=f && F[diag[i]]>0.0f && F[meta[1]+meta[2]+i]>0.0f;
 const float* b=rhs+4+rs*w;for(unsigned long long j=0;j<rs;++j)r=r && isfinite(b[j]);
 status[4+w]=(f?0.0f:1.0f)+(r?0.0f:2.0f);
}
extern "C" __global__ void mixed_solve_fused(const int* meta,const float* rhs,const float* factor,
 float* output,unsigned n,unsigned stride,unsigned nr,unsigned worlds) {
 unsigned group=blockIdx.x,w=group/nr,tid=threadIdx.x;
 const int* flags=meta+4;const int* offsets=flags+2*n;const int* updates=offsets+meta[0]+1;
 const int* small=updates+3*offsets[meta[0]];
 const float* F=factor+4+1ull*w*stride;const float* L=F+meta[1];const float* D=L+meta[2];
 const float* b=rhs+4+1ull*group*n;float* x=output+4+1ull*group*n;
 for(unsigned i=tid;i<n;i+=128) {
  if(flags[i]==-1)x[i]=b[i];else if(flags[i]==-2)x[i]=__fmul_rn(D[i],b[i]);
 }
 for(unsigned block=tid;block<(unsigned)meta[3];block+=128) {
  int start=small[3*block],size=small[3*block+1],a=small[3*block+2];
  switch(size) {
   case 2:mixed_small_solve<2>(F,b,x,start,a);break;
   case 3:mixed_small_solve<3>(F,b,x,start,a);break;
   case 4:mixed_small_solve<4>(F,b,x,start,a);break;
   case 5:mixed_small_solve<5>(F,b,x,start,a);break;
   case 6:mixed_small_solve<6>(F,b,x,start,a);break;
  }
 }
 __syncthreads();
 for(int level=meta[0]-1;level>=0;--level) {
  for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
   int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];float value=__fmul_rn(L[a],x[k]),ignored;
   asm volatile("atom.global.add.f32 %0, [%1], %2;":"=f"(ignored):"l"(x+i),"f"(-value):"memory");
  }
  __syncthreads();
 }
 for(unsigned i=tid;i<n;i+=128)if(flags[i]==-1)x[i]=__fmul_rn(x[i],D[i]);
 __syncthreads();
 for(int level=0;level<meta[0];++level) {
  for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
   int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];float value=__fmul_rn(L[a],x[i]),ignored;
   asm volatile("atom.global.add.f32 %0, [%1], %2;":"=f"(ignored):"l"(x+k),"f"(-value):"memory");
  }
  __syncthreads();
 }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cuda-probe")]
    use crate as engine;
    #[cfg(feature = "cuda-probe")]
    mod samples {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/mixed_mass_solve.rs"
        ));
    }
    fn input() -> SparseMassInput {
        let refs: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/mixed-mass-solve/reference.json"
        )))
        .unwrap();
        let f = &refs["models"][0]["fields"];
        let ints = |key: &str| {
            f[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i32)
                .collect()
        };
        SparseMassInput::new(crate::model::SparseMassFields {
            dof_parentid: ints("dof_parentid"),
            m_rownnz: ints("m_rownnz"),
            m_rowadr: ints("m_rowadr"),
            m_colind: ints("m_colind"),
            tree_dofadr: ints("tree_dofadr"),
            tree_dofnum: ints("tree_dofnum"),
        })
        .unwrap()
    }
    fn validation_metadata() -> Vec<i32> {
        vec![0, 4, 3, 0, -1, -1, -1, 0, 3, 9]
    }
    fn fused_metadata() -> Vec<i32> {
        vec![
            1, 4, 2, 1, // 层数、Small前缀、CSR跨度、Small数量。
            -1, -1, 0, 0, // Sparse与Small路由。
            6, 7, 2, 3, // 校验入口使用的对角地址。
            0, 1, // 单层更新区间。
            0, 1, 0, // 同层写0、读1、L地址0。
            2, 2, 0, // Small从2开始，大小2，前缀地址0。
        ]
    }
    #[test]
    fn mixed_rhs_layout_rejects_zero_and_overflow_without_allocation() {
        for (w, n, r) in [
            (0, 86, 3),
            (1, 0, 3),
            (1, 86, 0),
            (usize::MAX, 86, 1),
            (1, usize::MAX, 3),
        ] {
            assert!(layout(w, n, r).is_err());
        }
        assert_eq!(layout(257, 86, 3).unwrap().total_elements(), 66306);
    }
    #[test]
    fn packed_route_diagonals_and_small_groups_keep_distinct_regions() {
        let input = input();
        let m = metadata(&input).unwrap();
        let n = input.nv();
        let flags = &m[4..4 + n];
        assert_eq!(&m[..4], &[64, 90, 2201, 5]);
        assert!(flags[..65].iter().all(|&v| v == -1));
        assert_eq!(flags[85], -2);
        let diag = &m[4 + n..4 + 2 * n];
        assert_eq!(diag[65], 0);
        assert_eq!(diag[66], 3);
        assert_eq!(diag[85], 2290);
        let offsets = &m[4 + 2 * n..4 + 2 * n + 65];
        let count = offsets[64] as usize;
        let updates = &m[4 + 2 * n + 65..4 + 2 * n + 65 + 3 * count];
        assert!(
            updates
                .as_chunks::<3>()
                .0
                .iter()
                .all(|u| u[0] < 65 && u[1] < 65)
        );
        assert_eq!(&m[m.len() - 15..m.len() - 12], &[65, 2, 0]);
    }
    #[test]
    fn mixed_solve_validation_metadata_proves_diag_and_inverse_regions() {
        let values = validation_metadata();
        assert!(StaticMixedSolveValidationMetadata::validate(&values, &[3, 10]).is_ok());
        assert!(StaticMixedSolveValidationMetadata::validate(&values[..9], &[3, 10]).is_err());

        let mut negative_diag = values.clone();
        negative_diag[7] = -1;
        assert!(StaticMixedSolveValidationMetadata::validate(&negative_diag, &[3, 10]).is_err());
        let mut large_diag = values.clone();
        large_diag[9] = 10;
        assert!(StaticMixedSolveValidationMetadata::validate(&large_diag, &[3, 10]).is_err());

        let mut signed_overflow = values.clone();
        signed_overflow[1] = i32::MAX;
        signed_overflow[2] = 1;
        assert!(StaticMixedSolveValidationMetadata::validate(&signed_overflow, &[3, 10]).is_err());
        let mut negative_inverse = values.clone();
        negative_inverse[1] = -5;
        assert!(StaticMixedSolveValidationMetadata::validate(&negative_inverse, &[3, 10]).is_err());
        let mut long_inverse = values;
        long_inverse[1] = 8;
        long_inverse[2] = 0;
        assert!(StaticMixedSolveValidationMetadata::validate(&long_inverse, &[3, 10]).is_err());
    }
    #[test]
    fn typed_mixed_solve_validation_plan_freezes_exact_contract() {
        use crate::runtime::launcher::DeviceSpan;

        let dims = [3, 10, 2, 257];
        let values = validation_metadata();
        let metadata_elements = values.len() as u64;
        let (metadata_min, rhs, factor, status) = mixed_solve_validation_lengths(&dims).unwrap();
        assert_eq!((metadata_min, rhs, factor, status), (10, 1550, 2578, 265));
        let metadata_span = DeviceSpan::new(7, 0x1000, metadata_elements * 4).unwrap();
        let revision = MetadataRevision::new();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof = unsafe {
            Checked::<MixedSolveValidationLaunchMetadata>::assume_uploaded(
                metadata_span,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata_span,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(7, 0x2000, rhs * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(7, 0x5000, factor * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(7, 0x9000, status * 4).unwrap(),
                elem: ElemType::F32,
            },
        ];
        let launch = mixed_solve_validation_plan(
            [3, 10],
            metadata_elements,
            &dims,
            &bindings,
            proof.binding(0),
        )
        .unwrap();
        assert_eq!(launch.grid(), (2, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));

        for (slot, elements) in [(0, metadata_elements), (1, rhs), (2, factor), (3, status)] {
            for bytes in [elements * 4 - 4, elements * 4 + 4] {
                let mut changed = bindings;
                changed[slot].span = DeviceSpan::new(7, changed[slot].span.addr(), bytes).unwrap();
                assert!(
                    mixed_solve_validation_plan(
                        [3, 10],
                        metadata_elements,
                        &dims,
                        &changed,
                        proof.binding(0),
                    )
                    .is_err()
                );
            }
        }

        let mut alias = bindings;
        alias[3].span = alias[1].span.view(0, status * 4).unwrap();
        assert!(matches!(
            mixed_solve_validation_plan(
                [3, 10],
                metadata_elements,
                &dims,
                &alias,
                proof.binding(0),
            ),
            Err(LauncherError::Alias {
                first: 1,
                second: 3,
            })
        ));
        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x5000, factor * 4).unwrap();
        assert!(matches!(
            mixed_solve_validation_plan(
                [3, 10],
                metadata_elements,
                &dims,
                &foreign,
                proof.binding(0),
            ),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));
        let mut replaced = bindings;
        replaced[0].span = DeviceSpan::new(7, 0xa000, metadata_elements * 4).unwrap();
        assert!(matches!(
            mixed_solve_validation_plan(
                [3, 10],
                metadata_elements,
                &dims,
                &replaced,
                proof.binding(0),
            ),
            Err(LauncherError::MetadataMismatch(_))
        ));
        assert!(
            mixed_solve_validation_plan([3, 10], u64::MAX, &dims, &bindings, proof.binding(0),)
                .is_err()
        );
        assert!(mixed_solve_validation_lengths(&[1, 1, 1, u32::MAX - 254]).is_err());
        assert!(matches!(
            mixed_solve_validation_lengths(&[u32::MAX, u32::MAX, u32::MAX, u32::MAX - 255,]),
            Err(LauncherError::Overflow(_))
        ));
        assert!(
            mixed_solve_validation_plan(
                [4, 10],
                metadata_elements,
                &dims,
                &bindings,
                proof.binding(0),
            )
            .is_err()
        );
        revision.invalidate();
        assert!(matches!(
            mixed_solve_validation_plan(
                [3, 10],
                metadata_elements,
                &dims,
                &bindings,
                proof.binding(0),
            ),
            Err(LauncherError::StaleMetadata)
        ));
    }
    #[test]
    fn mixed_solve_fused_metadata_proves_routes_levels_and_small_regions() {
        let values = fused_metadata();
        assert!(StaticMixedSolveFusedMetadata::validate(&values, &[4, 10]).is_ok());
        assert!(
            StaticMixedSolveFusedMetadata::validate(&values[..values.len() - 1], &[4, 10]).is_err()
        );
        let mut long = values.clone();
        long.push(0);
        assert!(StaticMixedSolveFusedMetadata::validate(&long, &[4, 10]).is_err());

        let crossed = vec![
            1, 0, 2, 0, -1, -1, -1, -1, 0, 0, 0, 0, 0, 2, 0, 1, 0, 1, 2, 1,
        ];
        // 两条更新各自满足顺序。
        // 它们在索引1发生交叉。
        assert!(StaticMixedSolveFusedMetadata::validate(&crossed, &[4, 6]).is_err());
        let mut bad_factor = values.clone();
        bad_factor[16] = 2;
        assert!(StaticMixedSolveFusedMetadata::validate(&bad_factor, &[4, 10]).is_err());
        let mut bad_small = values.clone();
        bad_small[18] = 3;
        assert!(StaticMixedSolveFusedMetadata::validate(&bad_small, &[4, 10]).is_err());
        let mut bad_route = values.clone();
        bad_route[5] = 0;
        assert!(StaticMixedSolveFusedMetadata::validate(&bad_route, &[4, 10]).is_err());
        let mut reversed = values.clone();
        reversed[14] = 1;
        reversed[15] = 0;
        assert!(StaticMixedSolveFusedMetadata::validate(&reversed, &[4, 10]).is_err());
        let mut compact_endpoint = values.clone();
        compact_endpoint[5] = -2;
        assert!(StaticMixedSolveFusedMetadata::validate(&compact_endpoint, &[4, 10]).is_err());

        let same_target = vec![
            1, 4, 2, 1, -1, -1, 0, 0, 6, 7, 2, 3, 0, 2, 0, 1, 0, 0, 1, 1, 2, 2, 0,
        ];
        assert!(StaticMixedSolveFusedMetadata::validate(&same_target, &[4, 10]).is_ok());

        let placeholder_only = vec![0, 0, 2, 0, -1, -2, 2, 3, 0, 0, 0, 0];
        assert!(StaticMixedSolveFusedMetadata::validate(&placeholder_only, &[2, 4]).is_ok());
        let placeholder_before_small =
            vec![0, 4, 2, 1, -1, -2, 0, 0, 6, 7, 2, 3, 0, 0, 0, 0, 2, 2, 0];
        assert!(
            StaticMixedSolveFusedMetadata::validate(&placeholder_before_small, &[4, 10]).is_err()
        );

        let input = input();
        let actual = metadata(&input).unwrap();
        let stride = input.qld_block_total() + input.nc() + input.nv();
        assert!(
            StaticMixedSolveFusedMetadata::validate(&actual, &[input.nv() as u32, stride as u32],)
                .is_ok()
        );
    }
    #[test]
    fn typed_mixed_solve_fused_plan_freezes_exact_contract() {
        use crate::runtime::launcher::DeviceSpan;

        let values = fused_metadata();
        let dims = MixedSolveFusedDims {
            abi: [4, 10, 129, 2],
            metadata_elements: values.len() as u64,
        };
        let (metadata_len, rhs, factor, output) = mixed_solve_fused_lengths(&dims).unwrap();
        assert_eq!((metadata_len, rhs, factor, output), (20, 1040, 28, 1040));
        let metadata_span = DeviceSpan::new(7, 0x1000, metadata_len * 4).unwrap();
        let revision = MetadataRevision::new();
        // SAFETY: 测试跨度代表同一份完整宿主快照。
        let proof = unsafe {
            Checked::<MixedSolveFusedLaunchMetadata>::assume_uploaded(
                metadata_span,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata_span,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(7, 0x2000, rhs * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(7, 0x4000, factor * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(7, 0x6000, output * 4).unwrap(),
                elem: ElemType::F32,
            },
        ];
        let launch = mixed_solve_fused_plan([4, 10], &dims, &bindings, proof.binding(0)).unwrap();
        assert_eq!(launch.grid(), (258, 1, 1));
        assert_eq!(launch.block(), (128, 1, 1));
        assert_eq!(launch.shared_bytes(), 0);

        for (slot, elements) in [metadata_len, rhs, factor, output].into_iter().enumerate() {
            for bytes in [elements * 4 - 4, elements * 4 + 4] {
                let mut changed = bindings;
                changed[slot].span = DeviceSpan::new(7, changed[slot].span.addr(), bytes).unwrap();
                assert!(
                    mixed_solve_fused_plan([4, 10], &dims, &changed, proof.binding(0)).is_err()
                );
            }
        }
        let mut alias = bindings;
        alias[3].span = alias[1].span;
        assert!(matches!(
            mixed_solve_fused_plan([4, 10], &dims, &alias, proof.binding(0)),
            Err(LauncherError::Alias {
                first: 1,
                second: 3
            })
        ));
        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x4000, factor * 4).unwrap();
        assert!(matches!(
            mixed_solve_fused_plan([4, 10], &dims, &foreign, proof.binding(0)),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));
        assert!(
            mixed_solve_fused_groups(&MixedSolveFusedDims {
                abi: [4, 10, 65536, 65536],
                metadata_elements: metadata_len,
            })
            .is_err()
        );
        assert!(
            mixed_solve_fused_groups(&MixedSolveFusedDims {
                abi: [4, 10, 1, i32::MAX as u32 + 1],
                metadata_elements: metadata_len,
            })
            .is_err()
        );
        revision.invalidate();
        assert!(matches!(
            mixed_solve_fused_plan([4, 10], &dims, &bindings, proof.binding(0)),
            Err(LauncherError::StaleMetadata)
        ));
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn gpu_mixed_solve_validation_wrapper_checks_tail_and_rejections() {
        let session = TransferSession::new(0).unwrap();
        let foreign = TransferSession::new(0).unwrap();
        let mut metadata =
            MixedSolveValidationMetadataBuffer::upload(&session, fused_metadata(), 4, 10).unwrap();
        let kernel = MixedSolveValidationKernel::compile(&session, &metadata).unwrap();
        let dims = kernel.dimensions(2, 257).unwrap();
        let (_, rhs_len, factor_len, status_len) = mixed_solve_validation_lengths(&dims).unwrap();
        let mut rhs_values = vec![-131072.0f32; rhs_len as usize];
        let rhs_end = rhs_values.len() - 4;
        rhs_values[4..rhs_end].fill(0.25);
        let mut factor_values = vec![-131072.0f32; factor_len as usize];
        let factor_end = factor_values.len() - 4;
        factor_values[4..factor_end].fill(1.0);
        let mut status_values = vec![-131072.0f32; status_len as usize];
        status_values[4..4 + 257].fill(f32::NAN);
        let rhs = session.upload(&rhs_values).unwrap();
        let factor = session.upload(&factor_values).unwrap();
        let mut status = session.upload(&status_values).unwrap();

        kernel
            .launch(&metadata, &rhs, &factor, &mut status, dims)
            .unwrap();
        let mut result = vec![0.0; status.len()];
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(&result[..4], &status_values[..4]);
        assert_eq!(
            &result[result.len() - 4..],
            &status_values[status_values.len() - 4..]
        );
        assert!(result[4..4 + 257].iter().all(|&value| value == 0.0));
        let stable = result.clone();

        metadata.write_for_test(7, 1).unwrap();
        assert!(
            kernel
                .launch(&metadata, &rhs, &factor, &mut status, dims)
                .is_err()
        );
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);
        metadata.restore_for_test().unwrap();
        kernel
            .launch(&metadata, &rhs, &factor, &mut status, dims)
            .unwrap();

        let short_rhs = session.upload(&rhs_values[..rhs_values.len() - 1]).unwrap();
        let mut long_rhs_values = rhs_values.clone();
        long_rhs_values.push(0.0);
        let long_rhs = session.upload(&long_rhs_values).unwrap();
        for candidate in [&short_rhs, &long_rhs] {
            assert!(
                kernel
                    .launch(&metadata, candidate, &factor, &mut status, dims)
                    .is_err()
            );
        }
        let short_factor = session
            .upload(&factor_values[..factor_values.len() - 1])
            .unwrap();
        let mut long_factor_values = factor_values.clone();
        long_factor_values.push(0.0);
        let long_factor = session.upload(&long_factor_values).unwrap();
        for candidate in [&short_factor, &long_factor] {
            assert!(
                kernel
                    .launch(&metadata, &rhs, candidate, &mut status, dims)
                    .is_err()
            );
        }
        let mut short_status = session
            .upload(&status_values[..status_values.len() - 1])
            .unwrap();
        let mut long_status_values = status_values.clone();
        long_status_values.push(-131072.0);
        let mut long_status = session.upload(&long_status_values).unwrap();
        for candidate in [&mut short_status, &mut long_status] {
            assert!(
                kernel
                    .launch(&metadata, &rhs, &factor, candidate, dims)
                    .is_err()
            );
        }
        let foreign_factor = foreign.upload(&factor_values).unwrap();
        assert!(matches!(
            kernel.launch(&metadata, &rhs, &foreign_factor, &mut status, dims),
            Err(TransferError::SessionMismatch)
        ));
        assert!(kernel.dimensions(usize::MAX, 257).is_err());
        assert!(kernel.dimensions(1, u32::MAX as usize - 254).is_err());
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn gpu_mixed_solve_fused_rejects_in_bounds_small_overlap_without_state_change() {
        let refs = samples::reference();
        let model = &refs["models"][0];
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassMixedSolvePlan::new(&session, samples::input(model)).unwrap();
        let mut data = plan.create_data(1, 129).unwrap();
        data.write_matrix(&samples::matrix(model, 0)).unwrap();
        data.write_rhs(&samples::rhs(model, 0, 129)).unwrap();
        plan.update(&mut data).unwrap();

        let f32_bits = |buffer: &TransferBuffer<f32>| {
            let mut values = vec![0.0; buffer.len()];
            buffer.read_range_into(0, &mut values).unwrap();
            values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        };
        let i32_bits = |buffer: &TransferBuffer<i32>| {
            let mut values = vec![0; buffer.len()];
            buffer.read_range_into(0, &mut values).unwrap();
            values
        };
        let metadata_before = i32_bits(plan.metadata.as_buffer());
        let rhs_before = f32_bits(&data.rhs);
        let factor_before = f32_bits(&data.mass.output);
        let output_before = f32_bits(&data.output);
        let status_before = f32_bits(&data.status);
        let ready_before = data.ready;
        let factor_ready_before = data.mass.ready;
        let proof_span = plan.metadata.fused_checked.span();
        let proof_dims = *plan.metadata.fused_checked.dims();
        let proof_digest = plan.metadata.fused_checked.digest();

        let mut overlap = plan.metadata.values.clone();
        let n = usize::try_from(plan.metadata.n).unwrap();
        let levels = usize::try_from(overlap[0]).unwrap();
        let diagonal_end = 4 + 2 * n;
        let offsets_end = diagonal_end + levels + 1;
        let update_count = usize::try_from(overlap[offsets_end - 1]).unwrap();
        let small_base = offsets_end + 3 * update_count.max(1);
        assert!(overlap[3] >= 2);
        let first = [
            overlap[small_base],
            overlap[small_base + 1],
            overlap[small_base + 2],
        ];
        overlap[small_base + 3..small_base + 6].copy_from_slice(&first);

        let start = usize::try_from(first[0]).unwrap();
        let size = usize::try_from(first[1]).unwrap();
        let address = usize::try_from(first[2]).unwrap();
        let prefix = usize::try_from(overlap[1]).unwrap();
        assert!((2..=6).contains(&size));
        assert!(start + size <= n);
        assert!(address + size * size <= prefix);
        assert_eq!(
            &overlap[small_base..small_base + 3],
            &overlap[small_base + 3..small_base + 6]
        );
        assert!(
            overlap[4 + start..4 + start + size]
                .iter()
                .all(|&flag| flag == first[2])
        );
        assert!(
            StaticMixedSolveValidationMetadata::validate(
                &overlap,
                &[plan.metadata.n, plan.metadata.stride],
            )
            .is_ok()
        );
        assert!(matches!(
            StaticMixedSolveFusedMetadata::validate(
                &overlap,
                &[plan.metadata.n, plan.metadata.stride],
            ),
            Err(LauncherError::InvalidMetadata("混合融合Small分区冲突"))
        ));

        let rejection = match MixedSolveValidationMetadataBuffer::upload(
            &session,
            overlap,
            n,
            usize::try_from(plan.metadata.stride).unwrap(),
        ) {
            Ok(_) => panic!("Small重叠元数据意外通过"),
            Err(error) => error,
        };
        assert!(matches!(
            rejection,
            TransferError::Input(InputError::InvalidTopology {
                field: "mixed_solve_fused_metadata",
                index: 0,
                reason: "invalid_or_stale_proof",
            })
        ));

        assert_eq!(
            plan.metadata.fused_checked.span().context(),
            proof_span.context()
        );
        assert_eq!(plan.metadata.fused_checked.span().addr(), proof_span.addr());
        assert_eq!(
            plan.metadata.fused_checked.span().bytes(),
            proof_span.bytes()
        );
        assert_eq!(*plan.metadata.fused_checked.dims(), proof_dims);
        assert_eq!(plan.metadata.fused_checked.digest(), proof_digest);
        assert_eq!(i32_bits(plan.metadata.as_buffer()), metadata_before);
        assert_eq!(f32_bits(&data.rhs), rhs_before);
        assert_eq!(f32_bits(&data.mass.output), factor_before);
        assert_eq!(f32_bits(&data.output), output_before);
        assert_eq!(f32_bits(&data.status), status_before);
        assert_eq!(data.ready, ready_before);
        assert_eq!(data.mass.ready, factor_ready_before);

        plan.update_solve(&mut data).unwrap();
        assert!(data.ready);
        let expected = samples::solution(model, 0, 129);
        let recovered = data.readback().unwrap();
        for (&value, &reference) in recovered.world(0).unwrap().solution.iter().zip(&expected) {
            assert!((f64::from(value) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
        }
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn gpu_mixed_solve_fused_wrapper_checks_groups_spans_and_recovery() {
        let refs = samples::reference();
        let model = &refs["models"][0];
        let session = TransferSession::new(0).unwrap();
        let foreign = TransferSession::new(0).unwrap();
        let mut plan = SparseMassMixedSolvePlan::new(&session, samples::input(model)).unwrap();
        let mut data = plan.create_data(1, 129).unwrap();
        data.write_matrix(&samples::matrix(model, 0)).unwrap();
        data.write_rhs(&samples::rhs(model, 0, 129)).unwrap();
        plan.update(&mut data).unwrap();
        let expected = samples::solution(model, 0, 129);
        let actual = data.readback().unwrap();
        for (&value, &reference) in actual.world(0).unwrap().solution.iter().zip(&expected) {
            assert!((f64::from(value) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
        }

        let bits = |buffer: &TransferBuffer<f32>| {
            let mut values = vec![0.0; buffer.len()];
            buffer.read_range_into(0, &mut values).unwrap();
            values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        };
        let stable = bits(&data.output);
        let dimensions = plan.solve.dimensions(129, 1).unwrap();
        data.ready = false;
        plan.metadata.write_for_test(0, -1).unwrap();
        assert!(
            plan.solve
                .launch(
                    &plan.metadata,
                    &data.rhs,
                    &data.mass.output,
                    &mut data.output,
                    dimensions,
                )
                .is_err()
        );
        assert!(!data.ready);
        assert!(data.readback().is_err());
        assert_eq!(bits(&data.output), stable);
        plan.metadata.restore_for_test().unwrap();

        let fused_dims = MixedSolveFusedDims {
            abi: dimensions,
            metadata_elements: plan.metadata.element_count().unwrap(),
        };
        let (_, rhs_len, factor_len, output_len) = mixed_solve_fused_lengths(&fused_dims).unwrap();
        let short_rhs = session.upload(&vec![0.0f32; rhs_len as usize - 1]).unwrap();
        let long_rhs = session.upload(&vec![0.0f32; rhs_len as usize + 1]).unwrap();
        for candidate in [&short_rhs, &long_rhs] {
            assert!(
                plan.solve
                    .launch(
                        &plan.metadata,
                        candidate,
                        &data.mass.output,
                        &mut data.output,
                        dimensions,
                    )
                    .is_err()
            );
        }
        let short_factor = session
            .upload(&vec![0.0f32; factor_len as usize - 1])
            .unwrap();
        let long_factor = session
            .upload(&vec![0.0f32; factor_len as usize + 1])
            .unwrap();
        for candidate in [&short_factor, &long_factor] {
            assert!(
                plan.solve
                    .launch(
                        &plan.metadata,
                        &data.rhs,
                        candidate,
                        &mut data.output,
                        dimensions,
                    )
                    .is_err()
            );
        }
        let mut short_output = session
            .upload(&vec![0.0f32; output_len as usize - 1])
            .unwrap();
        let mut long_output = session
            .upload(&vec![0.0f32; output_len as usize + 1])
            .unwrap();
        for candidate in [&mut short_output, &mut long_output] {
            assert!(
                plan.solve
                    .launch(
                        &plan.metadata,
                        &data.rhs,
                        &data.mass.output,
                        candidate,
                        dimensions,
                    )
                    .is_err()
            );
        }
        let foreign_factor = foreign.upload(&vec![0.0f32; factor_len as usize]).unwrap();
        assert!(matches!(
            plan.solve.launch(
                &plan.metadata,
                &data.rhs,
                &foreign_factor,
                &mut data.output,
                dimensions,
            ),
            Err(TransferError::SessionMismatch)
        ));
        assert!(plan.solve.dimensions(usize::MAX, 1).is_err());
        assert!(plan.solve.dimensions(65536, 65536).is_err());
        assert_eq!(bits(&data.output), stable);

        plan.update_solve(&mut data).unwrap();
        let recovered = data.readback().unwrap();
        for (&value, &reference) in recovered.world(0).unwrap().solution.iter().zip(&expected) {
            assert!((f64::from(value) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
        }
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new mixed consumer cached-only factor/source/guard failure purpose"]
    fn cached_merged_factor_ignores_old_sources_and_rejects_each_consumer_fault() {
        let refs = samples::reference();
        let model = &refs["models"][0];
        let session = TransferSession::new(0).unwrap();
        let mut plan = SparseMassMixedSolvePlan::new(&session, samples::input(model)).unwrap();
        let mut data = plan.create_data(2, 3).unwrap();
        data.write_matrix(&samples::matrix(model, 0).repeat(2))
            .unwrap();
        let rhs = samples::rhs(model, 0, 3);
        data.write_rhs(&rhs.repeat(2)).unwrap();
        plan.update(&mut data).unwrap();
        let before = data.readback().unwrap();
        let expected = samples::solution(model, 0, 3);
        let bits = |buffer: &TransferBuffer<f32>| {
            let mut values = vec![0.0; buffer.len()];
            buffer.read_range_into(0, &mut values).unwrap();
            values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        };
        let stable_solution = bits(&data.output);
        let stable_status = bits(&data.status);
        plan.metadata.write_for_test(0, -1).unwrap();
        data.write_rhs(&rhs.repeat(2)).unwrap();
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.mass.ready);
        assert!(data.readback().is_err());
        assert_eq!(bits(&data.output), stable_solution);
        assert_eq!(bits(&data.status), stable_status);
        plan.metadata.restore_for_test().unwrap();
        plan.update_solve(&mut data).unwrap();
        let mut factor = vec![0.0; data.mass.output.len()];
        data.mass.output.read_range_into(0, &mut factor).unwrap();
        // Both stale primary and producer CSR can be invalid; the published
        // merged factor alone owns this solve's lifetime and physics inputs.
        let mut a = [0.0];
        let mut b = [0.0];
        data.mass.matrix.read_range_into(4, &mut a).unwrap();
        data.mass.sparse.matrix.read_range_into(4, &mut b).unwrap();
        data.mass.matrix.write_range(4, &[f32::NAN]).unwrap();
        data.mass.sparse.matrix.write_range(4, &[f32::NAN]).unwrap();
        data.write_rhs(&rhs.repeat(2)).unwrap();
        plan.update_solve(&mut data).unwrap();
        let after = data.readback().unwrap();
        let mut unchanged = vec![0.0; factor.len()];
        data.mass.output.read_range_into(0, &mut unchanged).unwrap();
        assert_eq!(
            factor.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            unchanged.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        for w in 0..2 {
            for (&actual, &reference) in after.world(w).unwrap().solution.iter().zip(&expected) {
                assert!((f64::from(actual) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
            }
        }
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        assert!(plan.update_gather(&mut data).is_err());
        data.mass.matrix.write_range(4, &a).unwrap();
        data.mass.sparse.matrix.write_range(4, &b).unwrap();
        plan.update(&mut data).unwrap();
        // A poisoned old Small producer must not invalidate a completed merge.
        data.mass
            .small
            .test_factor_buffer_mut()
            .write_range(0, &[0.0])
            .unwrap();
        data.write_rhs(&rhs.repeat(2)).unwrap();
        plan.update_solve(&mut data).unwrap();
        assert!(data.readback().is_ok());
        data.mass
            .small
            .test_factor_buffer_mut()
            .write_range(0, &[-131072.0])
            .unwrap();
        let prefix = plan.input().qld_block_total();
        let inverse = 4 + prefix + plan.input().nc();
        for (offset, bad) in [
            (0, 0.0),
            (factor.len() - 4, 0.0),
            (4, -1.0),
            (5, f32::NAN),
            (4 + prefix, -1.0),
            (inverse, -1.0),
            (inverse + 65, f32::INFINITY),
        ] {
            data.mass.output.write_range(offset, &[bad]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(!data.mass.ready);
            data.mass
                .output
                .write_range(offset, &[factor[offset]])
                .unwrap();
            plan.update_factor(&mut data).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        let len = data.rhs.len();
        for (offset, bad) in [(0, 0.0), (len - 4, 0.0), (4, f32::NAN)] {
            let mut saved = [0.0];
            data.rhs.read_range_into(offset, &mut saved).unwrap();
            data.rhs.write_range(offset, &[bad]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.mass.ready);
            data.rhs.write_range(offset, &saved).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        for offset in [0, len - 4] {
            data.output.write_range(offset, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.mass.ready);
            data.output.write_range(offset, &[-131072.0]).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        data.output.write_range(4, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        plan.update_solve(&mut data).unwrap();
        for offset in [0, data.worlds() + 4] {
            data.status.write_range(offset, &[0.0]).unwrap();
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.mass.ready);
            assert!(data.readback().is_err());
            data.status.write_range(offset, &[-131072.0]).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        let mut raw = vec![0.0; len];
        data.rhs.read_range_into(0, &mut raw).unwrap();
        data.write_world_rhs(1, &samples::rhs(model, 1, 3)).unwrap();
        let mut next = vec![0.0; len];
        data.rhs.read_range_into(0, &mut next).unwrap();
        for i in (0..4 + 258).chain(4 + 516..len) {
            assert_eq!(raw[i].to_bits(), next[i].to_bits());
        }
        plan.update_solve(&mut data).unwrap();
        assert!(data.readback().is_ok());
        assert_eq!(before.rhs_count(), 3);
    }
}
