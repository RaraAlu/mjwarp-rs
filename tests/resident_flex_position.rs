#![cfg(feature = "cuda-probe")]
//! G01柔体位置子集验收。
//! 产品测试只读取静态参考。
use mjwarp_rs::model::{FlexPositionFields, FlexPositionModelInput};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/flex_position_reference.rs"]
mod reference;
use reference::{fields, fixture, floats, tendons};
#[test]
fn reference_hashes_and_checked_flex_fields() {
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/flex-position/manifest.json")).unwrap();
    for f in manifest["files"].as_array().unwrap() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/flex-position")
            .join(f["path"].as_str().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(path).unwrap())),
            f["sha256"].as_str().unwrap()
        );
    }
    let v = fixture();
    let model = FlexPositionModelInput::new(tendons(&v), fields(&v)).unwrap();
    assert_eq!(
        (model.nflex(), model.nflexnode(), model.nflexvert()),
        (4, 20, 80)
    );
    assert_eq!(model.fields().flex_interp, &[0, 0, 1, 1]);
    assert_eq!(model.fields().flex_centered, &[false, true, false, true]);
    assert_eq!(&model.fields().flex_cellnum[6..9], &[2, 1, 1]);
    assert_eq!(v["cases"].as_array().unwrap().len(), 12);
}

