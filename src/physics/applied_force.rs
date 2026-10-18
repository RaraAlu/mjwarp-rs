//! 世界空间外力经雅可比映射到广义力。
//! 调用方提供几何字段；本入口不执行位置生产者。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc, SlotKind, SlotRequirement,
    SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Topology {
    parent: Vec<i32>,
    // 默认配置无设备路径；仅cuda-probe读取。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    root: Vec<i32>,
    body: Vec<i32>,
}
fn topology(parent: Vec<i32>, root: Vec<i32>, body: Vec<i32>) -> Result<Topology, InputError> {
    let nb = parent.len();
    if nb == 0 || body.is_empty() {
        return Err(InputError::InvalidDimension {
            field: "applied_force_topology",
        });
    }
    nb.checked_mul(2)
        .and_then(|v| v.checked_add(body.len()))
        .and_then(|v| v.checked_add(8))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "applied_force_metadata",
        })?;
    if root.len() != nb {
        return Err(InputError::LengthMismatch {
            field: "body_rootid",
            expected: nb,
            actual: root.len(),
        });
    }
    for b in 0..nb {
        let p = parent[b];
        if (b == 0 && p != 0) || (b > 0 && (p < 0 || p as usize >= b)) {
            return Err(InputError::InvalidTopology {
                field: "body_parentid",
                index: b,
                reason: "parent_must_precede_child",
            });
        }
        let r = root[b];
        if r < 0 || r as usize >= nb {
            return Err(InputError::InvalidTopology {
                field: "body_rootid",
                index: b,
                reason: "root_out_of_range",
            });
        }
        let expected = if b == 0 {
            0
        } else if p == 0 {
            b as i32
        } else {
            root[p as usize]
        };
        if r != expected {
            return Err(InputError::InvalidTopology {
                field: "body_rootid",
                index: b,
                reason: "root_must_follow_parent_chain",
            });
        }
    }
    if let Some(index) = body.iter().position(|&b| b < 0 || b as usize >= nb) {
        return Err(InputError::InvalidTopology {
            field: "dof_bodyid",
            index,
            reason: "body_reference_out_of_range",
        });
    }
    Ok(Topology { parent, root, body })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppliedForceGeometry {
    Xipos,
    SubtreeCom,
    Cdof,
}
impl AppliedForceGeometry {
    fn name(self) -> &'static str {
        match self {
            Self::Xipos => "xipos",
            Self::SubtreeCom => "subtree_com",
            Self::Cdof => "cdof",
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Xipos => 0,
            Self::SubtreeCom => 1,
            Self::Cdof => 2,
        }
    }
}
#[derive(Clone, Copy)]
struct Layout {
    vector: BatchLayout,
    geometry: [BatchLayout; 3],
    offsets: [usize; 3],
    // 默认配置无设备路径；仅cuda-probe读取。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    geometry_len: usize,
    ft: BatchLayout,
}
fn layout(worlds: usize, nb: usize, n: usize) -> Result<Layout, InputError> {
    let overflow = || InputError::Overflow {
        field: "applied_force_layout",
    };
    if nb == 0
        || n == 0
        || worlds == 0
        || worlds
            .checked_mul(n)
            .is_none_or(|v| v > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "applied_force_work",
        });
    }
    nb.checked_mul(2)
        .and_then(|v| v.checked_add(n))
        .and_then(|v| v.checked_add(8))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or_else(overflow)?;
    let xpos = BatchLayout::new(worlds, nb.checked_mul(3).ok_or_else(overflow)?, 4)?;
    let cdof = BatchLayout::new(worlds, n.checked_mul(6).ok_or_else(overflow)?, 4)?;
    let offset2 = xpos.total_elements().checked_mul(2).ok_or_else(overflow)?;
    let geometry_len = offset2
        .checked_add(cdof.total_elements())
        .ok_or_else(overflow)?;
    let vector = BatchLayout::new(worlds, n, 4)?;
    let ft = BatchLayout::new(worlds, nb.checked_mul(6).ok_or_else(overflow)?, 4)?;
    for count in [
        geometry_len,
        vector.total_elements(),
        ft.total_elements(),
        worlds,
    ] {
        BatchLayout::new(1, count.checked_add(8).ok_or_else(overflow)?, 4)?;
    }
    Ok(Layout {
        vector,
        geometry: [xpos, xpos, cdof],
        offsets: [0, xpos.total_elements(), offset2],
        geometry_len,
        ft,
    })
}

#[cfg(feature = "cuda-probe")]
const APPLIED_FORCE_DELTA_METADATA_KIND: ProofKind = ProofKind("applied_force_delta_metadata");

#[cfg(feature = "cuda-probe")]
const APPLIED_FORCE_DELTA_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: APPLIED_FORCE_DELTA_METADATA_KIND,
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
struct AppliedForceDeltaSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结外力增量ABI。
unsafe impl KernelAbi for AppliedForceDeltaSpec {
    const NAME: &'static str = "applied_force_delta";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &APPLIED_FORCE_DELTA_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for AppliedForceDeltaSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let [metadata, geometry, ft, output] = applied_force_delta_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: geometry,
            },
            SlotRequirement {
                slot: 2,
                min_elements: ft,
            },
            SlotRequirement {
                slot: 3,
                min_elements: output,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        applied_force_delta_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn applied_force_delta_lengths(dims: &[u32; 4]) -> Result<[u64; 4], LauncherError> {
    let [nb, n, worlds, work] = *dims;
    if nb == 0 || n == 0 || worlds == 0 || work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("外力增量维度无效"));
    }
    if worlds.checked_mul(n) != Some(work) {
        return Err(LauncherError::InvalidArray("外力增量工作量不匹配"));
    }
    let nb = u64::from(nb);
    let n = u64::from(n);
    let worlds = u64::from(worlds);
    let metadata = nb
        .checked_mul(2)
        .and_then(|value| value.checked_add(n))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("applied_force_delta_metadata"))?;
    let geometry = nb
        .checked_add(n)
        .and_then(|value| value.checked_mul(worlds))
        .and_then(|value| value.checked_mul(6))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("applied_force_delta_geometry"))?;
    let ft = nb
        .checked_mul(worlds)
        .and_then(|value| value.checked_mul(6))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("applied_force_delta_ft"))?;
    let output = u64::from(work)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("applied_force_delta_output"))?;
    Ok([metadata, geometry, ft, output])
}

