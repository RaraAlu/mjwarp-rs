//! 常驻RNE偏置力子集。
//! 完整G02与qacc路径仍待实现。

use super::{ComVelocityData, ComVelocityPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout};
use crate::runtime::TransferSession;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 一次上传与编译的偏置力计划。
/// 重力与模型参数共享一份。
/// 显式零重力关闭重力贡献。
/// 本子集固定采用flg_acc=false。
pub struct RneBiasPlan {
    velocity: ComVelocityPlan,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: RneBiasMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    kernel: RneBiasKernel,
}

/// 独占位置、速度与偏置力设备状态。
/// 合法写入废弃已有偏置力。
pub struct RneBiasData {
    velocity: ComVelocityData,
    layout: RneBiasLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

/// 显式回读后的独立宿主快照。
#[derive(Clone, Debug)]
pub struct RneBiasSnapshot {
    layout: RneBiasLayout,
    values: Vec<f32>,
}

/// 单世界广义偏置力只读视图。
#[derive(Clone, Copy, Debug)]
pub struct RneBiasWorld<'a> {
    pub qfrc_bias: &'a [f32],
}

#[derive(Clone, Copy, Debug)]
struct RneBiasLayout {
    output: BatchLayout,
    #[cfg(feature = "cuda-probe")]
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}

