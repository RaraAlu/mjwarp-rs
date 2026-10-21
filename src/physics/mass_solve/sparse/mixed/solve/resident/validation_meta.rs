//! 常驻混合校验槽位、规格与错误映射。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) const RESIDENT_MIXED_VALIDATION_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Write,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMixedValidationSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结常驻混合校验ABI。
unsafe impl KernelAbi for ResidentMixedValidationSpec {
    const NAME: &'static str = "resident_mixed_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RESIDENT_MIXED_VALIDATION_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentMixedValidationSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (source, status) = resident_mixed_validation_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 1,
                min_elements: source,
            },
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
        resident_mixed_validation_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mixed_validation_lengths(
    dims: &[u32; 4],
) -> Result<(u64, u64), LauncherError> {
    let [nb, n, _nc, worlds] = *dims;
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻混合校验维度无效"));
    }
    let body = 10u64
        .checked_mul(u64::from(nb))
        .ok_or(LauncherError::Overflow("resident_mixed_validation_body"))?;
    let dense = u64::from(n)
        .checked_mul(u64::from(n))
        .ok_or(LauncherError::Overflow("resident_mixed_validation_dense"))?;
    let stride = body
        .checked_add(dense)
        .ok_or(LauncherError::Overflow("resident_mixed_validation_stride"))?;
    let source = stride
        .checked_mul(u64::from(worlds))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_mixed_validation_source"))?;
    let status = u64::from(worlds)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("resident_mixed_validation_status"))?;
    Ok((source, status))
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mixed_validation_plan(
    expected: [u32; 3],
    dims: &[u32; 4],
    bindings: &[SlotBinding; 4],
) -> Result<(), LauncherError> {
    if dims[..3] != expected {
        return Err(LauncherError::InvalidArray("常驻混合校验静态维度不匹配"));
    }
    let (source, status) = resident_mixed_validation_lengths(dims)?;
    let source_bytes = source.checked_mul(4).ok_or(LauncherError::Overflow(
        "resident_mixed_validation_source_bytes",
    ))?;
    let status_bytes = status.checked_mul(4).ok_or(LauncherError::Overflow(
        "resident_mixed_validation_status_bytes",
    ))?;
    if bindings[0].span.bytes() < 4
        || bindings[1].span.bytes() != source_bytes
        || bindings[2].span.bytes() < 4
        || bindings[3].span.bytes() != status_bytes
    {
        return Err(LauncherError::InvalidArray("常驻混合校验跨度无效"));
    }
    Kernel::<ResidentMixedValidationSpec>::new()?.plan(dims, bindings, &[])?;
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mixed_validation_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        _ => InputError::InvalidDimension {
            field: "resident_mixed_validation_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn resident_mixed_validation_dimension(
    value: usize,
    field: &'static str,
) -> Result<u32, TransferError> {
    u32::try_from(value).map_err(|_| InputError::Overflow { field }.into())
}
