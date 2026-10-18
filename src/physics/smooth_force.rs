//! 四项广义力的常驻合并辅助。
//! 不累加空间外力，不计算平滑加速度。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, SmoothForceInput};
use crate::runtime::TransferSession;
#[cfg(all(feature = "cuda-probe", test))]
use crate::runtime::launcher::DeviceSpan;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;
mod joint_passive;
pub use joint_passive::{
    JointPassiveData, JointPassiveOptions, JointPassiveOutput, JointPassiveParameters,
    JointPassivePlan,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SmoothForceOptions {
    pub sleep_enabled: bool,
    pub island_disabled: bool,
}
impl SmoothForceOptions {
    pub fn enable_sleep(self) -> bool {
        self.sleep_enabled && !self.island_disabled
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmoothForceField {
    Passive,
    Bias,
    Actuator,
    Applied,
}
impl SmoothForceField {
    fn index(self) -> usize {
        match self {
            Self::Passive => 0,
            Self::Bias => 1,
            Self::Actuator => 2,
            Self::Applied => 3,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Passive => "qfrc_passive",
            Self::Bias => "qfrc_bias",
            Self::Actuator => "qfrc_actuator",
            Self::Applied => "qfrc_applied",
        }
    }
}
#[derive(Clone, Copy)]
struct Layout {
    vector: BatchLayout,
    awake: BatchLayout,
    // 默认配置无设备路径；仅cuda-probe读取。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    force_len: usize,
    meta_len: usize,
}
fn layout(worlds: usize, nbody: usize, nv: usize, ntree: usize) -> Result<Layout, InputError> {
    let overflow = || InputError::Overflow {
        field: "smooth_force_layout",
    };
    if worlds == 0
        || nbody == 0
        || nv == 0
        || worlds
            .checked_mul(nv)
            .is_none_or(|v| v > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "smooth_force_work",
        });
    }
    let vector = BatchLayout::new(worlds, nv, 4)?;
    let awake = BatchLayout::new(worlds, ntree, 4)?;
    let force_len = vector
        .total_elements()
        .checked_mul(4)
        .ok_or_else(overflow)?;
    let meta_len = nbody
        .checked_add(nv)
        .and_then(|v| v.checked_add(awake.total_elements()))
        .and_then(|v| v.checked_add(8))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or_else(overflow)?;
    for count in [force_len, vector.total_elements(), worlds] {
        BatchLayout::new(1, count.checked_add(8).ok_or_else(overflow)?, 4)?;
    }
    BatchLayout::new(1, meta_len, 4)?;
    Ok(Layout {
        vector,
        awake,
        force_len,
        meta_len,
    })
}

#[cfg(feature = "cuda-probe")]
const SMOOTH_FORCE_VALIDATE_METADATA_KIND: ProofKind = ProofKind("smooth_force_validate_metadata");

#[cfg(feature = "cuda-probe")]
const SMOOTH_FORCE_VALIDATE_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SMOOTH_FORCE_VALIDATE_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Write,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SmoothForceValidateDims {
    abi: [u32; 4],
    sleep_enabled: bool,
}

#[cfg(feature = "cuda-probe")]
struct SmoothForceValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结平滑力验证ABI。
unsafe impl KernelAbi for SmoothForceValidateSpec {
    const NAME: &'static str = "smooth_force_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SMOOTH_FORCE_VALIDATE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SmoothForceValidateSpec {
    type Dims = SmoothForceValidateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let [metadata, forces, _, status] = smooth_force_validate_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: forces,
            },
            // 关闭休眠时内核不读载荷，但底层ABI仍要求非空分配。
            SlotRequirement {
                slot: 2,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        smooth_force_validate_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn smooth_force_validate_lengths(
    dims: &SmoothForceValidateDims,
) -> Result<[u64; 4], LauncherError> {
    let [nb, n, nt, worlds] = dims.abi;
    if nb == 0 || n == 0 || worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("平滑力验证维度无效"));
    }
    let nb = u64::from(nb);
    let n = u64::from(n);
    let nt = u64::from(nt);
    let worlds = u64::from(worlds);
    let metadata = worlds
        .checked_mul(nt)
        .and_then(|value| value.checked_add(nb))
        .and_then(|value| value.checked_add(n))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("smooth_force_validate_metadata"))?;
    let forces = worlds
        .checked_mul(n)
        .and_then(|value| value.checked_mul(4))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("smooth_force_validate_forces"))?;
    let sleep = worlds
        .checked_mul(
            nt.checked_mul(2)
                .and_then(|value| value.checked_add(3))
                .ok_or(LauncherError::Overflow(
                    "smooth_force_validate_sleep_stride",
                ))?,
        )
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("smooth_force_validate_sleep"))?;
    let status = worlds
        .checked_add(8)
        .ok_or(LauncherError::Overflow("smooth_force_validate_status"))?;
    Ok([metadata, forces, sleep, status])
}

#[cfg(feature = "cuda-probe")]
fn smooth_force_validate_plan(
    dims: &SmoothForceValidateDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, SmoothForceValidateDims>],
) -> Result<(), LauncherError> {
    Kernel::<SmoothForceValidateSpec>::new()?.plan(dims, bindings, proofs)?;
    let lengths = smooth_force_validate_lengths(dims)?;
    let binding = |slot| {
        bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })
    };
    for slot in [0usize, 1, 3] {
        let actual = binding(slot)?.span.bytes();
        let required = lengths[slot]
            .checked_mul(binding(slot)?.elem.bytes())
            .ok_or(LauncherError::Overflow("smooth_force_validate_exact_bytes"))?;
        if actual != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual,
            });
        }
    }
    let metadata = binding(0)?;
    let forces = binding(1)?;
    let sleep = binding(2)?;
    let exact_metadata_alias = metadata.span == sleep.span;
    if dims.sleep_enabled && !exact_metadata_alias {
        let required = lengths[2]
            .checked_mul(sleep.elem.bytes())
            .ok_or(LauncherError::Overflow("smooth_force_validate_sleep_bytes"))?;
        if sleep.span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot: 2,
                required,
                actual: sleep.span.bytes(),
            });
        }
    }
    for (first, a, second, b) in [
        (0, metadata, 1, forces),
        (0, metadata, 2, sleep),
        (1, forces, 2, sleep),
    ] {
        if a.span.overlaps(&b.span) && !(first == 0 && second == 2 && exact_metadata_alias) {
            return Err(LauncherError::Alias { first, second });
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
fn validate_smooth_force_metadata(
    values: &[i32],
    dims: &SmoothForceValidateDims,
) -> Result<(), LauncherError> {
    let expected = smooth_force_validate_lengths(dims)?[0];
    if u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("平滑力元数据长度无效"));
    }
    if values[..4] != [-131072; 4] || values[values.len() - 4..] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("平滑力元数据守卫无效"));
    }
    let [nb, n, nt, _] = dims.abi;
    let nb = usize::try_from(nb).map_err(|_| LauncherError::Overflow("smooth_force_nb"))?;
    let n = usize::try_from(n).map_err(|_| LauncherError::Overflow("smooth_force_n"))?;
    let tree = &values[4..4 + nb];
    let body = &values[4 + nb..4 + nb + n];
    if tree
        .iter()
        .any(|&value| value < -1 || (value >= 0 && value as u32 >= nt))
    {
        return Err(LauncherError::InvalidMetadata("平滑力树索引越界"));
    }
    if body.iter().any(|&value| value < 0 || value as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("平滑力自由度体索引越界"));
    }
    // awake允许任意整数；零表示休眠，其他值表示唤醒。
    Ok(())
}

