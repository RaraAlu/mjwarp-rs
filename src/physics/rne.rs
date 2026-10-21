//! 常驻RNE加速度辅助。
//! 不替代完整G02或等价调度。
#[cfg(feature = "cuda-probe")]
use self::kernels::RneBackwardKernel;
#[cfg(feature = "cuda-probe")]
use self::kernels::RneCaccKernel;
#[cfg(feature = "cuda-probe")]
use self::kernels::RneCfrcKernel;
#[cfg(feature = "cuda-probe")]
use self::kernels::RneQfrcKernel;
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::RneMetadataBuffer;

// 子模块：纯移动自原单文件，行为不变。
mod data;
mod kernels;
mod launcher_abi;
mod launcher_meta;
mod launcher_plans;
mod metadata_buffer;
mod plan;

#[cfg(test)]
mod tests {
    #[cfg(feature = "cuda-probe")]
    pub(super) use super::tests_support::*;
}
#[cfg(test)]
#[path = "rne/tests/backward_check.rs"]
mod tests_backward_check;
#[cfg(test)]
#[path = "rne/tests/backward_run.rs"]
mod tests_backward_run;
#[cfg(test)]
#[path = "rne/tests/cacc_run.rs"]
mod tests_cacc_run;
#[cfg(test)]
#[path = "rne/tests/cfrc_qfrc.rs"]
mod tests_cfrc_qfrc;
#[cfg(test)]
#[path = "rne/tests/cfrc_run.rs"]
mod tests_cfrc_run;
#[cfg(test)]
#[path = "rne/tests/check.rs"]
mod tests_check;
#[cfg(test)]
#[path = "rne/tests/qfrc_run.rs"]
mod tests_qfrc_run;
#[cfg(test)]
#[path = "rne/tests/support.rs"]
mod tests_support;

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

/// 复用空间速度底座的严格辅助。
/// 共享重力；不转换完整Option。
pub struct RnePlan {
    velocity: ComVelocityPlan,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    depth: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: RneMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    backward_kernel: RneBackwardKernel,
    #[cfg(feature = "cuda-probe")]
    cacc_kernel: RneCaccKernel,
    #[cfg(feature = "cuda-probe")]
    cfrc_kernel: RneCfrcKernel,
    #[cfg(feature = "cuda-probe")]
    qfrc_kernel: RneQfrcKernel,
}

/// 独占qacc与RNE工作区。
pub struct RneData {
    velocity: ComVelocityData,
    layout: RneLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    flg_acc: bool,
    #[cfg(feature = "cuda-probe")]
    qacc: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

#[derive(Clone, Debug)]
pub struct RneSnapshot {
    layout: RneLayout,
    flg_acc: bool,
    values: Vec<f32>,
}

/// 空间向量采用角量在前。
#[derive(Clone, Copy, Debug)]
pub struct RneWorld<'a> {
    pub cacc: &'a [f32],
    pub cfrc_int: &'a [f32],
    /// 跟随上游字段名。
    /// flg_acc=true时包含加速度。
    pub qfrc_bias: &'a [f32],
}

#[derive(Clone, Copy, Debug)]
struct RneLayout {
    output: BatchLayout,
    qacc: BatchLayout,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}

impl RneLayout {
    fn new(worlds: usize, nbody: usize, nv: usize) -> Result<Self, InputError> {
        let overflow = || InputError::Overflow {
            field: "rne_layout",
        };
        if worlds == 0 || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "rne_dimensions",
            });
        }
        worlds
            .checked_mul(nbody)
            .filter(|&v| v <= (u32::MAX - 255) as usize)
            .ok_or_else(overflow)?;
        let width = nbody
            .checked_mul(12)
            .and_then(|n| n.checked_add(nv))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let metadata = nbody
            .checked_mul(4)
            .and_then(|n| n.checked_add(nv))
            .and_then(|n| n.checked_add(5))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, metadata, 4)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        let qacc = BatchLayout::new(worlds, nv, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        let qa = qacc.total_elements().checked_add(8).ok_or_else(overflow)?;
        BatchLayout::new(1, qa, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = guarded;
        Ok(Self {
            output,
            qacc,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

#[cfg(feature = "cuda-probe")]
const RNE_CUDA: &str = include_str!("rne.cu");

#[cfg(all(test, feature = "cuda-probe"))]
#[path = "../../tests/support/rne_acceleration_frozen.rs"]
mod frozen;
