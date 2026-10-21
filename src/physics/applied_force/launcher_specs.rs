//! 施加外力三类规格与长度。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::APPLIED_FORCE_DELTA_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::APPLIED_FORCE_PUBLISH_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::APPLIED_FORCE_VALIDATE_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForceValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结外力验证ABI。
unsafe impl KernelAbi for AppliedForceValidateSpec {
    const NAME: &'static str = "applied_force_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &APPLIED_FORCE_VALIDATE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for AppliedForceValidateSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let [metadata, geometry, ft, status] = applied_force_validate_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: geometry,
            },
            SlotRequirement {
                slot: 2,
                min_elements: ft,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        applied_force_validate_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn applied_force_validate_lengths(dims: &[u32; 4]) -> Result<[u64; 4], LauncherError> {
    let [nb, n, worlds, work] = *dims;
    if nb == 0 || n == 0 || worlds == 0 || work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("外力验证维度无效"));
    }
    if work != worlds {
        return Err(LauncherError::InvalidArray("外力验证工作量不匹配"));
    }
    let nb = u64::from(nb);
    let n = u64::from(n);
    let worlds = u64::from(worlds);
    let metadata = nb
        .checked_mul(2)
        .and_then(|value| value.checked_add(n))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("applied_force_validate_metadata"))?;
    // 元数据上限也证明设备表达式中的nb+n不会发生u32回绕。
    let geometry = nb
        .checked_add(n)
        .and_then(|value| value.checked_mul(worlds))
        .and_then(|value| value.checked_mul(6))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("applied_force_validate_geometry"))?;
    // n非零，因此几何跨度成功后，较小的FT跨度不会溢出。
    let ft = nb
        .checked_mul(worlds)
        .and_then(|value| value.checked_mul(6))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("applied_force_validate_ft"))?;
    // work已受u32上界约束，因此状态跨度不会溢出u64。
    let status = u64::from(work)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("applied_force_validate_status"))?;
    Ok([metadata, geometry, ft, status])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForceDeltaSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结外力增量ABI。
unsafe impl KernelAbi for AppliedForceDeltaSpec {
    const NAME: &'static str = "applied_force_delta";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &APPLIED_FORCE_DELTA_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for AppliedForceDeltaSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let [metadata, geometry, ft, output] = applied_force_delta_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: geometry,
            },
            SlotRequirement {
                slot: 2,
                min_elements: ft,
            },
            SlotRequirement {
                slot: 3,
                min_elements: output,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        applied_force_delta_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn applied_force_delta_lengths(dims: &[u32; 4]) -> Result<[u64; 4], LauncherError> {
    let [nb, n, worlds, work] = *dims;
    if nb == 0 || n == 0 || worlds == 0 || work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("外力增量维度无效"));
    }
    if worlds.checked_mul(n) != Some(work) {
        return Err(LauncherError::InvalidArray("外力增量工作量不匹配"));
    }
    let nb = u64::from(nb);
    let n = u64::from(n);
    let worlds = u64::from(worlds);
    let metadata = nb
        .checked_mul(2)
        .and_then(|value| value.checked_add(n))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("applied_force_delta_metadata"))?;
    let geometry = nb
        .checked_add(n)
        .and_then(|value| value.checked_mul(worlds))
        .and_then(|value| value.checked_mul(6))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("applied_force_delta_geometry"))?;
    let ft = nb
        .checked_mul(worlds)
        .and_then(|value| value.checked_mul(6))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("applied_force_delta_ft"))?;
    let output = u64::from(work)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("applied_force_delta_output"))?;
    Ok([metadata, geometry, ft, output])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct AppliedForcePublishSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结外力发布ABI。
unsafe impl KernelAbi for AppliedForcePublishSpec {
    const NAME: &'static str = "applied_force_publish";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &APPLIED_FORCE_PUBLISH_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for AppliedForcePublishSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let [metadata, current, delta, staged] = applied_force_publish_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: current,
            },
            SlotRequirement {
                slot: 2,
                min_elements: delta,
            },
            SlotRequirement {
                slot: 3,
                min_elements: staged,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        applied_force_publish_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn applied_force_publish_lengths(dims: &[u32; 4]) -> Result<[u64; 4], LauncherError> {
    let [n, worlds, add, work] = *dims;
    if n == 0 || worlds == 0 || work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("外力发布维度无效"));
    }
    if add > 1 {
        return Err(LauncherError::InvalidArray("外力发布累加开关无效"));
    }
    if worlds.checked_mul(n) != Some(work) {
        return Err(LauncherError::InvalidArray("外力发布工作量不匹配"));
    }
    let vector = u64::from(work)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("applied_force_publish_vector"))?;
    // meta未参与CUDA寻址，但底层同步发射仍要求非空设备分配。
    Ok([1, vector, vector, vector])
}

#[cfg(feature = "cuda-probe")]
pub(super) fn applied_force_publish_plan(
    dims: &[u32; 4],
    bindings: &[SlotBinding],
) -> Result<(), LauncherError> {
    Kernel::<AppliedForcePublishSpec>::new()?.plan(dims, bindings, &[])?;
    let current = bindings
        .iter()
        .find(|binding| binding.slot == 1)
        .ok_or(LauncherError::MissingBinding { slot: 1 })?;
    let delta = bindings
        .iter()
        .find(|binding| binding.slot == 2)
        .ok_or(LauncherError::MissingBinding { slot: 2 })?;
    // 通用规则允许只读槽重叠；发布语义要求两份输入独立。
    if current.span.overlaps(&delta.span) {
        return Err(LauncherError::Alias {
            first: 1,
            second: 2,
        });
    }
    Ok(())
}