#[cfg(feature = "cuda-probe")]
fn validate_applied_force_metadata(values: &[i32], nb: u32, n: u32) -> Result<(), LauncherError> {
    let expected = u64::from(nb)
        .checked_mul(2)
        .and_then(|value| value.checked_add(u64::from(n)))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("applied_force_delta_metadata"))?;
    if expected > i32::MAX as u64 || u64::try_from(values.len()).ok() != Some(expected) {
        return Err(LauncherError::InvalidMetadata("外力元数据长度无效"));
    }
    if values[..4] != [-131072; 4] || values[values.len() - 4..] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("外力元数据守卫无效"));
    }
    let nb = usize::try_from(nb).map_err(|_| LauncherError::Overflow("applied_force_nb"))?;
    let n = usize::try_from(n).map_err(|_| LauncherError::Overflow("applied_force_n"))?;
    let parent = &values[4..4 + nb];
    let root = &values[4 + nb..4 + 2 * nb];
    let body = &values[4 + 2 * nb..4 + 2 * nb + n];
    for b in 0..nb {
        let p = parent[b];
        if (b == 0 && p != 0) || (b > 0 && (p < 0 || p as usize >= b)) {
            return Err(LauncherError::InvalidMetadata("外力父体索引无效"));
        }
        let r = root[b];
        if r < 0 || r as usize >= nb {
            return Err(LauncherError::InvalidMetadata("外力根索引越界"));
        }
        let expected = if b == 0 {
            0
        } else if p == 0 {
            b as i32
        } else {
            root[p as usize]
        };
        if r != expected {
            return Err(LauncherError::InvalidMetadata("外力根索引无效"));
        }
    }
    if body.iter().any(|&value| value < 0 || value as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("外力自由度体索引越界"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
struct StaticAppliedForceMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticAppliedForceMetadata {
    type Dims = [u32; 2];

    const KIND: ProofKind = APPLIED_FORCE_DELTA_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_applied_force_metadata(values, dims[0], dims[1])
    }
}

#[cfg(feature = "cuda-probe")]
struct AppliedForceDeltaMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for AppliedForceDeltaMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = APPLIED_FORCE_DELTA_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        applied_force_delta_lengths(dims)?;
        validate_applied_force_metadata(values, dims[0], dims[1])
    }
}