impl RneBiasLayout {
    fn new(worlds: usize, nbody: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "rne_bias_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "rne_bias_layout",
        };
        let width = nbody
            .checked_mul(12)
            .and_then(|n| n.checked_add(nv))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let metadata = nbody
            .checked_mul(3)
            .and_then(|n| n.checked_add(nv))
            .and_then(|n| n.checked_add(5))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, metadata, 4)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = guarded;
        Ok(Self {
            output,
            #[cfg(feature = "cuda-probe")]
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

#[cfg(any(feature = "cuda-probe", test))]
const RNE_BIAS_METADATA_KIND: ProofKind = ProofKind("rne.bias.metadata.v1");

#[cfg(any(feature = "cuda-probe", test))]
const RNE_BIAS_SLOTS: [SlotDesc; 6] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RNE_BIAS_METADATA_KIND,
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
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
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

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RneBiasDims {
    worlds: u32,
    nbody: u32,
    nv: u32,
    metadata_elements: u64,
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_bias_metadata_elements(nbody: u32, nv: u32) -> Result<u64, LauncherError> {
    if nbody == 0 || nbody > i32::MAX as u32 || nv > i32::MAX as u32 {
        return Err(LauncherError::Overflow("rne_bias_signed_dimensions"));
    }
    u64::from(nbody)
        .checked_mul(3)
        .and_then(|value| value.checked_add(u64::from(nv)))
        .and_then(|value| value.checked_add(5))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("rne_bias_metadata_elements"))
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_bias_lengths(dims: &RneBiasDims) -> Result<[u64; 5], LauncherError> {
    if dims.worlds == 0 || dims.worlds > u32::MAX - 255 || dims.nbody == 0 {
        return Err(LauncherError::InvalidArray("RNE偏置力维度无效"));
    }
    let expected = rne_bias_metadata_elements(dims.nbody, dims.nv)?;
    if dims.metadata_elements != expected {
        return Err(LauncherError::InvalidMetadata("RNE偏置力元数据长度不匹配"));
    }
    let worlds = u64::from(dims.worlds);
    let nbody = u64::from(dims.nbody);
    let nv = u64::from(dims.nv);
    let guarded = |width: u64, field| {
        worlds
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_bias_com_width"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_bias_velocity_width"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_bias_output_width"))?;
    let qvel = guarded(nv, "rne_bias_qvel")?;
    let com = guarded(com_width, "rne_bias_com")?;
    let velocity = guarded(velocity_width, "rne_bias_velocity")?;
    let output = guarded(output_width, "rne_bias_output")?;
    if output_width > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rne_bias_output_width"));
    }
    Ok([dims.metadata_elements, qvel, com, velocity, output])
}

#[cfg(any(feature = "cuda-probe", test))]
fn validate_rne_bias_metadata(values: &[i32], dims: RneBiasDims) -> Result<(), LauncherError> {
    let expected = rne_bias_metadata_elements(dims.nbody, dims.nv)?;
    if dims.metadata_elements != expected
        || u64::try_from(values.len()).ok() != Some(dims.metadata_elements)
    {
        return Err(LauncherError::InvalidMetadata("RNE偏置力元数据长度无效"));
    }
    let nbody = usize::try_from(dims.nbody)
        .map_err(|_| LauncherError::Overflow("rne_bias_metadata_nbody"))?;
    let nv =
        usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("rne_bias_metadata_nv"))?;
    if values[0] != dims.nbody as i32 || values[1] != dims.nv as i32 {
        return Err(LauncherError::InvalidMetadata("RNE偏置力元数据维度不匹配"));
    }
    if values[2..5]
        .iter()
        .any(|&bits| !f32::from_bits(bits as u32).is_finite())
    {
        return Err(LauncherError::InvalidMetadata("RNE偏置力重力不是有限值"));
    }

    let parent_start = 5;
    let count_start = parent_start + nbody;
    let address_start = count_start + nbody;
    let body_start = address_start + nbody;
    let parent = &values[parent_start..count_start];
    let count = &values[count_start..address_start];
    let address = &values[address_start..body_start];
    let body = &values[body_start..];
    if parent[0] != 0 || count[0] != 0 || address[0] != 0 {
        return Err(LauncherError::InvalidMetadata("RNE偏置力根节点元数据无效"));
    }
    let mut covered = vec![false; nv];
    for index in 1..nbody {
        let parent_index = usize::try_from(parent[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE偏置力父节点为负"))?;
        if parent_index >= index {
            return Err(LauncherError::InvalidMetadata("RNE偏置力父节点顺序无效"));
        }
        let dof_count = usize::try_from(count[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE偏置力自由度数量为负"))?;
        let dof_start = usize::try_from(address[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE偏置力自由度地址为负"))?;
        let dof_end = dof_start
            .checked_add(dof_count)
            .filter(|&end| end <= nv)
            .ok_or(LauncherError::InvalidMetadata("RNE偏置力自由度范围越界"))?;
        for dof in dof_start..dof_end {
            if body[dof] != index as i32 || covered[dof] {
                return Err(LauncherError::InvalidMetadata("RNE偏置力自由度归属无效"));
            }
            covered[dof] = true;
        }
    }
    if covered.contains(&false) {
        return Err(LauncherError::InvalidMetadata("RNE偏置力自由度映射不完整"));
    }
    for &body_index in body {
        let body_index = usize::try_from(body_index)
            .map_err(|_| LauncherError::InvalidMetadata("RNE偏置力体索引为负"))?;
        if body_index >= nbody {
            return Err(LauncherError::InvalidMetadata("RNE偏置力体索引越界"));
        }
    }
    rne_bias_lengths(&dims).map(|_| ())
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneBiasSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_bias私有ABI。
unsafe impl KernelAbi for RneBiasSpec {
    const NAME: &'static str = "rne_bias";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_BIAS_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneBiasSpec {
    type Dims = RneBiasDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_bias_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_bias_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneBiasMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneBiasMetadata {
    type Dims = RneBiasDims;

    const KIND: ProofKind = RNE_BIAS_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_bias_metadata(values, *dims)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_bias_plan(
    dims: &RneBiasDims,
    bindings: &[SlotBinding; 5],
    proofs: &[CheckedBinding<'_, RneBiasDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_bias_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_bias_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneBiasSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
fn rne_bias_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_bias_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_bias_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_bias_abi",
        }
        .into(),
    }
}

fn check_gravity(gravity: &[f32; 3]) -> Result<(), InputError> {
    if let Some(index) = gravity.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite {
            field: "rne_bias_gravity",
            index,
        });
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
struct RneBiasMetadataBuffer {
    session: TransferSession,
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    dims: RneBiasDims,
    revision: MetadataRevision,
    checked: Checked<RneBiasMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl RneBiasMetadataBuffer {
    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        nbody: usize,
        nv: usize,
    ) -> Result<Self, TransferError> {
        let dims = RneBiasDims {
            worlds: 1,
            nbody: u32::try_from(nbody).map_err(|_| InputError::Overflow {
                field: "rne_bias_metadata_nbody",
            })?,
            nv: u32::try_from(nv).map_err(|_| InputError::Overflow {
                field: "rne_bias_metadata_nv",
            })?,
            metadata_elements: u64::try_from(values.len()).map_err(|_| InputError::Overflow {
                field: "rne_bias_metadata",
            })?,
        };
        validate_rne_bias_metadata(&values, dims).map_err(rne_bias_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<RneBiasMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(rne_bias_launch_error)?;
        Ok(Self {
            session: session.clone(),
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

    fn proof(&self, worlds: u32) -> Result<Checked<RneBiasMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(rne_bias_launch_error)?;
        let dims = RneBiasDims {
            worlds,
            ..self.dims
        };
        // SAFETY: 基础证明已核对真实缓冲、镜像和写入代际。
        unsafe {
            Checked::<RneBiasMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_bias_launch_error)
    }

    #[cfg(test)]
    fn overwrite_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入已同步；跨度与镜像重新绑定。
        self.checked = unsafe {
            Checked::<RneBiasMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_bias_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
struct RneBiasLaunch {
    proof: Checked<RneBiasMetadata>,
}

#[cfg(feature = "cuda-probe")]
struct RneBiasKernel {
    session: TransferSession,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneBiasKernel {
    fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_BIAS_CUDA, "rne_bias")?,
        })
    }

    fn bindings(
        metadata: &RneBiasMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 5], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
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
                span: velocity.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    #[allow(clippy::too_many_arguments)]
    fn preflight(
        &self,
        session: &TransferSession,
        metadata: &RneBiasMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<RneBiasLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.proof(worlds)?;
        let bindings = Self::bindings(metadata, qvel, com, velocity, output)?;
        rne_bias_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_bias_launch_error)?;
        Ok(RneBiasLaunch { proof })
    }

    fn launch(
        &self,
        launch: &RneBiasLaunch,
        metadata: &RneBiasMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, qvel, com, velocity, output)?;
        rne_bias_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_bias_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel.launch_flex_edges(
                metadata.as_buffer(),
                qvel,
                com,
                velocity,
                output,
                dims.worlds,
            )
        }
    }
}

impl RneBiasPlan {
    /// 调用方显式提供共享重力。
    /// 本入口不转换完整原生Option。
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        gravity: [f32; 3],
    ) -> Result<Self, TransferError> {
        check_gravity(&gravity)?;
        let k = model.rigid().kinematics();
        let (nbody, nv) = (k.nbody(), k.nv());
        RneBiasLayout::new(1, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let values = {
            let f = k.fields();
            let mut m = crate::runtime::host_staging::<i32>(5 + 3 * nbody + nv)?;
            m[0] = nbody as i32;
            m[1] = nv as i32;
            for i in 0..3 {
                m[2 + i] = gravity[i].to_bits() as i32;
            }
            m[5..5 + nbody].copy_from_slice(&f.body_parentid);
            for b in 1..nbody {
                let start = f.body_jntadr[b] as usize;
                let count = f.body_jntnum[b] as usize;
                if count != 0 {
                    m[5 + 2 * nbody + b] = f.jnt_dofadr[start];
                    m[5 + nbody + b] = f.jnt_type[start..start + count]
                        .iter()
                        .map(|&t| match t {
                            0 => 6,
                            1 => 3,
                            _ => 1,
                        })
                        .sum();
                }
            }
            m[5 + 3 * nbody..].copy_from_slice(&model.rigid().fields().dof_bodyid);
            m
        };
        let velocity = ComVelocityPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let metadata = RneBiasMetadataBuffer::upload(session, values, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let kernel = RneBiasKernel::compile(session)?;
        Ok(Self {
            velocity,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            kernel,
        })
    }

    pub fn device(&self) -> usize {
        self.velocity.device()
    }

    pub fn create_data(&self, worlds: usize) -> Result<RneBiasData, TransferError> {
        let layout = RneBiasLayout::new(worlds, self.nbody, self.nv)?;
        let velocity = self.velocity.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, velocity);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
            values.fill(-131072.0);
            values[4..layout.guarded - 4].fill(f32::NAN);
            Ok(RneBiasData {
                velocity,
                layout,
                ready: false,
                output: self.session.upload(&values)?,
            })
        }
    }

    /// GPU更新位置、质心、速度与偏置力。
    /// 阶段间不回读宿主中间量。
    pub fn update(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.update_velocity(data)?;
        self.update_bias(data)
    }

    /// 显式准备本子集前置阶段。
    /// 本调用废弃已有偏置力。
    pub fn update_velocity(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        self.velocity.update(&mut data.velocity)
    }

    /// 只读取已有空间速度与质心。
    /// 不隐式重算任何前置阶段。
    /// 本严格子集要求新鲜度。
    /// 它不替代完整等价rne入口。
    pub fn update_bias(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let inputs = data.velocity.rne_inputs()?;
            let launch = self.kernel.preflight(
                &self.session,
                &self.metadata,
                inputs.qvel,
                inputs.com,
                inputs.velocity,
                &data.output,
                data.layout.output.worlds() as u32,
            )?;
            self.kernel.launch(
                &launch,
                &self.metadata,
                inputs.qvel,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )?;
            data.ready = true;
            Ok(())
        }
    }
}

impl RneBiasData {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }

    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }

    pub fn write_qpos(&mut self, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qpos(qpos);
        self.after_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qpos(world, qpos);
        self.after_write(result)
    }
    pub fn write_qvel(&mut self, qvel: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qvel(qvel);
        self.after_write(result)
    }
    pub fn write_world_qvel(&mut self, world: usize, qvel: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qvel(world, qvel);
        self.after_write(result)
    }

    /// 显式检查前置字段与全部守卫。
    /// 回读包含内部工作区检查。
    pub fn readback(&self) -> Result<RneBiasSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady { stage: "rne_bias" });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.velocity.readback()?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            super::check_device_values(&values, "rne_bias_output")?;
            let mut bias = crate::runtime::host_staging::<f32>(self.worlds() * self.layout.nv)?;
            for w in 0..self.worlds() {
                let start =
                    4 + self.layout.output.world_elements(w)?.start + 12 * self.layout.nbody;
                for d in 0..self.layout.nv {
                    bias[w * self.layout.nv + d] = values[start + d];
                }
            }
            if let Some(index) = bias.iter().position(|v| !v.is_finite()) {
                return Err(InputError::NonFinite {
                    field: "rne_bias_output",
                    index,
                }
                .into());
            }
            Ok(RneBiasSnapshot {
                layout: self.layout,
                values: bias,
            })
        }
    }
}

impl RneBiasSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<RneBiasWorld<'_>, InputError> {
        self.layout.output.world_elements(world)?;
        let start = world * self.nv();
        Ok(RneBiasWorld {
            qfrc_bias: &self.values[start..start + self.nv()],
        })
    }
}

