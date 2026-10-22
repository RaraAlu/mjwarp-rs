//! 借用实际设备肌腱雅可比。
//! 独占速度；不交付完整G02。
#[cfg(feature = "cuda-probe")]
use self::kernels::{TendonVelocityComputeKernel, TendonVelocityValidateKernel};
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::TendonVelocityMetadataBuffer;
use super::{KinematicsData, KinematicsPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, TendonRows};
#[cfg(all(feature = "cuda-probe", test))]
use crate::runtime::launcher::DeviceSpan;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

struct Spec {
    nv: usize,
    nt: usize,
    nnz: usize,
    stride: usize,
    meta: Vec<i32>,
}
fn spec(nv: usize, rows: &TendonRows) -> Result<Spec, InputError> {
    let nt = rows.ntendon();
    let nnz = rows.nnz();
    if nv == 0 || nt == 0 {
        return Err(InputError::InvalidDimension {
            field: "tendon_velocity_dimensions",
        });
    }
    let overflow = || InputError::Overflow {
        field: "tendon_velocity_metadata",
    };
    let len = nt
        .checked_mul(2)
        .and_then(|n| n.checked_add(nnz))
        .and_then(|n| n.checked_add(8))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or_else(overflow)?;
    let stride = rows
        .nwrap()
        .checked_mul(6)
        .and_then(|n| n.checked_add(nt))
        .and_then(|n| n.checked_add(nnz))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or_else(overflow)?;
    let mut meta = Vec::with_capacity(len);
    meta.extend([-131072; 4]);
    meta.extend(rows.rowadr());
    meta.extend(rows.rownnz());
    meta.extend(rows.colind());
    meta.extend([-131072; 4]);
    Ok(Spec {
        nv,
        nt,
        nnz,
        stride,
        meta,
    })
}
fn layout(worlds: usize, width: usize) -> Result<BatchLayout, InputError> {
    if worlds == 0
        || width == 0
        || worlds
            .checked_mul(width)
            .is_none_or(|n| n > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "tendon_velocity_work",
        });
    }
    let l = BatchLayout::new(worlds, width, 4)?;
    l.total_elements()
        .checked_add(8)
        .and_then(|n| n.checked_mul(4))
        .ok_or(InputError::Overflow {
            field: "tendon_velocity_guards",
        })?;
    Ok(l)
}

/// 计划借用实际位置生产者。
pub struct TendonVelocityPlan<'p> {
    source: &'p KinematicsPlan,
    spec: Arc<Spec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TendonVelocityMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validate: TendonVelocityValidateKernel,
    #[cfg(feature = "cuda-probe")]
    compute: TendonVelocityComputeKernel,
}
/// qvel不借用柔体边状态。
pub struct TendonVelocityData {
    spec: Arc<Spec>,
    velocity: BatchLayout,
    output: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    qvel: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
/// 视图锁定实际源与速度状态。
pub struct TendonVelocityView<'a, 'p> {
    plan: &'a TendonVelocityPlan<'p>,
    source: &'a KinematicsData,
    data: &'a TendonVelocityData,
}
#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityDeviceView<'a> {
    pub source: &'a TransferBuffer<f32>,
    pub velocity: &'a TransferBuffer<f32>,
    pub worlds: usize,
}
/// 独占历史值，不认证未来状态。
#[derive(Clone, Debug)]
pub struct TendonVelocitySnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl TendonVelocitySnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}

// 子模块：纯移动自原单文件，行为不变。
mod data;
mod kernels;
mod launcher_abi;
mod launcher_meta;
mod launcher_specs;
mod metadata_buffer;
mod plan;
#[cfg(test)]
#[path = "tendon_velocity/tests/gpu_consumer.rs"]
mod tests_gpu_consumer;
#[cfg(test)]
#[path = "tendon_velocity/tests/gpu_disable.rs"]
mod tests_gpu_disable;
#[cfg(test)]
#[path = "tendon_velocity/tests/gpu_guards.rs"]
mod tests_gpu_guards;
#[cfg(test)]
#[path = "tendon_velocity/tests/gpu_passive.rs"]
mod tests_gpu_passive;
#[cfg(test)]
#[path = "tendon_velocity/tests/gpu_periods.rs"]
mod tests_gpu_periods;
#[cfg(test)]
#[path = "tendon_velocity/tests/host.rs"]
mod tests_host;
#[cfg(test)]
#[path = "tendon_velocity/tests/launcher.rs"]
mod tests_launcher;
#[cfg(test)]
#[path = "tendon_velocity/tests/support.rs"]
mod tests_support;
mod view;

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
// Adapted from frozen forward.py::_tendon_velocity (71da24d), Apache-2.0.
// Copyright 2025 The Newton Developers; LICENSES/Apache-2.0.txt.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void tendon_velocity_validate(const int* meta,const float* jac,const float* v,float* status,unsigned a,unsigned b,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=true;
 for(unsigned k=0;k<4;++k)ok=ok&&meta[k]==-131072&&meta[4+2ull*TV_NT+TV_NNZ+k]==-131072&&jac[k]==-131072.0f&&jac[4+1ull*worlds*TV_STRIDE+k]==-131072.0f&&v[k]==-131072.0f&&v[4+1ull*worlds*TV_NV+k]==-131072.0f;
 for(unsigned t=0;t<TV_NT;++t){int start=meta[4+t],n=meta[4+TV_NT+t];if(start<0||n<0||start>TV_NNZ||n>TV_NNZ-start){ok=false;continue;}for(int i=0;i<n;++i){int c=meta[4+2ull*TV_NT+start+i];if(c<0||c>=TV_NV)ok=false;}}
 for(unsigned i=0;i<TV_NNZ;++i)ok=ok&&isfinite(jac[4+1ull*w*TV_STRIDE+TV_NT+i]);
 for(unsigned i=0;i<TV_NV;++i)ok=ok&&isfinite(v[4+1ull*w*TV_NV+i]);status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void tendon_velocity_compute(const int* meta,const float* jac,const float* v,float* out,unsigned a,unsigned b,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/TV_NT,t=tid%TV_NT;int start=meta[4+t],n=meta[4+TV_NT+t];float velocity=0.0f;
 for(int i=0;i<n;++i){int sparse=start+i;float J=jac[4+1ull*w*TV_STRIDE+TV_NT+sparse];if(J!=0.0f){int col=meta[4+2ull*TV_NT+sparse];velocity+=J*v[4+1ull*w*TV_NV+col];}}
 out[4+tid]=velocity;
}
"#;
