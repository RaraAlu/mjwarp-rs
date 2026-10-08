#![cfg(feature = "cuda-probe")]
//! G01线性壳体面子集验收。
use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexEdgeFields,
    FlexFaceFields, FlexPositionFields, FlexPositionModelInput, MocapModelInput, TendonFields,
    TendonModelInput,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;
fn base() -> Value {
    serde_json::from_str(include_str!("../fixtures/flex-position/flex-tree.json")).unwrap()
}
fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/flex-face/face-state.json")).unwrap()
}
fn ids(v: &Value, k: &str) -> Vec<i32> {
    v[k].as_array()
        .unwrap()
        .iter()
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
fn tendons(v: &Value) -> TendonModelInput {
    let m = &v["model"];
    let a = AttachedModelInput::new(
        reference::model(v),
        AttachedFields {
            geom_bodyid: ids(m, "geom_bodyid"),
            geom_pos: floats(m, "geom_pos"),
            geom_quat: floats(m, "geom_quat"),
            site_bodyid: ids(m, "site_bodyid"),
            site_pos: floats(m, "site_pos"),
            site_quat: floats(m, "site_quat"),
        },
    )
    .unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(a, ids(m, "body_mocapid")).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    TendonModelInput::new(
        cam,
        TendonFields {
            tendon_adr: ids(m, "tendon_adr"),
            tendon_num: ids(m, "tendon_num"),
            wrap_type: ids(m, "wrap_type"),
            wrap_objid: ids(m, "wrap_objid"),
            wrap_prm: floats(m, "wrap_prm"),
            ten_j_rowadr: ids(m, "ten_J_rowadr"),
            ten_j_rownnz: ids(m, "ten_J_rownnz"),
            ten_j_colind: ids(m, "ten_J_colind"),
        },
    )
    .unwrap()
}
fn positions(b: &Value, e: &Value) -> FlexPositionFields {
    let m = &b["model"];
    FlexPositionFields {
        flex_interp: ids(&e["model"], "flex_interp"),
        flex_cellnum: ids(m, "flex_cellnum"),
        flex_nodeadr: ids(m, "flex_nodeadr"),
        flex_nodenum: ids(m, "flex_nodenum"),
        flex_vertadr: ids(m, "flex_vertadr"),
        flex_vertnum: ids(m, "flex_vertnum"),
        flex_centered: ids(m, "flex_centered").iter().map(|&b| b != 0).collect(),
        flex_nodebodyid: ids(m, "flex_nodebodyid"),
        flex_vertbodyid: ids(m, "flex_vertbodyid"),
        flex_node: floats(m, "flex_node"),
        flex_vert: floats(m, "flex_vert"),
        flex_vert0: floats(m, "flex_vert0"),
    }
}
fn faces(e: &Value) -> FlexFaceFields {
    FlexFaceFields {
        flex_face_map: ids(&e["model"], "flex_face_map"),
        flex_face: ids(&e["model"], "flex_face"),
    }
}
fn input(b: &Value, e: &Value) -> FlexPositionModelInput {
    FlexPositionModelInput::new(tendons(b), positions(b, e))
        .unwrap()
        .with_faces(faces(e))
        .unwrap()
}
#[test]
fn reference_hashes_pin_native_nodes_and_independent_polar_outputs() {
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/flex-face/manifest.json")).unwrap();
    for f in manifest["files"].as_array().unwrap() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/flex-face")
            .join(f["path"].as_str().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(p).unwrap())),
            f["sha256"].as_str().unwrap()
        );
    }
    let b = base();
    let e = fixture();
    let m = input(&b, &e);
    assert_eq!(m.faces().unwrap().nflexface(), 16);
    assert!(m.edges().is_none());
    for c in 0..12 {
        for field in ["qpos", "mocap_pos", "mocap_quat"] {
            assert_eq!(b["cases"][c][field], e["cases"][c][field]);
        }
    }
}
#[test]
fn rejects_face_shapes_missing_duplicate_foreign_and_reordered_nodes() {
    let b = base();
    let e = fixture();
    let f = faces(&e);
    let check = |f| {
        FlexPositionModelInput::new(tendons(&b), positions(&b, &e))
            .unwrap()
            .with_faces(f)
    };
    for i in 0..8 {
        let mut bad = f.clone();
        match i {
            0 => {
                bad.flex_face_map.pop();
            }
            1 => {
                bad.flex_face.pop();
            }
            2 => bad.flex_face_map[2] = 0,
            3 => bad.flex_face_map[3] = 0,
            4 => bad.flex_face[0] = -1,
            5 => bad.flex_face[0] = i32::MAX,
            6 => bad.flex_face.swap(0, 1),
            _ => bad.flex_face[4] = 0,
        }
        assert!(check(bad).is_err(), "case {i}");
    }
    assert!(check(FlexFaceFields::default()).is_err());
    for mode in [-2, 2, i32::MIN] {
        let mut p = positions(&b, &e);
        p.flex_interp[2] = mode;
        assert!(FlexPositionModelInput::new(tendons(&b), p).is_err());
    }
}
mod gpu {
    use super::*;
    use mjwarp_rs::{
        diagnostics::TransferError,
        model::{
            CamLightParameters, KinematicsParameters, ParameterBatch, TendonWakeFields,
            TendonWakeModelInput,
        },
        physics::{KinematicsData, KinematicsPlan, KinematicsSnapshot},
        runtime::TransferSession,
    };
    fn plan(s: &TransferSession, m: FlexPositionModelInput) -> KinematicsPlan {
        KinematicsPlan::with_flex_positions(
            s,
            m,
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
        let mut max = 0.0f64;
        for (i, (&a, e)) in a.iter().zip(e).enumerate() {
            let e = e.as_f64().unwrap();
            let err = (f64::from(a) - e).abs();
            max = max.max(err);
            assert!(
                a.is_finite() && err <= 2e-5 + 2e-5 * e.abs(),
                "G01 face index={i} actual={a} expected={e} error={err}"
            );
        }
        max
    }
    fn native(out: &KinematicsSnapshot, w: usize, c: &Value) -> (f64, f64) {
        let p = out.flex_positions().unwrap().world(w).unwrap();
        // Frozen clear semantics, separate from the native position reference.
        assert_eq!(p.flex_hessian_valid, &[false; 4]);
        compare(p.flexnode_xpos, &c["flexnode_xpos"]);
        compare(p.flexvert_xpos, &c["flexvert_xpos"]);
        let f = out.flex_faces().unwrap().world(w).unwrap();
        let pos = compare(f.face_xpos, &c["face_xpos"]);
        let refs: Vec<f64> = c["face_quat"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let mut quat = 0.0f64;
        for (a, e) in f
            .face_quat
            .as_chunks::<4>()
            .0
            .iter()
            .zip(refs.as_chunks::<4>().0)
        {
            let sign = if a.iter().zip(e).map(|(a, b)| f64::from(*a) * b).sum::<f64>() < 0.0 {
                -1.0
            } else {
                1.0
            };
            for (&a, &e) in a.iter().zip(e) {
                let err = (f64::from(a) - sign * e).abs();
                assert!(
                    err <= 2e-5 + 2e-5 * e.abs(),
                    "polar actual={a} expected={e} error={err}"
                );
                quat = quat.max(err);
            }
            assert!((a.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 2e-6);
        }
        (pos, quat)
    }
    fn bits(out: &KinematicsSnapshot, w: usize) -> Vec<u32> {
        let f = out.flex_faces().unwrap().world(w).unwrap();
        f.face_xpos
            .iter()
            .chain(f.face_quat)
            .map(|v| v.to_bits())
            .collect()
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn matches_native_nodes_and_eigen_polar_across_uneven_worlds() {
        let s = TransferSession::new(0).unwrap();
        let b = base();
        let e = fixture();
        let p = plan(&s, input(&b, &e));
        let (mut count, mut pos, mut quat) = (0, 0.0f64, 0.0f64);
        for nw in [1, 2, 5, 513] {
            let mut d = p.create_data(nw).unwrap();
            for round in 0..4 {
                for w in 0..nw {
                    write(&mut d, w, &e["cases"][(w + round * 3) % 12]);
                }
                p.update(&mut d).unwrap();
                let out = d.readback().unwrap();
                for w in 0..nw {
                    let (a, q) = native(&out, w, &e["cases"][(w + round * 3) % 12]);
                    pos = pos.max(a);
                    quat = quat.max(q);
                    count += 1;
                }
                p.update_flex_faces(&mut d).unwrap();
                let again = d.readback().unwrap();
                for w in 0..nw {
                    assert_eq!(bits(&out, w), bits(&again, w));
                }
            }
        }
        assert_eq!(count, 2084);
        println!(
            "G01-flex-face states={count} scalars={} max_position_error={pos} max_quaternion_error={quat}",
            count * 496
        );
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn preserves_face_readiness_identity_snapshots_and_lifetimes() {
        let s = TransferSession::new(0).unwrap();
        let b = base();
        let e = fixture();
        let p = plan(&s, input(&b, &e));
        let other = plan(&s, input(&b, &e));
        let mut d = p.create_data(513).unwrap();
        let mut independent = p.create_data(1).unwrap();
        assert_eq!(d.flex_face_fields(), p.flex_face_fields());
        assert!(matches!(
            p.update_flex_faces(&mut d),
            Err(TransferError::StageNotReady {
                stage: "flex_positions"
            })
        ));
        p.update_rigid(&mut d).unwrap();
        p.update_flex_positions(&mut d).unwrap();
        p.update_flex_faces(&mut d).unwrap();
        // Face computation needs neither COM nor qvel.
        assert!(d.write_qvel(&[0.0; 65]).is_err());
        p.update(&mut d).unwrap();
        let first = d.readback().unwrap();
        assert!(matches!(
            other.update_flex_faces(&mut d),
            Err(TransferError::ModelMismatch)
        ));
        assert!(
            d.write_world_qpos(513, &floats(&e["cases"][1], "qpos"))
                .is_err()
        );
        assert!(
            d.write_world_mocap(0, &[f32::NAN; 3], &[1.0, 0.0, 0.0, 0.0])
                .is_err()
        );
        assert_eq!(bits(&first, 0), bits(&d.readback().unwrap(), 0));
        write(&mut d, 0, &e["cases"][7]);
        assert!(d.readback().is_err());
        p.update(&mut d).unwrap();
        let changed = d.readback().unwrap();
        native(&changed, 0, &e["cases"][7]);
        assert_ne!(bits(&first, 0), bits(&changed, 0));
        assert_eq!(bits(&first, 512), bits(&changed, 512));
        p.update_com(&mut d).unwrap();
        p.update_camlight(&mut d).unwrap();
        p.update_tendons(&mut d).unwrap();
        d.readback().unwrap();
        p.update_flex_positions(&mut d).unwrap();
        assert!(matches!(
            d.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_faces"
            })
        ));
        p.update_flex_faces(&mut d).unwrap();
        p.update(&mut independent).unwrap();
        native(&independent.readback().unwrap(), 0, &e["cases"][0]);
        drop(other);
        drop(p);
        drop(s);
        native(&d.readback().unwrap(), 0, &e["cases"][7]);
        native(&first, 0, &e["cases"][0]);
        assert_eq!(first.flex_faces().unwrap().fields(), &faces(&e));
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn preserves_old_empty_edge_and_tendon_wake_routes() {
        let s = TransferSession::new(0).unwrap();
        let b = base();
        let e = fixture();
        let old = plan(
            &s,
            FlexPositionModelInput::new(tendons(&b), positions(&b, &b)).unwrap(),
        );
        let mut d = old.create_data(2).unwrap();
        old.update(&mut d).unwrap();
        assert!(d.readback().unwrap().flex_faces().is_none());
        let empty = plan(
            &s,
            FlexPositionModelInput::new(tendons(&b), FlexPositionFields::default())
                .unwrap()
                .with_faces(FlexFaceFields::default())
                .unwrap(),
        );
        let mut d = empty.create_data(5).unwrap();
        empty.update(&mut d).unwrap();
        assert!(
            d.readback()
                .unwrap()
                .flex_faces()
                .unwrap()
                .world(4)
                .unwrap()
                .face_quat
                .is_empty()
        );
        let edge: Value =
            serde_json::from_str(include_str!("../fixtures/flex-edge/edge-state.json")).unwrap();
        let m = &edge["model"];
        let edges = FlexEdgeFields {
            flex_edgeadr: ids(m, "flex_edgeadr"),
            flex_edgenum: ids(m, "flex_edgenum"),
            flex_edge: ids(m, "flex_edge"),
            flexedge_j_rowadr: ids(m, "flexedge_J_rowadr"),
            flexedge_j_rownnz: ids(m, "flexedge_J_rownnz"),
            flexedge_j_colind: ids(m, "flexedge_J_colind"),
        };
        let model = input(&b, &e).with_edges(edges).unwrap();
        let parents = ids(&b["model"], "body_parentid");
        let nums = ids(&b["model"], "body_jntnum");
        let mut trees = vec![-1; parents.len()];
        let mut count = 0;
        for i in 1..trees.len() {
            trees[i] = trees[parents[i] as usize];
            if trees[i] < 0 && nums[i] > 0 {
                trees[i] = count;
                count += 1;
            }
        }
        let wake = TendonWakeFields {
            body_treeid: trees,
            tendon_limited: vec![false; 2],
            tendon_range: ParameterBatch::new(1, 4, vec![-10.0, 10.0, -10.0, 10.0]).unwrap(),
            tendon_margin: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
            sleep_enabled: true,
            island_disabled: false,
        };
        let model = TendonWakeModelInput::with_flex_positions(model, wake).unwrap();
        assert!(model.flex_faces().is_some());
        assert!(model.flex_edges().is_some());
        let p = KinematicsPlan::with_tendon_wake(
            &s,
            model,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut d = p.create_data(5).unwrap();
        for w in 0..5 {
            write(&mut d, w, &e["cases"][w]);
            d.write_world_qvel(w, &floats(&edge["cases"][w], "qvel"))
                .unwrap();
        }
        p.update(&mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..5 {
            native(&out, w, &e["cases"][w]);
            assert!(out.sleep_trees().is_some());
        }
        let preserved = bits(&out, 0);
        d.write_world_qvel(0, &[0.0; 65]).unwrap();
        p.update_flex_edges(&mut d).unwrap();
        assert_eq!(preserved, bits(&d.readback().unwrap(), 0));
    }
}
