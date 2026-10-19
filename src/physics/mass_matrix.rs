//! G02刚体质量矩阵辅助探针。
//! 不实现分解、求解或完整阶段。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, InertialModelInput};
use crate::runtime::TransferSession;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

mod resident;
pub use resident::{MassMatrixData, MassMatrixPlan};
mod multiply;
pub use multiply::{
    MassMatrixEnergyData, MassMatrixEnergyPlan, MassMatrixEnergySnapshot, MassMatrixEnergyView,
};
pub use multiply::{MassMatrixMultiplyData, MassMatrixMultiplyPlan, MassMatrixMultiplyView};
pub use multiply::{MassMultiplyData, MassMultiplyFormat, MassMultiplyOutput, MassMultiplyPlan};

/// 同步结果的只读世界视图。
#[derive(Clone, Copy, Debug)]
pub struct MassMatrixWorld<'a> {
    /// 每体十项复合惯量。
    /// 打包顺序沿用cinert。
    pub crb: &'a [f32],
    /// 对称稠密矩阵，按行展开。
    /// 混合坐标不共享同一单位。
    pub matrix: &'a [f32],
}

/// GPU质量矩阵的宿主结果。
/// 结果不承诺正定或可逆。
#[derive(Clone, Debug)]
pub struct MassMatrixOutput {
    layout: MassMatrixLayout,
    values: Vec<f32>,
}
impl MassMatrixOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<MassMatrixWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let values = &self.values[r.start + 4..r.end + 4];
        Ok(MassMatrixWorld {
            crb: &values[..10 * self.nbody()],
            matrix: &values[10 * self.nbody()..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct MassMatrixLayout {
    output: BatchLayout,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    metadata: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}
impl MassMatrixLayout {
    fn new(worlds: usize, nbody: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "mass_matrix_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "mass_matrix_layout",
        };
        let metadata = nv
            .checked_mul(2)
            .and_then(|n| n.checked_add(nbody))
            .ok_or_else(overflow)?;
        let stride = nbody
            .checked_mul(10)
            .and_then(|n| nv.checked_mul(nv).and_then(|m| n.checked_add(m)))
            .ok_or_else(overflow)?;
        if metadata > i32::MAX as usize || stride > i32::MAX as usize {
            return Err(overflow());
        }
        let output = BatchLayout::new(worlds, stride, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (metadata, guarded);
        Ok(Self {
            output,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

#[cfg(any(feature = "cuda-probe", test))]
const RIGID_MASS_MATRIX_METADATA_KIND: ProofKind = ProofKind("rigid_mass_matrix.topology.v1");

#[cfg(any(feature = "cuda-probe", test))]
const RIGID_MASS_MATRIX_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RIGID_MASS_MATRIX_METADATA_KIND,
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
    // 内核回读已写入的复合惯量。
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
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

/// 冻结ABI的四项标量：nb、nv、unused、nw。
#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RigidMassMatrixDims {
    abi: [u32; 4],
}

/// 返回元数据、armature、质心state与output的精确元素数。
#[cfg(any(feature = "cuda-probe", test))]
fn rigid_mass_matrix_lengths(dims: &RigidMassMatrixDims) -> Result<[u64; 4], LauncherError> {
    let [nbody, nv, unused, worlds] = dims.abi;
    if nbody == 0 || unused != 0 || worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("刚体质量矩阵维度无效"));
    }
    let nbody = u64::from(nbody);
    let nv = u64::from(nv);
    let worlds = u64::from(worlds);
    let metadata = nv
        .checked_mul(2)
        .and_then(|value| value.checked_add(nbody))
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_metadata"))?;
    if metadata > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_mass_matrix_metadata"));
    }
    // 内核以u32计算两种世界步长。
    let state_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_state"))?;
    if state_width > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_mass_matrix_state_width"));
    }
    let output_width = nbody
        .checked_mul(10)
        .and_then(|value| {
            nv.checked_mul(nv)
                .and_then(|square| value.checked_add(square))
        })
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_output"))?;
    if output_width > i32::MAX as u64 {
        return Err(LauncherError::Overflow("rigid_mass_matrix_output_width"));
    }
    let state = worlds
        .checked_mul(state_width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_state"))?;
    let output = worlds
        .checked_mul(output_width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("rigid_mass_matrix_output"))?;
    // 零DOF保留真实单元素armature占位。
    Ok([metadata, nv.max(1), state, output])
}

/// 校验内核读取的三段拓扑索引。
#[cfg(any(feature = "cuda-probe", test))]
fn validate_rigid_mass_matrix_metadata(
    values: &[i32],
    dims: RigidMassMatrixDims,
) -> Result<(), LauncherError> {
    let [metadata, ..] = rigid_mass_matrix_lengths(&dims)?;
    if values.len() as u64 != metadata {
        return Err(LauncherError::InvalidMetadata("质量矩阵元数据长度无效"));
    }
    let nbody = dims.abi[0] as usize;
    let nv = dims.abi[1] as usize;
    let (parent, rest) = values.split_at(nbody);
    let (body, dof_parent) = rest.split_at(nv);
    if parent[0] != 0 {
        return Err(LauncherError::InvalidMetadata("质量矩阵根父体无效"));
    }
    for (index, &value) in parent.iter().enumerate().skip(1) {
        if !usize::try_from(value).is_ok_and(|value| value < index) {
            return Err(LauncherError::InvalidMetadata("质量矩阵父体未前序"));
        }
    }
    if body
        .iter()
        .any(|&value| !usize::try_from(value).is_ok_and(|value| value < nbody))
    {
        return Err(LauncherError::InvalidMetadata("质量矩阵自由度刚体越界"));
    }
    for (index, &value) in dof_parent.iter().enumerate() {
        if value != -1 && !usize::try_from(value).is_ok_and(|value| value < index) {
            return Err(LauncherError::InvalidMetadata("质量矩阵父自由度未前序"));
        }
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
struct RigidMassMatrixMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RigidMassMatrixMetadata {
    type Dims = RigidMassMatrixDims;

    const KIND: ProofKind = RIGID_MASS_MATRIX_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rigid_mass_matrix_metadata(values, *dims)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct RigidMassMatrixSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rigid_mass_matrix私有ABI。
unsafe impl KernelAbi for RigidMassMatrixSpec {
    const NAME: &'static str = "rigid_mass_matrix";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RIGID_MASS_MATRIX_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RigidMassMatrixSpec {
    type Dims = RigidMassMatrixDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rigid_mass_matrix_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rigid_mass_matrix_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

/// 在最小跨度之外要求精确跨度。
#[cfg(any(feature = "cuda-probe", test))]
fn rigid_mass_matrix_plan(
    dims: &RigidMassMatrixDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RigidMassMatrixDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rigid_mass_matrix_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rigid_mass_matrix_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RigidMassMatrixSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
fn rigid_mass_matrix_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "mass_matrix_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mass_matrix_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mass_matrix_abi",
        }
        .into(),
    }
}

/// 持有设备拓扑、宿主镜像与写入代际。
#[cfg(feature = "cuda-probe")]
struct RigidMassMatrixMetadataBuffer {
    session: TransferSession,
    buffer: TransferBuffer<i32>,
    #[cfg(test)]
    values: Vec<i32>,
    #[cfg(test)]
    revision: MetadataRevision,
    checked: Checked<RigidMassMatrixMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl RigidMassMatrixMetadataBuffer {
    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        abi: [u32; 4],
    ) -> Result<Self, TransferError> {
        let dims = RigidMassMatrixDims { abi };
        validate_rigid_mass_matrix_metadata(&values, dims)
            .map_err(rigid_mass_matrix_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<RigidMassMatrixMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(rigid_mass_matrix_launch_error)?;
        #[cfg(not(test))]
        let _ = (values, revision);
        Ok(Self {
            session: session.clone(),
            buffer,
            #[cfg(test)]
            values,
            #[cfg(test)]
            revision,
            checked,
        })
    }

    fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
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
            Checked::<RigidMassMatrixMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                *self.checked.dims(),
                self.revision.clone(),
            )
        }
        .map_err(rigid_mass_matrix_launch_error)?;
        Ok(())
    }
}

/// 预检通过的发射维度。
#[cfg(feature = "cuda-probe")]
struct RigidMassMatrixLaunch {
    dims: RigidMassMatrixDims,
}

#[cfg(feature = "cuda-probe")]
struct RigidMassMatrixKernel {
    session: TransferSession,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RigidMassMatrixKernel {
    fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, MASS_MATRIX_CUDA, "rigid_mass_matrix")?,
        })
    }

    fn bindings(
        metadata: &RigidMassMatrixMetadataBuffer,
        armature: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 4], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: armature.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: state.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    fn preflight(
        &self,
        session: &TransferSession,
        metadata: &RigidMassMatrixMetadataBuffer,
        armature: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<RigidMassMatrixLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let dims = *metadata.checked.dims();
        let bindings = Self::bindings(metadata, armature, state, output)?;
        rigid_mass_matrix_plan(&dims, &bindings, &[metadata.checked.binding(0)])
            .map_err(rigid_mass_matrix_launch_error)?;
        Ok(RigidMassMatrixLaunch { dims })
    }

    fn launch(
        &self,
        launch: &RigidMassMatrixLaunch,
        metadata: &RigidMassMatrixMetadataBuffer,
        armature: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        if !self.session.same_session(&metadata.session) {
            return Err(TransferError::SessionMismatch);
        }
        let bindings = Self::bindings(metadata, armature, state, output)?;
        rigid_mass_matrix_plan(&launch.dims, &bindings, &[metadata.checked.binding(0)])
            .map_err(rigid_mass_matrix_launch_error)?;
        // SAFETY: 类型计划核对拓扑前序、精确跨度、会话、别名和网格。
        // 每线程只写本世界输出；同步适配器等待完成。
        unsafe {
            self.kernel.launch(
                metadata.as_buffer(),
                armature,
                state,
                output,
                launch.dims.abi,
            )
        }
    }
}

