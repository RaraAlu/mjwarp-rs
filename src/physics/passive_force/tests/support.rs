//! 被动合力测试共用模型与参数构造。

use super::*;

use crate::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexEdgeFields,
    FlexPositionFields, FlexPositionModelInput, InertialFields, InertialModelInput,
    KinematicFields, KinematicModelInput, MocapModelInput, ParameterBatch, TendonFields,
    TendonModelInput,
};
use crate::physics::{JointPassiveParameters, TendonPassiveParameters};

pub(super) fn kinematic(axis: [f32; 3]) -> KinematicModelInput {
    KinematicModelInput::new(
        1,
        KinematicFields {
            qpos0: vec![0.0],
            body_parentid: vec![0, 0],
            body_jntadr: vec![-1, 0],
            body_jntnum: vec![0, 1],
            body_pos: vec![0.0; 6],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            jnt_type: vec![2],
            jnt_bodyid: vec![1],
            jnt_qposadr: vec![0],
            jnt_dofadr: vec![0],
            jnt_pos: vec![0.0; 3],
            jnt_axis: axis.to_vec(),
        },
    )
    .unwrap()
}

pub(super) fn flex_model(model: KinematicModelInput) -> FlexPositionModelInput {
    let inertial = InertialModelInput::new(
        model,
        InertialFields {
            body_ipos: vec![0.0; 6],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            body_mass: vec![0.0, 1.0],
            body_inertia: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            dof_bodyid: vec![1],
            dof_jntid: vec![0],
            dof_parentid: vec![-1],
            dof_armature: vec![0.0],
            dof_damping: vec![0.0],
        },
    )
    .unwrap();
    let attached = AttachedModelInput::new(inertial, AttachedFields::default()).unwrap();
    let camlight = CamLightModelInput::new(
        MocapModelInput::new(attached, vec![-1; 2]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let tendons = TendonModelInput::new(
        camlight,
        TendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![1],
            wrap_type: vec![1],
            wrap_objid: vec![0],
            wrap_prm: vec![1.0],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![1],
            ten_j_colind: vec![0],
        },
    )
    .unwrap();
    FlexPositionModelInput::new(
        tendons,
        FlexPositionFields {
            flex_interp: vec![0],
            flex_cellnum: vec![1; 3],
            flex_nodeadr: vec![0],
            flex_nodenum: vec![0],
            flex_vertadr: vec![0],
            flex_vertnum: vec![2],
            flex_centered: vec![false],
            flex_nodebodyid: vec![],
            flex_vertbodyid: vec![0, 1],
            flex_node: vec![],
            flex_vert: vec![0.0; 6],
            flex_vert0: vec![0.0; 6],
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0],
        flex_edgenum: vec![1],
        flex_edge: vec![0, 1],
        flexedge_j_rowadr: vec![0],
        flexedge_j_rownnz: vec![1],
        flexedge_j_colind: vec![0],
    })
    .unwrap()
}

pub(super) fn sleep_kinematic() -> KinematicModelInput {
    KinematicModelInput::new(
        2,
        KinematicFields {
            qpos0: vec![0.0; 2],
            body_parentid: vec![0, 0, 0],
            body_jntadr: vec![-1, 0, 1],
            body_jntnum: vec![0, 1, 1],
            body_pos: vec![0.0; 9],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(3),
            jnt_type: vec![2, 2],
            jnt_bodyid: vec![1, 2],
            jnt_qposadr: vec![0, 1],
            jnt_dofadr: vec![0, 1],
            jnt_pos: vec![0.0; 6],
            jnt_axis: [1.0, 0.0, 0.0].repeat(2),
        },
    )
    .unwrap()
}

