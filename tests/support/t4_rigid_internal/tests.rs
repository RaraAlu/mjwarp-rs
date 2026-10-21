//! 刚体内部量与输出映射的回归测试。

use super::super::{between, digest};
use super::captures::{capture_variant, verify_captures};
use super::external::{Texts, verify_external};
use super::types::{inventory, kernel};
use super::validate::{validate, validate_boundaries, validate_outputs, validate_steps};
use super::{FIXTURE, OUTPUTS, PHYSICS, SOURCES, STEPS, historical_rigid_kernel};
use std::path::Path;

#[test]
fn covers_seventeen_internal_groups_and_seven_outputs() {
    validate(&inventory()).unwrap();
    assert_eq!((STEPS.len(), OUTPUTS.len()), (17, 7));
}

#[test]
fn rejects_missing_retyped_or_reclassified_internal_groups() {
    for index in 0..STEPS.len() {
        for mutation in 0..5 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.steps.remove(index);
                }
                1 => i.steps[index].scalar = "f64".into(),
                2 => i.steps[index].classification = "equivalent".into(),
                3 => {
                    i.steps[index].product.pop();
                }
                _ => i.steps[index].product[0] = "wrong_product".into(),
            }
            assert!(validate_steps(&i, kernel()).is_err(), "{index}/{mutation}");
        }
    }
}

#[test]
fn rejects_missing_or_misrouted_output_fields() {
    for index in 0..OUTPUTS.len() {
        for mutation in 0..5 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.outputs.remove(index);
                }
                1 => i.outputs[index].scalar = "f64".into(),
                2 => i.outputs[index].view = "&v[..]".into(),
                3 => i.outputs[index].pointer = "wrong_pointer".into(),
                _ => i.outputs[index].write = "wrong_write".into(),
            }
            assert!(validate_outputs(&i).is_err());
        }
    }
}

#[test]
fn preserves_zero_quaternion_evidence_and_active_macros() {
    validate_boundaries(&inventory(), &kernel().replace('\n', "\r\n")).unwrap();
    for mutation in 0..5 {
        let mut i = inventory();
        match mutation {
            0 => i.zero_quaternion.status = "accepted".into(),
            1 => i.zero_quaternion.warp_header_return = [0; 4],
            2 => i.zero_quaternion.upstream_generated_verified = false,
            3 => i.equivalence_eligible = true,
            _ => i.zero_quaternion.product_return = [0; 4],
        }
        assert!(validate_boundaries(&i, kernel()).is_err());
    }
    assert!(
        validate_boundaries(
            &inventory(),
            &kernel().replace("if(n==0.0f) return {0,0,0,1};", "changed_fallback")
        )
        .is_err()
    );
    assert_eq!(
        capture_variant("// #define MJWARP_G01_EQUIVALENT\n#ifdef MJWARP_MOCAP").unwrap(),
        (false, false)
    );
    assert!(capture_variant("#define MJWARP_G01_EQUIVALENT").is_err());
    // 两段已测溯源，各自只回溯一个修复。
    // 第一段：旋转分组修复（8a5bac0）。整文件还原为c74aede版本。
    let pre_rotation = super::super::rotate_lineage::before(PHYSICS).unwrap();
    assert_eq!(
        digest(&pre_rotation),
        "8e8bbc2311c5136791d50f4ee68545d755a689cc62d0d9d8b773071795c59018"
    );
    // 第二段：零四元数分支。模板还原为历史实测捕获。
    // 该段之前的整文件差异只有模块导出，故比较刚体模板。
    let pre_zero = historical_rigid_kernel(
        between(&pre_rotation, "const KINEMATICS_CUDA: &str = r#\"", "\"#;").unwrap(),
    )
    .unwrap();
    let kernels: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/precision/t4-kernels.json")).unwrap();
    let captured: Vec<_> = kernels["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["entry"] == "rigid_kinematics")
        .filter(|a| {
            a["ptx_sha256"] == "7d1db8ebffc8c03d39bec4b615e65847cca505e58dd8fb856ce5b624c3e03eb5"
        })
        .collect();
    assert_eq!(captured.len(), 1);
    assert_eq!(digest(&pre_zero), captured[0]["source_sha256"]);
    assert!(super::super::rotate_lineage::before(&PHYSICS.replace("2.0f*q.w", "2*q.w")).is_err());
    assert!(historical_rigid_kernel(&kernel().replace("{0,0,0,1}", "{1,0,0,0}")).is_err());
    let report: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/precision/g01-zero-quaternion-result.json"
    ))
    .unwrap();
    assert_eq!(report["zero_fallback"], serde_json::json!([0, 0, 0, 1]));
    assert_eq!(report["comparisons"], 608);
    assert_eq!(report["differences"], serde_json::json!([]));
    assert_eq!(report["upstream_revision"], super::super::REVISION);
    assert_eq!(
        report["upstream_ptx_sha256"],
        "1d9e4e3a983ca35090ade3a48fd83aa0dc86a23228bb0865010f681698ab75dd"
    );
    // 本次只刷新受影响的源码戳，不重跑旧整组配置。
    for text in [
        include_str!("../../../fixtures/precision/g01-state-packing.json"),
        include_str!("../../../fixtures/precision/g01-field-internal.json"),
        include_str!("../../../fixtures/precision/g01-topology-frames.json"),
        FIXTURE,
    ] {
        let value: serde_json::Value = serde_json::from_str(text).unwrap();
        let stamp = value["product_sources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["path"] == "src/physics/mod.rs")
            .unwrap();
        assert_eq!(stamp["sha256"], digest(PHYSICS));
    }
}

