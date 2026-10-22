//! 常驻肌腱合并与索引 FIELD_BATCH 变体的槽位、证明、维度与同步发射。

#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldBatchSpec;
#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_launch_dims;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_spans;
#[cfg(feature = "cuda-probe")]
use super::field_batch::launch_field_batch;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称；合并与索引共用同一模型元数据。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_tendon.metadata.v1");

#[cfg(feature = "cuda-probe")]
const fn read_slot(index: usize, elem: ElemType) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::DevicePtr { elem },
        access: Access::Read,
        role: SlotRole::Value,
    }
}

#[cfg(feature = "cuda-probe")]
const fn write_slot(index: usize, elem: ElemType) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::DevicePtr { elem },
        access: Access::ReadWrite,
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

#[cfg(feature = "cuda-probe")]
const fn meta_slot() -> SlotDesc {
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing { proof: PROOF },
    }
}

/// tendon_values：元数据、空间整数索引、源结果与合并输出，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const VALUES_SLOTS: [SlotDesc; 8] = [
    meta_slot(),
    read_slot(1, ElemType::I32),
    read_slot(2, ElemType::F32),
    write_slot(3, ElemType::F32),
    scalar_slot(4),
    scalar_slot(5),
    scalar_slot(6),
    scalar_slot(7),
];

/// tendon_indices：元数据、占位参数、空间整数索引与索引输出，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const INDICES_SLOTS: [SlotDesc; 8] = [
    meta_slot(),
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::I32),
    write_slot(3, ElemType::I32),
    scalar_slot(4),
    scalar_slot(5),
    scalar_slot(6),
    scalar_slot(7),
];

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TendonStaticDims {
    pub(super) nt: u32,
    pub(super) nnz: u32,
    pub(super) nw: u32,
    pub(super) fixed_nt: u32,
    pub(super) fixed_nnz: u32,
    pub(super) spatial_nt: u32,
    pub(super) spatial_nnz: u32,
    pub(super) spatial_nw: u32,
    pub(super) parameters_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl TendonStaticDims {
    /// 由全局与两类局部计数构造；参数缓冲至少一项。
    pub(super) fn new(
        counts: (usize, usize, usize),
        fixed: (usize, usize),
        spatial: (usize, usize, usize),
        parameters_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nt: value("resident_tendon_nt", counts.0)?,
            nnz: value("resident_tendon_nnz", counts.1)?,
            nw: value("resident_tendon_nw", counts.2)?,
            fixed_nt: value("resident_tendon_fixed_nt", fixed.0)?,
            fixed_nnz: value("resident_tendon_fixed_nnz", fixed.1)?,
            spatial_nt: value("resident_tendon_spatial_nt", spatial.0)?,
            spatial_nnz: value("resident_tendon_spatial_nnz", spatial.1)?,
            spatial_nw: value("resident_tendon_spatial_nw", spatial.2)?,
            parameters_len: value("resident_tendon_parameters", parameters_len)?,
        };
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：八项头加每肌腱五项。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.nt)
            .checked_mul(5)
            .and_then(|rows| rows.checked_add(8))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_tendon_metadata"))
    }

    /// 合并结果每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nt)
            .checked_add(u64::from(self.nnz))
            .and_then(|value| {
                u64::from(self.nw)
                    .checked_mul(6)
                    .and_then(|wraps| value.checked_add(wraps))
            })
            .ok_or(LauncherError::Overflow("resident_tendon_output"))
    }

    /// 索引输出每世界元素数。
    pub(super) fn indices_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nt)
            .checked_add(u64::from(self.nw))
            .and_then(|value| value.checked_mul(2))
            .ok_or(LauncherError::Overflow("resident_tendon_indices"))
    }

    /// 空间包裹整数缓冲每世界元素数。
    pub(super) fn spatial_ids_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.spatial_nt)
            .checked_add(u64::from(self.spatial_nw))
            .and_then(|value| value.checked_mul(2))
            .ok_or(LauncherError::Overflow("resident_tendon_spatial_ids"))
    }

    /// 合并输入每世界元素数；`kind`为0取固定肌腱，1取空间肌腱。
    pub(super) fn state_stride(self, kind: u32) -> Result<u64, LauncherError> {
        let width = match kind {
            0 => u64::from(self.fixed_nt).checked_add(u64::from(self.fixed_nnz)),
            1 => u64::from(self.spatial_nt)
                .checked_add(u64::from(self.spatial_nnz))
                .and_then(|value| {
                    u64::from(self.spatial_nw)
                        .checked_mul(19)
                        .and_then(|wraps| value.checked_add(wraps))
                }),
            _ => return Err(LauncherError::InvalidArray("常驻肌腱合并类别无效")),
        };
        width.ok_or(LauncherError::Overflow("resident_tendon_state"))
    }
}

/// 肌腱合并与索引发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TendonDims {
    pub(super) static_dims: TendonStaticDims,
    pub(super) worlds: u32,
    pub(super) kind: u32,
}

#[cfg(feature = "cuda-probe")]
impl TendonDims {
    /// 冻结ABI：合并类别、两个保留零与世界数。
    pub(super) fn abi(self) -> [u32; 4] {
        [self.kind, 0, 0, self.worlds]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻肌腱世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_tendon_guarded"))
}

/// 合并：空间索引、源结果与合并输出。
#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_values_lengths(dims: &TendonDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        guarded(dims.worlds, static_dims.spatial_ids_stride()?)?,
        guarded(dims.worlds, static_dims.state_stride(dims.kind)?)?,
        guarded(dims.worlds, static_dims.output_stride()?)?,
    ])
}

