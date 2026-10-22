//! 常驻CSR因子与独立RHS流；不读取稠密矩阵来回代。
//! 源算法跟随冻结上游；实际融合产物等价仍未验收。
use super::{SparseMassData, SparseMassFactorOutput, SparseMassFactorWorld, SparseMassPlan};
use crate::diagnostics::InputError;
use crate::model::{BatchLayout, SparseMassInput};
#[cfg(feature = "cuda-probe")]
use crate::runtime::TransferBuffer;

#[path = "solve/cuda.rs"]
mod cuda;
#[path = "solve/data.rs"]
mod data;
#[cfg(feature = "cuda-probe")]
#[path = "solve/kernels.rs"]
mod kernels;
#[cfg(feature = "cuda-probe")]
#[path = "solve/launcher_abi.rs"]
mod launcher_abi;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "solve/launcher_tests.rs"]
mod launcher_tests;
#[path = "solve/plan.rs"]
mod plan;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "solve/tests.rs"]
mod tests;

pub struct SparseMassSolvePlan {
    mass: SparseMassPlan,
    #[cfg(feature = "cuda-probe")]
    metadata: launcher_abi::SparseSolveMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    kernels: kernels::SparseSolveKernels,
}
pub struct SparseMassSolveData {
    mass: SparseMassData,
    layout: BatchLayout,
    rhs_count: usize,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    rhs: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassSolveOutput {
    factor: SparseMassFactorOutput,
    layout: BatchLayout,
    rhs_count: usize,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassSolveWorld<'a> {
    pub factor: SparseMassFactorWorld<'a>,
    /// 行优先[RHS,nv]，不包含守卫。
    pub solution: &'a [f32],
}
impl SparseMassSolveOutput {
    pub fn input(&self) -> &SparseMassInput {
        self.factor.input()
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    pub fn world(&self, world: usize) -> Result<SparseMassSolveWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(SparseMassSolveWorld {
            factor: self.factor.world(world)?,
            solution: &self.values[4 + r.start..4 + r.end],
        })
    }
}
fn layout(worlds: usize, nv: usize, rhs_count: usize) -> Result<BatchLayout, InputError> {
    let invalid = || InputError::InvalidDimension {
        field: "sparse_solve_layout",
    };
    if rhs_count == 0
        || rhs_count > i32::MAX as usize
        || worlds == 0
        || worlds
            .checked_mul(rhs_count)
            .is_none_or(|v| v > i32::MAX as usize)
    {
        return Err(invalid());
    }
    let stride = nv.checked_mul(rhs_count).ok_or_else(invalid)?;
    let layout = BatchLayout::new(worlds, stride, 4)?;
    BatchLayout::new(
        1,
        layout.total_elements().checked_add(8).ok_or_else(invalid)?,
        4,
    )?;
    Ok(layout)
}
