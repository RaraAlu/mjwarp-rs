//! 测试共用模型与计划构造。

use super::*;

use crate::model::{
    AttachedFields, InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
};

#[cfg(feature = "cuda-probe")]
pub(super) fn flex_hessian_plan(session: &TransferSession) -> KinematicsPlan {
    let cam = CamLightModelInput::new(
        MocapModelInput::new(parameter_model(true), vec![-1; 2]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let mut nodes = Vec::new();
    for _ in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    nodes.extend([x as f32, y as f32, z as f32]);
                }
            }
        }
    }
    let mut faces = FlexFaceFields::default();
    for f in 0..6 {
        faces.flex_face_map.extend([2, f]);
        faces.flex_face.extend(
            crate::model::flex::face_nodes(&[1; 3], f as usize)
                .1
                .map(|n| n + 8),
        );
        faces.flex_face.extend([-1; 5]);
    }
    let flex = FlexPositionModelInput::new(
        TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
        FlexPositionFields {
            flex_interp: vec![0, 1, -1],
            flex_cellnum: vec![1; 9],
            flex_nodeadr: vec![0, 0, 8],
            flex_nodenum: vec![0, 8, 8],
            flex_vertadr: vec![0, 2, 4],
            flex_vertnum: vec![2; 3],
            flex_centered: vec![false; 3],
            flex_nodebodyid: vec![1; 16],
            flex_vertbodyid: vec![1; 6],
            flex_node: nodes,
            flex_vert: [0.2, 0.3, 0.4, 0.7, 0.8, 0.9].repeat(3),
            flex_vert0: [0.2, 0.3, 0.4, 0.7, 0.8, 0.9].repeat(3),
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0, 1, 2],
        flex_edgenum: vec![1; 3],
        flex_edge: [0, 1].repeat(3),
        flexedge_j_rowadr: vec![0; 3],
        flexedge_j_rownnz: vec![0; 3],
        flexedge_j_colind: vec![],
    })
    .unwrap()
    .with_faces(faces)
    .unwrap()
    .with_hessian(FlexHessianFields {
        flex_dim: vec![1; 3],
        flex_rigid: vec![true, false, false],
        flex_elemadr: vec![0, 1, 2],
        flex_elemnum: vec![1; 3],
        flex_elemdataadr: vec![0, 2, 4],
        flex_elemedgeadr: vec![0, 1, 2],
        flex_stiffnessadr: vec![-1; 3],
        flex_elem: [0, 1].repeat(3),
        flex_elemedge: vec![0; 3],
        flexedge_length0: vec![1.0; 3],
        flex_stiffness: vec![],
    })
    .unwrap();
    let wake = TendonWakeModelInput::with_flex_positions(
        flex,
        crate::model::TendonWakeFields {
            body_treeid: vec![-1, 0],
            tendon_limited: vec![],
            tendon_range: crate::model::ParameterBatch::new(3, 0, vec![]).unwrap(),
            tendon_margin: crate::model::ParameterBatch::new(2, 0, vec![]).unwrap(),
            sleep_enabled: true,
            island_disabled: false,
        },
    )
    .unwrap();
    KinematicsPlan::with_tendon_wake(
        session,
        wake,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}

#[cfg(feature = "cuda-probe")]
pub(super) fn stretch_plan(session: &TransferSession) -> KinematicsPlan {
    let cam = CamLightModelInput::new(
        MocapModelInput::new(parameter_model(true), vec![-1; 2]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let mut stiffness = vec![0.0; 42];
    for base in [0, 21] {
        for k in [0, 3, 5] {
            stiffness[base + k] = 1.0;
        }
    }
    let input = FlexPositionModelInput::new(
        TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
        FlexPositionFields {
            flex_interp: vec![0; 2],
            flex_cellnum: vec![1; 6],
            flex_nodeadr: vec![0; 2],
            flex_nodenum: vec![0; 2],
            flex_vertadr: vec![0, 3],
            flex_vertnum: vec![3; 2],
            flex_centered: vec![false; 2],
            flex_nodebodyid: vec![],
            flex_node: vec![],
            flex_vertbodyid: [0, 1, 1].repeat(2),
            flex_vert: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0].repeat(2),
            flex_vert0: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0].repeat(2),
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0, 3],
        flex_edgenum: vec![3; 2],
        flex_edge: [1, 2, 2, 0, 0, 1].repeat(2),
        flexedge_j_rowadr: vec![0; 6],
        flexedge_j_rownnz: vec![0; 6],
        flexedge_j_colind: vec![],
    })
    .unwrap()
    .with_hessian(FlexHessianFields {
        flex_dim: vec![2; 2],
        flex_rigid: vec![false; 2],
        flex_elemadr: vec![0, 1],
        flex_elemnum: vec![1; 2],
        flex_elemdataadr: vec![0, 3],
        flex_elemedgeadr: vec![0, 3],
        flex_stiffnessadr: vec![0, 21],
        flex_elem: [0, 1, 2].repeat(2),
        flex_elemedge: [0, 1, 2].repeat(2),
        flexedge_length0: [2.0f32.sqrt(), 1.0, 1.0].repeat(2),
        flex_stiffness: stiffness,
    })
    .unwrap();
    KinematicsPlan::with_flex_positions(
        session,
        input,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}

