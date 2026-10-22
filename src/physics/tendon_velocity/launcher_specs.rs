//! 肌腱速度两类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据元素数：前后各四个守卫，中间两段行偏移与列索引。
#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_velocity_metadata_elements(
    dims: &TendonVelocityStaticDims,
) -> Result<u64, LauncherError> {
    if dims.nv == 0 || dims.nt == 0 {
        return Err(LauncherError::InvalidArray("肌腱速度维度无效"));
    }
    let nt = u64::from(dims.nt);
    let nnz = u64::from(dims.nnz);
    let minimum_stride = nt
        .checked_add(nnz)
        .ok_or(LauncherError::Overflow("tendon_velocity_stride"))?;
    if u64::from(dims.stride) < minimum_stride {
        return Err(LauncherError::InvalidArray("肌腱速度跨度无效"));
    }
    nt.checked_mul(2)
        .and_then(|value| value.checked_add(nnz))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("tendon_velocity_metadata"))
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("肌腱速度世界数无效"));
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
pub(super) fn tendon_velocity_validate_lengths(
    dims: &TendonVelocityValidateDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = tendon_velocity_metadata_elements(&static_dims)?;
    let worlds = world_count(dims.worlds)?;
    let jacobian = checked_buffer(
        8,
        worlds,
        u64::from(static_dims.stride),
        "tendon_velocity_jacobian",
    )?;
    let qvel = checked_buffer(8, worlds, u64::from(static_dims.nv), "tendon_velocity_qvel")?;
    let status = worlds
        .checked_add(8)
        .ok_or(LauncherError::Overflow("tendon_velocity_status"))?;
    Ok([metadata, jacobian, qvel, status])
}

/// 计算内核四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_velocity_compute_lengths(
    dims: &TendonVelocityComputeDims,
) -> Result<[u64; 4], LauncherError> {
    dims.abi()?;
    let static_dims = dims.static_dims;
    let metadata = tendon_velocity_metadata_elements(&static_dims)?;
    let worlds = world_count(dims.worlds)?;
    let jacobian = checked_buffer(
        8,
        worlds,
        u64::from(static_dims.stride),
        "tendon_velocity_jacobian",
    )?;
    let qvel = checked_buffer(8, worlds, u64::from(static_dims.nv), "tendon_velocity_qvel")?;
    let staged = checked_buffer(
        8,
        worlds,
        u64::from(static_dims.nt),
        "tendon_velocity_staged",
    )?;
    Ok([metadata, jacobian, qvel, staged])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结肌腱速度验证ABI。
unsafe impl KernelAbi for TendonVelocityValidateSpec {
    const NAME: &'static str = "tendon_velocity_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for TendonVelocityValidateSpec {
    type Dims = TendonVelocityValidateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(tendon_velocity_validate_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        tendon_velocity_validate_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityComputeSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结肌腱速度计算ABI。
unsafe impl KernelAbi for TendonVelocityComputeSpec {
    const NAME: &'static str = "tendon_velocity_compute";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for TendonVelocityComputeSpec {
    type Dims = TendonVelocityComputeDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(tendon_velocity_compute_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        tendon_velocity_compute_lengths(dims)?;
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
            .ok_or(LauncherError::Overflow("tendon_velocity_bytes"))?;
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
pub(super) fn tendon_velocity_validate_plan(
    dims: &TendonVelocityValidateDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, TendonVelocityValidateDims>],
) -> Result<(), LauncherError> {
    Kernel::<TendonVelocityValidateSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, tendon_velocity_validate_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn tendon_velocity_compute_plan(
    dims: &TendonVelocityComputeDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, TendonVelocityComputeDims>],
) -> Result<(), LauncherError> {
    Kernel::<TendonVelocityComputeSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, tendon_velocity_compute_lengths(dims)?)
}
