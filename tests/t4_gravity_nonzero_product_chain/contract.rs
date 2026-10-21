//! 冻结非零产品链契约自检。

use super::*;

#[test]
fn frozen_nonzero_product_chain_contract_is_self_consistent() {
    let package = load_package();
    let (input, output) = load_case(&package);
    let model_package = model_package(&package);
    for source in package["product_sources"].as_object().unwrap().values() {
        let path = source["path"].as_str().unwrap();
        let text = match path {
            "src/physics/mod.rs" => PHYSICS.to_owned(),
            "src/physics/resident.rs" => RESIDENT.to_owned(),
            _ => String::from_utf8(fs::read(root().join(path)).unwrap()).unwrap(),
        };
        assert_eq!(
            digest(text.replace("\r\n", "\n").as_bytes()),
            source["sha256"]
        );
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
