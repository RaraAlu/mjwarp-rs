//! 实机非零产品链与冻结上游比对。

use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, KinematicsParameter, KinematicsParameters, ParameterBatch,
};
use mjwarp_rs::physics::{
    GravityCompensationOptions, GravityCompensationParameters, GravityCompensationPlan,
    KinematicsPlan,
};
use mjwarp_rs::runtime::{FrozenWarpGravityArtifact, TransferSession};

use super::*;

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
