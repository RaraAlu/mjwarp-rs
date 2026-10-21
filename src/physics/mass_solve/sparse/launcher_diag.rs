//! 对角规格、长度与计划。

#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SPARSE_MASS_DIAG_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::SparseMassDiagDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::sparse_mass_update_regions;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const SPARSE_MASS_DIAG_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SPARSE_MASS_DIAG_METADATA_KIND,
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
pub(super) struct SparseMassDiagSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐项匹配冻结稀疏质量对角ABI。
unsafe impl KernelAbi for SparseMassDiagSpec {
    const NAME: &'static str = "sparse_mass_diag";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SPARSE_MASS_DIAG_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for SparseMassDiagSpec {
    type Dims = SparseMassDiagDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(sparse_mass_diag_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sparse_mass_diag_lengths(dims)?;
        LaunchDims::new((dims.launch[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn sparse_mass_diag_lengths(
    dims: &SparseMassDiagDims,
) -> Result<[u64; 4], LauncherError> {
    let [first, count, stride, work] = dims.launch;
    if first != 0 || count != 0 {
        return Err(LauncherError::InvalidArray("稀疏对角保留维度必须为零"));
    }
    let expected_stride = dims
        .nc
        .checked_add(dims.nv)
        .filter(|&value| value <= i32::MAX as u32)
        .ok_or(LauncherError::Overflow("sparse_mass_diag_stride"))?;
    let expected_work = dims
        .worlds
        .checked_mul(dims.nv)
        .ok_or(LauncherError::Overflow("sparse_mass_diag_work"))?;
    if stride != expected_stride
        || dims.worlds == 0
        || dims.nv == 0
        || work != expected_work
        || work > u32::MAX - 255
    {
        return Err(LauncherError::InvalidArray("稀疏对角维度无效"));
    }
    let (_, _, metadata_end) = sparse_mass_update_regions(dims.nv, dims.update_count)?;
    if usize::try_from(dims.metadata_elements).ok() != Some(metadata_end) {
        return Err(LauncherError::InvalidMetadata("稀疏对角元数据长度无效"));
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
            .ok_or(LauncherError::Overflow("sparse_mass_diag_matrix"))?,
        "sparse_mass_diag_matrix",
    )?;
    let output = guarded(
        worlds
            .checked_mul(u64::from(stride))
            .ok_or(LauncherError::Overflow("sparse_mass_diag_output"))?,
        "sparse_mass_diag_output",
    )?;
    Ok([u64::from(dims.metadata_elements), matrix, matrix, output])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn sparse_mass_diag_plan(
    dims: &SparseMassDiagDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, SparseMassDiagDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = sparse_mass_diag_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("sparse_mass_diag_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("稀疏对角跨度不符"));
        }
    }
    let plan = Kernel::<SparseMassDiagSpec>::new()?.plan(dims, bindings, proofs)?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_mass_diag_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "sparse_mass_diag_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "sparse_mass_diag_abi",
        }
        .into(),
    }
}
