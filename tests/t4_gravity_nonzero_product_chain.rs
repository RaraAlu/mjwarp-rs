//! 非零位形产品刚体、质心与重力串联门禁。
//! 产品测试不启动上游作者流程。

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
    runtime::{
        FrozenWarpGravityArtifact, FrozenWarpGravityInput, FrozenWarpKinematicsInput,
        TransferSession,
    },
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const PACKAGE: &str =
    include_str!("../fixtures/gravity-upstream-abi/nonzero-product-chain-package.json");
const PACKAGE_SHA256: &str = "9154df31fa852a0df23ac3fb31fc0cfbbcbdb5a678aeb42437b3853f184569ac";
const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
const TOLERANCE_BITS: u32 = 0x360637bd;

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
                    "sha256": "e9ec17dbc364a6e32038872f0a6bd1507d12bcbb04059b90f89e7d9f6235019e"
                },
                "resident": {
                    "path": "src/physics/resident.rs",
                    "sha256": "6389af92c39b687cdf3cd01fa3855d651bf264d6df7e2ff52fd8f230950718eb"
                },
                "gravity_compensation": {
                    "path": "src/physics/gravity_compensation.rs",
                    "sha256": "245cf69bb6f4a8502e75b11e9a3eb98b0d9180c4ae7895cc4298181c2a6715fb"
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

#[test]
fn frozen_nonzero_product_chain_contract_is_self_consistent() {
    let package = load_package();
    let (input, output) = load_case(&package);
    let model_package = model_package(&package);
    for source in package["product_sources"].as_object().unwrap().values() {
        let bytes = fs::read(root().join(source["path"].as_str().unwrap())).unwrap();
        assert_eq!(digest(&bytes), source["sha256"]);
    }
    assert_eq!(input.worlds, 7);
    assert_eq!(input.nbranch, 2);
    assert_eq!(flatten(&input.qpos).len(), 28);
    assert!(flatten(&input.qpos).iter().all(|value| *value != 0.0));
    assert_eq!(output.schema, 1);
    assert_eq!(
        output.evidence_kind,
        "frozen-upstream-generated-nonzero-qpos-kinematics-com-cdof"
    );
    assert_eq!(output.upstream_revision, REVISION);
    assert_eq!(output.warp_version, WARP_VERSION);
    assert_eq!(output.input_sha256, package["kinematics"]["input_sha256"]);
    assert_eq!(
        [
            output.worlds,
            output.nbody,
            output.njnt,
            output.nq,
            output.nv
        ],
        [7, 5, 4, 4, 4]
    );
    assert!(output.all_qpos_nonzero);
    assert!(output.nonzero_rotation_coverage);
    assert!(!output.candidate_kinematics_inputs);
    assert_eq!(output.xpos.len(), 105);
    assert_eq!(output.xquat_wxyz.len(), 140);
    assert_eq!(output.xipos.len(), 105);
    assert_eq!(output.xmat.len(), 315);
    assert_eq!(output.xanchor.len(), 84);
    assert_eq!(output.xaxis.len(), 84);
    assert_eq!(output.subtree_com.len(), 105);
    assert_eq!(output.cdof.len(), 168);
    let model = &model_package["model"];
    assert_eq!(input.body_parentid, i32_values(&model["body_parentid"]));
    assert_eq!(input.body_rootid, i32_values(&model["body_rootid"]));
    assert_eq!(flatten(&input.body_mass), f32_values(&model["body_mass"]));
    assert_eq!(input.jnt_bodyid, i32_values(&model["dof_bodyid"]));
    let gravity_receipt =
        fs::read(package_path(&package, "/gravity/directory").join("receipt.json")).unwrap();
    assert_eq!(
        digest(&gravity_receipt),
        package["gravity"]["receipt_sha256"]
    );

    for pointer in [
        "/expected/absolute_tolerance",
        "/expected/total_values",
        "/boundaries/all_qpos_nonzero",
        "/boundaries/full_state_space_accepted",
    ] {
        let mut changed = package.clone();
        *changed.pointer_mut(pointer).unwrap() = match pointer {
            "/expected/absolute_tolerance" => Value::from(0.00002),
            "/expected/total_values" => Value::from(1133),
            "/boundaries/all_qpos_nonzero" => Value::Bool(false),
            _ => Value::Bool(true),
        };
        assert!(validate_package(&changed).is_err(), "accepted {pointer}");
    }
}

#[cfg(windows)]
#[test]
#[ignore = "requires an NVIDIA GPU, frozen Warp artifacts and pinned NVRTC path"]
fn live_nonzero_product_chain_matches_frozen_upstream() {
    let package = load_package();
    let (input, upstream) = load_case(&package);
    let model_package = model_package(&package);
    let evidence_directory =
        PathBuf::from(std::env::var_os("MJWARP_GRAVITY_NONZERO_CHAIN_EVIDENCE_DIR").unwrap());
    assert!(
        !evidence_directory.exists(),
        "evidence directory already exists"
    );
    fs::create_dir_all(evidence_directory.parent().unwrap()).unwrap();
    fs::create_dir(&evidence_directory).unwrap();

    let qpos = flatten(&input.qpos);
    let qpos_bits = bits(&qpos);
    let identity = serde_json::json!({
        "package_sha256": PACKAGE_SHA256,
        "kinematics_receipt_sha256": package["kinematics"]["receipt_sha256"],
        "gravity_receipt_sha256": package["gravity"]["receipt_sha256"],
        "dimensions": package["dimensions"],
        "parameter_periods": package["parameter_periods"],
        "qpos": {
            "count": qpos_bits.len(),
            "little_endian_sha256": bit_digest(&qpos_bits),
            "all_nonzero": qpos_bits.iter().all(|value| value & 0x7fff_ffff != 0)
        },
        "branches": {
            "count": input.nbranch,
            "body_parentid": &input.body_parentid,
            "body_rootid": &input.body_rootid
        }
    });
    fs::copy(
        package_path(&package, "/kinematics/directory").join("raw-bits.json"),
        evidence_directory.join("upstream-kinematics-raw.json"),
    )
    .unwrap();

    let gravity_artifact =
        FrozenWarpGravityArtifact::load(package_path(&package, "/gravity/directory"), 0).unwrap();
    let upstream_gravity_readback = gravity_artifact
        .launch_diagnostic(&gravity_input(&model_package, &input, &upstream))
        .unwrap();
    let upstream_gravity_bits = upstream_gravity_readback.bits().to_vec();
    write_json(
        evidence_directory.join("upstream-gravity-raw.json"),
        &serde_json::json!({
            "schema": 1,
            "provenance": "live_frozen_upstream_gravity_before_finite_validation",
            "identity": &identity,
            "field_names": ["qfrc_gravcomp"],
            "field_lengths": [upstream_gravity_bits.len()],
            "total_values": upstream_gravity_bits.len(),
            "field_bits": [hex_bits(&upstream_gravity_bits)]
        }),
    );

    let session = TransferSession::new(0).unwrap();
    let mut source_parameters = KinematicsParameters::default();
    source_parameters.set(
        KinematicsParameter::BodyMass,
        ParameterBatch::new(
            usize_value(&package["parameter_periods"]["body_mass"]),
            input.nbody,
            flatten(&input.body_mass),
        )
        .unwrap(),
    );
    let source_plan = KinematicsPlan::with_parameters(
        &session,
        AttachedModelInput::new(inertial_model(&input), AttachedFields::default()).unwrap(),
        source_parameters,
    )
    .unwrap();
    let mut source = source_plan.create_data(input.worlds).unwrap();
    source.write_qpos(&qpos).unwrap();
    source_plan.update(&mut source).unwrap();
    let snapshot = source.readback().unwrap();

    let mut product_rigid = vec![Vec::<f32>::new(); 6];
    let mut product_subtree = Vec::new();
    let mut product_cdof = Vec::new();
    for world in 0..input.worlds {
        let rigid = snapshot.rigid().world(world).unwrap();
        for (target, field) in product_rigid.iter_mut().zip([
            rigid.xpos,
            rigid.xquat,
            rigid.xmat,
            rigid.xipos,
            rigid.xanchor,
            rigid.xaxis,
        ]) {
            target.extend_from_slice(field);
        }
        let com = snapshot.com().world(world).unwrap();
        product_subtree.extend_from_slice(com.subtree_com);
        product_cdof.extend_from_slice(com.cdof);
    }
    let product_rigid_bits = product_rigid
        .iter()
        .map(|field| bits(field))
        .collect::<Vec<_>>();
    let product_subtree_bits = bits(&product_subtree);
    let product_cdof_bits = bits(&product_cdof);
    write_json(
        evidence_directory.join("product-kinematics-com-raw.json"),
        &serde_json::json!({
            "schema": 1,
            "provenance": "live_product_resident_kinematics_com_before_comparison",
            "identity": &identity,
            "field_names": ["xpos", "xquat_wxyz", "xmat", "xipos", "xanchor", "xaxis", "subtree_com", "cdof"],
            "field_lengths": [105, 140, 315, 105, 84, 84, 105, 168],
            "total_values": 1106,
            "field_bits": [
                hex_bits(&product_rigid_bits[0]),
                hex_bits(&product_rigid_bits[1]),
                hex_bits(&product_rigid_bits[2]),
                hex_bits(&product_rigid_bits[3]),
                hex_bits(&product_rigid_bits[4]),
                hex_bits(&product_rigid_bits[5]),
                hex_bits(&product_subtree_bits),
                hex_bits(&product_cdof_bits)
            ]
        }),
    );

    let mut product_source_gravity_input = gravity_input(&model_package, &input, &upstream);
    product_source_gravity_input.xipos = product_rigid[3].clone();
    product_source_gravity_input.subtree_com = product_subtree.clone();
    product_source_gravity_input.cdof = product_cdof.clone();
    let upstream_product_source_readback = gravity_artifact
        .launch_diagnostic(&product_source_gravity_input)
        .unwrap();
    let upstream_product_source_bits = upstream_product_source_readback.bits().to_vec();
    write_json(
        evidence_directory.join("upstream-gravity-product-source-raw.json"),
        &serde_json::json!({
            "schema": 1,
            "provenance": "live_frozen_upstream_gravity_with_product_source_before_finite_validation",
            "identity": &identity,
            "field_names": ["qfrc_gravcomp"],
            "field_lengths": [upstream_product_source_bits.len()],
            "total_values": upstream_product_source_bits.len(),
            "field_bits": [hex_bits(&upstream_product_source_bits)]
        }),
    );
    upstream_product_source_readback.into_finite().unwrap();

    let model = &model_package["model"];
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
                input.nbody,
                f32_values(&model["body_gravcomp"]),
            )
            .unwrap(),
            jnt_actgravcomp: vec![0; input.njnt],
        },
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let mut gravity_data = gravity_plan.create_data(input.worlds).unwrap();
    let product_gravity_snapshot = gravity_plan
        .update(&source, &mut gravity_data)
        .unwrap()
        .snapshot()
        .unwrap();
    let mut product_gravity = Vec::new();
    for world in 0..input.worlds {
        product_gravity.extend_from_slice(product_gravity_snapshot.world(world).unwrap());
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

    let upstream_gravity = upstream_gravity_readback.into_finite().unwrap();
    let tolerance = package["expected"]["absolute_tolerance"].as_f64().unwrap() as f32;
    let upstream_rigid = [
        &upstream.xpos,
        &upstream.xquat_wxyz,
        &upstream.xmat,
        &upstream.xipos,
        &upstream.xanchor,
        &upstream.xaxis,
    ];
    let rigid_names = ["xpos", "xquat_wxyz", "xmat", "xipos", "xanchor", "xaxis"];
    let mut field_results = Vec::new();
    let mut total_values = 0usize;
    let mut total_bit_differences = 0usize;
    let mut total_outside_tolerance = 0usize;
    let mut max_absolute = 0.0f32;
    for ((name, actual), expected) in rigid_names
        .iter()
        .zip(&product_rigid_bits)
        .zip(upstream_rigid)
    {
        let (bit_differences, outside_tolerance, field_maximum) =
            compare(actual, &bits(expected), tolerance);
        total_values += actual.len();
        total_bit_differences += bit_differences;
        total_outside_tolerance += outside_tolerance;
        max_absolute = max_absolute.max(field_maximum);
        field_results.push(serde_json::json!({
            "field": name,
            "values": actual.len(),
            "bit_differences": bit_differences,
            "outside_tolerance": outside_tolerance,
            "max_absolute": field_maximum
        }));
    }
    for (name, actual, expected) in [
        (
            "subtree_com",
            product_subtree_bits.as_slice(),
            upstream.subtree_com.as_slice(),
        ),
        (
            "cdof",
            product_cdof_bits.as_slice(),
            upstream.cdof.as_slice(),
        ),
        (
            "qfrc_gravcomp",
            product_gravity_bits.as_slice(),
            upstream_gravity.as_slice(),
        ),
    ] {
        let (bit_differences, outside_tolerance, field_maximum) =
            compare(actual, &bits(expected), tolerance);
        total_values += actual.len();
        total_bit_differences += bit_differences;
        total_outside_tolerance += outside_tolerance;
        max_absolute = max_absolute.max(field_maximum);
        field_results.push(serde_json::json!({
            "field": name,
            "values": actual.len(),
            "bit_differences": bit_differences,
            "outside_tolerance": outside_tolerance,
            "max_absolute": field_maximum
        }));
    }
    assert_eq!(total_values, 1134);
    let (source_bit_differences, source_outside_tolerance, source_max_absolute) = compare(
        &upstream_product_source_bits,
        &upstream_gravity_bits,
        tolerance,
    );
    let (operator_bit_differences, operator_outside_tolerance, operator_max_absolute) = compare(
        &product_gravity_bits,
        &upstream_product_source_bits,
        tolerance,
    );
    write_json(
        evidence_directory.join("run.json"),
        &serde_json::json!({
            "schema": 1,
            "evidence_kind": "live-product-nonzero-qpos-chain-against-frozen-upstream",
            "identity": identity,
            "field_results": field_results,
            "total_compared_values": total_values,
            "total_bit_differences": total_bit_differences,
            "total_outside_tolerance": total_outside_tolerance,
            "max_absolute": max_absolute,
            "absolute_tolerance": tolerance,
            "gravity_source_propagation": {
                "values": upstream_product_source_bits.len(),
                "bit_differences": source_bit_differences,
                "outside_tolerance": source_outside_tolerance,
                "max_absolute": source_max_absolute
            },
            "gravity_operator": {
                "values": product_gravity_bits.len(),
                "bit_differences": operator_bit_differences,
                "outside_tolerance": operator_outside_tolerance,
                "max_absolute": operator_max_absolute
            },
            "raw_archived_before_comparison": true,
            "same_model_and_qpos": true,
            "same_resident_source": true,
            "all_qpos_nonzero": true,
            "nonzero_rotation_coverage": true,
            "candidate_kinematics_inputs": false,
            "product_python_dependency": false,
            "full_state_space_accepted": false,
            "np03_complete": false,
            "full_regression_run": false,
            "performance_accepted": false
        }),
    );
    assert_eq!(
        total_outside_tolerance, 0,
        "product chain exceeded frozen tolerance; inspect run.json"
    );
}
