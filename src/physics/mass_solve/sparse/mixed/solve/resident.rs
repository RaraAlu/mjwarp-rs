//! GPU CRB到三路混合分解/求解的常驻桥接。
//! 调用方只写qpos/RHS；物理CSR始终留在设备上。
use super::{SparseMassMixedSolveData, SparseMassMixedSolveOutput, SparseMassMixedSolvePlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, SparseMassInput};
use crate::physics::{MassMatrixData, MassMatrixOutput, MassMatrixPlan};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

// 子模块：纯移动自原单文件，行为不变。
mod cuda;
mod data;
mod gather_meta;
mod kernels;
mod plan;
mod validation_meta;
#[cfg(feature = "cuda-probe")]
use self::kernels::ResidentMixedGatherMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use self::kernels::{ResidentMixedGatherKernel, ResidentMixedValidationKernel};
#[cfg(test)]
#[path = "resident/tests/assembled.rs"]
mod tests_assembled;
#[cfg(test)]
#[path = "resident/tests/cache_fault.rs"]
mod tests_cache_fault;
#[cfg(test)]
#[path = "resident/tests/gpu_bridge.rs"]
mod tests_gpu_bridge;
#[cfg(test)]
#[path = "resident/tests/gpu_gather.rs"]
mod tests_gpu_gather;
#[cfg(test)]
#[path = "resident/tests/gpu_product.rs"]
mod tests_gpu_product;
#[cfg(test)]
#[path = "resident/tests/gpu_validation.rs"]
mod tests_gpu_validation;
#[cfg(test)]
#[path = "resident/tests/support.rs"]
mod tests_support;
#[cfg(test)]
#[path = "resident/tests/typed_plans.rs"]
mod tests_typed_plans;

pub struct SparseMassMixedResidentPlan {
    matrix: MassMatrixPlan,
    solve: SparseMassMixedSolvePlan,
    #[cfg(feature = "cuda-probe")]
    metadata: ResidentMixedGatherMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validation: ResidentMixedValidationKernel,
    #[cfg(feature = "cuda-probe")]
    gather: ResidentMixedGatherKernel,
}
pub struct SparseMassMixedResidentData {
    matrix: MassMatrixData,
    solve: SparseMassMixedSolveData,
    gathered: bool,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
