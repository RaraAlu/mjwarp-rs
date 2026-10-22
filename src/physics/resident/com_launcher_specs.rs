//! 常驻质心位置规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::com_launcher_abi::ResidentComDims;
#[cfg(feature = "cuda-probe")]
use super::com_launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻质心世界数无效"));
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

/// 常驻质心四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn resident_com_lengths(dims: &ResidentComDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = u64::from(static_dims.parameters_len).max(1);
    let worlds = world_count(dims.worlds)?;
    let state = checked_buffer(8, worlds, static_dims.state_stride()?, "resident_com_state")?;
    let output = checked_buffer(
        8,
        worlds,
        static_dims.output_stride()?,
        "resident_com_output",
    )?;
    Ok([metadata, parameters, state, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentComSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻质心FIELD_BATCH ABI。
unsafe impl KernelAbi for ResidentComSpec {
    const NAME: &'static str = "rigid_com_position";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentComSpec {
    type Dims = ResidentComDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(resident_com_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_com_lengths(dims)?;
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
            .ok_or(LauncherError::Overflow("resident_com_bytes"))?;
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
pub(super) fn resident_com_plan(
    dims: &ResidentComDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ResidentComDims>],
) -> Result<(), LauncherError> {
    Kernel::<ResidentComSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, resident_com_lengths(dims)?)
}
