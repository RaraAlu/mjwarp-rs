//! 流体被动力四入口的规格与同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::FluidForceDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::FluidForceEntry;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::FluidForceMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::FluidForceStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::fluid_force_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::fluid_force_lengths;
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
fn spans(dims: &FluidForceDims) -> Result<Vec<SlotRequirement>, LauncherError> {
    Ok(fluid_force_lengths(dims)?
        .into_iter()
        .enumerate()
        .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
        .collect())
}

#[cfg(feature = "cuda-probe")]
fn launch_dims(dims: &FluidForceDims) -> Result<LaunchDims, LauncherError> {
    fluid_force_lengths(dims)?;
    let work = dims.abi[3];
    if work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("流体被动力工作量无效"));
    }
    LaunchDims::new((work.div_ceil(256), 1, 1), (256, 1, 1), 0)
}

#[cfg(feature = "cuda-probe")]
fn exact_lengths(bindings: &[SlotBinding], lengths: [u64; 7]) -> Result<(), LauncherError> {
    for (slot, elements) in lengths.into_iter().enumerate() {
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        let required = elements
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("fluid_force_bytes"))?;
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
fn plan_with_spec<S: KernelSpec<Dims = FluidForceDims>>(
    dims: &FluidForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, FluidForceDims>],
) -> Result<(), LauncherError> {
    Kernel::<S>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, fluid_force_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FluidForceValidateSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结fluid_force_validate十一缓冲ABI。
unsafe impl KernelAbi for FluidForceValidateSpec {
    const NAME: &'static str = "fluid_force_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for FluidForceValidateSpec {
    type Dims = FluidForceDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FluidForceBodySpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结fluid_force_body十一缓冲ABI。
unsafe impl KernelAbi for FluidForceBodySpec {
    const NAME: &'static str = "fluid_force_body";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for FluidForceBodySpec {
    type Dims = FluidForceDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FluidForceProjectSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结fluid_force_project十一缓冲ABI。
unsafe impl KernelAbi for FluidForceProjectSpec {
    const NAME: &'static str = "fluid_force_project";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for FluidForceProjectSpec {
    type Dims = FluidForceDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FluidForceClearSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结fluid_force_clear十一缓冲ABI。
unsafe impl KernelAbi for FluidForceClearSpec {
    const NAME: &'static str = "fluid_force_clear";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for FluidForceClearSpec {
    type Dims = FluidForceDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_validate(
    dims: &FluidForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, FluidForceDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<FluidForceValidateSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_body(
    dims: &FluidForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, FluidForceDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<FluidForceBodySpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_project(
    dims: &FluidForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, FluidForceDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<FluidForceProjectSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_clear(
    dims: &FluidForceDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, FluidForceDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<FluidForceClearSpec>(dims, bindings, proofs)
}

/// 三阶段共享的五个输入缓冲。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy)]
pub(in crate::physics) struct FluidForceInputs<'a> {
    pub(in crate::physics) parameters: &'a TransferBuffer<f32>,
    pub(in crate::physics) rigid: &'a TransferBuffer<f32>,
    pub(in crate::physics) com: &'a TransferBuffer<f32>,
    /// 验证与刚体段为空间速度，投影段为刚体空间力。
    pub(in crate::physics) fourth: &'a TransferBuffer<f32>,
    pub(in crate::physics) attached: Option<&'a TransferBuffer<f32>>,
}

/// 流体被动力四入口同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FluidForceKernels {
    validate: SynchronousKernel,
    body: SynchronousKernel,
    project: SynchronousKernel,
    clear: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl FluidForceKernels {
    pub(in crate::physics) fn compile(
        session: &TransferSession,
        code: &str,
    ) -> Result<Self, TransferError> {
        Ok(Self {
            validate: SynchronousKernel::compile(session, code, "fluid_force_validate")?,
            body: SynchronousKernel::compile(session, code, "fluid_force_body")?,
            project: SynchronousKernel::compile(session, code, "fluid_force_project")?,
            clear: SynchronousKernel::compile(session, code, "fluid_force_clear")?,
        })
    }

    /// 第五槽：有椭球体时为附着位姿，否则退回参数占位。
    fn attached_slot<'a>(
        static_dims: FluidForceStaticDims,
        parameters: &'a TransferBuffer<f32>,
        attached: Option<&'a TransferBuffer<f32>>,
    ) -> Result<&'a TransferBuffer<f32>, TransferError> {
        match (static_dims.attached, attached) {
            (true, Some(buffer)) => Ok(buffer),
            (false, None) => Ok(parameters),
            _ => Err(InputError::InvalidDimension {
                field: "fluid_force_attached",
            }
            .into()),
        }
    }

    fn slot(slot: usize, buffer: &TransferBuffer<f32>) -> Result<SlotBinding, TransferError> {
        Ok(SlotBinding {
            slot,
            span: buffer.launcher_span()?,
            elem: ElemType::F32,
        })
    }

    fn metadata_binding(metadata: &FluidForceMetadataBuffer) -> Result<SlotBinding, TransferError> {
        Ok(SlotBinding {
            slot: 0,
            span: metadata.as_buffer().launcher_span()?,
            elem: ElemType::I32,
        })
    }

    fn work(worlds: u32, width: u64) -> Result<u32, TransferError> {
        u32::try_from(
            u64::from(worlds)
                .checked_mul(width)
                .ok_or(InputError::Overflow {
                    field: "fluid_force_work",
                })?,
        )
        .ok()
        .filter(|&value| value > 0 && value <= u32::MAX - 255)
        .ok_or(
            InputError::Overflow {
                field: "fluid_force_work",
            }
            .into(),
        )
    }

    /// 验证段：核对守卫、映射与非有限值后写每世界状态。
    pub(in crate::physics) fn launch_validate(
        &self,
        metadata: &FluidForceMetadataBuffer,
        inputs: FluidForceInputs<'_>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let dims = FluidForceDims {
            static_dims,
            worlds,
            abi: [static_dims.nb, static_dims.nv, worlds, worlds],
            entry: FluidForceEntry::Validate,
            clear_work: 0,
        };
        let proof = metadata.proof(dims)?;
        let fifth = Self::attached_slot(static_dims, inputs.parameters, inputs.attached)?;
        let bindings = [
            Self::metadata_binding(metadata)?,
            Self::slot(1, inputs.parameters)?,
            Self::slot(2, inputs.rigid)?,
            Self::slot(3, inputs.com)?,
            Self::slot(4, inputs.fourth)?,
            Self::slot(5, fifth)?,
            Self::slot(6, status)?,
        ];
        plan_validate(&dims, &bindings, &[proof.binding(0)]).map_err(fluid_force_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.validate.launch_fluid_force(
                metadata.as_buffer(),
                [
                    inputs.parameters,
                    inputs.rigid,
                    inputs.com,
                    inputs.fourth,
                    fifth,
                ],
                status,
                dims.abi,
            )
        }
    }

    /// 刚体段：每线程写一个刚体的六项空间力。
    pub(in crate::physics) fn launch_body(
        &self,
        metadata: &FluidForceMetadataBuffer,
        inputs: FluidForceInputs<'_>,
        body: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = Self::work(worlds, u64::from(static_dims.nb))?;
        let dims = FluidForceDims {
            static_dims,
            worlds,
            abi: [static_dims.nb, static_dims.nv, worlds, work],
            entry: FluidForceEntry::Body,
            clear_work: 0,
        };
        let proof = metadata.proof(dims)?;
        let fifth = Self::attached_slot(static_dims, inputs.parameters, inputs.attached)?;
        let bindings = [
            Self::metadata_binding(metadata)?,
            Self::slot(1, inputs.parameters)?,
            Self::slot(2, inputs.rigid)?,
            Self::slot(3, inputs.com)?,
            Self::slot(4, inputs.fourth)?,
            Self::slot(5, fifth)?,
            Self::slot(6, body)?,
        ];
        plan_body(&dims, &bindings, &[proof.binding(0)]).map_err(fluid_force_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.body.launch_fluid_force(
                metadata.as_buffer(),
                [
                    inputs.parameters,
                    inputs.rigid,
                    inputs.com,
                    inputs.fourth,
                    fifth,
                ],
                body,
                dims.abi,
            )
        }
    }

    /// 投影段：按 body 顺序把空间力投影到自由度。
    pub(in crate::physics) fn launch_project(
        &self,
        metadata: &FluidForceMetadataBuffer,
        inputs: FluidForceInputs<'_>,
        qfrc: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = Self::work(worlds, u64::from(static_dims.nv))?;
        let dims = FluidForceDims {
            static_dims,
            worlds,
            abi: [static_dims.nb, static_dims.nv, worlds, work],
            entry: FluidForceEntry::Project,
            clear_work: 0,
        };
        let proof = metadata.proof(dims)?;
        let fifth = Self::attached_slot(static_dims, inputs.parameters, inputs.attached)?;
        let bindings = [
            Self::metadata_binding(metadata)?,
            Self::slot(1, inputs.parameters)?,
            Self::slot(2, inputs.rigid)?,
            Self::slot(3, inputs.com)?,
            Self::slot(4, inputs.fourth)?,
            Self::slot(5, fifth)?,
            Self::slot(6, qfrc)?,
        ];
        plan_project(&dims, &bindings, &[proof.binding(0)]).map_err(fluid_force_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.project.launch_fluid_force(
                metadata.as_buffer(),
                [
                    inputs.parameters,
                    inputs.rigid,
                    inputs.com,
                    inputs.fourth,
                    fifth,
                ],
                qfrc,
                dims.abi,
            )
        }
    }

    /// 清零段：按输出缓冲元素数清零；五个输入只用参数占位。
    pub(in crate::physics) fn launch_clear(
        &self,
        metadata: &FluidForceMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
        work: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        if work == 0 || work > u32::MAX - 255 {
            return Err(InputError::Overflow {
                field: "fluid_force_clear_work",
            }
            .into());
        }
        let dims = FluidForceDims {
            static_dims,
            worlds,
            abi: [static_dims.nb, static_dims.nv, worlds, work],
            entry: FluidForceEntry::Clear,
            clear_work: work,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
            Self::metadata_binding(metadata)?,
            Self::slot(1, parameters)?,
            Self::slot(2, parameters)?,
            Self::slot(3, parameters)?,
            Self::slot(4, parameters)?,
            Self::slot(5, parameters)?,
            Self::slot(6, output)?,
        ];
        plan_clear(&dims, &bindings, &[proof.binding(0)]).map_err(fluid_force_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.clear.launch_fluid_force(
                metadata.as_buffer(),
                [parameters, parameters, parameters, parameters, parameters],
                output,
                dims.abi,
            )
        }
    }
}