pub(super) fn empty_model() -> AttachedModelInput {
    let k = KinematicModelInput::new(
        0,
        KinematicFields {
            body_parentid: vec![0],
            body_jntadr: vec![-1],
            body_jntnum: vec![0],
            body_pos: vec![0.0; 3],
            body_quat: vec![1.0, 0.0, 0.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap();
    let i = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 3],
            body_iquat: vec![1.0, 0.0, 0.0, 0.0],
            body_mass: vec![0.0],
            body_inertia: vec![0.0; 3],
            ..Default::default()
        },
    )
    .unwrap();
    AttachedModelInput::new(i, AttachedFields::default()).unwrap()
}

pub(super) fn parameter_model(free: bool) -> AttachedModelInput {
    let k = KinematicModelInput::new(
        if free { 6 } else { 1 },
        KinematicFields {
            qpos0: if free {
                vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]
            } else {
                vec![0.0]
            },
            body_parentid: vec![0, 0],
            body_jntadr: vec![-1, 0],
            body_jntnum: vec![0, 1],
            body_pos: vec![0.0; 6],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            jnt_type: vec![if free { 0 } else { 2 }],
            jnt_bodyid: vec![1],
            jnt_qposadr: vec![0],
            jnt_dofadr: vec![0],
            jnt_pos: vec![0.0; 3],
            jnt_axis: vec![1.0, 0.0, 0.0],
        },
    )
    .unwrap();
    let nv = k.nv();
    let rigid = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 6],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            body_mass: vec![0.0, 2.0],
            body_inertia: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            dof_bodyid: vec![1; nv],
            dof_jntid: vec![0; nv],
            dof_parentid: (0..nv).map(|i| i as i32 - 1).collect(),
            dof_armature: vec![0.0; nv],
            dof_damping: vec![0.0; nv],
        },
    )
    .unwrap();
    AttachedModelInput::new(
        rigid,
        AttachedFields {
            geom_bodyid: vec![1],
            geom_pos: vec![0.0; 3],
            geom_quat: vec![1.0, 0.0, 0.0, 0.0],
            site_bodyid: vec![1],
            site_pos: vec![0.0; 3],
            site_quat: vec![1.0, 0.0, 0.0, 0.0],
        },
    )
    .unwrap()
}

