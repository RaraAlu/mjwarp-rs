//! 刚体运动学的元数据证明与精确跨度。

use super::RigidVariant;
use crate::runtime::launcher::LauncherError;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{MetadataProof, ProofKind};

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.rigid_kinematics.metadata.v1");

/// 上传后不变的维度与已核验缓冲长度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RigidStaticDims {
    pub(super) variant: RigidVariant,
    pub(super) nq: u32,
    pub(super) nb: u32,
    pub(super) nj: u32,
    pub(super) nmocap: u32,
    pub(super) metadata_elements: u64,
    pub(super) parameter_elements: u64,
}

/// 发射维度；`abi`顺序固定为nq、nb、nj、worlds。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RigidDims {
    pub(super) static_dims: RigidStaticDims,
    pub(super) abi: [u32; 4],
}

#[cfg(feature = "cuda-probe")]
impl RigidDims {
    pub(super) fn abi(&self) -> [u32; 4] {
        self.abi
    }
}

/// 元数据元素数由变体固定。
pub(super) fn metadata_elements(
    variant: RigidVariant,
    nb: u32,
    nj: u32,
) -> Result<u64, LauncherError> {
    let nb = u64::from(nb);
    let nj = u64::from(nj);
    let base = nb
        .checked_mul(3)
        .and_then(|n| nj.checked_mul(2).and_then(|m| n.checked_add(m)))
        .ok_or(LauncherError::Overflow("rigid_kinematics_metadata"))?;
    let total = if variant.batched() {
        base.checked_add(nb)
            .and_then(|n| n.checked_add(15))
            .ok_or(LauncherError::Overflow("rigid_kinematics_metadata"))?
    } else {
        base
    };
    if total == 0 || total > i32::MAX as u64 {
        return Err(LauncherError::InvalidArray("刚体运动学元数据长度无效"));
    }
    Ok(total)
}

/// 探针变体的连续参数元素数。
pub(super) fn probe_parameters(nq: u32, nb: u32, nj: u32) -> u64 {
    u64::from(nq) + 14 * u64::from(nb) + 6 * u64::from(nj)
}

/// 每世界状态元素数；零自由度保留单元素占位。
fn state_elements(dims: &RigidStaticDims, worlds: u32) -> Result<u64, LauncherError> {
    let per_world = if dims.variant.batched() {
        u64::from(dims.nq)
            .checked_add(
                u64::from(dims.nmocap)
                    .checked_mul(7)
                    .ok_or(LauncherError::Overflow("rigid_kinematics_state"))?,
            )
            .ok_or(LauncherError::Overflow("rigid_kinematics_state"))?
    } else {
        u64::from(dims.nq)
    };
    let total = per_world
        .checked_mul(u64::from(worlds))
        .ok_or(LauncherError::Overflow("rigid_kinematics_state"))?;
    if total > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_kinematics_state"));
    }
    Ok(total.max(1))
}

/// 守卫结果元素数：每世界28nb+6nj，前后各4个守卫。
fn result_elements(dims: &RigidStaticDims, worlds: u32) -> Result<u64, LauncherError> {
    let per_world = u64::from(dims.nb)
        .checked_mul(28)
        .and_then(|n| {
            u64::from(dims.nj)
                .checked_mul(6)
                .and_then(|m| n.checked_add(m))
        })
        .ok_or(LauncherError::Overflow("rigid_kinematics_result"))?;
    per_world
        .checked_mul(u64::from(worlds))
        .and_then(|n| n.checked_add(8))
        .ok_or(LauncherError::Overflow("rigid_kinematics_result"))
}

/// 元数据、参数、状态与结果的精确元素数。
pub(super) fn rigid_lengths(dims: &RigidDims) -> Result<[u64; 4], LauncherError> {
    let d = &dims.static_dims;
    let worlds = dims.abi[3];
    if dims.abi[0..3] != [d.nq, d.nb, d.nj] || d.nb == 0 || worlds == 0 {
        return Err(LauncherError::InvalidArray("刚体运动学发射维度无效"));
    }
    if worlds > u32::MAX - 255 {
        return Err(LauncherError::Overflow("rigid_kinematics_worlds"));
    }
    let metadata = metadata_elements(d.variant, d.nb, d.nj)?;
    if metadata != d.metadata_elements {
        return Err(LauncherError::InvalidMetadata("元数据长度不符"));
    }
    if !d.variant.batched() && d.parameter_elements != probe_parameters(d.nq, d.nb, d.nj) {
        return Err(LauncherError::InvalidMetadata("参数长度不符"));
    }
    Ok([
        metadata,
        d.parameter_elements,
        state_elements(d, worlds)?,
        result_elements(d, worlds)?,
    ])
}

