//! G08@P5 窄相测试：接 G07 宽相候选、冻结上游接触对照、原生 f64 一级判定与
//! 校验面。

use super::*;
use crate::physics::{
    BroadphaseFilter, BroadphaseModel, BroadphaseOptions, BroadphaseOutput, BroadphaseState,
    GeomPair, nxn_broadphase,
};
use serde_json::Value;
use std::collections::BTreeMap;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../../fixtures/narrowphase/cases.json")).unwrap()
}

fn f32_list(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect()
}

fn i32_list(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap() as i32)
        .collect()
}

fn vec2_list(value: &Value) -> Vec<[f32; 2]> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| f32_list(item).try_into().unwrap())
        .collect()
}

fn vec3_list(value: &Value) -> Vec<[f32; 3]> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| f32_list(item).try_into().unwrap())
        .collect()
}

fn vec5_list(value: &Value) -> Vec<[f32; 5]> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| f32_list(item).try_into().unwrap())
        .collect()
}

fn vec2_i32_list(value: &Value) -> Vec<[i32; 2]> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| i32_list(item).try_into().unwrap())
        .collect()
}

fn vec3_flat(value: &Value) -> Vec<[f32; 3]> {
    f32_list(value).as_chunks::<3>().0.to_vec()
}

fn vec9_flat(value: &Value) -> Vec<[f32; 9]> {
    f32_list(value).as_chunks::<9>().0.to_vec()
}

struct OwnedBroadphase {
    ngeom: usize,
    geom_type: Vec<i32>,
    geom_dataid_batches: usize,
    geom_dataid: Vec<i32>,
    aabb_batches: usize,
    geom_aabb_center: Vec<[f32; 3]>,
    geom_aabb_size: Vec<[f32; 3]>,
    rbound_batches: usize,
    geom_rbound: Vec<f32>,
    margin_batches: usize,
    geom_margin: Vec<f32>,
    gap_batches: usize,
    geom_gap: Vec<f32>,
    pairs: Vec<GeomPair>,
    full_pairid: Vec<[i32; 2]>,
}

impl OwnedBroadphase {
    fn new(value: &Value) -> Self {
        let pairs = value["pairs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(value["pairids"].as_array().unwrap())
            .map(|(pair, pairid)| {
                let pair = i32_list(pair);
                let pairid = i32_list(pairid);
                GeomPair {
                    geom1: pair[0],
                    geom2: pair[1],
                    contact_pairid: pairid[0],
                    collision_pairid: pairid[1],
                }
            })
            .collect();
        Self {
            ngeom: value["ngeom"].as_u64().unwrap() as usize,
            geom_type: i32_list(&value["geom_type"]),
            geom_dataid_batches: value["dataid_batches"].as_u64().unwrap() as usize,
            geom_dataid: i32_list(&value["geom_dataid"]),
            aabb_batches: value["aabb_batches"].as_u64().unwrap() as usize,
            geom_aabb_center: vec3_list(&value["geom_aabb_center"]),
            geom_aabb_size: vec3_list(&value["geom_aabb_size"]),
            rbound_batches: value["rbound_batches"].as_u64().unwrap() as usize,
            geom_rbound: f32_list(&value["geom_rbound"]),
            margin_batches: value["margin_batches"].as_u64().unwrap() as usize,
            geom_margin: f32_list(&value["geom_margin"]),
            gap_batches: value["gap_batches"].as_u64().unwrap() as usize,
            geom_gap: f32_list(&value["geom_gap"]),
            pairs,
            full_pairid: vec2_i32_list(&value["full_pairid"]),
        }
    }

    fn view(&self) -> BroadphaseModel<'_> {
        BroadphaseModel {
            ngeom: self.ngeom,
            geom_type: &self.geom_type,
            geom_dataid_batches: self.geom_dataid_batches,
            geom_dataid: &self.geom_dataid,
            aabb_batches: self.aabb_batches,
            geom_aabb_center: &self.geom_aabb_center,
            geom_aabb_size: &self.geom_aabb_size,
            rbound_batches: self.rbound_batches,
            geom_rbound: &self.geom_rbound,
            margin_batches: self.margin_batches,
            geom_margin: &self.geom_margin,
            gap_batches: self.gap_batches,
            geom_gap: &self.geom_gap,
            pairs: &self.pairs,
            full_pairid: &self.full_pairid,
        }
    }
}

