#![cfg(feature = "cuda-probe")]
//! G01肌腱唤醒副作用验收。
//! 产品测试只读取冻结原生参考。

use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexPositionFields,
    FlexPositionModelInput, MocapModelInput, ParameterBatch, SleepTreeState, SpatialTendonGeometry,
    TendonFields, TendonModelInput, TendonWakeFields, TendonWakeModelInput,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/tendon-wake/tree-state.json")).unwrap()
}
fn base_fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/mixed-tendon/mixed-tree.json")).unwrap()
}
fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
fn cam(v: &Value) -> CamLightModelInput {
    let m = &v["model"];
    let attached = AttachedModelInput::new(
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
    CamLightModelInput::new(
        MocapModelInput::new(attached, ids(m, "body_mocapid")).unwrap(),
        CamLightFields::default(),
    )
    .unwrap()
}
fn model() -> TendonModelInput {
    let v = base_fixture();
    let m = &v["model"];
    TendonModelInput::with_geometry(
        cam(&v),
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
        SpatialTendonGeometry {
            geom_type: ids(m, "geom_type"),
            geom_size: ParameterBatch::new(1, 12, floats(m, "geom_size")).unwrap(),
        },
    )
    .unwrap()
}
fn fields() -> TendonWakeFields {
    let v = fixture();
    TendonWakeFields {
        body_treeid: ids(&v, "body_treeid"),
        tendon_limited: ids(&v, "tendon_limited")
            .into_iter()
            .map(|v| v != 0)
            .collect(),
        tendon_range: ParameterBatch::new(3, 28, floats(&v, "tendon_range")).unwrap(),
        tendon_margin: ParameterBatch::new(2, 14, floats(&v, "tendon_margin")).unwrap(),
        sleep_enabled: true,
        island_disabled: false,
    }
}
#[test]
fn reference_hashes_pin_native_tree_states_and_model_inputs() {
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/tendon-wake/manifest.json")).unwrap();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/tendon-wake");
    for item in manifest["files"].as_array().unwrap() {
        let bytes = std::fs::read(dir.join(item["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            item["sha256"].as_str().unwrap()
        );
    }
    let v = fixture();
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 2102);
    assert_eq!(v["cases"].as_array().unwrap().len(), 24);
    let m = TendonWakeModelInput::new(model(), fields()).unwrap();
    assert_eq!(m.ntree(), 3);
    assert_eq!(m.tendons().fields().wrap_type.len(), 43);
    assert!(m.flex_positions().is_none());
    assert_eq!(m.fields().tendon_range.batches(), 3);
    assert_eq!(m.fields().tendon_margin.batches(), 2);
}
#[test]
fn rejects_invalid_tree_mapping_limits_and_parameter_widths() {
    for kind in 0..8 {
        let mut f = fields();
        match kind {
            0 => {
                f.body_treeid.pop();
            }
            1 => f.body_treeid[0] = 0,
            2 => f.body_treeid[2] = 0,
            3 => {
                f.tendon_limited.pop();
            }
            4 => f.tendon_range = ParameterBatch::new(1, 27, vec![0.0; 27]).unwrap(),
            5 => f.tendon_margin = ParameterBatch::new(1, 13, vec![0.0; 13]).unwrap(),
            6 => {
                let mut v = floats(&fixture(), "tendon_range");
                v[0] = 3.0;
                f.tendon_range = ParameterBatch::new(3, 28, v).unwrap();
            }
            _ => f.tendon_margin = ParameterBatch::new(1, 14, vec![-1.0; 14]).unwrap(),
        }
        assert!(
            TendonWakeModelInput::new(model(), f).is_err(),
            "boundary {kind}"
        );
    }
    assert!(ParameterBatch::new(1, 1, vec![f32::NAN]).is_err());
}

#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use mjwarp_rs::{
        diagnostics::TransferError,
        model::{CamLightParameters, KinematicsParameters},
        physics::{KinematicsData, KinematicsPlan},
        runtime::TransferSession,
    };
    fn plan(session: &TransferSession, fields: TendonWakeFields) -> KinematicsPlan {
        KinematicsPlan::with_tendon_wake(
            session,
            TendonWakeModelInput::new(model(), fields).unwrap(),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }
    fn state(c: &Value) -> SleepTreeState {
        SleepTreeState {
            tree_asleep: ids(c, "input_tree_asleep"),
            nbody_awake: c["input_nbody_awake"].as_i64().unwrap() as i32,
            nv_awake: c["input_nv_awake"].as_i64().unwrap() as i32,
        }
    }
    fn sleep_state(values: Vec<i32>) -> SleepTreeState {
        SleepTreeState {
            tree_asleep: values,
            nbody_awake: 2,
            nv_awake: 2,
        }
    }
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn matches_native_tree_cycles_for_independent_periods_and_repeated_updates() {
        let v = fixture();
        let s = TransferSession::new(0).unwrap();
        let p = plan(&s, fields());
        for worlds in [1, 2, 5, 513] {
            let mut d = p.create_data(worlds).unwrap();
            for round in 0..4 {
                for w in 0..worlds {
                    d.write_world_sleep(w, &state(&v["cases"][6 * round + w % 6]))
                        .unwrap();
                }
                p.update(&mut d).unwrap();
                let out = d.readback().unwrap();
                for w in 0..worlds {
                    let c = &v["cases"][6 * round + w % 6];
                    let a = out.sleep_trees().unwrap().world(w).unwrap();
                    assert_eq!(a.tree_asleep, ids(c, "tree_asleep"));
                    assert_eq!(a.tree_awake, ids(c, "tree_awake"));
                    assert_eq!(a.ntree_awake, c["ntree_awake"].as_i64().unwrap() as i32);
                    // Warp轻量刷新不沿用原生身体计数。
                    assert_eq!((a.nbody_awake, a.nv_awake), (0, 0));
                    let length = out.tendon().unwrap().world(w).unwrap().ten_length;
                    for (t, (&a, e)) in length
                        .iter()
                        .zip(c["ten_length"].as_array().unwrap())
                        .enumerate()
                    {
                        let e = e.as_f64().unwrap();
                        assert!(
                            (f64::from(a) - e).abs() <= 2e-5 + 2e-5 * e.abs(),
                            "world={w} round={round} tendon={t} actual={a} native={e}"
                        );
                    }
                    println!(
                        "G01-tendon-wake kind=native round={round} world={w} exact_tree_state=true"
                    );
                }
                let previous: Vec<_> = (0..worlds)
                    .map(|w| {
                        out.sleep_trees()
                            .unwrap()
                            .world(w)
                            .unwrap()
                            .tree_asleep
                            .to_vec()
                    })
                    .collect();
                p.update(&mut d).unwrap();
                let again = d.readback().unwrap();
                for (w, expected) in previous.iter().enumerate() {
                    assert_eq!(
                        again.sleep_trees().unwrap().world(w).unwrap().tree_asleep,
                        expected
                    );
                }
            }
        }
    }
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn keeps_strict_margin_boundary_without_relaxing_float_comparisons() {
        let s = TransferSession::new(0).unwrap();
        let mut f = fields();
        f.tendon_limited.fill(false);
        f.tendon_limited[0] = true;
        let mut ranges = [-100.0, 100.0].repeat(42);
        for r in 0..3 {
            ranges[28 * r] = if r == 2 { -0.0625 } else { -0.125 };
            ranges[28 * r + 1] = if r == 2 { 0.0625 } else { 0.125 };
        }
        f.tendon_range = ParameterBatch::new(3, 28, ranges).unwrap();
        f.tendon_margin =
            ParameterBatch::new(2, 14, [vec![0.125; 14], vec![0.0; 14]].concat()).unwrap();
        let p = plan(&s, f);
        let mut d = p.create_data(513).unwrap();
        for w in 0..513 {
            d.write_world_sleep(w, &sleep_state(vec![-11, 2, 1]))
                .unwrap();
        }
        p.update(&mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..513 {
            let expected = if w % 3 == 2 && w % 2 == 0 {
                vec![-11, -11, -11]
            } else {
                vec![-11, 2, 1]
            };
            assert_eq!(
                out.sleep_trees().unwrap().world(w).unwrap().tree_asleep,
                expected
            );
        }
    }
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn wakes_fixed_geometry_and_pulley_paths_without_cpu_length_transfer() {
        let s = TransferSession::new(0).unwrap();
        for tendon in [0, 8, 11, 13] {
            let mut f = fields();
            f.tendon_limited.fill(false);
            f.tendon_limited[tendon] = true;
            f.tendon_range = ParameterBatch::new(1, 28, [100.0, 101.0].repeat(14)).unwrap();
            f.tendon_margin = ParameterBatch::new(1, 14, vec![0.0; 14]).unwrap();
            let p = plan(&s, f);
            let mut d = p.create_data(3).unwrap();
            for (w, awake) in [-1, -11, -17].into_iter().enumerate() {
                d.write_world_sleep(w, &sleep_state(vec![awake, 2, 1]))
                    .unwrap();
            }
            p.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for (w, awake) in [-1, -11, -17].into_iter().enumerate() {
                let v = awake.min(-11);
                assert_eq!(
                    out.sleep_trees().unwrap().world(w).unwrap().tree_asleep,
                    [awake, v, v]
                );
            }
        }
    }
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn honors_sleep_and_island_switches_and_empty_tendons() {
        let s = TransferSession::new(0).unwrap();
        for (sleep, island) in [(false, false), (true, true)] {
            let mut f = fields();
            f.sleep_enabled = sleep;
            f.island_disabled = island;
            let p = plan(&s, f);
            let mut d = p.create_data(5).unwrap();
            for w in 0..5 {
                d.write_world_sleep(w, &sleep_state(vec![-11, 2, 1]))
                    .unwrap();
            }
            p.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..5 {
                let a = out.sleep_trees().unwrap().world(w).unwrap();
                assert_eq!(a.tree_asleep, [-11, 2, 1]);
                assert_eq!(a.tree_awake, [1, 0, 0]);
                assert_eq!((a.ntree_awake, a.nbody_awake, a.nv_awake), (1, 2, 2));
            }
        }
        let mut f = fields();
        f.tendon_limited.clear();
        f.tendon_range = ParameterBatch::new(3, 0, vec![]).unwrap();
        f.tendon_margin = ParameterBatch::new(2, 0, vec![]).unwrap();
        let empty = TendonModelInput::new(cam(&base_fixture()), TendonFields::default()).unwrap();
        let flex = FlexPositionModelInput::new(empty, FlexPositionFields::default()).unwrap();
        let model = TendonWakeModelInput::with_flex_positions(flex, f).unwrap();
        let p = KinematicsPlan::with_tendon_wake(
            &s,
            model,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut d = p.create_data(2).unwrap();
        d.write_world_sleep(0, &sleep_state(vec![-11, 2, 1]))
            .unwrap();
        p.update(&mut d).unwrap();
        let out = d.readback().unwrap();
        let a = out.sleep_trees().unwrap().world(0).unwrap();
        assert_eq!(a.tree_asleep, [-11, 2, 1]);
        assert_eq!((a.ntree_awake, a.nbody_awake, a.nv_awake), (1, 2, 2));
        assert_eq!(out.flex_positions().unwrap().nflexnode(), 0);
    }
    fn ready(p: &KinematicsPlan, d: &mut KinematicsData) {
        p.update(d).unwrap();
        d.readback().unwrap();
    }
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn preserves_readiness_identity_rejected_writes_and_snapshot_ownership() {
        let s = TransferSession::new(0).unwrap();
        let p = plan(&s, fields());
        let other = plan(&s, fields());
        for worlds in [513, 1, 5] {
            let mut d = p.create_data(worlds).unwrap();
            assert!(matches!(
                p.update_tendon_wake(&mut d),
                Err(TransferError::StageNotReady { stage: "tendon" })
            ));
            ready(&p, &mut d);
            let held = d.readback().unwrap();
            let old = held
                .sleep_trees()
                .unwrap()
                .world(0)
                .unwrap()
                .tree_asleep
                .to_vec();
            for bad in [vec![3, 1, 2], vec![1, 1, 2], vec![-11]] {
                assert!(d.write_world_sleep(0, &sleep_state(bad)).is_err());
                d.readback().unwrap();
            }
            assert!(
                d.write_world_sleep(worlds, &sleep_state(vec![0, 1, 2]))
                    .is_err()
            );
            assert!(matches!(
                other.update_tendon_wake(&mut d),
                Err(TransferError::ModelMismatch)
            ));
            d.readback().unwrap();
            d.write_world_sleep(0, &sleep_state(vec![0, 1, 2])).unwrap();
            assert!(matches!(
                d.readback(),
                Err(TransferError::StageNotReady {
                    stage: "tendon_wake"
                })
            ));
            p.update_tendon_wake(&mut d).unwrap();
            let changed = d.readback().unwrap();
            assert_eq!(
                changed.sleep_trees().unwrap().world(0).unwrap().tree_asleep,
                [0, 1, 2]
            );
            assert_ne!(
                changed.sleep_trees().unwrap().world(0).unwrap().tree_asleep,
                old
            );
            p.update_fixed_tendons(&mut d).unwrap();
            assert!(matches!(
                p.update_tendon_wake(&mut d),
                Err(TransferError::StageNotReady { stage: "tendon" })
            ));
            ready(&p, &mut d);
            d.write_world_qpos(0, &floats(&base_fixture()["model"], "qpos0"))
                .unwrap();
            assert!(d.readback().is_err());
            ready(&p, &mut d);
            assert_eq!(
                held.sleep_trees().unwrap().world(0).unwrap().tree_asleep,
                old
            );
        }
        let mut d = p.create_data(2).unwrap();
        ready(&p, &mut d);
        let mut independent = p.create_data(2).unwrap();
        independent
            .write_world_sleep(0, &sleep_state(vec![0, 1, 2]))
            .unwrap();
        ready(&p, &mut independent);
        drop(independent);
        drop(p);
        drop(other);
        drop(s);
        assert_eq!(
            d.readback()
                .unwrap()
                .sleep_trees()
                .unwrap()
                .world(0)
                .unwrap()
                .tree_asleep,
            [-11, -11, -11]
        );
    }
}
