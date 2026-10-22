//! 常驻柔体边 FIELD_BATCH 变体的槽位、证明、维度与同步发射。

#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldBatchSpec;
#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_launch_dims;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_spans;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_flex_edges.metadata.v1");

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

/// 冻结ABI：元数据、qvel、质心、柔体位置与边输出，后接一个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const SLOTS: [SlotDesc; 6] = [
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
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlexEdgesStaticDims {
    pub(super) nb: u32,
    pub(super) nv: u32,
    pub(super) ne: u32,
    pub(super) nnz: u32,
    pub(super) nvert: u32,
    pub(super) com_stride: u32,
    pub(super) flex_stride: u32,
}

#[cfg(feature = "cuda-probe")]
impl FlexEdgesStaticDims {
    /// 由刚体、自由度、边与柔体顶点计数构造。
    pub(super) fn new(
        nb: usize,
        nv: usize,
        ne: usize,
        nnz: usize,
        nvert: usize,
        com_stride: usize,
        flex_stride: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nb: value("resident_flex_edges_nb", nb)?,
            nv: value("resident_flex_edges_nv", nv)?,
            ne: value("resident_flex_edges_ne", ne)?,
            nnz: value("resident_flex_edges_nnz", nnz)?,
            nvert: value("resident_flex_edges_nvert", nvert)?,
            com_stride: value("resident_flex_edges_com", com_stride)?,
            flex_stride: value("resident_flex_edges_flex", flex_stride)?,
        };
        // 冻结内核按nb寻址parent/root；单世界体（nb=1）模型合法。
        // 零边/零顶点也安全：内核只按ne/nvert循环，越界由元数据证明拦截。
        if dims.nb == 0 {
            return Err(LauncherError::InvalidArray("常驻柔体边维度无效"));
        }
        let expected_com = u64::from(dims.nb)
            .checked_mul(14)
            .and_then(|value| {
                u64::from(dims.nv)
                    .checked_mul(6)
                    .and_then(|dofs| value.checked_add(dofs))
            })
            .ok_or(LauncherError::Overflow("resident_flex_edges_com"))?;
        if expected_com != u64::from(dims.com_stride) {
            return Err(LauncherError::InvalidArray("常驻柔体边质心跨度不符"));
        }
        if u64::from(dims.flex_stride) < 3 * u64::from(dims.nvert) {
            return Err(LauncherError::InvalidArray("常驻柔体边位置跨度不足"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：七项头、边行、列索引、顶点体、体父链、根体与自由度体。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        let rows = u64::from(self.ne)
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("resident_flex_edges_metadata"))?;
        rows.checked_add(7)
            .and_then(|value| value.checked_add(u64::from(self.nnz)))
            .and_then(|value| value.checked_add(u64::from(self.nvert)))
            .and_then(|value| value.checked_add(2 * u64::from(self.nb)))
            .and_then(|value| value.checked_add(u64::from(self.nv)))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_flex_edges_metadata"))
    }

    /// 边输出每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.ne)
            .checked_mul(2)
            .and_then(|value| value.checked_add(u64::from(self.nnz)))
            .ok_or(LauncherError::Overflow("resident_flex_edges_output"))
    }
}

/// 柔体边发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlexEdgesDims {
    pub(super) static_dims: FlexEdgesStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl FlexEdgesDims {
    /// 冻结ABI：三个保留零与世界数。
    pub(super) fn abi(self) -> [u32; 4] {
        [0, 0, 0, self.worlds]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻柔体边世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_flex_edges_guarded"))
}

/// 五槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn flex_edges_lengths(dims: &FlexEdgesDims) -> Result<[u64; 5], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        guarded(dims.worlds, u64::from(static_dims.nv))?,
        guarded(dims.worlds, u64::from(static_dims.com_stride))?,
        guarded(dims.worlds, u64::from(static_dims.flex_stride))?,
        guarded(dims.worlds, static_dims.output_stride()?)?,
    ])
}

