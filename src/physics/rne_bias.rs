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

// 子模块：纯移动自原单文件，行为不变。
mod buffer;
mod data;
mod metadata;
mod plan;
mod snapshot;
mod spec;
#[cfg(feature = "cuda-probe")]
use buffer::RneBiasMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use spec::{RneBiasMetadata, rne_bias_launch_error, rne_bias_plan};
#[cfg(test)]
#[path = "rne_bias/tests/gpu.rs"]
mod tests_gpu;
#[cfg(test)]
#[path = "rne_bias/tests/host.rs"]
mod tests_host;
#[cfg(test)]
#[path = "rne_bias/tests/plans.rs"]
mod tests_plans;
#[cfg(test)]
#[path = "rne_bias/tests/support.rs"]
mod tests_support;

#[cfg(feature = "cuda-probe")]
struct RneBiasLaunch {
    proof: Checked<RneBiasMetadata>,
}

#[cfg(feature = "cuda-probe")]
struct RneBiasInputs<'a> {
    metadata: &'a RneBiasMetadataBuffer,
    qvel: &'a TransferBuffer<f32>,
    com: &'a TransferBuffer<f32>,
    velocity: &'a TransferBuffer<f32>,
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
        inputs: &RneBiasInputs<'_>,
        output: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 5], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: inputs.metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: inputs.qvel.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: inputs.com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: inputs.velocity.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    fn preflight(
        &self,
        session: &TransferSession,
        inputs: &RneBiasInputs<'_>,
        output: &TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<RneBiasLaunch, TransferError> {
        if !self.session.same_session(session) || !inputs.metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = inputs.metadata.proof(worlds)?;
        let bindings = Self::bindings(inputs, output)?;
        rne_bias_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_bias_launch_error)?;
        Ok(RneBiasLaunch { proof })
    }

    fn launch(
        &self,
        launch: &RneBiasLaunch,
        inputs: &RneBiasInputs<'_>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(inputs, output)?;
        rne_bias_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_bias_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel.launch_flex_edges(
                inputs.metadata.as_buffer(),
                inputs.qvel,
                inputs.com,
                inputs.velocity,
                output,
                dims.worlds,
            )
        }
    }
}

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
