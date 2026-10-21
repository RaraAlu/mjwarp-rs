//! 刚体质量求解元数据槽位、维度与长度。

#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const RIGID_MASS_SOLVE_METADATA_KIND: ProofKind =
    ProofKind("rigid_mass_solve.blocks.v1");

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const RIGID_MASS_SOLVE_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RIGID_MASS_SOLVE_METADATA_KIND,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RigidMassSolveDims {
    pub(super) abi: [u32; 4],
    pub(super) metadata_elements: u64,
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rigid_mass_solve_lengths(
    dims: &RigidMassSolveDims,
) -> Result<[u64; 4], LauncherError> {
    let [nbody, nv, rhs_count, worlds] = dims.abi;
    if worlds == 0 || rhs_count == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("刚体质量求解维度无效"));
    }
    if nv > i32::MAX as u32
        || rhs_count > i32::MAX as u32
        || dims.metadata_elements == 0
        || dims.metadata_elements > i32::MAX as u64
    {
        return Err(LauncherError::Overflow("rigid_mass_solve_dimensions"));
    }
    let nbody = u64::from(nbody);
    let nv = u64::from(nv);
    let rhs_count = u64::from(rhs_count);
    let worlds = u64::from(worlds);
    let square = nv
        .checked_mul(nv)
        .ok_or(LauncherError::Overflow("rigid_mass_solve_square"))?;
    let rhs_logical = worlds
        .checked_mul(rhs_count)
        .and_then(|value| value.checked_mul(nv))
        .ok_or(LauncherError::Overflow("rigid_mass_solve_rhs"))?;
    let rhs = if nv == 0 { 1 } else { rhs_logical };
    let state_width = nbody
        .checked_mul(10)
        .and_then(|value| value.checked_add(square))
        .ok_or(LauncherError::Overflow("rigid_mass_solve_state"))?;
    if state_width > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_mass_solve_state_width"));
    }
    let state = worlds
        .checked_mul(state_width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("rigid_mass_solve_state"))?;
    let output_width = square
        .checked_add(nv)
        .and_then(|value| {
            rhs_count
                .checked_mul(nv)
                .and_then(|rhs| value.checked_add(rhs))
        })
        .and_then(|value| value.checked_add(3))
        .ok_or(LauncherError::Overflow("rigid_mass_solve_output"))?;
    if output_width > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_mass_solve_output_width"));
    }
    let output = worlds
        .checked_mul(output_width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("rigid_mass_solve_output"))?;
    Ok([dims.metadata_elements, rhs, state, output])
}