/// 索引：占位参数、空间索引与索引输出。
#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_indices_lengths(dims: &TendonDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        u64::from(static_dims.parameters_len).max(1),
        guarded(dims.worlds, static_dims.spatial_ids_stride()?)?,
        guarded(dims.worlds, static_dims.indices_stride()?)?,
    ])
}

/// 校验元数据：八项头与每肌腱的类别、局部编号、偏移与全局CSR行。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_tendon_metadata(
    values: &[i32],
    dims: TendonStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("常驻肌腱元数据长度无效"));
    }
    let header = [
        dims.nt as i32,
        dims.nnz as i32,
        dims.nw as i32,
        dims.fixed_nt as i32,
        dims.fixed_nnz as i32,
        dims.spatial_nt as i32,
        dims.spatial_nnz as i32,
        dims.spatial_nw as i32,
    ];
    if values[..8] != header {
        return Err(LauncherError::InvalidMetadata("常驻肌腱头计数不符"));
    }
    let nt = usize::try_from(dims.nt).map_err(|_| LauncherError::Overflow("resident_tendon_nt"))?;
    for tendon in 0..nt {
        let row = &values[8 + 5 * tendon..13 + 5 * tendon];
        let kind = row[0];
        if kind != 0 && kind != 1 {
            return Err(LauncherError::InvalidMetadata("常驻肌腱合并类别越界"));
        }
        let local = u64::try_from(row[1])
            .map_err(|_| LauncherError::InvalidMetadata("常驻肌腱局部编号为负"))?;
        let offset = u64::try_from(row[2])
            .map_err(|_| LauncherError::InvalidMetadata("常驻肌腱局部偏移为负"))?;
        let address = u64::try_from(row[3])
            .map_err(|_| LauncherError::InvalidMetadata("常驻肌腱全局行为负"))?;
        let count = u64::try_from(row[4])
            .map_err(|_| LauncherError::InvalidMetadata("常驻肌腱全局计数为负"))?;
        let (local_nt, local_nnz) = if kind == 0 {
            (dims.fixed_nt, dims.fixed_nnz)
        } else {
            (dims.spatial_nt, dims.spatial_nnz)
        };
        if local >= u64::from(local_nt) {
            return Err(LauncherError::InvalidMetadata("常驻肌腱局部编号越界"));
        }
        if offset
            .checked_add(count)
            .is_none_or(|end| end > u64::from(local_nnz))
        {
            return Err(LauncherError::InvalidMetadata("常驻肌腱局部偏移越界"));
        }
        if address
            .checked_add(count)
            .is_none_or(|end| end > u64::from(dims.nnz))
        {
            return Err(LauncherError::InvalidMetadata("常驻肌腱全局行越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonValuesSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结tendon_values FIELD_BATCH ABI。
unsafe impl KernelAbi for TendonValuesSpec {
    const NAME: &'static str = "tendon_values";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &VALUES_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for TendonValuesSpec {
    type Dims = TendonDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for TendonValuesSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(tendon_values_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonIndicesSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结tendon_indices FIELD_BATCH ABI。
unsafe impl KernelAbi for TendonIndicesSpec {
    const NAME: &'static str = "tendon_indices";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &INDICES_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for TendonIndicesSpec {
    type Dims = TendonDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for TendonIndicesSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(tendon_indices_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for TendonMetadata {
    type Dims = TendonDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_tendon_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_tendon_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_tendon_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_tendon_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_tendon_abi",
        }
        .into(),
    }
}

/// 肌腱合并元数据缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) type TendonMetadataBuffer = FieldMetadataBuffer<TendonMetadata>;

/// 肌腱合并与索引同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct ResidentTendonKernel {
    values: SynchronousKernel,
    indices: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentTendonKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            values: SynchronousKernel::compile(
                session,
                crate::physics::tendon::TENDON_CUDA,
                "tendon_values",
            )?,
            indices: SynchronousKernel::compile(
                session,
                crate::physics::tendon::TENDON_CUDA,
                "tendon_indices",
            )?,
        })
    }

    /// 合并段：按类别读取固定或空间肌腱结果。
    pub(super) fn launch_values(
        &self,
        metadata: &TendonMetadataBuffer,
        spatial_ids: &TransferBuffer<i32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
        kind: u32,
    ) -> Result<(), TransferError> {
        let dims = TendonDims {
            static_dims: metadata.base().static_dims,
            worlds,
            kind,
        };
        launch_field_batch::<TendonValuesSpec, TendonMetadata, i32, f32, f32>(
            &self.values,
            metadata,
            &dims,
            spatial_ids,
            state,
            output,
        )
    }

    /// 索引段：把空间包裹编号映射为全局紧凑索引。
    pub(super) fn launch_indices(
        &self,
        metadata: &TendonMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        spatial_ids: &TransferBuffer<i32>,
        output: &mut TransferBuffer<i32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = TendonDims {
            static_dims: metadata.base().static_dims,
            worlds,
            kind: 0,
        };
        launch_field_batch::<TendonIndicesSpec, TendonMetadata, f32, i32, i32>(
            &self.indices,
            metadata,
            &dims,
            parameters,
            spatial_ids,
            output,
        )
    }
}
