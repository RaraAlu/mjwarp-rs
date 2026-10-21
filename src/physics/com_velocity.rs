//! 常驻G02空间速度子集。
//! 完整G02与等价阶段仍待实现。
mod cuda;
mod data;
mod layout;
mod plan;
mod resident;
#[cfg(test)]
mod tests;

use super::{KinematicsData, KinematicsPlan};
use crate::diagnostics::InputError;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
use crate::model::BatchLayout;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};

/// 一次上传与编译的空间速度计划。
/// 复用严格常驻位置与质心阶段。
/// 拓扑与参数只共享一份模型。
/// 不计算RNE、休眠与柔体动力学。
pub struct ComVelocityPlan {
    position: KinematicsPlan,
    nbody: usize,
    njnt: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernel: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    position_kernels: super::resident::VelocityPositionKernels,
}

/// 独占位置、速度与派生设备字段。
/// 相同尺寸不允许跨计划使用。
/// 合法状态写入废弃速度结果。
pub struct ComVelocityData {
    position: KinematicsData,
    layout: ComVelocityLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    qvel: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    rigid: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    com: TransferBuffer<f32>,
}

/// 借用常驻运动学状态的空间速度计划。
/// 它只拥有派生设备缓冲。
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
pub(super) struct ResidentComVelocityPlan<'p> {
    source: &'p KinematicsPlan,
    identity: std::sync::Arc<()>,
    nbody: usize,
    njnt: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernel: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    position_kernels: super::resident::VelocityPositionKernels,
}

/// 同源位置、质心与速度派生结果。
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
pub(super) struct ResidentComVelocityData {
    identity: std::sync::Arc<()>,
    layout: ComVelocityLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    rigid: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    com: TransferBuffer<f32>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RneInputs<'a> {
    pub qvel: &'a TransferBuffer<f32>,
    pub com: &'a TransferBuffer<f32>,
    pub velocity: &'a TransferBuffer<f32>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FluidInputs<'a> {
    pub rigid: &'a TransferBuffer<f32>,
    pub com: &'a TransferBuffer<f32>,
    pub velocity: &'a TransferBuffer<f32>,
    pub attached: Option<&'a TransferBuffer<f32>>,
}

/// 显式回读后的独立宿主快照。
/// 后续设备写入不改变快照。
#[derive(Clone, Debug)]
pub struct ComVelocitySnapshot {
    layout: ComVelocityLayout,
    values: Vec<f32>,
}

/// 单世界空间字段的只读视图。
/// 六维顺序先角向再线向。
#[derive(Clone, Copy, Debug)]
pub struct ComVelocityWorld<'a> {
    pub cvel: &'a [f32],
    pub cdof_dot: &'a [f32],
}

#[derive(Clone, Copy, Debug)]
struct ComVelocityLayout {
    output: BatchLayout,
    qvel: BatchLayout,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
    #[cfg(feature = "cuda-probe")]
    guarded_qvel: usize,
    #[cfg(feature = "cuda-probe")]
    guarded_rigid: usize,
    #[cfg(feature = "cuda-probe")]
    guarded_com: usize,
}

#[cfg(all(test, feature = "cuda-probe"))]
type FluidSnapshot = (Vec<f32>, Vec<f32>, Vec<f32>, Option<Vec<f32>>);

fn check_qvel(expected: usize, qvel: &[f32]) -> Result<(), InputError> {
    if qvel.len() != expected {
        return Err(InputError::LengthMismatch {
            field: "com_velocity_qvel",
            expected,
            actual: qvel.len(),
        });
    }
    if let Some(index) = qvel.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite {
            field: "com_velocity_qvel",
            index,
        });
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
fn velocity_workspace(
    session: &TransferSession,
    length: usize,
) -> Result<TransferBuffer<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(length)?;
    values.fill(-131072.0);
    values[4..length - 4].fill(f32::NAN);
    session.upload(&values)
}
