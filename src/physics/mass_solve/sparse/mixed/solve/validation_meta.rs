//! 混合求解校验元数据与类型化计划。

#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const MIXED_SOLVE_VALIDATION_METADATA_KIND: ProofKind =
    ProofKind("mixed_solve_validation_metadata");

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct StaticMixedSolveValidationMetadata;

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
pub(super) struct MixedSolveValidationLaunchMetadata;

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
pub(super) const MIXED_SOLVE_VALIDATION_SLOTS: [SlotDesc; 8] = [
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
pub(super) struct MixedSolveValidationSpec;

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
pub(super) fn mixed_solve_validation_lengths(
    dims: &[u32; 4],
) -> Result<(u64, u64, u64, u64), LauncherError> {
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
pub(super) fn mixed_solve_validation_plan(
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
