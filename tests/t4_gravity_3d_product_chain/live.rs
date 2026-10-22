use super::*;

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
    let (dof_bodyid, dof_jntid, dof_parentid) = dof_layout(input);
    InertialModelInput::new(
        kinematics,
        InertialFields {
            body_ipos: flatten(&input.body_ipos),
            body_iquat: flatten(&input.body_iquat_wxyz),
            body_mass: input.body_mass[0].clone(),
            body_inertia: vec![0.0; input.nbody * 3],
            dof_bodyid,
            dof_jntid,
            dof_parentid,
            dof_armature: vec![0.0; input.nv],
            dof_damping: vec![0.0; input.nv],
        },
    )
    .unwrap()
}

fn gravity_input(
    package: &Value,
    input: &FrozenWarpKinematicsInput,
    output: &UpstreamOutput,
) -> FrozenWarpGravityInput {
    let model = &package["model"];
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
    let (mut bit_differences, mut outside_tolerance, mut max_absolute) = (0usize, 0usize, 0.0f32);
    for (&a, &e) in actual.iter().zip(expected) {
        let av = f32::from_bits(a);
        let ev = f32::from_bits(e);
        assert!(av.is_finite() && ev.is_finite());
        let absolute = (av - ev).abs();
        outside_tolerance += usize::from(absolute > tolerance);
        bit_differences += usize::from(a != e);
        max_absolute = max_absolute.max(absolute);
    }
    (bit_differences, outside_tolerance, max_absolute)
}

