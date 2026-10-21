//! 流体规格构造与冻结参数偏移。

use super::*;

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
pub(super) struct Spec {
    pub(super) body_mass: ParameterBatch,
    pub(super) body_inertia: ParameterBatch,
    pub(super) input: FluidForceInput,
    pub(super) metadata: Vec<i32>,
    pub(super) offsets: [usize; 6],
    pub(super) fluid_offset: usize,
    pub(super) parameter_len: usize,
    pub(super) nb: usize,
    pub(super) nj: usize,
    pub(super) nv: usize,
    pub(super) ng: usize,
}

pub(super) fn spec(
    model: Arc<AttachedModelInput>,
    body_mass: ParameterBatch,
    body_inertia: ParameterBatch,
    input: FluidForceInput,
) -> Result<Spec, InputError> {
    let kinematic = model.rigid().kinematics();
    let inertial = model.rigid().fields();
    let (nb, nj, nv, ng) = (
        kinematic.nbody(),
        kinematic.njnt(),
        kinematic.nv(),
        model.ngeom(),
    );
    if nb < 2 || nv == 0 {
        return Err(InputError::InvalidDimension {
            field: "fluid_force_dimensions",
        });
    }
    for (field, actual, expected) in [
        ("opt_wind", input.wind.row_elements(), 3),
        ("opt_density", input.density.row_elements(), 1),
        ("opt_viscosity", input.viscosity.row_elements(), 1),
        ("body_mass", body_mass.row_elements(), nb),
        ("body_inertia", body_inertia.row_elements(), 3 * nb),
        ("geom_size", input.geom_size.row_elements(), 3 * ng),
        ("geom_type", input.geom_type.len(), ng),
        ("geom_fluid", input.geom_fluid.len(), 12 * ng),
        ("body_fluid_ellipsoid", input.body_fluid_ellipsoid.len(), nb),
    ] {
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    if let Some(index) = input.geom_fluid.iter().position(|value| !value.is_finite()) {
        return Err(InputError::NonFinite {
            field: "geom_fluid",
            index,
        });
    }

    let fields = [
        &input.wind,
        &input.density,
        &input.viscosity,
        &body_mass,
        &body_inertia,
        &input.geom_size,
    ];
    let mut offsets = [0; 6];
    let mut parameter_len = 0usize;
    for (index, field) in fields.iter().enumerate() {
        offsets[index] = parameter_len;
        parameter_len = parameter_len
            .checked_add(field.values().len())
            .filter(|&value| value <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "fluid_force_parameters",
            })?;
    }
    let fluid_offset = parameter_len;
    parameter_len = parameter_len
        .checked_add(input.geom_fluid.len())
        .filter(|&value| value <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "fluid_force_parameters",
        })?;

    let topology = kinematic.fields();
    let attached = model.fields();
    let mut roots = vec![0; nb];
    for (body, body_root) in roots.iter_mut().enumerate().skip(1) {
        let mut root = body;
        while topology.body_parentid[root] != 0 {
            root = topology.body_parentid[root] as usize;
        }
        *body_root = root as i32;
    }
    let metadata_len = nb
        .checked_mul(3)
        .and_then(|value| value.checked_add(nv))
        .and_then(|value| ng.checked_mul(2).and_then(|ng2| value.checked_add(ng2)))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "fluid_force_metadata",
        })?;
    let mut metadata = Vec::new();
    metadata
        .try_reserve_exact(metadata_len)
        .map_err(|_| InputError::Overflow {
            field: "fluid_force_metadata",
        })?;
    metadata.extend([-131072; 4]);
    metadata.extend(&topology.body_parentid);
    metadata.extend(roots);
    metadata.extend(&inertial.dof_bodyid);
    metadata.extend(&attached.geom_bodyid);
    metadata.extend(&input.geom_type);
    metadata.extend(
        input
            .body_fluid_ellipsoid
            .iter()
            .map(|&value| i32::from(value)),
    );
    metadata.extend([-131072; 4]);

    Ok(Spec {
        body_mass,
        body_inertia,
        input,
        metadata,
        offsets,
        fluid_offset,
        parameter_len,
        nb,
        nj,
        nv,
        ng,
    })
}
