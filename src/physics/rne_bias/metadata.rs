//! 偏置力元数据槽位、维度与校验。

#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const RNE_BIAS_METADATA_KIND: ProofKind = ProofKind("rne.bias.metadata.v1");

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const RNE_BIAS_SLOTS: [SlotDesc; 6] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RNE_BIAS_METADATA_KIND,
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
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RneBiasDims {
    pub(super) worlds: u32,
    pub(super) nbody: u32,
    pub(super) nv: u32,
    pub(super) metadata_elements: u64,
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_bias_metadata_elements(nbody: u32, nv: u32) -> Result<u64, LauncherError> {
    if nbody == 0 || nbody > i32::MAX as u32 || nv > i32::MAX as u32 {
        return Err(LauncherError::Overflow("rne_bias_signed_dimensions"));
    }
    u64::from(nbody)
        .checked_mul(3)
        .and_then(|value| value.checked_add(u64::from(nv)))
        .and_then(|value| value.checked_add(5))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("rne_bias_metadata_elements"))
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_bias_lengths(dims: &RneBiasDims) -> Result<[u64; 5], LauncherError> {
    if dims.worlds == 0 || dims.worlds > u32::MAX - 255 || dims.nbody == 0 {
        return Err(LauncherError::InvalidArray("RNE偏置力维度无效"));
    }
    let expected = rne_bias_metadata_elements(dims.nbody, dims.nv)?;
    if dims.metadata_elements != expected {
        return Err(LauncherError::InvalidMetadata("RNE偏置力元数据长度不匹配"));
    }
    let worlds = u64::from(dims.worlds);
    let nbody = u64::from(dims.nbody);
    let nv = u64::from(dims.nv);
    let guarded = |width: u64, field| {
        worlds
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_bias_com_width"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_bias_velocity_width"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_bias_output_width"))?;
    let qvel = guarded(nv, "rne_bias_qvel")?;
    let com = guarded(com_width, "rne_bias_com")?;
    let velocity = guarded(velocity_width, "rne_bias_velocity")?;
    let output = guarded(output_width, "rne_bias_output")?;
    if output_width > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rne_bias_output_width"));
    }
    Ok([dims.metadata_elements, qvel, com, velocity, output])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn validate_rne_bias_metadata(
    values: &[i32],
    dims: RneBiasDims,
) -> Result<(), LauncherError> {
    let expected = rne_bias_metadata_elements(dims.nbody, dims.nv)?;
    if dims.metadata_elements != expected
        || u64::try_from(values.len()).ok() != Some(dims.metadata_elements)
    {
        return Err(LauncherError::InvalidMetadata("RNE偏置力元数据长度无效"));
    }
    let nbody = usize::try_from(dims.nbody)
        .map_err(|_| LauncherError::Overflow("rne_bias_metadata_nbody"))?;
    let nv =
        usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("rne_bias_metadata_nv"))?;
    if values[0] != dims.nbody as i32 || values[1] != dims.nv as i32 {
        return Err(LauncherError::InvalidMetadata("RNE偏置力元数据维度不匹配"));
    }
    if values[2..5]
        .iter()
        .any(|&bits| !f32::from_bits(bits as u32).is_finite())
    {
        return Err(LauncherError::InvalidMetadata("RNE偏置力重力不是有限值"));
    }

    let parent_start = 5;
    let count_start = parent_start + nbody;
    let address_start = count_start + nbody;
    let body_start = address_start + nbody;
    let parent = &values[parent_start..count_start];
    let count = &values[count_start..address_start];
    let address = &values[address_start..body_start];
    let body = &values[body_start..];
    if parent[0] != 0 || count[0] != 0 || address[0] != 0 {
        return Err(LauncherError::InvalidMetadata("RNE偏置力根节点元数据无效"));
    }
    let mut covered = vec![false; nv];
    for index in 1..nbody {
        let parent_index = usize::try_from(parent[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE偏置力父节点为负"))?;
        if parent_index >= index {
            return Err(LauncherError::InvalidMetadata("RNE偏置力父节点顺序无效"));
        }
        let dof_count = usize::try_from(count[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE偏置力自由度数量为负"))?;
        let dof_start = usize::try_from(address[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE偏置力自由度地址为负"))?;
        let dof_end = dof_start
            .checked_add(dof_count)
            .filter(|&end| end <= nv)
            .ok_or(LauncherError::InvalidMetadata("RNE偏置力自由度范围越界"))?;
        for dof in dof_start..dof_end {
            if body[dof] != index as i32 || covered[dof] {
                return Err(LauncherError::InvalidMetadata("RNE偏置力自由度归属无效"));
            }
            covered[dof] = true;
        }
    }
    if covered.contains(&false) {
        return Err(LauncherError::InvalidMetadata("RNE偏置力自由度映射不完整"));
    }
    for &body_index in body {
        let body_index = usize::try_from(body_index)
            .map_err(|_| LauncherError::InvalidMetadata("RNE偏置力体索引为负"))?;
        if body_index >= nbody {
            return Err(LauncherError::InvalidMetadata("RNE偏置力体索引越界"));
        }
    }
    rne_bias_lengths(&dims).map(|_| ())
}
