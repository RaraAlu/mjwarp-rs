//! 退化目标、零质量体与空集合边界。

use super::compare;
use super::flattened;
use super::oracle::shared;
use mjwarp_rs::{
    diagnostics::TransferError,
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput,
        CamLightParameter as F, CamLightParameters, InertialFields, InertialModelInput,
        KinematicFields, KinematicModelInput, KinematicsParameters, MocapModelInput,
        ParameterBatch,
    },
    physics::KinematicsPlan,
    runtime::TransferSession,
};

fn zero_mass_base() -> MocapModelInput {
    let k = KinematicModelInput::new(
        0,
        KinematicFields {
            body_parentid: vec![0, 0],
            body_jntadr: vec![-1, -1],
            body_jntnum: vec![0, 0],
            body_pos: vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            ..Default::default()
        },
    )
    .unwrap();
    let i = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 6],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            body_mass: vec![0.0; 2],
            body_inertia: vec![0.0; 6],
            ..Default::default()
        },
    )
    .unwrap();
    MocapModelInput::new(
        AttachedModelInput::new(i, AttachedFields::default()).unwrap(),
        vec![-1, 0],
    )
    .unwrap()
}
fn edge_fields() -> CamLightFields {
    CamLightFields {
        cam_mode: vec![3, 4, 3, 4, 2, 4],
        cam_bodyid: vec![0, 0, 0, 0, 1, 1],
        cam_targetbodyid: vec![0, 0, 0, 0, -1, -2],
        cam_pos: vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1e-30, 0.0, 0.0, 1e-10, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0,
        ],
        cam_quat: [1.0, 0.0, 0.0, 0.0].repeat(6),
        cam_pos0: vec![0.0; 18],
        cam_poscom0: [0.5, -0.25, 0.75].repeat(6),
        cam_mat0: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0].repeat(6),
        light_mode: vec![0, 0, 0, 3, 4, 2, 4, 3],
        light_bodyid: vec![0, 0, 0, 1, 1, 1, 0, 0],
        light_targetbodyid: vec![-1, -1, -1, -2, -1, -1, 0, 0],
        light_pos: vec![0.0; 24],
        light_dir: vec![
            0.0, 0.0, 0.0, 1e-30, 0.0, 0.0, 1e-10, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        ],
        light_pos0: vec![0.0; 24],
        light_poscom0: [0.25, 0.5, -0.75].repeat(8),
        light_dir0: [0.0, 2.0, 0.0].repeat(8),
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn degenerate_targets_zero_mass_and_missing_light_normalization_match_warp() {
    let session = TransferSession::new(0).unwrap();
    let model = CamLightModelInput::new(zero_mass_base(), edge_fields()).unwrap();
    let plan = KinematicsPlan::with_camlight(
        &session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    for world in 0..513 {
        let c = out.camlight().world(world).unwrap();
        assert_eq!(
            &c.cam_xmat[..9],
            &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        );
        assert_eq!(&c.cam_xmat[9..27], &[0.0; 18]);
        assert_eq!(
            &c.cam_xmat[27..36],
            &[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(&c.cam_xpos[12..15], &[0.5, -0.25, 0.75]); // zero mass COM remains zero
        assert_eq!(&c.cam_xpos[15..18], &[1.0, 2.0, 3.0]);
        assert_eq!(
            &c.light_xdir[..9],
            &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
        );
        assert_eq!(
            &c.light_xdir[9..18],
            &[0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(&c.light_xdir[18..], &[0.0; 6]);
        assert_eq!(&c.light_xpos[15..18], &[0.25, 0.5, -0.75]);
    }
    data.write_world_mocap(512, &[-1.0, 0.5, 2.0], &[0.5, 0.0, 0.0, 0.5])
        .unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let c = out.camlight().world(512).unwrap();
    compare(&c.cam_xpos[15..18], &[-1.0, 0.5, 2.0]);
    compare(&c.light_xdir[9..15], &[0.0, 0.0, 2.0, 0.0, 0.0, 3.0]);
    assert_eq!(&c.cam_xpos[12..15], &[0.5, -0.25, 0.75]);
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn empty_camera_or_light_sets_preserve_periods_and_legacy_readback() {
    let session = TransferSession::new(0).unwrap();
    for (nc, nl) in [(0, 0), (6, 0), (0, 8)] {
        let mut f = edge_fields();
        f.cam_mode.truncate(nc);
        f.cam_bodyid.truncate(nc);
        f.cam_targetbodyid.truncate(nc);
        f.cam_pos.truncate(3 * nc);
        f.cam_quat.truncate(4 * nc);
        f.cam_pos0.truncate(3 * nc);
        f.cam_poscom0.truncate(3 * nc);
        f.cam_mat0.truncate(9 * nc);
        f.light_mode.truncate(nl);
        f.light_bodyid.truncate(nl);
        f.light_targetbodyid.truncate(nl);
        f.light_pos.truncate(3 * nl);
        f.light_dir.truncate(3 * nl);
        f.light_pos0.truncate(3 * nl);
        f.light_poscom0.truncate(3 * nl);
        f.light_dir0.truncate(3 * nl);
        let mut p = CamLightParameters::default();
        for field in F::ALL {
            if shared(&f, field).is_empty() {
                p.set(
                    field,
                    ParameterBatch::new(i32::MAX as usize, 0, vec![]).unwrap(),
                );
            }
        }
        let plan = KinematicsPlan::with_camlight(
            &session,
            CamLightModelInput::new(zero_mass_base(), f).unwrap(),
            KinematicsParameters::default(),
            p,
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        plan.update_rigid(&mut data).unwrap();
        plan.update_attached(&mut data).unwrap();
        plan.update_com(&mut data).unwrap();
        if nc == 0 && nl == 0 {
            assert!(
                data.readback()
                    .unwrap()
                    .camlight()
                    .world(512)
                    .unwrap()
                    .cam_xmat
                    .is_empty()
            );
        } else {
            assert!(matches!(
                data.readback(),
                Err(TransferError::StageNotReady { stage: "camlight" })
            ));
        }
        plan.update_camlight(&mut data).unwrap();
        let out = data.readback().unwrap();
        assert_eq!((out.camlight().ncam(), out.camlight().nlight()), (nc, nl));
        assert!(out.camlight().world(513).is_err());
        assert_eq!(flattened(&out, 512).len(), 12 * nc + 6 * nl);
    }
}
