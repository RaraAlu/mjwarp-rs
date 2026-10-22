//! 共享被动力四入口的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, Checked, ElemType, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotDesc,
    SlotKind, SlotRole,
};

/// 元数据证明名称：四入口只冻结八项守卫。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.passive_force.metadata.v1");

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

/// 冻结ABI：元数据、参数、状态/输入与输出，后接四个标量。
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
pub(super) enum PassiveForceEntry {
    Clear,
    Sum,
    ClearGravity,
    AddFluid,
}

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PassiveForceStaticDims {
    pub(super) nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl PassiveForceStaticDims {
    pub(super) fn new(nv: usize) -> Result<Self, LauncherError> {
        let nv = u32::try_from(nv).map_err(|_| LauncherError::Overflow("passive_force_nv"))?;
        if nv == 0 {
            return Err(LauncherError::InvalidArray("共享被动力维度无效"));
        }
        Ok(Self { nv })
    }

    /// 守卫元素数：首尾各四个，无载荷。
    pub(super) fn guard_elements() -> u64 {
        8
    }

    /// 单字段结果每世界元素数。
    pub(super) fn field_stride(self) -> u64 {
        u64::from(self.nv)
    }
}

/// 共享被动力发射维度；宽度与精确槽位随入口变化。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PassiveForceDims {
    pub(super) static_dims: PassiveForceStaticDims,
    pub(super) worlds: u32,
    pub(super) abi: [u32; 4],
    pub(super) entry: PassiveForceEntry,
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("共享被动力世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("passive_force_guarded"))
}

/// 四槽精确元素数；写宽结果缓冲的入口按最小容量校对输出。
#[cfg(feature = "cuda-probe")]
pub(super) fn passive_force_lengths(dims: &PassiveForceDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    if static_dims.nv == 0 {
        return Err(LauncherError::InvalidArray("共享被动力维度无效"));
    }
    let guarded_len = PassiveForceStaticDims::guard_elements();
    let field = guarded(dims.worlds, static_dims.field_stride())?;
    let parts = world_count(dims.worlds)?
        .checked_mul(
            static_dims
                .field_stride()
                .checked_mul(2)
                .ok_or(LauncherError::Overflow("passive_force_parts"))?,
        )
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("passive_force_parts"))?;
    match dims.entry {
        PassiveForceEntry::Clear => Ok([guarded_len, guarded_len, guarded_len, parts]),
        // 分片缓冲两字段；输出是四字段平滑力缓冲，只按最小单字段校对。
        PassiveForceEntry::Sum => Ok([guarded_len, guarded_len, parts, field]),
        PassiveForceEntry::ClearGravity => Ok([guarded_len, guarded_len, guarded_len, field]),
        PassiveForceEntry::AddFluid => Ok([guarded_len, guarded_len, field, field]),
    }
}

/// 需要精确跨度校对的槽位；输出为四字段平滑力缓冲的入口跳过输出槽。
#[cfg(feature = "cuda-probe")]
pub(super) fn passive_force_exact_slots(entry: PassiveForceEntry) -> &'static [usize] {
    match entry {
        PassiveForceEntry::Clear | PassiveForceEntry::ClearGravity => &[0, 1, 2, 3],
        PassiveForceEntry::Sum | PassiveForceEntry::AddFluid => &[0, 1, 2],
    }
}

/// 校验元数据：八项守卫，无载荷。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_passive_force_metadata(
    values: &[i32],
    _dims: PassiveForceStaticDims,
) -> Result<(), LauncherError> {
    if values != [-131072; 8] {
        return Err(LauncherError::InvalidMetadata("共享被动力元数据守卫无效"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct PassiveForceMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for PassiveForceMetadata {
    type Dims = PassiveForceDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_passive_force_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn passive_force_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "passive_force_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "passive_force_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "passive_force_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "passive_force_abi",
        }
        .into(),
    }
}

/// 计划级元数据缓冲；构造时上传一次，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct PassiveForceMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: PassiveForceStaticDims,
    revision: MetadataRevision,
    checked: Checked<PassiveForceMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl PassiveForceMetadataBuffer {
    /// 由冻结的八项守卫上传并建立证明。
    pub(super) fn from_values(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: PassiveForceStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = Self::base_dims(static_dims);
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<PassiveForceMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(passive_force_launch_error)?;
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

    pub(super) fn static_dims(&self) -> PassiveForceStaticDims {
        self.static_dims
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.buffer.len()
    }

    fn base_dims(static_dims: PassiveForceStaticDims) -> PassiveForceDims {
        PassiveForceDims {
            static_dims,
            worlds: 1,
            abi: [0; 4],
            entry: PassiveForceEntry::Clear,
        }
    }

    /// 发射证明：先核对证明未失效，再按实际维度重建。
    pub(super) fn proof(
        &self,
        dims: PassiveForceDims,
    ) -> Result<Checked<PassiveForceMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &Self::base_dims(self.static_dims))
            .map_err(passive_force_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际维度只影响跨度。
        unsafe {
            Checked::<PassiveForceMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(passive_force_launch_error)
    }
}