#[test]
fn rejects_source_drift_and_premature_precision_acceptance() {
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    for mutation in 0..4 {
        let mut i = inventory();
        match mutation {
            0 => i.np01_complete = true,
            1 => i.np02_complete = true,
            2 => {
                i.external_sources.pop();
            }
            _ => i.remaining.clear(),
        }
        assert!(validate(&i).is_err());
    }
}

#[test]
#[ignore = "reads frozen sources and existing captures without running Python or GPU"]
fn audit_frozen_rigid_internal_and_output_sources() {
    let i = inventory();
    validate(&i).unwrap();
    let mut texts = Texts::new();
    for s in &i.external_sources {
        let (root, path) = s.path.split_once('/').unwrap();
        let env = if root == "upstream" {
            "MJWARP_FROZEN_UPSTREAM_ROOT"
        } else {
            "MJWARP_FROZEN_WARP_ROOT"
        };
        let root = std::env::var_os(env).expect(env);
        texts.insert(
            s.path.clone(),
            std::fs::read_to_string(Path::new(&root).join(path))
                .unwrap()
                .replace("\r\n", "\n"),
        );
    }
    verify_external(&i, &texts).unwrap();
    let mut changed = i.clone();
    changed.steps[0].upstream[0] = "wrong_upstream".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = i.clone();
    changed.outputs[0].upstream_declaration = "array(\"nworld\", \"nbody\", float)".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = texts.clone();
    changed.get_mut("warp/native/quat.h").unwrap().push('\n');
    assert!(verify_external(&i, &changed).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("evidence root");
    let captures = verify_captures(Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1,"scope":i.scope,"inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision,"upstream_revision":i.upstream_revision,
        "internal_groups":i.steps.len(),"output_fields":i.outputs.len(),"historical_captured_variants":captures,
        "current_full_capture_verified":false,
        "zero_quaternion_status":i.zero_quaternion.status,"equivalence_eligible":false,
        "np01_complete":false,"np02_complete":false,"np03":"historical-unchanged","np04":"not-measured",
        "gpu_executions":0,"python_executions":0,"remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_RIGID_INTERNAL_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
