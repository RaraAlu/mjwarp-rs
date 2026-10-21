//! 冻结输入与工件门禁漂移测试。

use super::abi::*;
use super::artifact::*;
use super::package_gate::*;
use super::support::*;
use super::*;

#[test]
pub(super) fn frozen_input_and_artifact_gate_reject_drift() {
    let package = package().unwrap();
    validate_package(&package).unwrap();
    assert_eq!(
        package["artifact_requirements"]["actual_artifact_present"],
        true
    );
    assert_eq!(
        package["artifact_requirements"]["equivalence_eligible"],
        true
    );
    let missing = Temp::new("missing");
    assert_eq!(
        admit_artifact(&package, &missing.0),
        Err(GateError::Missing("artifact receipt"))
    );
    for (pointer, replacement) in [
        ("/upstream_revision", Value::String("wrong".into())),
        ("/warp_version", Value::String("1.17.0".into())),
        ("/dimensions/nv", Value::from(5)),
        ("/state/full_chain_independent", Value::Bool(true)),
    ] {
        let mut changed = package.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            validate_package(&changed).is_err(),
            "accepted drift: {pointer}"
        );
    }

    for (name, pointer, replacement) in [
        (
            "wrong-version",
            "/warp_version",
            Value::String("1.17.0".into()),
        ),
        ("wrong-rank", "/launch_rank", Value::from(2)),
        ("missing-array", "/array_arguments", Value::from(10)),
    ] {
        let temp = Temp::new(name);
        let mut value = receipt(&package);
        *value.pointer_mut(pointer).unwrap() = replacement;
        fs::write(
            temp.0.join("receipt.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validate_artifact_candidate(&package, &temp.0),
            Err(GateError::Mismatch("artifact receipt identity"))
        );
    }

    let valid = Temp::new("synthetic-valid");
    let (cu, ptx, value) = synthetic_candidate(&package);
    write_candidate(&valid.0, &cu, &ptx, &value);
    let identity = validate_artifact_candidate(&package, &valid.0).unwrap();
    assert_eq!(identity.evidence_kind, "synthetic-non-physical");
    assert_eq!(
        admit_artifact(&package, &valid.0),
        Err(GateError::Mismatch("artifact evidence kind"))
    );

    let generated = root().join(
        package["generation_pipeline"]["artifact_directory"]
            .as_str()
            .unwrap(),
    );
    let identity = admit_artifact(&package, &generated).unwrap();
    assert_eq!(
        identity.entry,
        "_gravity_force_7c63ac28_cuda_kernel_forward"
    );
    assert_eq!(
        identity.cu_sha256,
        "dd9338e9f38feb767273c3e1a445aea54bee3dee7d899076363202a4d26e865d"
    );
    assert_eq!(
        identity.ptx_sha256,
        "d06154da63f64118e8fdb19c0b646a59c5b485d580f4716e5edea97a57c1dd70"
    );

    let trusted_drift = Temp::new("trusted-drift");
    for name in [
        "generate.py",
        "generation-execution.json",
        "generation.log",
        "module.cu",
        "module.meta",
        "module.ptx",
        "receipt.json",
    ] {
        fs::copy(generated.join(name), trusted_drift.0.join(name)).unwrap();
    }
    let mut changed = fs::read(trusted_drift.0.join("module.cu")).unwrap();
    changed[0] ^= 1;
    fs::write(trusted_drift.0.join("module.cu"), changed).unwrap();
    assert!(FrozenWarpGravityArtifact::load(&trusted_drift.0, 0).is_err());

    let changed = Temp::new("changed-byte");
    let mut changed_cu = cu.clone();
    changed_cu[0] ^= 1;
    write_candidate(&changed.0, &changed_cu, &ptx, &value);
    assert_eq!(
        validate_artifact_candidate(&package, &changed.0),
        Err(GateError::Mismatch("artifact hash"))
    );

    for (name, cu_edit, ptx_edit, expected) in [
        (
            "bounds-size",
            None,
            Some(("_param_0[32]", "_param_0[24]")),
            GateError::Mismatch("ptx abi"),
        ),
        (
            "array-type",
            Some((
                "wp::array_t<wp::float32> var_body_mass",
                "wp::array_t<wp::int32> var_body_mass",
            )),
            None,
            GateError::Mismatch("cuda abi"),
        ),
        (
            "array-order",
            Some(("var_body_parentid", "var_body_rootid_changed")),
            None,
            GateError::Mismatch("cuda abi"),
        ),
        (
            "wrong-target",
            None,
            Some((".target sm_89", ".target sm_90")),
            GateError::Mismatch("ptx target"),
        ),
        (
            "wrong-address-size",
            None,
            Some((".address_size 64", ".address_size 32")),
            GateError::Mismatch("ptx address size"),
        ),
        (
            "missing-parameter",
            None,
            Some((
                ".param .align 8 .b8 _gravity_force_untrusted_cuda_kernel_forward_param_11[56]\n",
                "",
            )),
            GateError::Mismatch("ptx abi"),
        ),
        (
            "comment-entry",
            Some((
                "extern \"C\" __global__ void _gravity_force_untrusted_cuda_kernel_forward(",
                "// extern \"C\" __global__ void _gravity_force_untrusted_cuda_kernel_forward(",
            )),
            None,
            GateError::Mismatch("entry missing"),
        ),
        (
            "block-comment-entry",
            Some((
                "extern \"C\" __global__ void _gravity_force_untrusted_cuda_kernel_forward(",
                "/*\nextern \"C\" __global__ void _gravity_force_untrusted_cuda_kernel_forward(",
            )),
            None,
            GateError::Mismatch("entry missing"),
        ),
    ] {
        let temp = Temp::new(name);
        let mut changed_cu = String::from_utf8(cu.clone()).unwrap();
        let mut changed_ptx = String::from_utf8(ptx.clone()).unwrap();
        if let Some((from, to)) = cu_edit {
            changed_cu = changed_cu.replacen(from, to, 1);
        }
        if let Some((from, to)) = ptx_edit {
            changed_ptx = changed_ptx.replacen(from, to, 1);
        }
        let mut changed_receipt = value.clone();
        changed_receipt["cu_sha256"] = Value::String(digest(changed_cu.as_bytes()));
        changed_receipt["ptx_sha256"] = Value::String(digest(changed_ptx.as_bytes()));
        write_candidate(
            &temp.0,
            changed_cu.as_bytes(),
            changed_ptx.as_bytes(),
            &changed_receipt,
        );
        assert_eq!(
            validate_artifact_candidate(&package, &temp.0),
            Err(expected)
        );
    }
}
