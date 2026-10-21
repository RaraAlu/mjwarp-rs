//! 惯性盒与椭球流体被动力。

use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use super::com_velocity::FluidInputs;
use super::com_velocity::{ResidentComVelocityData, ResidentComVelocityPlan};
use super::{ComVelocityData, ComVelocityPlan, KinematicsData, KinematicsPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout, ParameterBatch};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

use self::spec::Spec;

// 子模块：纯移动自原单文件，行为不变。
mod cuda;
mod data;
mod plan_construct;
mod plan_data;
mod plan_launch;
mod plan_shared;
mod plan_update;
mod spec;
#[cfg(test)]
#[path = "fluid_force/tests/checks.rs"]
mod tests_checks;
#[cfg(test)]
#[path = "fluid_force/tests/gpu_box.rs"]
mod tests_gpu_box;
#[cfg(test)]
#[path = "fluid_force/tests/gpu_ellipsoid.rs"]
mod tests_gpu_ellipsoid;
#[cfg(test)]
#[path = "fluid_force/tests/host_periods.rs"]
mod tests_host_periods;
#[cfg(test)]
#[path = "fluid_force/tests/reference.rs"]
mod tests_reference;
#[cfg(test)]
#[path = "fluid_force/tests/support.rs"]
mod tests_support;

#[derive(Clone, Debug)]
pub struct FluidForceInput {
    pub wind: ParameterBatch,
    pub density: ParameterBatch,
    pub viscosity: ParameterBatch,
    pub geom_size: ParameterBatch,
    pub geom_type: Vec<i32>,
    pub geom_fluid: Vec<f32>,
    pub body_fluid_ellipsoid: Vec<bool>,
}

#[derive(Clone, Copy, Debug)]
struct Layout {
    body: BatchLayout,
    qfrc: BatchLayout,
}

fn layout(worlds: usize, nb: usize, nv: usize) -> Result<Layout, InputError> {
    if worlds == 0
        || worlds
            .checked_mul(nb.max(nv))
            .is_none_or(|value| value > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "fluid_force_work",
        });
    }
    let body = BatchLayout::new(
        worlds,
        nb.checked_mul(6).ok_or(InputError::Overflow {
            field: "fluid_force_layout",
        })?,
        4,
    )?;
    let qfrc = BatchLayout::new(worlds, nv, 4)?;
    for count in [body.total_elements(), qfrc.total_elements(), worlds] {
        BatchLayout::new(
            1,
            count.checked_add(8).ok_or(InputError::Overflow {
                field: "fluid_force_layout",
            })?,
            4,
        )?;
    }
    Ok(Layout { body, qfrc })
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
fn clear_work(value: usize) -> Result<u32, InputError> {
    u32::try_from(value)
        .ok()
        .filter(|&value| value > 0 && value <= u32::MAX - 255)
        .ok_or(InputError::Overflow {
            field: "fluid_force_clear_work",
        })
}

pub struct FluidForcePlan<'p> {
    source: FluidForceSource<'p>,
    spec: Arc<Spec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 4],
}

pub struct FluidForceData {
    spec: Arc<Spec>,
    layout: Layout,
    ready: bool,
    resident_velocity: Option<ResidentComVelocityData>,
    #[cfg(feature = "cuda-probe")]
    body: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    qfrc: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}

enum FluidForceSource<'p> {
    Independent(&'p ComVelocityPlan),
    Resident(ResidentComVelocityPlan<'p>),
}

pub struct FluidForceView<'a> {
    data: &'a FluidForceData,
    _source: &'a ComVelocityData,
}

#[derive(Clone, Copy, Debug)]
pub struct FluidForceWorld<'a> {
    pub body_force_torque: &'a [f32],
    pub qfrc: &'a [f32],
}

#[derive(Clone, Debug)]
pub struct FluidForceSnapshot {
    layout: Layout,
    body: Vec<f32>,
    qfrc: Vec<f32>,
}

impl FluidForceSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.qfrc.worlds()
    }

    pub fn world(&self, world: usize) -> Result<FluidForceWorld<'_>, InputError> {
        let body = self.layout.body.world_elements(world)?;
        let qfrc = self.layout.qfrc.world_elements(world)?;
        Ok(FluidForceWorld {
            body_force_torque: &self.body[4 + body.start..4 + body.end],
            qfrc: &self.qfrc[4 + qfrc.start..4 + qfrc.end],
        })
    }
}

#[cfg(feature = "cuda-probe")]
fn check_guards(buffer: &TransferBuffer<f32>) -> Result<(), TransferError> {
    let mut values = [0.0; 8];
    buffer.read_range_into(0, &mut values[..4])?;
    buffer.read_range_into(buffer.len() - 4, &mut values[4..])?;
    super::check_device_guards(&values)
}

#[cfg(feature = "cuda-probe")]
fn read_output(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut values)?;
    super::check_device_values(&values, field)?;
    Ok(values)
}