#[cfg(feature = "cuda-probe")]
struct SmoothForceValidateMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SmoothForceValidateMetadata {
    type Dims = SmoothForceValidateDims;

    const KIND: ProofKind = SMOOTH_FORCE_VALIDATE_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_smooth_force_metadata(values, dims)
    }
}

#[cfg(feature = "cuda-probe")]
fn smooth_force_validate_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "smooth_force_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_) | LauncherError::MetadataMismatch(_) => {
            InputError::InvalidTopology {
                field: "smooth_force_metadata",
                index: 0,
                reason: "invalid_metadata",
            }
            .into()
        }
        _ => InputError::InvalidDimension {
            field: "smooth_force_validate_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
struct SmoothForceMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    dims: SmoothForceValidateDims,
    revision: MetadataRevision,
    checked: Checked<SmoothForceValidateMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SmoothForceMetadataBuffer {
    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        dims: SmoothForceValidateDims,
    ) -> Result<Self, TransferError> {
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；跨度绑定该实际缓冲。
        let checked = unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(smooth_force_validate_launch_error)?;
        Ok(Self {
            buffer,
            values,
            dims,
            revision,
            checked,
        })
    }

    fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn proof(
        &self,
        dims: SmoothForceValidateDims,
    ) -> Result<&Checked<SmoothForceValidateMetadata>, TransferError> {
        if self.dims != dims {
            return Err(InputError::InvalidDimension {
                field: "smooth_force_validate_metadata_dims",
            }
            .into());
        }
        self.checked
            .binding(0)
            .validate_for(self.buffer.launcher_span()?, &dims)
            .map_err(smooth_force_validate_launch_error)?;
        Ok(&self.checked)
    }

    fn write_range(&mut self, start: usize, values: &[i32]) -> Result<(), TransferError> {
        let end = start
            .checked_add(values.len())
            .filter(|&end| end <= self.values.len())
            .ok_or(InputError::InvalidDimension {
                field: "smooth_force_metadata_write",
            })?;
        self.revision.invalidate();
        self.buffer.write_range(start, values)?;
        self.values[start..end].copy_from_slice(values);
        self.refresh_proof()
    }

    fn refresh_proof(&mut self) -> Result<(), TransferError> {
        // SAFETY: 私有包装同步维护设备值、宿主镜像和写入代际。
        self.checked = unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(smooth_force_validate_launch_error)?;
        Ok(())
    }

    #[cfg(test)]
    fn corrupt_device_for_validator_test(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        // 测试故意保留有效证明，让设备验证器读取坏值。
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    fn proof_snapshot_for_test(
        &self,
    ) -> Result<Checked<SmoothForceValidateMetadata>, TransferError> {
        // SAFETY: 测试快照绑定当前设备值、镜像和代际。
        unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(smooth_force_validate_launch_error)
    }

    #[cfg(test)]
    fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        self.refresh_proof()
    }
}

#[cfg(feature = "cuda-probe")]
struct SmoothForceValidateKernel {
    kernel: SynchronousKernel,
    sleep_enabled: bool,
}

#[cfg(feature = "cuda-probe")]
impl SmoothForceValidateKernel {
    fn compile(
        session: &TransferSession,
        source: &str,
        sleep_enabled: bool,
    ) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "smooth_force_validate")?,
            sleep_enabled,
        })
    }

    fn launch(
        &self,
        metadata: &SmoothForceMetadataBuffer,
        forces: &TransferBuffer<f32>,
        sleep: &TransferBuffer<i32>,
        status: &mut TransferBuffer<f32>,
        abi: [u32; 4],
    ) -> Result<(), TransferError> {
        let dims = SmoothForceValidateDims {
            abi,
            sleep_enabled: self.sleep_enabled,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: forces.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: sleep.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        smooth_force_validate_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(smooth_force_validate_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名和网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), forces, sleep, status, abi)
        }
    }
}