pub(super) fn sleep_flex_model(model: KinematicModelInput) -> FlexPositionModelInput {
    let inertial = InertialModelInput::new(
        model,
        InertialFields {
            body_ipos: vec![0.0; 9],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(3),
            body_mass: vec![0.0, 1.0, 1.0],
            body_inertia: vec![0.0; 9],
            dof_bodyid: vec![1, 2],
            dof_jntid: vec![0, 1],
            dof_parentid: vec![-1, -1],
            dof_armature: vec![0.0; 2],
            dof_damping: vec![0.0; 2],
        },
    )
    .unwrap();
    let attached = AttachedModelInput::new(inertial, AttachedFields::default()).unwrap();
    let camlight = CamLightModelInput::new(
        MocapModelInput::new(attached, vec![-1; 3]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let tendons = TendonModelInput::new(
        camlight,
        TendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![1],
            wrap_type: vec![1],
            wrap_objid: vec![0],
            wrap_prm: vec![1.0],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![1],
            ten_j_colind: vec![0],
        },
    )
    .unwrap();
    FlexPositionModelInput::new(
        tendons,
        FlexPositionFields {
            flex_interp: vec![0],
            flex_cellnum: vec![1; 3],
            flex_nodeadr: vec![0],
            flex_nodenum: vec![0],
            flex_vertadr: vec![0],
            flex_vertnum: vec![2],
            flex_centered: vec![false],
            flex_nodebodyid: vec![],
            flex_vertbodyid: vec![1, 2],
            flex_node: vec![],
            flex_vert: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            flex_vert0: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0],
        flex_edgenum: vec![1],
        flex_edge: vec![0, 1],
        flexedge_j_rowadr: vec![0],
        flexedge_j_rownnz: vec![2],
        flexedge_j_colind: vec![0, 1],
    })
    .unwrap()
}

pub(super) fn sleep_joint_parameters() -> JointPassiveParameters {
    JointPassiveParameters {
        qpos_spring: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
        stiffness: ParameterBatch::new(1, 2, vec![-2.0, -3.0]).unwrap(),
        stiffness_poly: ParameterBatch::new(1, 4, vec![0.0; 4]).unwrap(),
        damping: ParameterBatch::new(1, 2, vec![0.5, 0.75]).unwrap(),
        damping_poly: ParameterBatch::new(1, 4, vec![0.0; 4]).unwrap(),
    }
}

pub(super) fn rows(
    period: usize,
    width: usize,
    mut row: impl FnMut(usize) -> Vec<f32>,
) -> ParameterBatch {
    let mut values = Vec::with_capacity(period * width);
    for index in 0..period {
        let row = row(index);
        assert_eq!(row.len(), width);
        values.extend(row);
    }
    ParameterBatch::new(period, width, values).unwrap()
}

pub(super) fn joint_parameters() -> JointPassiveParameters {
    JointPassiveParameters {
        qpos_spring: rows(2, 1, |row| vec![row as f32 / 4.0]),
        stiffness: rows(3, 1, |row| {
            vec![if row == 0 {
                -16_777_216.0
            } else {
                -(8 << row) as f32
            }]
        }),
        stiffness_poly: rows(5, 2, |row| vec![0.0, row as f32 / 32.0]),
        damping: rows(7, 1, |row| vec![row as f32 / 16.0]),
        damping_poly: rows(11, 2, |row| vec![row as f32 / 64.0, 0.0]),
    }
}

pub(super) fn tendon_parameters() -> TendonPassiveParameters {
    TendonPassiveParameters {
        stiffness: rows(2, 1, |row| vec![-1.0 - row as f32]),
        stiffness_poly: rows(3, 2, |row| vec![row as f32 / 32.0, 0.0]),
        damping: rows(5, 1, |row| vec![row as f32 / 16.0]),
        damping_poly: rows(7, 2, |row| vec![row as f32 / 64.0, 0.0]),
        length_spring: rows(11, 2, |row| vec![0.0, row as f32 / 64.0]),
    }
}

pub(super) fn fluid_input(nbody: usize) -> FluidForceInput {
    FluidForceInput {
        wind: ParameterBatch::new(2, 3, vec![0.0, 0.0, 0.0, 0.25, -0.125, 0.0625]).unwrap(),
        density: ParameterBatch::new(3, 1, vec![0.5, 1.0, 0.25]).unwrap(),
        viscosity: ParameterBatch::new(5, 1, vec![0.125, 0.25, 0.375, 0.5, 0.625]).unwrap(),
        geom_size: ParameterBatch::new(1, 0, Vec::new()).unwrap(),
        geom_type: Vec::new(),
        geom_fluid: Vec::new(),
        body_fluid_ellipsoid: vec![false; nbody],
    }
}

