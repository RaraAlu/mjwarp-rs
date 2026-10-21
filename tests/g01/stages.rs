//! G01 低层阶段只读已准备字段。

use mjwarp_rs::physics::{camlight, com_pos, flex, kinematics, tendon};
use mjwarp_rs::runtime::TransferSession;

use crate::support::{cache, com_cache, compare, fixture, floats, input, plan, sleep_state, state};
use crate::two_reference::Audit;

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn raw_g01_stages_read_prepared_fields_without_implicit_dependencies() {
    let mut audit = Audit::new("G01-raw");
    let v = fixture();
    let model = input();
    let seed = sleep_state(&model);
    let session = TransferSession::new(0).unwrap();
    let p = plan(&session);
    let mut d = p.create_data(16).unwrap();
    let mut prepared = state(&v, 16, 0);
    // qpos故意对应另一状态。
    // 低层入口必须只读现有字段。
    prepared.qpos = (0..16)
        .flat_map(|w| floats(&v["cases"][(w + 1) % 16], "qpos"))
        .collect();
    prepared.rigid_cache = Some(
        (0..16)
            .flat_map(|w| {
                cache(
                    &v["cases"][w],
                    &[
                        "xpos", "xquat", "xmat", "xipos", "ximat", "xanchor", "xaxis",
                    ],
                )
            })
            .collect(),
    );
    prepared.com_cache = Some((0..16).flat_map(|w| com_cache(&v["cases"][w])).collect());
    prepared.attached_cache = Some(
        (0..16)
            .flat_map(|w| {
                cache(
                    &v["cases"][w],
                    &["geom_xpos", "geom_xmat", "site_xpos", "site_xmat"],
                )
            })
            .collect(),
    );
    prepared.sleep = Some(vec![seed; 16]);
    p.import_g01_state(&mut d, &prepared).unwrap();
    assert!(d.readback().is_err());
    let mut max = 0.0_f64;
    let mut scalars = 0;
    // 逆序调用拒绝隐式前置更新。
    camlight(&p, &mut d).unwrap();
    flex(&p, &mut d).unwrap();
    // 固定肌腱读取qpos，因此改回状态。
    d.write_qpos(&state(&v, 16, 0).qpos).unwrap();
    tendon(&p, &mut d).unwrap();
    assert!(d.readback().is_err());
    let out = d.readback_g01().unwrap();
    for w in 0..16 {
        let c = &v["cases"][w];
        let r = out.rigid().world(w).unwrap();
        let com = out.com().world(w).unwrap();
        let cam = out.camlight().world(w).unwrap();
        let f = out.flex_positions().unwrap().world(w).unwrap();
        let e = out.flex_edges().unwrap().world(w).unwrap();
        let t = out.tendon().unwrap().world(w).unwrap();
        for (a, key) in [
            (r.xpos, "xpos"),
            (r.xquat, "xquat"),
            (r.xmat, "xmat"),
            (com.subtree_com, "subtree_com"),
            (com.cinert, "cinert"),
            (com.cdof, "cdof"),
            (cam.cam_xpos, "cam_xpos"),
            (cam.cam_xmat, "cam_xmat"),
            (cam.light_xpos, "light_xpos"),
            (cam.light_xdir, "light_xdir"),
            (f.flexvert_xpos, "flexvert_xpos"),
            (f.flexnode_xpos, "flexnode_xpos"),
            (e.flexedge_length, "flexedge_length"),
            (e.flexedge_velocity, "flexedge_velocity"),
            (e.flexedge_jacobian, "flexedge_J"),
            (t.ten_length, "ten_length"),
            (t.ten_jacobian, "ten_J"),
        ] {
            compare(a, c, key, &mut max, &mut scalars, &mut audit);
        }
        assert_eq!(f.flex_hessian_valid, &[false; 4]);
        let sleep = out.sleep_trees().unwrap().world(w).unwrap();
        assert_eq!(
            sleep.tree_asleep,
            prepared.sleep.as_ref().unwrap()[w].tree_asleep
        );
        assert_eq!(
            sleep.tree_awake,
            prepared.sleep.as_ref().unwrap()[w].tree_awake
        );
    }
    com_pos(&p, &mut d).unwrap();
    let out = d.readback_g01().unwrap();
    for w in 0..16 {
        let c = &v["cases"][w];
        let com = out.com().world(w).unwrap();
        for (a, key) in [
            (com.subtree_com, "subtree_com"),
            (com.cinert, "cinert"),
            (com.cdof, "cdof"),
        ] {
            compare(a, c, key, &mut max, &mut scalars, &mut audit);
        }
    }
    kinematics(&p, &mut d).unwrap();
    let out = d.readback_g01().unwrap();
    for w in 0..16 {
        let r = out.rigid().world(w).unwrap();
        compare(
            r.xpos,
            &v["cases"][w],
            "xpos",
            &mut max,
            &mut scalars,
            &mut audit,
        );
    }
    println!("G01 raw scalars={scalars} max_abs_error={max:e}");
    audit.finish();
}
