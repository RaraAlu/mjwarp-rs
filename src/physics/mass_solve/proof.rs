//! 刚体质量求解证明类型与类型化规格。

#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RIGID_MASS_SOLVE_METADATA_KIND;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RIGID_MASS_SOLVE_SLOTS;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RigidMassSolveDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::rigid_mass_solve_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::validate::validate_rigid_mass_solve_metadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RigidMassSolveMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RigidMassSolveMetadata {
    type Dims = RigidMassSolveDims;

    const KIND: ProofKind = RIGID_MASS_SOLVE_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rigid_mass_solve_metadata(values, *dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RigidMassSolveBaseMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for RigidMassSolveBaseMetadata {
    type Dims = RigidMassSolveDims;

    const KIND: ProofKind = ProofKind("rigid_mass_solve.blocks.base.v1");

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rigid_mass_solve_metadata(values, *dims)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RigidMassSolveSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rigid_mass_solve私有ABI。
unsafe impl KernelAbi for RigidMassSolveSpec {
    const NAME: &'static str = "rigid_mass_solve";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RIGID_MASS_SOLVE_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RigidMassSolveSpec {
    type Dims = RigidMassSolveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rigid_mass_solve_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rigid_mass_solve_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rigid_mass_solve_plan(
    dims: &RigidMassSolveDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RigidMassSolveDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rigid_mass_solve_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rigid_mass_solve_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RigidMassSolveSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}
