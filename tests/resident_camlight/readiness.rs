//! 相机/光源就绪态、身份与逐世界写入严格性。

use super::reference::floats;
use super::{compare, fixture, flattened, model, native_expected};
use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{CamLightParameters, KinematicsParameters},
    physics::KinematicsPlan,
    runtime::TransferSession,
};

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn camlight_readiness_identity_lifetime_and_world_writes_are_strict() {
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let make = || {
        KinematicsPlan::with_camlight(
            &session,
            model(&v),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    };
    let plan = make();
    let other = make();
    let mut data = plan.create_data(513).unwrap();
    let mut second = plan.create_data(2).unwrap();
    assert_eq!(
        plan.update_camlight(&mut data),
        Err(TransferError::StageNotReady { stage: "rigid" })
    );
    plan.update_rigid(&mut data).unwrap();
    assert_eq!(
        plan.update_camlight(&mut data),
        Err(TransferError::StageNotReady { stage: "com" })
    );
    plan.update_attached(&mut data).unwrap();
    plan.update_com(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "camlight" })
    ));
    plan.update_camlight(&mut data).unwrap();
    let saved = data.readback().unwrap();
    plan.update(&mut second).unwrap();
    assert_eq!(
        other.update_camlight(&mut data),
        Err(TransferError::ModelMismatch)
    );
    assert_eq!(
        flattened(&saved, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    assert!(matches!(
        data.write_world_qpos(513, &[]),
        Err(TransferError::Input(InputError::InvalidIndex { .. }))
    ));
    assert!(data.write_world_qpos(512, &[f32::NAN, 0.0]).is_err());
    assert_eq!(
        flattened(&saved, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    for (world, id) in [(0, 1), (256, 4), (512, 7)] {
        let c = &v["cases"][id];
        data.write_world_qpos(world, &floats(c, "qpos")).unwrap();
        data.write_world_mocap(world, &floats(c, "mocap_pos"), &floats(c, "mocap_quat"))
            .unwrap();
    }
    plan.update(&mut data).unwrap();
    let updated = data.readback().unwrap();
    for world in 0..513 {
        let id = match world {
            0 => 1,
            256 => 4,
            512 => 7,
            _ => 0,
        };
        compare(
            &flattened(&updated, world),
            &native_expected(&v["cases"][id]),
        );
    }
    compare(
        &flattened(&second.readback().unwrap(), 1),
        &native_expected(&v["cases"][0]),
    );
    plan.update_com(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "camlight" })
    ));
    plan.update_camlight(&mut data).unwrap();
    plan.update_attached(&mut data).unwrap();
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    drop(other);
    drop(plan);
    drop(session);
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    compare(&flattened(&saved, 512), &native_expected(&v["cases"][0]));
}
