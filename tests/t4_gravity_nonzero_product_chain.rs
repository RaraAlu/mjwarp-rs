#![cfg(feature = "cuda-probe")]
//! 非零位形产品刚体、质心与重力串联门禁。
//! 产品测试不启动上游作者流程。

#[path = "t4_gravity_nonzero_product_chain/contract.rs"]
mod contract;
#[path = "t4_gravity_nonzero_product_chain/live.rs"]
mod live;

use mjwarp_rs::model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput};
use mjwarp_rs::runtime::{FrozenWarpGravityInput, FrozenWarpKinematicsInput};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const PACKAGE: &str =
    include_str!("../fixtures/gravity-upstream-abi/nonzero-product-chain-package.json");
const PACKAGE_SHA256: &str = "394267081315156edc1088648b1915fb4e94067befb8bdcc53f9aad05ca4a70e";
const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
const TOLERANCE_BITS: u32 = 0x360637bd;
const PHYSICS: &str = include!("../docs/development-evidence/t4-resident-anchors/physics.rs");
const RESIDENT: &str = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/evidence/t4-g03-retest/resident-source-anchor.rs"
));

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamOutput {
    schema: u32,
    evidence_kind: String,
    upstream_revision: String,
    warp_version: String,
    input_sha256: String,
    worlds: usize,
    nbody: usize,
    njnt: usize,
    nq: usize,
    nv: usize,
    xpos: Vec<f32>,
    xquat_wxyz: Vec<f32>,
    xipos: Vec<f32>,
    xmat: Vec<f32>,
    xanchor: Vec<f32>,
    xaxis: Vec<f32>,
    subtree_com: Vec<f32>,
    cdof: Vec<f32>,
    all_qpos_nonzero: bool,
    nonzero_rotation_coverage: bool,
    candidate_kinematics_inputs: bool,
}
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

fn validate_package(package: &Value) -> Result<(), &'static str> {
    let tolerance = package["expected"]["absolute_tolerance"]
        .as_f64()
        .ok_or("tolerance")? as f32;
    if package["schema"] != 1
        || package["scope"] != "gravity-live-product-nonzero-qpos-chain"
        || package["upstream_revision"] != REVISION
        || package["warp_version"] != WARP_VERSION
        || package["product_sources"]
            != serde_json::json!({
                "physics_core": {
                    "path": "src/physics/mod.rs",
                    "sha256": "9d10f056869d6612fa89ef2bb59c9e3a590b5fb04a1386d94d780b5c4a3ae9d1"
                },
                "resident": {
                    "path": "src/physics/resident.rs",
                    "sha256": "3c4de13bdce432a18fc0e76e3162e8cb2b433a7e9919efccfea4a8d4ebb588c6"
                },
                "gravity_compensation": {
                    "path": "src/physics/gravity_compensation.rs",
                    "sha256": "6135a4f252c13deae162ec186aaa7845cadfe87ff1ea31cac42ad824b0fd69d3"
                }
            })
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
        || package["expected"]["rigid_values"] != 833
        || package["expected"]["subtree_com_values"] != 105
        || package["expected"]["cdof_values"] != 168
        || package["expected"]["gravity_values"] != 28
        || package["expected"]["total_values"] != 1134
        || tolerance.to_bits() != TOLERANCE_BITS
        || package["boundaries"]["all_qpos_nonzero"] != true
        || package["boundaries"]["nonzero_rotation_coverage"] != true
        || package["boundaries"]["same_model_and_qpos"] != true
        || package["boundaries"]["same_resident_source"] != true
        || package["boundaries"]["candidate_kinematics_inputs"] != false
        || package["boundaries"]["product_python_dependency"] != false
        || package["boundaries"]["full_state_space_accepted"] != false
        || package["boundaries"]["np03_complete"] != false
        || package["boundaries"]["full_regression_run"] != false
        || package["boundaries"]["performance_accepted"] != false
    {
        return Err("package identity");
    }
    Ok(())
}

fn package_path(package: &Value, pointer: &str) -> PathBuf {
    root().join(package.pointer(pointer).unwrap().as_str().unwrap())
}

fn checked_bytes(package: &Value, path_pointer: &str, hash_pointer: &str) -> Vec<u8> {
    let bytes = fs::read(package_path(package, path_pointer)).unwrap();
    assert_eq!(
        digest(&bytes),
        package.pointer(hash_pointer).unwrap().as_str().unwrap()
    );
    bytes
}

fn checked_case_file(directory: &std::path::Path, name: &str, expected: &str) -> Vec<u8> {
    let path = std::path::Path::new(name);
    assert!(!path.is_absolute());
    assert!(!path.components().any(|part| matches!(
        part,
        std::path::Component::ParentDir | std::path::Component::RootDir
    )));
    let bytes = fs::read(directory.join(path)).unwrap();
    assert_eq!(digest(&bytes), expected);
    bytes
}

