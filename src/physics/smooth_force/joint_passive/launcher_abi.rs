//! 关节被动力三入口的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, Checked, ElemType, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotDesc,
    SlotKind, SlotRole,
};

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.joint_passive.metadata.v1");

#[cfg(feature = "cuda-probe")]
const fn read_slot(index: usize) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    }
}

#[cfg(feature = "cuda-probe")]
const fn scalar_slot(index: usize) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    }
}

/// 冻结ABI：元数据、参数、状态与输出，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing { proof: PROOF },
    },
    read_slot(1),
    read_slot(2),
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    scalar_slot(4),
    scalar_slot(5),
    scalar_slot(6),
    scalar_slot(7),
];

/// 入口种类；决定每槽精确宽度与精确校对的槽位。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JointPassiveEntry {
    Validate,
    Compute,
    Sum,
}

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct JointPassiveStaticDims {
    pub(super) nq: u32,
    pub(super) nv: u32,
    pub(super) nj: u32,
    pub(super) param_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl JointPassiveStaticDims {
    /// 由广义坐标、自由度、关节与参数计数构造。
    pub(super) fn new(
        nq: usize,
        nv: usize,
        nj: usize,
        param_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nq: value("joint_passive_nq", nq)?,
            nv: value("joint_passive_nv", nv)?,
            nj: value("joint_passive_nj", nj)?,
            param_len: value("joint_passive_parameters", param_len)?,
        };
        if dims.nq == 0 || dims.nv == 0 || dims.nj == 0 {
            return Err(LauncherError::InvalidArray("关节被动力维度无效"));
        }
        dims.metadata_elements()?;
        dims.parameters_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：首尾各四个守卫与四张逐自由度表。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.nv)
            .checked_mul(4)
            .and_then(|rows| rows.checked_add(8))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("joint_passive_metadata"))
    }

    /// 参数元素数：首尾各四个守卫与冻结批量。
    pub(super) fn parameters_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.param_len)
            .checked_add(8)
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("joint_passive_parameters"))
    }

    /// 单世界状态元素数：qpos 与 qvel。
    fn state_stride(self) -> u64 {
        u64::from(self.nq) + u64::from(self.nv)
    }
}

/// 关节被动力发射维度；宽度与精确槽位随入口变化。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct JointPassiveDims {
    pub(super) static_dims: JointPassiveStaticDims,
    pub(super) worlds: u32,
    pub(super) abi: [u32; 4],
    pub(super) entry: JointPassiveEntry,
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("关节被动力世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("joint_passive_guarded"))
}

/// 四槽精确元素数；求和入口的未读指针只要求非空。
#[cfg(feature = "cuda-probe")]
pub(super) fn joint_passive_lengths(dims: &JointPassiveDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let nv = u64::from(static_dims.nv);
    let metadata = static_dims.metadata_elements()?;
    match dims.entry {
        JointPassiveEntry::Validate => Ok([
            metadata,
            static_dims.parameters_elements()?,
            guarded(dims.worlds, static_dims.state_stride())?,
            guarded(dims.worlds, 1)?,
        ]),
        JointPassiveEntry::Compute => Ok([
            metadata,
            static_dims.parameters_elements()?,
            guarded(dims.worlds, static_dims.state_stride())?,
            guarded(dims.worlds, 2 * nv)?,
        ]),
        // 槽位1是共享分部；槽位2未读，只需满足底层ABI非空。
        // 求和只写结果缓冲首个字段，缓冲仍按四字段满载宽度校对。
        JointPassiveEntry::Sum => Ok([
            metadata,
            guarded(dims.worlds, 2 * nv)?,
            1,
            guarded(dims.worlds, 4 * nv)?,
        ]),
    }
}

/// 需要精确跨度校对的槽位；求和入口跳过未读指针。
#[cfg(feature = "cuda-probe")]
pub(super) fn joint_passive_exact_slots(entry: JointPassiveEntry) -> &'static [usize] {
    match entry {
        JointPassiveEntry::Validate | JointPassiveEntry::Compute => &[0, 1, 2, 3],
        JointPassiveEntry::Sum => &[0, 1, 3],
    }
}