/// 校验父体前序、关节区间、关节类型与qpos范围。
/// 批量变体另核mocap映射与七组字段描述符。
pub(super) fn validate_metadata(
    values: &[i32],
    dims: &RigidStaticDims,
) -> Result<(), LauncherError> {
    let metadata = metadata_elements(dims.variant, dims.nb, dims.nj)?;
    if u64::try_from(values.len()).ok() != Some(metadata) || metadata != dims.metadata_elements {
        return Err(LauncherError::InvalidMetadata("元数据长度无效"));
    }
    if dims.parameter_elements == 0 || dims.parameter_elements > i32::MAX as u64 {
        return Err(LauncherError::InvalidMetadata("参数长度无效"));
    }
    let bad = LauncherError::InvalidMetadata;
    let nb = dims.nb as usize;
    let nj = dims.nj as usize;
    let (parent, rest) = values.split_at(nb);
    let (adr, rest) = rest.split_at(nb);
    let (num, rest) = rest.split_at(nb);
    let (types, rest) = rest.split_at(nj);
    let (qposadr, rest) = rest.split_at(nj);
    if parent[0] != 0 {
        return Err(bad("根父体无效"));
    }
    for (index, &value) in parent.iter().enumerate().skip(1) {
        if !usize::try_from(value).is_ok_and(|value| value < index) {
            return Err(bad("父体未前序"));
        }
    }
    let nq = dims.nq as usize;
    for body in 0..nb {
        let count = usize::try_from(num[body])
            .ok()
            .filter(|&count| count <= nj)
            .ok_or(bad("关节区间长度越界"))?;
        if count == 0 {
            // 无关节的体沿用-1；内核只在长度为正时解引用起点。
            if adr[body] != -1 {
                return Err(bad("空关节区间非负一"));
            }
            continue;
        }
        let start = usize::try_from(adr[body])
            .ok()
            .filter(|&start| start <= nj)
            .ok_or(bad("关节区间起点越界"))?;
        if start + count > nj {
            return Err(bad("关节区间越界"));
        }
        for joint in start..start + count {
            let width = match types[joint] {
                0 => 7,
                1 => 4,
                2 | 3 => 1,
                _ => return Err(bad("关节类型无效")),
            };
            let begin = usize::try_from(qposadr[joint])
                .ok()
                .filter(|&begin| begin <= nq)
                .ok_or(bad("qpos地址越界"))?;
            if begin + width > nq {
                return Err(bad("qpos地址越界"));
            }
        }
    }
    if !dims.variant.batched() {
        if dims.parameter_elements != probe_parameters(dims.nq, dims.nb, dims.nj) {
            return Err(bad("参数长度不符"));
        }
        return Ok(());
    }
    let (nm, rest) = rest.split_first_chunk::<1>().ok_or(bad("mocap数量缺失"))?;
    if nm[0] < 0 || u32::try_from(nm[0]).ok() != Some(dims.nmocap) {
        return Err(bad("mocap数量不符"));
    }
    let (mocap, descriptors) = rest.split_at(nb);
    if descriptors.len() != 14 {
        return Err(bad("字段描述符缺失"));
    }
    for &value in mocap {
        if value != -1 && !u32::try_from(value).is_ok_and(|value| value < dims.nmocap) {
            return Err(bad("mocap映射越界"));
        }
    }
    let widths = [
        u64::from(dims.nq),
        3 * u64::from(dims.nb),
        4 * u64::from(dims.nb),
        3 * u64::from(dims.nb),
        4 * u64::from(dims.nb),
        3 * u64::from(dims.nj),
        3 * u64::from(dims.nj),
    ];
    let mut offset = 0u64;
    for (index, width) in widths.into_iter().enumerate() {
        let descriptor = &descriptors[2 * index..2 * index + 2];
        if u64::try_from(descriptor[0]).ok() != Some(offset) {
            return Err(bad("字段描述符偏移不连续"));
        }
        let period = u64::try_from(descriptor[1])
            .ok()
            .filter(|&period| period >= 1)
            .ok_or(bad("字段描述符周期非正"))?;
        offset = width
            .checked_mul(period)
            .and_then(|length| offset.checked_add(length))
            .ok_or(LauncherError::Overflow("rigid_kinematics_parameters"))?;
        if offset > dims.parameter_elements {
            return Err(bad("字段描述符越过参数"));
        }
    }
    if offset != dims.parameter_elements {
        return Err(bad("字段描述符总长不符"));
    }
    Ok(())
}

/// 上传时的静态证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct StaticRigidMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticRigidMetadata {
    type Dims = RigidStaticDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_metadata(values, dims)
    }
}

/// 发射时的完整证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct RigidMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for RigidMetadata {
    type Dims = RigidDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_metadata(values, &dims.static_dims)?;
        rigid_lengths(dims).map(|_| ())
    }
}
