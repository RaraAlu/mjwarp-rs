#![cfg(feature = "cuda-probe")]
//! U038 三维旋转/组合状态产品链门禁（free+ball+hinge+slide，四元数非零旋转）。
//! 产品测试只读静态冻结夹具，不启动上游作者流程。

use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
    KinematicModelInput, KinematicsParameter, KinematicsParameters, ParameterBatch,
};
use mjwarp_rs::physics::{
    GravityCompensationOptions, GravityCompensationParameters, GravityCompensationPlan,
    KinematicsPlan,
};
use mjwarp_rs::runtime::{
    FrozenWarpGravityArtifact, FrozenWarpGravityInput, FrozenWarpKinematicsInput, TransferSession,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

#[path = "t4_gravity_3d_product_chain/contract.rs"]
mod contract;
#[path = "t4_gravity_3d_product_chain/live.rs"]
mod live;

const PACKAGE: &str = include_str!("../fixtures/gravity-upstream-abi/three-d-package.json");
const PACKAGE_SHA256: &str = "476d648cf949414b6a0d19d5a1e832259640499dd33996c1322606ae65e19710";
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
    three_d_rotation_coverage: bool,
    candidate_kinematics_inputs: bool,
    authoring_only: bool,
    product_python_dependency: bool,
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
        .map(|v| {
            let original = v.as_f64().unwrap();
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
        .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
        .collect()
}
fn flatten(rows: &[Vec<f32>]) -> Vec<f32> {
    rows.iter().flatten().copied().collect()
}
fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|v| v.to_bits()).collect()
}
fn hex_bits(values: &[u32]) -> Vec<String> {
    values.iter().map(|v| format!("{v:08x}")).collect()
}
fn bit_digest(values: &[u32]) -> String {
    let bytes = values
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect::<Vec<_>>();
    digest(&bytes)
}
fn write_json(path: PathBuf, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn dof_layout(input: &FrozenWarpKinematicsInput) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let per_joint = [6usize, 3, 1, 1];
    let mut body_last = std::collections::BTreeMap::new();
    let (mut body, mut jnt, mut parent) = (Vec::new(), Vec::new(), Vec::new());
    for (j, (&b, &kind)) in input.jnt_bodyid.iter().zip(&input.jnt_type).enumerate() {
        let count = per_joint[kind as usize];
        let mut parent_body = input.body_parentid[b as usize];
        while parent_body != 0 && input.body_jntnum[parent_body as usize] == 0 {
            parent_body = input.body_parentid[parent_body as usize];
        }
        let mut p = if input.body_jntnum[parent_body as usize] == 0 {
            -1
        } else {
            *body_last.get(&parent_body).unwrap_or(&-1)
        };
        for _ in 0..count {
            body.push(b);
            jnt.push(j as i32);
            parent.push(p);
            p = body.len() as i32 - 1;
        }
        body_last.insert(b, body.len() as i32 - 1);
    }
    (body, jnt, parent)
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
        || package["scope"] != "gravity-live-product-3d-qpos-chain"
        || package["upstream_revision"] != REVISION
        || package["warp_version"] != WARP_VERSION
        || package["product_sources"]
            != serde_json::json!({
                "physics_core": {"path": "src/physics/mod.rs", "sha256": digest(PHYSICS.replace("\r\n", "\n").as_bytes())},
                "resident": {"path": "src/physics/resident.rs", "sha256": digest(RESIDENT.replace("\r\n", "\n").as_bytes())},
                "gravity_compensation": {"path": "src/physics/gravity_compensation.rs", "sha256": digest(&fs::read(root().join("src/physics/gravity_compensation.rs")).unwrap())}
            })
        || package["dimensions"]
            != serde_json::json!({"worlds": 7, "nbody": 5, "njnt": 4, "nq": 13, "nv": 11, "branches": 2})
        || package["parameter_periods"]
            != serde_json::json!({"gravity": 2, "body_mass": 3, "body_gravcomp": 5})
        || package["expected"]["rigid_values"] != 833
        || package["expected"]["subtree_com_values"] != 105
        || package["expected"]["cdof_values"] != 462
        || package["expected"]["gravity_values"] != 77
        || package["expected"]["total_values"] != 1477
        || tolerance.to_bits() != TOLERANCE_BITS
        || package["boundaries"]["all_qpos_nonzero"] != true
        || package["boundaries"]["nonzero_rotation_coverage"] != true
        || package["boundaries"]["three_d_rotation_coverage"] != true
        || package["boundaries"]["combined_joint_types"] != true
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
        "frozen-upstream-generated-3d-qpos-kinematics-com-cdof"
    );
    assert_eq!(receipt["upstream_revision"], REVISION);
    assert_eq!(receipt["warp_version"], WARP_VERSION);
    assert_eq!(receipt["generation_attempts"], 2);
    for (name, expected) in receipt["files"].as_object().unwrap() {
        let path = std::path::Path::new(name);
        assert!(
            !path.is_absolute()
                && !path
                    .components()
                    .any(|p| matches!(p, std::path::Component::ParentDir))
        );
        let bytes = fs::read(directory.join(path)).unwrap();
        assert_eq!(
            digest(&bytes),
            expected.as_str().unwrap(),
            "receipt file {name}"
        );
    }
    let module = &receipt["module_artifact"];
    let module_directory = directory.join(module["directory"].as_str().unwrap());
    for (name, key) in [
        ("smooth.cu", "cu_sha256"),
        ("smooth.meta", "meta_sha256"),
        ("smooth.ptx", "ptx_sha256"),
    ] {
        assert_eq!(
            digest(&fs::read(module_directory.join(name)).unwrap()),
            module[key]
        );
    }
    let input_bytes = fs::read(directory.join("input.json")).unwrap();
    assert_eq!(digest(&input_bytes), package["kinematics"]["input_sha256"]);
    let output_bytes = fs::read(directory.join("output.json")).unwrap();
    assert_eq!(
        digest(&output_bytes),
        package["kinematics"]["output_sha256"]
    );
    let raw_bytes = fs::read(directory.join("raw-bits.json")).unwrap();
    assert_eq!(digest(&raw_bytes), package["kinematics"]["raw_bits_sha256"]);
    let input: FrozenWarpKinematicsInput = serde_json::from_slice(&input_bytes).unwrap();
    let output: UpstreamOutput = serde_json::from_slice(&output_bytes).unwrap();
    let raw: Value = serde_json::from_slice(&raw_bytes).unwrap();
    assert_eq!(
        raw["provenance"],
        "direct_upstream_device_readback_before_finite_validation"
    );
    let raw_fields = raw["fields"].as_array().unwrap();
    assert_eq!(raw_fields.len(), 8);
    for ((name, values), field) in [
        ("xpos", &output.xpos),
        ("xquat_wxyz", &output.xquat_wxyz),
        ("xipos", &output.xipos),
        ("xmat", &output.xmat),
        ("xanchor", &output.xanchor),
        ("xaxis", &output.xaxis),
        ("subtree_com", &output.subtree_com),
        ("cdof", &output.cdof),
    ]
    .into_iter()
    .zip(raw_fields)
    {
        let field_bits = bits(values);
        assert_eq!(field["field"], name);
        assert_eq!(field["count"], field_bits.len());
        assert_eq!(field["little_endian_sha256"], bit_digest(&field_bits));
        let archived: Vec<&str> = field["bits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            archived,
            hex_bits(&field_bits)
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
        );
    }
    (input, output)
}
