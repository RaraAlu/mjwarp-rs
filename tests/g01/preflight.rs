//! G01 导入预检与拒绝时保留旧结果。

use mjwarp_rs::model::{CamLightParameters, KinematicsParameters};
use mjwarp_rs::physics::{
    KinematicsPlan, camlight, com_pos, flex, fwd_kinematics, kinematics, tendon,
};
use mjwarp_rs::runtime::TransferSession;

use crate::support::{cache, com_cache, fixture, input, sleep_state, state};

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn g01_import_preflights_all_fields_and_preserves_previous_results_on_rejection() {
    let v = fixture();
    let model = input();
    let seed = sleep_state(&model);
    let session = TransferSession::new(0).unwrap();
    let p = KinematicsPlan::for_g01(
        &session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    assert!(p.create_data(0).is_err());
    assert!(p.create_data(u32::MAX as usize).is_err());
    let mut d = p.create_data(2).unwrap();
    let mut state = state(&v, 2, 0);
    state.sleep = Some(vec![seed.clone(), seed]);
    state.world_pose = Some(vec![[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]; 2]);
    p.import_g01_state(&mut d, &state).unwrap();
    fwd_kinematics(&p, &mut d).unwrap();
    let previous = d.readback().unwrap();
    let a = previous.attached().world(0).unwrap();
    let row = a
        .geom_xpos
        .iter()
        .chain(a.geom_xmat)
        .chain(a.site_xpos)
        .chain(a.site_xmat)
        .copied()
        .collect::<Vec<_>>();
    state.attached_cache = Some(row.repeat(2));
    state.rigid_cache = Some(
        cache(
            &v["cases"][0],
            &[
                "xpos", "xquat", "xmat", "xipos", "ximat", "xanchor", "xaxis",
            ],
        )
        .repeat(2),
    );
    state.com_cache = Some(com_cache(&v["cases"][0]).repeat(2));
    for mutation in 0..19 {
        let mut bad = state.clone();
        match mutation {
            0 => {
                bad.qpos.pop();
            }
            1 => bad.qpos[0] = f32::NAN,
            2 => {
                bad.qvel.pop();
            }
            3 => bad.qvel[0] = f32::INFINITY,
            4 => {
                bad.mocap_pos.pop();
            }
            5 => bad.mocap_pos[0] = f32::NAN,
            6 => {
                bad.mocap_quat.pop();
            }
            7 => bad.mocap_quat[0] = f32::NAN,
            8 => {
                bad.world_pose.as_mut().unwrap().pop();
            }
            9 => bad.world_pose.as_mut().unwrap()[0][0] = f32::NAN,
            10 => {
                bad.attached_cache.as_mut().unwrap().pop();
            }
            11 => bad.attached_cache.as_mut().unwrap()[0] = f32::NAN,
            12 => {
                bad.flex_hessian_valid.as_mut().unwrap().pop();
            }
            13 => {
                bad.sleep.as_mut().unwrap()[1].tree_awake[0] = 2;
            }
            14 => {
                bad.sleep.as_mut().unwrap()[1].ntree_awake = -1;
            }
            15 => {
                bad.rigid_cache.as_mut().unwrap().pop();
            }
            16 => bad.rigid_cache.as_mut().unwrap()[0] = f32::NAN,
            17 => {
                bad.com_cache.as_mut().unwrap().pop();
            }
            _ => bad.com_cache.as_mut().unwrap()[0] = f32::INFINITY,
        }
        assert!(
            p.import_g01_state(&mut d, &bad).is_err(),
            "mutation {mutation}"
        );
        let out = d.readback().unwrap();
        for w in 0..2 {
            assert_eq!(
                out.rigid().world(w).unwrap().xpos,
                previous.rigid().world(w).unwrap().xpos
            );
            assert_eq!(
                out.attached().world(w).unwrap().geom_xpos,
                previous.attached().world(w).unwrap().geom_xpos
            );
            assert_eq!(
                out.tendon().unwrap().world(w).unwrap().ten_length,
                previous.tendon().unwrap().world(w).unwrap().ten_length
            );
            assert_eq!(
                out.sleep_trees().unwrap().world(w).unwrap().tree_asleep,
                previous
                    .sleep_trees()
                    .unwrap()
                    .world(w)
                    .unwrap()
                    .tree_asleep
            );
        }
    }
    let strict = KinematicsPlan::with_tendon_wake(
        &session,
        input(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut other = strict.create_data(1).unwrap();
    assert!(fwd_kinematics(&strict, &mut other).is_err());
    for stage in [kinematics, com_pos, camlight, flex, tendon] {
        assert!(stage(&strict, &mut other).is_err());
        assert!(matches!(
            stage(&strict, &mut d),
            Err(mjwarp_rs::diagnostics::TransferError::ModelMismatch)
        ));
    }
    assert!(other.readback_g01().is_err());
}
