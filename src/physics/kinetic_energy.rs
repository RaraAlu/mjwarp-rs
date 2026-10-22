//! 冻结默认32线程动能归约子集。
//! 调用方保证Mqvel与qvel来自同一模型和状态。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
use crate::model::data::SourceState;
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc, SlotKind, SlotRequirement,
    SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Spec {
    model_id: u64,
    nv: usize,
}
#[derive(Clone, Copy, Debug)]
struct Layout {
    pair: BatchLayout,
    energy: BatchLayout,
}
fn spec(model_id: u64, nv: usize) -> Result<Spec, InputError> {
    if model_id == 0 || !(1..=1024).contains(&nv) {
        return Err(InputError::InvalidDimension {
            field: "kinetic_energy_spec",
        });
    }
    Ok(Spec { model_id, nv })
}
fn layout(worlds: usize, nv: usize) -> Result<Layout, InputError> {
    if worlds == 0 || worlds > i32::MAX as usize || !(1..=1024).contains(&nv) {
        return Err(InputError::InvalidDimension {
            field: "kinetic_energy_layout",
        });
    }
    let pair = BatchLayout::new(worlds, 2 * nv, 4)?;
    let energy = BatchLayout::new(worlds, 2, 4)?;
    for count in [pair.total_elements(), energy.total_elements(), worlds] {
        count
            .checked_add(8)
            .and_then(|v| v.checked_mul(4))
            .ok_or(InputError::Overflow {
                field: "kinetic_energy_guards",
            })?;
    }
    Ok(Layout { pair, energy })
}

// 子模块：纯移动自原单文件，行为不变。
mod buffer;
mod checks;
mod cuda;
mod data;
mod kernels;
mod metadata;
mod plan;
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) use buffer::KineticEnergyMetadataBuffer;
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) use kernels::{KineticEnergy32Kernel, KineticValidationKernel};
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) use metadata::KINETIC_METADATA_KIND;
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) use metadata::KineticLaunchMetadata;
#[cfg(test)]
#[path = "kinetic_energy/tests/gpu_bits.rs"]
mod tests_gpu_bits;
#[cfg(test)]
#[path = "kinetic_energy/tests/gpu_guards.rs"]
mod tests_gpu_guards;
#[cfg(test)]
#[path = "kinetic_energy/tests/host.rs"]
mod tests_host;
#[cfg(test)]
#[path = "kinetic_energy/tests/support.rs"]
mod tests_support;
#[cfg(test)]
#[path = "kinetic_energy/tests/typed.rs"]
mod tests_typed;

/// 标签声明来源关联，不认证外部矩阵或生产者。
#[derive(Clone, Copy, Debug)]
pub struct KineticEnergyVector<'a> {
    pub model_id: u64,
    pub state_id: u64,
    pub values: &'a [f32],
}

impl<'a> KineticEnergyVector<'a> {
    /// 由 A1 来源快照派生标签：模型标签取模型身份，状态标签取来源 token。
    ///
    /// 与调用方自造整数不同，该标签可由同一 `SourceState` 稳定复算；
    /// 它仍不认证数值内容或外部矩阵，需调用方保证同模型同状态。
    pub fn from_source(source: SourceState, values: &'a [f32]) -> Self {
        Self {
            model_id: source.identity().model().value(),
            state_id: source.token(),
            values,
        }
    }
}

pub struct KineticEnergyPlan {
    spec: Arc<Spec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: KineticEnergyMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validator: KineticValidationKernel,
    #[cfg(feature = "cuda-probe")]
    reducer: KineticEnergy32Kernel,
}
pub struct KineticEnergyData {
    spec: Arc<Spec>,
    layout: Layout,
    states: Vec<Option<u64>>,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    pair: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct KineticEnergyOutput {
    layout: BatchLayout,
    model_id: u64,
    states: Vec<u64>,
    values: Vec<f32>,
}
impl KineticEnergyOutput {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn model_id(&self) -> u64 {
        self.model_id
    }
    /// 返回`[势能,动能]`；状态标签仅描述动能输入。
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
    pub fn state_id(&self, world: usize) -> Result<u64, InputError> {
        self.layout.world_elements(world)?;
        Ok(self.states[world])
    }
}
