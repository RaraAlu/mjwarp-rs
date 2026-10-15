#![cfg(feature = "cuda-probe")]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn sha256(path: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(fs::read(path).unwrap());
    format!("{:x}", digest.finalize())
}

fn object_strings(value: &Value) -> BTreeMap<String, String> {
    value
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| (key.clone(), value.as_str().unwrap().to_owned()))
        .collect()
}

fn assert_f32_input(value: &Value) {
    assert_eq!(value["dtype"], "float32");
    let elements: usize = value["shape"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_u64().unwrap() as usize)
        .product();
    let values = value["values"].as_array().unwrap();
    let bits = value["bits"].as_array().unwrap();
    assert_eq!(values.len(), elements);
    assert_eq!(bits.len(), elements);
    for (number, expected_bits) in values.iter().zip(bits) {
        let number = number.as_f64().unwrap() as f32;
        assert!(number.is_finite());
        assert_eq!(number.to_bits(), expected_bits.as_u64().unwrap() as u32);
    }
}

fn assert_field_shape(value: &Value) {
    let elements: usize = value["shape"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_u64().unwrap() as usize)
        .product();
    assert_eq!(value["values"].as_array().unwrap().len(), elements);
}

#[test]
fn reference_contract_is_self_consistent() {
    let fixture = root().join("fixtures/mjwarp-complex-physics");
    let manifest = read_json(&fixture.join("manifest.json"));
    let reference = read_json(&fixture.join("reference.json"));
    assert_eq!(manifest["upstream_revision"], REVISION);
    assert_eq!(reference["upstream_revision"], REVISION);
    assert_eq!(manifest["run_id"], "run-5");
    assert_eq!(
        manifest["reference_sha256"],
        sha256(&fixture.join("reference.json"))
    );
    assert_eq!(
        manifest["authoring_script_sha256"],
        sha256(&fixture.join("reference.py"))
    );
    assert_eq!(
        manifest["source_hashes"],
        manifest["installed_source_hashes"]
    );
    assert_eq!(
        manifest["coverage_contract"],
        reference["coverage_contract"]
    );

    let contract = reference["coverage_contract"].as_object().unwrap();
    assert_eq!(
        contract.keys().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "G07".to_owned(),
            "G08".to_owned(),
            "G09".to_owned(),
            "G10".to_owned(),
            "G11".to_owned(),
            "G22".to_owned(),
        ])
    );
    assert_eq!(reference["collision_cases"].as_array().unwrap().len(), 12);
    assert_eq!(reference["flex_cases"].as_array().unwrap().len(), 4);
    assert_eq!(reference["probes"].as_array().unwrap().len(), 1);

    let collision = reference["collision_cases"].as_array().unwrap();
    let names = collision
        .iter()
        .map(|case| case["name"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    for expected in [
        "plane-sphere-nxn",
        "plane-sphere-sap-tile",
        "plane-sphere-sap-segmented",
        "plane-sphere-sphere-only-precheck",
        "sphere-sphere",
        "convex-mesh",
        "box-box-multiccd",
        "degenerate-mesh",
        "hfield-box",
        "sdf-cylinder",
        "flex-geom",
        "flex-self",
    ] {
        assert!(names.contains(expected), "missing case {expected}");
    }
    let box_case = collision
        .iter()
        .find(|case| case["name"] == "box-box-multiccd")
        .unwrap();
    for count in box_case["boundary_evidence"]["contacts_per_world"]
        .as_object()
        .unwrap()
        .values()
    {
        assert!(count.as_u64().unwrap() >= 2);
    }
    let self_case = collision
        .iter()
        .find(|case| case["name"] == "flex-self")
        .unwrap();
    assert!(
        self_case["boundary_evidence"]["same_flex_contacts"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(
        !self_case["boundary_evidence"]["distinct_element_pairs"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    for case in collision
        .iter()
        .chain(reference["flex_cases"].as_array().unwrap())
    {
        if let Some(qpos) = case["input"].get("qpos") {
            assert_f32_input(qpos);
        }
        if let Some(qvel) = case["input"].get("qvel") {
            assert_f32_input(qvel);
        }
    }
    for case in reference["flex_cases"].as_array().unwrap() {
        for field in case["output"]["fields"].as_object().unwrap().values() {
            assert_field_shape(field);
        }
    }
    assert_eq!(reference["probes"][0]["name"], "volume-gradient-outside");
    assert_f32_input(&reference["probes"][0]["input"]["points"]);
}

#[test]
fn tracked_inputs_match_manifest() {
    let manifest = read_json(&root().join("fixtures/mjwarp-complex-physics/manifest.json"));
    for (path, expected) in object_strings(&manifest["inputs"]) {
        assert_eq!(sha256(&root().join(path)), expected);
    }
}

#[test]
fn generated_cache_matches_manifest() {
    let manifest = read_json(&root().join("fixtures/mjwarp-complex-physics/manifest.json"));
    let base = root().join("docs/evidence/d04-complex-physics/authoring-runs/run-5");
    let cache = object_strings(&manifest["cache_hashes"]);
    assert_eq!(cache.len(), 78);
    assert!(cache.keys().any(|path| path.ends_with(".cu")));
    assert!(cache.keys().any(|path| path.ends_with(".ptx")));
    for (path, expected) in cache {
        assert_eq!(sha256(&base.join(&path)), expected, "{path}");
    }
}
