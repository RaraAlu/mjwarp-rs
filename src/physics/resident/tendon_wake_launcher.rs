//! 常驻肌腱唤醒 FIELD_BATCH 变体的槽位、证明与同步发射。

#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldBatchSpec;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_launch_dims;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_spans;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::SLEEP_COPY_METADATA_KIND;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::SleepCopyMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::sleep_copy_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称；与休眠复制共用同一份睡眠元数据。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = SLEEP_COPY_METADATA_KIND;

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

/// 冻结ABI：元数据、参数、肌腱长度与i32状态输出，后接四个标量。
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
            elem: ElemType::I32,
        },
        access: Access::Write,
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
pub(super) struct TendonWakeStaticDims {
    pub(super) ntree: u32,
    pub(super) metadata_elements: u64,
    pub(super) parameter_elements: u64,
    pub(super) lengths_stride: u64,
    pub(super) scratch_stride: u64,
}

/// 肌腱唤醒源/状态跨度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug)]
pub(super) struct TendonWakeSpans {
    pub(super) lengths_stride: u64,
    pub(super) scratch_stride: u64,
}

/// 肌腱唤醒发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TendonWakeDims {
    pub(super) abi: [u32; 4],
    pub(super) static_dims: TendonWakeStaticDims,
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻肌腱唤醒世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_tendon_wake_guarded"))
}

/// 四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_wake_lengths(dims: &TendonWakeDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let worlds = dims.abi[3];
    Ok([
        static_dims.metadata_elements,
        static_dims.parameter_elements.max(1),
        guarded(worlds, static_dims.lengths_stride)?,
        guarded(worlds, static_dims.scratch_stride)?,
    ])
}