pub(super) fn ordered_atomic_sources(mut value: f32, tendon: f32, flex_edge: f32) -> f32 {
    if tendon != 0.0 {
        value += tendon;
    }
    if flex_edge != 0.0 {
        value += flex_edge;
    }
    value
}

pub(super) fn rotation_kinematic() -> KinematicModelInput {
    let mut qpos0 = vec![0.0; 13];
    qpos0[3] = 1.0;
    qpos0[7] = 1.0;
    let mut body_pos = vec![0.0; 15];
    body_pos[6] = 1.0;
    body_pos[12] = 1.0;
    KinematicModelInput::new(
        11,
        KinematicFields {
            qpos0,
            body_parentid: vec![0, 0, 1, 0, 0],
            body_jntadr: vec![-1, 0, 1, 2, 3],
            body_jntnum: vec![0, 1, 1, 1, 1],
            body_pos,
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(5),
            jnt_type: vec![0, 1, 2, 3],
            jnt_bodyid: vec![1, 2, 3, 4],
            jnt_qposadr: vec![0, 7, 11, 12],
            jnt_dofadr: vec![0, 6, 9, 10],
            jnt_pos: vec![0.0; 12],
            jnt_axis: [0.0, 0.0, 1.0].repeat(4),
        },
    )
    .unwrap()
}

pub(super) fn rotation_flex_model(model: KinematicModelInput) -> FlexPositionModelInput {
    let inertial = InertialModelInput::new(
        model,
        InertialFields {
            body_ipos: vec![0.0; 15],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(5),
            body_mass: vec![0.0, 1.0, 2.0, 1.0, 1.0],
            body_inertia: vec![0.0; 15],
            dof_bodyid: vec![1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 4],
            dof_jntid: vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 3],
            dof_parentid: vec![-1, 0, 1, 2, 3, 4, 5, 6, 7, -1, -1],
            dof_armature: vec![0.0; 11],
            dof_damping: vec![0.0; 11],
        },
    )
    .unwrap();
    let attached = AttachedModelInput::new(inertial, AttachedFields::default()).unwrap();
    let camlight = CamLightModelInput::new(
        MocapModelInput::new(attached, vec![-1; 5]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let tendons = TendonModelInput::new(
        camlight,
        TendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![1],
            wrap_type: vec![1],
            wrap_objid: vec![2],
            wrap_prm: vec![1.0],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![1],
            ten_j_colind: vec![9],
        },
    )
    .unwrap();
    FlexPositionModelInput::new(
        tendons,
        FlexPositionFields {
            flex_interp: vec![0],
            flex_cellnum: vec![1; 3],
            flex_nodeadr: vec![0],
            flex_nodenum: vec![0],
            flex_vertadr: vec![0],
            flex_vertnum: vec![2],
            flex_centered: vec![false],
            flex_nodebodyid: vec![],
            flex_vertbodyid: vec![3, 4],
            flex_node: vec![],
            flex_vert: vec![0.0; 6],
            flex_vert0: vec![0.0; 6],
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0],
        flex_edgenum: vec![1],
        flex_edge: vec![0, 1],
        flexedge_j_rowadr: vec![0],
        flexedge_j_rownnz: vec![2],
        flexedge_j_colind: vec![9, 10],
    })
    .unwrap()
}

pub(super) fn rotation_joint_parameters() -> JointPassiveParameters {
    JointPassiveParameters {
        qpos_spring: ParameterBatch::new(1, 13, rotation_kinematic().fields().qpos0.clone())
            .unwrap(),
        stiffness: ParameterBatch::new(1, 4, vec![0.5, 0.75, -2.0, -1.0]).unwrap(),
        stiffness_poly: ParameterBatch::new(1, 8, vec![0.0; 8]).unwrap(),
        damping: ParameterBatch::new(1, 11, vec![0.125; 11]).unwrap(),
        damping_poly: ParameterBatch::new(1, 22, vec![0.0; 22]).unwrap(),
    }
}
