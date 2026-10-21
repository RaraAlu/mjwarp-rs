//! 动能指标元数据、规格与长度。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) const KINETIC_METADATA_KIND: ProofKind = ProofKind("kinetic_energy_nv");

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticKineticMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticKineticMetadata {
    type Dims = u32;

    const KIND: ProofKind = KINETIC_METADATA_KIND;

    fn validate(values: &[i32], nv: &u32) -> Result<(), LauncherError> {
        if values == [i32::try_from(*nv).map_err(|_| LauncherError::Overflow("kinetic_nv"))?] {
            Ok(())
        } else {
            Err(LauncherError::InvalidMetadata("静态自由度不匹配"))
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct KineticLaunchMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for KineticLaunchMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = KINETIC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        kinetic_lengths(dims)?;
        StaticKineticMetadata::validate(values, &dims[0])
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) const KINETIC_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: KINETIC_METADATA_KIND,
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
pub(super) struct KineticEnergy32Spec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐项匹配冻结四指针与四标量ABI。
unsafe impl KernelAbi for KineticEnergy32Spec {
    const NAME: &'static str = "kinetic_energy_reduce32";
    const BLOCK: BlockRule = BlockRule::Fixed(32);
    const SLOTS: &'static [SlotDesc] = &KINETIC_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for KineticEnergy32Spec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (pair, energy) = kinetic_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 1,
                min_elements: pair,
            },
            SlotRequirement {
                slot: 2,
                min_elements: energy,
            },
            SlotRequirement {
                slot: 3,
                min_elements: energy,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        kinetic_lengths(dims)?;
        LaunchDims::new((dims[1], 1, 1), (32, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct KineticValidationSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐项匹配输入校验的四指针与四标量ABI。
unsafe impl KernelAbi for KineticValidationSpec {
    const NAME: &'static str = "kinetic_energy_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &KINETIC_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for KineticValidationSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (pair, energy) = kinetic_lengths(dims)?;
        let status = u64::from(dims[1])
            .checked_add(8)
            .ok_or(LauncherError::Overflow("kinetic_status_elements"))?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 1,
                min_elements: pair,
            },
            SlotRequirement {
                slot: 2,
                min_elements: energy,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        kinetic_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn kinetic_lengths(dims: &[u32; 4]) -> Result<(u64, u64), LauncherError> {
    let [nv, worlds, reserved, work] = *dims;
    if !(1..=1024).contains(&nv)
        || worlds == 0
        || worlds > i32::MAX as u32
        || reserved != 0
        || work != worlds
    {
        return Err(LauncherError::InvalidArray("动能维度无效"));
    }
    let pair = u64::from(worlds)
        .checked_mul(2 * u64::from(nv))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("kinetic_pair_elements"))?;
    let energy = u64::from(worlds)
        .checked_mul(2)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("kinetic_energy_elements"))?;
    Ok((pair, energy))
}

#[cfg(feature = "cuda-probe")]
pub(super) fn transfer_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "kinetic_energy_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "kinetic_energy32_abi",
        }
        .into(),
    }
}
