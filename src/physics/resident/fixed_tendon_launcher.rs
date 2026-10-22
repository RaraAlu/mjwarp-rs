//! 常驻固定肌腱 FIELD_BATCH 变体的槽位、证明、维度与同步发射。

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
use super::pack::PackedModel;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_fixed_tendon.metadata.v1");

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
const fn scalar_slot(index: usize) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    }
}

/// 冻结ABI：元数据、系数、常驻状态与固定肌腱输出，后接四个标量。
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
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::F32),
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
pub(super) struct FixedTendonStaticDims {
    pub(super) nq: u32,
    pub(super) nt: u32,
    pub(super) nnz: u32,
    pub(super) nwrap: u32,
    pub(super) nmocap: u32,
    pub(super) parameters_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl FixedTendonStaticDims {
    pub(super) fn new(
        nq: usize,
        nt: usize,
        nnz: usize,
        nwrap: usize,
        nmocap: usize,
        parameters_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nq: value("resident_fixed_tendon_nq", nq)?,
            nt: value("resident_fixed_tendon_nt", nt)?,
            nnz: value("resident_fixed_tendon_nnz", nnz)?,
            nwrap: value("resident_fixed_tendon_nwrap", nwrap)?,
            nmocap: value("resident_fixed_tendon_nmocap", nmocap)?,
            parameters_len: value("resident_fixed_tendon_parameters", parameters_len)?,
        };
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：包裹数、两列肌腱行与两列解算地址。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        let tendons = u64::from(self.nt)
            .checked_mul(2)
            .ok_or(LauncherError::Overflow("resident_fixed_tendon_metadata"))?;
        let wraps = u64::from(self.nwrap)
            .checked_mul(2)
            .ok_or(LauncherError::Overflow("resident_fixed_tendon_metadata"))?;
        tendons
            .checked_add(wraps)
            .and_then(|value| value.checked_add(1))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_fixed_tendon_metadata"))
    }

    /// 常驻状态每世界元素数：qpos后接mocap字段。
    pub(super) fn state_width(self) -> Result<u64, LauncherError> {
        u64::from(self.nmocap)
            .checked_mul(7)
            .and_then(|value| value.checked_add(u64::from(self.nq)))
            .ok_or(LauncherError::Overflow("resident_fixed_tendon_state"))
    }

    /// 固定肌腱输出每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nt)
            .checked_add(u64::from(self.nnz))
            .ok_or(LauncherError::Overflow("resident_fixed_tendon_output"))
    }
}

/// 固定肌腱发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FixedTendonDims {
    pub(super) static_dims: FixedTendonStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl FixedTendonDims {
    /// 冻结ABI：qpos宽度、肌腱数、非零数与世界数。
    pub(super) fn abi(self) -> [u32; 4] {
        [
            self.static_dims.nq,
            self.static_dims.nt,
            self.static_dims.nnz,
            self.worlds,
        ]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻固定肌腱世界数无效"));
    }
    Ok(u64::from(worlds))
}

/// 四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn fixed_tendon_lengths(dims: &FixedTendonDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = u64::from(static_dims.parameters_len).max(1);
    let worlds = world_count(dims.worlds)?;
    let state = worlds
        .checked_mul(static_dims.state_width()?)
        .ok_or(LauncherError::Overflow("resident_fixed_tendon_state"))?;
    let output = worlds
        .checked_mul(static_dims.output_stride()?)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_fixed_tendon_output"))?;
    Ok([metadata, parameters, state, output])
}

