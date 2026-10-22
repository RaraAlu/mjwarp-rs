//! 常驻相机与光源 FIELD_BATCH 变体的槽位、证明、维度与同步发射。

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

/// 元数据证明名称；两段共用同一模型元数据。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_camlight.metadata.v1");

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

/// 冻结ABI：元数据、模型参数、刚体或质心输入与位姿输出，后接四个标量。
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
pub(super) struct CamLightStaticDims {
    pub(super) nb: u32,
    pub(super) rigid_stride: u32,
    pub(super) com_stride: u32,
    pub(super) ncam: u32,
    pub(super) nlight: u32,
    pub(super) parameters_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl CamLightStaticDims {
    pub(super) fn new(
        nb: usize,
        rigid_stride: usize,
        com_stride: usize,
        ncam: usize,
        nlight: usize,
        parameters_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nb: value("resident_camlight_nb", nb)?,
            rigid_stride: value("resident_camlight_rigid", rigid_stride)?,
            com_stride: value("resident_camlight_com", com_stride)?,
            ncam: value("resident_camlight_cameras", ncam)?,
            nlight: value("resident_camlight_lights", nlight)?,
            parameters_len: value("resident_camlight_parameters", parameters_len)?,
        };
        if dims.nb == 0 {
            return Err(LauncherError::InvalidArray("常驻相机维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：两列跨度、三列相机、三列光源与十对描述符。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        let counts = [
            u64::from(self.ncam).checked_mul(3),
            u64::from(self.nlight).checked_mul(3),
        ];
        let mut total = 22u64;
        for count in counts {
            total = total
                .checked_add(count.ok_or(LauncherError::Overflow("resident_camlight_metadata"))?)
                .ok_or(LauncherError::Overflow("resident_camlight_metadata"))?;
        }
        if total > i32::MAX as u64 {
            return Err(LauncherError::Overflow("resident_camlight_metadata"));
        }
        Ok(total)
    }

    /// 位姿输出每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.ncam)
            .checked_mul(12)
            .and_then(|cams| {
                u64::from(self.nlight)
                    .checked_mul(6)
                    .and_then(|lights| cams.checked_add(lights))
            })
            .ok_or(LauncherError::Overflow("resident_camlight_output"))
    }
}

/// 相机与光源发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CamLightDims {
    pub(super) static_dims: CamLightStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl CamLightDims {
    /// 冻结ABI：刚体数、相机数、光源数与世界数。
    pub(super) fn abi(self) -> [u32; 4] {
        [
            self.static_dims.nb,
            self.static_dims.ncam,
            self.static_dims.nlight,
            self.worlds,
        ]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻相机世界数无效"));
    }
    Ok(u64::from(worlds))
}

/// 精确元素数；`is_rigid`选择刚体或质心输入跨度。
#[cfg(feature = "cuda-probe")]
fn camlight_lengths(dims: &CamLightDims, is_rigid: bool) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = u64::from(static_dims.parameters_len).max(1);
    let input_stride = if is_rigid {
        u64::from(static_dims.rigid_stride)
    } else {
        u64::from(static_dims.com_stride)
    };
    let worlds = world_count(dims.worlds)?;
    let input = worlds
        .checked_mul(input_stride)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_camlight_input"))?;
    let output = worlds
        .checked_mul(static_dims.output_stride()?)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_camlight_output"))?;
    Ok([metadata, parameters, input, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) fn camlight_rigid_lengths(dims: &CamLightDims) -> Result<[u64; 4], LauncherError> {
    camlight_lengths(dims, true)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn camlight_com_lengths(dims: &CamLightDims) -> Result<[u64; 4], LauncherError> {
    camlight_lengths(dims, false)
}

/// 校验元数据：跨度头、模式、体索引与十对字段描述符区间。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_camlight_metadata(
    values: &[i32],
    dims: CamLightStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("常驻相机元数据长度无效"));
    }
    let nc = usize::try_from(dims.ncam)
        .map_err(|_| LauncherError::Overflow("resident_camlight_cameras"))?;
    let nl = usize::try_from(dims.nlight)
        .map_err(|_| LauncherError::Overflow("resident_camlight_lights"))?;
    if values[0] != dims.rigid_stride as i32 || values[1] != dims.com_stride as i32 {
        return Err(LauncherError::InvalidMetadata("常驻相机跨度头不匹配"));
    }
    let modes = &values[2..2 + nc];
    let bodies = &values[2 + nc..2 + 2 * nc];
    let targets = &values[2 + 2 * nc..2 + 3 * nc];
    let light_modes = &values[2 + 3 * nc..2 + 3 * nc + nl];
    let light_bodies = &values[2 + 3 * nc + nl..2 + 3 * nc + 2 * nl];
    let light_targets = &values[2 + 3 * nc + 2 * nl..2 + 3 * nc + 3 * nl];
    if modes.iter().any(|&mode| !(0..=4).contains(&mode)) {
        return Err(LauncherError::InvalidMetadata("常驻相机模式越界"));
    }
    if light_modes.iter().any(|&mode| !(0..=4).contains(&mode)) {
        return Err(LauncherError::InvalidMetadata("常驻光源模式越界"));
    }
    for (bodies, targets) in [(bodies, targets), (light_bodies, light_targets)] {
        if bodies
            .iter()
            .any(|&body| body < 0 || body as u32 >= dims.nb)
        {
            return Err(LauncherError::InvalidMetadata("常驻相机体索引越界"));
        }
        // 负值是"无目标"哨兵；内核只在非负时使用。
        if targets
            .iter()
            .any(|&target| target >= 0 && target as u32 >= dims.nb)
        {
            return Err(LauncherError::InvalidMetadata("常驻相机目标体越界"));
        }
    }
    let descriptors = &values[2 + 3 * nc + 3 * nl..];
    let widths = [
        3 * nc,
        4 * nc,
        3 * nc,
        3 * nc,
        9 * nc,
        3 * nl,
        3 * nl,
        3 * nl,
        3 * nl,
        3 * nl,
    ];
    let parameters = u64::from(dims.parameters_len);
    for (index, width) in widths.into_iter().enumerate() {
        let offset = descriptors[2 * index];
        let batches = descriptors[2 * index + 1];
        if offset < 0 || batches <= 0 {
            return Err(LauncherError::InvalidMetadata("常驻相机字段区间为负"));
        }
        let end = u64::try_from(offset)
            .ok()
            .and_then(|value| {
                u64::try_from(batches)
                    .ok()?
                    .checked_mul(u64::try_from(width).ok()?)
                    .and_then(|batch| value.checked_add(batch))
            })
            .ok_or(LauncherError::Overflow("resident_camlight_parameters"))?;
        if end > parameters {
            return Err(LauncherError::InvalidMetadata("常驻相机字段区间越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct CamLightRigidSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结camlight_rigid FIELD_BATCH ABI。
unsafe impl KernelAbi for CamLightRigidSpec {
    const NAME: &'static str = "camlight_rigid";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for CamLightRigidSpec {
    type Dims = CamLightDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for CamLightRigidSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(camlight_rigid_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct CamLightComSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结camlight_com FIELD_BATCH ABI。
unsafe impl KernelAbi for CamLightComSpec {
    const NAME: &'static str = "camlight_com";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for CamLightComSpec {
    type Dims = CamLightDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for CamLightComSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(camlight_com_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct CamLightMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for CamLightMetadata {
    type Dims = CamLightDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_camlight_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn camlight_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_camlight_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_camlight_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_camlight_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_camlight_abi",
        }
        .into(),
    }
}

/// 相机与光源元数据缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) type CamLightMetadataBuffer = FieldMetadataBuffer<CamLightMetadata>;

/// 相机与光源两段同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct ResidentCamLightKernel {
    rigid: SynchronousKernel,
    com: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCamLightKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            rigid: SynchronousKernel::compile(session, source, "camlight_rigid")?,
            com: SynchronousKernel::compile(session, source, "camlight_com")?,
        })
    }

    /// 刚体段：核对证明、精确跨度、同会话、别名与256线程网格后发射。
    pub(super) fn launch_rigid(
        &self,
        metadata: &CamLightMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        rigid: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = CamLightDims {
            static_dims: metadata.base().static_dims,
            worlds,
        };
        launch_field_batch::<CamLightRigidSpec, CamLightMetadata, f32, f32, f32>(
            &self.rigid,
            metadata,
            &dims,
            parameters,
            rigid,
            output,
        )
    }

    /// 质心段：只调整质心相关模式，读取同一世界的位姿输出。
    pub(super) fn launch_com(
        &self,
        metadata: &CamLightMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = CamLightDims {
            static_dims: metadata.base().static_dims,
            worlds,
        };
        launch_field_batch::<CamLightComSpec, CamLightMetadata, f32, f32, f32>(
            &self.com, metadata, &dims, parameters, com, output,
        )
    }
}
