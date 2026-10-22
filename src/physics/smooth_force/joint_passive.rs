//! DOF阻尼与四类关节弹簧生产者；不代表完整passive。
use super::{
    SmoothForceData, SmoothForceField, SmoothForceOptions, SmoothForceOutput, SmoothForcePlan,
};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::data::SourceState;
use crate::model::topology::JointType;
use crate::model::{BatchLayout, KinematicModelInput, ParameterBatch, SmoothForceInput};
#[cfg(feature = "cuda-probe")]
use crate::runtime::TransferBuffer;
use crate::runtime::TransferSession;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JointPassiveOptions {
    pub disable_spring: bool,
    pub disable_damper: bool,
}
#[derive(Clone, Debug)]
pub struct JointPassiveParameters {
    pub qpos_spring: ParameterBatch,
    pub stiffness: ParameterBatch,
    pub stiffness_poly: ParameterBatch,
    pub damping: ParameterBatch,
    pub damping_poly: ParameterBatch,
}
impl JointPassiveParameters {
    fn fields(&self) -> [&ParameterBatch; 5] {
        [
            &self.qpos_spring,
            &self.stiffness,
            &self.stiffness_poly,
            &self.damping,
            &self.damping_poly,
        ]
    }
}
struct Input {
    model: KinematicModelInput,
    params: JointPassiveParameters,
    meta: Vec<i32>,
    offsets: [usize; 5],
    param_len: usize,
    body: Vec<i32>,
}
fn input(
    model: KinematicModelInput,
    params: JointPassiveParameters,
    _options: JointPassiveOptions,
) -> Result<Input, InputError> {
    let (nq, nv, nj) = (model.nq(), model.nv(), model.njnt());
    if nv == 0 || nj == 0 {
        return Err(InputError::InvalidDimension {
            field: "joint_passive_model",
        });
    }
    let widths = [nq, nj, 2 * nj, nv, 2 * nv];
    let mut offsets = [0; 5];
    let mut plen = 0usize;
    for (i, p) in params.fields().iter().enumerate() {
        if p.row_elements() != widths[i] {
            return Err(InputError::LengthMismatch {
                field: "joint_passive_parameter",
                expected: widths[i],
                actual: p.row_elements(),
            });
        }
        offsets[i] = plen;
        plen = plen
            .checked_add(p.values().len())
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "joint_passive_parameters",
            })?;
    }
    let mut kinds = vec![0; nv];
    let mut qadr = vec![0; nv];
    let mut joint = vec![0; nv];
    let mut local = vec![0; nv];
    let mut body = vec![0; nv];
    for j in 0..nj {
        let kind = JointType::try_from(model.fields().jnt_type[j])?;
        let d = model.fields().jnt_dofadr[j] as usize;
        for k in 0..kind.dof_width() {
            kinds[d + k] = kind as i32;
            qadr[d + k] = model.fields().jnt_qposadr[j];
            joint[d + k] = j as i32;
            local[d + k] = k as i32;
            body[d + k] = model.fields().jnt_bodyid[j];
        }
    }
    nv.checked_mul(4)
        .and_then(|v| v.checked_add(8))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "joint_passive_metadata",
        })?;
    let mut meta = vec![-131072; 4];
    meta.extend(kinds);
    meta.extend(qadr);
    meta.extend(joint);
    meta.extend(local);
    meta.extend([-131072; 4]);
    Ok(Input {
        model,
        params,
        meta,
        offsets,
        param_len: plen,
        body,
    })
}
#[derive(Clone, Copy)]
struct Layout {
    state: BatchLayout,
    vector: BatchLayout,
    // 默认配置无设备路径；仅cuda-probe读取。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    parts: BatchLayout,
}
fn layout(nq: usize, nv: usize, worlds: usize) -> Result<Layout, InputError> {
    if worlds == 0
        || nv == 0
        || worlds
            .checked_mul(nv)
            .is_none_or(|v| v > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "joint_passive_work",
        });
    }
    let state = BatchLayout::new(
        worlds,
        nq.checked_add(nv).ok_or(InputError::Overflow {
            field: "joint_passive_state",
        })?,
        4,
    )?;
    let vector = BatchLayout::new(worlds, nv, 4)?;
    let parts = BatchLayout::new(worlds, 2 * nv, 4)?;
    for n in [state.total_elements(), parts.total_elements(), worlds] {
        n.checked_add(8)
            .and_then(|v| v.checked_mul(4))
            .ok_or(InputError::Overflow {
                field: "joint_passive_guards",
            })?;
    }
    Ok(Layout {
        state,
        vector,
        parts,
    })
}
pub struct JointPassivePlan {
    input: Arc<Input>,
    options: JointPassiveOptions,
    smooth: SmoothForcePlan,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: launcher_abi::JointPassiveMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: kernels::JointPassiveKernels,
}
pub struct JointPassiveData {
    input: Arc<Input>,
    layout: Layout,
    ready: bool,
    smooth: SmoothForceData,
    /// A1：qpos/qvel副本的常驻来源快照；旧写入入口清空声明。
    source: Option<SourceState>,
    #[cfg(feature = "cuda-probe")]
    state: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct JointPassiveOutput {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl JointPassiveOutput {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn spring(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
    pub fn damper(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        let off = 4 + self.layout.total_elements();
        Ok(&self.values[off + r.start..off + r.end])
    }
}

// 子模块：纯移动自原单文件，行为不变。
mod data;
#[cfg(feature = "cuda-probe")]
#[path = "joint_passive/kernels.rs"]
mod kernels;
#[cfg(feature = "cuda-probe")]
#[path = "joint_passive/launcher_abi.rs"]
mod launcher_abi;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "joint_passive/launcher_tests.rs"]
mod launcher_tests;
mod plan;
#[cfg(test)]
#[path = "joint_passive/tests/gpu_free_ball.rs"]
mod tests_gpu_free_ball;
#[cfg(test)]
#[path = "joint_passive/tests/gpu_guards.rs"]
mod tests_gpu_guards;
#[cfg(test)]
#[path = "joint_passive/tests/gpu_joint.rs"]
mod tests_gpu_joint;
#[cfg(test)]
#[path = "joint_passive/tests/host.rs"]
mod tests_host;
#[cfg(test)]
#[path = "joint_passive/tests/oracle.rs"]
mod tests_oracle;
#[cfg(test)]
#[path = "joint_passive/tests/support.rs"]
mod tests_support;

#[cfg(feature = "cuda-probe")]
fn read(
    b: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
    b.read_range_into(0, &mut v)?;
    crate::physics::check_device_values(&v, field)?;
    Ok(v)
}
// Frozen passive.py82, math.py23/162/178, util_misc.py865.
// Baseline 71da24d, Apache-2.0.
// Copyright2025 The Newton Developers; LICENSES/Apache-2.0.txt.
// Ordinary float expressions preserve the original compiler contraction policy.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
__device__ float jp_poly(float linear,float a,float b,float x){float result=linear;result=result+a*x;result=result+(b*x)*x;return result;}
__device__ float jp_len3(float x,float y,float z){return sqrtf((x*x+y*y)+z*z);}
__device__ void jp_quat_sub(const float* qa,const float* qb,float* out){
 float n=sqrtf(((qa[0]*qa[0]+qa[1]*qa[1])+qa[2]*qa[2])+qa[3]*qa[3]);
 float a0,a1,a2,a3;
 if(n>0.0f){float inv=1.0f/n;a0=qa[0]*inv;a1=qa[1]*inv;a2=qa[2]*inv;a3=qa[3]*inv;}
 else{a0=0.0f;a1=0.0f;a2=0.0f;a3=1.0f;}
 float b0=qb[0],b1=-qb[1],b2=-qb[2],b3=-qb[3];
 float d0=((b0*a0-b1*a1)-b2*a2)-b3*a3;
 float d1=((b0*a1+b1*a0)+b2*a3)-b3*a2;
 float d2=((b0*a2-b1*a3)+b2*a0)+b3*a1;
 float d3=((b0*a3+b1*a2)-b2*a1)+b3*a0;
 float sine=jp_len3(d1,d2,d3);
 if(sine==0.0f){out[0]=0.0f;out[1]=0.0f;out[2]=0.0f;return;}
 float speed=2.0f*atan2f(sine,d0);
 if(speed>3.14159265358979323846f)speed-=6.28318530717958647692f;
 out[0]=(d1*speed)/sine;out[1]=(d2*speed)/sine;out[2]=(d3*speed)/sine;
}
extern "C" __global__ void joint_passive_validate(const int* meta,const float* p,const float* state,float* status,unsigned nq,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool valid=nq==JP_NQ&&nv==JP_NV;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[4+4ull*JP_NV+j]==-131072&&p[j]==-131072.0f&&p[4+JP_PL+j]==-131072.0f&&state[j]==-131072.0f&&state[4+1ull*worlds*(JP_NQ+JP_NV)+j]==-131072.0f;
 for(unsigned j=0;j<JP_PL;++j)valid=valid&&isfinite(p[4+j]);
 for(unsigned j=0;j<JP_NQ;++j)valid=valid&&isfinite(state[4+1ull*w*JP_NQ+j]);for(unsigned j=0;j<JP_NV;++j)valid=valid&&isfinite(state[4+1ull*worlds*JP_NQ+1ull*w*JP_NV+j]);
 for(unsigned i=0;i<JP_NV;++i){int kind=meta[4+i],q=meta[4+JP_NV+i],j=meta[4+2ull*JP_NV+i],local=meta[4+3ull*JP_NV+i];unsigned qw=kind==0?7u:(kind==1?4u:1u),dw=kind==0?6u:(kind==1?3u:1u);if(kind<0||kind>3||q<0||j<0||j>=JP_NJ||local<0||(unsigned)local>=dw||qw>nq||(unsigned)q>nq-qw){valid=false;continue;}}
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void joint_passive_compute(const int* meta,const float* p,const float* state,float* out,unsigned nq,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/nv,i=tid%nv;int kind=meta[4+i],q=meta[4+JP_NV+i],j=meta[4+2ull*JP_NV+i],local=meta[4+3ull*JP_NV+i];float spring=0.0f,damper=0.0f;
 if(!JP_D){unsigned di=4+JP_OFF[3]+(w%JP_BATCH[3])*JP_NV+i,pi=4+JP_OFF[4]+2*((w%JP_BATCH[4])*JP_NV+i);float d=p[di],a=p[pi],b=p[pi+1];if(d!=0.0f||a!=0.0f||b!=0.0f){float v=state[4+1ull*worlds*JP_NQ+1ull*w*JP_NV+i];damper=-v*jp_poly(d,a,b,fabsf(v));}}
 if(!JP_S){unsigned si=4+JP_OFF[1]+(w%JP_BATCH[1])*JP_NJ+j,pi=4+JP_OFF[2]+2*((w%JP_BATCH[2])*JP_NJ+j);float s=p[si],a=p[pi],b=p[pi+1];if(s!=0.0f||a!=0.0f||b!=0.0f){const float* qs=state+4+1ull*w*JP_NQ+q;const float* qr=p+4+JP_OFF[0]+(w%JP_BATCH[0])*JP_NQ+q;
   if(kind==0&&local<3){float x=qs[0]-qr[0],y=qs[1]-qr[1],z=qs[2]-qr[2],r=jp_len3(x,y,z),k=jp_poly(s,a,b,r);float dif=local==0?x:(local==1?y:z);spring=-k*dif;}
   else if(kind<2){unsigned qo=kind==0?3u:0u;float dif[3];jp_quat_sub(qs+qo,qr+qo,dif);float r=jp_len3(dif[0],dif[1],dif[2]),k=jp_poly(s,a,b,r);spring=-k*dif[kind==0?local-3:local];}
   else{float dif=qs[0]-qr[0];spring=-dif*jp_poly(s,a,b,dif);}}}
 out[4+tid]=spring;out[4+1ull*worlds*nv+tid]=damper;
}
extern "C" __global__ void joint_passive_sum(const int* meta,const float* parts,const float* unused,float* forces,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;forces[4+tid]=__fadd_rn(parts[4+tid],parts[4+1ull*worlds*nv+tid]);}
"#;