#[cfg(windows)]
#[test]
#[ignore = "requires an NVIDIA GPU, frozen Warp artifacts and pinned NVRTC path"]
fn live_three_d_product_chain_matches_frozen_upstream() {
    let package = load_package();
    let (input, upstream) = load_case(&package);
    let model_bytes = fs::read(package_path(&package, "/model_package/path")).unwrap();
    let model_package: Value = serde_json::from_slice(&model_bytes).unwrap();
    let evidence_directory =
        PathBuf::from(std::env::var_os("MJWARP_GRAVITY_3D_EVIDENCE_DIR").unwrap());
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
        "qpos": {"count": qpos_bits.len(), "little_endian_sha256": bit_digest(&qpos_bits), "all_nonzero": qpos_bits.iter().all(|v| v & 0x7fff_ffff != 0)},
    });
    let gravity_artifact =
        FrozenWarpGravityArtifact::load(package_path(&package, "/gravity/directory"), 0).unwrap();
    let upstream_gravity = gravity_artifact
        .launch_diagnostic(&gravity_input(&model_package, &input, &upstream))
        .unwrap();
    let upstream_gravity_bits = upstream_gravity.bits().to_vec();
    write_json(
        evidence_directory.join("upstream-gravity-raw.json"),
        &serde_json::json!({
            "schema": 1, "provenance": "live_frozen_upstream_gravity_before_finite_validation",
            "identity": &identity, "field_names": ["qfrc_gravcomp"], "field_lengths": [upstream_gravity_bits.len()],
            "total_values": upstream_gravity_bits.len(), "field_bits": [hex_bits(&upstream_gravity_bits)]
        }),
    );

    let session = TransferSession::new(0).unwrap();
    let mut parameters = KinematicsParameters::default();
    parameters.set(
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
        parameters,
    )
    .unwrap();
    let mut source = source_plan.create_data(input.worlds).unwrap();
    source.write_qpos(&qpos).unwrap();
    source_plan.update(&mut source).unwrap();
    let snapshot = source.readback().unwrap();
    let mut product_rigid = vec![Vec::<f32>::new(); 6];
    let (mut product_subtree, mut product_cdof) = (Vec::new(), Vec::new());
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
    let product_rigid_bits: Vec<Vec<u32>> = product_rigid.iter().map(|f| bits(f)).collect();
    let (product_subtree_bits, product_cdof_bits) = (bits(&product_subtree), bits(&product_cdof));
    let mut product_source_input = gravity_input(&model_package, &input, &upstream);
    product_source_input.xipos = product_rigid[3].clone();
    product_source_input.subtree_com = product_subtree.clone();
    product_source_input.cdof = product_cdof.clone();
    let upstream_product_source = gravity_artifact
        .launch_diagnostic(&product_source_input)
        .unwrap();
    let upstream_product_source_bits = upstream_product_source.bits().to_vec();
    upstream_product_source.into_finite().unwrap();

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
            "schema": 1, "provenance": "live_product_gravity_before_upstream_gravity_comparison",
            "identity": &identity, "field_names": ["qfrc_gravcomp"], "field_lengths": [product_gravity_bits.len()],
            "total_values": product_gravity_bits.len(), "field_bits": [hex_bits(&product_gravity_bits)]
        }),
    );

    let tolerance = package["expected"]["absolute_tolerance"].as_f64().unwrap() as f32;
    let rigid_names = ["xpos", "xquat_wxyz", "xmat", "xipos", "xanchor", "xaxis"];
    let upstream_rigid = [
        &upstream.xpos,
        &upstream.xquat_wxyz,
        &upstream.xmat,
        &upstream.xipos,
        &upstream.xanchor,
        &upstream.xaxis,
    ];
    let mut field_results = Vec::new();
    let (mut total_values, mut total_bit_differences, mut total_outside, mut max_absolute) =
        (0usize, 0usize, 0usize, 0.0f32);
    for ((name, actual), expected) in rigid_names
        .iter()
        .zip(&product_rigid_bits)
        .zip(upstream_rigid)
    {
        let (bit_differences, outside, field_max) = compare(actual, &bits(expected), tolerance);
        total_values += actual.len();
        total_bit_differences += bit_differences;
        total_outside += outside;
        max_absolute = max_absolute.max(field_max);
        field_results.push(serde_json::json!({"field": name, "values": actual.len(), "bit_differences": bit_differences, "outside_tolerance": outside, "max_absolute": field_max}));
    }
    let upstream_gravity_values = upstream_gravity.into_finite().unwrap();
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
            upstream_gravity_values.as_slice(),
        ),
    ] {
        let (bit_differences, outside, field_max) = compare(actual, &bits(expected), tolerance);
        total_values += actual.len();
        total_bit_differences += bit_differences;
        total_outside += outside;
        max_absolute = max_absolute.max(field_max);
        field_results.push(serde_json::json!({"field": name, "values": actual.len(), "bit_differences": bit_differences, "outside_tolerance": outside, "max_absolute": field_max}));
    }
    assert_eq!(total_values, 1477);
    let (source_bits, source_outside, source_max) = compare(
        &upstream_product_source_bits,
        &upstream_gravity_bits,
        tolerance,
    );
    let (operator_bits, operator_outside, operator_max) = compare(
        &product_gravity_bits,
        &upstream_product_source_bits,
        tolerance,
    );
    write_json(
        evidence_directory.join("run.json"),
        &serde_json::json!({
            "schema": 1,
            "evidence_kind": "live-product-3d-qpos-chain-against-frozen-upstream",
            "identity": identity,
            "field_results": field_results,
            "total_compared_values": total_values,
            "total_bit_differences": total_bit_differences,
            "total_outside_tolerance": total_outside,
            "max_absolute": max_absolute,
            "absolute_tolerance": tolerance,
            "gravity_source_propagation": {"values": upstream_product_source_bits.len(), "bit_differences": source_bits, "outside_tolerance": source_outside, "max_absolute": source_max},
            "gravity_operator": {"values": product_gravity_bits.len(), "bit_differences": operator_bits, "outside_tolerance": operator_outside, "max_absolute": operator_max},
            "three_d_rotation_coverage": true,
            "combined_joint_types": true,
            "full_state_space_accepted": false,
            "np03_complete": false
        }),
    );
    assert_eq!(
        total_outside, 0,
        "product chain exceeded frozen tolerance; inspect run.json"
    );
}
