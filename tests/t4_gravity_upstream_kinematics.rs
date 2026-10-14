//! 冻结上游位形、质心与重力串联门禁。
//! 产品测试只读取静态产物。

#![cfg(feature = "cuda-probe")]

use mjwarp_rs::{
    model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput},
    physics::probe_kinematics,
    runtime::{
        FrozenWarpGravityArtifact, FrozenWarpGravityInput, FrozenWarpKinematicsArtifact,
        FrozenWarpKinematicsInput, FrozenWarpKinematicsReadback, TransferSession,
    },
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const PACKAGE: &str = include_str!("../fixtures/gravity-upstream-abi/qpos-package.json");
const PACKAGE_SHA256: &str = "c0163db1a230b2575ccb42cb60c6e5b07de07e77580748285047d9397de57df0";
const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
const KINEMATICS_RECEIPT_SHA256: &str =
    "954e0640c7420f6b3927e6a83401e3a95c4044dcafab224a7e1c85d240fc858d";
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

fn flatten(rows: &[Vec<f32>]) -> Vec<f32> {
    rows.iter().flatten().copied().collect()
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
        || package["scope"] != "gravity-upstream-independent-qpos-chain"
        || package["upstream_revision"] != REVISION
        || package["warp_version"] != WARP_VERSION
        || package["dimensions"]
            != serde_json::json!({"worlds": 7, "nbody": 5, "njnt": 4, "nq": 4, "nv": 4})
        || package["artifacts"]["kinematics_directory"]
            != "fixtures/gravity-upstream-abi/independent-kinematics"
        || package["artifacts"]["gravity_directory"] != "fixtures/gravity-upstream-abi/generated"
        || package["expected"]["sha256"] != EXPECTED_SHA256
        || package["expected"]["kind"] != "analytic_certificate_not_upstream_device_output"
        || tolerance.to_bits() != TOLERANCE_BITS
        || package["boundaries"]["candidate_kinematics_inputs"] != false
        || package["boundaries"]["upstream_qpos_kinematics_device_output"] != true
        || package["boundaries"]["upstream_com_cdof_device_output"] != true
        || package["boundaries"]["upstream_gravity_device_output"] != true
        || package["boundaries"]["product_kinematics_comparison"] != true
        || package["boundaries"]["zero_qpos_only"] != true
        || package["boundaries"]["nonzero_rotation_coverage"] != false
        || package["boundaries"]["fixed_state_chain_independent"] != true
        || package["boundaries"]["full_state_space_accepted"] != false
        || package["boundaries"]["np03_complete"] != false
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

fn product_model(input: &FrozenWarpKinematicsInput) -> InertialModelInput {
    let kinematics = KinematicModelInput::new(
        input.nv,
        KinematicFields {
            qpos0: flatten(&input.qpos0),
            body_parentid: input.body_parentid.clone(),
            body_jntadr: input.body_jntadr.clone(),
            body_jntnum: input.body_jntnum.clone(),
            body_pos: flatten(&input.body_pos),
            body_quat: flatten(&input.body_quat_wxyz),
            jnt_type: input.jnt_type.clone(),
            jnt_bodyid: input.jnt_bodyid.clone(),
            jnt_qposadr: input.jnt_qposadr.clone(),
            jnt_dofadr: input.jnt_dofadr.clone(),
            jnt_pos: flatten(&input.jnt_pos),
            jnt_axis: flatten(&input.jnt_axis),
        },
    )
    .unwrap();
    InertialModelInput::new(
        kinematics,
        InertialFields {
            body_ipos: flatten(&input.body_ipos),
            body_iquat: flatten(&input.body_iquat_wxyz),
            body_mass: input.body_mass[0].clone(),
            body_inertia: vec![0.0; input.nbody * 3],
            dof_bodyid: input.jnt_bodyid.clone(),
            dof_jntid: (0..input.njnt as i32).collect(),
            dof_parentid: vec![-1, 0, 1, -1],
            dof_armature: vec![0.0; input.nv],
            dof_damping: vec![0.0; input.nv],
        },
    )
    .unwrap()
}

fn gravity_input(
    package: &Value,
    kinematics: &FrozenWarpKinematicsReadback,
) -> FrozenWarpGravityInput {
    let dimensions = &package["dimensions"];
    let worlds = usize_value(&dimensions["worlds"]);
    let nbody = usize_value(&dimensions["nbody"]);
    let nv = usize_value(&dimensions["nv"]);
    let model = &package["model"];
    FrozenWarpGravityInput {
        worlds,
        nbody,
        nv,
        opt_gravity: f32_values(&model["opt_gravity"]),
        body_parentid: i32_values(&model["body_parentid"]),
        body_rootid: i32_values(&model["body_rootid"]),
        body_mass: f32_values(&model["body_mass"]),
        body_gravcomp: f32_values(&model["body_gravcomp"]),
        dof_bodyid: i32_values(&model["dof_bodyid"]),
        body_isdofancestor: i32_values(&model["body_isdofancestor"]),
        xipos: kinematics.xipos.clone(),
        subtree_com: kinematics.subtree_com.clone(),
        cdof: kinematics.cdof.clone(),
    }
}

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn hex_bits(values: &[u32]) -> Vec<String> {
    values.iter().map(|value| format!("{value:08x}")).collect()
}

fn bit_digest(values: &[u32]) -> String {
    let bytes = values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>();
    digest(&bytes)
}

fn write_json(path: PathBuf, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

#[test]
fn frozen_upstream_qpos_chain_artifact_is_self_consistent() {
    let package = load_package();
    let directory = artifact_path(&package, "kinematics_directory");
    assert_eq!(
        digest(&fs::read(directory.join("receipt.json")).unwrap()),
        KINEMATICS_RECEIPT_SHA256
    );
    let artifact = FrozenWarpKinematicsArtifact::load(directory).unwrap();
    assert_eq!(artifact.receipt().generation_attempts, 3);
    assert!(!artifact.receipt().first_attempt_readback_preserved);
    assert!(artifact.receipt().second_attempt_output_preserved);
    assert!(
        artifact
            .receipt()
            .third_attempt_prevalidation_readback_preserved
    );
    assert!(artifact.receipt().qpos_generated_kinematics);
    assert!(artifact.receipt().zero_qpos_only);
    assert!(!artifact.receipt().nonzero_rotation_coverage);
    assert!(!artifact.receipt().candidate_kinematics_inputs);
    let readback = artifact.readback();
    assert_eq!(readback.xpos.len(), 105);
    assert_eq!(readback.xquat_wxyz.len(), 140);
    assert_eq!(readback.xipos.len(), 105);
    assert_eq!(readback.xmat.len(), 315);
    assert_eq!(readback.xanchor.len(), 84);
    assert_eq!(readback.xaxis.len(), 84);
    assert_eq!(readback.subtree_com.len(), 105);
    assert_eq!(readback.cdof.len(), 168);
    assert_eq!(readback.bits().len(), 8);
    assert!(readback.into_finite().is_ok());

    for pointer in [
        "/expected/absolute_tolerance",
        "/boundaries/zero_qpos_only",
        "/boundaries/full_state_space_accepted",
    ] {
        let mut changed = package.clone();
        *changed.pointer_mut(pointer).unwrap() = match pointer {
            "/expected/absolute_tolerance" => Value::from(0.00002),
            "/boundaries/zero_qpos_only" => Value::Bool(false),
            _ => Value::Bool(true),
        };
        assert!(validate_package(&changed).is_err(), "accepted {pointer}");
    }
}

#[cfg(windows)]
#[test]
#[ignore = "requires an SM89 GPU and frozen Warp 1.15 artifacts"]
fn frozen_upstream_qpos_chain_matches_product_and_gravity() {
    let package = load_package();
    let kinematics_directory = artifact_path(&package, "kinematics_directory");
    let artifact = FrozenWarpKinematicsArtifact::load(&kinematics_directory).unwrap();
    let evidence_directory =
        std::env::var_os("MJWARP_GRAVITY_UPSTREAM_KINEMATICS_EVIDENCE_DIR").map(PathBuf::from);
    if let Some(directory) = &evidence_directory {
        assert!(!directory.exists(), "evidence directory already exists");
        fs::create_dir_all(directory.parent().unwrap()).unwrap();
        fs::create_dir(directory).unwrap();
    }
    let qpos_bits = bits(&flatten(&artifact.input().qpos));
    let branch_roots = artifact
        .input()
        .body_parentid
        .iter()
        .enumerate()
        .filter_map(|(body, &parent)| (body > 0 && parent == 0).then_some(body))
        .collect::<Vec<_>>();
    let identity = serde_json::json!({
        "package_sha256": PACKAGE_SHA256,
        "kinematics_receipt_sha256": KINEMATICS_RECEIPT_SHA256,
        "kinematics_input_sha256": artifact.receipt().input_sha256,
        "dimensions": {
            "worlds": artifact.input().worlds,
            "nbody": artifact.input().nbody,
            "njnt": artifact.input().njnt,
            "nq": artifact.input().nq,
            "nv": artifact.input().nv
        },
        "qpos": {
            "count": qpos_bits.len(),
            "little_endian_sha256": bit_digest(&qpos_bits),
            "all_positive_zero": qpos_bits.iter().all(|&value| value == 0)
        },
        "branches": {
            "count": branch_roots.len(),
            "dynamic_root_bodyids": branch_roots,
            "body_parentid": artifact.input().body_parentid
        }
    });

    let device_readback = artifact.launch_diagnostic(0).unwrap();
    if let Some(directory) = &evidence_directory {
        let field_lengths = device_readback
            .bits()
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>();
        let raw = serde_json::json!({
            "schema": 1,
            "provenance": "direct_rust_launched_upstream_readback_before_finite_validation",
            "upstream_revision": REVISION,
            "identity": identity,
            "field_names": ["xpos", "xquat_wxyz", "xipos", "xmat", "xanchor", "xaxis", "subtree_com", "cdof"],
            "field_lengths": field_lengths,
            "total_values": field_lengths.iter().sum::<usize>(),
            "field_bits": device_readback.bits().iter().map(|field| {
                hex_bits(field)
            }).collect::<Vec<_>>()
        });
        write_json(directory.join("upstream-raw.json"), &raw);
    }
    let frozen = artifact.readback();
    assert_eq!(device_readback.bits().len(), frozen.bits().len());
    let mut upstream_values = 0usize;
    let mut upstream_bit_differences = 0usize;
    for (actual, expected) in device_readback.bits().iter().zip(frozen.bits()) {
        assert_eq!(actual.len(), expected.len());
        upstream_values += actual.len();
        upstream_bit_differences += actual
            .iter()
            .zip(expected)
            .filter(|(actual, expected)| actual != expected)
            .count();
    }
    assert_eq!(upstream_values, 1106);
    assert_eq!(upstream_bit_differences, 0);
    let upstream = device_readback.into_finite().unwrap();

    let session = TransferSession::new(0).unwrap();
    let model = product_model(artifact.input());
    let product = probe_kinematics(
        &session,
        &model,
        artifact.input().worlds,
        &flatten(&artifact.input().qpos),
    )
    .unwrap();
    let mut product_field_bits = vec![Vec::<u32>::new(); 6];
    for world in 0..artifact.input().worlds {
        let actual = product.world(world).unwrap();
        for (target, field) in product_field_bits.iter_mut().zip([
            actual.xpos,
            actual.xquat,
            actual.xmat,
            actual.xipos,
            actual.xanchor,
            actual.xaxis,
        ]) {
            target.extend(bits(field));
        }
    }
    if let Some(directory) = &evidence_directory {
        let field_lengths = product_field_bits.iter().map(Vec::len).collect::<Vec<_>>();
        let raw = serde_json::json!({
            "schema": 1,
            "provenance": "live_product_kinematics_readback_before_comparison",
            "identity": identity,
            "field_names": ["xpos", "xquat_wxyz", "xmat", "xipos", "xanchor", "xaxis"],
            "field_lengths": field_lengths,
            "total_values": field_lengths.iter().sum::<usize>(),
            "field_bits": product_field_bits.iter().map(|field| hex_bits(field)).collect::<Vec<_>>()
        });
        write_json(directory.join("product-raw.json"), &raw);
    }
    let upstream_fields = [
        (&upstream.xpos, 3 * artifact.input().nbody),
        (&upstream.xquat_wxyz, 4 * artifact.input().nbody),
        (&upstream.xmat, 9 * artifact.input().nbody),
        (&upstream.xipos, 3 * artifact.input().nbody),
        (&upstream.xanchor, 3 * artifact.input().njnt),
        (&upstream.xaxis, 3 * artifact.input().njnt),
    ];
    let mut product_bit_differences = 0usize;
    let mut product_values = 0usize;
    for world in 0..artifact.input().worlds {
        let actual = product.world(world).unwrap();
        let product_fields = [
            actual.xpos,
            actual.xquat,
            actual.xmat,
            actual.xipos,
            actual.xanchor,
            actual.xaxis,
        ];
        for ((expected, width), actual) in upstream_fields.iter().zip(product_fields) {
            let start = world * width;
            let expected = &expected[start..start + width];
            assert_eq!(actual.len(), expected.len());
            product_bit_differences += actual
                .iter()
                .zip(expected)
                .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
                .count();
            product_values += width;
        }
    }
    assert_eq!(product_values, 833);
    assert_eq!(product_bit_differences, 0);

    let gravity =
        FrozenWarpGravityArtifact::load(artifact_path(&package, "gravity_directory"), 0).unwrap();
    let gravity_readback = gravity
        .launch_diagnostic(&gravity_input(&package, &upstream))
        .unwrap();
    let gravity_bits = gravity_readback.bits().to_vec();
    if let Some(directory) = &evidence_directory {
        let raw = serde_json::json!({
            "schema": 1,
            "provenance": "direct_rust_launched_upstream_gravity_readback_before_finite_validation",
            "identity": identity,
            "field_names": ["qfrc_gravcomp"],
            "field_lengths": [gravity_bits.len()],
            "total_values": gravity_bits.len(),
            "field_bits": [hex_bits(&gravity_bits)]
        });
        write_json(directory.join("gravity-raw.json"), &raw);
    }
    let actual = gravity_readback.into_finite().unwrap();
    let expected = analytic_expected(&package);
    let tolerance = package["expected"]["absolute_tolerance"].as_f64().unwrap() as f32;
    assert_eq!(actual.len(), expected.len());
    assert_eq!(actual.len(), 28);
    let max_absolute = actual
        .iter()
        .zip(&expected)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0_f32, f32::max);
    let gravity_bit_differences = actual
        .iter()
        .zip(&expected)
        .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
        .count();
    assert!(max_absolute <= tolerance);

    if let Some(directory) = &evidence_directory {
        let report = serde_json::json!({
            "schema": 1,
            "evidence_kind": "frozen-upstream-qpos-to-generated-gravity",
            "upstream_revision": REVISION,
            "warp_version": WARP_VERSION,
            "package_sha256": PACKAGE_SHA256,
            "kinematics_receipt_sha256": KINEMATICS_RECEIPT_SHA256,
            "gravity_receipt_sha256": digest(&fs::read(artifact_path(&package, "gravity_directory").join("receipt.json")).unwrap()),
            "identity": identity,
            "upstream_field_values": upstream_values,
            "upstream_field_bit_differences": upstream_bit_differences,
            "product_compared_values": product_values,
            "product_bit_differences": product_bit_differences,
            "gravity_output_bits": hex_bits(&gravity_bits),
            "gravity_output": actual,
            "expected": expected,
            "tolerance": tolerance,
            "max_absolute": max_absolute,
            "gravity_bit_differences": gravity_bit_differences,
            "zero_qpos_only": true,
            "nonzero_rotation_coverage": false,
            "fixed_state_chain_independent": true,
            "full_state_space_accepted": false,
            "np03_complete": false,
            "product_python_dependency": false,
            "product_live_scope": "kinematics_only",
            "product_com_live": false,
            "product_gravity_live": false,
            "full_product_chain_live": false
        });
        write_json(directory.join("run.json"), &report);
    }
}
