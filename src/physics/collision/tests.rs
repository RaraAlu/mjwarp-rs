//! G07@P5 宽相碰撞测试：冻结上游逐案候选、NXN/SAP 一致性、原生 f64 一级判定
//! 与输入校验面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/broadphase/broadphase-cases.json"
    ))
    .unwrap()
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

fn vec2_i32_list(value: &Value) -> Vec<[i32; 2]> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| i32_list(item).try_into().unwrap())
        .collect()
}

fn vec3_flat(value: &Value) -> Vec<[f32; 3]> {
    let flat = f32_list(value);
    let (rows, _) = flat.as_chunks::<3>();
    rows.to_vec()
}

fn vec9_flat(value: &Value) -> Vec<[f32; 9]> {
    let flat = f32_list(value);
    let (rows, _) = flat.as_chunks::<9>();
    rows.to_vec()
}

fn vec3_list(value: &Value) -> Vec<[f32; 3]> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| f32_list(item).try_into().unwrap())
        .collect()
}

struct OwnedModel {
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

impl OwnedModel {
    fn from_json(value: &Value) -> Self {
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

fn case_model(fixture: &Value, case: &Value) -> OwnedModel {
    let name = case["model"].as_str().unwrap();
    let mut model = OwnedModel::from_json(&fixture["models"][name]);
    if let Some(batches) = case["options"].get("dataid_batches") {
        model.geom_dataid_batches = batches.as_u64().unwrap() as usize;
        model.geom_dataid = i32_list(&case["options"]["geom_dataid"]);
    }
    model
}

fn case_state(case: &Value) -> (Vec<[f32; 3]>, Vec<[f32; 9]>) {
    let mut positions = Vec::new();
    let mut matrices = Vec::new();
    for world in case["worlds"].as_array().unwrap() {
        positions.extend(vec3_flat(&world["geom_xpos"]));
        matrices.extend(vec9_flat(&world["geom_xmat"]));
    }
    (positions, matrices)
}

fn case_options(case: &Value) -> BroadphaseOptions {
    BroadphaseOptions {
        filter: BroadphaseFilter(case["options"]["broadphase_filter"].as_u64().unwrap() as u32),
        capacity: case["options"]["naconmax"].as_u64().unwrap() as usize,
    }
}

fn run_case(fixture: &Value, case: &Value) -> BroadphaseOutput {
    let model = case_model(fixture, case);
    let (positions, matrices) = case_state(case);
    let state = BroadphaseState {
        nworld: case["nworld"].as_u64().unwrap() as usize,
        geom_xpos: &positions,
        geom_xmat: &matrices,
    };
    let options = case_options(case);
    match BroadphaseType::from_i32(case["options"]["broadphase"].as_u64().unwrap() as i32).unwrap()
    {
        BroadphaseType::Nxn => nxn_broadphase(&model.view(), &state, options).unwrap(),
        sap => sap_broadphase(&model.view(), &state, sap, options).unwrap(),
    }
}

fn canonical(output: &BroadphaseOutput) -> Vec<(i32, [i32; 2], [i32; 2])> {
    let mut rows: Vec<(i32, [i32; 2], [i32; 2])> = output
        .worldids
        .iter()
        .zip(output.pairs.iter())
        .zip(output.pairids.iter())
        .map(|((world, pair), pairid)| (*world, *pair, *pairid))
        .collect();
    rows.sort_unstable();
    rows
}

fn expected_canonical(case: &Value) -> Vec<(i32, [i32; 2], [i32; 2])> {
    let output = &case["output"];
    let pairs = vec2_i32_list(&output["pairs"]);
    let pairids = vec2_i32_list(&output["pairids"]);
    let worldids = i32_list(&output["worldids"]);
    let mut rows: Vec<(i32, [i32; 2], [i32; 2])> = worldids
        .into_iter()
        .zip(pairs)
        .zip(pairids)
        .map(|((world, pair), pairid)| (world, pair, pairid))
        .collect();
    rows.sort_unstable();
    rows
}

fn find_case<'a>(fixture: &'a Value, name: &str) -> &'a Value {
    fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("missing case {name}"))
}

#[test]
fn broadphase_candidates_match_frozen_upstream_reference() {
    let fixture = fixture();
    assert_eq!(
        fixture["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g07-p5"
    );
    assert_eq!(fixture["enum"]["MJ_MAXVAL"].as_f64().unwrap(), 1.0e10);
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 47);
    let mut capacity_cases = 0;
    for case in cases {
        let output = run_case(&fixture, case);
        let expected = &case["output"];
        assert_eq!(
            output.ncollision,
            expected["ncollision"].as_u64().unwrap() as usize,
            "case {} ncollision",
            case["name"]
        );
        if let Some(reference) = case.get("full_reference") {
            capacity_cases += 1;
            let capacity = case["options"]["naconmax"].as_u64().unwrap() as usize;
            assert_eq!(output.pairs.len(), capacity);
            assert_eq!(output.pairids.len(), capacity);
            assert_eq!(output.worldids.len(), capacity);
            let full = find_case(&fixture, reference.as_str().unwrap());
            let full_output = run_case(&fixture, full);
            let allowed = canonical(&full_output);
            for (world, pair, pairid) in canonical(&output) {
                assert!(
                    allowed.contains(&(world, pair, pairid)),
                    "case {} wrote pair outside full candidate set",
                    case["name"]
                );
            }
        } else {
            assert_eq!(
                canonical(&output),
                expected_canonical(case),
                "case {} candidate set",
                case["name"]
            );
        }
    }
    assert_eq!(capacity_cases, 2);
}