/// 校验元数据：包裹数、肌腱行连续覆盖、qpos与力臂地址范围。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_fixed_tendon_metadata(
    values: &[i32],
    dims: FixedTendonStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("常驻固定肌腱元数据长度无效"));
    }
    let nt = usize::try_from(dims.nt)
        .map_err(|_| LauncherError::Overflow("resident_fixed_tendon_nt"))?;
    let nw = usize::try_from(dims.nwrap)
        .map_err(|_| LauncherError::Overflow("resident_fixed_tendon_nwrap"))?;
    if values[0] < 0 || values[0] as usize != nw {
        return Err(LauncherError::InvalidMetadata("常驻固定肌腱包裹数不匹配"));
    }
    let adr = &values[1..1 + nt];
    let num = &values[1 + nt..1 + 2 * nt];
    let qadr = &values[1 + 2 * nt..1 + 2 * nt + nw];
    let jslot = &values[1 + 2 * nt + nw..];
    let mut cursor = 0usize;
    for tendon in 0..nt {
        if num[tendon] < 0 || adr[tendon] < 0 {
            return Err(LauncherError::InvalidMetadata("常驻固定肌腱行区间为负"));
        }
        if adr[tendon] as usize != cursor {
            return Err(LauncherError::InvalidMetadata("常驻固定肌腱行未连续覆盖"));
        }
        cursor = cursor
            .checked_add(num[tendon] as usize)
            .ok_or(LauncherError::Overflow("resident_fixed_tendon_rows"))?;
    }
    if cursor != nw {
        return Err(LauncherError::InvalidMetadata(
            "常驻固定肌腱行数与包裹数不符",
        ));
    }
    if qadr
        .iter()
        .any(|&address| address < 0 || address as u32 >= dims.nq)
    {
        return Err(LauncherError::InvalidMetadata("常驻固定肌腱qpos地址越界"));
    }
    if jslot
        .iter()
        .any(|&slot| slot < 0 || slot as u32 >= dims.nnz)
    {
        return Err(LauncherError::InvalidMetadata("常驻固定肌腱力臂地址越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FixedTendonSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻固定肌腱FIELD_BATCH ABI。
unsafe impl KernelAbi for FixedTendonSpec {
    const NAME: &'static str = "fixed_tendon";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for FixedTendonSpec {
    type Dims = FixedTendonDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for FixedTendonSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(fixed_tendon_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FixedTendonMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for FixedTendonMetadata {
    type Dims = FixedTendonDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_fixed_tendon_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn fixed_tendon_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_fixed_tendon_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_fixed_tendon_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_fixed_tendon_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_fixed_tendon_abi",
        }
        .into(),
    }
}

/// 常驻固定肌腱元数据缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) type FixedTendonMetadataBuffer = FieldMetadataBuffer<FixedTendonMetadata>;

/// 常驻固定肌腱同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FixedTendonKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl FixedTendonKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::fixed_tendon::FIXED_TENDON_CUDA,
                "fixed_tendon",
            )?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &FixedTendonMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = FixedTendonDims {
            static_dims: metadata.base().static_dims,
            worlds,
        };
        launch_field_batch::<FixedTendonSpec, FixedTendonMetadata, f32, f32, f32>(
            &self.kernel,
            metadata,
            &dims,
            parameters,
            state,
            output,
        )
    }
}

/// 上传固定肌腱元数据、参数并编译内核。
#[cfg(feature = "cuda-probe")]
pub(super) fn upload_fixed_tendon(
    session: &TransferSession,
    packed: PackedModel,
    dims: (usize, usize, usize, usize, usize),
) -> Result<
    (
        FixedTendonMetadataBuffer,
        TransferBuffer<f32>,
        FixedTendonKernel,
    ),
    TransferError,
> {
    let (nq, nt, nnz, nwrap, nmocap) = dims;
    let static_dims =
        FixedTendonStaticDims::new(nq, nt, nnz, nwrap, nmocap, packed.parameters.len())
            .map_err(fixed_tendon_launch_error)?;
    let dims = FixedTendonDims {
        static_dims,
        worlds: 1,
    };
    let (metadata, parameters) =
        FixedTendonMetadataBuffer::from_packed(session, packed, dims, fixed_tendon_launch_error)?;
    Ok((metadata, parameters, FixedTendonKernel::compile(session)?))
}
