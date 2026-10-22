//! 重力补偿四类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::CLEAR_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::FIVE_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityClearDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravitySumDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("重力补偿世界数无效"));
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

#[cfg(feature = "cuda-probe")]
fn rigid_elements(static_dims: GravityStaticDims, worlds: u32) -> Result<u64, LauncherError> {
    checked_buffer(
        8,
        world_count(worlds)?,
        static_dims.rigid_stride()?,
        "gravity_compensation_rigid",
    )
}

#[cfg(feature = "cuda-probe")]
fn com_elements(static_dims: GravityStaticDims, worlds: u32) -> Result<u64, LauncherError> {
    checked_buffer(
        8,
        world_count(worlds)?,
        static_dims.com_stride()?,
        "gravity_compensation_com",
    )
}

#[cfg(feature = "cuda-probe")]
fn output_elements(static_dims: GravityStaticDims, worlds: u32) -> Result<u64, LauncherError> {
    checked_buffer(
        8,
        world_count(worlds)?,
        u64::from(static_dims.nv),
        "gravity_compensation_output",
    )
}

#[cfg(feature = "cuda-probe")]
fn status_elements(worlds: u32) -> Result<u64, LauncherError> {
    world_count(worlds)?;
    u64::from(worlds)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("gravity_compensation_status"))
}

/// 清零内核四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn gravity_clear_lengths(dims: &GravityClearDims) -> Result<[u64; 4], LauncherError> {
    dims.abi()?;
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = static_dims.parameters_elements()?;
    let output = output_elements(static_dims, dims.worlds)?;
    Ok([metadata, parameters, parameters, output])
}

/// 验证内核五槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn gravity_validate_lengths(
    dims: &GravityValidateDims,
) -> Result<[u64; 5], LauncherError> {
    world_count(dims.worlds)?;
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = static_dims.parameters_elements()?;
    let rigid = rigid_elements(static_dims, dims.worlds)?;
    let com = com_elements(static_dims, dims.worlds)?;
    let status = status_elements(dims.worlds)?;
    Ok([metadata, parameters, rigid, com, status])
}

/// 计算内核五槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn gravity_compute_lengths(
    dims: &GravityComputeDims,
) -> Result<[u64; 5], LauncherError> {
    dims.abi()?;
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = static_dims.parameters_elements()?;
    let rigid = rigid_elements(static_dims, dims.worlds)?;
    let com = com_elements(static_dims, dims.worlds)?;
    let output = output_elements(static_dims, dims.worlds)?;
    Ok([metadata, parameters, rigid, com, output])
}

/// 合并内核五槽精确元素数。
///
/// 分片缓冲按两字段读取（弹簧与阻尼各一世界段）；
/// 重力缓冲与输出最小宽度为单字段，输出可为四字段平滑力缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) fn gravity_sum_lengths(dims: &GravitySumDims) -> Result<[u64; 5], LauncherError> {
    dims.abi()?;
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let gravity = output_elements(static_dims, dims.worlds)?;
    let parts = world_count(dims.worlds)?
        .checked_mul(
            u64::from(static_dims.nv)
                .checked_mul(2)
                .ok_or(LauncherError::Overflow("gravity_compensation_parts"))?,
        )
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("gravity_compensation_parts"))?;
    Ok([metadata, parts, gravity, gravity, gravity])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityClearSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结重力补偿清零ABI。
unsafe impl KernelAbi for GravityClearSpec {
    const NAME: &'static str = "gravity_compensation_clear";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &CLEAR_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for GravityClearSpec {
    type Dims = GravityClearDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(gravity_clear_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        gravity_clear_lengths(dims)?;
        let abi = dims.abi()?;
        LaunchDims::new((abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结重力补偿验证ABI。
unsafe impl KernelAbi for GravityValidateSpec {
    const NAME: &'static str = "gravity_compensation_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &FIVE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for GravityValidateSpec {
    type Dims = GravityValidateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(gravity_validate_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        gravity_validate_lengths(dims)?;
        LaunchDims::new((dims.abi()[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityComputeSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结重力补偿计算ABI。
unsafe impl KernelAbi for GravityComputeSpec {
    const NAME: &'static str = "gravity_compensation_compute";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &FIVE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for GravityComputeSpec {
    type Dims = GravityComputeDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(gravity_compute_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        let abi = dims.abi()?;
        gravity_compute_lengths(dims)?;
        LaunchDims::new((abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravitySumSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结重力补偿合并ABI。
unsafe impl KernelAbi for GravitySumSpec {
    const NAME: &'static str = "gravity_compensation_sum";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &FIVE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for GravitySumSpec {
    type Dims = GravitySumDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(gravity_sum_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        let abi = dims.abi()?;
        gravity_sum_lengths(dims)?;
        LaunchDims::new((abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn exact_lengths_selected(
    bindings: &[SlotBinding],
    lengths: &[u64],
    slots: &[usize],
) -> Result<(), LauncherError> {
    for &slot in slots {
        let elements = lengths[slot];
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        let required = elements
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("gravity_compensation_bytes"))?;
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
pub(super) fn gravity_clear_plan(
    dims: &GravityClearDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, GravityClearDims>],
) -> Result<(), LauncherError> {
    Kernel::<GravityClearSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths_selected(bindings, &gravity_clear_lengths(dims)?, &[0, 1, 2, 3])
}

#[cfg(feature = "cuda-probe")]
pub(super) fn gravity_validate_plan(
    dims: &GravityValidateDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, GravityValidateDims>],
) -> Result<(), LauncherError> {
    Kernel::<GravityValidateSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths_selected(bindings, &gravity_validate_lengths(dims)?, &[0, 1, 2, 3, 4])
}

#[cfg(feature = "cuda-probe")]
pub(super) fn gravity_compute_plan(
    dims: &GravityComputeDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, GravityComputeDims>],
) -> Result<(), LauncherError> {
    Kernel::<GravityComputeSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths_selected(bindings, &gravity_compute_lengths(dims)?, &[0, 1, 2, 3, 4])
}

#[cfg(feature = "cuda-probe")]
pub(super) fn gravity_sum_plan(
    dims: &GravitySumDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, GravitySumDims>],
) -> Result<(), LauncherError> {
    Kernel::<GravitySumSpec>::new()?.plan(dims, bindings, proofs)?;
    // 输出槽可为四字段平滑力缓冲：内核只写前worlds*nv槽，按最小容量校对。
    exact_lengths_selected(bindings, &gravity_sum_lengths(dims)?, &[0, 1, 2, 3])
}
