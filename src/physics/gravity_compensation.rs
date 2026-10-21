//! 重力补偿设备子集。
//! 它借用常驻位姿与质心。

mod cuda;
mod plan;
#[cfg(test)]
mod tests;

use crate::diagnostics::InputError;
use crate::model::{AttachedModelInput, BatchLayout, ParameterBatch};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

use super::{KinematicsData, KinematicsPlan};

#[derive(Clone, Debug)]
pub struct GravityCompensationParameters {
    pub gravity: ParameterBatch,
    pub body_gravcomp: ParameterBatch,
    pub jnt_actgravcomp: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GravityCompensationOptions {
    pub disable_gravity: bool,
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
struct Spec {
    body_mass: ParameterBatch,
    parameters: GravityCompensationParameters,
    metadata: Vec<i32>,
    offsets: [usize; 3],
    parameter_len: usize,
    active: bool,
    nb: usize,
    nj: usize,
    nv: usize,
}

fn spec(
    model: Arc<AttachedModelInput>,
    body_mass: ParameterBatch,
    parameters: GravityCompensationParameters,
    options: GravityCompensationOptions,
) -> Result<Spec, InputError> {
    let k = model.rigid().kinematics();
    let (nb, nj, nv) = (k.nbody(), k.njnt(), k.nv());
    if nb < 2 || nv == 0 {
        return Err(InputError::InvalidDimension {
            field: "gravity_compensation_dimensions",
        });
    }
    for (field, actual, expected) in [
        ("opt_gravity", parameters.gravity.row_elements(), 3),
        ("body_mass", body_mass.row_elements(), nb),
        ("body_gravcomp", parameters.body_gravcomp.row_elements(), nb),
        ("jnt_actgravcomp", parameters.jnt_actgravcomp.len(), nj),
    ] {
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    for (index, &value) in parameters.jnt_actgravcomp.iter().enumerate() {
        if value != 0 && value != 1 {
            return Err(InputError::InvalidTopology {
                field: "jnt_actgravcomp",
                index,
                reason: "noncanonical_boolean",
            });
        }
    }
    let fields = [&parameters.gravity, &body_mass, &parameters.body_gravcomp];
    let mut offsets = [0; 3];
    let mut parameter_len = 0usize;
    for (index, field) in fields.iter().enumerate() {
        offsets[index] = parameter_len;
        parameter_len = parameter_len
            .checked_add(field.values().len())
            .filter(|&value| value <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_parameters",
            })?;
    }
    let topology = k.fields();
    let inertial = model.rigid().fields();
    let ancestry_len = nb.checked_mul(nv).ok_or(InputError::Overflow {
        field: "gravity_compensation_metadata",
    })?;
    let metadata_len = nb
        .checked_add(ancestry_len)
        .and_then(|value| value.checked_add(nv))
        .and_then(|value| value.checked_add(nj))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "gravity_compensation_metadata",
        })?;
    let mut roots = Vec::new();
    roots
        .try_reserve_exact(nb)
        .map_err(|_| InputError::Overflow {
            field: "gravity_compensation_metadata",
        })?;
    roots.resize(nb, 0);
    for (body, slot) in roots.iter_mut().enumerate().skip(1) {
        let mut root = body;
        while topology.body_parentid[root] != 0 {
            root = topology.body_parentid[root] as usize;
        }
        *slot = root as i32;
    }
    let mut ancestry = Vec::new();
    ancestry
        .try_reserve_exact(ancestry_len)
        .map_err(|_| InputError::Overflow {
            field: "gravity_compensation_metadata",
        })?;
    ancestry.resize(ancestry_len, 0);
    for body in 1..nb {
        for dof in 0..nv {
            let target = inertial.dof_bodyid[dof] as usize;
            let mut cursor = body;
            loop {
                if cursor == target {
                    ancestry[body * nv + dof] = 1;
                    break;
                }
                if cursor == 0 {
                    break;
                }
                cursor = topology.body_parentid[cursor] as usize;
            }
        }
    }
    let mut metadata = Vec::new();
    metadata
        .try_reserve_exact(metadata_len)
        .map_err(|_| InputError::Overflow {
            field: "gravity_compensation_metadata",
        })?;
    metadata.extend([-131072; 4]);
    metadata.extend(roots);
    metadata.extend(ancestry);
    metadata.extend(&inertial.dof_jntid);
    metadata.extend(&parameters.jnt_actgravcomp);
    metadata.extend([-131072; 4]);
    let active = !options.disable_gravity
        && parameters
            .body_gravcomp
            .values()
            .chunks_exact(nb)
            .any(|row| row[1..].iter().any(|&value| value != 0.0));
    Ok(Spec {
        body_mass,
        parameters,
        metadata,
        offsets,
        parameter_len,
        active,
        nb,
        nj,
        nv,
    })
}

fn layout(worlds: usize, nv: usize) -> Result<BatchLayout, InputError> {
    if worlds == 0
        || worlds
            .checked_mul(nv)
            .is_none_or(|value| value > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "gravity_compensation_work",
        });
    }
    let layout = BatchLayout::new(worlds, nv, 4)?;
    BatchLayout::new(
        1,
        layout
            .total_elements()
            .checked_add(8)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_output",
            })?,
        4,
    )?;
    Ok(layout)
}

pub struct GravityCompensationPlan<'p> {
    source: &'p KinematicsPlan,
    spec: Arc<Spec>,
    options: GravityCompensationOptions,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 4],
}

pub struct GravityCompensationData {
    spec: Arc<Spec>,
    layout: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}

pub struct GravityCompensationView<'a> {
    data: &'a GravityCompensationData,
    _source: &'a KinematicsData,
}

#[derive(Clone, Debug)]
pub struct GravityCompensationSnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}

impl GravityCompensationSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        Ok(&self.values[4 + range.start..4 + range.end])
    }
}