fn broadphase_candidates(fixture: &Value) -> BroadphaseOutput {
    let broadphase = OwnedBroadphase::new(&fixture["broadphase"]);
    let nworld = fixture["nworld"].as_u64().unwrap() as usize;
    let mut positions = Vec::new();
    let mut matrices = Vec::new();
    for world in fixture["worlds"].as_array().unwrap() {
        positions.extend(vec3_flat(&world["geom_xpos"]));
        matrices.extend(vec9_flat(&world["geom_xmat"]));
    }
    let state = BroadphaseState {
        nworld,
        geom_xpos: &positions,
        geom_xmat: &matrices,
    };
    nxn_broadphase(
        &broadphase.view(),
        &state,
        BroadphaseOptions {
            filter: BroadphaseFilter::ALL,
            capacity: 4096,
        },
    )
    .unwrap()
}

struct OwnedNarrowphase {
    geom_type: Vec<i32>,
    geom_condim: Vec<i32>,
    geom_priority: Vec<i32>,
    geom_solmix: Vec<f32>,
    geom_solref: Vec<[f32; 2]>,
    geom_solimp: Vec<[f32; 5]>,
    geom_friction: Vec<[f32; 3]>,
    geom_margin: Vec<f32>,
    geom_gap: Vec<f32>,
    geom_adhesion: Vec<f32>,
    geom_size: Vec<[f32; 3]>,
}

impl OwnedNarrowphase {
    fn new(value: &Value) -> Self {
        Self {
            geom_type: i32_list(&value["geom_type"]),
            geom_condim: i32_list(&value["geom_condim"]),
            geom_priority: i32_list(&value["geom_priority"]),
            geom_solmix: f32_list(&value["geom_solmix"]),
            geom_solref: vec2_list(&value["geom_solref"]),
            geom_solimp: vec5_list(&value["geom_solimp"]),
            geom_friction: vec3_list(&value["geom_friction"]),
            geom_margin: f32_list(&value["geom_margin"]),
            geom_gap: f32_list(&value["geom_gap"]),
            geom_adhesion: f32_list(&value["geom_adhesion"]),
            geom_size: vec3_list(&value["geom_size"]),
        }
    }

    fn view(&self, ngeom: usize) -> PrimitiveModel<'_> {
        PrimitiveModel {
            ngeom,
            geom_type: &self.geom_type,
            geom_condim: &self.geom_condim,
            geom_priority: &self.geom_priority,
            geom_solmix: &self.geom_solmix,
            geom_solref: &self.geom_solref,
            geom_solimp: &self.geom_solimp,
            geom_friction: &self.geom_friction,
            geom_margin: &self.geom_margin,
            geom_gap: &self.geom_gap,
            geom_adhesion: &self.geom_adhesion,
            geom_size: &self.geom_size,
        }
    }
}

fn max_abs(product: &[f32], expected: &[f32]) -> f64 {
    product
        .iter()
        .zip(expected.iter())
        .fold(0.0_f64, |acc, (left, right)| {
            acc.max((*left as f64 - *right as f64).abs())
        })
}

fn contact_key(worldid: i32, geom: [i32; 2]) -> (i32, i32, i32) {
    (worldid, geom[0], geom[1])
}

