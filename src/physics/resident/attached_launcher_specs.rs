//! 常驻附着运动学两类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::ResidentAttachedDims;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻附着世界数无效"));
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

/// 附着两类内核共用的四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn resident_attached_lengths(
    dims: &ResidentAttachedDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = u64::from(static_dims.parameters_len).max(1);
    let worlds = world_count(dims.worlds)?;
    let body = checked_buffer(
        8,
        worlds,
        static_dims.body_stride()?,
        "resident_attached_body",
    )?;
    let output = checked_buffer(
        8,
        worlds,
        static_dims.output_stride()?,
        "resident_attached_output",
    )?;
    Ok([metadata, parameters, body, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentAttachedSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻附着ABI。
unsafe impl KernelAbi for ResidentAttachedSpec {
    const NAME: &'static str = "attached_kinematics";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentAttachedSpec {
    type Dims = ResidentAttachedDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(resident_attached_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_attached_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentInitializeAttachedSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻附着初始化ABI。
unsafe impl KernelAbi for ResidentInitializeAttachedSpec {
    const NAME: &'static str = "initialize_attached";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentInitializeAttachedSpec {
    type Dims = ResidentAttachedDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(resident_attached_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_attached_lengths(dims)?;
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
            .ok_or(LauncherError::Overflow("resident_attached_bytes"))?;
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
pub(super) fn resident_attached_plan(
    dims: &ResidentAttachedDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ResidentAttachedDims>],
) -> Result<(), LauncherError> {
    Kernel::<ResidentAttachedSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, resident_attached_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_initialize_attached_plan(
    dims: &ResidentAttachedDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ResidentAttachedDims>],
) -> Result<(), LauncherError> {
    Kernel::<ResidentInitializeAttachedSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, resident_attached_lengths(dims)?)
}
