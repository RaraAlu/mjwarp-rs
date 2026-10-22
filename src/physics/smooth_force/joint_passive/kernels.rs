//! 关节被动力三入口的规格与同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::JointPassiveDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::JointPassiveEntry;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::JointPassiveMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::joint_passive_exact_slots;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::joint_passive_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::joint_passive_lengths;
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
fn spans(dims: &JointPassiveDims) -> Result<Vec<SlotRequirement>, LauncherError> {
    Ok(joint_passive_lengths(dims)?
        .into_iter()
        .enumerate()
        .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
        .collect())
}

#[cfg(feature = "cuda-probe")]
fn launch_dims(dims: &JointPassiveDims) -> Result<LaunchDims, LauncherError> {
    joint_passive_lengths(dims)?;
    let work = dims.abi[3];
    if work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("关节被动力工作量无效"));
    }
    LaunchDims::new((work.div_ceil(256), 1, 1), (256, 1, 1), 0)
}

#[cfg(feature = "cuda-probe")]
fn plan_with_spec<S: KernelSpec<Dims = JointPassiveDims>>(
    dims: &JointPassiveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, JointPassiveDims>],
) -> Result<(), LauncherError> {
    Kernel::<S>::new()?.plan(dims, bindings, proofs)?;
    let lengths = joint_passive_lengths(dims)?;
    let exact = joint_passive_exact_slots(dims.entry);
    for &slot in exact {
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        let required = lengths[slot]
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("joint_passive_bytes"))?;
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
pub(super) struct JointPassiveValidateSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结joint_passive_validate四缓冲ABI。
unsafe impl KernelAbi for JointPassiveValidateSpec {
    const NAME: &'static str = "joint_passive_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for JointPassiveValidateSpec {
    type Dims = JointPassiveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct JointPassiveComputeSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结joint_passive_compute四缓冲ABI。
unsafe impl KernelAbi for JointPassiveComputeSpec {
    const NAME: &'static str = "joint_passive_compute";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for JointPassiveComputeSpec {
    type Dims = JointPassiveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct JointPassiveSumSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结joint_passive_sum四缓冲ABI。
unsafe impl KernelAbi for JointPassiveSumSpec {
    const NAME: &'static str = "joint_passive_sum";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for JointPassiveSumSpec {
    type Dims = JointPassiveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_validate(
    dims: &JointPassiveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, JointPassiveDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<JointPassiveValidateSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_compute(
    dims: &JointPassiveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, JointPassiveDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<JointPassiveComputeSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_sum(
    dims: &JointPassiveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, JointPassiveDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<JointPassiveSumSpec>(dims, bindings, proofs)
}

/// 关节被动力三入口同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct JointPassiveKernels {
    validate: SynchronousKernel,
    compute: SynchronousKernel,
    sum: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl JointPassiveKernels {
    pub(in crate::physics) fn compile(
        session: &TransferSession,
        code: &str,
    ) -> Result<Self, TransferError> {
        Ok(Self {
            validate: SynchronousKernel::compile(session, code, "joint_passive_validate")?,
            compute: SynchronousKernel::compile(session, code, "joint_passive_compute")?,
            sum: SynchronousKernel::compile(session, code, "joint_passive_sum")?,
        })
    }

    fn bindings(
        metadata: &JointPassiveMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
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
                span: parameters.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: state.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    /// 验证段：核对关节种类、qpos 区间与有限值后写每世界状态。
    pub(in crate::physics) fn launch_validate(
        &self,
        metadata: &JointPassiveMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let dims = JointPassiveDims {
            static_dims,
            worlds,
            abi: [static_dims.nq, static_dims.nv, worlds, worlds],
            entry: JointPassiveEntry::Validate,
        };
        let proof = metadata.proof(dims)?;
        let bindings = Self::bindings(metadata, parameters, state, status)?;
        plan_validate(&dims, &bindings, &[proof.binding(0)]).map_err(joint_passive_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.validate.launch_joint_passive(
                metadata.as_buffer(),
                parameters,
                state,
                status,
                dims.abi,
            )
        }
    }

    /// 计算段：每线程写一个弹簧槽与一个阻尼槽。
    pub(in crate::physics) fn launch_compute(
        &self,
        metadata: &JointPassiveMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        target: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = worlds
            .checked_mul(static_dims.nv)
            .ok_or(InputError::Overflow {
                field: "joint_passive_work",
            })?;
        let dims = JointPassiveDims {
            static_dims,
            worlds,
            abi: [static_dims.nq, static_dims.nv, worlds, work],
            entry: JointPassiveEntry::Compute,
        };
        let proof = metadata.proof(dims)?;
        let bindings = Self::bindings(metadata, parameters, state, target)?;
        plan_compute(&dims, &bindings, &[proof.binding(0)]).map_err(joint_passive_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.compute.launch_joint_passive(
                metadata.as_buffer(),
                parameters,
                state,
                target,
                dims.abi,
            )
        }
    }

    /// 求和段：把弹簧与阻尼两部分相加写入平滑力首个字段。
    pub(in crate::physics) fn launch_sum(
        &self,
        metadata: &JointPassiveMetadataBuffer,
        parts: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        forces: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = worlds
            .checked_mul(static_dims.nv)
            .ok_or(InputError::Overflow {
                field: "joint_passive_work",
            })?;
        let dims = JointPassiveDims {
            static_dims,
            worlds,
            abi: [0, static_dims.nv, worlds, work],
            entry: JointPassiveEntry::Sum,
        };
        let proof = metadata.proof(dims)?;
        let bindings = Self::bindings(metadata, parts, unused, forces)?;
        plan_sum(&dims, &bindings, &[proof.binding(0)]).map_err(joint_passive_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.sum
                .launch_joint_passive(metadata.as_buffer(), parts, unused, forces, dims.abi)
        }
    }
}
