//! 混合求解融合槽位、规格与分组计划。

#[cfg(any(feature = "cuda-probe", test))]
use super::fused_meta::MIXED_SOLVE_FUSED_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::fused_meta::MixedSolveFusedDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const MIXED_SOLVE_FUSED_SLOTS: [SlotDesc; 8] = [
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
pub(super) struct MixedSolveFusedSpec;

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
pub(super) fn mixed_solve_fused_groups(dims: &MixedSolveFusedDims) -> Result<u32, LauncherError> {
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
pub(super) fn mixed_solve_fused_lengths(
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
pub(super) fn mixed_solve_fused_plan(
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
