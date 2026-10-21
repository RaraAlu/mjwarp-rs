//! 当前源与漂移拒绝测试。

use super::lineage::lineage;
use super::recipe::retained_recipe;
use super::*;

#[test]
fn preserves_current_sources_and_all_thirty_variants() {
    let value = current();
    lineage(&value).unwrap();
    let physics = value
        .product_sources
        .iter()
        .find(|stamp| stamp.path == "src/physics/mod.rs")
        .unwrap();
    assert_eq!(physics.sha256, digest(normalized(PHYSICS).as_bytes()));
    let rigid_hashes = sources().map(|source| digest(source.as_bytes()));
    let recorded: Vec<_> = value
        .artifacts
        .iter()
        .filter(|artifact| artifact.entry == ENTRY)
        .map(|artifact| artifact.source_sha256.as_str())
        .collect();
    assert_eq!(recorded.len(), 3);
    for hash in &rigid_hashes {
        assert!(recorded.contains(&hash.as_str()));
    }
    assert_eq!(normalized(RUNTIME).matches(HOOK).count(), 1);
    // 编译选项必须同时匹配实际产品调用，禁止诊断侧另选精度。
    for anchor in [
        "fmad: Some(PHYSICS_FMAD)",
        "use_fast_math: Some(false)",
        "options: vec![\"--std=c++17\".into(), \"--gpu-architecture=compute_70\".into()]",
    ] {
        assert!(RUNTIME.contains(anchor));
    }
    // 同时绑定模板组合与实际编译调用点。
    for anchor in [
        "let source = format!(\"{FIELD_BATCH_CUDA}\\n{}\", crate::physics::COM_POSITION_CUDA);",
        "crate::physics::fixed_tendon::FIXED_TENDON_CUDA,",
        "crate::physics::tendon_wrap::TENDON_WRAP_CUDA,",
        "crate::physics::spatial_tendon::SPATIAL_TENDON_CUDA",
        "crate::physics::tendon::TENDON_CUDA,",
        "crate::physics::flex::FLEX_POSITION_CUDA,",
        "crate::physics::sleep::TENDON_WAKE_CUDA,",
        "crate::physics::flex_edge::FLEX_EDGE_CUDA,",
        "crate::physics::flex_face::FLEX_FACE_CUDA,",
        "crate::physics::flex_hessian::FLEX_HESSIAN_CUDA,",
    ] {
        assert!(
            RESIDENT.contains(anchor),
            "missing resident route: {anchor}"
        );
    }
    for (source, anchor) in [
        (COM_VELOCITY, "COM_VELOCITY_CUDA, \"com_velocity\""),
        (MASS_MATRIX, "MASS_MATRIX_CUDA, \"rigid_mass_matrix\""),
        (MASS_SOLVE, "MASS_SOLVE_CUDA, \"rigid_mass_solve\""),
        (RNE_BIAS, "RNE_BIAS_CUDA, \"rne_bias\""),
    ] {
        assert!(source.contains(anchor), "missing direct route: {anchor}");
    }
    let identity: serde_json::Value = serde_json::from_str(SOURCE_IDENTITY).unwrap();
    assert_eq!(identity["variants"], 30);
    assert_eq!(identity["affected_variants"], 10);
    assert_eq!(identity["retained_variants"], 20);
    assert_eq!(identity["retained_template_matches"], 20);
    let mut expected: Vec<_> = value
        .artifacts
        .iter()
        .filter(|artifact| retained_recipe(artifact).is_some())
        .map(|artifact| {
            (
                artifact.entry.clone(),
                artifact.source_sha256.clone(),
                artifact.ptx_sha256.clone(),
            )
        })
        .collect();
    let mut observed: Vec<_> = identity["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["entry"].as_str().unwrap().to_owned(),
                row["source_sha256"].as_str().unwrap().to_owned(),
                row["ptx_sha256"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    expected.sort();
    observed.sort();
    assert_eq!(observed, expected);
}

#[test]
fn rejects_unrelated_drift_and_missing_rigid_variants() {
    let value = current();
    for n in 0..7 {
        let mut changed = value.clone();
        match n {
            0 => {
                changed.artifacts.pop();
            }
            1 => changed.artifacts[0].source_sha256 = "0".repeat(64),
            2 => {
                changed
                    .artifacts
                    .iter_mut()
                    .find(|a| a.entry == ENTRY)
                    .unwrap()
                    .source_sha256 = "0".repeat(64)
            }
            3 => changed.remaining.clear(),
            4 => changed.mappings[0].fields.pop().map(|_| ()).unwrap(),
            5 => changed.source_revision = "0".repeat(40),
            _ => changed.artifacts[0] = base().artifacts[0].clone(),
        }
        assert!(lineage(&changed).is_err(), "mutation {n}");
    }
}
