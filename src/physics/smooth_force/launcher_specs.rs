//! 平滑力两类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SMOOTH_FORCE_COMPUTE_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SMOOTH_FORCE_VALIDATE_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceValidateDims;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SmoothForceValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结平滑力验证ABI。
unsafe impl KernelAbi for SmoothForceValidateSpec {
    const NAME: &'static str = "smooth_force_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SMOOTH_FORCE_VALIDATE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SmoothForceValidateSpec {
    type Dims = SmoothForceValidateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let [metadata, forces, _, status] = smooth_force_validate_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: forces,
            },
            // 关闭休眠时内核不读载荷，但底层ABI仍要求非空分配。
            SlotRequirement {
                slot: 2,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        smooth_force_validate_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn smooth_force_validate_lengths(
    dims: &SmoothForceValidateDims,
) -> Result<[u64; 4], LauncherError> {
    let [nb, n, nt, worlds] = dims.abi;
    if nb == 0 || n == 0 || worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("平滑力验证维度无效"));
    }
    let nb = u64::from(nb);
    let n = u64::from(n);
    let nt = u64::from(nt);
    let worlds = u64::from(worlds);
    let metadata = worlds
        .checked_mul(nt)
        .and_then(|value| value.checked_add(nb))
        .and_then(|value| value.checked_add(n))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("smooth_force_validate_metadata"))?;
    let forces = worlds
        .checked_mul(n)
        .and_then(|value| value.checked_mul(4))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("smooth_force_validate_forces"))?;
    let sleep = worlds
        .checked_mul(
            nt.checked_mul(2)
                .and_then(|value| value.checked_add(3))
                .ok_or(LauncherError::Overflow(
                    "smooth_force_validate_sleep_stride",
                ))?,
        )
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("smooth_force_validate_sleep"))?;
    let status = worlds
        .checked_add(8)
        .ok_or(LauncherError::Overflow("smooth_force_validate_status"))?;
    Ok([metadata, forces, sleep, status])
}

#[cfg(feature = "cuda-probe")]
pub(super) fn smooth_force_validate_plan(
    dims: &SmoothForceValidateDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, SmoothForceValidateDims>],
) -> Result<(), LauncherError> {
    Kernel::<SmoothForceValidateSpec>::new()?.plan(dims, bindings, proofs)?;
    let lengths = smooth_force_validate_lengths(dims)?;
    let binding = |slot| {
        bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })
    };
    for slot in [0usize, 1, 3] {
        let actual = binding(slot)?.span.bytes();
        let required = lengths[slot]
            .checked_mul(binding(slot)?.elem.bytes())
            .ok_or(LauncherError::Overflow("smooth_force_validate_exact_bytes"))?;
        if actual != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual,
            });
        }
    }
    let metadata = binding(0)?;
    let forces = binding(1)?;
    let sleep = binding(2)?;
    let exact_metadata_alias = metadata.span == sleep.span;
    if dims.sleep_enabled && !exact_metadata_alias {
        let required = lengths[2]
            .checked_mul(sleep.elem.bytes())
            .ok_or(LauncherError::Overflow("smooth_force_validate_sleep_bytes"))?;
        if sleep.span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot: 2,
                required,
                actual: sleep.span.bytes(),
            });
        }
    }
    for (first, a, second, b) in [
        (0, metadata, 1, forces),
        (0, metadata, 2, sleep),
        (1, forces, 2, sleep),
    ] {
        if a.span.overlaps(&b.span) && !(first == 0 && second == 2 && exact_metadata_alias) {
            return Err(LauncherError::Alias { first, second });
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SmoothForceComputeSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结平滑力计算ABI。
unsafe impl KernelAbi for SmoothForceComputeSpec {
    const NAME: &'static str = "smooth_force";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SMOOTH_FORCE_COMPUTE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SmoothForceComputeSpec {
    type Dims = SmoothForceComputeDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let [metadata, forces, _, output] = smooth_force_compute_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: forces,
            },
            // 关闭休眠时内核不读载荷，但底层ABI仍要求非空分配。
            SlotRequirement {
                slot: 2,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 3,
                min_elements: output,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        smooth_force_compute_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn smooth_force_compute_worlds(
    dims: &SmoothForceComputeDims,
) -> Result<u32, LauncherError> {
    let [nb, n, _, work] = dims.abi;
    if nb == 0 || n == 0 || work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("平滑力计算维度无效"));
    }
    if !work.is_multiple_of(n) {
        return Err(LauncherError::InvalidArray("平滑力计算工作量不匹配"));
    }
    Ok(work / n)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn smooth_force_compute_lengths(
    dims: &SmoothForceComputeDims,
) -> Result<[u64; 4], LauncherError> {
    let [nb, n, nt, work] = dims.abi;
    let worlds = smooth_force_compute_worlds(dims)?;
    let nb = u64::from(nb);
    let n = u64::from(n);
    let nt = u64::from(nt);
    let work = u64::from(work);
    let worlds = u64::from(worlds);
    let metadata = worlds
        .checked_mul(nt)
        .and_then(|value| value.checked_add(nb))
        .and_then(|value| value.checked_add(n))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("smooth_force_compute_metadata"))?;
    let forces = work
        .checked_mul(4)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("smooth_force_compute_forces"))?;
    let sleep = worlds
        .checked_mul(
            nt.checked_mul(2)
                .and_then(|value| value.checked_add(3))
                .ok_or(LauncherError::Overflow("smooth_force_compute_sleep_stride"))?,
        )
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("smooth_force_compute_sleep"))?;
    let output = work
        .checked_add(8)
        .ok_or(LauncherError::Overflow("smooth_force_compute_output"))?;
    Ok([metadata, forces, sleep, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) fn smooth_force_compute_plan(
    dims: &SmoothForceComputeDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, SmoothForceComputeDims>],
) -> Result<(), LauncherError> {
    Kernel::<SmoothForceComputeSpec>::new()?.plan(dims, bindings, proofs)?;
    let lengths = smooth_force_compute_lengths(dims)?;
    let binding = |slot| {
        bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })
    };
    for slot in [0usize, 1, 3] {
        let actual = binding(slot)?.span.bytes();
        let required = lengths[slot]
            .checked_mul(binding(slot)?.elem.bytes())
            .ok_or(LauncherError::Overflow("smooth_force_compute_exact_bytes"))?;
        if actual != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual,
            });
        }
    }
    let metadata = binding(0)?;
    let forces = binding(1)?;
    let sleep = binding(2)?;
    let exact_metadata_alias = metadata.span == sleep.span;
    if dims.sleep_enabled && !exact_metadata_alias {
        let required = lengths[2]
            .checked_mul(sleep.elem.bytes())
            .ok_or(LauncherError::Overflow("smooth_force_compute_sleep_bytes"))?;
        if sleep.span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot: 2,
                required,
                actual: sleep.span.bytes(),
            });
        }
    }
    for (first, a, second, b) in [
        (0, metadata, 1, forces),
        (0, metadata, 2, sleep),
        (1, forces, 2, sleep),
    ] {
        if a.span.overlaps(&b.span) && !(first == 0 && second == 2 && exact_metadata_alias) {
            return Err(LauncherError::Alias { first, second });
        }
    }
    Ok(())
}
