//! 肌腱速度测试共用模型与参数工具。

#[cfg(feature = "cuda-probe")]
use super::*;

use crate::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, InertialFields,
    InertialModelInput, KinematicFields, KinematicModelInput, MocapModelInput, TendonFields,
    TendonModelInput,
};
#[cfg(feature = "cuda-probe")]
use crate::model::{CamLightParameters, KinematicsParameters, ParameterBatch};

pub(super) fn coefficient(j: usize) -> f32 {
    (j as i32 % 5 - 2) as f32 / 8.0
}
pub(super) fn model(nv: usize) -> TendonModelInput {
    let k = KinematicModelInput::new(
        nv,
        KinematicFields {
            qpos0: vec![0.0; nv],
            body_parentid: vec![0; nv + 1],
            body_jntadr: std::iter::once(-1).chain(0..nv as i32).collect(),
            body_jntnum: std::iter::once(0)
                .chain(std::iter::repeat_n(1, nv))
                .collect(),
            body_pos: vec![0.0; 3 * (nv + 1)],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(nv + 1),
            jnt_type: vec![2; nv],
            jnt_bodyid: (1..=nv as i32).collect(),
            jnt_qposadr: (0..nv as i32).collect(),
            jnt_dofadr: (0..nv as i32).collect(),
            jnt_pos: vec![0.0; 3 * nv],
            jnt_axis: [1.0, 0.0, 0.0].repeat(nv),
        },
    )
    .unwrap();
    let rigid = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 3 * (nv + 1)],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(nv + 1),
            body_mass: std::iter::once(0.0)
                .chain(std::iter::repeat_n(1.0, nv))
                .collect(),
            body_inertia: std::iter::once([0.0; 3])
                .chain(std::iter::repeat_n([1.0; 3], nv))
                .flatten()
                .collect(),
            dof_bodyid: (1..=nv as i32).collect(),
            dof_jntid: (0..nv as i32).collect(),
            dof_parentid: vec![-1; nv],
            dof_armature: vec![0.0; nv],
            dof_damping: vec![0.0; nv],
        },
    )
    .unwrap();
    let attached = AttachedModelInput::new(
        rigid,
        AttachedFields {
            site_bodyid: vec![1, 2, 0, 0],
            site_pos: vec![0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0],
            site_quat: [1.0, 0.0, 0.0, 0.0].repeat(4),
            ..Default::default()
        },
    )
    .unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(attached, vec![-1; nv + 1]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    TendonModelInput::new(
        cam,
        TendonFields {
            tendon_adr: vec![0, 2, (2 + nv) as i32, (3 + nv) as i32, (4 + nv) as i32],
            tendon_num: vec![2, nv as i32, 1, 1, 2],
            wrap_type: [vec![3, 3], vec![1; nv + 2], vec![3, 3]].concat(),
            wrap_objid: [
                vec![0, 1],
                (0..nv as i32).collect(),
                vec![1, nv as i32 - 1],
                vec![2, 3],
            ]
            .concat(),
            wrap_prm: [
                vec![0.0; 2],
                (0..nv).map(coefficient).collect(),
                vec![0.0, 1.0],
                vec![0.0; 2],
            ]
            .concat(),
            ten_j_rowadr: vec![
                0,
                nv as i32,
                2 * nv as i32,
                2 * nv as i32 + 1,
                2 * nv as i32 + 2,
            ],
            ten_j_rownnz: vec![nv as i32, nv as i32, 1, 1, 0],
            ten_j_colind: [
                (0..nv as i32).collect::<Vec<_>>(),
                (0..nv as i32).collect(),
                vec![1, nv as i32 - 1],
            ]
            .concat(),
        },
    )
    .unwrap()
}

