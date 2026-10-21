//! 消费同代合并因子，求解Sparse、Small与Compact。
//! RHS独立批量；Tile与CRB桥接仍不支持。
use super::{
    SparseMassMixedFactorData, SparseMassMixedFactorOutput, SparseMassMixedFactorPlan,
    SparseMassMixedFactorWorld,
};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
#[cfg(any(feature = "cuda-probe", test))]
use crate::model::SparseMassRoute;
use crate::model::{BatchLayout, SparseMassInput};
use crate::runtime::TransferSession;
mod resident;
#[cfg(feature = "cuda-probe")]
use self::kernels::{MixedSolveFusedKernel, MixedSolveValidationKernel};
#[cfg(feature = "cuda-probe")]
use self::validation_buffer::MixedSolveValidationMetadataBuffer;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
pub use resident::{SparseMassMixedResidentData, SparseMassMixedResidentPlan};

// 子模块：纯移动自原单文件，行为不变。
mod cuda;
mod data;
mod errors;
mod fused_meta;
mod fused_spec;
mod kernels;
mod layout;
mod plan;
#[cfg(test)]
#[path = "solve/tests/cached_factor.rs"]
mod tests_cached_factor;
#[cfg(test)]
#[path = "solve/tests/fused_plan.rs"]
mod tests_fused_plan;
#[cfg(test)]
#[path = "solve/tests/gpu_checks.rs"]
mod tests_gpu_checks;
#[cfg(test)]
#[path = "solve/tests/gpu_fused.rs"]
mod tests_gpu_fused;
#[cfg(test)]
#[path = "solve/tests/layout.rs"]
mod tests_layout;
#[cfg(test)]
#[path = "solve/tests/support.rs"]
mod tests_support;
#[cfg(test)]
#[path = "solve/tests/validation_plan.rs"]
mod tests_validation_plan;
mod validation_buffer;
mod validation_meta;

pub struct SparseMassMixedSolvePlan {
    mass: SparseMassMixedFactorPlan,
    #[cfg(feature = "cuda-probe")]
    metadata: MixedSolveValidationMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validation: MixedSolveValidationKernel,
    #[cfg(feature = "cuda-probe")]
    solve: MixedSolveFusedKernel,
}
pub struct SparseMassMixedSolveData {
    mass: SparseMassMixedFactorData,
    layout: BatchLayout,
    rhs_count: usize,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    rhs: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassMixedSolveOutput {
    factor: SparseMassMixedFactorOutput,
    layout: BatchLayout,
    rhs_count: usize,
    rhs: Vec<f32>,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassMixedSolveWorld<'a> {
    pub factor: SparseMassMixedFactorWorld<'a>,
    /// 每世界按RHS、DOF连续排列。
    pub rhs: &'a [f32],
    pub solution: &'a [f32],
}
impl SparseMassMixedSolveOutput {
    pub fn input(&self) -> &SparseMassInput {
        self.factor.input()
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    pub fn world(&self, world: usize) -> Result<SparseMassMixedSolveWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(SparseMassMixedSolveWorld {
            factor: self.factor.world(world)?,
            rhs: &self.rhs[4 + r.start..4 + r.end],
            solution: &self.values[4 + r.start..4 + r.end],
        })
    }
}
