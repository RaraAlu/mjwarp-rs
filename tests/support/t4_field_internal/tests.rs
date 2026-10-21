use super::checks::*;
use super::*;
use std::path::Path;

#[test]
fn maps_all_batched_fields_and_com_steps() {
    validate(&inventory()).unwrap();
}

#[test]
fn rejects_each_missing_or_misrouted_field() {
    for index in 0..23 {
        for mutation in 0..7 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.fields.remove(index);
                }
                1 => i.fields[index] = i.fields[(index + 1) % 23].clone(),
                2 => i.fields[index].parameter_scalar = "f64".into(),
                3 => i.fields[index].descriptor_scalar = "f32".into(),
                4 => i.fields[index].slot += 1,
                5 => i.fields[index].width = "1".into(),
                _ => i.fields[index].reader = "wrong_reader".into(),
            }
            assert!(validate(&i).is_err(), "field {index}, mutation {mutation}");
        }
    }
}

#[test]
fn rejects_each_missing_or_mistyped_internal_step() {
    for index in 0..STEPS.len() {
        for mutation in 0..6 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.com_steps.remove(index);
                }
                1 => i.com_steps[index] = i.com_steps[(index + 1) % STEPS.len()].clone(),
                2 => i.com_steps[index].scalar = "f64".into(),
                3 => i.com_steps[index].upstream_type = "wp.float64".into(),
                4 => i.com_steps[index].classification = "equivalent".into(),
                _ => i.com_steps[index].product.clear(),
            }
            assert!(
                validate_com(&i, com_source()).is_err(),
                "step {index}, mutation {mutation}"
            );
        }
    }
}

#[test]
fn rejects_source_drift_and_premature_completion() {
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    let i = inventory();
    for (old, new) in [
        ("float v=0.0f;", "double v=0.0;"),
        ("sm[up]+=sm[b]", "sm[up]=sm[b]"),
        ("float t[9]", "double t[9]"),
        ("1.0f", "1.0"),
    ] {
        assert!(validate_com(&i, &com_source().replace(old, new)).is_err());
    }
    let mut i = inventory();
    i.np01_complete = true;
    assert!(validate(&i).is_err());
    i.np01_complete = false;
    i.remaining.clear();
    assert!(validate(&i).is_err());
}

#[test]
#[ignore = "reads frozen sources and existing captures without GPU execution"]
fn audit_field_internal_sources_and_captures() {
    let i = inventory();
    validate(&i).unwrap();
    let root = std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream root");
    let smooth = std::fs::read_to_string(Path::new(&root).join("_src/smooth.py"))
        .unwrap()
        .replace("\r\n", "\n");
    let types = std::fs::read_to_string(Path::new(&root).join("_src/types.py")).unwrap();
    let warp_root = std::env::var_os("MJWARP_FROZEN_WARP_ROOT").expect("set Warp root");
    let warp = std::fs::read_to_string(Path::new(&warp_root).join("_src/types.py")).unwrap();
    verify_upstream(&i, &smooth, &types, &warp).unwrap();
    let mut changed = i.clone();
    changed.com_steps[0].upstream[0] = "body_mass: wp.array2d[wp.float64],".into();
    assert!(verify_upstream(&changed, &smooth, &types, &warp).is_err());
    assert!(verify_upstream(&i, &(smooth.clone() + "\n"), &types, &warp).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("set capture root");
    let (captures, batched) = verify_captures(&i, Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1, "scope":i.scope, "inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision, "upstream_revision":i.upstream_revision,
        "batched_fields":i.fields.len(), "com_step_groups":i.com_steps.len(),
        "captured_variants":captures, "batched_variants":batched,
        "gpu_executions":0, "python_executions":0, "np01_complete":false,
        "np02_complete":false, "np03":"historical-unchanged", "np04":"not-measured",
        "remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_FIELD_INTERNAL_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
