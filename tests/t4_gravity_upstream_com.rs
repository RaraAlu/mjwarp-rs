//! 冻结上游质心中间量与重力核串联门禁。
//! 产品测试只读取静态产物。

#![cfg(feature = "cuda-probe")]

use mjwarp_rs::runtime::{
    FrozenWarpComArtifact, FrozenWarpGravityArtifact, FrozenWarpGravityInput,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const PACKAGE: &str = include_str!("../fixtures/gravity-upstream-abi/independent-package.json");
const PACKAGE_SHA256: &str = "08bca47e27f7e6f767d06905810f9f9e0b8916d5a223736c4cf8dfa1f9e87699";
const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
const EXPECTED_SHA256: &str = "19841745d04a41a627386e3c6b2c5a399eaddc7cc1a5728f7f44f55872de4409";
const TOLERANCE_BITS: u32 = 0x360637bd;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn usize_value(value: &Value) -> usize {
    usize::try_from(value.as_u64().unwrap()).unwrap()
}

fn f32_values(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| {
            let original = value.as_f64().unwrap();
            let narrowed = original as f32;
            assert!(narrowed.is_finite());
            assert_eq!(f64::from(narrowed), original);
            narrowed
        })
        .collect()
}

fn i32_values(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| i32::try_from(value.as_i64().unwrap()).unwrap())
        .collect()
}

fn load_package() -> Value {
    assert_eq!(digest(PACKAGE.as_bytes()), PACKAGE_SHA256);
    let package: Value = serde_json::from_str(PACKAGE).unwrap();
    validate_package(&package).unwrap();
    package
}

fn validate_package(package: &Value) -> Result<(), &'static str> {
    let tolerance = package["expected"]["absolute_tolerance"]
        .as_f64()
        .ok_or("tolerance")? as f32;
    if package["schema"] != 1
        || package["scope"] != "gravity-upstream-independent-com-cdof-chain"
        || package["upstream_revision"] != REVISION
        || package["warp_version"] != WARP_VERSION
        || package["dimensions"] != serde_json::json!({"worlds": 7, "nbody": 5, "nv": 4})
        || package["artifacts"]["com_directory"]
            != "fixtures/gravity-upstream-abi/independent-inputs"
        || package["artifacts"]["gravity_directory"] != "fixtures/gravity-upstream-abi/generated"
        || package["expected"]["path"]
            != "docs/evidence/gravity-compensation/multi-body-contract.json"
        || package["expected"]["sha256"] != EXPECTED_SHA256
        || package["expected"]["kind"] != "analytic_certificate_not_upstream_device_output"
        || tolerance.to_bits() != TOLERANCE_BITS
        || package["boundaries"]["candidate_com_cdof_inputs"] != false
        || package["boundaries"]["upstream_com_cdof_device_output"] != true
        || package["boundaries"]["static_kinematics_inputs"] != true
        || package["boundaries"]["full_qpos_chain_independent"] != false
        || package["boundaries"]["full_chain_independent"] != false
        || package["boundaries"]["product_python_dependency"] != false
        || package["boundaries"]["full_regression_run"] != false
    {
        return Err("package identity");
    }
    Ok(())
}

fn artifact_path(package: &Value, field: &str) -> PathBuf {
    root().join(package["artifacts"][field].as_str().unwrap())
}

fn analytic_expected(package: &Value) -> Vec<f32> {
    let path = root().join(package["expected"]["path"].as_str().unwrap());
    let bytes = fs::read(path).unwrap();
    assert_eq!(digest(&bytes), package["expected"]["sha256"]);
    let contract: Value = serde_json::from_slice(&bytes).unwrap();
    contract["expected"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(f32_values)
        .collect()
}

fn gravity_input(package: &Value, com: &FrozenWarpComArtifact) -> FrozenWarpGravityInput {
    let dimensions = &package["dimensions"];
    let worlds = usize_value(&dimensions["worlds"]);
    let nbody = usize_value(&dimensions["nbody"]);
    let nv = usize_value(&dimensions["nv"]);
    assert_eq!((worlds, nbody, nv), (7, 5, 4));
    assert_eq!(com.input().worlds, worlds);
    assert_eq!(com.input().nbody, nbody);
    assert_eq!(com.input().nv, nv);

    let model = &package["model"];
    let body_mass = f32_values(&model["body_mass"]);
    let independent_mass = com
        .input()
        .body_mass
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        body_mass
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        independent_mass
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        i32_values(&model["body_parentid"]),
        com.input().body_parentid
    );
    assert_eq!(i32_values(&model["body_rootid"]), com.input().body_rootid);

    let upstream = com.readback().into_finite().unwrap();
    FrozenWarpGravityInput {
        worlds,
        nbody,
        nv,
        opt_gravity: f32_values(&model["opt_gravity"]),
        body_parentid: i32_values(&model["body_parentid"]),
        body_rootid: i32_values(&model["body_rootid"]),
        body_mass,
        body_gravcomp: f32_values(&model["body_gravcomp"]),
        dof_bodyid: i32_values(&model["dof_bodyid"]),
        body_isdofancestor: i32_values(&model["body_isdofancestor"]),
        xipos: com.input().xipos_row.repeat(worlds),
        subtree_com: upstream.subtree_com,
        cdof: upstream.cdof,
    }
}

