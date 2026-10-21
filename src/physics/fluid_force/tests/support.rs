//! 流体测试共用模型与参数构造。

use super::*;

use crate::model::{
    AttachedFields, InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
    KinematicsParameter, KinematicsParameters,
};

pub(super) fn simple_model() -> AttachedModelInput {
    let kinematic = KinematicModelInput::new(
        1,
        KinematicFields {
            qpos0: vec![0.0],
            body_parentid: vec![0, 0, 1],
            body_jntadr: vec![-1, 0, -1],
            body_jntnum: vec![0, 1, 0],
            body_pos: vec![0.0; 9],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(3),
            jnt_type: vec![3],
            jnt_bodyid: vec![1],
            jnt_qposadr: vec![0],
            jnt_dofadr: vec![0],
            jnt_pos: vec![0.0; 3],
            jnt_axis: vec![0.0, 0.0, 1.0],
        },
    )
    .unwrap();
    let inertial = InertialModelInput::new(
        kinematic,
        InertialFields {
            body_ipos: vec![0.0; 9],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(3),
            body_mass: vec![0.0, 6.0, 0.0],
            body_inertia: vec![0.0, 0.0, 0.0, 5.0, 4.0, 3.0, 0.0, 0.0, 0.0],
            dof_bodyid: vec![1],
            dof_jntid: vec![0],
            dof_parentid: vec![-1],
            dof_armature: vec![0.0],
            dof_damping: vec![0.0],
        },
    )
    .unwrap();
    AttachedModelInput::new(inertial, AttachedFields::default()).unwrap()
}

pub(super) fn simple_kinematics_parameters() -> KinematicsParameters {
    let mut parameters = KinematicsParameters::default();
    let mass = (0..7)
        .flat_map(|row| [0.0, 6.0 + 0.5 * (row % 3) as f32, 0.0])
        .collect();
    let inertia = (0..11)
        .flat_map(|row| {
            let delta = 0.125 * (row % 4) as f32;
            [0.0, 0.0, 0.0, 5.0 + delta, 4.0, 3.0, 0.0, 0.0, 0.0]
        })
        .collect();
    parameters.set(
        KinematicsParameter::BodyMass,
        ParameterBatch::new(7, 3, mass).unwrap(),
    );
    parameters.set(
        KinematicsParameter::BodyInertia,
        ParameterBatch::new(11, 9, inertia).unwrap(),
    );
    parameters
}

pub(super) fn model() -> AttachedModelInput {
    let kinematic = KinematicModelInput::new(
        2,
        KinematicFields {
            qpos0: vec![0.0; 2],
            body_parentid: vec![0, 0, 1, 2, 1],
            body_jntadr: vec![-1, 0, 1, -1, -1],
            body_jntnum: vec![0, 1, 1, 0, 0],
            body_pos: vec![
                0.0, 0.0, 0.0, 0.4, -0.2, 0.3, 0.8, 0.1, -0.15, 0.35, -0.25, 0.2, -0.3, 0.45, 0.1,
            ],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(5),
            jnt_type: vec![3, 3],
            jnt_bodyid: vec![1, 2],
            jnt_qposadr: vec![0, 1],
            jnt_dofadr: vec![0, 1],
            jnt_pos: vec![0.1, 0.0, 0.0, 0.0, 0.15, -0.05],
            jnt_axis: vec![0.0, 0.0, 1.0, 0.0, 1.0, 0.0],
        },
    )
    .unwrap();
    let inertial = InertialModelInput::new(
        kinematic,
        InertialFields {
            body_ipos: vec![
                0.0, 0.0, 0.0, 0.2, 0.05, 0.0, -0.1, 0.15, 0.05, 0.05, -0.1, 0.12, 0.0, 0.0, 0.0,
            ],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(5),
            body_mass: vec![0.0, 6.0, 4.0, 2.0, 0.0],
            body_inertia: vec![
                0.0, 0.0, 0.0, 5.0, 4.0, 3.0, 3.0, 2.5, 2.0, 2.0, 1.5, 1.0, 0.0, 0.0, 0.0,
            ],
            dof_bodyid: vec![1, 2],
            dof_jntid: vec![0, 1],
            dof_parentid: vec![-1, 0],
            dof_armature: vec![0.0; 2],
            dof_damping: vec![0.0; 2],
        },
    )
    .unwrap();
    AttachedModelInput::new(
        inertial,
        AttachedFields {
            geom_bodyid: vec![1, 1, 1, 2, 4],
            geom_pos: vec![
                0.2, 0.0, 0.1, -0.15, 0.05, 0.0, 0.0, -0.2, 0.15, 0.1, 0.0, -0.1, 0.0, 0.0, 0.0,
            ],
            geom_quat: vec![
                1.0, 0.0, 0.0, 0.0, 0.9238795, 0.0, 0.38268343, 0.0, 1.0, 0.0, 0.0, 0.0,
                0.70710677, 0.0, 0.0, 0.70710677, 1.0, 0.0, 0.0, 0.0,
            ],
            ..Default::default()
        },
    )
    .unwrap()
}

