//! G03正定刚体质量矩阵辅助。
//! 不替代完整分解与求解入口。

use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, InertialModelInput};
use crate::runtime::TransferSession;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

mod resident;
pub use resident::{MassSolveData, MassSolvePlan};
mod sparse;
pub use sparse::{
    SparseMassData, SparseMassDenseData, SparseMassDenseOutput, SparseMassDensePlan,
    SparseMassFactorData, SparseMassFactorOutput, SparseMassFactorPlan, SparseMassFactorWorld,
    SparseMassMixedFactorData, SparseMassMixedFactorOutput, SparseMassMixedFactorPlan,
    SparseMassMixedFactorWorld, SparseMassMixedResidentData, SparseMassMixedResidentPlan,
    SparseMassMixedSolveData, SparseMassMixedSolveOutput, SparseMassMixedSolvePlan,
    SparseMassMixedSolveWorld, SparseMassOutput, SparseMassPlan, SparseMassSmallData,
    SparseMassSmallOutput, SparseMassSmallPlan, SparseMassSolveData, SparseMassSolveOutput,
    SparseMassSolvePlan, SparseMassSolveWorld, SparseMassWorld,
};

// 子模块：纯移动自原单文件，行为不变。
mod buffer;
mod cuda;
mod errors;
mod factor;
mod kernel;
mod metadata;
mod output;
mod probe;
mod proof;
mod validate;
#[cfg(feature = "cuda-probe")]
use cuda::MASS_SOLVE_CUDA;
pub use factor::{MassFactorBlock, MassFactorKind};
#[cfg(feature = "cuda-probe")]
use factor::{check_block_intervals, factor_blocks};
pub use output::{MassSolveOutput, MassSolveWorld};
#[cfg(feature = "cuda-probe")]
use probe::check_result;
pub use probe::probe_mass_solve;
#[cfg(all(test, feature = "cuda-probe"))]
use probe::solve_device;

#[cfg(test)]
#[path = "mass_solve/tests/device_factor.rs"]
mod tests_device_factor;
#[cfg(test)]
#[path = "mass_solve/tests/device_tile.rs"]
mod tests_device_tile;
#[cfg(test)]
#[path = "mass_solve/tests/device_wrapper.rs"]
mod tests_device_wrapper;
#[cfg(test)]
#[path = "mass_solve/tests/host.rs"]
mod tests_host;
#[cfg(test)]
#[path = "mass_solve/tests/plans.rs"]
mod tests_plans;
#[cfg(test)]
#[path = "mass_solve/tests/trees.rs"]
mod tests_trees;

#[derive(Clone, Copy, Debug)]
struct MassSolveLayout {
    output: BatchLayout,
    rhs: BatchLayout,
    nv: usize,
    rhs_count: usize,
    #[cfg(any(feature = "cuda-probe", test))]
    guarded: usize,
}
impl MassSolveLayout {
    fn new(worlds: usize, nv: usize, rhs_count: usize) -> Result<Self, InputError> {
        if worlds == 0
            || worlds > (u32::MAX - 255) as usize
            || rhs_count == 0
            || rhs_count > i32::MAX as usize
        {
            return Err(InputError::InvalidDimension {
                field: "mass_solve_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "mass_solve_layout",
        };
        let rhs_stride = nv.checked_mul(rhs_count).ok_or_else(overflow)?;
        let stride = nv
            .checked_mul(nv)
            .and_then(|n| n.checked_add(nv))
            .and_then(|n| n.checked_add(rhs_stride))
            .and_then(|n| n.checked_add(3))
            .ok_or_else(overflow)?;
        if stride > i32::MAX as usize {
            return Err(overflow());
        }
        let output = BatchLayout::new(worlds, stride, 4)?;
        let rhs = BatchLayout::new(worlds, rhs_stride, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        Ok(Self {
            output,
            rhs,
            nv,
            rhs_count,
            #[cfg(any(feature = "cuda-probe", test))]
            guarded,
        })
    }
    fn check_rhs(&self, rhs: &[f32]) -> Result<(), InputError> {
        if rhs.len() != self.rhs.total_elements() {
            return Err(InputError::LengthMismatch {
                field: "mass_solve_rhs",
                expected: self.rhs.total_elements(),
                actual: rhs.len(),
            });
        }
        for (index, &v) in rhs.iter().enumerate() {
            if !v.is_finite() {
                return Err(InputError::NonFinite {
                    field: "mass_solve_rhs",
                    index,
                });
            }
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "cuda-probe"))]
#[path = "../../tests/support/g03_first_difference.rs"]
mod first_difference;
