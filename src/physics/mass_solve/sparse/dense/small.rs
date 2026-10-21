//! 非紧凑Small前缀U；不生成Sparse/Compact因子或Tile替代。
use super::{SparseMassDenseData, SparseMassDensePlan};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, SparseMassInput, SparseMassRoute};
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
mod buffer;
mod cuda;
mod data;
mod factor_meta;
mod kernels;
mod plan;
mod validation_meta;
#[cfg(feature = "cuda-probe")]
use buffer::SmallFactorMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use kernels::{SmallFactorKernel, SmallValidationKernel};
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "small/tests/gpu_cache.rs"]
mod tests_gpu_cache;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "small/tests/gpu_factor.rs"]
mod tests_gpu_factor;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "small/tests/gpu_product.rs"]
mod tests_gpu_product;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "small/tests/gpu_validation.rs"]
mod tests_gpu_validation;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "small/tests/metadata.rs"]
mod tests_metadata;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "small/tests/support.rs"]
mod tests_support;

pub struct SparseMassSmallPlan {
    source: SparseMassDensePlan,
    #[cfg(feature = "cuda-probe")]
    metadata: SmallFactorMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validator: SmallValidationKernel,
    #[cfg(feature = "cuda-probe")]
    groups: Vec<SmallFactorKernel>,
}
pub struct SparseMassSmallData {
    source: SparseMassDenseData,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    factor: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassSmallOutput {
    input: Arc<SparseMassInput>,
    layout: BatchLayout,
    values: Vec<f32>,
}
impl SparseMassSmallOutput {
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    /// 仅含Small上三角U；低三角保持正零。
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
fn check_routes(input: &SparseMassInput) -> Result<(), InputError> {
    for block in input.blocks() {
        if block.route == SparseMassRoute::TileCholesky {
            return Err(InputError::InvalidTopology {
                field: "small_prefix_route",
                index: block.start,
                reason: "MathDx_tile_factor_not_implemented",
            });
        }
    }
    Ok(())
}