/// G02刚体子集的同步GPU探针。
/// GPU复用质心阶段设备结果。
/// GPU累加复合惯量并填充矩阵。
/// 自由度armature只加到对角线。
/// 参数仅共享一份模型。
/// 支持静态祖先与零自由度。
/// 不叠加肌腱或执行器惯量。
/// 不处理休眠、mocap或柔性体。
/// 不提供稀疏布局、分解或求解。
/// 不替代完整crb与G02入口。
/// 需要cuda-probe及NVRTC。
pub fn probe_mass_matrix(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<MassMatrixOutput, TransferError> {
    #[cfg(not(feature = "cuda-probe"))]
    {
        let (fk, com) = super::check_com_position(model, worlds, qpos)?;
        let k = model.kinematics();
        let layout = MassMatrixLayout::new(worlds, k.nbody(), k.nv())?;
        let _ = (session, fk, com, layout);
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        mass_matrix_device(session, model, worlds, qpos).map(|(_, result)| result)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn mass_matrix_device(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<(crate::runtime::TransferBuffer<f32>, MassMatrixOutput), TransferError> {
    let (fk, com) = super::check_com_position(model, worlds, qpos)?;
    let k = model.kinematics();
    let layout = MassMatrixLayout::new(worlds, k.nbody(), k.nv())?;
    let (state, checkpoint) = super::com_position_device(session, model, fk, com, qpos)?;
    drop(checkpoint);
    let mut metadata = crate::runtime::host_staging::<i32>(layout.metadata)?;
    let mut cursor = 0;
    for field in [
        &k.fields().body_parentid,
        &model.fields().dof_bodyid,
        &model.fields().dof_parentid,
    ] {
        metadata[cursor..cursor + field.len()].copy_from_slice(field);
        cursor += field.len();
    }
    let abi = [
        u32::try_from(k.nbody()).map_err(|_| InputError::Overflow {
            field: "mass_matrix_nbody",
        })?,
        u32::try_from(k.nv()).map_err(|_| InputError::Overflow {
            field: "mass_matrix_nv",
        })?,
        0,
        u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "mass_matrix_worlds",
        })?,
    ];
    let metadata = RigidMassMatrixMetadataBuffer::upload(session, metadata, abi)?;
    // A real pointer preserves the fixed ABI for the zero-DOF case.
    let armature = &model.fields().dof_armature;
    let parameters = session.upload(if armature.is_empty() {
        &[0.0]
    } else {
        armature
    })?;
    let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
    values.fill(-131072.0);
    values[4..layout.guarded - 4].fill(f32::NAN);
    let mut output = session.upload(&values)?;
    let kernel = RigidMassMatrixKernel::compile(session)?;
    let launch = kernel.preflight(session, &metadata, &parameters, &state, &output)?;
    kernel.launch(&launch, &metadata, &parameters, &state, &mut output)?;
    output.read_range_into(0, &mut values)?;
    super::check_device_values(&values, "mass_matrix_output")?;
    Ok((output, MassMatrixOutput { layout, values }))
}

// Adapted from frozen smooth.py crb and math.py inert_vec at
// 71da24d956378a87a703b6e1442b13aec0c4ac29.
// Copyright 2025 The Newton Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[cfg(feature = "cuda-probe")]
const MASS_MATRIX_CUDA: &str = r#"
extern "C" __global__ void rigid_mass_matrix(const int* m,const float* p,
    const float* state,float* output,unsigned nb,unsigned nv,unsigned unused,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=nw) return;
  const int* parent=m; const int* body=m+nb; const int* dparent=body+nv;
  const float* s=state+4+(unsigned long long)w*(14*nb+6*nv);
  const float* ci=s+4*nb; const float* cd=ci+10*nb;
  float* crb=output+4+(unsigned long long)w*(10*nb+nv*nv);
  float* mat=crb+10*nb;
  for(unsigned a=0;a<10*nb;a++) crb[a]=ci[a];
  // Root subtrees use different COM frames. Never accumulate into world.
  for(int b=int(nb)-1;b>0;b--) if(parent[b]>0)
    for(int a=0;a<10;a++) crb[10*parent[b]+a]+=crb[10*b+a];
  for(unsigned a=0;a<nv*nv;a++) mat[a]=0.0f;
  for(unsigned d=0;d<nv;d++) {
    const float* i=crb+10*body[d]; const float* v=cd+6*d;
    float buf[6]={
      i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5],
      i[3]*v[0]+i[1]*v[1]+i[5]*v[2]+i[8]*v[3]-i[6]*v[5],
      i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4],
      i[8]*v[1]-i[7]*v[2]+i[9]*v[3],
      i[6]*v[2]-i[8]*v[0]+i[9]*v[4],
      i[7]*v[0]-i[6]*v[1]+i[9]*v[5]};
    for(int j=int(d);j>=0;j=dparent[j]) {
      // Frozen _M finishes wp.dot before adding the diagonal armature.
      // Seeding the dot with armature changes the f32 rounding sequence.
      float value=cd[6*j]*buf[0];
      for(int a=1;a<6;a++) value+=cd[6*j+a]*buf[a];
      if(j==int(d)) value=__fadd_rn(value,p[d]);
      mat[d*nv+j]=value; mat[j*nv+d]=value;
    }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_mass_matrix_dimensions_square_capacity_and_kernel_indices() {
        for dims in [
            (0, 1, 0),
            (1, 0, 0),
            (usize::MAX, 1, 0),
            (1, usize::MAX, 0),
            (1, 1, usize::MAX),
            (1, 1, 46341),
            (1, i32::MAX as usize, 1),
        ] {
            assert!(MassMatrixLayout::new(dims.0, dims.1, dims.2).is_err());
        }
        assert!(MassMatrixLayout::new(1, 1, 46340).is_ok());
    }
    #[test]
    fn mass_matrix_views_separate_worlds_inertias_and_dense_rows() {
        let layout = MassMatrixLayout::new(2, 2, 3).unwrap();
        let out = MassMatrixOutput {
            layout,
            values: (0..66).map(|n| n as f32).collect(),
        };
        assert_eq!(out.worlds(), 2);
        assert_eq!(out.nbody(), 2);
        assert_eq!(out.nv(), 3);
        let w = out.world(1).unwrap();
        assert_eq!(w.crb.len(), 20);
        assert_eq!(w.crb[0], 33.0);
        assert_eq!(
            w.matrix,
            &[53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0]
        );
        assert!(out.world(2).is_err());
        let out = MassMatrixOutput {
            layout: MassMatrixLayout::new(1, 1, 0).unwrap(),
            values: vec![0.0; 18],
        };
        assert!(out.world(0).unwrap().matrix.is_empty());
    }
    #[test]
    fn preflight_keeps_existing_com_and_state_rejections() {
        let m = super::super::tests::test_model(Some(3));
        assert!(super::super::check_com_position(&m, 2, &[0.0]).is_err());
        assert!(super::super::check_com_position(&m, 2, &[0.0, f32::NAN]).is_err());
        assert!(super::super::check_com_position(&m, 2, &[0.0, 0.5]).is_ok());
    }

    #[test]
    fn rigid_mass_matrix_plan_checks_spans_aliases_topology_and_overflow() {
        use crate::runtime::launcher::DeviceSpan;

        let bind = |slot, context, address, bytes, elem| SlotBinding {
            slot,
            span: DeviceSpan::new(context, address, bytes).unwrap(),
            elem,
        };
        let values = vec![0, 0, 1, 1, 2, -1, 0];
        let dims = RigidMassMatrixDims { abi: [3, 2, 0, 2] };
        assert_eq!(rigid_mass_matrix_lengths(&dims).unwrap(), [7, 2, 116, 76]);
        let revision = MetadataRevision::new();
        let metadata_span = DeviceSpan::new(7, 0x1000, 28).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof = unsafe {
            Checked::<RigidMassMatrixMetadata>::assume_uploaded(
                metadata_span,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            bind(0, 7, 0x1000, 28, ElemType::I32),
            bind(1, 7, 0x2000, 8, ElemType::F32),
            bind(2, 7, 0x3000, 464, ElemType::F32),
            bind(3, 7, 0x4000, 304, ElemType::F32),
        ];
        let launch = rigid_mass_matrix_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
        assert_eq!(launch.grid(), (1, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));
        assert_eq!(launch.shared_bytes(), 0);
        assert!(matches!(
            rigid_mass_matrix_plan(&dims, &bindings, &[]),
            Err(LauncherError::MissingMetadataProof { slot: 0 })
        ));

        let addresses = [0x1000, 0x2000, 0x3000, 0x4000];
        let bytes = [28, 8, 464, 304];
        for slot in 0..4 {
            let mut short = bindings;
            short[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] - 4).unwrap();
            assert!(matches!(
                rigid_mass_matrix_plan(&dims, &short, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));
            let mut long = bindings;
            long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
            assert!(matches!(
                rigid_mass_matrix_plan(&dims, &long, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));
        }

        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x3000, 464).unwrap();
        assert!(matches!(
            rigid_mass_matrix_plan(&dims, &foreign, &[proof.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));
        let mut alias = bindings;
        alias[3].span = DeviceSpan::new(7, 0x3000, 304).unwrap();
        assert!(matches!(
            rigid_mass_matrix_plan(&dims, &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 2,
                second: 3
            })
        ));
        // 只读槽之间允许重叠。
        let mut shared = bindings;
        shared[1].span = DeviceSpan::new(7, 0x3000, 8).unwrap();
        assert!(rigid_mass_matrix_plan(&dims, &shared, &[proof.binding(0)]).is_ok());
        let other = RigidMassMatrixDims { abi: [3, 2, 0, 1] };
        assert!(matches!(
            rigid_mass_matrix_plan(
                &other,
                &[
                    bindings[0],
                    bindings[1],
                    bind(2, 7, 0x3000, 248, ElemType::F32),
                    bind(3, 7, 0x4000, 168, ElemType::F32),
                ],
                &[proof.binding(0)],
            ),
            Err(LauncherError::MetadataMismatch(_))
        ));
        revision.invalidate();
        assert!(matches!(
            rigid_mass_matrix_plan(&dims, &bindings, &[proof.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));

        for invalid in [
            vec![0, 0, 1, 1, 2, -1],
            vec![0, 0, 1, 1, 2, -1, 0, 0],
            vec![1, 0, 1, 1, 2, -1, 0],
            vec![0, -1, 1, 1, 2, -1, 0],
            vec![0, 0, 2, 1, 2, -1, 0],
            vec![0, 0, 1, 3, 2, -1, 0],
            vec![0, 0, 1, -1, 2, -1, 0],
            vec![0, 0, 1, 1, 2, 0, 0],
            vec![0, 0, 1, 1, 2, -1, 1],
            vec![0, 0, 1, 1, 2, -1, -2],
        ] {
            assert!(validate_rigid_mass_matrix_metadata(&invalid, dims).is_err());
        }
        assert!(validate_rigid_mass_matrix_metadata(&[0, 0, 1, 1, 2, -1, -1], dims).is_ok());
        let zero = RigidMassMatrixDims { abi: [2, 0, 0, 2] };
        assert_eq!(rigid_mass_matrix_lengths(&zero).unwrap(), [2, 1, 64, 48]);
        assert!(validate_rigid_mass_matrix_metadata(&[0, 0], zero).is_ok());

        for abi in [
            [0, 0, 0, 1],
            [1, 0, 1, 1],
            [1, 0, 0, 0],
            [1, 0, 0, u32::MAX - 254],
        ] {
            assert!(matches!(
                rigid_mass_matrix_lengths(&RigidMassMatrixDims { abi }),
                Err(LauncherError::InvalidArray(_))
            ));
        }
        assert!(
            rigid_mass_matrix_lengths(&RigidMassMatrixDims {
                abi: [1, 0, 0, u32::MAX - 255]
            })
            .is_ok()
        );
    }

    #[test]
    fn rigid_mass_matrix_plan_preserves_u32_strides_and_byte_bounds() {
        let state_limit = RigidMassMatrixDims {
            abi: [i32::MAX as u32 / 14, 0, 0, 1],
        };
        assert!(rigid_mass_matrix_lengths(&state_limit).is_ok());
        assert!(matches!(
            rigid_mass_matrix_lengths(&RigidMassMatrixDims {
                abi: [i32::MAX as u32 / 14 + 1, 0, 0, 1],
            }),
            Err(LauncherError::Overflow("rigid_mass_matrix_state_width"))
        ));
        assert!(
            rigid_mass_matrix_lengths(&RigidMassMatrixDims {
                abi: [1, 46_340, 0, 1]
            })
            .is_ok()
        );
        assert!(matches!(
            rigid_mass_matrix_lengths(&RigidMassMatrixDims {
                abi: [1, 46_341, 0, 1],
            }),
            Err(LauncherError::Overflow("rigid_mass_matrix_output_width"))
        ));
        assert!(matches!(
            rigid_mass_matrix_lengths(&RigidMassMatrixDims {
                abi: [1, i32::MAX as u32, 0, 1],
            }),
            Err(LauncherError::Overflow("rigid_mass_matrix_metadata"))
        ));

        let byte_overflow = RigidMassMatrixDims {
            abi: [i32::MAX as u32 / 14, 0, 0, u32::MAX - 255],
        };
        assert!(rigid_mass_matrix_lengths(&byte_overflow).is_ok());
        let bind = |slot, address, bytes, elem| SlotBinding {
            slot,
            span: crate::runtime::launcher::DeviceSpan::new(7, address, bytes).unwrap(),
            elem,
        };
        // 前两槽精确匹配，state字节数溢出u64。
        assert!(matches!(
            rigid_mass_matrix_plan(
                &byte_overflow,
                &[
                    bind(0, 0x1000, 4 * (i32::MAX as u64 / 14), ElemType::I32),
                    bind(1, 0x4000_0000, 4, ElemType::F32),
                    bind(2, 0x5000_0000, 4, ElemType::F32),
                    bind(3, 0x6000_0000, 4, ElemType::F32),
                ],
                &[],
            ),
            Err(LauncherError::Overflow("rigid_mass_matrix_bytes"))
        ));
    }
    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn device_adds_armature_after_the_completed_spatial_dot() {
        let session = TransferSession::new(0).unwrap();
        let metadata = session.upload(&[0_i32, 0, -1]).unwrap();
        let parameters = session.upload(&[1.0_f32]).unwrap();
        // 用二进制幂隔离舍入，不构造物理惯量。
        let mut values = vec![-131072.0_f32; 28];
        values[4..24].fill(0.0);
        values[8..11].copy_from_slice(&[134217728.0, -134217728.0, 8.0]);
        values[18..21].fill(1.0);
        let state = session.upload(&values).unwrap();
        let mut values = vec![-131072.0_f32; 19];
        values[4..15].fill(f32::NAN);
        let mut output = session.upload(&values).unwrap();
        let kernel = crate::runtime::SynchronousKernel::compile(
            &session,
            MASS_MATRIX_CUDA,
            "rigid_mass_matrix",
        )
        .unwrap();
        // SAFETY: One world uses 20 COM scalars and 11 result scalars.
        // All buffers share this session; guards bound both layouts.
        unsafe {
            kernel
                .launch(&metadata, &parameters, &state, &mut output, [1, 1, 0, 1])
                .unwrap();
        }
        output.read_range_into(0, &mut values).unwrap();
        super::super::check_device_values(&values, "mass_matrix_output").unwrap();
        assert_eq!(values[14], 9.0);
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn runs_rigid_mass_matrix_wrapper_paths_and_recovers() {
        fn captured<T>(label: &str, run: impl FnOnce() -> T) -> T {
            let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
            if let Some(root) = capture.as_ref() {
                let child = std::path::PathBuf::from(root).join(label);
                assert!(!child.exists(), "捕获目录必须为空：{}", child.display());
                // SAFETY: 本测试单线程运行，并在本次编译后恢复环境。
                unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &child) };
            }
            let result = run();
            if let Some(root) = capture.as_ref() {
                // SAFETY: 本测试单线程运行，并恢复调用方环境。
                unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
            } else {
                // SAFETY: 本测试单线程运行，调用前也没有该变量。
                unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
            }
            result
        }
        fn bits(values: &[f32]) -> Vec<u32> {
            values.iter().map(|value| value.to_bits()).collect()
        }
        fn read(buffer: &TransferBuffer<f32>) -> Vec<f32> {
            let mut values = vec![0.0; buffer.len()];
            buffer.read_range_into(0, &mut values).unwrap();
            values
        }
        fn guarded(len: usize) -> Vec<f32> {
            let mut values = vec![f32::NAN; len];
            values[..4].fill(-131072.0);
            values[len - 4..].fill(-131072.0);
            values
        }

        let session = TransferSession::new(0).unwrap();
        // 合成拓扑覆盖分叉、静态祖先与多自由度刚体。
        let worlds = 257usize;
        let (nbody, nv) = (5usize, 6usize);
        let topology = vec![0, 0, 1, 1, 3, 1, 1, 2, 3, 4, 4, -1, 0, 1, 0, 3, 4];
        let abi = [nbody as u32, nv as u32, 0, worlds as u32];
        let mut metadata =
            RigidMassMatrixMetadataBuffer::upload(&session, topology.clone(), abi).unwrap();
        let armature = (0..nv)
            .map(|dof| 0.125 * (dof + 1) as f32)
            .collect::<Vec<_>>();
        let armature_buffer = session.upload(&armature).unwrap();
        let state_width = 14 * nbody + 6 * nv;
        let mut state = vec![-131072.0_f32; 8 + worlds * state_width];
        for (index, value) in state[4..4 + worlds * state_width].iter_mut().enumerate() {
            *value = ((index * 37 + 11) % 101) as f32 / 64.0 - 0.75;
        }
        let state_buffer = session.upload(&state).unwrap();
        let initial = guarded(8 + worlds * (10 * nbody + nv * nv));
        let mut output = session.upload(&initial).unwrap();
        let raw_kernel = captured("raw-baseline", || {
            SynchronousKernel::compile(&session, MASS_MATRIX_CUDA, "rigid_mass_matrix")
        })
        .unwrap();
        // SAFETY: 迁移前原始调用；拓扑已验证，缓冲精确匹配冻结ABI。
        unsafe {
            raw_kernel
                .launch(
                    metadata.as_buffer(),
                    &armature_buffer,
                    &state_buffer,
                    &mut output,
                    abi,
                )
                .unwrap();
        }
        let raw = read(&output);
        super::super::check_device_values(&raw, "mass_matrix_output").unwrap();
        output.write_range(0, &initial).unwrap();
        let kernel = captured("typed-main", || RigidMassMatrixKernel::compile(&session)).unwrap();
        let launch = kernel
            .preflight(
                &session,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &output,
            )
            .unwrap();
        kernel
            .launch(
                &launch,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &mut output,
            )
            .unwrap();
        let typed = read(&output);
        assert_eq!(bits(&raw), bits(&typed));

        // 拒绝发生在写入之前。
        let short = session.upload(&armature[..nv - 1]).unwrap();
        assert!(
            kernel
                .preflight(&session, &metadata, &short, &state_buffer, &output)
                .is_err()
        );
        let short_state = session.upload(&state[..state.len() - 1]).unwrap();
        assert!(
            kernel
                .preflight(&session, &metadata, &armature_buffer, &short_state, &output)
                .is_err()
        );
        let mut long_output = session.upload(&guarded(initial.len() + 1)).unwrap();
        assert!(
            kernel
                .launch(
                    &launch,
                    &metadata,
                    &armature_buffer,
                    &state_buffer,
                    &mut long_output,
                )
                .is_err()
        );
        assert_eq!(bits(&read(&long_output)), bits(&guarded(initial.len() + 1)));
        assert!(matches!(
            RigidMassMatrixMetadataBuffer::upload(
                &session,
                vec![0, 0, 2, 1, 3, 1, 1, 2, 3, 4, 4, -1, 0, 1, 0, 3, 4],
                abi
            ),
            Err(TransferError::Input(InputError::InvalidTopology { .. }))
        ));
        assert!(matches!(
            RigidMassMatrixMetadataBuffer::upload(
                &session,
                topology.clone(),
                [nbody as u32, nv as u32, 1, worlds as u32]
            ),
            Err(TransferError::Input(InputError::InvalidDimension { .. }))
        ));
        assert_eq!(bits(&read(&output)), bits(&typed));

        let stale = kernel
            .preflight(
                &session,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &output,
            )
            .unwrap();
        metadata.overwrite_for_test(1, 7).unwrap();
        assert!(
            kernel
                .launch(
                    &stale,
                    &metadata,
                    &armature_buffer,
                    &state_buffer,
                    &mut output,
                )
                .is_err()
        );
        assert!(
            kernel
                .preflight(
                    &session,
                    &metadata,
                    &armature_buffer,
                    &state_buffer,
                    &output
                )
                .is_err()
        );
        assert_eq!(bits(&read(&output)), bits(&typed));
        metadata.restore_for_test().unwrap();
        output.write_range(0, &initial).unwrap();
        let recovered = kernel
            .preflight(
                &session,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &output,
            )
            .unwrap();
        kernel
            .launch(
                &recovered,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &mut output,
            )
            .unwrap();
        assert_eq!(bits(&read(&output)), bits(&typed));

        let foreign_session = TransferSession::new(0).unwrap();
        let foreign = captured("foreign-session", || {
            RigidMassMatrixKernel::compile(&foreign_session)
        })
        .unwrap();
        assert!(matches!(
            foreign.preflight(
                &session,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &output
            ),
            Err(TransferError::SessionMismatch)
        ));
        let foreign_state = foreign_session.upload(&state).unwrap();
        assert!(matches!(
            kernel.preflight(
                &session,
                &metadata,
                &armature_buffer,
                &foreign_state,
                &output
            ),
            Err(TransferError::SessionMismatch)
        ));
        assert_eq!(bits(&read(&output)), bits(&typed));

        // 零自由度保留真实armature占位。
        let zero_abi = [2, 0, 0, 2];
        let zero_metadata =
            RigidMassMatrixMetadataBuffer::upload(&session, vec![0, 0], zero_abi).unwrap();
        let zero_armature = session.upload(&[0.0_f32]).unwrap();
        let mut zero_state = vec![-131072.0_f32; 64];
        for (index, value) in zero_state[4..60].iter_mut().enumerate() {
            *value = index as f32 / 8.0;
        }
        let zero_state = session.upload(&zero_state).unwrap();
        let zero_initial = guarded(48);
        let mut zero_output = session.upload(&zero_initial).unwrap();
        // SAFETY: 零自由度原始调用；缓冲精确匹配冻结ABI。
        unsafe {
            raw_kernel
                .launch(
                    zero_metadata.as_buffer(),
                    &zero_armature,
                    &zero_state,
                    &mut zero_output,
                    zero_abi,
                )
                .unwrap();
        }
        let zero_raw = read(&zero_output);
        zero_output.write_range(0, &zero_initial).unwrap();
        let zero_launch = kernel
            .preflight(
                &session,
                &zero_metadata,
                &zero_armature,
                &zero_state,
                &zero_output,
            )
            .unwrap();
        kernel
            .launch(
                &zero_launch,
                &zero_metadata,
                &zero_armature,
                &zero_state,
                &mut zero_output,
            )
            .unwrap();
        assert_eq!(bits(&zero_raw), bits(&read(&zero_output)));
        super::super::check_device_values(&zero_raw, "mass_matrix_output").unwrap();

        // 产品入口与同输入原始调用逐位一致。
        let model = super::super::tests::test_model(Some(0));
        let qpos = [0.25, -0.5, 0.75, 0.5, 0.5, -0.5, 0.5].repeat(3);
        let (_, product) = captured("product-main", || {
            mass_matrix_device(&session, &model, 3, &qpos)
        })
        .unwrap();
        let (fk, com) = super::super::check_com_position(&model, 3, &qpos).unwrap();
        let (com_state, _) =
            super::super::com_position_device(&session, &model, fk, com, &qpos).unwrap();
        let k = model.kinematics();
        let mut packed = k.fields().body_parentid.clone();
        packed.extend(&model.fields().dof_bodyid);
        packed.extend(&model.fields().dof_parentid);
        let product_metadata = session.upload(&packed).unwrap();
        let product_armature = session.upload(&model.fields().dof_armature).unwrap();
        let product_initial = guarded(product.values.len());
        let mut product_output = session.upload(&product_initial).unwrap();
        // SAFETY: 产品模型已通过拓扑校验；缓冲精确匹配冻结ABI。
        unsafe {
            raw_kernel
                .launch(
                    &product_metadata,
                    &product_armature,
                    &com_state,
                    &mut product_output,
                    [k.nbody() as u32, k.nv() as u32, 0, 3],
                )
                .unwrap();
        }
        assert_eq!(bits(&read(&product_output)), bits(&product.values));
    }
}