/// 校验睡眠元数据：长度、树表、肌腱行、限位标志与参数区间。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_wake_metadata(
    values: &[i32],
    dims: TendonWakeStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements) {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒元数据长度无效"));
    }
    let ntree = u64::from(dims.ntree);
    if values[0] < 0 || values[0] as u64 != ntree {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒树数量不符"));
    }
    if dims.ntree == 0 || dims.scratch_stride != 2 * ntree + 3 {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒状态跨度不符"));
    }
    let nt = u64::try_from(values[1])
        .map_err(|_| LauncherError::InvalidMetadata("常驻肌腱唤醒肌腱数为负"))?;
    let nw = u64::try_from(values[2])
        .map_err(|_| LauncherError::InvalidMetadata("常驻肌腱唤醒包裹数为负"))?;
    if values[3] < 1 || values[4] < 1 {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒参数批次无效"));
    }
    if values[5] < 0 || u64::try_from(values[5]).ok() != Some(dims.lengths_stride) {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒长度跨度不符"));
    }
    if values[6] != 0 && values[6] != 1 {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒使能标志无效"));
    }
    if dims.metadata_elements != 7 + 3 * nt + nw {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒元数据结构不符"));
    }
    let batches = u64::try_from(values[3])
        .ok()
        .and_then(|range| {
            u64::try_from(values[4])
                .ok()
                .and_then(|margin| range.checked_mul(2)?.checked_add(margin))
        })
        .and_then(|factor| factor.checked_mul(nt))
        .ok_or(LauncherError::Overflow("resident_tendon_wake_parameters"))?;
    if batches > dims.parameter_elements {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒参数区间越界"));
    }
    let nt = usize::try_from(nt).map_err(|_| LauncherError::Overflow("resident_tendon_wake_nt"))?;
    let nw = usize::try_from(nw).map_err(|_| LauncherError::Overflow("resident_tendon_wake_nw"))?;
    let mut cursor = 7;
    let adr = &values[cursor..cursor + nt];
    cursor += nt;
    let num = &values[cursor..cursor + nt];
    cursor += nt;
    let limited = &values[cursor..cursor + nt];
    cursor += nt;
    let trees = &values[cursor..cursor + nw];
    let mut row = 0usize;
    for tendon in 0..nt {
        if adr[tendon] < 0 || num[tendon] < 0 {
            return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒行区间为负"));
        }
        if adr[tendon] as usize != row {
            return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒行未连续覆盖"));
        }
        row = row
            .checked_add(num[tendon] as usize)
            .ok_or(LauncherError::Overflow("resident_tendon_wake_rows"))?;
    }
    if row != nw {
        return Err(LauncherError::InvalidMetadata(
            "常驻肌腱唤醒行数与包裹数不符",
        ));
    }
    if limited.iter().any(|&flag| flag != 0 && flag != 1) {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒限位标志无效"));
    }
    if trees
        .iter()
        .any(|&tree| tree < -1 || (tree >= 0 && tree as u32 >= dims.ntree))
    {
        return Err(LauncherError::InvalidMetadata("常驻肌腱唤醒树索引越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonWakeSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结tendon_wake FIELD_BATCH ABI。
unsafe impl KernelAbi for TendonWakeSpec {
    const NAME: &'static str = "tendon_wake";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for TendonWakeSpec {
    type Dims = TendonWakeDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for TendonWakeSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(tendon_wake_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonWakeMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for TendonWakeMetadata {
    type Dims = TendonWakeDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_wake_metadata(values, dims.static_dims)
    }
}

/// 复用睡眠元数据缓冲，重建唤醒证明；写入代际与设备跨度必须一致。
#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_wake_proof(
    metadata: &SleepCopyMetadataBuffer,
    static_dims: TendonWakeStaticDims,
    worlds: u32,
) -> Result<Checked<TendonWakeMetadata>, TransferError> {
    if metadata.dims.metadata_elements != static_dims.metadata_elements
        || metadata.dims.ntree != static_dims.ntree
    {
        return Err(InputError::InvalidTopology {
            field: "sleep_copy_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into());
    }
    let span = metadata.buffer.launcher_span()?;
    metadata
        .checked
        .binding(0)
        .validate_for(span, &metadata.dims)
        .map_err(sleep_copy_launch_error)?;
    let dims = TendonWakeDims {
        abi: [0, 0, 0, worlds],
        static_dims,
    };
    // SAFETY: 缓冲、宿主镜像与写入代际一致；世界数只影响跨度。
    unsafe {
        Checked::<TendonWakeMetadata>::assume_uploaded(
            span,
            &metadata.values,
            dims,
            metadata.revision.clone(),
        )
    }
    .map_err(sleep_copy_launch_error)
}

/// 肌腱唤醒同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct TendonWakeKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl TendonWakeKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::sleep::TENDON_WAKE_CUDA,
                "tendon_wake",
            )?,
        })
    }

    /// 先核对睡眠元数据证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &SleepCopyMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        lengths: &TransferBuffer<f32>,
        result: &mut TransferBuffer<i32>,
        spans: TendonWakeSpans,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = TendonWakeStaticDims {
            ntree: metadata.dims.ntree,
            metadata_elements: metadata.dims.metadata_elements,
            parameter_elements: u64::try_from(parameters.len()).map_err(|_| {
                InputError::Overflow {
                    field: "resident_tendon_wake_parameters",
                }
            })?,
            lengths_stride: spans.lengths_stride,
            scratch_stride: spans.scratch_stride,
        };
        let proof = tendon_wake_proof(metadata, static_dims, worlds)?;
        let dims = TendonWakeDims {
            abi: [0, 0, 0, worlds],
            static_dims,
        };
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.buffer.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: parameters.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: lengths.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: result.launcher_span()?,
                elem: ElemType::I32,
            },
        ];
        field_plan::<TendonWakeSpec>(&dims, &bindings, &[proof.binding(0)])
            .map_err(sleep_copy_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与256线程网格；
        // 内核只写状态输出槽且同步完成。
        unsafe {
            self.kernel
                .launch(&metadata.buffer, parameters, lengths, result, dims.abi)
        }
    }
}