#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use mjwarp_rs::{
        diagnostics::TransferError,
        model::{CamLightParameters, KinematicsParameters},
        physics::{KinematicsData, KinematicsPlan, KinematicsSnapshot},
        runtime::TransferSession,
    };
    fn plan(s: &TransferSession, v: &Value) -> KinematicsPlan {
        KinematicsPlan::with_flex_positions(
            s,
            FlexPositionModelInput::new(tendons(v), fields(v)).unwrap(),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }
    fn write(d: &mut KinematicsData, w: usize, c: &Value) {
        d.write_world_qpos(w, &floats(c, "qpos")).unwrap();
        d.write_world_mocap(w, &floats(c, "mocap_pos"), &floats(c, "mocap_quat"))
            .unwrap();
    }
    fn compare(a: &[f32], e: &Value) -> f64 {
        let e = e.as_array().unwrap();
        assert_eq!(a.len(), e.len());
        let mut max: f64 = 0.0;
        for (i, (&a, e)) in a.iter().zip(e).enumerate() {
            let e = e.as_f64().unwrap();
            let err = (f64::from(a) - e).abs();
            max = max.max(err);
            assert!(
                a.is_finite() && err <= 2e-5 + 2e-5 * e.abs(),
                "G01 flex index={i} actual={a} expected={e} error={err}"
            );
        }
        max
    }
    fn native(out: &KinematicsSnapshot, w: usize, c: &Value) -> f64 {
        let f = out.flex_positions().unwrap().world(w).unwrap();
        // Check the frozen flag contract separately from native positions.
        assert_eq!(out.flex_positions().unwrap().nflex(), 4);
        assert_eq!(f.flex_hessian_valid, &[false; 4]);
        let max = compare(f.flexnode_xpos, &c["flexnode_xpos"])
            .max(compare(f.flexvert_xpos, &c["flexvert_xpos"]));
        compare(out.rigid().world(w).unwrap().xpos, &c["xpos"]);
        compare(out.rigid().world(w).unwrap().xmat, &c["xmat"]);
        let t = out.tendon().unwrap().world(w).unwrap();
        compare(t.ten_length, &c["ten_length"]);
        compare(t.ten_jacobian, &c["ten_J"]);
        max
    }
    fn bits(out: &KinematicsSnapshot, w: usize) -> Vec<u32> {
        let f = out.flex_positions().unwrap().world(w).unwrap();
        f.flexnode_xpos
            .iter()
            .chain(f.flexvert_xpos)
            .map(|f| f.to_bits())
            .collect()
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn matches_native_flex_positions_across_uneven_worlds_and_repeated_updates() {
        let s = TransferSession::new(0).unwrap();
        let v = fixture();
        let p = plan(&s, &v);
        let mut count = 0;
        let mut max: f64 = 0.0;
        for nw in [1, 2, 5, 513] {
            let mut d = p.create_data(nw).unwrap();
            for round in 0..4 {
                for w in 0..nw {
                    write(&mut d, w, &v["cases"][(w + round * 3) % 12]);
                }
                p.update(&mut d).unwrap();
                let out = d.readback().unwrap();
                for w in 0..nw {
                    max = max.max(native(&out, w, &v["cases"][(w + round * 3) % 12]));
                    count += 1;
                }
            }
        }
        assert_eq!(count, 2084);
        println!(
            "G01-flex states={count} scalars={} max_abs_error={max}",
            count * 300
        );
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn preserves_stage_dependencies_identity_and_rejected_writes() {
        let s = TransferSession::new(0).unwrap();
        let v = fixture();
        let p = plan(&s, &v);
        let other = plan(&s, &v);
        let mut d = p.create_data(2).unwrap();
        assert!(matches!(
            p.update_flex_positions(&mut d),
            Err(TransferError::StageNotReady { stage: "rigid" })
        ));
        p.update_rigid(&mut d).unwrap();
        p.update_attached(&mut d).unwrap();
        p.update_com(&mut d).unwrap();
        p.update_tendons(&mut d).unwrap();
        assert!(matches!(
            d.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_positions"
            })
        ));
        p.update_flex_positions(&mut d).unwrap();
        let before = d.readback().unwrap();
        assert!(matches!(
            other.update_flex_positions(&mut d),
            Err(TransferError::ModelMismatch)
        ));
        assert!(d.write_world_qpos(0, &[f32::NAN; 66]).is_err());
        assert!(
            d.write_world_mocap(0, &[f32::NAN; 3], &[1.0, 0.0, 0.0, 0.0])
                .is_err()
        );
        assert!(
            d.write_world_qpos(2, &floats(&v["cases"][0], "qpos"))
                .is_err()
        );
        assert_eq!(bits(&before, 0), bits(&d.readback().unwrap(), 0));
        write(&mut d, 1, &v["cases"][7]);
        p.update_rigid(&mut d).unwrap();
        p.update_flex_positions(&mut d).unwrap();
        p.update_com(&mut d).unwrap();
        p.update_attached(&mut d).unwrap();
        p.update_tendons(&mut d).unwrap();
        let out = d.readback().unwrap();
        native(&out, 1, &v["cases"][7]);
        assert_eq!(bits(&out, 0), bits(&before, 0));
        p.update_rigid(&mut d).unwrap();
        p.update_com(&mut d).unwrap();
        p.update_attached(&mut d).unwrap();
        p.update_tendons(&mut d).unwrap();
        assert!(matches!(
            d.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_positions"
            })
        ));
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn keeps_optional_empty_and_direct_only_routes_compatible() {
        let s = TransferSession::new(0).unwrap();
        let v = fixture();
        let old = KinematicsPlan::with_tendons(
            &s,
            tendons(&v),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut d = old.create_data(2).unwrap();
        old.update(&mut d).unwrap();
        assert!(d.readback().unwrap().flex_positions().is_none());
        let mut f = fields(&v);
        f.flex_interp.truncate(2);
        f.flex_cellnum.truncate(6);
        f.flex_nodeadr.truncate(2);
        f.flex_nodenum.truncate(2);
        f.flex_vertadr.truncate(2);
        f.flex_vertnum.truncate(2);
        f.flex_centered.truncate(2);
        f.flex_nodebodyid.clear();
        f.flex_node.clear();
        f.flex_vertbodyid.truncate(8);
        f.flex_vert.truncate(24);
        f.flex_vert0.truncate(24);
        for fields in [f, FlexPositionFields::default()] {
            let direct = !fields.flex_interp.is_empty();
            let p = KinematicsPlan::with_flex_positions(
                &s,
                FlexPositionModelInput::new(tendons(&v), fields).unwrap(),
                KinematicsParameters::default(),
                CamLightParameters::default(),
            )
            .unwrap();
            let mut d = p.create_data(5).unwrap();
            for w in 0..5 {
                write(&mut d, w, &v["cases"][w]);
            }
            p.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..5 {
                let f = out.flex_positions().unwrap().world(w).unwrap();
                assert!(f.flexnode_xpos.is_empty());
                if direct {
                    compare(
                        f.flexvert_xpos,
                        &Value::Array(
                            v["cases"][w]["flexvert_xpos"].as_array().unwrap()[..24].to_vec(),
                        ),
                    );
                } else {
                    assert!(f.flexvert_xpos.is_empty());
                }
            }
        }
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn keeps_independent_data_snapshots_and_output_owners_alive() {
        let s = TransferSession::new(0).unwrap();
        let v = fixture();
        let p = plan(&s, &v);
        let mut a = p.create_data(2).unwrap();
        let mut b = p.create_data(5).unwrap();
        p.update(&mut a).unwrap();
        let snap = a.readback().unwrap();
        let initial = bits(&snap, 1);
        for w in 0..5 {
            write(&mut b, w, &v["cases"][w + 2]);
        }
        p.update(&mut b).unwrap();
        assert_eq!(bits(&a.readback().unwrap(), 1), initial);
        write(&mut a, 1, &v["cases"][10]);
        p.update(&mut a).unwrap();
        native(&a.readback().unwrap(), 1, &v["cases"][10]);
        assert_eq!(bits(&snap, 1), initial);
        drop(p);
        drop(s);
        native(&a.readback().unwrap(), 1, &v["cases"][10]);
        native(&b.readback().unwrap(), 4, &v["cases"][6]);
    }
}