/// 校验元数据：四守卫与逐自由度关节种类、qpos 偏移、关节号、局部自由度。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_joint_passive_metadata(
    values: &[i32],
    dims: JointPassiveStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("关节被动力元数据长度无效"));
    }
    let nv = usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("joint_passive_nv"))?;
    let tail = 4 + 4 * nv;
    if values[..4] != [-131072; 4] || values[tail..tail + 4] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("关节被动力元数据守卫无效"));
    }
    let kinds = &values[4..4 + nv];
    let qadr = &values[4 + nv..4 + 2 * nv];
    let joint = &values[4 + 2 * nv..4 + 3 * nv];
    let local = &values[4 + 3 * nv..tail];
    for index in 0..nv {
        let (qpos_width, dof_width) = match kinds[index] {
            0 => (7, 6),
            1 => (4, 3),
            2 | 3 => (1, 1),
            _ => return Err(LauncherError::InvalidMetadata("关节被动力关节种类无效")),
        };
        let start = usize::try_from(qadr[index])
            .map_err(|_| LauncherError::InvalidMetadata("关节被动力qpos偏移为负"))?;
        if start
            .checked_add(qpos_width)
            .is_none_or(|end| end > dims.nq as usize)
        {
            return Err(LauncherError::InvalidMetadata("关节被动力qpos区间越界"));
        }
        if joint[index] < 0 || joint[index] as u32 >= dims.nj {
            return Err(LauncherError::InvalidMetadata("关节被动力关节号越界"));
        }
        if local[index] < 0 || local[index] as usize >= dof_width {
            return Err(LauncherError::InvalidMetadata("关节被动力局部自由度越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct JointPassiveMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for JointPassiveMetadata {
    type Dims = JointPassiveDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_joint_passive_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn joint_passive_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "joint_passive_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "joint_passive_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "joint_passive_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "joint_passive_abi",
        }
        .into(),
    }
}

/// 计划级元数据缓冲；构造时上传一次，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct JointPassiveMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: JointPassiveStaticDims,
    revision: MetadataRevision,
    checked: Checked<JointPassiveMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl JointPassiveMetadataBuffer {
    /// 由计划构造期冻结的宿主镜像上传并建立证明。
    pub(super) fn from_values(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: JointPassiveStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = Self::base_dims(static_dims);
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<JointPassiveMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(joint_passive_launch_error)?;
        Ok(Self {
            buffer,
            values,
            static_dims,
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn static_dims(&self) -> JointPassiveStaticDims {
        self.static_dims
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.buffer.len()
    }

    #[cfg(test)]
    pub(super) fn read_range_into(
        &self,
        start: usize,
        out: &mut [i32],
    ) -> Result<(), TransferError> {
        self.buffer.read_range_into(start, out)
    }

    /// 只改设备元数据、保留有效证明，供设备验证器测试读取坏值。
    #[cfg(test)]
    pub(super) fn corrupt_device_for_validator_test(
        &mut self,
        start: usize,
        values: &[i32],
    ) -> Result<(), TransferError> {
        self.buffer.write_range(start, values)
    }

    fn base_dims(static_dims: JointPassiveStaticDims) -> JointPassiveDims {
        JointPassiveDims {
            static_dims,
            worlds: 1,
            abi: [0; 4],
            entry: JointPassiveEntry::Validate,
        }
    }

    /// 发射证明：先核对证明未失效，再按实际维度重建。
    pub(super) fn proof(
        &self,
        dims: JointPassiveDims,
    ) -> Result<Checked<JointPassiveMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &Self::base_dims(self.static_dims))
            .map_err(joint_passive_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际维度只影响跨度。
        unsafe {
            Checked::<JointPassiveMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(joint_passive_launch_error)
    }
}
