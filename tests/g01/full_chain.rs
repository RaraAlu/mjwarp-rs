//! G01 完整链原生参考比对。

use mjwarp_rs::physics::fwd_kinematics;
use mjwarp_rs::runtime::TransferSession;

use crate::support::{compare, fixture, plan, state};
use crate::two_reference::{Audit, evaluate, native_reference};

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn full_chain_matches_native_reference_without_freshness_ticket() {
    let mut audit = Audit::new("G01-complete");
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let p = plan(&session);
    drop(session);
    let (mut states, mut scalars, mut max) = (0, 0, 0.0_f64);
    for nw in [1, 2, 5, 513] {
        let mut d = p.create_data(nw).unwrap();
        for round in 0..4 {
            p.import_g01_state(&mut d, &state(&v, nw, round)).unwrap();
            assert!(d.readback().is_err());
            fwd_kinematics(&p, &mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..nw {
                let c = &v["cases"][(w + 3 * round) % 16];
                let r = out.rigid().world(w).unwrap();
                let com = out.com().world(w).unwrap();
                let a = out.attached().world(w).unwrap();
                let cam = out.camlight().world(w).unwrap();
                let flex = out.flex_positions().unwrap().world(w).unwrap();
                let edge = out.flex_edges().unwrap().world(w).unwrap();
                let face = out.flex_faces().unwrap().world(w).unwrap();
                let t = out.tendon().unwrap().world(w).unwrap();
                assert_eq!(flex.flex_hessian_valid, &[false; 4]);
                for (actual, key) in [
                    (r.xpos, "xpos"),
                    (r.xquat, "xquat"),
                    (r.xmat, "xmat"),
                    (r.xipos, "xipos"),
                    (r.ximat, "ximat"),
                    (r.xanchor, "xanchor"),
                    (r.xaxis, "xaxis"),
                    (com.subtree_com, "subtree_com"),
                    (com.cinert, "cinert"),
                    (com.cdof, "cdof"),
                    (a.geom_xpos, "geom_xpos"),
                    (a.geom_xmat, "geom_xmat"),
                    (a.site_xpos, "site_xpos"),
                    (a.site_xmat, "site_xmat"),
                    (cam.cam_xpos, "cam_xpos"),
                    (cam.cam_xmat, "cam_xmat"),
                    (cam.light_xpos, "light_xpos"),
                    (cam.light_xdir, "light_xdir"),
                    (flex.flexnode_xpos, "flexnode_xpos"),
                    (flex.flexvert_xpos, "flexvert_xpos"),
                    (edge.flexedge_length, "flexedge_length"),
                    (edge.flexedge_velocity, "flexedge_velocity"),
                    (edge.flexedge_jacobian, "flexedge_J"),
                    (t.ten_length, "ten_length"),
                    (t.ten_jacobian, "ten_J"),
                    (face.face_xpos, "face_xpos"),
                ] {
                    compare(actual, c, key, &mut max, &mut scalars, &mut audit);
                }
                let quats = c["face_quat"].as_array().unwrap();
                assert_eq!(face.face_quat.len(), quats.len());
                assert_eq!(quats.len() % 4, 0);
                let mut aligned = Vec::with_capacity(quats.len());
                for (a, e) in face
                    .face_quat
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(quats.as_chunks::<4>().0)
                {
                    let dot = a
                        .iter()
                        .zip(e)
                        .map(|(&a, e)| f64::from(a) * e.as_f64().unwrap())
                        .sum::<f64>();
                    aligned.extend(
                        e.iter()
                            .map(|e| e.as_f64().unwrap() * if dot < 0.0 { -1.0 } else { 1.0 }),
                    );
                }
                let decision = evaluate(
                    face.face_quat,
                    Some(native_reference(
                        "MuJoCo3.12.0:g01:face_quat:sign_aligned",
                        aligned,
                        2e-5,
                        2e-5,
                    )),
                    || Ok(None),
                );
                max = max.max(audit.record(
                    serde_json::json!({"world": w, "field": "face_quat"}),
                    decision,
                ));
                scalars += face.face_quat.len();
                let points = c["wrap_obj"].as_array().unwrap().len();
                compare(
                    &t.wrap_xpos[..3 * points],
                    c,
                    "wrap_xpos",
                    &mut max,
                    &mut scalars,
                    &mut audit,
                );
                assert!(t.wrap_xpos[3 * points..].iter().all(|&x| x == 0.0));
                for (a, key) in [
                    (t.ten_wrapadr, "ten_wrapadr"),
                    (t.ten_wrapnum, "ten_wrapnum"),
                    (&t.wrap_obj[..points], "wrap_obj"),
                ] {
                    let e = c[key]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_i64().unwrap() as i32)
                        .collect::<Vec<_>>();
                    assert_eq!(a, e, "{key}");
                }
                let sleep = out.sleep_trees().unwrap().world(w).unwrap();
                assert!(sleep.tree_asleep.iter().all(|&x| x == -11));
                assert_eq!((sleep.nbody_awake, sleep.nv_awake), (0, 0));
                states += 1;
            }
        }
    }
    assert_eq!(states, 2084);
    println!("G01-complete states={states} scalars={scalars} max_abs_error={max}");
    audit.finish();
}
