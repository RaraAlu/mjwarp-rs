//! 质心位置规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComPositionDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("质心位置世界数无效"));
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

/// 质心位置四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn com_position_lengths(dims: &ComPositionDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = static_dims.parameters_elements()?;
    let worlds = world_count(dims.worlds)?;
    let state = checked_buffer(8, worlds, static_dims.state_stride()?, "com_position_state")?;
    let output = checked_buffer(
        8,
        worlds,
        static_dims.output_stride()?,
        "com_position_output",
    )?;
    Ok([metadata, parameters, state, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ComPositionSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结质心位置ABI。
unsafe impl KernelAbi for ComPositionSpec {
    const NAME: &'static str = "rigid_com_position";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ComPositionSpec {
    type Dims = ComPositionDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(com_position_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        com_position_lengths(dims)?;
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
            .ok_or(LauncherError::Overflow("com_position_bytes"))?;
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
pub(super) fn com_position_plan(
    dims: &ComPositionDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ComPositionDims>],
) -> Result<(), LauncherError> {
    Kernel::<ComPositionSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, com_position_lengths(dims)?)
}