pub(super) fn simple_fluid() -> FluidForceInput {
    FluidForceInput {
        wind: ParameterBatch::new(2, 3, vec![0.0, 0.0, 0.0, 0.5, -0.25, 0.125]).unwrap(),
        density: ParameterBatch::new(3, 1, vec![0.0, 0.5, 1.0]).unwrap(),
        viscosity: ParameterBatch::new(5, 1, vec![0.0, 0.125, 0.25, 0.375, 0.5]).unwrap(),
        geom_size: ParameterBatch::new(1, 0, Vec::new()).unwrap(),
        geom_type: Vec::new(),
        geom_fluid: Vec::new(),
        body_fluid_ellipsoid: vec![false; 3],
    }
}

pub(super) fn fluid() -> FluidForceInput {
    let mut input = simple_fluid();
    input.body_fluid_ellipsoid.resize(5, false);
    input.geom_size = ParameterBatch::new(
        4,
        15,
        (0..4)
            .flat_map(|row| {
                let delta = row as f32 * 0.015625;
                [
                    0.2 + delta,
                    0.2,
                    0.2,
                    0.1,
                    0.25 + delta,
                    0.1,
                    0.3,
                    0.2,
                    0.15 + delta,
                    0.12,
                    0.22,
                    0.12,
                    0.18,
                    0.14,
                    0.1 + delta,
                ]
            })
            .collect(),
    )
    .unwrap();
    input.geom_type = vec![2, 3, 4, 5, 6];
    input.geom_fluid = vec![0.0; 60];
    input
}

pub(super) fn ellipsoid_fluid() -> FluidForceInput {
    let mut input = fluid();
    input.wind = ParameterBatch::new(2, 3, vec![0.0, 0.0, 0.0, 0.25, -0.125, 0.0625]).unwrap();
    input.density = ParameterBatch::new(3, 1, vec![1.0, 0.5, 1.25]).unwrap();
    input.viscosity = ParameterBatch::new(5, 1, vec![0.2, 0.125, 0.25, 0.375, 0.5]).unwrap();
    input.body_fluid_ellipsoid = vec![false, true, true, false, true];
    for (geom, coefficient) in [1.0, 0.75, 0.0, -1.0, 1.0].into_iter().enumerate() {
        let field = &mut input.geom_fluid[12 * geom..12 * geom + 12];
        field.copy_from_slice(&[
            coefficient,
            0.8,
            0.35,
            0.2,
            0.45,
            0.3,
            0.04,
            0.06,
            0.08,
            0.02,
            0.03,
            0.05,
        ]);
        if coefficient == 0.0 {
            field[1..].fill(1.0e6);
        }
    }
    input
}

pub(super) fn kinematics_parameters() -> KinematicsParameters {
    let mut parameters = KinematicsParameters::default();
    let mass = (0..7)
        .flat_map(|row| {
            [
                0.0,
                6.0 + 0.5 * (row % 3) as f32,
                4.0 + 0.25 * ((row + 1) % 3) as f32,
                2.0 + 0.125 * (row % 4) as f32,
                0.0,
            ]
        })
        .collect();
    let inertia = (0..11)
        .flat_map(|row| {
            let delta = 0.125 * (row % 4) as f32;
            [
                0.0,
                0.0,
                0.0,
                5.0 + delta,
                4.0,
                3.0,
                3.0,
                2.5 + 0.5 * delta,
                2.0,
                2.0,
                1.5,
                1.0 + 0.25 * delta,
                0.0,
                0.0,
                0.0,
            ]
        })
        .collect();
    parameters.set(
        KinematicsParameter::BodyMass,
        ParameterBatch::new(7, 5, mass).unwrap(),
    );
    parameters.set(
        KinematicsParameter::BodyInertia,
        ParameterBatch::new(11, 15, inertia).unwrap(),
    );
    parameters.set(
        KinematicsParameter::GeomPos,
        ParameterBatch::new(
            3,
            15,
            (0..3)
                .flat_map(|row| {
                    let delta = row as f32 * 0.03125;
                    [
                        0.2 + delta,
                        0.0,
                        0.1,
                        -0.15,
                        0.05 + delta,
                        0.0,
                        0.0,
                        -0.2,
                        0.15,
                        0.1,
                        0.0,
                        -0.1,
                        0.0,
                        0.0,
                        0.0,
                    ]
                })
                .collect(),
        )
        .unwrap(),
    );
    parameters.set(
        KinematicsParameter::GeomQuat,
        ParameterBatch::new(
            2,
            20,
            [
                1.0, 0.0, 0.0, 0.0, 0.9238795, 0.0, 0.38268343, 0.0, 1.0, 0.0, 0.0, 0.0,
                0.70710677, 0.0, 0.0, 0.70710677, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                0.8660254, 0.0, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0, 0.70710677, 0.0, 0.0, 0.70710677,
                1.0, 0.0, 0.0, 0.0,
            ]
            .to_vec(),
        )
        .unwrap(),
    );
    parameters
}
