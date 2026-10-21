//! G01 非单位参数与零四元数接受边界。

use mjwarp_rs::model::{
    CamLightParameter, CamLightParameters, KinematicsParameter, KinematicsParameters,
    ParameterBatch,
};
use mjwarp_rs::physics::{KinematicsPlan, fwd_kinematics};
use mjwarp_rs::runtime::TransferSession;

use crate::support::{fixture, input, state};

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn g01_accepts_finite_nonunit_parameters_and_zero_state_quaternions() {
    let v = fixture();
    let model = input();
    let attached = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .attached();
    let k = attached.rigid().kinematics();
    let fields = k.fields().clone();
    let mut params = KinematicsParameters::default();
    let values = fields
        .body_quat
        .iter()
        .map(|&q| q * 2.0)
        .collect::<Vec<_>>();
    params.set(
        KinematicsParameter::BodyQuat,
        ParameterBatch::new(1, values.len(), values).unwrap(),
    );
    let mut cams = CamLightParameters::default();
    let values = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .fields()
        .cam_quat
        .iter()
        .map(|&q| q * 2.0)
        .collect::<Vec<_>>();
    cams.set(
        CamLightParameter::CamQuat,
        ParameterBatch::new(1, values.len(), values).unwrap(),
    );
    let session = TransferSession::new(0).unwrap();
    assert!(
        KinematicsPlan::with_tendon_wake(&session, model.clone(), params.clone(), cams.clone())
            .is_err()
    );
    let p = KinematicsPlan::for_g01(&session, model, params, cams).unwrap();
    let mut d = p.create_data(1).unwrap();
    let mut state = state(&v, 1, 0);
    state.mocap_quat.fill(0.0);
    for (j, &kind) in fields.jnt_type.iter().enumerate() {
        if kind <= 1 {
            let start = fields.jnt_qposadr[j] as usize + if kind == 0 { 3 } else { 0 };
            state.qpos[start..start + 4].fill(0.0);
        }
    }
    p.import_g01_state(&mut d, &state).unwrap();
    fwd_kinematics(&p, &mut d).unwrap();
    let out = d.readback().unwrap();
    let r = out.rigid().world(0).unwrap();
    for (j, &kind) in fields.jnt_type.iter().enumerate() {
        if kind <= 1 {
            let b = fields.jnt_bodyid[j] as usize;
            // 冻结场景的球关节直属世界体，局部旋转为单位量。
            // Warp按存储槽返回末位一，不返回MJWarp单位旋转。
            if kind == 1 {
                assert_eq!(fields.body_parentid[b], 0);
                assert_eq!(&fields.body_quat[4 * b..4 * b + 4], &[1.0, 0.0, 0.0, 0.0]);
            }
            assert_eq!(&r.xquat[4 * b..4 * b + 4], &[0.0, 0.0, 0.0, 1.0]);
        }
    }
    assert!(
        out.flex_positions()
            .unwrap()
            .world(0)
            .unwrap()
            .flex_hessian_valid
            .iter()
            .all(|&v| !v)
    );
}
