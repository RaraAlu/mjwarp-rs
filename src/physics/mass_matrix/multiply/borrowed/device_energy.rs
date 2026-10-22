//! 严格当前态Mqvel视图接入默认32线程动能。
//! GPU内成对复制，不采用宿主物理回传。

pub(super) use super::{MassMatrixMultiplyPlan, MassMatrixMultiplyView, Spec};
#[cfg(not(feature = "cuda-probe"))]
pub(super) use crate::diagnostics::ProbeError;
pub(super) use crate::diagnostics::{InputError, TransferError};
pub(super) use crate::model::BatchLayout;
#[cfg(feature = "cuda-probe")]
pub(super) use crate::physics::kinetic_energy::{
    KINETIC_METADATA_KIND, KineticEnergy32Kernel, KineticEnergyMetadataBuffer,
    KineticValidationKernel,
};
#[cfg(feature = "cuda-probe")]
pub(super) use crate::runtime::launcher::{
    Access, BlockRule, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    SlotBinding, SlotDesc, SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
pub(super) use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
pub(super) use std::sync::Arc;

mod plan;
mod schedule;
mod spec;
#[cfg(test)]
mod tests;
#[cfg(feature = "cuda-probe")]
mod validation;

pub use plan::{
    MassMatrixEnergyData, MassMatrixEnergyPlan, MassMatrixEnergySnapshot, MassMatrixEnergyView,
};
pub use schedule::schedule_energy_chain;