// Adapted from frozen smooth.py RNE and math.py spatial inertia/force algebra.
// Copyright 2025 The Newton Developers.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[cfg(feature = "cuda-probe")]
const RNE_BIAS_CUDA: &str = r#"
__device__ void inertia_vector(const float* i,const float* v,float* f) {
  f[0]=i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5];
  f[1]=i[3]*v[0]+i[1]*v[1]+i[5]*v[2]+i[8]*v[3]-i[6]*v[5];
  f[2]=i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4];
  f[3]=i[8]*v[1]-i[7]*v[2]+i[9]*v[3];
  f[4]=i[6]*v[2]-i[8]*v[0]+i[9]*v[4];
  f[5]=i[7]*v[0]-i[6]*v[1]+i[9]*v[5];
}
__device__ void cross_force(const float* v,const float* f,float* out) {
  out[0]=(v[1]*f[2]-v[2]*f[1])+(v[4]*f[5]-v[5]*f[4]);
  out[1]=(v[2]*f[0]-v[0]*f[2])+(v[5]*f[3]-v[3]*f[5]);
  out[2]=(v[0]*f[1]-v[1]*f[0])+(v[3]*f[4]-v[4]*f[3]);
  out[3]=v[1]*f[5]-v[2]*f[4];out[4]=v[2]*f[3]-v[0]*f[5];out[5]=v[0]*f[4]-v[1]*f[3];
}
extern "C" __global__ void rne_bias(const int* m,const float* velocities,
    const float* com,const float* velocity,float* output,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  unsigned nb=m[0],nv=m[1];const int* parent=m+5;
  const int* num=parent+nb;const int* adr=num+nb;const int* body=adr+nb;
  const float* qvel=velocities+4+(unsigned long long)w*nv;
  const float* ci=com+4+(unsigned long long)w*(14ull*nb+6ull*nv)+4ull*nb;
  const float* cdof=ci+10ull*nb;
  const float* cvel=velocity+4+(unsigned long long)w*(6ull*nb+6ull*nv);
  const float* dot=cvel+6ull*nb;
  float* acc=output+4+(unsigned long long)w*(12ull*nb+nv);
  float* frc=acc+6ull*nb;float* bias=frc+6ull*nb;
  for(int k=0;k<6;++k) { acc[k]=k<3?0.0f:-__int_as_float(m[k-1]);frc[k]=0.0f; }
  for(unsigned b=1;b<nb;++b) {
    float* a=acc+6ull*b;
    for(int k=0;k<6;++k) a[k]=acc[6ull*parent[b]+k];
    for(int d=adr[b];d<adr[b]+num[b];++d)
      for(int k=0;k<6;++k) a[k]+=dot[6ull*d+k]*qvel[d];
    float iv[6],cross[6],v[6];for(int k=0;k<6;++k) v[k]=cvel[6ull*b+k];
    inertia_vector(ci+10ull*b,v,iv);cross_force(v,iv,cross);
    inertia_vector(ci+10ull*b,a,frc+6ull*b);
    for(int k=0;k<6;++k) frc[6ull*b+k]+=cross[k];
  }
  for(int b=int(nb)-1;b>0;--b)
    for(int k=0;k<6;++k) frc[6ull*parent[b]+k]+=frc[6ull*b+k];
  for(unsigned d=0;d<nv;++d) {
    float f=0.0f;for(int k=0;k<6;++k) f+=cdof[6ull*d+k]*frc[6ull*body[d]+k];bias[d]=f;
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "cuda-probe")]
    fn reference_model(v: &serde_json::Value) -> AttachedModelInput {
        use crate::model::{
            InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
        };

        let model = &v["model"];
        let floats = |key: &str| {
            model[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_f64().unwrap() as f32)
                .collect()
        };
        let integers = |key: &str| {
            model[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| i32::try_from(value.as_i64().unwrap()).unwrap())
                .collect()
        };
        let kinematics = KinematicModelInput::new(
            v["nv"].as_u64().unwrap() as usize,
            KinematicFields {
                qpos0: floats("qpos0"),
                body_parentid: integers("body_parentid"),
                body_jntadr: integers("body_jntadr"),
                body_jntnum: integers("body_jntnum"),
                body_pos: floats("body_pos"),
                body_quat: floats("body_quat"),
                jnt_type: integers("jnt_type"),
                jnt_bodyid: integers("jnt_bodyid"),
                jnt_qposadr: integers("jnt_qposadr"),
                jnt_dofadr: integers("jnt_dofadr"),
                jnt_pos: floats("jnt_pos"),
                jnt_axis: floats("jnt_axis"),
            },
        )
        .unwrap();
        let rigid = InertialModelInput::new(
            kinematics,
            InertialFields {
                body_ipos: floats("body_ipos"),
                body_iquat: floats("body_iquat"),
                body_mass: floats("body_mass"),
                body_inertia: floats("body_inertia"),
                dof_bodyid: integers("dof_bodyid"),
                dof_jntid: integers("dof_jntid"),
                dof_parentid: integers("dof_parentid"),
                dof_armature: floats("dof_armature"),
                dof_damping: floats("dof_damping"),
            },
        )
        .unwrap();
        AttachedModelInput::new(rigid, crate::model::AttachedFields::default()).unwrap()
    }

    #[test]
    fn rejects_bias_layout_dimensions_and_overflow() {
        for (w, b, v) in [
            (0, 1, 0),
            (1, 0, 0),
            (u32::MAX as usize, 1, 0),
            (1, usize::MAX, 0),
            (1, 1, usize::MAX),
        ] {
            assert!(RneBiasLayout::new(w, b, v).is_err());
        }
        let layout = RneBiasLayout::new(513, 1, 0).unwrap();
        assert_eq!(layout.output.total_bytes(), 513 * 12 * size_of::<f32>());
    }

    #[test]
    fn rejects_nonfinite_gravity_and_checks_empty_bias_views() {
        for v in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for i in 0..3 {
                let mut g = [0.0; 3];
                g[i] = v;
                assert!(check_gravity(&g).is_err());
            }
        }
        assert!(check_gravity(&[1.0, -2.0, 3.0]).is_ok());
        let out = RneBiasSnapshot {
            layout: RneBiasLayout::new(2, 1, 0).unwrap(),
            values: vec![],
        };
        assert_eq!(out.worlds(), 2);
        assert!(out.world(1).unwrap().qfrc_bias.is_empty());
        assert!(out.world(2).is_err());
    }

    #[test]
    fn rne_bias_plan_checks_metadata_spans_aliases_and_overflow() {
        use crate::runtime::launcher::DeviceSpan;

        let bind = |slot, context, addr, bytes, elem| SlotBinding {
            slot,
            span: DeviceSpan::new(context, addr, bytes).unwrap(),
            elem,
        };
        let dims = RneBiasDims {
            worlds: 2,
            nbody: 2,
            nv: 1,
            metadata_elements: 12,
        };
        let values = vec![2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1];
        assert_eq!(rne_bias_lengths(&dims).unwrap(), [12, 10, 76, 44, 58]);
        let revision = MetadataRevision::new();
        let metadata_span = DeviceSpan::new(11, 0x1000, 48).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof = unsafe {
            Checked::<RneBiasMetadata>::assume_uploaded(
                metadata_span,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            bind(0, 11, 0x1000, 48, ElemType::I32),
            bind(1, 11, 0x2000, 40, ElemType::F32),
            bind(2, 11, 0x3000, 304, ElemType::F32),
            bind(3, 11, 0x4000, 176, ElemType::F32),
            bind(4, 11, 0x5000, 232, ElemType::F32),
        ];
        let launch = rne_bias_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
        assert_eq!(launch.grid(), (1, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));
        assert_eq!(launch.shared_bytes(), 0);
        assert!(matches!(
            rne_bias_plan(&dims, &bindings, &[]),
            Err(LauncherError::MissingMetadataProof { slot: 0 })
        ));

        let addresses = [0x1000, 0x2000, 0x3000, 0x4000, 0x5000];
        let bytes = [48, 40, 304, 176, 232];
        for slot in 0..5 {
            let mut short = bindings;
            short[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] - 4).unwrap();
            assert!(matches!(
                rne_bias_plan(&dims, &short, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));

            let mut long = bindings;
            long[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] + 4).unwrap();
            assert!(matches!(
                rne_bias_plan(&dims, &long, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));
        }

        let mut foreign = bindings;
        foreign[3].span = DeviceSpan::new(12, 0x4000, 176).unwrap();
        assert!(matches!(
            rne_bias_plan(&dims, &foreign, &[proof.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 3 })
        ));
        let mut alias = bindings;
        alias[4].span = DeviceSpan::new(11, 0x2000, 232).unwrap();
        assert!(matches!(
            rne_bias_plan(&dims, &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 4
            })
        ));
        let mut read_alias = bindings;
        read_alias[2].span = DeviceSpan::new(11, 0x2000, 304).unwrap();
        rne_bias_plan(&dims, &read_alias, &[proof.binding(0)]).unwrap();

        revision.invalidate();
        assert!(matches!(
            rne_bias_plan(&dims, &bindings, &[proof.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));
        for invalid in [
            RneBiasDims { worlds: 0, ..dims },
            RneBiasDims {
                worlds: u32::MAX - 254,
                ..dims
            },
            RneBiasDims {
                metadata_elements: 11,
                ..dims
            },
        ] {
            assert!(rne_bias_lengths(&invalid).is_err());
        }
        assert!(rne_bias_metadata_elements(i32::MAX as u32 + 1, 0).is_err());
        assert!(rne_bias_metadata_elements(715_827_881, 0).is_err());
        let layout_wide = RneBiasDims {
            worlds: 1,
            nbody: i32::MAX as u32 / 12 + 1,
            nv: 0,
            metadata_elements: rne_bias_metadata_elements(i32::MAX as u32 / 12 + 1, 0).unwrap(),
        };
        assert!(matches!(
            rne_bias_lengths(&layout_wide),
            Err(LauncherError::Overflow("rne_bias_output_width"))
        ));
        let wide = RneBiasDims {
            worlds: u32::MAX - 255,
            nbody: 400_000_000,
            nv: 0,
            metadata_elements: 1_200_000_005,
        };
        assert!(matches!(
            rne_bias_lengths(&wide),
            Err(LauncherError::Overflow("rne_bias_com"))
        ));
        let byte_overflow = RneBiasDims {
            worlds: u32::MAX - 255,
            nbody: 100_000_000,
            nv: 0,
            metadata_elements: 300_000_005,
        };
        let byte_overflow_bindings = [
            bind(0, 11, 0x1000, 1_200_000_020, ElemType::I32),
            bind(1, 11, 0x1_0000_0000, 32, ElemType::F32),
            bind(2, 11, 0x2_0000_0000, 4, ElemType::F32),
            bind(3, 11, 0x3_0000_0000, 4, ElemType::F32),
            bind(4, 11, 0x4_0000_0000, 4, ElemType::F32),
        ];
        assert!(matches!(
            rne_bias_plan(&byte_overflow, &byte_overflow_bindings, &[]),
            Err(LauncherError::Overflow("rne_bias_bytes"))
        ));
        let max_width = RneBiasDims {
            worlds: 1,
            nbody: 1,
            nv: i32::MAX as u32 - 12,
            metadata_elements: i32::MAX as u64 - 4,
        };
        assert!(rne_bias_lengths(&max_width).is_ok());
        let too_wide = RneBiasDims {
            nv: i32::MAX as u32 - 11,
            metadata_elements: i32::MAX as u64 - 3,
            ..max_width
        };
        assert!(matches!(
            rne_bias_lengths(&too_wide),
            Err(LauncherError::Overflow("rne_bias_output_width"))
        ));

        for (index, value) in [
            (2, f32::NAN.to_bits() as i32),
            (5, 1),
            (6, 1),
            (7, 1),
            (8, 2),
            (10, 1),
            (11, 0),
            (11, 2),
        ] {
            let mut invalid = values.clone();
            invalid[index] = value;
            assert!(validate_rne_bias_metadata(&invalid, dims).is_err());
        }
        let mut incomplete = values;
        incomplete[8] = 0;
        assert!(validate_rne_bias_metadata(&incomplete, dims).is_err());

        let tail = RneBiasDims {
            worlds: 513,
            ..dims
        };
        assert_eq!(RneBiasSpec::launch_dims(&tail).unwrap().grid(), (3, 1, 1));
    }

    #[test]
    fn rne_bias_plan_accepts_zero_dof_boundary() {
        use crate::runtime::launcher::DeviceSpan;

        let dims = RneBiasDims {
            worlds: 2,
            nbody: 2,
            nv: 0,
            metadata_elements: 11,
        };
        let values = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let revision = MetadataRevision::new();
        let span = DeviceSpan::new(13, 0x1000, 44).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof =
            unsafe { Checked::<RneBiasMetadata>::assume_uploaded(span, &values, dims, revision) }
                .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(13, 0x2000, 32).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(13, 0x3000, 256).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(13, 0x4000, 128).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: DeviceSpan::new(13, 0x5000, 224).unwrap(),
                elem: ElemType::F32,
            },
        ];
        assert_eq!(rne_bias_lengths(&dims).unwrap(), [11, 8, 64, 32, 56]);
        rne_bias_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn runs_rne_bias_wrapper_and_recovers_after_real_metadata_write() {
        let session = TransferSession::new(0).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/rne-bias/mixed-joints.json")).unwrap(),
        )
        .unwrap();
        let model = reference_model(&fixture);
        assert_eq!(
            model.rigid().kinematics().fields().body_parentid,
            [0, 0, 1, 2, 3, 0]
        );
        let mut plan = RneBiasPlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
        assert_eq!((plan.nbody, plan.nv), (6, 11));
        const WORLDS: usize = 513;
        let mut data = plan.create_data(WORLDS).unwrap();
        let qvel: Vec<_> = (0..WORLDS * plan.nv)
            .map(|index| (index as f32 - 2821.0) * 0.000_125)
            .collect();
        data.write_qvel(&qvel).unwrap();
        plan.update_velocity(&mut data).unwrap();
        let width = 12 * plan.nbody + plan.nv;
        let sentinel = -777.25f32;
        let mut initial = vec![sentinel; data.output.len()];
        initial[..4].fill(-131072.0);
        let end = initial.len() - 4;
        initial[end..].fill(-131072.0);

        let (raw, typed) = {
            let inputs = data.velocity.rne_inputs().unwrap();
            data.output.write_range(0, &initial).unwrap();
            // SAFETY: 复用产品计划的同一实际缓冲和冻结私有ABI。
            unsafe {
                plan.kernel.kernel.launch_flex_edges(
                    plan.metadata.as_buffer(),
                    inputs.qvel,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                    WORLDS as u32,
                )
            }
            .unwrap();
            let mut raw = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut raw).unwrap();

            data.output.write_range(0, &initial).unwrap();
            let launch = plan
                .kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    inputs.qvel,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    WORLDS as u32,
                )
                .unwrap();
            plan.kernel
                .launch(
                    &launch,
                    &plan.metadata,
                    inputs.qvel,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )
                .unwrap();
            let mut typed = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut typed).unwrap();
            (raw, typed)
        };
        assert_eq!(
            raw.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            typed
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(&raw[..4], &initial[..4]);
        assert_eq!(&raw[raw.len() - 4..], &initial[initial.len() - 4..]);
        for world in 0..WORLDS {
            let start = 4 + world * width;
            assert!(
                raw[start..start + width]
                    .iter()
                    .all(|value| value.to_bits() != sentinel.to_bits())
            );
        }

        {
            let inputs = data.velocity.rne_inputs().unwrap();
            let short_qvel = session.upload(&vec![0.0; inputs.qvel.len() - 1]).unwrap();
            assert!(
                plan.kernel
                    .preflight(
                        &plan.session,
                        &plan.metadata,
                        &short_qvel,
                        inputs.com,
                        inputs.velocity,
                        &data.output,
                        WORLDS as u32,
                    )
                    .is_err()
            );
            let proof = plan.metadata.proof(WORLDS as u32).unwrap();
            let mut alias = RneBiasKernel::bindings(
                &plan.metadata,
                inputs.qvel,
                inputs.com,
                inputs.velocity,
                &data.output,
            )
            .unwrap();
            let output_bytes = rne_bias_lengths(proof.dims()).unwrap()[4] * 4;
            alias[4].span = alias[2].span.view(0, output_bytes).unwrap();
            assert!(matches!(
                rne_bias_plan(proof.dims(), &alias, &[proof.binding(0)]),
                Err(LauncherError::Alias {
                    first: 2,
                    second: 4
                })
            ));
        }

        let sentinel_workspace = |data: &mut RneBiasData, seed: f32| {
            let mut values = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut values).unwrap();
            let end = values.len() - 4;
            for (index, value) in values[4..end].iter_mut().enumerate() {
                *value = seed + (index % 31) as f32 * 0.125;
            }
            data.output.write_range(0, &values).unwrap();
            values
        };
        plan.update_bias(&mut data).unwrap();
        data.readback().unwrap();
        let before = sentinel_workspace(&mut data, -4096.0);
        plan.metadata.overwrite_for_test(0, 1).unwrap();
        assert!(plan.update_bias(&mut data).is_err());
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne_bias" })
        ));
        let mut rejected = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut rejected).unwrap();
        assert_eq!(
            before
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            rejected
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.metadata.restore_for_test().unwrap();
        plan.update_bias(&mut data).unwrap();
        data.readback().unwrap();

        let before_session = sentinel_workspace(&mut data, -8192.0);
        let foreign_session = TransferSession::new(0).unwrap();
        let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
        if let Some(root) = capture.as_ref() {
            let foreign = std::path::PathBuf::from(root).join("foreign-session");
            assert!(!foreign.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
        }
        let foreign_kernel = RneBiasKernel::compile(&foreign_session).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        } else {
            // SAFETY: 本测试单线程运行，调用前也没有该变量。
            unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
        }
        let correct_kernel = std::mem::replace(&mut plan.kernel, foreign_kernel);
        assert!(matches!(
            plan.update_bias(&mut data),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne_bias" })
        ));
        let mut rejected_session = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut rejected_session)
            .unwrap();
        assert_eq!(
            before_session
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            rejected_session
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.kernel = correct_kernel;
        plan.update_bias(&mut data).unwrap();
        data.readback().unwrap();

        let zero = super::super::tests::test_model(None);
        let zero = AttachedModelInput::new(zero, crate::model::AttachedFields::default()).unwrap();
        if let Some(root) = capture.as_ref() {
            let zero_capture = std::path::PathBuf::from(root).join("zero-dof");
            assert!(!zero_capture.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &zero_capture) };
        }
        let zero_plan = RneBiasPlan::new(&session, zero, [0.0; 3]).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        }
        let mut zero_data = zero_plan.create_data(2).unwrap();
        zero_plan.update(&mut zero_data).unwrap();
        let zero_output = zero_data.readback().unwrap();
        for world in 0..2 {
            assert!(zero_output.world(world).unwrap().qfrc_bias.is_empty());
        }
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn bias_readback_rejects_guards_nonfinite_workspaces_and_results() {
        let m = super::super::tests::test_model(Some(3));
        let m = AttachedModelInput::new(m, crate::model::AttachedFields::default()).unwrap();
        let s = TransferSession::new(0).unwrap();
        let p = RneBiasPlan::new(&s, m, [0.0; 3]).unwrap();
        let mut d = p.create_data(1).unwrap();
        p.update(&mut d).unwrap();
        d.readback().unwrap();
        for i in [0, d.layout.guarded - 1] {
            d.output.write_range(i, &[0.0]).unwrap();
            assert!(d.readback().is_err());
            d.output.write_range(i, &[-131072.0]).unwrap();
        }
        for i in [4, 4 + 6 * d.layout.nbody, 4 + 12 * d.layout.nbody] {
            d.output.write_range(i, &[f32::NAN]).unwrap();
            assert!(d.readback().is_err());
            p.update_bias(&mut d).unwrap();
            d.readback().unwrap();
        }
        d.output
            .write_range(4 + 12 * d.layout.nbody, &[f32::INFINITY])
            .unwrap();
        assert!(d.readback().is_err());
        p.update_bias(&mut d).unwrap();
        d.readback().unwrap();
    }
}
