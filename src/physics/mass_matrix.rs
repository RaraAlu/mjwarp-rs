//! G02刚体质量矩阵辅助探针。
//! 不实现分解、求解或完整阶段。
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
use crate::model::InertialModelInput;
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{ElemType, SlotBinding};
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

// 子模块：纯移动自原单文件，行为不变。
mod output;
use output::MassMatrixLayout;
pub use output::{MassMatrixOutput, MassMatrixWorld};
mod rigid;
#[cfg(feature = "cuda-probe")]
use rigid::{RigidMassMatrixDims, rigid_mass_matrix_launch_error, rigid_mass_matrix_plan};
#[cfg(feature = "cuda-probe")]
mod buffer;
#[cfg(feature = "cuda-probe")]
use buffer::RigidMassMatrixMetadataBuffer;
#[cfg(test)]
#[path = "mass_matrix/tests/gpu.rs"]
mod tests_gpu;
#[cfg(test)]
#[path = "mass_matrix/tests/host.rs"]
mod tests_host;
#[cfg(test)]
#[path = "mass_matrix/tests/plans.rs"]
mod tests_plans;

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
// Frozen math.inert_vec / wp.dot emit a specific mul/add/fma order; the driver
// JIT decides contraction on that exact sequence. Emit the same unqualified
// PTX operations to reproduce its rounding.
__device__ float mmul(float a,float b) { float r; asm("mul.f32 %0,%1,%2;" : "=f"(r) : "f"(a),"f"(b)); return r; }
__device__ float madd(float a,float b) { float r; asm("add.f32 %0,%1,%2;" : "=f"(r) : "f"(a),"f"(b)); return r; }
__device__ float msub(float a,float b) { float r; asm("sub.f32 %0,%1,%2;" : "=f"(r) : "f"(a),"f"(b)); return r; }
__device__ float mfma(float a,float b,float c) { float r; asm("fma.rn.f32 %0,%1,%2,%3;" : "=f"(r) : "f"(a),"f"(b),"f"(c)); return r; }
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
    const float* i=crb+10*body[d];
    float i0=i[0],i1=i[1],i2=i[2],i3=i[3],i4=i[4],i5=i[5],i6=i[6],i7=i[7],i8=i[8],i9=i[9];
    float v0=cd[6*d],v1=cd[6*d+1],v2=cd[6*d+2],v3=cd[6*d+3],v4=cd[6*d+4],v5=cd[6*d+5];
    float b0=mfma(i7,v5,msub(mfma(i4,v2,mfma(i0,v0,mmul(i3,v1))),mmul(i8,v4)));
    float b1=msub(mfma(i8,v3,mfma(i5,v2,mfma(i3,v0,mmul(i1,v1)))),mmul(i6,v5));
    float b2=mfma(i6,v4,msub(mfma(i2,v2,mfma(i4,v0,mmul(i5,v1))),mmul(i7,v3)));
    float b3=mfma(i9,v3,msub(mmul(i8,v1),mmul(i7,v2)));
    float b4=mfma(i9,v4,msub(mmul(i6,v2),mmul(i8,v0)));
    float b5=mfma(i9,v5,msub(mmul(i7,v0),mmul(i6,v1)));
    for(int j=int(d);j>=0;j=dparent[j]) {
      const float* w=cd+6*j;
      // Frozen _M: chained fma dot (seeded with zero) then armature added last.
      float value=mfma(w[0],b0,0.0f);
      value=mfma(w[1],b1,value); value=mfma(w[2],b2,value);
      value=mfma(w[3],b3,value); value=mfma(w[4],b4,value); value=mfma(w[5],b5,value);
      if(j==int(d)) value=madd(value,p[d]);
      mat[d*nv+j]=value; mat[j*nv+d]=value;
    }
  }
}
"#;
