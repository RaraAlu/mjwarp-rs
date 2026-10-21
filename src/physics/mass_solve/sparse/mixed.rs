//! 常驻Sparse、Small与Compact混合分解；Tile仍拒绝。
//! 合并qLD前缀与CSR尾；不计算CPU物理结果。
mod cuda;
mod data;
mod merge;
mod plan;
mod solve;
#[cfg(test)]
mod tests;

use super::{SparseMassFactorData, SparseMassFactorPlan, SparseMassSmallData, SparseMassSmallPlan};
use crate::diagnostics::InputError;
use crate::model::{BatchLayout, SparseMassInput, SparseMassRoute};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{TransferBuffer, TransferSession};
use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use merge::MixedFactorMergeKernel;
pub use solve::{
    SparseMassMixedResidentData, SparseMassMixedResidentPlan, SparseMassMixedSolveData,
    SparseMassMixedSolveOutput, SparseMassMixedSolvePlan, SparseMassMixedSolveWorld,
};

pub struct SparseMassMixedFactorPlan {
    sparse: SparseMassFactorPlan,
    small: SparseMassSmallPlan,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    merge: MixedFactorMergeKernel,
}
pub struct SparseMassMixedFactorData {
    sparse: SparseMassFactorData,
    small: SparseMassSmallData,
    layout: BatchLayout,
    gathered: bool,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    matrix: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassMixedFactorOutput {
    input: Arc<SparseMassInput>,
    layout: BatchLayout,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassMixedFactorWorld<'a> {
    /// Small上三角U前缀，下三角保持正零。
    pub dense_upper: &'a [f32],
    /// 原CSR尾；非紧凑Small行仍保留原M。
    pub sparse_factors: &'a [f32],
    /// 冻结稀疏阶段处理全部nv的倒数。
    /// Small使用原M对角，不是U对角。
    pub diagonal_inverse: &'a [f32],
}
impl SparseMassMixedFactorOutput {
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<SparseMassMixedFactorWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        let v = &self.values[4 + r.start..4 + r.end];
        let prefix = self.input.qld_block_total();
        let tail = prefix + self.input.nc();
        Ok(SparseMassMixedFactorWorld {
            dense_upper: &v[..prefix],
            sparse_factors: &v[prefix..tail],
            diagonal_inverse: &v[tail..],
        })
    }
}
fn routes(input: &SparseMassInput) -> Result<(), InputError> {
    for block in input.blocks() {
        if block.route == SparseMassRoute::TileCholesky {
            return Err(InputError::InvalidTopology {
                field: "mixed_factor_route",
                index: block.start,
                reason: "MathDx_product_precision_admission_missing",
            });
        }
    }
    if !input
        .blocks()
        .iter()
        .any(|b| b.route == SparseMassRoute::SparseLdl)
    {
        return Err(InputError::InvalidTopology {
            field: "mixed_factor_route",
            index: 0,
            reason: "sparse_CSR_tail_required",
        });
    }
    Ok(())
}
fn layout(input: &SparseMassInput, worlds: usize) -> Result<BatchLayout, InputError> {
    let stride = input
        .qld_block_total()
        .checked_add(input.nc())
        .and_then(|v| v.checked_add(input.nv()))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "mixed_factor_stride",
        })?;
    let layout = BatchLayout::new(worlds, stride, 4)?;
    if worlds == 0 || layout.total_elements() > (u32::MAX - 255) as usize {
        return Err(InputError::InvalidDimension {
            field: "mixed_factor_work",
        });
    }
    BatchLayout::new(
        1,
        layout
            .total_elements()
            .checked_add(8)
            .ok_or(InputError::Overflow {
                field: "mixed_factor_guards",
            })?,
        4,
    )?;
    Ok(layout)
}