fn output_fields(output: &UpstreamOutput) -> [(&'static str, &[f32]); 8] {
    [
        ("xpos", &output.xpos),
        ("xquat_wxyz", &output.xquat_wxyz),
        ("xipos", &output.xipos),
        ("xmat", &output.xmat),
        ("xanchor", &output.xanchor),
        ("xaxis", &output.xaxis),
        ("subtree_com", &output.subtree_com),
        ("cdof", &output.cdof),
    ]
}

fn load_case(package: &Value) -> (FrozenWarpKinematicsInput, UpstreamOutput) {
    let directory = package_path(package, "/kinematics/directory");
    let receipt_bytes = fs::read(directory.join("receipt.json")).unwrap();
    assert_eq!(
        digest(&receipt_bytes),
        package["kinematics"]["receipt_sha256"]
    );
    let receipt: Value = serde_json::from_slice(&receipt_bytes).unwrap();
    assert_eq!(receipt["schema"], 1);
    assert_eq!(
        receipt["evidence_kind"],
        "frozen-upstream-generated-nonzero-qpos-kinematics-com-cdof"
    );
    assert_eq!(receipt["upstream_revision"], REVISION);
    assert_eq!(receipt["warp_version"], WARP_VERSION);
    for (name, expected) in receipt["files"].as_object().unwrap() {
        checked_case_file(&directory, name, expected.as_str().unwrap());
    }
    let module = &receipt["module_artifact"];
    assert_eq!(module["directory"], "../independent-kinematics");
    let module_directory = directory.join("../independent-kinematics");
    for (name, key) in [
        ("smooth.cu", "cu_sha256"),
        ("smooth.meta", "meta_sha256"),
        ("smooth.ptx", "ptx_sha256"),
    ] {
        let bytes = fs::read(module_directory.join(name)).unwrap();
        assert_eq!(digest(&bytes), module[key]);
    }
    let input_bytes = checked_case_file(
        &directory,
        "input.json",
        package["kinematics"]["input_sha256"].as_str().unwrap(),
    );
    let output_bytes = checked_case_file(
        &directory,
        "output.json",
        package["kinematics"]["output_sha256"].as_str().unwrap(),
    );
    let raw_bytes = checked_case_file(
        &directory,
        "raw-bits.json",
        package["kinematics"]["raw_bits_sha256"].as_str().unwrap(),
    );
    let input: FrozenWarpKinematicsInput = serde_json::from_slice(&input_bytes).unwrap();
    let output: UpstreamOutput = serde_json::from_slice(&output_bytes).unwrap();
    let raw: Value = serde_json::from_slice(&raw_bytes).unwrap();
    assert_eq!(raw["schema"], 1);
    assert_eq!(
        raw["provenance"],
        "direct_upstream_device_readback_before_finite_validation"
    );
    assert_eq!(raw["upstream_revision"], REVISION);
    let raw_fields = raw["fields"].as_array().unwrap();
    assert_eq!(raw_fields.len(), 8);
    for ((name, values), field) in output_fields(&output).into_iter().zip(raw_fields) {
        let field_bits = bits(values);
        assert_eq!(field["field"], name);
        assert_eq!(field["count"], field_bits.len());
        assert_eq!(field["little_endian_sha256"], bit_digest(&field_bits));
        let archived = field["bits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(archived, hex_bits(&field_bits));
    }
    (input, output)
}

fn model_package(package: &Value) -> Value {
    let bytes = checked_bytes(package, "/model_package/path", "/model_package/sha256");
    serde_json::from_slice(&bytes).unwrap()
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

fn gravity_input(
    model_package: &Value,
    input: &FrozenWarpKinematicsInput,
    output: &UpstreamOutput,
) -> FrozenWarpGravityInput {
    let model = &model_package["model"];
    FrozenWarpGravityInput {
        worlds: input.worlds,
        nbody: input.nbody,
        nv: input.nv,
        opt_gravity: f32_values(&model["opt_gravity"]),
        body_parentid: i32_values(&model["body_parentid"]),
        body_rootid: i32_values(&model["body_rootid"]),
        body_mass: f32_values(&model["body_mass"]),
        body_gravcomp: f32_values(&model["body_gravcomp"]),
        dof_bodyid: i32_values(&model["dof_bodyid"]),
        body_isdofancestor: i32_values(&model["body_isdofancestor"]),
        xipos: output.xipos.clone(),
        subtree_com: output.subtree_com.clone(),
        cdof: output.cdof.clone(),
    }
}

fn compare(actual: &[u32], expected: &[u32], tolerance: f32) -> (usize, usize, f32) {
    assert_eq!(actual.len(), expected.len());
    let mut bit_differences = 0usize;
    let mut outside_tolerance = 0usize;
    let mut max_absolute = 0.0f32;
    for (&actual, &expected) in actual.iter().zip(expected) {
        let actual_value = f32::from_bits(actual);
        let expected_value = f32::from_bits(expected);
        let absolute = (actual_value - expected_value).abs();
        assert!(actual_value.is_finite());
        assert!(expected_value.is_finite());
        outside_tolerance += usize::from(absolute > tolerance);
        bit_differences += usize::from(actual != expected);
        max_absolute = max_absolute.max(absolute);
    }
    (bit_differences, outside_tolerance, max_absolute)
}
