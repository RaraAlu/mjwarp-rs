//! 常驻柔体面 FIELD_BATCH 变体的槽位、证明、维度与同步发射。

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

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_flex_faces.metadata.v1");

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

/// 冻结ABI：元数据、参数、柔体节点输入与面输出，后接四个标量。
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
pub(super) struct FlexFacesStaticDims {
    pub(super) nface: u32,
    pub(super) nn: u32,
    pub(super) node_stride: u32,
    pub(super) parameters_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl FlexFacesStaticDims {
    /// 由面数、节点数与柔体节点输出跨度构造。
    pub(super) fn new(
        nface: usize,
        nn: usize,
        node_stride: usize,
        parameters_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nface: value("resident_flex_faces_nface", nface)?,
            nn: value("resident_flex_faces_nn", nn)?,
            node_stride: value("resident_flex_faces_nodes", node_stride)?,
            parameters_len: value("resident_flex_faces_parameters", parameters_len)?,
        };
        if dims.nface == 0 || dims.node_stride < 3 * dims.nn {
            return Err(LauncherError::InvalidArray("常驻柔体面维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：两项头加每面十项。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.nface)
            .checked_mul(10)
            .and_then(|rows| rows.checked_add(2))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_flex_faces_metadata"))
    }

    /// 面输出每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nface)
            .checked_mul(31)
            .ok_or(LauncherError::Overflow("resident_flex_faces_output"))
    }
}

/// 柔体面发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlexFacesDims {
    pub(super) static_dims: FlexFacesStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl FlexFacesDims {
    /// 冻结ABI：三个保留零与世界数。
    pub(super) fn abi(self) -> [u32; 4] {
        [0, 0, 0, self.worlds]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻柔体面世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_flex_faces_guarded"))
}

/// 四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn flex_faces_lengths(dims: &FlexFacesDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        u64::from(static_dims.parameters_len).max(1),
        guarded(dims.worlds, u64::from(static_dims.node_stride))?,
        guarded(dims.worlds, static_dims.output_stride()?)?,
    ])
}

/// 校验元数据：两项头、每面轴向与九个节点索引。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_flex_faces_metadata(
    values: &[i32],
    dims: FlexFacesStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("常驻柔体面元数据长度无效"));
    }
    if values[0] != dims.nface as i32 || values[1] != dims.node_stride as i32 {
        return Err(LauncherError::InvalidMetadata("常驻柔体面头计数不符"));
    }
    let nface = usize::try_from(dims.nface)
        .map_err(|_| LauncherError::Overflow("resident_flex_faces_nface"))?;
    for face in 0..nface {
        let row = &values[2 + 10 * face..12 + 10 * face];
        if !(0..=2).contains(&row[0]) {
            return Err(LauncherError::InvalidMetadata("常驻柔体面轴向越界"));
        }
        if row[1..]
            .iter()
            .any(|&node| node < -1 || (node >= 0 && node as u32 >= dims.nn))
        {
            return Err(LauncherError::InvalidMetadata("常驻柔体面节点索引越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexFacesSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结flex_faces FIELD_BATCH ABI。
unsafe impl KernelAbi for FlexFacesSpec {
    const NAME: &'static str = "flex_faces";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for FlexFacesSpec {
    type Dims = FlexFacesDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for FlexFacesSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(flex_faces_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexFacesMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for FlexFacesMetadata {
    type Dims = FlexFacesDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_flex_faces_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn flex_faces_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_flex_faces_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_flex_faces_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_flex_faces_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_flex_faces_abi",
        }
        .into(),
    }
}

/// 柔体面元数据缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) type FlexFacesMetadataBuffer = FieldMetadataBuffer<FlexFacesMetadata>;

/// 柔体面同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FlexFacesKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl FlexFacesKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::flex_face::FLEX_FACE_CUDA,
                "flex_faces",
            )?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &FlexFacesMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        nodes: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = FlexFacesDims {
            static_dims: metadata.base().static_dims,
            worlds,
        };
        launch_field_batch::<FlexFacesSpec, FlexFacesMetadata, f32, f32, f32>(
            &self.kernel,
            metadata,
            &dims,
            parameters,
            nodes,
            output,
        )
    }
}
