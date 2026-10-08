#![cfg(feature = "cuda-probe")]
//! G01柔体边子集验收。
//! 产品测试只读静态原生参考。
use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexEdgeFields,
    FlexPositionFields, FlexPositionModelInput, MocapModelInput, TendonFields, TendonModelInput,
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
    serde_json::from_str(include_str!("../fixtures/flex-edge/edge-state.json")).unwrap()
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
fn positions(v: &Value) -> FlexPositionFields {
    let m = &v["model"];
    FlexPositionFields {
        flex_interp: ids(m, "flex_interp"),
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
fn edges(v: &Value) -> FlexEdgeFields {
    let m = &v["model"];
    FlexEdgeFields {
        flex_edgeadr: ids(m, "flex_edgeadr"),
        flex_edgenum: ids(m, "flex_edgenum"),
        flex_edge: ids(m, "flex_edge"),
        flexedge_j_rowadr: ids(m, "flexedge_J_rowadr"),
        flexedge_j_rownnz: ids(m, "flexedge_J_rownnz"),
        flexedge_j_colind: ids(m, "flexedge_J_colind"),
    }
}
fn input(b: &Value, e: &Value) -> FlexPositionModelInput {
    FlexPositionModelInput::new(tendons(b), positions(b))
        .unwrap()
        .with_edges(edges(e))
        .unwrap()
}
#[test]
fn reference_hashes_pin_native_edges_and_base_inputs() {
    let m: Value =
        serde_json::from_str(include_str!("../fixtures/flex-edge/manifest.json")).unwrap();
    for f in m["files"].as_array().unwrap() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/flex-edge")
            .join(f["path"].as_str().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(p).unwrap())),
            f["sha256"].as_str().unwrap()
        );
    }
    let b = base();
    let e = fixture();
    let m = input(&b, &e);
    let f = m.edges().unwrap();
    assert_eq!((f.nflexedge(), f.nnz()), (284, 10));
    assert_eq!(&f.flexedge_j_rownnz[..6], &[1, 2, 1, 3, 2, 1]);
    assert!(f.flexedge_j_rownnz[6..].iter().all(|&n| n == 0));
    for c in 0..12 {
        for field in ["qpos", "mocap_pos", "mocap_quat"] {
            assert_eq!(b["cases"][c][field], e["cases"][c][field]);
        }
    }
}
#[test]
fn rejects_invalid_edge_partitions_endpoints_and_sparse_rows() {
    let b = base();
    let e = fixture();
    let f = edges(&e);
    let check = |f| {
        FlexPositionModelInput::new(tendons(&b), positions(&b))
            .unwrap()
            .with_edges(f)
    };
    let mut cases = Vec::new();
    let mut x = f.clone();
    x.flex_edgeadr[1] = 0;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edgenum[0] = -1;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edgenum[3] -= 1;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edge[0] = -1;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edge[0] = 5;
    cases.push(x);
    let mut x = f.clone();
    x.flex_edge.pop();
    cases.push(x);
    let mut x = f.clone();
    x.flex_edgeadr.clear();
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rowadr[1] = 0;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rowadr[6] = 11;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rowadr.pop();
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rownnz[0] = -1;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_rownnz[6] = 1;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind[0] = 1;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind[0] = -1;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind[0] = 65;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind[2] = 0;
    cases.push(x);
    let mut x = f.clone();
    x.flexedge_j_colind.push(65);
    cases.push(x);
    for x in cases {
        assert!(check(x).is_err());
    }
    let mut zero = f;
    zero.flexedge_j_rownnz.fill(0);
    zero.flexedge_j_rowadr.fill(0);
    zero.flexedge_j_colind.clear();
    check(zero).unwrap();
    let mut reserved = edges(&e);
    reserved.flexedge_j_rownnz.fill(0);
    reserved.flexedge_j_rowadr.fill(0);
    check(reserved).unwrap();
    FlexPositionModelInput::new(tendons(&b), FlexPositionFields::default())
        .unwrap()
        .with_edges(FlexEdgeFields::default())
        .unwrap();
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
    fn plan(s: &TransferSession, b: &Value, e: &Value) -> KinematicsPlan {
        KinematicsPlan::with_flex_positions(
            s,
            input(b, e),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }
    fn write(d: &mut KinematicsData, w: usize, c: &Value) {
        d.write_world_qpos(w, &floats(c, "qpos")).unwrap();
        d.write_world_qvel(w, &floats(c, "qvel")).unwrap();
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
                "G01 edge index={i} actual={a} expected={e} error={err}"
            );
        }
        max
    }
    fn native(o: &KinematicsSnapshot, w: usize, c: &Value) -> f64 {
        let f = o.flex_edges().unwrap().world(w).unwrap();
        compare(f.flexedge_length, &c["flexedge_length"])
            .max(compare(f.flexedge_velocity, &c["flexedge_velocity"]))
            .max(compare(f.flexedge_jacobian, &c["flexedge_J"]))
    }
    fn bits(o: &KinematicsSnapshot, w: usize) -> Vec<u32> {
        let f = o.flex_edges().unwrap().world(w).unwrap();
        f.flexedge_length
            .iter()
            .chain(f.flexedge_velocity)
            .chain(f.flexedge_jacobian)
            .map(|v| v.to_bits())
            .collect()
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn matches_native_edges_across_uneven_worlds_and_repeated_updates() {
        let s = TransferSession::new(0).unwrap();
        let b = base();
        let e = fixture();
        let p = plan(&s, &b, &e);
        let (mut count, mut max) = (0, 0.0_f64);
        for nw in [1, 2, 5, 513] {
            let mut d = p.create_data(nw).unwrap();
            for round in 0..4 {
                for w in 0..nw {
                    write(&mut d, w, &e["cases"][(w + round * 3) % 12]);
                }
                p.update(&mut d).unwrap();
                let out = d.readback().unwrap();
                for w in 0..nw {
                    max = max.max(native(&out, w, &e["cases"][(w + round * 3) % 12]));
                    count += 1;
                }
                p.update_flex_edges(&mut d).unwrap();
                let repeated = d.readback().unwrap();
                for w in 0..nw {
                    assert_eq!(bits(&out, w), bits(&repeated, w));
                }
            }
        }
        assert_eq!(count, 2084);
        println!(
            "G01-flex-edge states={count} scalars={} max_abs_error={max}",
            count * 578
        );
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn preserves_qvel_readiness_identity_snapshot_ownership_and_lifetimes() {
        let s = TransferSession::new(0).unwrap();
        let b = base();
        let e = fixture();
        let p = plan(&s, &b, &e);
        let other = plan(&s, &b, &e);
        let mut d = p.create_data(513).unwrap();
        let mut independent = p.create_data(1).unwrap();
        assert_eq!(d.flex_edge_fields(), p.flex_edge_fields());
        assert!(matches!(
            p.update_flex_edges(&mut d),
            Err(TransferError::StageNotReady { stage: "com" })
        ));
        p.update_rigid(&mut d).unwrap();
        p.update_com(&mut d).unwrap();
        assert!(matches!(
            p.update_flex_edges(&mut d),
            Err(TransferError::StageNotReady {
                stage: "flex_positions"
            })
        ));
        for w in 0..513 {
            write(&mut d, w, &e["cases"][w % 12]);
        }
        p.update(&mut d).unwrap();
        let held = d.readback().unwrap();
        assert!(matches!(
            other.update_flex_edges(&mut d),
            Err(TransferError::ModelMismatch)
        ));
        for bad in [
            vec![],
            vec![0.0; 64],
            vec![f32::NAN; 65],
            vec![f32::INFINITY; 65],
        ] {
            assert!(d.write_world_qvel(0, &bad).is_err());
        }
        assert!(d.write_world_qvel(513, &[0.0; 65]).is_err());
        assert!(d.write_qvel(&[0.0; 65]).is_err());
        assert_eq!(bits(&held, 0), bits(&d.readback().unwrap(), 0));
        d.write_world_qvel(0, &[0.0; 65]).unwrap();
        assert!(matches!(
            d.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_edges"
            })
        ));
        p.update_flex_edges(&mut d).unwrap();
        let after = d.readback().unwrap();
        let f = after.flex_edges().unwrap().world(0).unwrap();
        assert!(f.flexedge_velocity.iter().all(|&v| v == 0.0));
        assert_eq!(
            f.flexedge_length,
            held.flex_edges().unwrap().world(0).unwrap().flexedge_length
        );
        assert_eq!(
            f.flexedge_jacobian,
            held.flex_edges()
                .unwrap()
                .world(0)
                .unwrap()
                .flexedge_jacobian
        );
        assert_ne!(bits(&after, 0), bits(&held, 0));
        assert_eq!(bits(&after, 512), bits(&held, 512));
        p.update_com(&mut d).unwrap();
        assert!(matches!(
            d.readback(),
            Err(TransferError::StageNotReady {
                stage: "spatial_tendon"
            })
        ));
        p.update_tendons(&mut d).unwrap();
        assert!(matches!(
            d.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_edges"
            })
        ));
        p.update_flex_edges(&mut d).unwrap();
        p.update_flex_positions(&mut d).unwrap();
        assert!(matches!(
            d.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_edges"
            })
        ));
        p.update_flex_edges(&mut d).unwrap();
        d.write_qvel(&vec![0.0; 513 * 65]).unwrap();
        p.update_flex_edges(&mut d).unwrap();
        p.update(&mut independent).unwrap();
        let saved = independent.readback().unwrap();
        drop(p);
        drop(other);
        drop(s);
        assert_eq!(held.flex_edges().unwrap().fields(), &edges(&e));
        assert_eq!(bits(&saved, 0), bits(&independent.readback().unwrap(), 0));
        assert_ne!(bits(&held, 0), bits(&d.readback().unwrap(), 0));
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn handles_coincident_reversed_static_and_same_body_edges() {
        let s = TransferSession::new(0).unwrap();
        let b = base();
        let bp = floats(&b["model"], "body_pos");
        let mut verts = Vec::from(&bp[3..6]);
        verts.extend([0.0; 3]);
        verts.extend([1.0, 0.0, 0.0]);
        verts.extend([bp[3], bp[4] + 2.0, bp[5]]);
        let f = FlexPositionFields {
            flex_interp: vec![0],
            flex_cellnum: vec![1; 3],
            flex_nodeadr: vec![0],
            flex_nodenum: vec![0],
            flex_vertadr: vec![0],
            flex_vertnum: vec![4],
            flex_centered: vec![false],
            flex_vertbodyid: vec![0, 1, 1, 0],
            flex_vert: verts,
            flex_vert0: vec![0.0; 12],
            ..Default::default()
        };
        let edges = FlexEdgeFields {
            flex_edgeadr: vec![0],
            flex_edgenum: vec![5],
            flex_edge: vec![0, 1, 3, 2, 2, 3, 1, 2, 0, 3],
            flexedge_j_rowadr: vec![0, 1, 2, 3, 0],
            flexedge_j_rownnz: vec![1, 1, 1, 1, 0],
            flexedge_j_colind: vec![0; 4],
        };
        let input = FlexPositionModelInput::new(tendons(&b), f)
            .unwrap()
            .with_edges(edges)
            .unwrap();
        let p = KinematicsPlan::with_flex_positions(
            &s,
            input,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut d = p.create_data(513).unwrap();
        let mut qvel = vec![0.0; 65];
        qvel[0] = 1.7;
        for w in 0..513 {
            d.write_world_qvel(w, &qvel).unwrap();
        }
        p.update(&mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..513 {
            let f = out.flex_edges().unwrap().world(w).unwrap();
            assert_eq!(f.flexedge_length[0], 0.0);
            assert_eq!(f.flexedge_length[1], f.flexedge_length[2]);
            assert_eq!(f.flexedge_velocity[0], 0.0);
            // Reversing endpoints also reverses direction: length gradient stays equal.
            assert_eq!(f.flexedge_jacobian[1], f.flexedge_jacobian[2]);
            assert!(f.flexedge_jacobian[1].abs() > 0.01);
            assert!((f.flexedge_velocity[1] - 1.7 * f.flexedge_jacobian[1]).abs() < 1e-6);
            assert!(f.flexedge_jacobian[3].abs() < 1e-6);
            assert_eq!(f.flexedge_velocity[4], 0.0);
            assert!((f.flexedge_length[4] - 2.0).abs() < 1e-6);
        }
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn keeps_optional_empty_old_routes_and_tendon_wake_composition() {
        let s = TransferSession::new(0).unwrap();
        let b = base();
        let e = fixture();
        let mut inactive = e.clone();
        inactive["model"] = inactive["inactive_model"].clone();
        let inactive_plan = plan(&s, &b, &inactive);
        let mut inactive_data = inactive_plan.create_data(5).unwrap();
        inactive_plan.update(&mut inactive_data).unwrap();
        let inactive_out = inactive_data.readback().unwrap();
        for w in 0..5 {
            let f = inactive_out.flex_edges().unwrap().world(w).unwrap();
            compare(f.flexedge_length, &e["cases"][0]["flexedge_length"]);
            assert_eq!(f.flexedge_jacobian.len(), 10);
            assert!(f.flexedge_jacobian.iter().all(|&v| v == 0.0));
            assert!(f.flexedge_velocity.iter().all(|&v| v == 0.0));
        }
        let old = KinematicsPlan::with_flex_positions(
            &s,
            FlexPositionModelInput::new(tendons(&b), positions(&b)).unwrap(),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut d = old.create_data(2).unwrap();
        old.update(&mut d).unwrap();
        assert!(d.readback().unwrap().flex_edges().is_none());
        assert!(d.write_world_qvel(0, &[0.0; 65]).is_err());
        let empty = FlexPositionModelInput::new(tendons(&b), FlexPositionFields::default())
            .unwrap()
            .with_edges(FlexEdgeFields::default())
            .unwrap();
        let p = KinematicsPlan::with_flex_positions(
            &s,
            empty,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut d = p.create_data(5).unwrap();
        d.write_qvel(&[0.0; 325]).unwrap();
        p.update(&mut d).unwrap();
        assert!(
            d.readback()
                .unwrap()
                .flex_edges()
                .unwrap()
                .world(4)
                .unwrap()
                .flexedge_length
                .is_empty()
        );
        let parents = ids(&b["model"], "body_parentid");
        let num = ids(&b["model"], "body_jntnum");
        let mut trees = vec![-1; parents.len()];
        let mut count = 0;
        for i in 1..trees.len() {
            trees[i] = trees[parents[i] as usize];
            if trees[i] < 0 && num[i] > 0 {
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
        let model = TendonWakeModelInput::with_flex_positions(input(&b, &e), wake).unwrap();
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
        }
        p.update(&mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..5 {
            native(&out, w, &e["cases"][w]);
            assert_eq!(
                out.sleep_trees().unwrap().world(w).unwrap().ntree_awake,
                count
            );
        }
    }
}
