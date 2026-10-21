//! 复制与更新规格、长度与计划。

#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SPARSE_MASS_COPY_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SPARSE_MASS_UPDATE_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassCopyDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassUpdateDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::sparse_mass_update_regions;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const SPARSE_MASS_COPY_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SPARSE_MASS_COPY_METADATA_KIND,
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
pub(super) struct SparseMassCopySpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐项匹配冻结稀疏质量复制ABI。
unsafe impl KernelAbi for SparseMassCopySpec {
    const NAME: &'static str = "sparse_mass_copy";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SPARSE_MASS_COPY_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for SparseMassCopySpec {
    type Dims = SparseMassCopyDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(sparse_mass_copy_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sparse_mass_copy_lengths(dims)?;
        LaunchDims::new((dims.launch[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn sparse_mass_copy_lengths(
    dims: &SparseMassCopyDims,
) -> Result<[u64; 4], LauncherError> {
    let [first, count, stride, work] = dims.launch;
    if first != 0 || count != 0 {
        return Err(LauncherError::InvalidArray("稀疏复制保留维度必须为零"));
    }
    let expected_stride = dims
        .nc
        .checked_add(dims.nv)
        .filter(|&value| value <= i32::MAX as u32)
        .ok_or(LauncherError::Overflow("sparse_mass_copy_stride"))?;
    if stride != expected_stride || work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("稀疏复制维度无效"));
    }
    if dims.metadata_elements < 2 {
        return Err(LauncherError::InvalidMetadata("稀疏复制元数据长度无效"));
    }
    let guarded = |elements: u64, field| {
        elements
            .checked_add(8)
            .ok_or(LauncherError::Overflow(field))
    };
    let worlds = u64::from(work);
    let matrix = guarded(
        worlds
            .checked_mul(u64::from(dims.nc))
            .ok_or(LauncherError::Overflow("sparse_mass_copy_matrix"))?,
        "sparse_mass_copy_matrix",
    )?;
    let output = guarded(
        worlds
            .checked_mul(u64::from(stride))
            .ok_or(LauncherError::Overflow("sparse_mass_copy_output"))?,
        "sparse_mass_copy_output",
    )?;
    Ok([u64::from(dims.metadata_elements), matrix, matrix, output])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn sparse_mass_copy_plan(
    dims: &SparseMassCopyDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, SparseMassCopyDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = sparse_mass_copy_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("sparse_mass_copy_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("稀疏复制跨度不符"));
        }
    }
    let plan = Kernel::<SparseMassCopySpec>::new()?.plan(dims, bindings, proofs)?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_mass_copy_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "sparse_mass_copy_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "sparse_mass_copy_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const SPARSE_MASS_UPDATE_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SPARSE_MASS_UPDATE_METADATA_KIND,
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
        access: Access::ReadWrite,
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
pub(super) struct SparseMassUpdateSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐项匹配冻结稀疏质量更新ABI。
unsafe impl KernelAbi for SparseMassUpdateSpec {
    const NAME: &'static str = "sparse_mass_update";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SPARSE_MASS_UPDATE_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for SparseMassUpdateSpec {
    type Dims = SparseMassUpdateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(sparse_mass_update_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sparse_mass_update_lengths(dims)?;
        LaunchDims::new((dims.launch[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn sparse_mass_update_lengths(
    dims: &SparseMassUpdateDims,
) -> Result<[u64; 4], LauncherError> {
    let [first, count, stride, work] = dims.launch;
    first
        .checked_add(count)
        .filter(|&value| count != 0 && value <= dims.update_count)
        .ok_or(LauncherError::InvalidArray("稀疏更新区间无效"))?;
    let expected_stride = dims
        .nc
        .checked_add(dims.nv)
        .filter(|&value| value <= i32::MAX as u32)
        .ok_or(LauncherError::Overflow("sparse_mass_update_stride"))?;
    let expected_work = dims
        .worlds
        .checked_mul(count)
        .ok_or(LauncherError::Overflow("sparse_mass_update_work"))?;
    if stride != expected_stride
        || dims.worlds == 0
        || work != expected_work
        || work == 0
        || work > u32::MAX - 255
    {
        return Err(LauncherError::InvalidArray("稀疏更新维度无效"));
    }
    let (_, _, metadata_end) = sparse_mass_update_regions(dims.nv, dims.update_count)?;
    if usize::try_from(dims.metadata_elements).ok() != Some(metadata_end) {
        return Err(LauncherError::InvalidMetadata("稀疏更新元数据长度无效"));
    }
    let guarded = |elements: u64, field| {
        elements
            .checked_add(8)
            .ok_or(LauncherError::Overflow(field))
    };
    let worlds = u64::from(dims.worlds);
    let matrix = guarded(
        worlds
            .checked_mul(u64::from(dims.nc))
            .ok_or(LauncherError::Overflow("sparse_mass_update_matrix"))?,
        "sparse_mass_update_matrix",
    )?;
    let output = guarded(
        worlds
            .checked_mul(u64::from(stride))
            .ok_or(LauncherError::Overflow("sparse_mass_update_output"))?,
        "sparse_mass_update_output",
    )?;
    Ok([u64::from(dims.metadata_elements), matrix, matrix, output])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn sparse_mass_update_plan(
    dims: &SparseMassUpdateDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, SparseMassUpdateDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = sparse_mass_update_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("sparse_mass_update_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("稀疏更新跨度不符"));
        }
    }
    let plan = Kernel::<SparseMassUpdateSpec>::new()?.plan(dims, bindings, proofs)?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_mass_update_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "sparse_mass_update_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "sparse_mass_update_abi",
        }
        .into(),
    }
}
