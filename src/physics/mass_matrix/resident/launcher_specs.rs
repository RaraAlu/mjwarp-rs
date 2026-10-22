//! 常驻CRB三类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbAccumulateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbInitDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbMatrixDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻CRB世界数无效"));
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

/// 三类内核共用的四槽精确元素数。
#[cfg(feature = "cuda-probe")]
fn resident_crb_lengths(
    static_dims: ResidentCrbStaticDims,
    worlds: u32,
) -> Result<[u64; 4], LauncherError> {
    let metadata = static_dims.metadata_elements()?;
    let worlds = world_count(worlds)?;
    let armature = static_dims.armature_elements();
    let state = checked_buffer(8, worlds, static_dims.state_stride()?, "resident_crb_state")?;
    let output = checked_buffer(
        8,
        worlds,
        static_dims.output_stride()?,
        "resident_crb_output",
    )?;
    Ok([metadata, armature, state, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_crb_init_lengths(
    dims: &ResidentCrbInitDims,
) -> Result<[u64; 4], LauncherError> {
    resident_crb_lengths(dims.static_dims, dims.worlds)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_crb_accumulate_lengths(
    dims: &ResidentCrbAccumulateDims,
) -> Result<[u64; 4], LauncherError> {
    if dims.level == 0 || dims.level > dims.static_dims.depth {
        return Err(LauncherError::InvalidArray("常驻CRB累加层级无效"));
    }
    resident_crb_lengths(dims.static_dims, dims.worlds)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_crb_matrix_lengths(
    dims: &ResidentCrbMatrixDims,
) -> Result<[u64; 4], LauncherError> {
    resident_crb_lengths(dims.static_dims, dims.worlds)
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbInitSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻CRB初始化ABI。
unsafe impl KernelAbi for ResidentCrbInitSpec {
    const NAME: &'static str = "resident_crb_init";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentCrbInitSpec {
    type Dims = ResidentCrbInitDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(resident_crb_init_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_crb_init_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbAccumulateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻CRB累加ABI。
unsafe impl KernelAbi for ResidentCrbAccumulateSpec {
    const NAME: &'static str = "resident_crb_accumulate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentCrbAccumulateSpec {
    type Dims = ResidentCrbAccumulateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(resident_crb_accumulate_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_crb_accumulate_lengths(dims)?;
        let abi = dims.abi()?;
        LaunchDims::new((abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbMatrixSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻CRB矩阵ABI。
unsafe impl KernelAbi for ResidentCrbMatrixSpec {
    const NAME: &'static str = "resident_crb_matrix";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentCrbMatrixSpec {
    type Dims = ResidentCrbMatrixDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(resident_crb_matrix_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_crb_matrix_lengths(dims)?;
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
            .ok_or(LauncherError::Overflow("resident_crb_bytes"))?;
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
pub(super) fn resident_crb_init_plan(
    dims: &ResidentCrbInitDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ResidentCrbInitDims>],
) -> Result<(), LauncherError> {
    Kernel::<ResidentCrbInitSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, resident_crb_init_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_crb_accumulate_plan(
    dims: &ResidentCrbAccumulateDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ResidentCrbAccumulateDims>],
) -> Result<(), LauncherError> {
    Kernel::<ResidentCrbAccumulateSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, resident_crb_accumulate_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_crb_matrix_plan(
    dims: &ResidentCrbMatrixDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, ResidentCrbMatrixDims>],
) -> Result<(), LauncherError> {
    Kernel::<ResidentCrbMatrixSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, resident_crb_matrix_lengths(dims)?)
}
