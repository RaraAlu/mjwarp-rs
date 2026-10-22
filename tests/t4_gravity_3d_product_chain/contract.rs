use super::*;

#[test]
fn frozen_three_d_package_contract_is_self_consistent() {
    let package = load_package();
    let (input, output) = load_case(&package);
    assert_eq!(input.worlds, 7);
    assert_eq!(input.nbranch, 2);
    assert_eq!(flatten(&input.qpos).len(), 7 * 13);
    assert!(flatten(&input.qpos).iter().all(|v| *v != 0.0));
    assert_eq!(output.schema, 1);
    assert_eq!(output.upstream_revision, REVISION);
    assert_eq!(output.warp_version, WARP_VERSION);
    assert_eq!(
        output.evidence_kind,
        "frozen-upstream-generated-3d-qpos-kinematics-com-cdof"
    );
    assert_eq!(output.input_sha256, package["kinematics"]["input_sha256"]);
    assert_eq!(
        [
            output.worlds,
            output.nbody,
            output.njnt,
            output.nq,
            output.nv
        ],
        [7, 5, 4, 13, 11]
    );
    assert!(
        output.all_qpos_nonzero
            && output.nonzero_rotation_coverage
            && output.three_d_rotation_coverage
    );
    assert!(output.authoring_only && !output.product_python_dependency);
    assert!(!output.candidate_kinematics_inputs);
    assert_eq!(
        [
            output.xpos.len(),
            output.xquat_wxyz.len(),
            output.xipos.len(),
            output.xmat.len(),
            output.xanchor.len(),
            output.xaxis.len(),
            output.subtree_com.len(),
            output.cdof.len()
        ],
        [105, 140, 105, 315, 84, 84, 105, 462]
    );
    let model_bytes = fs::read(package_path(&package, "/model_package/path")).unwrap();
    assert_eq!(digest(&model_bytes), package["model_package"]["sha256"]);
    let model: Value = serde_json::from_slice(&model_bytes).unwrap();
    assert_eq!(
        i32_values(&model["model"]["body_parentid"]),
        input.body_parentid
    );
    assert_eq!(
        i32_values(&model["model"]["body_rootid"]),
        input.body_rootid
    );
    assert_eq!(
        i32_values(&model["model"]["dof_bodyid"]),
        dof_layout(&input).0
    );
    assert_eq!(
        f32_values(&model["model"]["body_mass"])[0..5],
        input.body_mass[0][..5]
    );
    let gravity_receipt =
        fs::read(package_path(&package, "/gravity/directory").join("receipt.json")).unwrap();
    assert_eq!(
        digest(&gravity_receipt),
        package["gravity"]["receipt_sha256"]
    );
    for pointer in [
        "/expected/absolute_tolerance",
        "/expected/total_values",
        "/boundaries/three_d_rotation_coverage",
        "/boundaries/full_state_space_accepted",
    ] {
        let mut changed = package.clone();
        *changed.pointer_mut(pointer).unwrap() = match pointer {
            "/expected/absolute_tolerance" => Value::from(0.00002),
            "/expected/total_values" => Value::from(1476),
            "/boundaries/three_d_rotation_coverage" => Value::Bool(false),
            _ => Value::Bool(true),
        };
        assert!(validate_package(&changed).is_err(), "accepted {pointer}");
    }
}
