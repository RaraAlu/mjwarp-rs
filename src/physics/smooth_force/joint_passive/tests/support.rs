//! 关节被动测试共用模型与参数构造。

use super::*;
pub(super) fn model() -> KinematicModelInput {
    use crate::model::KinematicFields;
    let mut qpos0 = vec![0.0; 13];
    qpos0[3] = 1.0;
    qpos0[7] = 1.0;
    KinematicModelInput::new(
        11,
        KinematicFields {
            qpos0,
            body_parentid: vec![0; 5],
            body_jntadr: vec![-1, 0, 1, 2, 3],
            body_jntnum: vec![0, 1, 1, 1, 1],
            body_pos: vec![0.0; 15],
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
pub(super) fn params(periods: [usize; 5]) -> JointPassiveParameters {
    let mut q = Vec::new();
    for r in 0..periods[0] {
        let mut row = model().fields().qpos0.clone();
        row[11] = (r as f32 - 1.0) / 16.0;
        row[12] = -(r as f32) / 8.0;
        q.extend(row);
    }
    let mut s = Vec::new();
    for r in 0..periods[1] {
        s.extend([0.0, 0.0, (r % 3) as f32 / 4.0, 0.5 + (r % 3) as f32 / 16.0]);
    }
    let mut sp = Vec::new();
    for r in 0..periods[2] {
        sp.extend([
            0.0,
            0.0,
            0.0,
            0.0,
            0.125 + (r % 2) as f32 / 16.0,
            0.0625,
            -0.0625,
            0.125,
        ]);
    }
    let mut damp = Vec::new();
    for r in 0..periods[3] {
        for i in 0..11 {
            damp.push(if i % 5 == 0 || i % 3 == 0 {
                0.0
            } else {
                0.25 + ((i + r) % 4) as f32 / 8.0
            });
        }
    }
    let mut dp = Vec::new();
    for r in 0..periods[4] {
        for i in 0..11 {
            dp.extend(if i % 5 == 0 {
                [0.0, 0.0]
            } else {
                [
                    if i % 3 == 0 {
                        0.125 + (r % 2) as f32 / 16.0
                    } else {
                        0.0
                    },
                    if i % 4 == 0 { 0.0625 } else { 0.0 },
                ]
            });
        }
    }
    JointPassiveParameters {
        qpos_spring: ParameterBatch::new(periods[0], 13, q).unwrap(),
        stiffness: ParameterBatch::new(periods[1], 4, s).unwrap(),
        stiffness_poly: ParameterBatch::new(periods[2], 8, sp).unwrap(),
        damping: ParameterBatch::new(periods[3], 11, damp).unwrap(),
        damping_poly: ParameterBatch::new(periods[4], 22, dp).unwrap(),
    }
}