#[test]
fn narrowphase_contacts_match_frozen_upstream_and_native() {
    let fixture = fixture();
    assert_eq!(
        fixture["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g08-p5"
    );
    let ngeom = fixture["narrowphase"]["ngeom"].as_u64().unwrap() as usize;
    let nworld = fixture["nworld"].as_u64().unwrap() as usize;
    let upstream_tolerance = fixture["tolerance"]["upstream_abs"].as_f64().unwrap();
    let native_tolerance = fixture["tolerance"]["native_abs"].as_f64().unwrap();

    // 1) G07 宽相候选与冻结上游候选一致。
    let candidates = broadphase_candidates(&fixture);
    let expected_candidates = &fixture["candidates"];
    assert_eq!(
        candidates.ncollision,
        expected_candidates["ncollision"].as_u64().unwrap() as usize
    );
    let mut candidate_rows: Vec<(i32, [i32; 2], [i32; 2])> = candidates
        .worldids
        .iter()
        .zip(candidates.pairs.iter())
        .zip(candidates.pairids.iter())
        .map(|((world, pair), pairid)| (*world, *pair, *pairid))
        .collect();
    candidate_rows.sort_unstable();
    let expected_pairs = vec2_i32_list(&expected_candidates["pairs"]);
    let expected_pairids = vec2_i32_list(&expected_candidates["pairids"]);
    let expected_worlds = i32_list(&expected_candidates["worldids"]);
    let mut expected_rows: Vec<(i32, [i32; 2], [i32; 2])> = expected_worlds
        .into_iter()
        .zip(expected_pairs)
        .zip(expected_pairids)
        .map(|((world, pair), pairid)| (world, pair, pairid))
        .collect();
    expected_rows.sort_unstable();
    assert_eq!(candidate_rows, expected_rows);

    // 2) 窄相接触与冻结上游逐字段对照。
    let model = OwnedNarrowphase::new(&fixture["narrowphase"]);
    let mut positions = Vec::new();
    let mut matrices = Vec::new();
    for world in fixture["worlds"].as_array().unwrap() {
        positions.extend(vec3_flat(&world["geom_xpos"]));
        matrices.extend(vec9_flat(&world["geom_xmat"]));
    }
    let state = PrimitiveState {
        nworld,
        geom_xpos: &positions,
        geom_xmat: &matrices,
    };
    let contacts = primitive_narrowphase(&model.view(ngeom), &state, &candidates).unwrap();
    let fields = &fixture["contacts"]["fields"];
    let expected_count = fixture["contacts"]["count"].as_u64().unwrap() as usize;
    assert_eq!(contacts.len(), expected_count, "contact count");

    let expected_geom = vec2_i32_list(&fields["geom"]);
    let expected_worldid = i32_list(&fields["worldid"]);
    let expected_dist = f32_list(&fields["dist"]);
    let expected_pos = f32_list(&fields["pos"]);
    let expected_frame = f32_list(&fields["frame"]);
    let expected_includemargin = f32_list(&fields["includemargin"]);
    let expected_dim = i32_list(&fields["dim"]);
    let expected_friction = f32_list(&fields["friction"]);
    let expected_solref = f32_list(&fields["solref"]);
    let expected_solimp = f32_list(&fields["solimp"]);
    let expected_adhesion = f32_list(&fields["adhesion"]);
    let expected_type = i32_list(&fields["type"]);
    let mut expected: BTreeMap<(i32, i32, i32), usize> = BTreeMap::new();
    for index in 0..expected_count {
        let key = contact_key(expected_worldid[index], expected_geom[index]);
        assert!(
            expected.insert(key, index).is_none(),
            "duplicate upstream contact key"
        );
    }
    let mut contact_max = 0.0_f64;
    for contact in &contacts {
        let key = contact_key(contact.worldid, contact.geom);
        let index = *expected.get(&key).expect("product contact not in upstream");
        contact_max = contact_max.max(max_abs(&[contact.dist], &[expected_dist[index]]));
        contact_max = contact_max.max(max_abs(
            &contact.pos,
            &expected_pos[index * 3..index * 3 + 3],
        ));
        contact_max = contact_max.max(max_abs(
            &contact.frame,
            &expected_frame[index * 9..index * 9 + 9],
        ));
        contact_max = contact_max.max(max_abs(
            &[contact.includemargin],
            &[expected_includemargin[index]],
        ));
        contact_max = contact_max.max(max_abs(
            &contact.friction,
            &expected_friction[index * 5..index * 5 + 5],
        ));
        contact_max = contact_max.max(max_abs(
            &contact.solref,
            &expected_solref[index * 2..index * 2 + 2],
        ));
        contact_max = contact_max.max(max_abs(
            &contact.solimp,
            &expected_solimp[index * 5..index * 5 + 5],
        ));
        contact_max = contact_max.max(max_abs(&[contact.adhesion], &[expected_adhesion[index]]));
        assert_eq!(contact.dim, expected_dim[index]);
        assert_eq!(contact.contact_type, expected_type[index]);
        assert_eq!(contact.geomcollisionid, 0);
        for value in contact.friction {
            assert!(value >= MJ_MINMU);
        }
    }
    assert!(
        contact_max <= upstream_tolerance,
        "contact upstream delta {contact_max:.3e}"
    );

    // 3) 原生 f64 mj_collision 一级判定。
    let native = fixture["native"]["worlds"].as_array().unwrap();
    let mut native_max = 0.0_f64;
    for (worldid, world) in native.iter().enumerate() {
        let ncon = world["ncon"].as_u64().unwrap() as usize;
        let world_contacts: Vec<&GeomContact> = contacts
            .iter()
            .filter(|contact| contact.worldid as usize == worldid)
            .collect();
        assert_eq!(world_contacts.len(), ncon, "world {worldid} contact count");
        let native_geom = vec2_i32_list(&world["contacts"]["geom"]);
        let native_dist = f32_list(&world["contacts"]["dist"]);
        let native_pos = f32_list(&world["contacts"]["pos"]);
        let native_frame = f32_list(&world["contacts"]["frame"]);
        let native_includemargin = f32_list(&world["contacts"]["includemargin"]);
        let native_dim = i32_list(&world["contacts"]["dim"]);
        for (index, geom) in native_geom.iter().enumerate() {
            let contact = world_contacts
                .iter()
                .find(|contact| contact.geom == *geom)
                .expect("native contact not found in product");
            native_max = native_max.max(max_abs(&[contact.dist], &[native_dist[index]]));
            native_max =
                native_max.max(max_abs(&contact.pos, &native_pos[index * 3..index * 3 + 3]));
            native_max = native_max.max(max_abs(
                &contact.frame,
                &native_frame[index * 9..index * 9 + 9],
            ));
            native_max = native_max.max(max_abs(
                &[contact.includemargin],
                &[native_includemargin[index]],
            ));
            assert_eq!(contact.dim, native_dim[index]);
        }
    }
    assert!(
        native_max <= native_tolerance,
        "contact native delta {native_max:.3e}"
    );
    println!(
        "narrowphase candidates={} contacts={} upstream_max={contact_max:.3e} native_max={native_max:.3e}",
        candidates.ncollision,
        contacts.len()
    );
}

#[test]
fn rejects_unsupported_pairs_and_invalid_shapes() {
    let fixture = fixture();
    let ngeom = fixture["narrowphase"]["ngeom"].as_u64().unwrap() as usize;
    let model = OwnedNarrowphase::new(&fixture["narrowphase"]);
    let mut positions = Vec::new();
    let mut matrices = Vec::new();
    for world in fixture["worlds"].as_array().unwrap() {
        positions.extend(vec3_flat(&world["geom_xpos"]));
        matrices.extend(vec9_flat(&world["geom_xmat"]));
    }
    let nworld = fixture["nworld"].as_u64().unwrap() as usize;
    let state = PrimitiveState {
        nworld,
        geom_xpos: &positions,
        geom_xmat: &matrices,
    };
    let mut candidates = broadphase_candidates(&fixture);
    candidates.pairs[0] = [GEOM_PLANE, 3];
    let mut broken = OwnedNarrowphase::new(&fixture["narrowphase"]);
    broken.geom_type[3] = 3;
    let error = primitive_narrowphase(&broken.view(ngeom), &state, &candidates).unwrap_err();
    assert!(matches!(error, NarrowphaseError::UnsupportedPair { .. }));

    let mut broken_state_positions = positions.clone();
    broken_state_positions.pop();
    let broken_state = PrimitiveState {
        nworld,
        geom_xpos: &broken_state_positions,
        geom_xmat: &matrices,
    };
    let error = primitive_narrowphase(&model.view(ngeom), &broken_state, &candidates).unwrap_err();
    assert!(matches!(
        error,
        NarrowphaseError::InvalidLength {
            field: "geom_xpos",
            ..
        }
    ));
}

#[test]
fn primitive_geometry_formulas_match_upstream() {
    // 平面-球：dist 为带符号间距，pos 位于球心沿法线内移 (r + 0.5*dist)。
    let (dist, pos) = plane_sphere([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.05], 0.1);
    assert_eq!(dist, -0.05);
    assert!((pos[2] - (-0.025)).abs() < 1.0e-6);
    // 球-球重叠与零距离退化。
    let (dist, pos, normal) = sphere_sphere([0.0, 0.0, 0.0], 0.1, [0.15, 0.0, 0.0], 0.1);
    assert!((dist - (-0.05)).abs() < 1.0e-6);
    assert_eq!(normal, [1.0, 0.0, 0.0]);
    assert!((pos[0] - 0.075).abs() < 1.0e-6);
    let (dist, _, normal) = sphere_sphere([1.0, 2.0, 3.0], 0.1, [1.0, 2.0, 3.0], 0.1);
    assert!((dist - (-0.2)).abs() < 1.0e-6);
    assert_eq!(normal, [1.0, 0.0, 0.0]);
    // make_frame：第一行为单位法线，三行两两正交。
    let frame = make_frame([0.0, 0.6, 0.8]);
    assert!((frame[0] * frame[0] + frame[1] * frame[1] + frame[2] * frame[2] - 1.0).abs() < 1e-6);
    for row in 1..3 {
        let dot = frame[0] * frame[row * 3]
            + frame[1] * frame[row * 3 + 1]
            + frame[2] * frame[row * 3 + 2];
        assert!(dot.abs() < 1e-6);
    }
}

mod plane_box;
