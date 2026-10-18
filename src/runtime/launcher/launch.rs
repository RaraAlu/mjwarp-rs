//! W17及W05前置：发射计划和安全校验。

use super::{
    AbiSignature, BlockRule, CheckedBinding, KernelAbi, LauncherError, SlotBinding,
    SlotRequirement, SlotRole, validate_bindings,
};
use std::marker::PhantomData;

/// CUDA网格、线程块与共享内存。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchDims {
    grid: (u32, u32, u32),
    block: (u32, u32, u32),
    shared_bytes: u32,
}

impl LaunchDims {
    pub fn new(
        grid: (u32, u32, u32),
        block: (u32, u32, u32),
        shared_bytes: u32,
    ) -> Result<Self, LauncherError> {
        if [grid.0, grid.1, grid.2, block.0, block.1, block.2].contains(&0) {
            return Err(LauncherError::InvalidArray("发射网格包含零维"));
        }
        let threads = u64::from(block.0)
            .checked_mul(u64::from(block.1))
            .and_then(|value| value.checked_mul(u64::from(block.2)))
            .ok_or(LauncherError::Overflow("launch_block_threads"))?;
        if threads > 1024 {
            return Err(LauncherError::InvalidArray("线程块超过1024线程"));
        }
        Ok(Self {
            grid,
            block,
            shared_bytes,
        })
    }

    pub fn grid(&self) -> (u32, u32, u32) {
        self.grid
    }

    pub fn block(&self) -> (u32, u32, u32) {
        self.block
    }

    pub fn shared_bytes(&self) -> u32 {
        self.shared_bytes
    }

    fn validate_rule(&self, rule: BlockRule) -> Result<(), LauncherError> {
        let expected = rule.threads();
        if self.block != (expected, 1, 1) {
            return Err(LauncherError::InvalidArray("线程块违反ABI规则"));
        }
        Ok(())
    }
}

/// 安全包装所需的静态规则。
pub trait KernelSpec: KernelAbi {
    type Dims: Clone + Eq;

    /// 由维度推导最小元素数。
    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError>;

    /// 由维度推导发射网格。
    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError>;
}

/// 已通过宿主校验的静态计划。
///
/// 此类型不能执行GPU工作。
pub struct LaunchPlan<K: KernelSpec> {
    dims: LaunchDims,
    marker: PhantomData<K>,
}

impl<K: KernelSpec> std::fmt::Debug for LaunchPlan<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LaunchPlan")
            .field("dims", &self.dims)
            .finish()
    }
}

impl<K: KernelSpec> LaunchPlan<K> {
    pub fn dims(&self) -> LaunchDims {
        self.dims
    }
}

/// 一个内核的安全宿主包装。
pub struct Kernel<K: KernelSpec> {
    signature: AbiSignature,
    marker: PhantomData<K>,
}

impl<K: KernelSpec> Kernel<K> {
    pub fn new() -> Result<Self, LauncherError> {
        Ok(Self {
            signature: AbiSignature::of::<K>()?,
            marker: PhantomData,
        })
    }

    /// 校验跨度、别名、上下文与元数据。
    pub fn plan(
        &self,
        dims: &K::Dims,
        bindings: &[SlotBinding],
        proofs: &[CheckedBinding<'_, K::Dims>],
    ) -> Result<LaunchPlan<K>, LauncherError> {
        let launch_dims = K::launch_dims(dims)?;
        launch_dims.validate_rule(self.signature.block())?;
        let requirements = K::spans(dims)?;
        validate_bindings(&self.signature, bindings, &requirements)?;
        let mut used = vec![false; proofs.len()];
        for desc in self.signature.slots() {
            let SlotRole::AddressBearing { proof: required } = desc.role else {
                continue;
            };
            let binding = bindings
                .iter()
                .find(|binding| binding.slot == desc.index)
                .ok_or(LauncherError::MissingBinding { slot: desc.index })?;
            let mut matches = proofs
                .iter()
                .enumerate()
                .filter(|(_, proof)| proof.slot == desc.index);
            let Some((proof_index, proof)) = matches.next() else {
                return Err(LauncherError::MissingMetadataProof { slot: desc.index });
            };
            if matches.next().is_some() {
                return Err(LauncherError::DuplicateMetadataProof { slot: desc.index });
            }
            if proof.kind() != required {
                return Err(LauncherError::MetadataMismatch("证明类型不同"));
            }
            proof.validate_for(binding.span, dims)?;
            used[proof_index] = true;
        }
        if let Some(proof) = proofs
            .iter()
            .zip(used)
            .find_map(|(proof, used)| (!used).then_some(proof))
        {
            return Err(LauncherError::UnexpectedBinding { slot: proof.slot });
        }
        Ok(LaunchPlan {
            dims: launch_dims,
            marker: PhantomData,
        })
    }
}
