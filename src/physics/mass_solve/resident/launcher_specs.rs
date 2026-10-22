//! 常驻质量分解两类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassFactorDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassSolveDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻质量分解世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn rhs_count(rhs: u32) -> Result<u64, LauncherError> {
    if rhs == 0 || rhs > i32::MAX as u32 {
        return Err(LauncherError::InvalidArray("常驻质量分解右端项数无效"));
    }
    Ok(u64::from(rhs))
}

#[cfg(feature = "cuda-probe")]
fn checked_buffer(
    base: u64,
    worlds: u64,
    width: u64,
    field: &'static str,
) -> Result<u64, LauncherError> {
    worlds
        .checked_mul(width)
        .and_then(|value| value.checked_add(base))
        .ok_or(LauncherError::Overflow(field))
}

#[cfg(feature = "cuda-probe")]
fn rhs_elements(
    static_dims: ResidentMassStaticDims,
    worlds: u32,
    rhs: u32,
) -> Result<u64, LauncherError> {
    let width = u64::from(rhs)
        .checked_mul(u64::from(static_dims.n))
        .ok_or(LauncherError::Overflow("resident_mass_rhs"))?;
    checked_buffer(8, world_count(worlds)?, width, "resident_mass_rhs")
}

/// 分解内核四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mass_factor_lengths(
    dims: &ResidentMassFactorDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    rhs_count(dims.rhs_count)?;
    let metadata = static_dims.metadata_elements()?;
    let worlds = world_count(dims.worlds)?;
    let rhs = rhs_elements(static_dims, dims.worlds, dims.rhs_count)?;
    let matrix = checked_buffer(
        8,
        worlds,
        static_dims.matrix_stride()?,
        "resident_mass_matrix",
    )?;
    let factor = checked_buffer(
        8,
        worlds,
        static_dims.factor_stride()?,
        "resident_mass_factor",
    )?;
    Ok([metadata, rhs, matrix, factor])
}

/// 回代内核四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mass_solve_lengths(
    dims: &ResidentMassSolveDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    rhs_count(dims.rhs_count)?;
    let metadata = static_dims.metadata_elements()?;
    let worlds = world_count(dims.worlds)?;
    let rhs = rhs_elements(static_dims, dims.worlds, dims.rhs_count)?;
    let factor = checked_buffer(
        8,
        worlds,
        static_dims.factor_stride()?,
        "resident_mass_factor",
    )?;
    let output = checked_buffer(
        8,
        worlds,
        static_dims.solve_stride(dims.rhs_count)?,
        "resident_mass_output",
    )?;
    Ok([metadata, rhs, factor, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMassFactorSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻质量分解ABI。
unsafe impl KernelAbi for ResidentMassFactorSpec {
    const NAME: &'static str = "resident_mass_factor";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentMassFactorSpec {
    type Dims = ResidentMassFactorDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(resident_mass_factor_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_mass_factor_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMassSolveSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻质量回代ABI。
unsafe impl KernelAbi for ResidentMassSolveSpec {
    const NAME: &'static str = "resident_mass_substitute";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentMassSolveSpec {
    type Dims = ResidentMassSolveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(resident_mass_solve_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_mass_solve_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn exact_lengths(bindings: &[SlotBinding], lengths: [u64; 4]) -> Result<(), LauncherError> {
    for (slot, elements) in lengths.into_iter().enumerate() {
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        let required = elements
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("resident_mass_bytes"))?;
        if binding.span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: binding.span.bytes(),
            });
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mass_factor_plan(
    dims: &ResidentMassFactorDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ResidentMassFactorDims>],
) -> Result<(), LauncherError> {
    Kernel::<ResidentMassFactorSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, resident_mass_factor_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mass_solve_plan(
    dims: &ResidentMassSolveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ResidentMassSolveDims>],
) -> Result<(), LauncherError> {
    Kernel::<ResidentMassSolveSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, resident_mass_solve_lengths(dims)?)
}
