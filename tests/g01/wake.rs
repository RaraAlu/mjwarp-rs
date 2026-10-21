//! G01 唤醒状态保持与轻量刷新。

use mjwarp_rs::model::{CamLightParameters, KinematicsParameters};
use mjwarp_rs::physics::{KinematicsPlan, fwd_kinematics};
use mjwarp_rs::runtime::TransferSession;

use crate::support::{fixture, floats, input, sleep_state, state, wake_oracle, with_sleep_flags};

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn g01_wake_preserves_input_activity_until_lightweight_refresh() {
    let v = fixture();
    let native = input();
    let session = TransferSession::new(0).unwrap();
    let mut seeds = Vec::new();
    let k = native
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .attached()
        .rigid()
        .kinematics();
    let hinge_body = k.fields().jnt_bodyid[0] as usize;
    let hinge = native.fields().body_treeid[hinge_body] as usize;
    let slide_joint = k
        .fields()
        .jnt_type
        .iter()
        .enumerate()
        .find(|&(j, &t)| {
            t == 2
                && native.fields().body_treeid[k.fields().jnt_bodyid[j] as usize] as usize != hinge
        })
        .unwrap()
        .0;
    let slide = native.fields().body_treeid[k.fields().jnt_bodyid[slide_joint] as usize] as usize;
    assert_ne!(hinge, slide);
    for mode in 0..4 {
        let mut s = sleep_state(&native);
        if mode == 0 || mode == 2 {
            s.tree_asleep[hinge] = if mode == 2 { -15 } else { -3 };
            s.tree_awake[hinge] = 1;
            s.ntree_awake = 1;
        }
        if mode == 1 {
            s.tree_asleep[hinge] = slide as i32;
            s.tree_asleep[slide] = hinge as i32;
        }
        if mode == 2 {
            let other = (0..native.ntree())
                .find(|&t| t != hinge && t != slide)
                .unwrap();
            s.tree_asleep[slide] = other as i32;
            s.tree_asleep[other] = slide as i32;
        }
        if mode == 3 {
            s.tree_asleep.fill(-11);
        }
        s.nbody_awake = 3;
        s.nv_awake = 2;
        seeds.push(s);
    }
    for (enabled, disabled) in [(true, false), (false, false), (true, true)] {
        let model = with_sleep_flags(&native, enabled, disabled);
        let p = KinematicsPlan::for_g01(
            &session,
            model.clone(),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut d = p.create_data(4).unwrap();
        let mut state = state(&v, 4, 0);
        state.sleep = Some(seeds.clone());
        p.import_g01_state(&mut d, &state).unwrap();
        fwd_kinematics(&p, &mut d).unwrap();
        let out = d.readback().unwrap();
        for (world, seed) in seeds.iter().enumerate() {
            let expected = wake_oracle(
                &model,
                seed.clone(),
                &floats(&v["cases"][world], "ten_length"),
            );
            let actual = out.sleep_trees().unwrap().world(world).unwrap();
            assert_eq!(actual.tree_asleep, expected.tree_asleep);
            assert_eq!(actual.tree_awake, expected.tree_awake);
            assert_eq!(
                (actual.ntree_awake, actual.nbody_awake, actual.nv_awake),
                (
                    expected.ntree_awake,
                    expected.nbody_awake,
                    expected.nv_awake
                )
            );
        }
    }
}