#[test]
fn nxn_and_sap_agree_on_candidate_sets() {
    let fixture = fixture();
    for key in ["rest", "near", "cluster", "order"] {
        let nxn = run_case(&fixture, find_case(&fixture, &format!("mixed-{key}-nxn")));
        let tile = run_case(
            &fixture,
            find_case(&fixture, &format!("mixed-{key}-sap-tile")),
        );
        let segmented = run_case(
            &fixture,
            find_case(&fixture, &format!("mixed-{key}-sap-segmented")),
        );
        assert_eq!(
            canonical(&nxn),
            canonical(&tile),
            "mixed {key} nxn vs sap-tile"
        );
        assert_eq!(
            canonical(&nxn),
            canonical(&segmented),
            "mixed {key} nxn vs sap-segmented"
        );
    }
    let all = run_case(&fixture, find_case(&fixture, "filters-near-all"));
    for mode in ["sap-tile", "sap-segmented"] {
        let sap = run_case(
            &fixture,
            find_case(&fixture, &format!("filters-near-{mode}")),
        );
        assert_eq!(
            canonical(&all),
            canonical(&sap),
            "filters near nxn vs {mode}"
        );
    }
    let mesh_nxn = run_case(&fixture, find_case(&fixture, "mesh-missing-nxn"));
    let mesh_sap = run_case(&fixture, find_case(&fixture, "mesh-missing-sap-segmented"));
    assert_eq!(canonical(&mesh_nxn), canonical(&mesh_sap), "mesh missing");
}

#[test]
fn candidates_cover_native_contacts() {
    let fixture = fixture();
    let mut checked_cases = 0;
    let mut native_contacts = 0;
    let mut margin_cases = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let Some(native) = case.get("native") else {
            continue;
        };
        let output = run_case(&fixture, case);
        let mut known: Vec<(i32, [i32; 2])> = output
            .worldids
            .iter()
            .zip(output.pairs.iter())
            .map(|(world, pair)| (*world, [pair[0].min(pair[1]), pair[0].max(pair[1])]))
            .collect();
        known.sort_unstable();
        let native_ncon = i32_list(&native["native_ncon"]);
        let mut total = 0;
        for (world, row) in native["native_pairs"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            for pair in row.as_array().unwrap() {
                let pair = i32_list(pair);
                let key = (world as i32, [pair[0].min(pair[1]), pair[0].max(pair[1])]);
                assert!(
                    known.contains(&key),
                    "case {} world {world}: native contact {pair:?} missing from candidates",
                    case["name"]
                );
                total += 1;
            }
        }
        assert_eq!(total, native_ncon.iter().sum::<i32>() as usize);
        checked_cases += 1;
        native_contacts += total;
        if case["name"].as_str().unwrap().starts_with("margin-") {
            margin_cases += 1;
            assert_eq!(
                output.ncollision, native_ncon[0] as usize,
                "case {} margin candidate count",
                case["name"]
            );
        }
    }
    assert!(checked_cases >= 20, "native cross check coverage");
    assert!(native_contacts >= 20, "native contact coverage");
    assert_eq!(margin_cases, 6);
}

#[test]
fn rejects_invalid_shapes() {
    let fixture = fixture();
    let case = find_case(&fixture, "mixed-cluster-nxn");
    let model = case_model(&fixture, case);
    let (positions, matrices) = case_state(case);
    let state = BroadphaseState {
        nworld: 1,
        geom_xpos: &positions[1..],
        geom_xmat: &matrices,
    };
    let error = nxn_broadphase(&model.view(), &state, case_options(case)).unwrap_err();
    assert!(matches!(
        error,
        BroadphaseError::InvalidLength {
            field: "geom_xpos",
            ..
        }
    ));

    let mut broken = case_model(&fixture, case);
    broken.full_pairid.pop();
    let state = BroadphaseState {
        nworld: 1,
        geom_xpos: &positions,
        geom_xmat: &matrices,
    };
    let error = nxn_broadphase(&broken.view(), &state, case_options(case)).unwrap_err();
    assert!(matches!(
        error,
        BroadphaseError::InvalidLength {
            field: "full_pairid",
            ..
        }
    ));

    let mut broken = case_model(&fixture, case);
    broken.geom_dataid_batches = 0;
    let error = sap_broadphase(
        &broken.view(),
        &state,
        BroadphaseType::SapTile,
        case_options(case),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        BroadphaseError::InvalidBatch {
            field: "geom_dataid",
            ..
        }
    ));

    let error = sap_broadphase(
        &model.view(),
        &state,
        BroadphaseType::Nxn,
        case_options(case),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        BroadphaseError::UnsupportedBroadphase { value: 0 }
    ));
}

#[test]
fn upper_tri_index_matches_upstream_formula() {
    let expected = [
        (3, 0, 1, 0),
        (3, 0, 2, 1),
        (3, 1, 2, 2),
        (3, 2, 0, 1),
        (7, 0, 6, 5),
        (7, 1, 2, 6),
        (7, 3, 6, 17),
    ];
    for (ngeom, geom1, geom2, index) in expected {
        assert_eq!(upper_tri_index(ngeom, geom1, geom2), index);
    }
}
