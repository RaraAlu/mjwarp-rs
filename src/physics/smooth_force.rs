//! 四项广义力的常驻合并辅助。
//! 不累加空间外力，不计算平滑加速度。
#[cfg(feature = "cuda-probe")]
use self::kernels::SmoothForceComputeKernel;
#[cfg(feature = "cuda-probe")]
use self::kernels::SmoothForceValidateKernel;
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::SmoothForceMetadataBuffer;
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

// 子模块：纯移动自原单文件，行为不变。
mod data;
mod kernels;
mod launcher_abi;
mod launcher_meta;
mod launcher_specs;
mod metadata_buffer;
mod plan;
#[cfg(test)]
#[path = "smooth_force/tests/checks.rs"]
mod tests_checks;
#[cfg(test)]
#[path = "smooth_force/tests/gpu_compute.rs"]
mod tests_gpu_compute;
#[cfg(test)]
#[path = "smooth_force/tests/gpu_merge.rs"]
mod tests_gpu_merge;
#[cfg(test)]
#[path = "smooth_force/tests/gpu_validate.rs"]
mod tests_gpu_validate;
#[cfg(test)]
#[path = "smooth_force/tests/support.rs"]
mod tests_support;
#[cfg(test)]
#[path = "smooth_force/tests/wrappers.rs"]
mod tests_wrappers;

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

pub struct SmoothForcePlan {
    input: Arc<SmoothForceInput>,
    options: SmoothForceOptions,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    validate: SmoothForceValidateKernel,
    #[cfg(feature = "cuda-probe")]
    compute: SmoothForceComputeKernel,
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
