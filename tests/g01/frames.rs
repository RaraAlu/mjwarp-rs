//! G01 世界位姿与静态几何缓存保持。

use mjwarp_rs::model::{CamLightParameters, KinematicsParameters};
use mjwarp_rs::physics::{KinematicsPlan, fwd_kinematics};
use mjwarp_rs::runtime::TransferSession;

use crate::support::{fixture, floats, input, plan, state};

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn g01_preserves_imported_world_frames_and_static_geometry_cache() {
    let v = fixture();
    let model = input();
    let attached = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .attached();
    let mask = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .static_geom()
        .to_vec();
    let ng = attached.ngeom();
    let ns = attached.nsite();
    let stride = 12 * (ng + ns);
    let free = attached
        .rigid()
        .kinematics()
        .fields()
        .jnt_type
        .iter()
        .position(|&t| t == 0)
        .unwrap();
    let body = attached.rigid().kinematics().fields().jnt_bodyid[free] as usize;
    let session = TransferSession::new(0).unwrap();
    let p = KinematicsPlan::for_g01(
        &session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut d = p.create_data(3).unwrap();
    let mut state = state(&v, 3, 0);
    state.world_pose = Some(vec![[1.0, -2.0, 0.5, 1.0, 0.0, 0.0, 0.0]; 3]);
    state.attached_cache = Some(vec![777.0; stride * 3]);
    p.import_g01_state(&mut d, &state).unwrap();
    for _ in 0..2 {
        fwd_kinematics(&p, &mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..3 {
            let r = out.rigid().world(w).unwrap();
            let a = out.attached().world(w).unwrap();
            assert_eq!(&r.xpos[..3], &[1.0, -2.0, 0.5]);
            assert_eq!(&r.xquat[..4], &[1.0, 0.0, 0.0, 0.0]);
            let c = &v["cases"][w];
            let expected = floats(c, "xpos");
            for (a, &e) in r.xpos[3 * body..3 * body + 3]
                .iter()
                .zip(&expected[3 * body..3 * body + 3])
            {
                assert!((a - e).abs() < 2e-5);
            }
            for (g, &is_static) in mask.iter().enumerate() {
                if is_static {
                    assert_eq!(&a.geom_xpos[3 * g..3 * g + 3], &[777.0; 3]);
                    assert_eq!(&a.geom_xmat[9 * g..9 * g + 9], &[777.0; 9]);
                } else {
                    assert!(a.geom_xpos[3 * g..3 * g + 3].iter().all(|&x| x != 777.0));
                }
            }
            assert!(a.site_xpos.iter().all(|&x| x != 777.0));
        }
    }
    let second = plan(&session);
    assert!(matches!(
        fwd_kinematics(&second, &mut d),
        Err(mjwarp_rs::diagnostics::TransferError::ModelMismatch)
    ));
}
