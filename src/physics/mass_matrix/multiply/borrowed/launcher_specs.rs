//! 借用质量矩阵两类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("借用质量矩阵世界数无效"));
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
pub(super) fn borrowed_mass_validate_lengths(
    dims: &BorrowedMassValidateDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements(dims.worlds)?;
    let worlds = world_count(dims.worlds)?;
    let source = checked_buffer(8, worlds, static_dims.stride()?, "borrowed_mass_source")?;
    let vector = checked_buffer(8, worlds, u64::from(static_dims.n), "borrowed_mass_vector")?;
    let status = worlds
        .checked_add(8)
        .ok_or(LauncherError::Overflow("borrowed_mass_status"))?;
    Ok([metadata, source, vector, status])
}

/// 计算内核四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn borrowed_mass_compute_lengths(
    dims: &BorrowedMassComputeDims,
) -> Result<[u64; 4], LauncherError> {
    dims.abi()?;
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements(dims.worlds)?;
    let worlds = world_count(dims.worlds)?;
    let source = checked_buffer(8, worlds, static_dims.stride()?, "borrowed_mass_source")?;
    let vector = checked_buffer(8, worlds, u64::from(static_dims.n), "borrowed_mass_vector")?;
    let staged = checked_buffer(8, worlds, u64::from(static_dims.n), "borrowed_mass_staged")?;
    Ok([metadata, source, vector, staged])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct BorrowedMassValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结借用质量矩阵验证ABI。
unsafe impl KernelAbi for BorrowedMassValidateSpec {
    const NAME: &'static str = "borrowed_mass_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for BorrowedMassValidateSpec {
    type Dims = BorrowedMassValidateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(borrowed_mass_validate_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        borrowed_mass_validate_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct BorrowedMassComputeSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结借用质量矩阵计算ABI。
unsafe impl KernelAbi for BorrowedMassComputeSpec {
    const NAME: &'static str = "borrowed_mass_multiply";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for BorrowedMassComputeSpec {
    type Dims = BorrowedMassComputeDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(borrowed_mass_compute_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        borrowed_mass_compute_lengths(dims)?;
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
            .ok_or(LauncherError::Overflow("borrowed_mass_bytes"))?;
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
pub(super) fn borrowed_mass_validate_plan(
    dims: &BorrowedMassValidateDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, BorrowedMassValidateDims>],
) -> Result<(), LauncherError> {
    Kernel::<BorrowedMassValidateSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, borrowed_mass_validate_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn borrowed_mass_compute_plan(
    dims: &BorrowedMassComputeDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, BorrowedMassComputeDims>],
) -> Result<(), LauncherError> {
    Kernel::<BorrowedMassComputeSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, borrowed_mass_compute_lengths(dims)?)
}
