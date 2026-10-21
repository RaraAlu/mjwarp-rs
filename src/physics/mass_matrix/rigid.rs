//! 刚体质量矩阵冻结ABI、元数据校验与发射计划。

#[cfg(feature = "cuda-probe")]
use crate::diagnostics::{InputError, TransferError};
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims,
    LauncherError, MetadataProof, ProofKind, SlotBinding, SlotDesc, SlotKind, SlotRequirement,
    SlotRole,
};

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const RIGID_MASS_MATRIX_METADATA_KIND: ProofKind =
    ProofKind("rigid_mass_matrix.topology.v1");

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const RIGID_MASS_MATRIX_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RIGID_MASS_MATRIX_METADATA_KIND,
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
    // 内核回读已写入的复合惯量。
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

/// 冻结ABI的四项标量：nb、nv、unused、nw。
#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RigidMassMatrixDims {
    pub(super) abi: [u32; 4],
}

/// 返回元数据、armature、质心state与output的精确元素数。
#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rigid_mass_matrix_lengths(
    dims: &RigidMassMatrixDims,
) -> Result<[u64; 4], LauncherError> {
    let [nbody, nv, unused, worlds] = dims.abi;
    if nbody == 0 || unused != 0 || worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("刚体质量矩阵维度无效"));
    }
    let nbody = u64::from(nbody);
    let nv = u64::from(nv);
    let worlds = u64::from(worlds);
    let metadata = nv
        .checked_mul(2)
        .and_then(|value| value.checked_add(nbody))
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_metadata"))?;
    if metadata > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_mass_matrix_metadata"));
    }
    // 内核以u32计算两种世界步长。
    let state_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_state"))?;
    if state_width > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_mass_matrix_state_width"));
    }
    let output_width = nbody
        .checked_mul(10)
        .and_then(|value| {
            nv.checked_mul(nv)
                .and_then(|square| value.checked_add(square))
        })
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_output"))?;
    if output_width > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_mass_matrix_output_width"));
    }
    let state = worlds
        .checked_mul(state_width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_state"))?;
    let output = worlds
        .checked_mul(output_width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_output"))?;
    // 零DOF保留真实单元素armature占位。
    Ok([metadata, nv.max(1), state, output])
}

/// 校验内核读取的三段拓扑索引。
#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn validate_rigid_mass_matrix_metadata(
    values: &[i32],
    dims: RigidMassMatrixDims,
) -> Result<(), LauncherError> {
    let [metadata, ..] = rigid_mass_matrix_lengths(&dims)?;
    if values.len() as u64 != metadata {
        return Err(LauncherError::InvalidMetadata("质量矩阵元数据长度无效"));
    }
    let nbody = dims.abi[0] as usize;
    let nv = dims.abi[1] as usize;
    let (parent, rest) = values.split_at(nbody);
    let (body, dof_parent) = rest.split_at(nv);
    if parent[0] != 0 {
        return Err(LauncherError::InvalidMetadata("质量矩阵根父体无效"));
    }
    for (index, &value) in parent.iter().enumerate().skip(1) {
        if !usize::try_from(value).is_ok_and(|value| value < index) {
            return Err(LauncherError::InvalidMetadata("质量矩阵父体未前序"));
        }
    }
    if body
        .iter()
        .any(|&value| !usize::try_from(value).is_ok_and(|value| value < nbody))
    {
        return Err(LauncherError::InvalidMetadata("质量矩阵自由度刚体越界"));
    }
    for (index, &value) in dof_parent.iter().enumerate() {
        if value != -1 && !usize::try_from(value).is_ok_and(|value| value < index) {
            return Err(LauncherError::InvalidMetadata("质量矩阵父自由度未前序"));
        }
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RigidMassMatrixMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RigidMassMatrixMetadata {
    type Dims = RigidMassMatrixDims;

    const KIND: ProofKind = RIGID_MASS_MATRIX_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rigid_mass_matrix_metadata(values, *dims)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct RigidMassMatrixSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rigid_mass_matrix私有ABI。
unsafe impl KernelAbi for RigidMassMatrixSpec {
    const NAME: &'static str = "rigid_mass_matrix";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RIGID_MASS_MATRIX_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RigidMassMatrixSpec {
    type Dims = RigidMassMatrixDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rigid_mass_matrix_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rigid_mass_matrix_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

/// 在最小跨度之外要求精确跨度。
#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rigid_mass_matrix_plan(
    dims: &RigidMassMatrixDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RigidMassMatrixDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rigid_mass_matrix_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rigid_mass_matrix_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RigidMassMatrixSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn rigid_mass_matrix_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "mass_matrix_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mass_matrix_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mass_matrix_abi",
        }
        .into(),
    }
}
