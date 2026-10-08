#![cfg(feature = "cuda-probe")]
//! G01缓存失效与被动拉伸子集。
use mjwarp_rs as engine;
#[path = "support/flex_stretch_reference.rs"]
mod reference;
use reference::{edges, fields, fixture, floats, input};
use sha2::{Digest, Sha256};
#[test]
fn reference_hashes_pin_native_geometry_and_independent_dense_derivatives() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/flex-stretch/manifest.json")).unwrap();
    for f in manifest["files"].as_array().unwrap() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/flex-stretch")
            .join(f["path"].as_str().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(p).unwrap())),
            f["sha256"].as_str().unwrap()
        );
    }
    let v = fixture();
    let i = input(&v).with_hessian(fields(&v)).unwrap();
    assert_eq!((i.nflex(), i.nflexnode(), i.nflexvert()), (5, 8, 47));
    assert_eq!(i.edges().unwrap().nflexedge(), 131);
    assert_eq!(i.hessian().unwrap().flex_stiffness.len(), 210);
}
#[test]
fn rejects_hessian_lengths_coefficients_and_unsafe_element_mappings() {
    let v = fixture();
    let h = fields(&v);
    for k in 0..11 {
        let mut b = h.clone();
        match k {
            0 => {
                b.flex_dim.pop();
            }
            1 => {
                b.flex_rigid.pop();
            }
            2 => {
                b.flex_elemadr.pop();
            }
            3 => {
                b.flex_elemnum.pop();
            }
            4 => {
                b.flex_elemdataadr.pop();
            }
            5 => {
                b.flex_elemedgeadr.pop();
            }
            6 => {
                b.flex_stiffnessadr.pop();
            }
            7 => {
                b.flex_elem.pop();
            }
            8 => {
                b.flex_elemedge.pop();
            }
            9 => {
                b.flexedge_length0.pop();
            }
            _ => {
                b.flex_stiffness.pop();
            }
        }
        assert!(input(&v).with_hessian(b).is_err(), "length {k}");
    }
    for k in 0..13 {
        let mut b = h.clone();
        match k {
            0 => b.flex_dim[2] = 4,
            1 => b.flex_elemadr[2] = -1,
            2 => b.flex_elemnum[2] = i32::MAX,
            3 => b.flex_elemdataadr[2] = 0,
            4 => b.flex_elemedgeadr[2] = 0,
            5 => b.flex_stiffnessadr[2] = -2,
            6 => b.flex_stiffnessadr[3] = 85,
            7 => b.flex_elem[195] = -1,
            8 => b.flex_elem[195] = b.flex_elem[196],
            9 => b.flex_elemedge[291] = 131,
            10 => b.flex_elemedge[291] = b.flex_elemedge[292],
            11 => b.flex_rigid[2] = true,
            _ => b.flex_stiffness.extend([0.0; 30]),
        }
        assert!(input(&v).with_hessian(b).is_err(), "topology {k}");
    }
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut b = h.clone();
        b.flexedge_length0[0] = x;
        assert!(input(&v).with_hessian(b).is_err());
        let mut b = h.clone();
        b.flex_stiffness[0] = x;
        assert!(input(&v).with_hessian(b).is_err());
    }
    let mut b = h.clone();
    b.flexedge_length0[0] = -1.0;
    assert!(input(&v).with_hessian(b).is_err());
    let i = input(&v);
    let no_edges =
        engine::model::FlexPositionModelInput::new(i.tendons().clone(), i.fields().clone())
            .unwrap();
    assert!(no_edges.with_hessian(h.clone()).is_err());
    let mut e = edges(&v);
    let base = e.flex_edgeadr[2] as usize * 2;
    e.flex_edge[base..base + 2].reverse();
    assert!(
        input(&v)
            .with_hessian(h.clone())
            .unwrap()
            .with_edges(e)
            .is_ok()
    );
    let mut e = edges(&v);
    e.flex_edge[base] = e.flex_edge[base + 2];
    e.flex_edge[base + 1] = e.flex_edge[base + 3];
    assert!(input(&v).with_hessian(h).unwrap().with_edges(e).is_err());
}
#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use engine::{
        diagnostics::TransferError,
        model::{CamLightParameters, KinematicsParameters},
        physics::{KinematicsPlan, KinematicsSnapshot},
        runtime::TransferSession,
    };
    fn plan(s: &TransferSession, i: engine::model::FlexPositionModelInput) -> KinematicsPlan {
        KinematicsPlan::with_flex_positions(
            s,
            i,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }
    fn bits(s: &KinematicsSnapshot, w: usize) -> Vec<u32> {
        let h = s.flex_hessian().unwrap().world(w).unwrap();
        h.flexvert_hessian
            .iter()
            .chain(h.flexedge_hessian)
            .map(|v| v.to_bits())
            .collect()
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn matches_independent_dense_projection_and_preserves_cached_blocks() {
        let s = TransferSession::new(0).unwrap();
        let v = fixture();
        let p = plan(&s, input(&v).with_hessian(fields(&v)).unwrap());
        let mut count = 0;
        let mut scalars = 0;
        let mut max = 0.0f64;
        for worlds in [1, 2, 5, 513] {
            let mut d = p.create_data(worlds).unwrap();
            assert!(matches!(
                p.update_flex_hessian(&mut d),
                Err(TransferError::StageNotReady {
                    stage: "flex_positions"
                })
            ));
            p.update_rigid(&mut d).unwrap();
            p.update_flex_positions(&mut d).unwrap();
            assert!(matches!(
                p.update_flex_hessian(&mut d),
                Err(TransferError::StageNotReady {
                    stage: "flex_edges"
                })
            ));
            for round in 0..4 {
                for w in 0..worlds {
                    d.write_world_qpos(w, &floats(&v["cases"][(w + round * 3) % 12], "qpos"))
                        .unwrap();
                }
                p.update(&mut d).unwrap();
                assert!(d.readback().unwrap().flex_hessian().is_none());
                p.update_flex_hessian(&mut d).unwrap();
                let out = d.readback().unwrap();
                for w in 0..worlds {
                    let c = &v["cases"][(w + round * 3) % 12];
                    let h = out.flex_hessian().unwrap().world(w).unwrap();
                    max = max.max(reference::compare(
                        h.flexvert_hessian,
                        &c["flexvert_hessian"],
                    ));
                    max = max.max(reference::compare(
                        h.flexedge_hessian,
                        &c["flexedge_hessian"],
                    ));
                    reference::compare(
                        out.flex_positions()
                            .unwrap()
                            .world(w)
                            .unwrap()
                            .flexvert_xpos,
                        &c["flexvert_xpos"],
                    );
                    reference::compare(
                        out.flex_edges().unwrap().world(w).unwrap().flexedge_length,
                        &c["flexedge_length"],
                    );
                    assert_eq!(
                        out.flex_positions()
                            .unwrap()
                            .world(w)
                            .unwrap()
                            .flex_hessian_valid,
                        &[true; 5]
                    );
                    count += 1;
                    scalars += h.flexvert_hessian.len() + h.flexedge_hessian.len();
                }
                p.update_flex_hessian(&mut d).unwrap();
                let again = d.readback().unwrap();
                for w in 0..worlds {
                    assert_eq!(bits(&out, w), bits(&again, w));
                }
            }
        }
        assert_eq!(count, 2084);
        assert_eq!(scalars, 3044724);
        println!("flex-stretch states={count} scalars={scalars} max_abs_error={max:e}");
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn transposes_reversed_edges_and_checks_inactive_stiffness_routes() {
        let s = TransferSession::new(0).unwrap();
        let v = fixture();
        let mut e = edges(&v);
        for pair in e.flex_edge.as_chunks_mut::<2>().0 {
            pair.reverse();
        }
        let p = plan(
            &s,
            input(&v)
                .with_edges(e)
                .unwrap()
                .with_hessian(fields(&v))
                .unwrap(),
        );
        let mut d = p.create_data(1).unwrap();
        d.write_qpos(&floats(&v["cases"][8], "qpos")).unwrap();
        p.update(&mut d).unwrap();
        p.update_flex_hessian(&mut d).unwrap();
        let out = d.readback().unwrap();
        let h = out.flex_hessian().unwrap().world(0).unwrap();
        reference::compare(h.flexvert_hessian, &v["cases"][8]["flexvert_hessian"]);
        let expected = floats(&v["cases"][8], "flexedge_hessian");
        for (a, b) in h
            .flexedge_hessian
            .as_chunks::<9>()
            .0
            .iter()
            .zip(expected.as_chunks::<9>().0)
        {
            for r in 0..3 {
                for c in 0..3 {
                    assert!(
                        (a[3 * r + c] - b[3 * c + r]).abs() <= 2e-5 + 2e-5 * b[3 * c + r].abs()
                    );
                }
            }
        }
        let mut h = fields(&v);
        h.flex_stiffness[0] = 0.0;
        h.flex_stiffnessadr[3] = -1;
        h.flex_stiffness.truncate(84);
        let p = plan(&s, input(&v).with_hessian(h).unwrap());
        let mut d = p.create_data(2).unwrap();
        p.update(&mut d).unwrap();
        p.update_flex_hessian(&mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..2 {
            assert!(bits(&out, w).iter().all(|&b| b == 0 || b == 0x80000000));
            assert_eq!(
                out.flex_positions()
                    .unwrap()
                    .world(w)
                    .unwrap()
                    .flex_hessian_valid,
                &[true; 5]
            );
        }
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn preserves_geometric_cache_across_qvel_and_rejects_foreign_plans() {
        let s = TransferSession::new(0).unwrap();
        let v = fixture();
        let i = input(&v).with_hessian(fields(&v)).unwrap();
        let p = plan(&s, i.clone());
        let other = plan(&s, i);
        let mut d = p.create_data(2).unwrap();
        p.update(&mut d).unwrap();
        p.update_flex_hessian(&mut d).unwrap();
        let before = d.readback().unwrap();
        assert!(matches!(
            other.update_flex_hessian(&mut d),
            Err(TransferError::ModelMismatch)
        ));
        assert!(d.write_qpos(&[f32::NAN]).is_err());
        assert_eq!(bits(&before, 0), bits(&d.readback().unwrap(), 0));
        d.write_qvel(&vec![0.5; d.worlds() * v["nv"].as_u64().unwrap() as usize])
            .unwrap();
        p.update_flex_edges(&mut d).unwrap();
        p.update_flex_hessian(&mut d).unwrap();
        assert_eq!(bits(&before, 0), bits(&d.readback().unwrap(), 0));
        p.update_com(&mut d).unwrap();
        p.update_camlight(&mut d).unwrap();
        p.update_spatial_tendons(&mut d).unwrap();
        p.update_tendons(&mut d).unwrap();
        p.update_flex_edges(&mut d).unwrap();
        assert_eq!(bits(&before, 0), bits(&d.readback().unwrap(), 0));
        d.write_world_qpos(1, &floats(&v["cases"][1], "qpos"))
            .unwrap();
        p.update(&mut d).unwrap();
        assert!(d.readback().unwrap().flex_hessian().is_none());
        p.update_flex_hessian(&mut d).unwrap();
        let retained = d.readback().unwrap();
        drop(p);
        drop(other);
        drop(s);
        drop(d);
        reference::compare(
            retained
                .flex_hessian()
                .unwrap()
                .world(1)
                .unwrap()
                .flexedge_hessian,
            &v["cases"][1]["flexedge_hessian"],
        );
        assert_eq!(bits(&before, 0), bits(&before, 1));
    }
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn handles_empty_enabled_and_legacy_disabled_routes() {
        let s = TransferSession::new(0).unwrap();
        let v = fixture();
        let i = input(&v);
        let empty = engine::model::FlexPositionModelInput::new(
            i.tendons().clone(),
            engine::model::FlexPositionFields::default(),
        )
        .unwrap()
        .with_edges(engine::model::FlexEdgeFields::default())
        .unwrap()
        .with_hessian(engine::model::FlexHessianFields::default())
        .unwrap();
        let p = plan(&s, empty);
        let mut d = p.create_data(513).unwrap();
        p.update(&mut d).unwrap();
        assert!(d.readback().unwrap().flex_hessian().is_none());
        p.update_flex_hessian(&mut d).unwrap();
        let out = d.readback().unwrap();
        let h = out.flex_hessian().unwrap();
        assert!(h.world(512).unwrap().flexvert_hessian.is_empty());
        assert!(h.world(513).is_err());
        assert!(p.flex_hessian_fields().is_some());
        assert!(d.flex_hessian_fields().is_some());
        let p = plan(&s, i);
        let mut d = p.create_data(1).unwrap();
        p.update(&mut d).unwrap();
        p.update_flex_hessian(&mut d).unwrap();
        assert!(d.readback().unwrap().flex_hessian().is_none());
        assert!(p.flex_hessian_fields().is_none());
        assert!(d.flex_hessian_fields().is_none());
    }
}
