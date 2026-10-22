//! 肌腱被动力三入口的规格与同步内核包装。

#[cfg(feature = "cuda-probe")]
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonPassiveDims;
#[cfg(feature = "cuda-probe")]
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonPassiveMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::tendon_passive_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::tendon_passive_lengths;
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
fn spans(dims: &TendonPassiveDims) -> Result<Vec<SlotRequirement>, LauncherError> {
    Ok(tendon_passive_lengths(dims)?
        .into_iter()
        .enumerate()
        .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
        .collect())
}

#[cfg(feature = "cuda-probe")]
fn launch_dims(dims: &TendonPassiveDims) -> Result<LaunchDims, LauncherError> {
    tendon_passive_lengths(dims)?;
    let work = dims.abi[3];
    if work == 0 || work > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("肌腱被动力工作量无效"));
    }
    LaunchDims::new((work.div_ceil(256), 1, 1), (256, 1, 1), 0)
}

#[cfg(feature = "cuda-probe")]
fn exact_lengths(bindings: &[SlotBinding], lengths: [u64; 5]) -> Result<(), LauncherError> {
    for (slot, elements) in lengths.into_iter().enumerate() {
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        let required = elements
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("tendon_passive_bytes"))?;
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
fn plan_with_spec<S: KernelSpec<Dims = TendonPassiveDims>>(
    dims: &TendonPassiveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, TendonPassiveDims>],
) -> Result<(), LauncherError> {
    Kernel::<S>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, tendon_passive_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonPassiveValidateSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结tendon_passive_validate五缓冲ABI。
unsafe impl KernelAbi for TendonPassiveValidateSpec {
    const NAME: &'static str = "tendon_passive_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for TendonPassiveValidateSpec {
    type Dims = TendonPassiveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonPassiveClearSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结tendon_passive_clear五缓冲ABI。
unsafe impl KernelAbi for TendonPassiveClearSpec {
    const NAME: &'static str = "tendon_passive_clear";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for TendonPassiveClearSpec {
    type Dims = TendonPassiveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct TendonPassiveComputeSpec;
#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结tendon_passive_compute五缓冲ABI。
unsafe impl KernelAbi for TendonPassiveComputeSpec {
    const NAME: &'static str = "tendon_passive_compute";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}
#[cfg(feature = "cuda-probe")]
impl KernelSpec for TendonPassiveComputeSpec {
    type Dims = TendonPassiveDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        spans(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        launch_dims(dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_validate(
    dims: &TendonPassiveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, TendonPassiveDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<TendonPassiveValidateSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_clear(
    dims: &TendonPassiveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, TendonPassiveDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<TendonPassiveClearSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn plan_compute(
    dims: &TendonPassiveDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, TendonPassiveDims>],
) -> Result<(), LauncherError> {
    plan_with_spec::<TendonPassiveComputeSpec>(dims, bindings, proofs)
}

#[cfg(feature = "cuda-probe")]
/// 肌腱被动力三入口同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct TendonPassiveKernels {
    validate: SynchronousKernel,
    clear: SynchronousKernel,
    compute: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl TendonPassiveKernels {
    pub(in crate::physics) fn compile(
        session: &TransferSession,
        code: &str,
    ) -> Result<Self, TransferError> {
        Ok(Self {
            validate: SynchronousKernel::compile(session, code, "tendon_passive_validate")?,
            clear: SynchronousKernel::compile(session, code, "tendon_passive_clear")?,
            compute: SynchronousKernel::compile(session, code, "tendon_passive_compute")?,
        })
    }

    #[cfg(feature = "cuda-probe")]
    fn bindings(
        metadata: &TendonPassiveMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        source: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 5], TransferError> {
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
                span: source.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: velocity.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    /// 验证段：核对源/速度守卫与非有限值后写每世界状态。
    pub(in crate::physics) fn launch_validate(
        &self,
        metadata: &TendonPassiveMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        source: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = TendonPassiveDims {
            static_dims: metadata.static_dims(),
            worlds,
            abi: [
                metadata.static_dims().nt,
                metadata.static_dims().nv,
                worlds,
                worlds,
            ],
            output_width: 1,
        };
        let proof = metadata.proof(dims)?;
        let bindings = Self::bindings(metadata, parameters, source, velocity, status)?;
        plan_validate(&dims, &bindings, &[proof.binding(0)])
            .map_err(tendon_passive_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.validate.launch_tendon_passive(
                metadata.as_buffer(),
                parameters,
                source,
                velocity,
                status,
                dims.abi,
            )
        }
    }

    /// 清零段：按两倍自由度槽清空暂存结果。
    pub(in crate::physics) fn launch_clear(
        &self,
        metadata: &TendonPassiveMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        source: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        staged: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = worlds
            .checked_mul(static_dims.nv)
            .and_then(|slots| slots.checked_mul(2))
            .ok_or(InputError::Overflow {
                field: "tendon_passive_work",
            })?;
        let dims = TendonPassiveDims {
            static_dims,
            worlds,
            abi: [0, 0, worlds, work],
            output_width: 2 * u64::from(static_dims.nv),
        };
        let proof = metadata.proof(dims)?;
        let bindings = Self::bindings(metadata, parameters, source, velocity, staged)?;
        plan_clear(&dims, &bindings, &[proof.binding(0)]).map_err(tendon_passive_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.clear.launch_tendon_passive(
                metadata.as_buffer(),
                parameters,
                source,
                velocity,
                staged,
                dims.abi,
            )
        }
    }

    /// 计算段：每个稀疏槽聚合弹性与阻尼力。
    pub(in crate::physics) fn launch_compute(
        &self,
        metadata: &TendonPassiveMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        source: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        target: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let static_dims = metadata.static_dims();
        let work = worlds
            .checked_mul(static_dims.nt)
            .and_then(|rows| rows.checked_mul(static_dims.max_row))
            .ok_or(InputError::Overflow {
                field: "tendon_passive_work",
            })?;
        let dims = TendonPassiveDims {
            static_dims,
            worlds,
            abi: [static_dims.nt, static_dims.nv, worlds, work],
            output_width: 2 * u64::from(static_dims.nv),
        };
        let proof = metadata.proof(dims)?;
        let bindings = Self::bindings(metadata, parameters, source, velocity, target)?;
        plan_compute(&dims, &bindings, &[proof.binding(0)]).map_err(tendon_passive_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与工作量网格。
        unsafe {
            self.compute.launch_tendon_passive(
                metadata.as_buffer(),
                parameters,
                source,
                velocity,
                target,
                dims.abi,
            )
        }
    }
}
