//! 柔体边被动力三入口的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, Checked, ElemType, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotDesc,
    SlotKind, SlotRole,
};

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.flex_edge_passive.metadata.v1");

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

/// 冻结ABI：元数据、参数、源与输出，后接四个标量。
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

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlexEdgePassiveStaticDims {
    pub(super) nv: u32,
    pub(super) ne: u32,
    pub(super) nf: u32,
    pub(super) nnz: u32,
    pub(super) stride: u32,
}

#[cfg(feature = "cuda-probe")]
impl FlexEdgePassiveStaticDims {
    /// 由自由度、柔体边、柔体与稀疏行计数构造。
    pub(super) fn new(
        nv: usize,
        ne: usize,
        nf: usize,
        nnz: usize,
        stride: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nv: value("flex_edge_passive_nv", nv)?,
            ne: value("flex_edge_passive_ne", ne)?,
            nf: value("flex_edge_passive_nf", nf)?,
            nnz: value("flex_edge_passive_nnz", nnz)?,
            stride: value("flex_edge_passive_stride", stride)?,
        };
        if dims.nv == 0 || dims.ne == 0 || dims.nf == 0 {
            return Err(LauncherError::InvalidArray("柔体边被动力维度无效"));
        }
        let expected = u64::from(dims.ne)
            .checked_mul(2)
            .and_then(|value| value.checked_add(u64::from(dims.nnz)))
            .ok_or(LauncherError::Overflow("flex_edge_passive_stride"))?;
        if expected != u64::from(dims.stride) {
            return Err(LauncherError::InvalidArray("柔体边被动力源跨度不符"));
        }
        dims.metadata_elements()?;
        dims.parameters_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：首尾各四个守卫、行偏移、行数、列索引、柔体号与刚体标记。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.ne)
            .checked_mul(4)
            .and_then(|rows| rows.checked_add(u64::from(self.nnz)))
            .and_then(|value| value.checked_add(8))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("flex_edge_passive_metadata"))
    }

    /// 参数元素数：首尾各四个守卫、静态长度、刚度与阻尼。
    pub(super) fn parameters_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.nf)
            .checked_mul(2)
            .and_then(|value| value.checked_add(u64::from(self.ne)))
            .and_then(|value| value.checked_add(8))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("flex_edge_passive_parameters"))
    }

    /// 源结果每世界元素数。
    pub(super) fn source_stride(self) -> u64 {
        u64::from(self.stride)
    }
}

/// 柔体边被动力发射维度；输出宽度与四个标量随入口变化。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlexEdgePassiveDims {
    pub(super) static_dims: FlexEdgePassiveStaticDims,
    pub(super) worlds: u32,
    pub(super) abi: [u32; 4],
    pub(super) output_width: u64,
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("柔体边被动力世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("flex_edge_passive_guarded"))
}

/// 四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn flex_edge_passive_lengths(
    dims: &FlexEdgePassiveDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        static_dims.parameters_elements()?,
        guarded(dims.worlds, static_dims.source_stride())?,
        guarded(dims.worlds, dims.output_width)?,
    ])
}

/// 校验元数据：四守卫、行偏移/行数范围、列索引、柔体号与刚体标记。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_flex_edge_passive_metadata(
    values: &[i32],
    dims: FlexEdgePassiveStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("柔体边被动力元数据长度无效"));
    }
    let ne =
        usize::try_from(dims.ne).map_err(|_| LauncherError::Overflow("flex_edge_passive_ne"))?;
    let nf =
        usize::try_from(dims.nf).map_err(|_| LauncherError::Overflow("flex_edge_passive_nf"))?;
    let nnz =
        usize::try_from(dims.nnz).map_err(|_| LauncherError::Overflow("flex_edge_passive_nnz"))?;
    let tail = 4 + 4 * ne + nnz;
    if values[..4] != [-131072; 4] || values[tail..tail + 4] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("柔体边被动力元数据守卫无效"));
    }
    let adr = &values[4..4 + ne];
    let num = &values[4 + ne..4 + 2 * ne];
    for (&start, &count) in adr.iter().zip(num) {
        let start = usize::try_from(start)
            .map_err(|_| LauncherError::InvalidMetadata("柔体边被动力行偏移为负"))?;
        let count = usize::try_from(count)
            .map_err(|_| LauncherError::InvalidMetadata("柔体边被动力行数为负"))?;
        if start.checked_add(count).is_none_or(|end| end > nnz) {
            return Err(LauncherError::InvalidMetadata("柔体边被动力行区间越界"));
        }
    }
    if values[4 + 2 * ne..4 + 2 * ne + nnz]
        .iter()
        .any(|&column| column < 0 || column as u32 >= dims.nv)
    {
        return Err(LauncherError::InvalidMetadata("柔体边被动力列索引越界"));
    }
    if values[4 + 2 * ne + nnz..4 + 3 * ne + nnz]
        .iter()
        .any(|&flex| flex < 0 || flex as usize >= nf)
    {
        return Err(LauncherError::InvalidMetadata("柔体边被动力柔体号越界"));
    }
    if values[tail - ne..tail]
        .iter()
        .any(|&rigid| rigid != 0 && rigid != 1)
    {
        return Err(LauncherError::InvalidMetadata("柔体边被动力刚体标记无效"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexEdgePassiveMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for FlexEdgePassiveMetadata {
    type Dims = FlexEdgePassiveDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_flex_edge_passive_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn flex_edge_passive_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "flex_edge_passive_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "flex_edge_passive_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "flex_edge_passive_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "flex_edge_passive_abi",
        }
        .into(),
    }
}

/// 计划级元数据缓冲；构造时上传一次，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct FlexEdgePassiveMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: FlexEdgePassiveStaticDims,
    revision: MetadataRevision,
    checked: Checked<FlexEdgePassiveMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl FlexEdgePassiveMetadataBuffer {
    /// 由计划构造期冻结的宿主镜像上传并建立证明。
    pub(super) fn from_values(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: FlexEdgePassiveStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = FlexEdgePassiveDims {
            static_dims,
            worlds: 1,
            abi: [0; 4],
            output_width: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<FlexEdgePassiveMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(flex_edge_passive_launch_error)?;
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

    pub(super) fn static_dims(&self) -> FlexEdgePassiveStaticDims {
        self.static_dims
    }

    /// 发射证明：先核对证明未失效，再按实际维度重建。
    pub(super) fn proof(
        &self,
        dims: FlexEdgePassiveDims,
    ) -> Result<Checked<FlexEdgePassiveMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(
                span,
                &FlexEdgePassiveDims {
                    static_dims: self.static_dims,
                    worlds: 1,
                    abi: [0; 4],
                    output_width: 1,
                },
            )
            .map_err(flex_edge_passive_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际维度只影响跨度。
        unsafe {
            Checked::<FlexEdgePassiveMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(flex_edge_passive_launch_error)
    }
}
