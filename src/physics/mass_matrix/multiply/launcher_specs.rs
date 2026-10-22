//! 质量矩阵乘法两类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("质量矩阵乘法世界数无效"));
    }
    Ok(u64::from(worlds))
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

/// 验证内核四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn mass_multiply_validate_lengths(
    dims: &MassMultiplyValidateDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements(dims.worlds)?;
    let worlds = world_count(dims.worlds)?;
    let matrix = checked_buffer(8, worlds, u64::from(static_dims.mc), "mass_multiply_matrix")?;
    let vector = checked_buffer(8, worlds, u64::from(static_dims.n), "mass_multiply_vector")?;
    let status = worlds
        .checked_add(8)
        .ok_or(LauncherError::Overflow("mass_multiply_status"))?;
    Ok([metadata, matrix, vector, status])
}

/// 计算内核四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn mass_multiply_compute_lengths(
    dims: &MassMultiplyComputeDims,
) -> Result<[u64; 4], LauncherError> {
    dims.abi()?;
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements(dims.worlds)?;
    let worlds = world_count(dims.worlds)?;
    let matrix = checked_buffer(8, worlds, u64::from(static_dims.mc), "mass_multiply_matrix")?;
    let vector = checked_buffer(8, worlds, u64::from(static_dims.n), "mass_multiply_vector")?;
    let output = checked_buffer(8, worlds, u64::from(static_dims.n), "mass_multiply_output")?;
    Ok([metadata, matrix, vector, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct MassMultiplyValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结质量矩阵乘法验证ABI。
unsafe impl KernelAbi for MassMultiplyValidateSpec {
    const NAME: &'static str = "mass_multiply_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for MassMultiplyValidateSpec {
    type Dims = MassMultiplyValidateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(mass_multiply_validate_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        mass_multiply_validate_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct MassMultiplyComputeSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结质量矩阵乘法计算ABI。
unsafe impl KernelAbi for MassMultiplyComputeSpec {
    const NAME: &'static str = "mass_multiply";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for MassMultiplyComputeSpec {
    type Dims = MassMultiplyComputeDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(mass_multiply_compute_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        mass_multiply_compute_lengths(dims)?;
        let abi = dims.abi()?;
        LaunchDims::new((abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
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
            .ok_or(LauncherError::Overflow("mass_multiply_bytes"))?;
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
pub(super) fn mass_multiply_validate_plan(
    dims: &MassMultiplyValidateDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, MassMultiplyValidateDims>],
) -> Result<(), LauncherError> {
    Kernel::<MassMultiplyValidateSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, mass_multiply_validate_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn mass_multiply_compute_plan(
    dims: &MassMultiplyComputeDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, MassMultiplyComputeDims>],
) -> Result<(), LauncherError> {
    Kernel::<MassMultiplyComputeSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, mass_multiply_compute_lengths(dims)?)
}
