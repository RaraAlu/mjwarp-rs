//! 零位形产品质心与重力串联门禁。
//! 上游结果只读取冻结产物。

#![cfg(feature = "cuda-probe")]

use mjwarp_rs::{
    model::{
        AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
        KinematicModelInput, KinematicsParameter, KinematicsParameters, ParameterBatch,
    },
    physics::{
        GravityCompensationOptions, GravityCompensationParameters, GravityCompensationPlan,
        KinematicsPlan,
    },
    runtime::{FrozenWarpKinematicsArtifact, FrozenWarpKinematicsInput, TransferSession},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const PACKAGE: &str = include_str!("../fixtures/gravity-upstream-abi/product-chain-package.json");
const BASE_PACKAGE: &str = include_str!("../fixtures/gravity-upstream-abi/qpos-package.json");
const PACKAGE_SHA256: &str = "733cb3868c3ae6510645943d457e072e0acea87e91f40fc895cb9b84202fe55e";
const BASE_PACKAGE_SHA256: &str =
    "c0163db1a230b2575ccb42cb60c6e5b07de07e77580748285047d9397de57df0";
const KINEMATICS_RECEIPT_SHA256: &str =
    "954e0640c7420f6b3927e6a83401e3a95c4044dcafab224a7e1c85d240fc858d";
const UPSTREAM_GRAVITY_RUN_SHA256: &str =
    "6586ea1aac919ba09b664cae1fea2a2f725467dd40a587d6e7e26e59291140c8";
const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
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

fn flatten(rows: &[Vec<f32>]) -> Vec<f32> {
    rows.iter().flatten().copied().collect()
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

fn load_package() -> Value {
    assert_eq!(digest(PACKAGE.as_bytes()), PACKAGE_SHA256);
    let package: Value = serde_json::from_str(PACKAGE).unwrap();
    validate_package(&package).unwrap();
    package
}

fn load_base_package() -> Value {
    assert_eq!(digest(BASE_PACKAGE.as_bytes()), BASE_PACKAGE_SHA256);
    serde_json::from_str(BASE_PACKAGE).unwrap()
}

fn validate_package(package: &Value) -> Result<(), &'static str> {
    let tolerance = package["expected"]["absolute_tolerance"]
        .as_f64()
        .ok_or("tolerance")? as f32;
    if package["schema"] != 1
        || package["scope"] != "gravity-live-product-com-gravity-zero-qpos"
        || package["upstream_revision"] != REVISION
        || package["warp_version"] != WARP_VERSION
        || package["base_package"]["sha256"] != BASE_PACKAGE_SHA256
        || package["upstream_kinematics"]["receipt_sha256"] != KINEMATICS_RECEIPT_SHA256
        || package["upstream_gravity"]["sha256"] != UPSTREAM_GRAVITY_RUN_SHA256
        || package["dimensions"]
            != serde_json::json!({
                "worlds": 7,
                "nbody": 5,
                "njnt": 4,
                "nq": 4,
                "nv": 4,
                "branches": 2
            })
        || package["parameter_periods"]
            != serde_json::json!({"gravity": 2, "body_mass": 3, "body_gravcomp": 5})
        || package["expected"]["subtree_com_values"] != 105
        || package["expected"]["cdof_values"] != 168
        || package["expected"]["gravity_values"] != 28
        || package["expected"]["total_values"] != 301
        || tolerance.to_bits() != TOLERANCE_BITS
        || package["boundaries"]["frozen_upstream_relaunched"] != false
        || package["boundaries"]["authoring_rerun"] != false
        || package["boundaries"]["live_product_rigid"] != true
        || package["boundaries"]["live_product_com"] != true
        || package["boundaries"]["live_product_gravity"] != true
        || package["boundaries"]["same_resident_source"] != true
        || package["boundaries"]["zero_qpos_only"] != true
        || package["boundaries"]["nonzero_rotation_coverage"] != false
        || package["boundaries"]["full_state_space_accepted"] != false
        || package["boundaries"]["np03_complete"] != false
        || package["boundaries"]["product_python_dependency"] != false
        || package["boundaries"]["full_regression_run"] != false
        || package["boundaries"]["performance_accepted"] != false
    {
        return Err("package identity");
    }
    Ok(())
}

fn path(package: &Value, pointer: &str) -> PathBuf {
    root().join(package.pointer(pointer).unwrap().as_str().unwrap())
}

fn load_upstream_gravity(package: &Value) -> Vec<u32> {
    let file = path(package, "/upstream_gravity/path");
    let bytes = fs::read(file).unwrap();
    assert_eq!(digest(&bytes), UPSTREAM_GRAVITY_RUN_SHA256);
    let run: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(run["upstream_revision"], REVISION);
    assert_eq!(run["warp_version"], WARP_VERSION);
    assert_eq!(run["upstream_field_values"], 1106);
    assert_eq!(run["upstream_field_bit_differences"], 0);
    assert_eq!(run["gravity_output_bits"].as_array().unwrap().len(), 28);
    run["gravity_output_bits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::from_str_radix(value.as_str().unwrap(), 16).unwrap())
        .collect()
}

fn inertial_model(input: &FrozenWarpKinematicsInput) -> InertialModelInput {
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

fn compare(actual: &[u32], expected: &[u32], tolerance: f32) -> (usize, f32) {
    assert_eq!(actual.len(), expected.len());
    let mut bit_differences = 0usize;
    let mut max_absolute = 0.0f32;
    for (&actual, &expected) in actual.iter().zip(expected) {
        let actual_value = f32::from_bits(actual);
        let expected_value = f32::from_bits(expected);
        let absolute = (actual_value - expected_value).abs();
        assert!(actual_value.is_finite());
        assert!(expected_value.is_finite());
        assert!(absolute <= tolerance);
        bit_differences += usize::from(actual != expected);
        max_absolute = max_absolute.max(absolute);
    }
    (bit_differences, max_absolute)
}

#[test]
fn frozen_live_product_chain_contract_is_self_consistent() {
    let package = load_package();
    let base = load_base_package();
    assert_eq!(
        digest(&fs::read(path(&package, "/base_package/path")).unwrap()),
        BASE_PACKAGE_SHA256
    );
    let directory = path(&package, "/upstream_kinematics/directory");
    assert_eq!(
        digest(&fs::read(directory.join("receipt.json")).unwrap()),
        KINEMATICS_RECEIPT_SHA256
    );
    let artifact = FrozenWarpKinematicsArtifact::load(directory).unwrap();
    assert_eq!(artifact.input().body_mass.len(), 3);
    assert_eq!(artifact.input().qpos.len(), 7);
    assert!(
        flatten(&artifact.input().qpos)
            .iter()
            .all(|value| value.to_bits() == 0)
    );
    assert_eq!(artifact.readback().subtree_com.len(), 105);
    assert_eq!(artifact.readback().cdof.len(), 168);
    assert_eq!(load_upstream_gravity(&package).len(), 28);
    assert_eq!(base["model"]["opt_gravity"].as_array().unwrap().len(), 6);
    assert_eq!(base["model"]["body_gravcomp"].as_array().unwrap().len(), 25);

    for pointer in [
        "/expected/absolute_tolerance",
        "/parameter_periods/body_mass",
        "/boundaries/live_product_com",
        "/boundaries/nonzero_rotation_coverage",
    ] {
        let mut changed = package.clone();
        *changed.pointer_mut(pointer).unwrap() = match pointer {
            "/expected/absolute_tolerance" => Value::from(0.00002),
            "/parameter_periods/body_mass" => Value::from(4),
            "/boundaries/live_product_com" => Value::Bool(false),
            _ => Value::Bool(true),
        };
        assert!(validate_package(&changed).is_err(), "accepted {pointer}");
    }
}

#[cfg(windows)]
#[test]
#[ignore = "requires an NVIDIA GPU and pinned NVRTC path"]
fn live_product_com_and_gravity_match_frozen_upstream() {
    let package = load_package();
    let base = load_base_package();
    let evidence_directory =
        PathBuf::from(std::env::var_os("MJWARP_GRAVITY_PRODUCT_CHAIN_EVIDENCE_DIR").unwrap());
    assert!(
        !evidence_directory.exists(),
        "evidence directory already exists"
    );
    fs::create_dir_all(evidence_directory.parent().unwrap()).unwrap();
    fs::create_dir(&evidence_directory).unwrap();

    let artifact =
        FrozenWarpKinematicsArtifact::load(path(&package, "/upstream_kinematics/directory"))
            .unwrap();
    let upstream = artifact.readback();
    let upstream_subtree_bits = bits(&upstream.subtree_com);
    let upstream_cdof_bits = bits(&upstream.cdof);
    let upstream_gravity_bits = load_upstream_gravity(&package);
    let qpos = flatten(&artifact.input().qpos);
    let qpos_bits = bits(&qpos);
    let identity = serde_json::json!({
        "package_sha256": PACKAGE_SHA256,
        "base_package_sha256": BASE_PACKAGE_SHA256,
        "kinematics_receipt_sha256": KINEMATICS_RECEIPT_SHA256,
        "upstream_gravity_run_sha256": UPSTREAM_GRAVITY_RUN_SHA256,
        "dimensions": package["dimensions"],
        "parameter_periods": package["parameter_periods"],
        "qpos": {
            "count": qpos_bits.len(),
            "little_endian_sha256": bit_digest(&qpos_bits),
            "all_positive_zero": qpos_bits.iter().all(|&value| value == 0)
        },
        "branches": {
            "count": artifact.input().nbranch,
            "body_parentid": artifact.input().body_parentid,
            "body_rootid": artifact.input().body_rootid
        }
    });
    write_json(
        evidence_directory.join("upstream-reference-raw.json"),
        &serde_json::json!({
            "schema": 1,
            "provenance": "frozen_upstream_device_outputs_without_relaunch",
            "identity": &identity,
            "field_names": ["subtree_com", "cdof", "qfrc_gravcomp"],
            "field_lengths": [upstream_subtree_bits.len(), upstream_cdof_bits.len(), upstream_gravity_bits.len()],
            "total_values": upstream_subtree_bits.len() + upstream_cdof_bits.len() + upstream_gravity_bits.len(),
            "field_bits": [
                hex_bits(&upstream_subtree_bits),
                hex_bits(&upstream_cdof_bits),
                hex_bits(&upstream_gravity_bits)
            ]
        }),
    );

    let session = TransferSession::new(0).unwrap();
    let mut source_parameters = KinematicsParameters::default();
    source_parameters.set(
        KinematicsParameter::BodyMass,
        ParameterBatch::new(
            usize_value(&package["parameter_periods"]["body_mass"]),
            artifact.input().nbody,
            flatten(&artifact.input().body_mass),
        )
        .unwrap(),
    );
    let source_plan = KinematicsPlan::with_parameters(
        &session,
        AttachedModelInput::new(inertial_model(artifact.input()), AttachedFields::default())
            .unwrap(),
        source_parameters,
    )
    .unwrap();
    let mut source = source_plan.create_data(artifact.input().worlds).unwrap();
    source.write_qpos(&qpos).unwrap();
    source_plan.update(&mut source).unwrap();
    let snapshot = source.readback().unwrap();
    let mut product_subtree = Vec::new();
    let mut product_cdof = Vec::new();
    for world in 0..artifact.input().worlds {
        let com = snapshot.com().world(world).unwrap();
        product_subtree.extend_from_slice(com.subtree_com);
        product_cdof.extend_from_slice(com.cdof);
    }
    let product_subtree_bits = bits(&product_subtree);
    let product_cdof_bits = bits(&product_cdof);
    write_json(
        evidence_directory.join("product-com-raw.json"),
        &serde_json::json!({
            "schema": 1,
            "provenance": "live_product_resident_com_before_upstream_comparison",
            "identity": &identity,
            "field_names": ["subtree_com", "cdof"],
            "field_lengths": [product_subtree_bits.len(), product_cdof_bits.len()],
            "total_values": product_subtree_bits.len() + product_cdof_bits.len(),
            "field_bits": [hex_bits(&product_subtree_bits), hex_bits(&product_cdof_bits)]
        }),
    );

    let model = &base["model"];
    let gravity_plan = GravityCompensationPlan::new(
        &source_plan,
        GravityCompensationParameters {
            gravity: ParameterBatch::new(
                usize_value(&package["parameter_periods"]["gravity"]),
                3,
                f32_values(&model["opt_gravity"]),
            )
            .unwrap(),
            body_gravcomp: ParameterBatch::new(
                usize_value(&package["parameter_periods"]["body_gravcomp"]),
                artifact.input().nbody,
                f32_values(&model["body_gravcomp"]),
            )
            .unwrap(),
            jnt_actgravcomp: vec![0; artifact.input().njnt],
        },
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let mut gravity_data = gravity_plan.create_data(artifact.input().worlds).unwrap();
    let gravity = gravity_plan
        .update(&source, &mut gravity_data)
        .unwrap()
        .snapshot()
        .unwrap();
    let mut product_gravity = Vec::new();
    for world in 0..artifact.input().worlds {
        product_gravity.extend_from_slice(gravity.world(world).unwrap());
    }
    let product_gravity_bits = bits(&product_gravity);
    write_json(
        evidence_directory.join("product-gravity-raw.json"),
        &serde_json::json!({
            "schema": 1,
            "provenance": "live_product_gravity_before_upstream_gravity_comparison",
            "identity": &identity,
            "field_names": ["qfrc_gravcomp"],
            "field_lengths": [product_gravity_bits.len()],
            "total_values": product_gravity_bits.len(),
            "field_bits": [hex_bits(&product_gravity_bits)]
        }),
    );

    assert_eq!(upstream_subtree_bits.len(), 105);
    assert_eq!(upstream_cdof_bits.len(), 168);
    assert_eq!(upstream_gravity_bits.len(), 28);
    assert_eq!(product_subtree_bits.len(), upstream_subtree_bits.len());
    assert_eq!(product_cdof_bits.len(), upstream_cdof_bits.len());
    assert_eq!(product_gravity_bits.len(), upstream_gravity_bits.len());
    let tolerance = package["expected"]["absolute_tolerance"].as_f64().unwrap() as f32;
    let (subtree_bit_differences, subtree_max_absolute) =
        compare(&product_subtree_bits, &upstream_subtree_bits, tolerance);
    let (cdof_bit_differences, cdof_max_absolute) =
        compare(&product_cdof_bits, &upstream_cdof_bits, tolerance);
    let (gravity_bit_differences, gravity_max_absolute) =
        compare(&product_gravity_bits, &upstream_gravity_bits, tolerance);
    let total_values =
        product_subtree_bits.len() + product_cdof_bits.len() + product_gravity_bits.len();
    assert_eq!(total_values, 301);

    write_json(
        evidence_directory.join("run.json"),
        &serde_json::json!({
            "schema": 1,
            "evidence_kind": "live-product-zero-qpos-com-gravity-against-frozen-upstream",
            "identity": &identity,
            "product_subtree_com_values": product_subtree_bits.len(),
            "product_subtree_com_bit_differences": subtree_bit_differences,
            "product_subtree_com_max_absolute": subtree_max_absolute,
            "product_cdof_values": product_cdof_bits.len(),
            "product_cdof_bit_differences": cdof_bit_differences,
            "product_cdof_max_absolute": cdof_max_absolute,
            "product_gravity_values": product_gravity_bits.len(),
            "product_gravity_bit_differences": gravity_bit_differences,
            "product_gravity_max_absolute": gravity_max_absolute,
            "total_compared_values": total_values,
            "absolute_tolerance": tolerance,
            "raw_archived_before_comparison": true,
            "same_resident_source": true,
            "frozen_upstream_relaunched": false,
            "authoring_rerun": false,
            "zero_qpos_only": true,
            "nonzero_rotation_coverage": false,
            "full_state_space_accepted": false,
            "np03_complete": false,
            "product_python_dependency": false,
            "full_regression_run": false,
            "performance_accepted": false
        }),
    );
}