#[cfg(feature = "cuda-probe")]
pub(super) fn fixed_tendon_plan(session: &TransferSession, coefficient: f32) -> KinematicsPlan {
    let camlight = CamLightModelInput::new(
        MocapModelInput::new(parameter_model(false), vec![-1; 2]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let model = FixedTendonModelInput::new(
        camlight,
        FixedTendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![1],
            wrap_type: vec![1],
            wrap_objid: vec![0],
            wrap_prm: vec![coefficient],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![1],
            ten_j_colind: vec![0],
        },
    )
    .unwrap();
    KinematicsPlan::with_fixed_tendons(
        session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}

#[cfg(feature = "cuda-probe")]
pub(super) fn spatial_tendon_plan(session: &TransferSession) -> KinematicsPlan {
    let base = parameter_model(false);
    let mut fields = base.fields().clone();
    fields.site_bodyid = vec![0, 1];
    fields.site_pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    fields.site_quat = [1.0, 0.0, 0.0, 0.0].repeat(2);
    let attached = AttachedModelInput::new(base.rigid().clone(), fields).unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(attached, vec![-1; 2]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let fixed = FixedTendonModelInput::new(cam, FixedTendonFields::default()).unwrap();
    let model = SpatialTendonModelInput::new(
        fixed,
        SpatialTendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![2],
            wrap_type: vec![3, 3],
            wrap_objid: vec![0, 1],
            wrap_prm: vec![0.0; 2],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![1],
            ten_j_colind: vec![0],
        },
    )
    .unwrap();
    KinematicsPlan::with_spatial_tendons(
        session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}

#[cfg(feature = "cuda-probe")]
pub(super) fn flex_face_plan(
    session: &TransferSession,
    nodes: Vec<f32>,
    cells: [i32; 3],
) -> KinematicsPlan {
    let cam = CamLightModelInput::new(
        MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let tendons = TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap();
    let count = nodes.len() / 3;
    let mut faces = FlexFaceFields::default();
    let nf = 2 * (cells[0] * cells[1] + cells[0] * cells[2] + cells[1] * cells[2]);
    for f in 0..nf {
        faces.flex_face_map.extend([0, f]);
        faces
            .flex_face
            .extend(crate::model::flex::face_nodes(&cells, f as usize).1);
        faces.flex_face.extend([-1; 5]);
    }
    let model = FlexPositionModelInput::new(
        tendons,
        FlexPositionFields {
            flex_interp: vec![-1],
            flex_cellnum: cells.to_vec(),
            flex_nodeadr: vec![0],
            flex_nodenum: vec![count as i32],
            flex_vertadr: vec![0],
            flex_vertnum: vec![1],
            flex_centered: vec![false],
            flex_nodebodyid: vec![0; count],
            flex_vertbodyid: vec![-1],
            flex_node: nodes,
            flex_vert: vec![0.0; 3],
            flex_vert0: vec![0.5; 3],
        },
    )
    .unwrap()
    .with_faces(faces)
    .unwrap();
    KinematicsPlan::with_flex_positions(
        session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}

/// 自由、球关节与mocap体共用退化四元数输入。
#[cfg(feature = "cuda-probe")]
pub(super) fn degenerate_quaternion_plan(
    session: &TransferSession,
    equivalent: bool,
) -> KinematicsPlan {
    let k = KinematicModelInput::new(
        9,
        KinematicFields {
            qpos0: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            body_parentid: vec![0, 0, 1, 0],
            body_jntadr: vec![-1, 0, 1, -1],
            body_jntnum: vec![0, 1, 1, 0],
            body_pos: vec![0.0, 0.0, 0.0, 0.1, 0.2, 0.3, 0.0, 0.0, 0.5, 1.0, 2.0, 3.0],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(4),
            jnt_type: vec![0, 1],
            jnt_bodyid: vec![1, 2],
            jnt_qposadr: vec![0, 7],
            jnt_dofadr: vec![0, 6],
            jnt_pos: vec![0.0, 0.0, 0.0, 0.05, 0.0, 0.0],
            jnt_axis: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
        },
    )
    .unwrap();
    let rigid = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.0, 0.1, 0.0, 0.0, 0.0, 0.0],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(4),
            body_mass: vec![0.0, 1.0, 2.0, 0.5],
            body_inertia: [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0].to_vec(),
            dof_bodyid: vec![1, 1, 1, 1, 1, 1, 2, 2, 2],
            dof_jntid: vec![0, 0, 0, 0, 0, 0, 1, 1, 1],
            dof_parentid: (0..9).map(|dof| dof - 1).collect(),
            dof_armature: vec![0.0; 9],
            dof_damping: vec![0.0; 9],
        },
    )
    .unwrap();
    let attached = AttachedModelInput::new(
        rigid,
        AttachedFields {
            geom_bodyid: vec![],
            geom_pos: vec![],
            geom_quat: vec![],
            site_bodyid: vec![],
            site_pos: vec![],
            site_quat: vec![],
        },
    )
    .unwrap();
    let mocap = MocapModelInput::new(attached, vec![-1, -1, -1, 0]).unwrap();
    let camlight = if equivalent {
        CamLightModelInput::new_equivalent(mocap, CamLightFields::default())
    } else {
        CamLightModelInput::new(mocap, CamLightFields::default())
    }
    .unwrap();
    let fixed = FixedTendonModelInput::new(camlight, FixedTendonFields::default()).unwrap();
    let model = SpatialTendonModelInput::new(fixed, SpatialTendonFields::default()).unwrap();
    KinematicsPlan::with_tendon_subsets(
        session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
        OptionalSubsets {
            equivalent,
            ..Default::default()
        },
    )
    .unwrap()
}

/// 每世界依次为正零、负零、下溢与单位四元数。
#[cfg(feature = "cuda-probe")]
pub(super) fn degenerate_quaternion_state() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let quats: [[f32; 4]; 4] = [
        [0.0, 0.0, 0.0, 0.0],
        [-0.0, -0.0, 0.0, -0.0],
        [1.0e-30, 0.0, -0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
    ];
    let mut qpos = Vec::new();
    let mut mocap_pos = Vec::new();
    let mut mocap_quat = Vec::new();
    for world in 0..4 {
        let free = quats[world];
        let ball = quats[(world + 1) % 4];
        qpos.extend([0.25 * world as f32, -0.5, 0.75]);
        qpos.extend(free);
        qpos.extend(ball);
        mocap_pos.extend([1.0, -2.0 + world as f32, 0.5]);
        mocap_quat.extend(quats[(world + 2) % 4]);
    }
    (qpos, mocap_pos, mocap_quat)
}
