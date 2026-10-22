//! 肌腱被动力三入口的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, Checked, ElemType, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotDesc,
    SlotKind, SlotRole,
};

/// 元数据证明名称；三入口共用肌腱速度计划的冻结元数据。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.tendon_passive.metadata.v1");

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

/// 冻结ABI：元数据、参数、源、速度与输出，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const SLOTS: [SlotDesc; 9] = [
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
    read_slot(3),
    SlotDesc {
        index: 4,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    scalar_slot(5),
    scalar_slot(6),
    scalar_slot(7),
    scalar_slot(8),
];

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TendonPassiveStaticDims {
    pub(super) nt: u32,
    pub(super) nv: u32,
    pub(super) nnz: u32,
    pub(super) stride: u32,
    pub(super) parameters_len: u32,
    pub(super) max_row: u32,
}

#[cfg(feature = "cuda-probe")]
impl TendonPassiveStaticDims {
    pub(super) fn new(
        nt: usize,
        nv: usize,
        nnz: usize,
        stride: usize,
        parameters_len: usize,
        max_row: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nt: value("tendon_passive_nt", nt)?,
            nv: value("tendon_passive_nv", nv)?,
            nnz: value("tendon_passive_nnz", nnz)?,
            stride: value("tendon_passive_stride", stride)?,
            parameters_len: value("tendon_passive_parameters", parameters_len)?,
            max_row: value("tendon_passive_max_row", max_row)?,
        };
        if dims.nt == 0 || dims.nv == 0 || dims.max_row == 0 {
            return Err(LauncherError::InvalidArray("肌腱被动力维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：首尾各四个守卫、行偏移、行数与列索引。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.nt)
            .checked_mul(2)
            .and_then(|rows| rows.checked_add(u64::from(self.nnz)))
            .and_then(|value| value.checked_add(8))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("tendon_passive_metadata"))
    }

    /// 源结果每世界元素数。
    pub(super) fn source_stride(self) -> u64 {
        u64::from(self.stride)
    }

    /// 速度每世界元素数。
    pub(super) fn velocity_stride(self) -> u64 {
        u64::from(self.nt)
    }
}

/// 肌腱被动力发射维度；输出宽度与四个标量随入口变化。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TendonPassiveDims {
    pub(super) static_dims: TendonPassiveStaticDims,
    pub(super) worlds: u32,
    pub(super) abi: [u32; 4],
    pub(super) output_width: u64,
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("肌腱被动力世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("tendon_passive_guarded"))
}

/// 逐槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_passive_lengths(dims: &TendonPassiveDims) -> Result<[u64; 5], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        u64::from(static_dims.parameters_len)
            .checked_add(8)
            .ok_or(LauncherError::Overflow("tendon_passive_parameters"))?,
        guarded(dims.worlds, static_dims.source_stride())?,
        guarded(dims.worlds, static_dims.velocity_stride())?,
        guarded(dims.worlds, dims.output_width)?,
    ])
}

/// 校验元数据：四守卫、行偏移/行数范围与列索引上界。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_tendon_passive_metadata(
    values: &[i32],
    dims: TendonPassiveStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("肌腱被动力元数据长度无效"));
    }
    let nt = usize::try_from(dims.nt).map_err(|_| LauncherError::Overflow("tendon_passive_nt"))?;
    let nnz =
        usize::try_from(dims.nnz).map_err(|_| LauncherError::Overflow("tendon_passive_nnz"))?;
    let tail = 4 + 2 * nt + nnz;
    if values[..4] != [-131072; 4] || values[tail..tail + 4] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("肌腱被动力元数据守卫无效"));
    }
    let adr = &values[4..4 + nt];
    let num = &values[4 + nt..4 + 2 * nt];
    for (&start, &count) in adr.iter().zip(num) {
        let start = usize::try_from(start)
            .map_err(|_| LauncherError::InvalidMetadata("肌腱被动力行偏移为负"))?;
        let count = usize::try_from(count)
            .map_err(|_| LauncherError::InvalidMetadata("肌腱被动力行数为负"))?;
        if start.checked_add(count).is_none_or(|end| end > nnz) {
            return Err(LauncherError::InvalidMetadata("肌腱被动力行区间越界"));
        }
    }
    if values[4 + 2 * nt..tail]
        .iter()
        .any(|&column| column < 0 || column as u32 >= dims.nv)
    {
        return Err(LauncherError::InvalidMetadata("肌腱被动力列索引越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonPassiveMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for TendonPassiveMetadata {
    type Dims = TendonPassiveDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_tendon_passive_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_passive_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "tendon_passive_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "tendon_passive_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "tendon_passive_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "tendon_passive_abi",
        }
        .into(),
    }
}

/// 计划级元数据缓冲；构造时上传一次，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct TendonPassiveMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: TendonPassiveStaticDims,
    revision: MetadataRevision,
    checked: Checked<TendonPassiveMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl TendonPassiveMetadataBuffer {
    /// 由肌腱速度计划的冻结宿主镜像重建本计划元数据。
    pub(super) fn from_values(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: TendonPassiveStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = TendonPassiveDims {
            static_dims,
            worlds: 1,
            abi: [0; 4],
            output_width: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<TendonPassiveMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(tendon_passive_launch_error)?;
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

    pub(super) fn static_dims(&self) -> TendonPassiveStaticDims {
        self.static_dims
    }

    /// 发射证明：先核对证明未失效，再按实际维度重建。
    pub(super) fn proof(
        &self,
        dims: TendonPassiveDims,
    ) -> Result<Checked<TendonPassiveMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(
                span,
                &TendonPassiveDims {
                    static_dims: self.static_dims,
                    worlds: 1,
                    abi: [0; 4],
                    output_width: 1,
                },
            )
            .map_err(tendon_passive_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际维度只影响跨度。
        unsafe {
            Checked::<TendonPassiveMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(tendon_passive_launch_error)
    }
}
