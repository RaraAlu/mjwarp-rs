//! 空间速度规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComVelocityDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("空间速度世界数无效"));
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

/// 空间速度四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn com_velocity_lengths(dims: &ComVelocityDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let worlds = world_count(dims.worlds)?;
    let qvel = checked_buffer(8, worlds, u64::from(static_dims.nv), "com_velocity_qvel")?;
    let com = checked_buffer(8, worlds, static_dims.com_stride()?, "com_velocity_com")?;
    let output = checked_buffer(
        8,
        worlds,
        static_dims.output_stride()?,
        "com_velocity_output",
    )?;
    Ok([metadata, qvel, com, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ComVelocitySpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结空间速度ABI。
unsafe impl KernelAbi for ComVelocitySpec {
    const NAME: &'static str = "com_velocity";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ComVelocitySpec {
    type Dims = ComVelocityDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(com_velocity_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        com_velocity_lengths(dims)?;
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
            .ok_or(LauncherError::Overflow("com_velocity_bytes"))?;
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
pub(super) fn com_velocity_plan(
    dims: &ComVelocityDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ComVelocityDims>],
) -> Result<(), LauncherError> {
    Kernel::<ComVelocitySpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, com_velocity_lengths(dims)?)
}
