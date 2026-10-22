//! W17及W05前置：发射计划和安全校验。

use super::{
    AbiSignature, BlockRule, CheckedBinding, KernelAbi, LauncherError, SlotBinding,
    SlotRequirement, SlotRole, validate_bindings,
};
use std::marker::PhantomData;

/// 设备共享内存上限：默认每块上限与显式 opt-in 上限。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryLimits {
    per_block: u32,
    opt_in: u32,
}

impl SharedMemoryLimits {
    /// 未查询设备属性时的保守默认值：48KiB，不启用 opt-in。
    pub const DEFAULT: Self = Self {
        per_block: 48 * 1024,
        opt_in: 48 * 1024,
    };

    /// 由设备属性构造；opt-in上限不得小于默认每块上限。
    pub fn new(per_block: u32, opt_in: u32) -> Result<Self, LauncherError> {
        if per_block == 0 || opt_in < per_block {
            return Err(LauncherError::InvalidArray("共享内存上限非法"));
        }
        Ok(Self { per_block, opt_in })
    }

    pub fn per_block(&self) -> u32 {
        self.per_block
    }

    pub fn opt_in(&self) -> u32 {
        self.opt_in
    }

    /// 与设备查询上限取交集：注入值更大时收紧到设备值。
    pub fn intersect(self, device: DeviceSharedMemoryLimits) -> Self {
        let per_block = self.per_block.min(device.per_block);
        let opt_in = self.opt_in.min(device.opt_in).max(per_block);
        Self { per_block, opt_in }
    }

    /// 查询设备上限（失败时保留保守默认）：调用方无需自行处理查询失败。
    pub fn for_device(device: usize) -> Self {
        match DeviceSharedMemoryLimits::query(device) {
            Some(limits) => Self::DEFAULT.intersect(limits),
            None => Self::DEFAULT,
        }
    }
}

/// 设备查询得到的共享内存上限。
///
/// 经 cudarc driver 绑定调用 `cuDeviceGetAttribute`：
/// `CU_DEVICE_ATTRIBUTE_MAX_SHARED_MEMORY_PER_BLOCK` 与
/// `CU_DEVICE_ATTRIBUTE_MAX_SHARED_MEMORY_PER_BLOCK_OPTIN`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceSharedMemoryLimits {
    per_block: u32,
    opt_in: u32,
}

impl DeviceSharedMemoryLimits {
    pub fn new(per_block: u32, opt_in: u32) -> Result<Self, LauncherError> {
        if per_block == 0 || opt_in < per_block {
            return Err(LauncherError::InvalidArray("设备共享内存上限非法"));
        }
        Ok(Self { per_block, opt_in })
    }

    /// 读取真实设备属性；驱动不可用、属性缺失或数值非法时返回 `None`。
    pub fn query(device: usize) -> Option<Self> {
        #[cfg(feature = "cuda-probe")]
        {
            use cudarc::driver::sys::CUdevice_attribute;
            let ordinal = i32::try_from(device).ok()?;
            std::panic::catch_unwind(|| {
                cudarc::driver::result::init().ok()?;
                let raw = cudarc::driver::result::device::get(ordinal).ok()?;
                let read = |attribute| {
                    // SAFETY: `raw` 来自 cuDeviceGet；属性查询只写一个 i32 输出。
                    unsafe { cudarc::driver::result::device::get_attribute(raw, attribute).ok() }
                };
                let per_block = u32::try_from(read(
                    CUdevice_attribute::CU_DEVICE_ATTRIBUTE_MAX_SHARED_MEMORY_PER_BLOCK,
                )?)
                .ok()?;
                let opt_in = u32::try_from(read(
                    CUdevice_attribute::CU_DEVICE_ATTRIBUTE_MAX_SHARED_MEMORY_PER_BLOCK_OPTIN,
                )?)
                .ok()?;
                Self::new(per_block, opt_in).ok()
            })
            .ok()?
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = device;
            None
        }
    }

    pub fn per_block(&self) -> u32 {
        self.per_block
    }

    pub fn opt_in(&self) -> u32 {
        self.opt_in
    }
}

/// CUDA网格、线程块与共享内存。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchDims {
    grid: (u32, u32, u32),
    block: (u32, u32, u32),
    shared_bytes: u32,
}

impl LaunchDims {
    /// 静态共享内存路径：发射参数 `shared_bytes` 必须为0。
    pub fn new(
        grid: (u32, u32, u32),
        block: (u32, u32, u32),
        shared_bytes: u32,
    ) -> Result<Self, LauncherError> {
        Self::with_limits(
            grid,
            block,
            shared_bytes,
            SharedMemoryLimits::DEFAULT,
            false,
        )
    }

    /// 带设备上限与动态共享内存 opt-in 的构造。
    pub fn with_limits(
        grid: (u32, u32, u32),
        block: (u32, u32, u32),
        shared_bytes: u32,
        limits: SharedMemoryLimits,
        dynamic_opt_in: bool,
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
        let limit = if dynamic_opt_in {
            limits.opt_in
        } else {
            limits.per_block
        };
        if shared_bytes > limit {
            return Err(LauncherError::InvalidArray("共享内存超过设备上限"));
        }
        if shared_bytes > 0 && !dynamic_opt_in {
            return Err(LauncherError::Unsupported("动态共享内存未opt-in"));
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

    /// 校验跨度、别名、上下文与元数据；共享内存按保守默认上限核对。
    pub fn plan(
        &self,
        dims: &K::Dims,
        bindings: &[SlotBinding],
        proofs: &[CheckedBinding<'_, K::Dims>],
    ) -> Result<LaunchPlan<K>, LauncherError> {
        self.plan_with_limits(dims, bindings, proofs, SharedMemoryLimits::DEFAULT)
    }

    /// 带设备共享内存上限的规划。
    pub fn plan_with_limits(
        &self,
        dims: &K::Dims,
        bindings: &[SlotBinding],
        proofs: &[CheckedBinding<'_, K::Dims>],
        limits: SharedMemoryLimits,
    ) -> Result<LaunchPlan<K>, LauncherError> {
        let launch_dims = K::launch_dims(dims)?;
        launch_dims.validate_rule(self.signature.block())?;
        if self.signature.shared_bytes() > limits.per_block() {
            return Err(LauncherError::InvalidArray("静态共享内存超过设备上限"));
        }
        if launch_dims.shared_bytes() > 0 {
            if self.signature.shared_bytes() > 0 {
                return Err(LauncherError::InvalidArray(
                    "静态与动态共享内存不能同时声明",
                ));
            }
            if launch_dims.shared_bytes() > limits.opt_in() {
                return Err(LauncherError::InvalidArray("共享内存超过设备上限"));
            }
        }
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