#[cfg(feature = "cuda-probe")]
fn applied_force_delta_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "applied_force_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "applied_force_delta_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
struct AppliedForceMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    dims: [u32; 2],
    revision: MetadataRevision,
    checked: Checked<StaticAppliedForceMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl AppliedForceMetadataBuffer {
    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        dims: [u32; 2],
    ) -> Result<Self, TransferError> {
        StaticAppliedForceMetadata::validate(&values, &dims)
            .map_err(applied_force_delta_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且设备跨度来自该缓冲。
        let checked = unsafe {
            Checked::<StaticAppliedForceMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(applied_force_delta_launch_error)?;
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

    fn proof(&self, dims: [u32; 4]) -> Result<Checked<AppliedForceDeltaMetadata>, TransferError> {
        if self.dims != dims[..2] {
            return Err(InputError::InvalidDimension {
                field: "applied_force_delta_static_dims",
            }
            .into());
        }
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(applied_force_delta_launch_error)?;
        // SAFETY: 私有包装保留上传值、设备跨度及写入代际。
        unsafe {
            Checked::<AppliedForceDeltaMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(applied_force_delta_launch_error)
    }

    #[cfg(test)]
    fn write_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入同步完成且跨度未变。
        self.checked = unsafe {
            Checked::<StaticAppliedForceMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(applied_force_delta_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
struct AppliedForceDeltaKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl AppliedForceDeltaKernel {
    fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "applied_force_delta")?,
        })
    }

    fn launch(
        &self,
        metadata: &AppliedForceMetadataBuffer,
        geometry: &TransferBuffer<f32>,
        ft: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: geometry.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: ft.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<AppliedForceDeltaSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(applied_force_delta_launch_error)?;
        // SAFETY: 类型化计划核对元数据、跨度、会话和256线程网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), geometry, ft, output, dims)
        }
    }
}

pub struct AppliedForcePlan {
    input: Arc<Topology>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: AppliedForceMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validate: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    delta: AppliedForceDeltaKernel,
    #[cfg(feature = "cuda-probe")]
    publish: SynchronousKernel,
}
pub struct AppliedForceData {
    input: Arc<Topology>,
    layout: Layout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    geometry: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    ft: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    delta: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct AppliedForceOutput {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl AppliedForceOutput {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
impl AppliedForcePlan {
    pub fn new(
        session: &TransferSession,
        body_parentid: Vec<i32>,
        body_rootid: Vec<i32>,
        dof_bodyid: Vec<i32>,
    ) -> Result<Self, TransferError> {
        let input = topology(body_parentid, body_rootid, dof_bodyid)?;
        layout(1, input.parent.len(), input.body.len())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = session;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut meta = vec![-131072; 4];
            meta.extend(&input.parent);
            meta.extend(&input.root);
            meta.extend(&input.body);
            meta.extend([-131072; 4]);
            let nb = u32::try_from(input.parent.len()).map_err(|_| InputError::Overflow {
                field: "applied_force_nb",
            })?;
            let n = u32::try_from(input.body.len()).map_err(|_| InputError::Overflow {
                field: "applied_force_n",
            })?;
            Ok(Self {
                input: Arc::new(input),
                session: session.clone(),
                metadata: AppliedForceMetadataBuffer::upload(session, meta, [nb, n])?,
                validate: SynchronousKernel::compile(session, CUDA, "applied_force_validate")?,
                delta: AppliedForceDeltaKernel::compile(session)?,
                publish: SynchronousKernel::compile(session, CUDA, "applied_force_publish")?,
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<AppliedForceData, TransferError> {
        let l = layout(worlds, self.input.parent.len(), self.input.body.len())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = l;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let alloc = |count, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            Ok(AppliedForceData {
                input: self.input.clone(),
                layout: l,
                ready: false,
                geometry: alloc(l.geometry_len, 0.0)?,
                ft: alloc(l.ft.total_elements(), 0.0)?,
                current: alloc(l.vector.total_elements(), 0.0)?,
                delta: alloc(l.vector.total_elements(), f32::NAN)?,
                staged: alloc(l.vector.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    /// U075语义：每次调用累加到当前广义力。
    pub fn accumulate(&self, data: &mut AppliedForceData) -> Result<(), TransferError> {
        self.apply(data, true)
    }
    pub fn replace(&self, data: &mut AppliedForceData) -> Result<(), TransferError> {
        self.apply(data, false)
    }
    fn apply(&self, data: &mut AppliedForceData, add: bool) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = add;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let nb = self.input.parent.len() as u32;
            let n = self.input.body.len() as u32;
            let worlds = data.worlds() as u32;
            let work = data.layout.vector.total_elements() as u32;
            // SAFETY: Same-plan metadata owns2*nb+n guarded indexes. Validator
            // checks parent ordering/root/DOF bounds before traversal. Geometry
            // and FT own their fully checked guarded world/field strides.
            unsafe {
                self.validate.launch(
                    self.metadata.as_buffer(),
                    &data.geometry,
                    &data.ft,
                    &mut data.status,
                    [nb, n, worlds, worlds],
                )?;
            }
            let status = read(&data.status, worlds as usize, "applied_force_status")?;
            if let Some(index) = status[4..4 + worlds as usize]
                .iter()
                .position(|&v| v != 1.0)
            {
                return Err(InputError::InvalidTopology {
                    field: "applied_force_source",
                    index,
                    reason: "nonfinite_guard_or_mapping",
                }
                .into());
            }
            if add {
                read(
                    &data.current,
                    data.layout.vector.total_elements(),
                    "qfrc_current",
                )?;
            } else {
                guards(&data.current)?;
            }
            self.delta.launch(
                &self.metadata,
                &data.geometry,
                &data.ft,
                &mut data.delta,
                [nb, n, worlds, work],
            )?;
            read(
                &data.delta,
                data.layout.vector.total_elements(),
                "applied_force_delta",
            )?;
            // SAFETY: Disjoint current/delta/staged guarded vector buffers share
            // one checked work stride. Final RN add or copy owns one slot/thread.
            unsafe {
                self.publish.launch(
                    self.metadata.as_buffer(),
                    &data.current,
                    &data.delta,
                    &mut data.staged,
                    [n, worlds, u32::from(add), work],
                )?;
            }
            read(
                &data.staged,
                data.layout.vector.total_elements(),
                "qfrc_staged",
            )?;
            // 只有完整有限结果才替换当前缓冲。
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(())
        }
    }
}
#[cfg(feature = "cuda-probe")]
fn read(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut v)?;
    crate::physics::check_device_values(&v, field)?;
    Ok(v)
}
#[cfg(feature = "cuda-probe")]
fn guards(buffer: &TransferBuffer<f32>) -> Result<(), TransferError> {
    let mut v = [0.0; 8];
    buffer.read_range_into(0, &mut v[..4])?;
    buffer.read_range_into(buffer.len() - 4, &mut v[4..])?;
    crate::physics::check_device_guards(&v)
}
fn check(values: &[f32], expected: usize, field: &'static str) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite { field, index });
    }
    Ok(())
}
impl AppliedForceData {
    pub fn worlds(&self) -> usize {
        self.layout.vector.worlds()
    }
    pub fn write_geometry(
        &mut self,
        field: AppliedForceGeometry,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let i = field.index();
        self.geom(field, 0, self.layout.geometry[i].total_elements(), values)
    }
    pub fn write_world_geometry(
        &mut self,
        field: AppliedForceGeometry,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.layout.geometry[field.index()].world_elements(world)?;
        self.geom(field, r.start, r.len(), values)
    }
    fn geom(
        &mut self,
        field: AppliedForceGeometry,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check(values, expected, field.name())?;
        let a = 4 + self.layout.offsets[field.index()] + offset;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = a;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.geometry.write_range(a, values)
        }
    }
    pub fn write_ft(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(false, 0, self.layout.ft.total_elements(), values)
    }
    pub fn write_world_ft(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let r = self.layout.ft.world_elements(world)?;
        self.write(false, r.start, r.len(), values)
    }
    pub fn write_qfrc(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(true, 0, self.layout.vector.total_elements(), values)
    }
    pub fn write_world_qfrc(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let r = self.layout.vector.world_elements(world)?;
        self.write(true, r.start, r.len(), values)
    }
    fn write(
        &mut self,
        qfrc: bool,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check(values, expected, if qfrc { "qfrc" } else { "xfrc_applied" })?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if qfrc {
                self.current.write_range(4 + offset, values)
            } else {
                self.ft.write_range(4 + offset, values)
            }
        }
    }
    pub fn readback(&self) -> Result<AppliedForceOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "xfrc_accumulate",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(AppliedForceOutput {
                layout: self.layout.vector,
                values: read(&self.current, self.layout.vector.total_elements(), "qfrc")?,
            })
        }
    }
}
// Frozen support.py and Warp vec.h (Apache-2.0): preserve C++ contraction
// permission inside cross/dot; no atomics, no changed floating precision.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void applied_force_validate(const int* meta,const float* geometry,
 const float* ft,float* status,unsigned nb,unsigned n,unsigned worlds,unsigned work) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 bool valid=true;unsigned long long ml=2ull*nb+n,gl=6ull*worlds*(nb+n),fl=6ull*worlds*nb;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[4+ml+j]==-131072&&geometry[j]==-131072.0f&&geometry[4+gl+j]==-131072.0f&&ft[j]==-131072.0f&&ft[4+fl+j]==-131072.0f;
 const int* parent=meta+4;const int* root=parent+nb;const int* body=root+nb;
 for(unsigned b=0;b<nb;++b){int p=parent[b],r=root[b];if((b==0&&p!=0)||(b>0&&(p<0||(unsigned)p>=b))||r<0||(unsigned)r>=nb){valid=false;continue;}
  int expected=b==0?0:(p==0?(int)b:root[p]);if(r!=expected)valid=false;}
 for(unsigned i=0;i<n;++i)if(body[i]<0||(unsigned)body[i]>=nb)valid=false;
 for(unsigned long long j=0;j<3ull*nb;++j)valid=valid&&isfinite(geometry[4+3ull*w*nb+j])&&isfinite(geometry[4+3ull*worlds*nb+3ull*w*nb+j]);
 for(unsigned long long j=0;j<6ull*n;++j)valid=valid&&isfinite(geometry[4+6ull*worlds*nb+6ull*w*n+j]);
 for(unsigned long long j=0;j<6ull*nb;++j)valid=valid&&isfinite(ft[4+6ull*w*nb+j]);status[4+w]=valid?1.0f:0.0f;
}
__device__ float force_dot6(const float* a,const float* b){float ret=0.0f;
 #pragma unroll
 for(unsigned i=0;i<6;++i)ret+=a[i]*b[i];return ret;}
__device__ float force_dot3(const float* a,const float* b){return a[0]*b[0]+a[1]*b[1]+a[2]*b[2];}
extern "C" __global__ void applied_force_delta(const int* meta,const float* geometry,
 const float* ft,float* output,unsigned nb,unsigned n,unsigned worlds,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/n,i=tid%n;
 const int* parent=meta+4;const int* root=parent+nb;const int* body=root+nb;
 const float* xpos=geometry+4+3ull*w*nb;const float* com=geometry+4+3ull*worlds*nb+3ull*w*nb;
 const float* cdof=geometry+4+6ull*worlds*nb+6ull*w*n+6ull*i;const float* force=ft+4+6ull*w*nb;
 float jac[6]={cdof[3],cdof[4],cdof[5],cdof[0],cdof[1],cdof[2]};float accum=0.0f;int dofbody=body[i];
 for(unsigned b=dofbody;b<nb;++b){const float* f=force+6ull*b;if(f[0]==0.0f&&f[1]==0.0f&&f[2]==0.0f&&f[3]==0.0f&&f[4]==0.0f&&f[5]==0.0f)continue;
  int p=b;while(p!=0&&p!=dofbody)p=parent[p];if(p==0)continue;
  const float* point=xpos+3ull*b;const float* center=com+3ull*(unsigned)root[b];
  float offset[3]={point[0]-center[0],point[1]-center[1],point[2]-center[2]};
  float cross[3]={cdof[1]*offset[2]-cdof[2]*offset[1],cdof[2]*offset[0]-cdof[0]*offset[2],cdof[0]*offset[1]-cdof[1]*offset[0]};
  accum+=force_dot6(jac,f)+force_dot3(cross,f);
 }output[4+tid]=accum;
}
extern "C" __global__ void applied_force_publish(const int* meta,const float* current,
 const float* delta,float* staged,unsigned n,unsigned worlds,unsigned add,unsigned work){
 unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i>=work)return;
 staged[4+i]=add?__fadd_rn(current[4+i],delta[4+i]):delta[4+i];
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn reference() -> serde_json::Value {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/applied-force/reference.json"
        )))
        .unwrap()
    }
    fn ints(v: &serde_json::Value, key: &str) -> Vec<i32> {
        v[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i32)
            .collect()
    }
    fn floats(v: &serde_json::Value, key: &str) -> Vec<f32> {
        v[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect()
    }
    // Independent virtual work: project torque+offset×force, not the GPU jac
    // permutation or dot6+dot3 decomposition. Diagnostic f64 never feeds GPU.
    fn oracle(t: &Topology, xpos: &[f32], com: &[f32], cdof: &[f32], ft: &[f32]) -> Vec<f64> {
        let mut out = vec![0.0; t.body.len()];
        for (i, value) in out.iter_mut().enumerate() {
            let d = t.body[i];
            if d == 0 {
                continue;
            }
            for b in 1..t.parent.len() {
                let mut p = b as i32;
                while p != 0 && p != d {
                    p = t.parent[p as usize];
                }
                if p != d {
                    continue;
                }
                let mut r = [0.0; 3];
                let mut f = [0.0; 3];
                for k in 0..3 {
                    r[k] = f64::from(xpos[3 * b + k]) - f64::from(com[3 * t.root[b] as usize + k]);
                    f[k] = f64::from(ft[6 * b + k]);
                }
                let moment = [
                    r[1] * f[2] - r[2] * f[1],
                    r[2] * f[0] - r[0] * f[2],
                    r[0] * f[1] - r[1] * f[0],
                ];
                for k in 0..3 {
                    *value +=
                        f64::from(cdof[6 * i + k]) * (f64::from(ft[6 * b + 3 + k]) + moment[k]);
                    *value += f64::from(cdof[6 * i + 3 + k]) * f[k];
                }
            }
        }
        out
    }
    #[test]
    fn rejects_cycles_wrong_roots_and_bad_counts_before_device_allocation() {
        for p in [vec![], vec![1], vec![0, -1], vec![0, 1], vec![0, 2, 1]] {
            assert!(topology(p.clone(), vec![0; p.len()], vec![0]).is_err());
        }
        for r in [vec![0], vec![0, -1], vec![0, 2], vec![0, 0]] {
            assert!(topology(vec![0, 0], r, vec![1]).is_err());
        }
        assert!(topology(vec![0], vec![0], vec![]).is_err());
        for d in [-1, 1, i32::MAX] {
            assert!(topology(vec![0], vec![0], vec![d]).is_err());
        }
        for args in [
            (0, 4, 4),
            (1, 0, 1),
            (1, 1, 0),
            (usize::MAX, 1, 1),
            (1, usize::MAX, 1),
            (1, 1, usize::MAX),
        ] {
            assert!(layout(args.0, args.1, args.2).is_err());
        }
        let l = layout(257, 273, 513).unwrap();
        assert_eq!(l.geometry_len, 257 * 6 * (273 + 513));
        assert_eq!(l.ft.total_elements(), 257 * 6 * 273);
        assert!(check(&[f32::NAN], 1, "ft").is_err());
        assert!(check(&[], 1, "ft").is_err());
    }
    #[test]
    fn independently_certifies_frame_component_order_and_subtree_exclusion() {
        let v = reference();
        let t = topology(
            ints(&v, "parents"),
            ints(&v, "roots"),
            ints(&v, "dof_bodyid"),
        )
        .unwrap();
        let q = oracle(
            &t,
            &floats(&v, "xipos"),
            &floats(&v, "subtree_com"),
            &floats(&v, "cdof"),
            &floats(&v, "ft_force_torque"),
        );
        assert_eq!(q, vec![4.0, -12.0, -40.0, 0.0]);
        for (i, &x) in q.iter().enumerate() {
            assert_eq!(
                (x as f32).to_bits(),
                u32::from_str_radix(v["delta_bits"][i].as_str().unwrap(), 16).unwrap()
            );
        }
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    fn delta_wrapper_rejects_capacity_context_alias_metadata_and_work_faults() {
        use crate::runtime::launcher::DeviceSpan;

        let dims = [2, 1, 2, 2];
        let values = [
            -131072, -131072, -131072, -131072, 0, 0, 0, 1, 1, -131072, -131072, -131072, -131072,
        ];
        let revision = MetadataRevision::new();
        let meta = DeviceSpan::new(7, 0x1000, 52).unwrap();
        // SAFETY: 测试跨度代表与宿主值一致的只读元数据。
        let proof = unsafe {
            Checked::<AppliedForceDeltaMetadata>::assume_uploaded(
                meta,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: meta,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(7, 0x2000, 176).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(7, 0x3000, 128).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(7, 0x4000, 40).unwrap(),
                elem: ElemType::F32,
            },
        ];
        let kernel = Kernel::<AppliedForceDeltaSpec>::new().unwrap();
        assert!(kernel.plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

        let mut short = bindings;
        short[3].span = DeviceSpan::new(7, 0x4000, 36).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &short, &[proof.binding(0)]),
            Err(LauncherError::Capacity { slot: 3, .. })
        ));
        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x3000, 128).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &foreign, &[proof.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));
        let mut alias = bindings;
        alias[3].span = DeviceSpan::new(7, 0x2000, 40).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3
            })
        ));
        revision.invalidate();
        assert!(matches!(
            kernel.plan(&dims, &bindings, &[proof.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));

        let mut bad = values;
        bad[8] = 2;
        assert!(AppliedForceDeltaMetadata::validate(&bad, &dims).is_err());
        assert!(applied_force_delta_lengths(&[2, 1, 2, 1]).is_err());
        assert!(applied_force_delta_lengths(&[1, 1, u32::MAX, u32::MAX]).is_err());
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    fn delta_metadata_span_accepts_i32_max_and_rejects_next_nb() {
        const MAX_NB: u32 = (i32::MAX as u32 - 9) / 2;

        let accepted = applied_force_delta_lengths(&[MAX_NB, 1, 1, 1]).unwrap();
        assert_eq!(accepted[0], i32::MAX as u64);

        assert!(matches!(
            applied_force_delta_lengths(&[MAX_NB, 2, 1, 2]),
            Err(LauncherError::Overflow("applied_force_delta_metadata"))
        ));
        assert!(matches!(
            applied_force_delta_lengths(&[MAX_NB + 1, 1, 1, 1]),
            Err(LauncherError::Overflow("applied_force_delta_metadata"))
        ));
    }
    #[cfg(feature = "cuda-probe")]
    fn model(nb: usize, n: usize) -> Topology {
        let parent = (0..nb)
            .map(|b| {
                if b == 0 || (b - 1) % 17 == 0 {
                    0
                } else {
                    (b - 1) as i32
                }
            })
            .collect::<Vec<_>>();
        let mut root = vec![0; nb];
        for b in 1..nb {
            root[b] = if parent[b] == 0 {
                b as i32
            } else {
                root[parent[b] as usize]
            };
        }
        topology(
            parent,
            root,
            (0..n).map(|i| ((i * 17 + 1) % nb) as i32).collect(),
        )
        .unwrap()
    }
    #[cfg(feature = "cuda-probe")]
    type Scene = (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>);

    #[cfg(feature = "cuda-probe")]
    fn scene(t: &Topology, w: usize, sample: usize) -> Scene {
        let nb = t.parent.len();
        let n = t.body.len();
        let x = (0..3 * nb)
            .map(|j| ((j / 3 + j % 3 + sample) % 9) as f32 * 0.25 - 1.0 + (w % 3) as f32 * 0.5)
            .collect();
        let c = (0..3 * nb)
            .map(|j| {
                ((2 * (j / 3) + j % 3 + sample) % 7) as f32 * 0.25 - 0.75 + (w % 3) as f32 * 0.5
            })
            .collect();
        let d = (0..6 * n)
            .map(|j| ((j / 6 + 2 * (j % 6) + sample) % 7) as f32 * 0.125 - 0.375)
            .collect();
        let ft = (0..6 * nb)
            .map(|j| {
                if j < 6 {
                    ((j + sample) % 7) as f32 * 128.0
                } else if (j / 6) % 4 == 0 {
                    if j % 2 == 0 { 0.0 } else { -0.0 }
                } else {
                    ((j / 6 + 3 * (j % 6) + w + sample) % 9) as f32 * 0.25 - 1.0
                }
            })
            .collect();
        let q = (0..n)
            .map(|i| {
                let v = ((w + i + sample) % 9) as f32 * 0.25 - 1.0;
                if v == 0.0 && i % 2 == 0 { -0.0 } else { v }
            })
            .collect();
        (x, c, d, ft, q)
    }
    #[cfg(feature = "cuda-probe")]
    fn fill(plan: &AppliedForcePlan, data: &mut AppliedForceData, sample: usize) -> Vec<Vec<f64>> {
        let mut x = Vec::new();
        let mut c = Vec::new();
        let mut d = Vec::new();
        let mut f = Vec::new();
        let mut q = Vec::new();
        let mut expected = Vec::new();
        for w in 0..data.worlds() {
            let (wx, wc, wd, wf, wq) = scene(&plan.input, w, sample);
            expected.push(oracle(&plan.input, &wx, &wc, &wd, &wf));
            x.extend(wx);
            c.extend(wc);
            d.extend(wd);
            f.extend(wf);
            q.extend(wq);
        }
        data.write_geometry(AppliedForceGeometry::Xipos, &x)
            .unwrap();
        data.write_geometry(AppliedForceGeometry::SubtreeCom, &c)
            .unwrap();
        data.write_geometry(AppliedForceGeometry::Cdof, &d).unwrap();
        data.write_ft(&f).unwrap();
        data.write_qfrc(&q).unwrap();
        expected
    }
    #[cfg(feature = "cuda-probe")]
    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|v| v.to_bits()).collect()
    }
    #[cfg(feature = "cuda-probe")]
    fn raw(v: &TransferBuffer<f32>) -> Vec<f32> {
        let mut out = vec![0.0; v.len()];
        v.read_range_into(0, &mut out).unwrap();
        out
    }
    #[cfg(feature = "cuda-probe")]
    fn raw_meta(v: &TransferBuffer<i32>) -> Vec<i32> {
        let mut out = vec![0; v.len()];
        v.read_range_into(0, &mut out).unwrap();
        out
    }
    #[cfg(feature = "cuda-probe")]
    fn tokens(d: &AppliedForceData) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
        (
            bits(&raw(&d.geometry)),
            bits(&raw(&d.ft)),
            bits(&raw(&d.current)),
        )
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new U075 projection/add/replace, independent lever reference and exact dyadic coverage"]
    fn gpu_projects_subtrees_and_repeatedly_accumulates_current_force_without_old_producers() {
        let session = TransferSession::new(0).unwrap();
        let start = std::time::Instant::now();
        let mut batches = 0;
        let mut slots = 0;
        let mut delta_slots = 0;
        let mut capacity = 0;
        let v = reference();
        // Fixed expected bits independently certify sign, component and root use.
        let plan = AppliedForcePlan::new(
            &session,
            ints(&v, "parents"),
            ints(&v, "roots"),
            ints(&v, "dof_bodyid"),
        )
        .unwrap();
        let mut d = plan.create_data(1).unwrap();
        for (field, key) in [
            (AppliedForceGeometry::Xipos, "xipos"),
            (AppliedForceGeometry::SubtreeCom, "subtree_com"),
            (AppliedForceGeometry::Cdof, "cdof"),
        ] {
            d.write_geometry(field, &floats(&v, key)).unwrap();
        }
        d.write_ft(&floats(&v, "ft_force_torque")).unwrap();
        d.write_qfrc(&floats(&v, "initial_qfrc")).unwrap();
        plan.accumulate(&mut d).unwrap();
        let added = d.readback().unwrap();
        for (i, &x) in added.world(0).unwrap().iter().enumerate() {
            assert_eq!(
                x.to_bits(),
                u32::from_str_radix(v["added_bits"][i].as_str().unwrap(), 16).unwrap()
            );
        }
        plan.replace(&mut d).unwrap();
        for (i, &x) in d.readback().unwrap().world(0).unwrap().iter().enumerate() {
            assert_eq!(
                x.to_bits(),
                u32::from_str_radix(v["delta_bits"][i].as_str().unwrap(), 16).unwrap()
            );
        }
        plan.accumulate(&mut d).unwrap();
        for (&x, &a) in d
            .readback()
            .unwrap()
            .world(0)
            .unwrap()
            .iter()
            .zip([4.0, -12.0, -40.0, 0.0].iter())
        {
            assert_eq!(x, 2.0 * a);
        }
        for nb in [8, 273] {
            for n in [1, 65, 513] {
                let t = model(nb, n);
                let plan = AppliedForcePlan::new(
                    &session,
                    t.parent.clone(),
                    t.root.clone(),
                    t.body.clone(),
                )
                .unwrap();
                for sample in 0..2 {
                    for worlds in [1, 2, 257] {
                        let mut d = plan.create_data(worlds).unwrap();
                        assert!(d.readback().is_err());
                        let expected = fill(&plan, &mut d, sample);
                        for add in [false, true] {
                            let seed = (0..worlds)
                                .flat_map(|w| scene(&t, w, sample).4)
                                .collect::<Vec<_>>();
                            d.write_qfrc(&seed).unwrap();
                            if add {
                                plan.accumulate(&mut d).unwrap();
                            } else {
                                plan.replace(&mut d).unwrap();
                            }
                            let out = d.readback().unwrap();
                            let delta = raw(&d.delta);
                            for w in 0..worlds {
                                for i in 0..n {
                                    let e = expected[w][i];
                                    assert_eq!(
                                        f64::from(e as f32),
                                        e,
                                        "dyadic reference must be exactly representable"
                                    );
                                    assert_eq!(
                                        delta[4 + w * n + i].to_bits(),
                                        (e as f32).to_bits()
                                    );
                                    delta_slots += 1;
                                    let final_e = if add {
                                        f64::from(seed[w * n + i]) + e
                                    } else {
                                        e
                                    };
                                    assert_eq!(f64::from(final_e as f32), final_e);
                                    assert_eq!(
                                        out.world(w).unwrap()[i].to_bits(),
                                        (final_e as f32).to_bits()
                                    );
                                    slots += 1;
                                }
                            }
                            assert!(out.world(worlds).is_err());
                            capacity = capacity.max(
                                4 * (plan.metadata.len()
                                    + d.geometry.len()
                                    + d.ft.len()
                                    + d.current.len()
                                    + d.delta.len()
                                    + d.staged.len()
                                    + d.status.len()),
                            );
                            batches += 1;
                        }
                    }
                }
            }
        }
        assert_eq!((batches, slots, delta_slots), (72, 1204320, 1204320));
        // FMA side is a compiler-order diagnostic, not frozen Warp acceptance.
        let c = &v["fma_control"];
        let plan = AppliedForcePlan::new(
            &session,
            ints(c, "parents"),
            ints(c, "roots"),
            ints(c, "dof_bodyid"),
        )
        .unwrap();
        let mut d = plan.create_data(1).unwrap();
        let from_bits = |key: &str| {
            c[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| f32::from_bits(u32::from_str_radix(v.as_str().unwrap(), 16).unwrap()))
                .collect::<Vec<_>>()
        };
        d.write_geometry(AppliedForceGeometry::Xipos, &from_bits("xipos_bits"))
            .unwrap();
        d.write_geometry(
            AppliedForceGeometry::SubtreeCom,
            &from_bits("subtree_com_bits"),
        )
        .unwrap();
        d.write_geometry(AppliedForceGeometry::Cdof, &floats(c, "cdof"))
            .unwrap();
        d.write_ft(&from_bits("ft_bits")).unwrap();
        plan.replace(&mut d).unwrap();
        let fma = d.readback().unwrap().world(0).unwrap()[0];
        let fma_expected = c["independent_f64_delta"].as_f64().unwrap();
        assert!((f64::from(fma) - fma_expected).abs() <= 2e-7 + 2e-7 * fma_expected.abs());
        assert!(
            c["compiler_order_diagnostic_bits"]
                .as_array()
                .unwrap()
                .iter()
                .any(|b| u32::from_str_radix(b.as_str().unwrap(), 16).unwrap() == fma.to_bits())
        );
        // New geometry/FT/current-force lifecycle, no old qpos/RHS stage calls.
        let t = model(8, 65);
        let plan =
            AppliedForcePlan::new(&session, t.parent.clone(), t.root.clone(), t.body.clone())
                .unwrap();
        let other =
            AppliedForcePlan::new(&session, t.parent.clone(), t.root.clone(), t.body.clone())
                .unwrap();
        let mut d = plan.create_data(257).unwrap();
        let expected = fill(&plan, &mut d, 0);
        plan.replace(&mut d).unwrap();
        let before = d.readback().unwrap();
        plan.accumulate(&mut d).unwrap();
        // 同一下标同时读取回读与参考。
        #[allow(clippy::needless_range_loop)]
        for w in 0..257 {
            for (i, &value) in d.readback().unwrap().world(w).unwrap().iter().enumerate() {
                assert_eq!(value.to_bits(), (2.0 * expected[w][i] as f32).to_bits());
            }
        }
        plan.replace(&mut d).unwrap();
        assert_eq!(bits(&before.values), bits(&d.readback().unwrap().values));
        let unchanged = tokens(&d);
        assert!(other.accumulate(&mut d).is_err());
        assert_eq!(tokens(&d), unchanged);
        assert!(d.readback().is_ok());
        for field in [
            AppliedForceGeometry::Xipos,
            AppliedForceGeometry::SubtreeCom,
            AppliedForceGeometry::Cdof,
        ] {
            let width = d.layout.geometry[field.index()].elements_per_world();
            assert!(
                d.write_world_geometry(field, 257, &vec![0.0; width])
                    .is_err()
            );
            assert!(
                d.write_world_geometry(field, 0, &vec![0.0; width - 1])
                    .is_err()
            );
            let mut nan = vec![0.0; width];
            nan[0] = f32::NAN;
            assert!(d.write_world_geometry(field, 0, &nan).is_err());
            assert_eq!(tokens(&d), unchanged);
            assert!(d.readback().is_ok());
        }
        assert!(d.write_world_ft(257, &[0.0; 48]).is_err());
        assert!(d.write_world_ft(0, &[0.0; 47]).is_err());
        assert!(d.write_world_qfrc(257, &[0.0; 65]).is_err());
        assert!(d.write_world_qfrc(0, &[0.0; 64]).is_err());
        assert!(d.write_ft(&[f32::INFINITY]).is_err());
        assert_eq!(tokens(&d), unchanged);
        assert!(d.readback().is_ok());
        for field in [
            AppliedForceGeometry::Xipos,
            AppliedForceGeometry::SubtreeCom,
            AppliedForceGeometry::Cdof,
        ] {
            let old = raw(&d.geometry);
            let l = d.layout.geometry[field.index()];
            let r = l.world_elements(256).unwrap();
            let a = 4 + d.layout.offsets[field.index()] + r.start;
            let changed = old[a..a + r.len()]
                .iter()
                .map(|v| v + 0.125)
                .collect::<Vec<_>>();
            d.write_world_geometry(field, 256, &changed).unwrap();
            let now = raw(&d.geometry);
            for i in (0..a).chain(a + r.len()..old.len()) {
                assert_eq!(old[i].to_bits(), now[i].to_bits());
            }
            assert_eq!(bits(&now[a..a + r.len()]), bits(&changed));
            assert!(d.readback().is_err());
            d.write_world_geometry(field, 256, &old[a..a + r.len()])
                .unwrap();
            plan.replace(&mut d).unwrap();
            assert_eq!(bits(&d.readback().unwrap().values), bits(&before.values));
        }
        let old = raw(&d.ft);
        let a = 4 + 256 * 48;
        d.write_world_ft(256, &[-0.0; 48]).unwrap();
        let now = raw(&d.ft);
        for i in (0..a).chain(a + 48..old.len()) {
            assert_eq!(old[i].to_bits(), now[i].to_bits());
        }
        assert!(d.readback().is_err());
        plan.replace(&mut d).unwrap();
        assert!(
            d.readback()
                .unwrap()
                .world(256)
                .unwrap()
                .iter()
                .all(|v| v.to_bits() == 0)
        );
        for w in 0..256 {
            assert_eq!(
                bits(d.readback().unwrap().world(w).unwrap()),
                bits(before.world(w).unwrap())
            );
        }
        let old = raw(&d.current);
        let a = 4 + 256 * 65;
        d.write_world_qfrc(256, &vec![7.0; 65]).unwrap();
        let now = raw(&d.current);
        for i in (0..a).chain(a + 65..old.len()) {
            assert_eq!(old[i].to_bits(), now[i].to_bits());
        }
        assert!(d.readback().is_err());
        plan.accumulate(&mut d).unwrap();
        let snapshot = d.readback().unwrap();
        assert!(snapshot.world(256).unwrap().iter().all(|&v| v == 7.0));
        drop(d);
        drop(plan);
        drop(other);
        drop(session);
        assert!(snapshot.world(256).unwrap().iter().all(|&v| v == 7.0));
        let report = serde_json::json!({"batches":batches,"qfrc_slots":slots,"delta_slots":delta_slots,"dyadic_bit_differences":0,"fma_control_bits":format!("{:08x}",fma.to_bits()),"fma_f64_error":(f64::from(fma)-fma_expected).abs(),"phase_ms":start.elapsed().as_millis(),"max_owned_plan_Data_bytes":capacity,"full_U075_NP":false});
        std::fs::write(
            std::env::var_os("MJWARP_APPLIED_FORCE_REPORT").unwrap(),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new U075 guard/index/finite failures must preserve current qfrc before atomic publication"]
    fn gpu_rejects_faults_without_partial_current_mutation_and_replace_ignores_old_nan_payload() {
        let session = TransferSession::new(0).unwrap();
        let mut plan = AppliedForcePlan::new(&session, vec![0, 0], vec![0, 1], vec![1]).unwrap();
        let mut d = plan.create_data(2).unwrap();
        d.write_geometry(
            AppliedForceGeometry::Cdof,
            &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0].repeat(2),
        )
        .unwrap();
        d.write_ft(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0].repeat(2))
            .unwrap();
        d.write_qfrc(&[3.0, 4.0]).unwrap();
        plan.accumulate(&mut d).unwrap();
        assert_eq!(d.readback().unwrap().world(0).unwrap(), &[5.0]);
        for (kind, offset, bad) in [
            (0, 0, 0.0),
            (0, 40, 0.0),
            (0, 4, f32::NAN),
            (1, 0, 0.0),
            (1, 28, 0.0),
            (1, 10, f32::INFINITY),
            (2, 0, 0.0),
            (2, 6, 0.0),
            (2, 4, f32::NAN),
            (3, 0, 0.0),
            (3, 6, 0.0),
            (4, 0, 0.0),
            (4, 6, 0.0),
            (5, 0, 0.0),
            (5, 6, 0.0),
        ] {
            let buffer = match kind {
                0 => &mut d.geometry,
                1 => &mut d.ft,
                2 => &mut d.current,
                3 => &mut d.delta,
                4 => &mut d.staged,
                _ => &mut d.status,
            };
            let mut saved = [0.0];
            buffer.read_range_into(offset, &mut saved).unwrap();
            buffer.write_range(offset, &[bad]).unwrap();
            let before = bits(&raw(&d.current));
            assert!(plan.accumulate(&mut d).is_err());
            assert_eq!(bits(&raw(&d.current)), before);
            assert!(d.readback().is_err());
            let buffer = match kind {
                0 => &mut d.geometry,
                1 => &mut d.ft,
                2 => &mut d.current,
                3 => &mut d.delta,
                4 => &mut d.staged,
                _ => &mut d.status,
            };
            buffer.write_range(offset, &saved).unwrap();
            d.write_qfrc(&[3.0, 4.0]).unwrap();
            plan.accumulate(&mut d).unwrap();
            assert_eq!(d.readback().unwrap().world(0).unwrap(), &[5.0]);
        }
        for (offset, bad) in [
            (0, 0),
            (9, 0),
            (5, 1),
            (5, -1),
            (7, 0),
            (7, 2),
            (8, -1),
            (8, 2),
        ] {
            let mut saved = [0];
            plan.metadata
                .as_buffer()
                .read_range_into(offset, &mut saved)
                .unwrap();
            plan.metadata.write_for_test(offset, bad).unwrap();
            let before = bits(&raw(&d.current));
            assert!(plan.accumulate(&mut d).is_err());
            assert_eq!(bits(&raw(&d.current)), before);
            assert!(d.readback().is_err());
            plan.metadata.restore_for_test().unwrap();
            d.write_qfrc(&[3.0, 4.0]).unwrap();
            plan.accumulate(&mut d).unwrap();
        }
        d.current.write_range(4, &[f32::NAN]).unwrap();
        plan.replace(&mut d).unwrap();
        assert_eq!(d.readback().unwrap().world(0).unwrap(), &[2.0]);
        d.write_geometry(
            AppliedForceGeometry::Cdof,
            &[0.0, 0.0, 0.0, 2.0, 0.0, 0.0].repeat(2),
        )
        .unwrap();
        d.write_ft(
            &[
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                f32::MAX,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ]
            .repeat(2),
        )
        .unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.accumulate(&mut d).is_err());
        assert_eq!(bits(&raw(&d.current)), before);
        assert!(d.readback().is_err());
        d.write_geometry(
            AppliedForceGeometry::Cdof,
            &[0.0, 0.0, 0.0, 0.5, 0.0, 0.0].repeat(2),
        )
        .unwrap();
        d.write_qfrc(&[f32::MAX; 2]).unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.accumulate(&mut d).is_err());
        assert_eq!(bits(&raw(&d.current)), before);
        assert!(d.readback().is_err());
        plan.replace(&mut d).unwrap();
        assert_eq!(d.readback().unwrap().world(0).unwrap(), &[f32::MAX * 0.5]);
        let _ = raw_meta(plan.metadata.as_buffer());
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "M3外力增量类型化包装、同步位型与陈旧证明恢复"]
    fn gpu_m3_delta_wrapper_preserves_bits_and_recovers_stale_metadata() {
        let session = TransferSession::new(0).unwrap();
        let v = reference();
        let mut plan = AppliedForcePlan::new(
            &session,
            ints(&v, "parents"),
            ints(&v, "roots"),
            ints(&v, "dof_bodyid"),
        )
        .unwrap();
        let mut data = plan.create_data(1).unwrap();
        for (field, key) in [
            (AppliedForceGeometry::Xipos, "xipos"),
            (AppliedForceGeometry::SubtreeCom, "subtree_com"),
            (AppliedForceGeometry::Cdof, "cdof"),
        ] {
            data.write_geometry(field, &floats(&v, key)).unwrap();
        }
        data.write_ft(&floats(&v, "ft_force_torque")).unwrap();
        data.write_qfrc(&floats(&v, "initial_qfrc")).unwrap();
        plan.replace(&mut data).unwrap();
        let expected = v["delta_bits"].as_array().unwrap();
        for (actual, expected) in data
            .readback()
            .unwrap()
            .world(0)
            .unwrap()
            .iter()
            .zip(expected)
        {
            assert_eq!(
                actual.to_bits(),
                u32::from_str_radix(expected.as_str().unwrap(), 16).unwrap()
            );
        }

        let before = bits(&raw(&data.current));
        plan.metadata.write_for_test(4, 0).unwrap();
        assert!(plan.replace(&mut data).is_err());
        assert_eq!(bits(&raw(&data.current)), before);
        assert!(data.readback().is_err());
        plan.metadata.restore_for_test().unwrap();
        plan.replace(&mut data).unwrap();
        assert_eq!(bits(&raw(&data.current)), before);
        assert!(data.readback().is_ok());
    }
}
