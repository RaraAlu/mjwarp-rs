//! 混合因子合并内核封装与ABI。

#[cfg(feature = "cuda-probe")]
use crate::diagnostics::{InputError, TransferError};
#[cfg(feature = "cuda-probe")]
use crate::model::SparseMassInput;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    SlotBinding, SlotDesc, SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;

#[cfg(any(feature = "cuda-probe", test))]
const MIXED_FACTOR_MERGE_SLOTS: [SlotDesc; 8] = [
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

#[cfg(any(feature = "cuda-probe", test))]
struct MixedFactorMergeSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐项匹配冻结混合因子合并ABI。
unsafe impl KernelAbi for MixedFactorMergeSpec {
    const NAME: &'static str = "mixed_factor_merge";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &MIXED_FACTOR_MERGE_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for MixedFactorMergeSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let lengths = mixed_factor_merge_lengths(dims)?;
        Ok(lengths
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        mixed_factor_merge_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn mixed_factor_merge_lengths(dims: &[u32; 4]) -> Result<[u64; 4], LauncherError> {
    let [prefix, nc, nv, work] = *dims;
    let tail = nc
        .checked_add(nv)
        .ok_or(LauncherError::Overflow("mixed_factor_merge_tail"))?;
    let stride = prefix
        .checked_add(tail)
        .filter(|&value| value != 0 && value <= i32::MAX as u32)
        .ok_or(LauncherError::InvalidArray("混合因子合并步长无效"))?;
    if nc == 0 || nv == 0 || work == 0 || work > u32::MAX - 255 || !work.is_multiple_of(stride) {
        return Err(LauncherError::InvalidArray("混合因子合并维度无效"));
    }
    let worlds = u64::from(work / stride);
    let guarded = |elements: u64, field| {
        elements
            .checked_add(8)
            .ok_or(LauncherError::Overflow(field))
    };
    Ok([
        1,
        guarded(
            worlds
                .checked_mul(u64::from(prefix))
                .ok_or(LauncherError::Overflow("mixed_factor_merge_small"))?,
            "mixed_factor_merge_small",
        )?,
        guarded(
            worlds
                .checked_mul(u64::from(tail))
                .ok_or(LauncherError::Overflow("mixed_factor_merge_sparse"))?,
            "mixed_factor_merge_sparse",
        )?,
        guarded(u64::from(work), "mixed_factor_merge_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn mixed_factor_merge_plan(
    expected: [u32; 3],
    dims: &[u32; 4],
    bindings: &[SlotBinding; 4],
) -> Result<LaunchDims, LauncherError> {
    if dims[..3] != expected {
        return Err(LauncherError::InvalidArray("混合因子合并静态维度不匹配"));
    }
    let lengths = mixed_factor_merge_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("mixed_factor_merge_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("混合因子合并跨度不符"));
        }
    }
    let plan = Kernel::<MixedFactorMergeSpec>::new()?.plan(dims, bindings, &[])?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
fn mixed_factor_merge_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mixed_factor_merge_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mixed_factor_merge_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct MixedFactorMergeKernel {
    kernel: SynchronousKernel,
    prefix: u32,
    nc: u32,
    nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl MixedFactorMergeKernel {
    pub(super) fn compile(
        session: &TransferSession,
        input: &SparseMassInput,
    ) -> Result<Self, TransferError> {
        let prefix = u32::try_from(input.qld_block_total()).map_err(|_| InputError::Overflow {
            field: "mixed_factor_merge_prefix",
        })?;
        let nc = u32::try_from(input.nc()).map_err(|_| InputError::Overflow {
            field: "mixed_factor_merge_nc",
        })?;
        let nv = u32::try_from(input.nv()).map_err(|_| InputError::Overflow {
            field: "mixed_factor_merge_nv",
        })?;
        let work = prefix
            .checked_add(nc)
            .and_then(|value| value.checked_add(nv))
            .ok_or(InputError::Overflow {
                field: "mixed_factor_merge_stride",
            })?;
        mixed_factor_merge_lengths(&[prefix, nc, nv, work])
            .map_err(mixed_factor_merge_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "mixed_factor_merge")?,
            prefix,
            nc,
            nv,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &TransferBuffer<i32>,
        small: &TransferBuffer<f32>,
        sparse: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "mixed_factor_merge_worlds",
        })?;
        let stride = self
            .prefix
            .checked_add(self.nc)
            .and_then(|value| value.checked_add(self.nv))
            .ok_or(InputError::Overflow {
                field: "mixed_factor_merge_stride",
            })?;
        let work = worlds.checked_mul(stride).ok_or(InputError::Overflow {
            field: "mixed_factor_merge_work",
        })?;
        let dims = [self.prefix, self.nc, self.nv, work];
        let lengths = mixed_factor_merge_lengths(&dims).map_err(mixed_factor_merge_launch_error)?;
        if [metadata.len(), small.len(), sparse.len(), output.len()]
            .into_iter()
            .zip(lengths)
            .any(|(actual, expected)| u64::try_from(actual).ok() != Some(expected))
        {
            return Err(InputError::InvalidDimension {
                field: "mixed_factor_merge_abi",
            }
            .into());
        }
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: small.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: sparse.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        mixed_factor_merge_plan([self.prefix, self.nc, self.nv], &dims, &bindings)
            .map_err(mixed_factor_merge_launch_error)?;
        // SAFETY: 类型化计划核对实际跨度、会话和写入别名。
        unsafe { self.kernel.launch(metadata, small, sparse, output, dims) }
    }
}
