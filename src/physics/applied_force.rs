//! 世界空间外力经雅可比映射到广义力。
//! 调用方提供几何字段；本入口不执行位置生产者。
#[cfg(feature = "cuda-probe")]
use self::kernels::AppliedForceDeltaKernel;
#[cfg(feature = "cuda-probe")]
use self::kernels::AppliedForcePublishKernel;
#[cfg(feature = "cuda-probe")]
use self::kernels::AppliedForceValidateKernel;
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::AppliedForceMetadataBuffer;
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

// 子模块：纯移动自原单文件，行为不变。
mod data;
mod kernels;
mod launcher_abi;
mod launcher_meta;
mod launcher_specs;
mod metadata_buffer;
mod plan;
#[cfg(test)]
#[path = "applied_force/tests/checks.rs"]
mod tests_checks;
#[cfg(test)]
#[path = "applied_force/tests/gpu.rs"]
mod tests_gpu;
#[cfg(test)]
#[path = "applied_force/tests/support.rs"]
mod tests_support;
#[cfg(test)]
#[path = "applied_force/tests/wrappers.rs"]
mod tests_wrappers;

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

pub struct AppliedForcePlan {
    input: Arc<Topology>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: AppliedForceMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validate: AppliedForceValidateKernel,
    #[cfg(feature = "cuda-probe")]
    delta: AppliedForceDeltaKernel,
    #[cfg(feature = "cuda-probe")]
    publish: AppliedForcePublishKernel,
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
