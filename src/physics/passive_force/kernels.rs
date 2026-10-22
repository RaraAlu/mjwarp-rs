//! 共享被动力四入口的规格与同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PassiveForceDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PassiveForceEntry;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::PassiveForceMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::passive_force_exact_slots;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::passive_force_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::passive_force_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::runtime::SynchronousKernel;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    BlockRule, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    SlotBinding, SlotDesc, SlotRequirement,
};

#[cfg(feature = "cuda-probe")]
fn spans(dims: &PassiveForceDims) -> Result<Vec<SlotRequirement>, LauncherError> {
    Ok(passive_force_lengths(dims)?
        .into_iter()
        .enumerate()
        .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
        .collect())
}

#[cfg(feature = "cuda-probe")]
fn launch_dims(dims: &PassiveForceDims) -> Result<LaunchDims, LauncherError> {
    passive_force_lengths(dims)?;
    let work = dims.abi[3];
    if work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("共享被动力工作量无效"));
    }
    LaunchDims::new((work.div_ceil(256), 1, 1), (256, 1, 1), 0)
}

#[cfg(feature = "cuda-probe")]
fn plan_with_spec<S: KernelSpec<Dims = PassiveForceDims>>(
    dims: &PassiveForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, PassiveForceDims>],
) -> Result<(), LauncherError> {
    Kernel::<S>::new()?.plan(dims, bindings, proofs)?;
    let lengths = passive_force_lengths(dims)?;
    for &slot in passive_force_exact_slots(dims.entry) {
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        let required = lengths[slot]
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("passive_force_bytes"))?;
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
pub(super) struct PassiveForceClearSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结passive_force_clear四缓冲ABI。
unsafe impl KernelAbi for PassiveForceClearSpec {
    const NAME: &'static str = "passive_force_clear";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for PassiveForceClearSpec {
    type Dims = PassiveForceDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct PassiveForceSumSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结passive_force_sum四缓冲ABI。
unsafe impl KernelAbi for PassiveForceSumSpec {
    const NAME: &'static str = "passive_force_sum";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for PassiveForceSumSpec {
    type Dims = PassiveForceDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct PassiveForceClearGravitySpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结passive_force_clear_gravity四缓冲ABI。
unsafe impl KernelAbi for PassiveForceClearGravitySpec {
    const NAME: &'static str = "passive_force_clear_gravity";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for PassiveForceClearGravitySpec {
    type Dims = PassiveForceDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct PassiveForceAddFluidSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结passive_force_add_fluid四缓冲ABI。
unsafe impl KernelAbi for PassiveForceAddFluidSpec {
    const NAME: &'static str = "passive_force_add_fluid";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for PassiveForceAddFluidSpec {
    type Dims = PassiveForceDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_clear(
    dims: &PassiveForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, PassiveForceDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<PassiveForceClearSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_sum(
    dims: &PassiveForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, PassiveForceDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<PassiveForceSumSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_clear_gravity(
    dims: &PassiveForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, PassiveForceDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<PassiveForceClearGravitySpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_add_fluid(
    dims: &PassiveForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, PassiveForceDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<PassiveForceAddFluidSpec>(dims, bindings, proofs)
}

/// 共享被动力四入口同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct PassiveForceKernels {
    clear: SynchronousKernel,
    sum: SynchronousKernel,
    clear_gravity: SynchronousKernel,
    add_fluid: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl PassiveForceKernels {
    pub(in crate::physics) fn compile(
        session: &TransferSession,
        code: &str,
    ) -> Result<Self, TransferError> {
        Ok(Self {
            clear: SynchronousKernel::compile(session, code, "passive_force_clear")?,
            sum: SynchronousKernel::compile(session, code, "passive_force_sum")?,
            clear_gravity: SynchronousKernel::compile(
                session,
                code,
                "passive_force_clear_gravity",
            )?,
            add_fluid: SynchronousKernel::compile(session, code, "passive_force_add_fluid")?,
        })
    }

    fn metadata_binding(
        metadata: &PassiveForceMetadataBuffer,
    ) -> Result<SlotBinding, TransferError> {
        Ok(SlotBinding {
            slot: 0,
            span: metadata.as_buffer().launcher_span()?,
            elem: ElemType::I32,
        })
    }

    fn value_binding(
        slot: usize,
        buffer: &TransferBuffer<f32>,
    ) -> Result<SlotBinding, TransferError> {
        Ok(SlotBinding {
            slot,
            span: buffer.launcher_span()?,
            elem: ElemType::F32,
        })
    }

    fn work(
        worlds: u32,
        static_dims: super::launcher_abi::PassiveForceStaticDims,
    ) -> Result<u32, TransferError> {
        worlds.checked_mul(static_dims.nv).ok_or(
            InputError::Overflow {
                field: "passive_force_work",
            }
            .into(),
        )
    }

    /// 清零段：每线程清零两个分量槽。
    pub(in crate::physics) fn launch_clear(
        &self,
        metadata: &PassiveForceMetadataBuffer,
        dummy: &TransferBuffer<f32>,
        parts: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = Self::work(worlds, static_dims)?;
        let dims = PassiveForceDims {
            static_dims,
            worlds,
            abi: [0, static_dims.nv, worlds, work],
            entry: PassiveForceEntry::Clear,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            Self::metadata_binding(metadata)?,
            Self::value_binding(1, dummy)?,
            Self::value_binding(2, dummy)?,
            Self::value_binding(3, parts)?,
        ];
        plan_clear(&dims, &bindings, &[proof.binding(0)]).map_err(passive_force_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.clear
                .launch_passive_force(metadata.as_buffer(), dummy, dummy, parts, dims.abi)
        }
    }

    /// 求和段：把弹簧与阻尼分量相加写入平滑力首个字段。
    pub(in crate::physics) fn launch_sum(
        &self,
        metadata: &PassiveForceMetadataBuffer,
        dummy: &TransferBuffer<f32>,
        parts: &TransferBuffer<f32>,
        forces: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = Self::work(worlds, static_dims)?;
        let dims = PassiveForceDims {
            static_dims,
            worlds,
            abi: [0, static_dims.nv, worlds, work],
            entry: PassiveForceEntry::Sum,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            Self::metadata_binding(metadata)?,
            Self::value_binding(1, dummy)?,
            Self::value_binding(2, parts)?,
            Self::value_binding(3, forces)?,
        ];
        plan_sum(&dims, &bindings, &[proof.binding(0)]).map_err(passive_force_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.sum
                .launch_passive_force(metadata.as_buffer(), dummy, parts, forces, dims.abi)
        }
    }

    /// 重力清零段：每线程清零一个重力槽。
    pub(in crate::physics) fn launch_clear_gravity(
        &self,
        metadata: &PassiveForceMetadataBuffer,
        dummy: &TransferBuffer<f32>,
        gravity: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = Self::work(worlds, static_dims)?;
        let dims = PassiveForceDims {
            static_dims,
            worlds,
            abi: [0, static_dims.nv, worlds, work],
            entry: PassiveForceEntry::ClearGravity,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            Self::metadata_binding(metadata)?,
            Self::value_binding(1, dummy)?,
            Self::value_binding(2, dummy)?,
            Self::value_binding(3, gravity)?,
        ];
        plan_clear_gravity(&dims, &bindings, &[proof.binding(0)])
            .map_err(passive_force_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.clear_gravity.launch_passive_force(
                metadata.as_buffer(),
                dummy,
                dummy,
                gravity,
                dims.abi,
            )
        }
    }

    /// 流体累加段：把流体结果加进平滑力首个字段。
    pub(in crate::physics) fn launch_add_fluid(
        &self,
        metadata: &PassiveForceMetadataBuffer,
        dummy: &TransferBuffer<f32>,
        fluid: &TransferBuffer<f32>,
        forces: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = Self::work(worlds, static_dims)?;
        let dims = PassiveForceDims {
            static_dims,
            worlds,
            abi: [0, static_dims.nv, worlds, work],
            entry: PassiveForceEntry::AddFluid,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            Self::metadata_binding(metadata)?,
            Self::value_binding(1, dummy)?,
            Self::value_binding(2, fluid)?,
            Self::value_binding(3, forces)?,
        ];
        plan_add_fluid(&dims, &bindings, &[proof.binding(0)])
            .map_err(passive_force_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.add_fluid.launch_passive_force(
                metadata.as_buffer(),
                dummy,
                fluid,
                forces,
                dims.abi,
            )
        }
    }
}