#[test]
fn frozen_upstream_com_cdof_artifact_is_self_consistent() {
    let package = load_package();
    let com = FrozenWarpComArtifact::load(artifact_path(&package, "com_directory")).unwrap();
    assert_eq!(com.receipt().generation_attempts, 2);
    assert!(!com.receipt().candidate_intermediate_inputs);
    assert!(com.receipt().static_kinematics_inputs);
    assert!(!com.receipt().full_qpos_chain_independent);
    let raw = com.readback();
    assert_eq!(raw.subtree_com().len(), 7 * 5 * 3);
    assert_eq!(raw.subtree_com_bits().len(), 7 * 5 * 3);
    assert_eq!(raw.cdof().len(), 7 * 4 * 6);
    assert_eq!(raw.cdof_bits().len(), 7 * 4 * 6);
    assert!(raw.into_finite().is_ok());
    let input = gravity_input(&package, &com);
    assert_eq!(input.subtree_com.len(), 105);
    assert_eq!(input.cdof.len(), 168);

    for pointer in [
        "/expected/sha256",
        "/expected/absolute_tolerance",
        "/boundaries/candidate_com_cdof_inputs",
    ] {
        let mut changed = package.clone();
        *changed.pointer_mut(pointer).unwrap() = match pointer {
            "/expected/sha256" => Value::String("wrong".into()),
            "/expected/absolute_tolerance" => Value::from(0.00002),
            _ => Value::Bool(true),
        };
        assert!(validate_package(&changed).is_err(), "accepted {pointer}");
    }
}

#[cfg(windows)]
#[test]
#[ignore = "requires an SM89 GPU and both frozen generated Warp 1.15 artifacts"]
fn frozen_upstream_com_cdof_feed_generated_gravity() {
    let package = load_package();
    let com = FrozenWarpComArtifact::load(artifact_path(&package, "com_directory")).unwrap();
    let gravity =
        FrozenWarpGravityArtifact::load(artifact_path(&package, "gravity_directory"), 0).unwrap();
    let input = gravity_input(&package, &com);
    let upstream = com.readback();
    let readback = gravity.launch_diagnostic(&input).unwrap();
    let output_bits = readback.bits().to_vec();
    let evidence_directory =
        std::env::var_os("MJWARP_GRAVITY_UPSTREAM_COM_EVIDENCE_DIR").map(PathBuf::from);
    if let Some(directory) = &evidence_directory {
        fs::create_dir_all(directory).unwrap();
        let raw = serde_json::json!({
            "schema": 1,
            "provenance": "direct_generated_gravity_readback_before_finite_validation",
            "upstream_revision": REVISION,
            "package_sha256": PACKAGE_SHA256,
            "com_input_sha256": com.receipt().input_sha256,
            "com_output_sha256": com.receipt().output_sha256,
            "com_ptx_sha256": com.receipt().ptx_sha256,
            "gravity_ptx_sha256": gravity.receipt().ptx_sha256,
            "gravity_output_bits": output_bits
                .iter()
                .map(|value| format!("{value:08x}"))
                .collect::<Vec<_>>(),
            "count": output_bits.len()
        });
        fs::write(
            directory.join("raw.json"),
            serde_json::to_vec_pretty(&raw).unwrap(),
        )
        .unwrap();
    }
    let actual = readback.into_finite().unwrap();
    let expected = analytic_expected(&package);
    let tolerance = package["expected"]["absolute_tolerance"].as_f64().unwrap() as f32;
    let max_absolute = actual
        .iter()
        .zip(&expected)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0_f32, f32::max);
    let bit_differences = actual
        .iter()
        .zip(&expected)
        .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
        .count();

    if let Some(directory) = &evidence_directory {
        let report = serde_json::json!({
            "schema": 1,
            "evidence_kind": "frozen-upstream-com-cdof-to-generated-gravity",
            "upstream_revision": REVISION,
            "warp_version": WARP_VERSION,
            "package_sha256": PACKAGE_SHA256,
            "com_receipt_sha256": digest(&fs::read(artifact_path(&package, "com_directory").join("receipt.json")).unwrap()),
            "gravity_receipt_sha256": digest(&fs::read(artifact_path(&package, "gravity_directory").join("receipt.json")).unwrap()),
            "com_input_sha256": com.receipt().input_sha256,
            "com_output_sha256": com.receipt().output_sha256,
            "com_ptx_sha256": com.receipt().ptx_sha256,
            "gravity_ptx_sha256": gravity.receipt().ptx_sha256,
            "subtree_com_bits": upstream
                .subtree_com_bits()
                .iter()
                .map(|value| format!("{value:08x}"))
                .collect::<Vec<_>>(),
            "cdof_bits": upstream
                .cdof_bits()
                .iter()
                .map(|value| format!("{value:08x}"))
                .collect::<Vec<_>>(),
            "gravity_output_bits": output_bits
                .iter()
                .map(|value| format!("{value:08x}"))
                .collect::<Vec<_>>(),
            "gravity_output": actual,
            "expected": expected,
            "tolerance": tolerance,
            "max_absolute": max_absolute,
            "bit_differences": bit_differences,
            "candidate_com_cdof_inputs": false,
            "upstream_com_cdof_device_output": true,
            "static_kinematics_inputs": true,
            "full_qpos_chain_independent": false,
            "full_chain_independent": false,
            "product_python_dependency": false
        });
        fs::write(
            directory.join("run.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(actual.len(), expected.len());
    assert!(max_absolute <= tolerance);
}
