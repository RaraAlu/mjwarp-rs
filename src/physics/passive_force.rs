//! 共享被动力累加链。
//! 本模块不代表完整被动力阶段。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
#[cfg(feature = "cuda-probe")]
use crate::model::SmoothForceInput;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{TransferBuffer, TransferSession};
use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use super::SmoothForceOptions;
use super::{
    FlexEdgePassivePlan, FluidForceData, FluidForceInput, FluidForcePlan,
    GravityCompensationOptions, GravityCompensationParameters, GravityCompensationPlan,
    JointPassiveData, JointPassivePlan, KinematicsData, SmoothForceData, SmoothForceField,
    SmoothForceOutput, SmoothForcePlan, TendonPassivePlan, TendonVelocityView,
};

// 子模块：纯移动自原单文件，行为不变。
mod construct;
mod data;
#[cfg(feature = "cuda-probe")]
#[path = "passive_force/kernels.rs"]
mod kernels;
#[cfg(feature = "cuda-probe")]
#[path = "passive_force/launcher_abi.rs"]
mod launcher_abi;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "passive_force/launcher_tests.rs"]
mod launcher_tests;
mod output;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "passive_force/tests/accumulate.rs"]
mod tests_accumulate;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "passive_force/tests/fluid.rs"]
mod tests_fluid;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "passive_force/tests/rotation.rs"]
mod tests_rotation;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "passive_force/tests/sleep.rs"]
mod tests_sleep;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "passive_force/tests/support.rs"]
mod tests_support;
mod update;

pub struct PassiveForcePlan<'a, 'v, 'p, 'f> {
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    joint: &'a JointPassivePlan,
    tendon: &'a TendonPassivePlan<'v, 'p>,
    flex_edge: &'a FlexEdgePassivePlan<'f>,
    gravity: Option<GravityCompensationPlan<'f>>,
    fluid: Option<FluidForcePlan<'f>>,
    identity: Arc<()>,
    smooth: SmoothForcePlan,
    nv: usize,
    disable_spring: bool,
    disable_damper: bool,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: launcher_abi::PassiveForceMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    dummy: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: kernels::PassiveForceKernels,
}

pub struct PassiveForceData {
    identity: Arc<()>,
    layout: BatchLayout,
    ready: bool,
    smooth: SmoothForceData,
    fluid: Option<FluidForceData>,
    #[cfg(feature = "cuda-probe")]
    parts: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    gravity: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}

pub struct PassiveForceView<'a, 's, 'p> {
    data: &'a PassiveForceData,
    _joint: &'a JointPassiveData,
    _tendon_velocity: &'a TendonVelocityView<'s, 'p>,
    _flex_source: &'a KinematicsData,
}

pub struct ClearedPassiveForceView<'a> {
    data: &'a PassiveForceData,
}

pub struct SleepClearedPassiveForceView<'a> {
    data: &'a PassiveForceData,
    _sleep_source: &'a KinematicsData,
}

#[derive(Clone, Debug)]
pub struct PassiveForceSnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
    gravity: Vec<f32>,
}

impl PassiveForceSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    pub fn spring(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        Ok(&self.values[4 + range.start..4 + range.end])
    }

    pub fn damper(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        let offset = 4 + self.layout.total_elements();
        Ok(&self.values[offset + range.start..offset + range.end])
    }

    pub fn gravity(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        Ok(&self.gravity[4 + range.start..4 + range.end])
    }
}

#[cfg(feature = "cuda-probe")]
fn read_parts(buffer: &TransferBuffer<f32>, count: usize) -> Result<Vec<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut values)?;
    crate::physics::check_device_values(&values, "passive_force_parts")?;
    Ok(values)
}

// 冻结顺序：passive.py::_spring_damper。
// 基线：71da24d，许可：Apache-2.0。
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void passive_force_clear(const int* meta,const float* p,const float* state,float* out,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;out[4+tid]=0.0f;out[4+(unsigned long long)work+tid]=0.0f;
}
extern "C" __global__ void passive_force_sum(const int* meta,const float* p,const float* parts,float* forces,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;forces[4+tid]=__fadd_rn(parts[4+tid],parts[4+1ull*worlds*nv+tid]);
}
extern "C" __global__ void passive_force_clear_gravity(const int* meta,const float* p,const float* state,float* out,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;out[4+tid]=0.0f;
}
extern "C" __global__ void passive_force_add_fluid(const int* meta,const float* p,const float* fluid,float* out,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;out[4+tid]=__fadd_rn(out[4+tid],fluid[4+tid]);
}
"#;