pub struct SmoothForcePlan {
    input: Arc<SmoothForceInput>,
    options: SmoothForceOptions,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    validate: SmoothForceValidateKernel,
    #[cfg(feature = "cuda-probe")]
    compute: SynchronousKernel,
}
pub struct SmoothForceData {
    input: Arc<SmoothForceInput>,
    layout: Layout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    metadata: SmoothForceMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    forces: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SmoothForceOutput {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl SmoothForceOutput {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
impl SmoothForcePlan {
    pub fn new(
        session: &TransferSession,
        input: SmoothForceInput,
        options: SmoothForceOptions,
    ) -> Result<Self, TransferError> {
        layout(1, input.nbody(), input.nv(), input.ntree())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, options);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = format!(
                "#define SMOOTH_SLEEP {}\n{}",
                i32::from(options.enable_sleep()),
                CUDA
            );
            Ok(Self {
                input: Arc::new(input),
                options,
                session: session.clone(),
                validate: SmoothForceValidateKernel::compile(
                    session,
                    &source,
                    options.enable_sleep(),
                )?,
                compute: SynchronousKernel::compile(session, &source, "smooth_force")?,
            })
        }
    }
    pub fn input(&self) -> &SmoothForceInput {
        &self.input
    }
    pub fn options(&self) -> SmoothForceOptions {
        self.options
    }
    pub fn create_data(&self, worlds: usize) -> Result<SmoothForceData, TransferError> {
        let l = layout(
            worlds,
            self.input.nbody(),
            self.input.nv(),
            self.input.ntree(),
        )?;
        // 整数守卫包围静态映射与逐世界树状态。
        let mut meta = crate::runtime::host_staging::<i32>(l.meta_len)?;
        meta.fill(-131072);
        let body_end = 4 + self.input.nbody();
        let awake_start = body_end + self.input.nv();
        meta[4..body_end].copy_from_slice(self.input.body_treeid());
        meta[body_end..awake_start].copy_from_slice(self.input.dof_bodyid());
        meta[awake_start..l.meta_len - 4].fill(1);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = meta;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            Ok(SmoothForceData {
                input: self.input.clone(),
                layout: l,
                ready: false,
                metadata: SmoothForceMetadataBuffer::upload(
                    &self.session,
                    meta,
                    SmoothForceValidateDims {
                        abi: [
                            self.input.nbody() as u32,
                            self.input.nv() as u32,
                            self.input.ntree() as u32,
                            worlds as u32,
                        ],
                        sleep_enabled: self.options.enable_sleep(),
                    },
                )?,
                forces: allocate(l.force_len, 0.0)?,
                output: allocate(l.vector.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut SmoothForceData) -> Result<(), TransferError> {
        #[cfg(feature = "cuda-probe")]
        {
            self.update_with_sleep_source(data, None)
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            if !Arc::ptr_eq(&self.input, &data.input) {
                return Err(TransferError::ModelMismatch);
            }
            data.ready = false;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn update_with_sleep_source(
        &self,
        data: &mut SmoothForceData,
        sleep: Option<&TransferBuffer<i32>>,
    ) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        let nb = self.input.nbody() as u32;
        let n = self.input.nv() as u32;
        let nt = self.input.ntree() as u32;
        let worlds = data.worlds() as u32;
        if self.options.enable_sleep()
            && let Some(source) = sleep
        {
            let expected = data
                .worlds()
                .checked_mul(
                    self.input
                        .ntree()
                        .checked_mul(2)
                        .and_then(|n| n.checked_add(3))
                        .ok_or(InputError::Overflow {
                            field: "smooth_force_sleep_source",
                        })?,
                )
                .and_then(|n| n.checked_add(8))
                .ok_or(InputError::Overflow {
                    field: "smooth_force_sleep_source",
                })?;
            if source.len() != expected {
                return Err(InputError::LengthMismatch {
                    field: "smooth_force_sleep_source",
                    expected,
                    actual: source.len(),
                }
                .into());
            }
        }
        let sleep = sleep.unwrap_or(data.metadata.as_buffer());
        self.validate.launch(
            &data.metadata,
            &data.forces,
            sleep,
            &mut data.status,
            [nb, n, nt, worlds],
        )?;
        let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
        data.status.read_range_into(0, &mut status)?;
        crate::physics::check_device_values(&status, "smooth_force_status")?;
        if let Some(index) = status[4..4 + data.worlds()].iter().position(|&v| v != 1.0) {
            return Err(InputError::InvalidTopology {
                field: "smooth_force_source",
                index,
                reason: "nonfinite_guard_or_mapping",
            }
            .into());
        }
        // SAFETY: Validation proves all mappings and active force payloads.
        // Field-major input owns4*worlds*n f32s; one thread owns each result.
        // Sleeping threads write positive zero without reading any forces.
        unsafe {
            self.compute.launch(
                data.metadata.as_buffer(),
                &data.forces,
                sleep,
                &mut data.output,
                [nb, n, nt, data.layout.vector.total_elements() as u32],
            )?;
        }
        data.checked_values()?;
        data.ready = true;
        Ok(())
    }
}
fn check_force(values: &[f32], expected: usize, field: SmoothForceField) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field: field.name(),
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite {
            field: field.name(),
            index,
        });
    }
    Ok(())
}
impl SmoothForceData {
    pub fn worlds(&self) -> usize {
        self.layout.vector.worlds()
    }
    pub(super) fn invalidate(&mut self) {
        self.ready = false;
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn passive_target(&mut self) -> &mut TransferBuffer<f32> {
        &mut self.forces
    }
    pub fn write_force(
        &mut self,
        field: SmoothForceField,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.write(field, 0, self.layout.vector.total_elements(), values)
    }
    pub fn write_world_force(
        &mut self,
        field: SmoothForceField,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.layout.vector.world_elements(world)?;
        self.write(field, r.start, r.len(), values)
    }
    fn write(
        &mut self,
        field: SmoothForceField,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check_force(values, expected, field)?;
        let start = 4 + field.index() * self.layout.vector.total_elements() + offset;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = start;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.forces.write_range(start, values)
        }
    }
    pub fn write_awake(&mut self, values: &[i32]) -> Result<(), TransferError> {
        self.awake(0, self.layout.awake.total_elements(), values)
    }
    pub fn write_world_awake(&mut self, world: usize, values: &[i32]) -> Result<(), TransferError> {
        let r = self.layout.awake.world_elements(world)?;
        self.awake(r.start, r.len(), values)
    }
    fn awake(
        &mut self,
        offset: usize,
        expected: usize,
        values: &[i32],
    ) -> Result<(), TransferError> {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "tree_awake",
                expected,
                actual: values.len(),
            }
            .into());
        }
        let start = 4 + self.input.nbody() + self.input.nv() + offset;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = start;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.metadata.write_range(start, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut v = crate::runtime::host_staging::<f32>(self.layout.vector.total_elements() + 8)?;
        self.output.read_range_into(0, &mut v)?;
        crate::physics::check_device_values(&v, "qfrc_smooth")?;
        Ok(v)
    }
    pub fn readback(&self) -> Result<SmoothForceOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "qfrc_smooth",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(SmoothForceOutput {
                layout: self.layout.vector,
                values: self.checked_values()?,
            })
        }
    }
}
// Frozen forward.py _qfrc_smooth (Apache-2.0): three separate f32 rounds.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void smooth_force_validate(const int* meta,const float* forces,
 const int* sleep,float* status,unsigned nb,unsigned n,unsigned nt,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long total=1ull*worlds*n,mlen=1ull*nb+n+1ull*worlds*nt;bool valid=true;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[4+mlen+j]==-131072&&forces[j]==-131072.0f&&forces[4+4*total+j]==-131072.0f;
 const int* tree=meta+4;const int* body=tree+nb;const int* awake=body+n;unsigned long long stride=nt;
 #if SMOOTH_SLEEP
 if(sleep!=meta){unsigned long long sw=2ull*nt+3,sl=1ull*worlds*sw;for(unsigned j=0;j<4;++j)valid=valid&&sleep[j]==-131072&&sleep[4+sl+j]==-131072;awake=sleep+4+nt;stride=sw;}
 #endif
 for(unsigned b=0;b<nb;++b)if(tree[b]<-1||(tree[b]>=0&&(unsigned)tree[b]>=nt))valid=false;
 for(unsigned i=0;i<n;++i){int b=body[i];if(b<0||(unsigned)b>=nb){valid=false;continue;}
  int t=tree[b];if(t<-1||(t>=0&&(unsigned)t>=nt)){valid=false;continue;}
  #if SMOOTH_SLEEP
  if(t>=0&&awake[1ull*w*stride+t]==0)continue;
  #endif
  for(unsigned f=0;f<4;++f)valid=valid&&isfinite(forces[4+f*total+1ull*w*n+i]);
 }
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void smooth_force(const int* meta,const float* forces,
 const int* sleep,float* output,unsigned nb,unsigned n,unsigned nt,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/n,i=tid%n;const int* tree=meta+4;const int* body=tree+nb;const int* awake=body+n;
 #if SMOOTH_SLEEP
 unsigned long long stride=nt;if(sleep!=meta){awake=sleep+4+nt;stride=2ull*nt+3;}
 int t=tree[body[i]];if(t>=0&&awake[1ull*w*stride+t]==0){output[4+tid]=0.0f;return;}
 #endif
 const float* f=forces+4+tid;float acc=__fsub_rn(f[0],f[work]);
 acc=__fadd_rn(acc,f[2ull*work]);output[4+tid]=__fadd_rn(acc,f[3ull*work]);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn reference() -> serde_json::Value {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/smooth-force/reference.json"
        )))
        .unwrap()
    }
    fn row(v: &serde_json::Value, index: usize) -> ([f32; 4], u32) {
        let r = &v["rows"][index];
        let mut x = [0.0; 4];
        for (j, value) in x.iter_mut().enumerate() {
            *value = f32::from_bits(
                u32::from_str_radix(r["input_bits"][j].as_str().unwrap(), 16).unwrap(),
            );
        }
        let bits = u32::from_str_radix(r["output_bits"].as_str().unwrap(), 16).unwrap();
        (x, bits)
    }
    #[test]
    fn fixed_scalar_certificate_distinguishes_step_rounding_and_signed_zeros() {
        let v = reference();
        assert_eq!(v["rows"].as_array().unwrap().len(), 18);
        for i in 0..18 {
            let (x, bits) = row(&v, i);
            let first = x[0] - x[1];
            let second = first + x[2];
            let third = second + x[3];
            assert_eq!(third.to_bits(), bits, "case{i}");
        }
        let (x, bits) = row(&v, 2);
        assert_eq!(
            f64::from(x[0]) - f64::from(x[1]) + f64::from(x[2]) + f64::from(x[3]),
            0.0
        );
        assert_eq!(bits, 1.0f32.to_bits());
        assert_eq!(row(&v, 7).1, (-0.0f32).to_bits());
    }
    #[test]
    fn bounds_layout_and_force_lengths_without_gpu_allocation() {
        for (w, nb, n, nt) in [
            (0, 1, 1, 0),
            (1, 0, 1, 0),
            (1, 1, 0, 0),
            (usize::MAX, 1, 1, 1),
            (1, 1, 1, usize::MAX),
            (1, usize::MAX, 1, 0),
        ] {
            assert!(layout(w, nb, n, nt).is_err());
        }
        let l = layout(257, 273, 513, 129).unwrap();
        assert_eq!(l.force_len, 4 * 257 * 513);
        assert_eq!(l.meta_len, 273 + 513 + 257 * 129 + 8);
        assert!(layout(2, 2, 2, 0).is_ok());
        for f in [
            SmoothForceField::Passive,
            SmoothForceField::Bias,
            SmoothForceField::Actuator,
            SmoothForceField::Applied,
        ] {
            assert!(check_force(&[], 1, f).is_err());
            assert!(check_force(&[f32::NAN], 1, f).is_err());
        }
        for (sleep, island, enabled) in [
            (false, false, false),
            (false, true, false),
            (true, false, true),
            (true, true, false),
        ] {
            assert_eq!(
                SmoothForceOptions {
                    sleep_enabled: sleep,
                    island_disabled: island
                }
                .enable_sleep(),
                enabled
            );
        }
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    fn validate_wrapper_rejects_capacity_context_alias_span_dimension_and_proof_faults() {
        let dims = SmoothForceValidateDims {
            abi: [2, 3, 2, 2],
            sleep_enabled: true,
        };
        let lengths = smooth_force_validate_lengths(&dims).unwrap();
        assert_eq!(lengths, [17, 32, 22, 10]);
        for abi in [[0, 3, 2, 2], [2, 0, 2, 2], [2, 3, 2, 0]] {
            assert!(matches!(
                smooth_force_validate_lengths(&SmoothForceValidateDims {
                    abi,
                    sleep_enabled: true,
                }),
                Err(LauncherError::InvalidArray("平滑力验证维度无效"))
            ));
        }
        let max_grid_worlds = u32::MAX - 255;
        assert!(
            smooth_force_validate_lengths(&SmoothForceValidateDims {
                abi: [1, 1, 0, max_grid_worlds],
                sleep_enabled: false,
            })
            .is_ok()
        );
        assert!(matches!(
            smooth_force_validate_lengths(&SmoothForceValidateDims {
                abi: [1, 1, 0, max_grid_worlds + 1],
                sleep_enabled: false,
            }),
            Err(LauncherError::InvalidArray("平滑力验证维度无效"))
        ));
        let metadata_limit = SmoothForceValidateDims {
            abi: [1, 1, i32::MAX as u32 - 10, 1],
            sleep_enabled: true,
        };
        assert_eq!(
            smooth_force_validate_lengths(&metadata_limit).unwrap()[0],
            i32::MAX as u64
        );
        let metadata_over = SmoothForceValidateDims {
            abi: [1, 1, i32::MAX as u32 - 9, 1],
            sleep_enabled: true,
        };
        assert!(matches!(
            smooth_force_validate_lengths(&metadata_over),
            Err(LauncherError::Overflow("smooth_force_validate_metadata"))
        ));
        let forces_over = SmoothForceValidateDims {
            abi: [1, i32::MAX as u32 - 9, 0, max_grid_worlds],
            sleep_enabled: false,
        };
        assert!(matches!(
            smooth_force_validate_lengths(&forces_over),
            Err(LauncherError::Overflow("smooth_force_validate_forces"))
        ));
        // status由u32加8构成，静态上界不会触及u64溢出。
        assert!(u64::from(u32::MAX).checked_add(8).is_some());
        // 合法meta约束worlds*nt不超过i32上界。
        // 因此外部sleep的静态上界也远低于u64上界。
        let sleep_static_upper = 2u64 * i32::MAX as u64 + 3u64 * u64::from(max_grid_worlds) + 8;
        assert!(sleep_static_upper < u64::MAX);
        let spans = [
            DeviceSpan::new(7, 0x1000, lengths[0] * 4).unwrap(),
            DeviceSpan::new(7, 0x2000, lengths[1] * 4).unwrap(),
            DeviceSpan::new(7, 0x3000, lengths[2] * 4).unwrap(),
            DeviceSpan::new(7, 0x4000, lengths[3] * 4).unwrap(),
        ];
        let bindings = [
            SlotBinding {
                slot: 0,
                span: spans[0],
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: spans[1],
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: spans[2],
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 3,
                span: spans[3],
                elem: ElemType::F32,
            },
        ];
        let values = vec![
            -131072,
            -131072,
            -131072,
            -131072,
            -1,
            1,
            0,
            1,
            1,
            0,
            -11,
            2,
            i32::MAX,
            -131072,
            -131072,
            -131072,
            -131072,
        ];
        let revision = MetadataRevision::new();
        // SAFETY: 合成跨度与宿主快照长度一致；本测试不访问设备。
        let checked = unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                spans[0],
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        assert!(smooth_force_validate_plan(&dims, &bindings, &[checked.binding(0)]).is_ok());
        // SAFETY: 短快照与短跨度一致，只验证宿主内容。
        assert!(matches!(
            unsafe {
                Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                    DeviceSpan::new(7, 0x5000, (lengths[0] - 1) * 4).unwrap(),
                    &values[..values.len() - 1],
                    dims,
                    MetadataRevision::new(),
                )
            },
            Err(LauncherError::InvalidMetadata(_))
        ));

        let mut wrong_metadata_span = bindings;
        wrong_metadata_span[0].span =
            DeviceSpan::new(7, spans[0].addr(), lengths[0] * 4 + 4).unwrap();
        assert!(matches!(
            smooth_force_validate_plan(&dims, &wrong_metadata_span, &[checked.binding(0)]),
            Err(LauncherError::MetadataMismatch(_))
        ));
        for (slot, bytes) in [
            (1, lengths[1] * 4 - 4),
            (3, lengths[3] * 4 - 4),
            (3, lengths[3] * 4 + 4),
        ] {
            let mut bad = bindings;
            bad[slot].span = DeviceSpan::new(7, bad[slot].span.addr(), bytes).unwrap();
            assert!(matches!(
                smooth_force_validate_plan(&dims, &bad, &[checked.binding(0)]),
                Err(LauncherError::Capacity { slot: actual, .. }) if actual == slot
            ));
        }
        let mut bad_sleep = bindings;
        for bytes in [lengths[2] * 4 - 4, lengths[2] * 4 + 4] {
            bad_sleep[2].span = DeviceSpan::new(7, 0x3000, bytes).unwrap();
            assert!(matches!(
                smooth_force_validate_plan(&dims, &bad_sleep, &[checked.binding(0)]),
                Err(LauncherError::Capacity { slot: 2, .. })
            ));
        }
        let mut wrong_context = bindings;
        wrong_context[2].span = DeviceSpan::new(8, 0x3000, lengths[2] * 4).unwrap();
        assert!(matches!(
            smooth_force_validate_plan(&dims, &wrong_context, &[checked.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));
        let mut partial_alias = bindings;
        partial_alias[2].span = DeviceSpan::new(7, spans[0].addr() + 4, lengths[2] * 4).unwrap();
        assert!(matches!(
            smooth_force_validate_plan(&dims, &partial_alias, &[checked.binding(0)]),
            Err(LauncherError::Alias {
                first: 0,
                second: 2
            })
        ));
        let mut read_alias = bindings;
        read_alias[2].span = DeviceSpan::new(7, spans[1].addr(), lengths[2] * 4).unwrap();
        assert!(matches!(
            smooth_force_validate_plan(&dims, &read_alias, &[checked.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 2
            })
        ));
        let mut status_alias = bindings;
        status_alias[3].span = spans[1];
        assert!(matches!(
            smooth_force_validate_plan(&dims, &status_alias, &[checked.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3
            })
        ));
        let mut exact_alias = bindings;
        exact_alias[2].span = spans[0];
        assert!(smooth_force_validate_plan(&dims, &exact_alias, &[checked.binding(0)]).is_ok());

        let disabled = SmoothForceValidateDims {
            sleep_enabled: false,
            ..dims
        };
        // SAFETY: 合成跨度与同一有效宿主快照绑定。
        let disabled_checked = unsafe {
            Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                spans[0],
                &values,
                disabled,
                MetadataRevision::new(),
            )
        }
        .unwrap();
        let mut ignored_sleep = bindings;
        ignored_sleep[2].span = DeviceSpan::new(7, 0x3000, 4).unwrap();
        assert!(
            smooth_force_validate_plan(&disabled, &ignored_sleep, &[disabled_checked.binding(0)])
                .is_ok()
        );
        assert!(matches!(
            DeviceSpan::new(7, 0x3000, 0),
            Err(LauncherError::InvalidSpan("区间长度为零"))
        ));

        revision.invalidate();
        assert!(matches!(
            smooth_force_validate_plan(&dims, &bindings, &[checked.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));
        let mut invalid_values = values.clone();
        invalid_values[4] = -2;
        // SAFETY: 合成跨度只用于宿主校验，不触发设备访问。
        assert!(matches!(
            unsafe {
                Checked::<SmoothForceValidateMetadata>::assume_uploaded(
                    spans[0],
                    &invalid_values,
                    dims,
                    MetadataRevision::new(),
                )
            },
            Err(LauncherError::InvalidMetadata(_))
        ));
    }
    #[cfg(feature = "cuda-probe")]
    const FIELDS: [SmoothForceField; 4] = [
        SmoothForceField::Passive,
        SmoothForceField::Bias,
        SmoothForceField::Actuator,
        SmoothForceField::Applied,
    ];
    #[cfg(feature = "cuda-probe")]
    fn model(n: usize) -> SmoothForceInput {
        SmoothForceInput::new(
            129,
            (0..273)
                .map(|b| if b % 11 == 0 { -1 } else { b * 7 % 129 })
                .collect(),
            (0..n).map(|i| ((i * 17 + 1) % 273) as i32).collect(),
        )
        .unwrap()
    }
    #[cfg(feature = "cuda-probe")]
    fn raw_f32(buffer: &TransferBuffer<f32>) -> Vec<f32> {
        let mut v = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut v).unwrap();
        v
    }
    #[cfg(feature = "cuda-probe")]
    fn raw_i32(buffer: &TransferBuffer<i32>) -> Vec<i32> {
        let mut v = vec![0; buffer.len()];
        buffer.read_range_into(0, &mut v).unwrap();
        v
    }
    #[cfg(feature = "cuda-probe")]
    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|v| v.to_bits()).collect()
    }
    #[cfg(feature = "cuda-probe")]
    fn tokens(d: &SmoothForceData) -> (Vec<u32>, Vec<i32>, Vec<u32>) {
        (
            bits(&raw_f32(&d.forces)),
            raw_i32(d.metadata.as_buffer()),
            bits(&raw_f32(&d.output)),
        )
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "M3仅验证类型包装：关闭休眠、真实拒绝和同计划恢复"]
    fn gpu_m3_validate_wrapper_sleep_disabled_preserves_output_and_recovers() {
        let session = TransferSession::new(0).unwrap();
        let input = SmoothForceInput::new(2, vec![-1, 0, 1], vec![1, 2, 0]).unwrap();
        let plan = SmoothForcePlan::new(&session, input, SmoothForceOptions::default()).unwrap();
        let mut data = plan.create_data(2).unwrap();
        for (field, value) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
            data.write_force(*field, &[value; 6]).unwrap();
        }
        data.write_awake(&[0, 0, 0, 0]).unwrap();
        let ignored = session.upload(&[i32::MIN]).unwrap();
        plan.update_with_sleep_source(&mut data, Some(&ignored))
            .unwrap();
        let output = data.readback().unwrap();
        for world in 0..2 {
            assert!(
                output
                    .world(world)
                    .unwrap()
                    .iter()
                    .all(|value| value.to_bits() == 10.0f32.to_bits())
            );
        }
        let good = bits(&output.values);
        let output_before = bits(&raw_f32(&data.output));

        data.forces.write_range(4, &[f32::NAN]).unwrap();
        let error = plan
            .update_with_sleep_source(&mut data, Some(&ignored))
            .unwrap_err();
        assert_eq!(
            error,
            TransferError::Input(InputError::InvalidTopology {
                field: "smooth_force_source",
                index: 0,
                reason: "nonfinite_guard_or_mapping",
            })
        );
        assert_eq!(raw_f32(&data.status)[4..6], [0.0, 1.0]);
        assert!(data.readback().is_err());
        assert_eq!(bits(&raw_f32(&data.output)), output_before);
        data.forces.write_range(4, &[5.0]).unwrap();
        plan.update_with_sleep_source(&mut data, Some(&ignored))
            .unwrap();
        assert_eq!(bits(&data.readback().unwrap().values), good);

        let report = serde_json::json!({
            "variant": "sleep-off",
            "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
            "plans_compiled": 1,
            "validation_rejection_entered_gpu": true,
            "output_bits_unchanged": true,
            "same_plan_recovery": true,
            "ignored_sleep_elements": ignored.len(),
            "task_gpu_memory": "unknown"
        });
        if let Some(path) = std::env::var_os("MJWARP_SMOOTH_VALIDATE_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!("{report}");
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "M3仅验证类型包装：休眠来源、非零awake、ntree零和恢复"]
    fn gpu_m3_validate_wrapper_sleep_enabled_covers_sources_and_ntree_zero() {
        let session = TransferSession::new(0).unwrap();
        let input = SmoothForceInput::new(2, vec![-1, 0, 1], vec![1, 2, 0]).unwrap();
        let plan = SmoothForcePlan::new(
            &session,
            input,
            SmoothForceOptions {
                sleep_enabled: true,
                island_disabled: false,
            },
        )
        .unwrap();
        let mut data = plan.create_data(2).unwrap();
        for (field, value) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
            data.write_force(*field, &[value; 6]).unwrap();
        }
        let stale = data.metadata.proof_snapshot_for_test().unwrap();
        data.write_awake(&[-11, 0, 2, i32::MAX]).unwrap();
        let metadata_dims = data.metadata.dims;
        let metadata_span = data.metadata.as_buffer().launcher_span().unwrap();
        assert!(matches!(
            stale.binding(0).validate_for(metadata_span, &metadata_dims),
            Err(LauncherError::StaleMetadata)
        ));
        assert!(data.metadata.proof(metadata_dims).is_ok());
        plan.update(&mut data).unwrap();
        let internal = data.readback().unwrap();
        assert_eq!(internal.world(0).unwrap()[0].to_bits(), 10.0f32.to_bits());
        assert_eq!(internal.world(0).unwrap()[1].to_bits(), 0);
        assert_eq!(internal.world(0).unwrap()[2].to_bits(), 10.0f32.to_bits());
        assert!(internal.world(1).unwrap().iter().all(|&v| v == 10.0));
        let internal_bits = bits(&internal.values);

        let stride = 2 * 2 + 3;
        let mut external = vec![-131072; 2 * stride + 8];
        for world in 0..2 {
            let awake = 4 + world * stride + 2;
            external[awake..awake + 2].copy_from_slice(if world == 0 {
                &[0, -11]
            } else {
                &[2, i32::MAX]
            });
        }
        let mut external = session.upload(&external).unwrap();
        plan.update_with_sleep_source(&mut data, Some(&external))
            .unwrap();
        let external_output = data.readback().unwrap();
        assert_eq!(
            bits(external_output.world(0).unwrap()),
            [0.0f32, 10.0, 10.0].map(f32::to_bits)
        );
        assert_eq!(
            bits(external_output.world(1).unwrap()),
            [10.0f32; 3].map(f32::to_bits)
        );
        let external_good = bits(&external_output.values);
        let output_before = bits(&raw_f32(&data.output));
        external.write_range(0, &[0]).unwrap();
        let error = plan
            .update_with_sleep_source(&mut data, Some(&external))
            .unwrap_err();
        assert_eq!(
            error,
            TransferError::Input(InputError::InvalidTopology {
                field: "smooth_force_source",
                index: 0,
                reason: "nonfinite_guard_or_mapping",
            })
        );
        assert_eq!(raw_f32(&data.status)[4..6], [0.0, 0.0]);
        assert!(data.readback().is_err());
        assert_eq!(bits(&raw_f32(&data.output)), output_before);
        external.write_range(0, &[-131072]).unwrap();
        plan.update_with_sleep_source(&mut data, Some(&external))
            .unwrap();
        assert_eq!(bits(&data.readback().unwrap().values), external_good);

        let output_before_metadata_fault = bits(&raw_f32(&data.output));
        data.metadata
            .corrupt_device_for_validator_test(0, 0)
            .unwrap();
        let error = plan.update(&mut data).unwrap_err();
        assert_eq!(
            error,
            TransferError::Input(InputError::InvalidTopology {
                field: "smooth_force_source",
                index: 0,
                reason: "nonfinite_guard_or_mapping",
            })
        );
        assert_eq!(raw_f32(&data.status)[4..6], [0.0, 0.0]);
        assert!(data.readback().is_err());
        assert_eq!(bits(&raw_f32(&data.output)), output_before_metadata_fault);
        data.metadata.restore_for_test().unwrap();
        plan.update(&mut data).unwrap();
        assert_eq!(bits(&data.readback().unwrap().values), internal_bits);

        let dims = SmoothForceValidateDims {
            abi: [2, 2, 0, 2],
            sleep_enabled: true,
        };
        let metadata = vec![
            -131072, -131072, -131072, -131072, -1, -1, 0, 1, -131072, -131072, -131072, -131072,
        ];
        let metadata = SmoothForceMetadataBuffer::upload(&session, metadata, dims).unwrap();
        let mut forces = vec![-131072.0; 4 * 2 * 2 + 8];
        forces[4..4 + 16].fill(1.0);
        let forces = session.upload(&forces).unwrap();
        let mut status_values = vec![-131072.0; 2 + 8];
        status_values[4..6].fill(f32::NAN);
        let mut status = session.upload(&status_values).unwrap();
        plan.validate
            .launch(
                &metadata,
                &forces,
                metadata.as_buffer(),
                &mut status,
                dims.abi,
            )
            .unwrap();
        let status = raw_f32(&status);
        assert_eq!(status[4..6], [1.0, 1.0]);

        let report = serde_json::json!({
            "variant": "sleep-on",
            "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
            "plans_compiled": 1,
            "internal_awake": true,
            "external_sleep": true,
            "nonzero_awake": [-11, 2, i32::MAX],
            "ntree_zero_reused_validation_kernel": true,
            "validation_rejection_entered_gpu": true,
            "output_bits_unchanged": true,
            "same_plan_recovery": true,
            "task_gpu_memory": "unknown"
        });
        if let Some(path) = std::env::var_os("MJWARP_SMOOTH_VALIDATE_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!("{report}");
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new U016 four-force merge, independent scalar bits, sleep/island and owned snapshots"]
    fn gpu_merges_exact_f32_steps_and_zeroes_sleeping_dofs_in_all_option_combinations() {
        let v = reference();
        let session = TransferSession::new(0).unwrap();
        let start = std::time::Instant::now();
        let mut batches = 0usize;
        let mut slots = 0usize;
        let mut asleep = 0usize;
        let mut static_active = 0usize;
        let mut negative_zero = 0usize;
        let mut capacity = 0usize;
        let mut seen = [0usize; 18];
        for n in [1, 65, 513] {
            let input = model(n);
            for (sleep, island) in [(false, false), (false, true), (true, false), (true, true)] {
                let options = SmoothForceOptions {
                    sleep_enabled: sleep,
                    island_disabled: island,
                };
                let plan = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
                assert_eq!(plan.options(), options);
                assert_eq!(plan.input(), &input);
                for sample in 0..2 {
                    for worlds in [1, 2, 257] {
                        let mut d = plan.create_data(worlds).unwrap();
                        assert!(d.readback().is_err());
                        let mut values: [Vec<f32>; 4] =
                            std::array::from_fn(|_| Vec::with_capacity(worlds * n));
                        for w in 0..worlds {
                            for i in 0..n {
                                let (x, _) = row(&v, (i + 3 * w + 5 * sample) % 18);
                                for f in 0..4 {
                                    values[f].push(x[f]);
                                }
                            }
                        }
                        let awake = (0..worlds * 129)
                            .map(|k| match (k % 129 + k / 129 + sample) % 5 {
                                0 => 0,
                                1 => -11,
                                2 => 2,
                                3 => 1,
                                _ => i32::MAX,
                            })
                            .collect::<Vec<_>>();
                        for (f, values) in FIELDS.iter().zip(&values) {
                            d.write_force(*f, values).unwrap();
                        }
                        d.write_awake(&awake).unwrap();
                        plan.update(&mut d).unwrap();
                        let out = d.readback().unwrap();
                        for w in 0..worlds {
                            for (i, &actual) in out.world(w).unwrap().iter().enumerate() {
                                let tree = input.body_treeid()[input.dof_bodyid()[i] as usize];
                                let index = (i + 3 * w + 5 * sample) % 18;
                                let sleeping = options.enable_sleep()
                                    && tree >= 0
                                    && awake[w * 129 + tree as usize] == 0;
                                let expected = if sleeping {
                                    asleep += 1;
                                    0
                                } else {
                                    seen[index] += 1;
                                    if tree < 0 {
                                        static_active += 1;
                                    }
                                    row(&v, index).1
                                };
                                assert_eq!(
                                    actual.to_bits(),
                                    expected,
                                    "n{n}/w{w}/i{i}/sample{sample}/{options:?}"
                                );
                                if !sleeping && expected == 0x80000000 {
                                    negative_zero += 1;
                                }
                                slots += 1;
                            }
                        }
                        assert!(out.world(worlds).is_err());
                        capacity = capacity.max(
                            4 * (d.forces.len()
                                + d.output.len()
                                + d.status.len()
                                + d.metadata.len()),
                        );
                        batches += 1;
                    }
                }
            }
        }
        assert_eq!((batches, slots), (72, 1204320));
        assert!(asleep > 0 && static_active > 0 && negative_zero > 0);
        assert!(seen.iter().all(|&c| c > 0));
        // New field/awake lifecycle only; no qpos, mass or RHS producer exists.
        let input = model(65);
        let options = SmoothForceOptions {
            sleep_enabled: true,
            island_disabled: false,
        };
        let plan = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
        let other = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
        let mut d = plan.create_data(257).unwrap();
        for (f, x) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
            d.write_force(*f, &vec![x; 257 * 65]).unwrap();
        }
        plan.update(&mut d).unwrap();
        let unchanged = tokens(&d);
        assert!(other.update(&mut d).is_err());
        assert_eq!(tokens(&d), unchanged);
        assert!(d.readback().is_ok());
        for f in FIELDS {
            for (w, length) in [(257, 65), (0, 64)] {
                assert!(d.write_world_force(f, w, &vec![1.0; length]).is_err());
                assert_eq!(tokens(&d), unchanged);
                assert!(d.readback().is_ok());
            }
            let mut bad = vec![1.0; 65];
            bad[3] = f32::NAN;
            assert!(d.write_world_force(f, 0, &bad).is_err());
            assert!(d.write_force(f, &[]).is_err());
            assert_eq!(tokens(&d), unchanged);
            assert!(d.readback().is_ok());
        }
        assert!(d.write_world_awake(257, &[1; 129]).is_err());
        assert!(d.write_world_awake(0, &[1; 128]).is_err());
        assert!(d.write_awake(&[]).is_err());
        assert_eq!(tokens(&d), unchanged);
        assert!(d.readback().is_ok());
        for (j, f) in FIELDS.iter().enumerate() {
            let before = raw_f32(&d.forces);
            let values = vec![[7.0, 1.0, 6.0, 8.0][j]; 65];
            let a = 4 + j * 257 * 65 + 256 * 65;
            d.write_world_force(*f, 256, &values).unwrap();
            let after = raw_f32(&d.forces);
            for i in (0..a).chain(a + 65..before.len()) {
                assert_eq!(before[i].to_bits(), after[i].to_bits());
            }
            assert_eq!(bits(&after[a..a + 65]), bits(&values));
            assert!(d.readback().is_err());
            plan.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..257 {
                let expected = if w == 256 {
                    [12.0f32, 11.0, 13.0, 14.0][j]
                } else {
                    10.0
                };
                assert!(
                    out.world(w)
                        .unwrap()
                        .iter()
                        .all(|v| v.to_bits() == expected.to_bits())
                );
            }
            d.write_world_force(*f, 256, &vec![[5.0, 2.0, 3.0, 4.0][j]; 65])
                .unwrap();
            plan.update(&mut d).unwrap();
        }
        let before = raw_i32(d.metadata.as_buffer());
        let a = 4 + 273 + 65 + 256 * 129;
        d.write_world_awake(256, &[0; 129]).unwrap();
        let after = raw_i32(d.metadata.as_buffer());
        for i in (0..a).chain(a + 129..before.len()) {
            assert_eq!(before[i], after[i]);
        }
        assert!(after[a..a + 129].iter().all(|&v| v == 0));
        assert!(d.readback().is_err());
        plan.update(&mut d).unwrap();
        let sleeping = d.readback().unwrap();
        for w in 0..257 {
            for (i, &value) in sleeping.world(w).unwrap().iter().enumerate() {
                let tree = input.body_treeid()[input.dof_bodyid()[i] as usize];
                let expected = if w == 256 && tree >= 0 { 0.0f32 } else { 10.0 };
                assert_eq!(value.to_bits(), expected.to_bits());
            }
        }
        d.write_world_awake(256, &[-11; 129]).unwrap();
        plan.update(&mut d).unwrap();
        let snapshot = d.readback().unwrap();
        drop(d);
        drop(plan);
        drop(other);
        drop(session);
        assert!(
            snapshot
                .world(256)
                .unwrap()
                .iter()
                .all(|v| v.to_bits() == 10.0f32.to_bits())
        );
        // ntree0 has no awake payload; static DOFs still preserve active -0.
        let session = TransferSession::new(0).unwrap();
        let plan = SmoothForcePlan::new(
            &session,
            SmoothForceInput::new(0, vec![-1, -1], vec![1, 0]).unwrap(),
            options,
        )
        .unwrap();
        let mut d = plan.create_data(2).unwrap();
        let (x, _) = row(&v, 7);
        for (f, x) in FIELDS.iter().zip(x) {
            d.write_force(*f, &[x; 4]).unwrap();
        }
        d.write_world_awake(1, &[]).unwrap();
        plan.update(&mut d).unwrap();
        assert!(
            d.readback()
                .unwrap()
                .world(1)
                .unwrap()
                .iter()
                .all(|v| v.to_bits() == 0x80000000)
        );
        let report = serde_json::json!({"batches":batches,"slots":slots,"bit_differences":0,"asleep_positive_zero_slots":asleep,"static_active_slots":static_active,"active_negative_zero_slots":negative_zero,"active_case_coverage":seen,"phase_ms":start.elapsed().as_millis(),"owned_Data_max_bytes":capacity,"full_fwd_acceleration_NP":false});
        std::fs::write(
            std::env::var_os("MJWARP_SMOOTH_FORCE_REPORT").unwrap(),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new smooth-force finite/overflow, mapping and full guarded-buffer rejection"]
    fn gpu_rejects_guard_mapping_and_active_overflow_but_overwrites_sleeping_output_with_zero() {
        let session = TransferSession::new(0).unwrap();
        let input = SmoothForceInput::new(2, vec![-1, 0, 1], vec![1, 2, 0]).unwrap();
        let options = SmoothForceOptions {
            sleep_enabled: true,
            island_disabled: false,
        };
        let plan = SmoothForcePlan::new(&session, input, options).unwrap();
        let mut d = plan.create_data(2).unwrap();
        for (f, x) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
            d.write_force(*f, &[x; 6]).unwrap();
        }
        plan.update(&mut d).unwrap();
        let good = d.readback().unwrap();
        for (kind, offset, bad) in [
            (0, 0, 0.0),
            (0, 28, 0.0),
            (0, 4, f32::NAN),
            (0, 4 + 18 + 2, f32::INFINITY),
            (1, 0, 0.0),
            (1, 10, 0.0),
            (2, 0, 0.0),
            (2, 6, 0.0),
        ] {
            let buffer = match kind {
                0 => &mut d.forces,
                1 => &mut d.output,
                _ => &mut d.status,
            };
            let mut old = [0.0];
            buffer.read_range_into(offset, &mut old).unwrap();
            buffer.write_range(offset, &[bad]).unwrap();
            assert!(plan.update(&mut d).is_err());
            assert!(d.readback().is_err());
            let buffer = match kind {
                0 => &mut d.forces,
                1 => &mut d.output,
                _ => &mut d.status,
            };
            buffer.write_range(offset, &old).unwrap();
            plan.update(&mut d).unwrap();
            assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
        }
        for (offset, bad) in [(0, 0), (14, 0), (5, -2), (5, 2), (7, -1), (7, 3)] {
            let mut old = [0];
            d.metadata
                .as_buffer()
                .read_range_into(offset, &mut old)
                .unwrap();
            d.metadata
                .corrupt_device_for_validator_test(offset, bad)
                .unwrap();
            assert!(plan.update(&mut d).is_err());
            assert!(d.readback().is_err());
            d.metadata.restore_for_test().unwrap();
            plan.update(&mut d).unwrap();
            assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
        }
        d.write_world_awake(0, &[0, 1]).unwrap();
        d.forces.write_range(4, &[f32::NAN]).unwrap();
        plan.update(&mut d).unwrap();
        let asleep = d.readback().unwrap();
        assert_eq!(asleep.world(0).unwrap()[0].to_bits(), 0);
        assert_eq!(asleep.world(0).unwrap()[1], 10.0);
        assert_eq!(asleep.world(0).unwrap()[2], 10.0);
        d.write_world_awake(0, &[2, -11]).unwrap();
        assert!(plan.update(&mut d).is_err());
        assert!(d.readback().is_err());
        d.forces.write_range(4, &[5.0]).unwrap();
        plan.update(&mut d).unwrap();
        d.write_world_force(SmoothForceField::Passive, 0, &[f32::MAX, 5.0, 5.0])
            .unwrap();
        d.write_world_force(SmoothForceField::Bias, 0, &[-f32::MAX, 2.0, 2.0])
            .unwrap();
        assert!(plan.update(&mut d).is_err());
        assert!(d.readback().is_err());
        d.write_world_awake(0, &[0, 1]).unwrap();
        plan.update(&mut d).unwrap();
        assert_eq!(d.readback().unwrap().world(0).unwrap()[0].to_bits(), 0);
        d.write_world_force(SmoothForceField::Passive, 0, &[5.0; 3])
            .unwrap();
        d.write_world_force(SmoothForceField::Bias, 0, &[2.0; 3])
            .unwrap();
        d.write_world_awake(0, &[1, 1]).unwrap();
        plan.update(&mut d).unwrap();
        assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
    }
}
