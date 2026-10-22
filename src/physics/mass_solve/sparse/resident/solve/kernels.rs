//! 常驻稀疏求解两入口的规格与同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseSolveDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseSolveEntry;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseSolveMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::sparse_solve_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::sparse_solve_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
#[cfg(feature = "cuda-probe")]
use crate::runtime::SynchronousKernel;
#[cfg(feature = "cuda-probe")]
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    BlockRule, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    SlotBinding, SlotDesc, SlotRequirement,
};

#[cfg(feature = "cuda-probe")]
fn spans(dims: &SparseSolveDims) -> Result<Vec<SlotRequirement>, LauncherError> {
    Ok(sparse_solve_lengths(dims)?
        .into_iter()
        .enumerate()
        .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
        .collect())
}

#[cfg(feature = "cuda-probe")]
fn validate_launch_dims(dims: &SparseSolveDims) -> Result<LaunchDims, LauncherError> {
    sparse_solve_lengths(dims)?;
    let worlds = dims.worlds;
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("稀疏求解世界数无效"));
    }
    LaunchDims::new((worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
}

#[cfg(feature = "cuda-probe")]
fn fused_launch_dims(dims: &SparseSolveDims) -> Result<LaunchDims, LauncherError> {
    sparse_solve_lengths(dims)?;
    let groups = u64::from(dims.worlds)
        .checked_mul(u64::from(dims.rhs_count))
        .filter(|&value| value > 0 && value <= i32::MAX as u64)
        .ok_or(LauncherError::InvalidArray("稀疏求解分组数无效"))?;
    LaunchDims::new((groups as u32, 1, 1), (128, 1, 1), 0)
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
            .ok_or(LauncherError::Overflow("sparse_solve_bytes"))?;
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
fn plan_with_spec<S: KernelSpec<Dims = SparseSolveDims>>(
    dims: &SparseSolveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, SparseSolveDims>],
) -> Result<(), LauncherError> {
    Kernel::<S>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, sparse_solve_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseSolveValidateSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结sparse_solve_validate四缓冲ABI。
unsafe impl KernelAbi for SparseSolveValidateSpec {
    const NAME: &'static str = "sparse_solve_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for SparseSolveValidateSpec {
    type Dims = SparseSolveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        validate_launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseSolveFusedSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结sparse_solve_fused四缓冲ABI。
unsafe impl KernelAbi for SparseSolveFusedSpec {
    const NAME: &'static str = "sparse_solve_fused";
    const BLOCK: BlockRule = BlockRule::Fixed(128);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for SparseSolveFusedSpec {
    type Dims = SparseSolveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        fused_launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_validate(
    dims: &SparseSolveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, SparseSolveDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<SparseSolveValidateSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_fused(
    dims: &SparseSolveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, SparseSolveDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<SparseSolveFusedSpec>(dims, bindings, proofs)
}

/// 常驻稀疏求解两入口同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct SparseSolveKernels {
    validate: SynchronousKernel,
    fused: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl SparseSolveKernels {
    pub(in crate::physics) fn compile(
        session: &TransferSession,
        code: &str,
    ) -> Result<Self, TransferError> {
        Ok(Self {
            validate: SynchronousKernel::compile(session, code, "sparse_solve_validate")?,
            fused: SynchronousKernel::compile(session, code, "sparse_solve_fused")?,
        })
    }

    fn bindings(
        metadata: &SparseSolveMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 4], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: rhs.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: factor.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    fn dims(
        metadata: &SparseSolveMetadataBuffer,
        worlds: u32,
        rhs_count: u32,
        entry: SparseSolveEntry,
    ) -> SparseSolveDims {
        SparseSolveDims {
            static_dims: metadata.static_dims(),
            worlds,
            rhs_count,
            entry,
        }
    }

    /// 验证段：每世界检查因子守卫、主元与右端项。
    pub(in crate::physics) fn launch_validate(
        &self,
        metadata: &SparseSolveMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
        rhs_count: u32,
    ) -> Result<(), TransferError> {
        let dims = Self::dims(metadata, worlds, rhs_count, SparseSolveEntry::Validate);
        let proof = metadata.proof(dims)?;
        let bindings = Self::bindings(metadata, rhs, factor, status)?;
        plan_validate(&dims, &bindings, &[proof.binding(0)]).map_err(sparse_solve_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与256线程网格。
        unsafe {
            self.validate.launch_sparse_solve_validate(
                metadata.as_buffer(),
                rhs,
                factor,
                status,
                dims.abi(),
            )
        }
    }

    /// 融合段：每世界每右端项一个128线程块，层间同步。
    pub(in crate::physics) fn launch_fused(
        &self,
        metadata: &SparseSolveMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
        rhs_count: u32,
    ) -> Result<(), TransferError> {
        let dims = Self::dims(metadata, worlds, rhs_count, SparseSolveEntry::Fused);
        let proof = metadata.proof(dims)?;
        let bindings = Self::bindings(metadata, rhs, factor, output)?;
        plan_fused(&dims, &bindings, &[proof.binding(0)]).map_err(sparse_solve_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与128线程网格。
        unsafe {
            self.fused
                .launch_sparse_solve(metadata.as_buffer(), rhs, factor, output, dims.abi())
        }
    }
}