/// 校验元数据：七项头、边行与稀疏行、顶点体、体父链、根体与自由度体。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_flex_edges_metadata(
    values: &[i32],
    dims: FlexEdgesStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("常驻柔体边元数据长度无效"));
    }
    let header = [
        dims.nb as i32,
        dims.nv as i32,
        dims.ne as i32,
        dims.nnz as i32,
        dims.com_stride as i32,
        dims.flex_stride as i32,
        dims.nvert as i32,
    ];
    if values[..7] != header {
        return Err(LauncherError::InvalidMetadata("常驻柔体边头计数不符"));
    }
    let nb =
        usize::try_from(dims.nb).map_err(|_| LauncherError::Overflow("resident_flex_edges_nb"))?;
    let ne =
        usize::try_from(dims.ne).map_err(|_| LauncherError::Overflow("resident_flex_edges_ne"))?;
    let nnz = usize::try_from(dims.nnz)
        .map_err(|_| LauncherError::Overflow("resident_flex_edges_nnz"))?;
    let nvert = usize::try_from(dims.nvert)
        .map_err(|_| LauncherError::Overflow("resident_flex_edges_nvert"))?;
    let mut cursor = 7;
    let rows = &values[cursor..cursor + 4 * ne];
    cursor += 4 * ne;
    let cols = &values[cursor..cursor + nnz];
    cursor += nnz;
    let vertbody = &values[cursor..cursor + nvert];
    cursor += nvert;
    let parent = &values[cursor..cursor + nb];
    cursor += nb;
    let root = &values[cursor..cursor + nb];
    cursor += nb;
    let dofbody = &values[cursor..];
    for edge in 0..ne {
        let row = &rows[4 * edge..4 * edge + 4];
        if row[0] < 0 || row[0] as u32 >= dims.nvert {
            return Err(LauncherError::InvalidMetadata("常驻柔体边端点越界"));
        }
        if row[1] < 0 || row[1] as u32 >= dims.nvert {
            return Err(LauncherError::InvalidMetadata("常驻柔体边端点越界"));
        }
        if row[2] < 0 || row[3] < 0 {
            return Err(LauncherError::InvalidMetadata("常驻柔体边稀疏行为负"));
        }
        if (row[2] as usize)
            .checked_add(row[3] as usize)
            .is_none_or(|end| end > nnz)
        {
            return Err(LauncherError::InvalidMetadata("常驻柔体边稀疏行越界"));
        }
    }
    if cols.iter().any(|&dof| dof < 0 || dof as u32 >= dims.nv) {
        return Err(LauncherError::InvalidMetadata("常驻柔体边自由度索引越界"));
    }
    // 顶点体为负值时内核跳过该边；非负值必须指向有效体（含世界体0）。
    if vertbody
        .iter()
        .any(|&body| body >= 0 && body as u32 >= dims.nb)
    {
        return Err(LauncherError::InvalidMetadata("常驻柔体边顶点体越界"));
    }
    // MuJoCo根体父引用为0，部分来源为-1；两者都只在根体0不被使用时安全。
    if parent[0] != -1 && parent[0] != 0 {
        return Err(LauncherError::InvalidMetadata("常驻柔体边根体父引用无效"));
    }
    for (body, &parent) in parent.iter().enumerate().skip(1) {
        if parent < 0 || parent as usize >= body {
            return Err(LauncherError::InvalidMetadata("常驻柔体边体父链无效"));
        }
    }
    // 世界体作为顶点体时要求根体0可寻址。
    let uses_world = vertbody.contains(&0);
    let root0 = if uses_world {
        root[0] >= 0 && (root[0] as u32) < dims.nb
    } else {
        root[0] == -1 || root[0] == 0
    };
    if !root0 {
        return Err(LauncherError::InvalidMetadata("常驻柔体边根体索引无效"));
    }
    if root
        .iter()
        .enumerate()
        .skip(1)
        .any(|(body, &root)| root < 1 || root as usize > body || root as usize >= nb)
    {
        return Err(LauncherError::InvalidMetadata("常驻柔体边根体表越界"));
    }
    if dofbody
        .iter()
        .any(|&body| body < 0 || body as u32 >= dims.nb)
    {
        return Err(LauncherError::InvalidMetadata("常驻柔体边自由度体越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexEdgesSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结flex_edges FIELD_BATCH ABI。
unsafe impl KernelAbi for FlexEdgesSpec {
    const NAME: &'static str = "flex_edges";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for FlexEdgesSpec {
    type Dims = FlexEdgesDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for FlexEdgesSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(flex_edges_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexEdgesMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for FlexEdgesMetadata {
    type Dims = FlexEdgesDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_flex_edges_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn flex_edges_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_flex_edges_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_flex_edges_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_flex_edges_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_flex_edges_abi",
        }
        .into(),
    }
}

/// 柔体边元数据缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) type FlexEdgesMetadataBuffer = FieldMetadataBuffer<FlexEdgesMetadata>;

/// 柔体边同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FlexEdgesKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl FlexEdgesKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::flex_edge::FLEX_EDGE_CUDA,
                "flex_edges",
            )?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &FlexEdgesMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        flex: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = FlexEdgesDims {
            static_dims: metadata.base().static_dims,
            worlds,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            metadata.binding()?,
            SlotBinding {
                slot: 1,
                span: qvel.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: flex.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        field_plan::<FlexEdgesSpec>(&dims, &bindings, &[proof.binding(0)])
            .map_err(flex_edges_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与256线程网格。
        unsafe {
            self.kernel
                .launch_flex_edges(metadata.as_buffer(), qvel, com, flex, output, worlds)
        }
    }
}