#[cfg(feature = "cuda-probe")]
pub(super) fn producer(session: &TransferSession, nv: usize) -> KinematicsPlan {
    KinematicsPlan::with_tendons(
        session,
        model(nv),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn raw(b: &TransferBuffer<f32>) -> Vec<f32> {
    let mut v = vec![0.0; b.len()];
    b.read_range_into(0, &mut v).unwrap();
    v
}
#[cfg(feature = "cuda-probe")]
pub(super) fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|v| v.to_bits()).collect()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn expected(v: &[f32]) -> Vec<f32> {
    let mut fixed = 0.0_f64;
    for (j, &x) in v.iter().enumerate() {
        fixed += coefficient(j) as f64 * x as f64;
    }
    let values = [
        0.0_f64 - v[0] as f64 + v[1] as f64,
        fixed,
        0.0,
        0.0 + v[v.len() - 1] as f64,
        0.0,
    ];
    values
        .into_iter()
        .map(|x| {
            assert_eq!(x as f32 as f64, x);
            x as f32
        })
        .collect()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn passive_repeated(period: usize, width: usize, row: &[f32]) -> ParameterBatch {
    assert_eq!(width, row.len());
    ParameterBatch::new(period, width, row.repeat(period)).unwrap()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn passive_parameters(periods: [usize; 5]) -> crate::physics::TendonPassiveParameters {
    let mut stiffness = vec![0.0; 5];
    stiffness[0] = 0.25;
    let mut stiffness_poly = vec![0.0; 10];
    stiffness_poly[0] = 0.125;
    stiffness_poly[1] = 0.0625;
    let mut damping = vec![0.0; 5];
    damping[0] = 0.125;
    let mut damping_poly = vec![0.0; 10];
    damping_poly[0] = 0.125;
    damping_poly[1] = 0.0625;
    let mut spring = vec![0.0; 10];
    spring[1] = 1.0;
    crate::physics::TendonPassiveParameters {
        stiffness: passive_repeated(periods[0], 5, &stiffness),
        stiffness_poly: passive_repeated(periods[1], 10, &stiffness_poly),
        damping: passive_repeated(periods[2], 5, &damping),
        damping_poly: passive_repeated(periods[3], 10, &damping_poly),
        length_spring: passive_repeated(periods[4], 10, &spring),
    }
}
#[cfg(feature = "cuda-probe")]
pub(super) fn varying_passive_parameters() -> crate::physics::TendonPassiveParameters {
    let scalar = |period: usize, base: f32| {
        let mut values = Vec::with_capacity(5 * period);
        for row in 0..period {
            let mut field = vec![0.0; 5];
            field[0] = base + row as f32 / 64.0;
            values.extend(field);
        }
        ParameterBatch::new(period, 5, values).unwrap()
    };
    let poly = |period: usize| {
        let mut values = Vec::with_capacity(10 * period);
        for row in 0..period {
            let mut field = vec![0.0; 10];
            field[0] = row as f32 / 64.0;
            values.extend(field);
        }
        ParameterBatch::new(period, 10, values).unwrap()
    };
    let mut lengths = Vec::with_capacity(110);
    for row in 0..11 {
        let mut field = vec![0.0; 10];
        field[1] = 1.0 + row as f32 / 32.0;
        lengths.extend(field);
    }
    crate::physics::TendonPassiveParameters {
        stiffness: scalar(2, 0.25),
        stiffness_poly: poly(3),
        damping: scalar(5, 0.125),
        damping_poly: poly(7),
        length_spring: ParameterBatch::new(11, 10, lengths).unwrap(),
    }
}
#[cfg(feature = "cuda-probe")]
pub(super) fn empty_row_passive_parameters() -> crate::physics::TendonPassiveParameters {
    let mut stiffness = vec![0.0; 5];
    stiffness[4] = 0.25;
    let mut damping = vec![0.0; 5];
    damping[4] = 0.125;
    crate::physics::TendonPassiveParameters {
        stiffness: ParameterBatch::new(1, 5, stiffness).unwrap(),
        stiffness_poly: ParameterBatch::new(1, 10, vec![0.0; 10]).unwrap(),
        damping: ParameterBatch::new(1, 5, damping).unwrap(),
        damping_poly: ParameterBatch::new(1, 10, vec![0.0; 10]).unwrap(),
        length_spring: ParameterBatch::new(1, 10, vec![0.0; 10]).unwrap(),
    }
}
