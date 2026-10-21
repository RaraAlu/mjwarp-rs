//! 六项常驻子集共享更新测试。

use mjwarp_rs::model::{
    CamLightFields, CamLightModelInput, FixedTendonFields, FixedTendonModelInput,
};

use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn all_six_resident_subsets_share_one_update_without_tendon_index_aliasing() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let m = model(&v);
    let cam = CamLightModelInput::new(
        m.fixed().camlight().mocap().clone(),
        CamLightFields {
            cam_mode: vec![0],
            cam_bodyid: vec![0],
            cam_targetbodyid: vec![-1],
            cam_pos: vec![1.0, 2.0, 3.0],
            cam_quat: vec![1.0, 0.0, 0.0, 0.0],
            cam_poscom0: vec![0.0; 3],
            cam_pos0: vec![0.0; 3],
            cam_mat0: vec![0.0; 9],
            light_mode: vec![0],
            light_bodyid: vec![0],
            light_targetbodyid: vec![-1],
            light_pos: vec![3.0, 2.0, 1.0],
            light_dir: vec![0.0, 0.0, 1.0],
            light_poscom0: vec![0.0; 3],
            light_pos0: vec![0.0; 3],
            light_dir0: vec![0.0; 3],
        },
    )
    .unwrap();
    let fixed = FixedTendonModelInput::new(
        cam,
        FixedTendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![1],
            wrap_type: vec![1],
            wrap_objid: vec![2],
            wrap_prm: vec![2.0],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![1],
            ten_j_colind: vec![9],
        },
    )
    .unwrap();
    let plan = KinematicsPlan::with_spatial_tendons(
        &s,
        SpatialTendonModelInput::new(fixed, m.fields().clone()).unwrap(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    for w in [0, 256, 512] {
        write_case(&mut data, w, &v["cases"][w % 8]);
    }
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    for w in [0, 256, 512] {
        let case = &v["cases"][w % 8];
        compare_native(&out, w, case);
        let fixed = out.fixed_tendon().world(w).unwrap();
        assert_eq!(fixed.ten_length, &[2.0 * floats(case, "qpos")[11]]);
        assert_eq!(fixed.ten_jacobian, &[2.0]);
        let cam = out.camlight().world(w).unwrap();
        assert_eq!(cam.cam_xpos, &[1.0, 2.0, 3.0]);
        assert_eq!(cam.light_xpos, &[3.0, 2.0, 1.0]);
        assert_eq!(cam.light_xdir, &[0.0, 0.0, 1.0]);
    }
    assert_eq!(out.fixed_tendon().rows().ntendon(), 1);
    assert_eq!(out.spatial_tendon().rows().ntendon(), 5);
}
