use super::checks::*;
use super::inventory::*;
use super::*;

#[test]
fn covers_twenty_one_groups_four_float_outputs_and_cache_publication() {
    validate(&inventory()).unwrap();
    assert_eq!((GROUPS.len(), OUTPUTS.len()), (21, 4));
}
#[test]
fn rejects_group_and_output_precision_drift() {
    for n in 0..GROUPS.len() {
        for mutation in 0..4 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.groups.remove(n);
                }
                1 => i.groups[n].scalar = "f64".into(),
                2 => i.groups[n].classification = "equivalent".into(),
                _ => i.groups[n].product[0] = "missing-expression".into(),
            }
            assert!(validate(&i).is_err(), "{n}/{mutation}");
        }
    }
    for n in 0..OUTPUTS.len() {
        for mutation in 0..3 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.outputs.remove(n);
                }
                1 => i.outputs[n].scalar = "f64".into(),
                _ => i.outputs[n].view = "&values[..]".into(),
            }
            assert!(validate(&i).is_err(), "{n}/{mutation}");
        }
    }
}
#[test]
fn rejects_source_and_acceptance_boundary_drift() {
    for n in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[n].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    for n in 0..5 {
        let mut i = inventory();
        match n {
            0 => i.np01_complete = true,
            1 => i.np02_complete = true,
            2 => i.equivalence_eligible = true,
            3 => i.remaining.clear(),
            _ => {
                i.external_sources.pop();
            }
        }
        assert!(validate(&i).is_err());
    }
}
#[test]
#[ignore = "reads frozen face/Hessian sources and cached artifacts; no GPU or Python"]
fn audits_frozen_matrices_and_generated_face_artifacts() {
    let i = inventory();
    validate(&i).unwrap();
    let mut texts = Texts::new();
    for s in &i.external_sources {
        let (kind, path) = s.path.split_once('/').unwrap();
        let key = if kind == "upstream" {
            "MJWARP_FROZEN_UPSTREAM_ROOT"
        } else {
            "MJWARP_FROZEN_WARP_ROOT"
        };
        let root = std::env::var_os(key).expect(key);
        texts.insert(
            s.path.clone(),
            std::fs::read_to_string(Path::new(&root).join(path))
                .unwrap()
                .replace("\r\n", "\n"),
        );
    }
    verify_external(&i, &texts).unwrap();
    let mut changed = i.clone();
    changed.groups[0].upstream[0] = "missing-upstream".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = texts.clone();
    changed
        .get_mut("upstream/_src/passive.py")
        .unwrap()
        .push('\n');
    assert!(verify_external(&i, &changed).is_err());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pins: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/precision/g01-zero-quaternion.json"
    ))
    .unwrap();
    let prefix = root.join(pins["artifact_prefix"].as_str().unwrap());
    // 缓存名含点号；追加后缀，不能替换已有扩展名。
    let cuda = std::fs::read_to_string(format!("{}.cu", prefix.display())).unwrap();
    let ptx = std::fs::read(format!("{}.sm89.ptx", prefix.display())).unwrap();
    verify_generated(&cuda, &ptx).unwrap();
    assert!(
        verify_generated(
            &cuda.replace("var_58 = wp::normalize(var_57);", "missing-normalize"),
            &ptx
        )
        .is_err()
    );
    let mut corrupt = ptx.clone();
    corrupt[0] ^= 1;
    assert!(verify_generated(&cuda, &corrupt).is_err());
    let history = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("historical captures");
    let current = std::env::var_os("MJWARP_T4_CURRENT_CAPTURE_ROOT").expect("current captures");
    let count = verify_captures(Path::new(&history), Path::new(&current)).unwrap();
    let report = serde_json::json!({
        "schema":1,"scope":i.scope,"product_revision":i.product_revision,"upstream_revision":i.upstream_revision,
        "map_sha256":digest(FIXTURE),"internal_groups":21,"float_groups":19,"private_flag_groups":1,
        "host_gate_groups":1,"float_outputs":4,"cache_publication_fields":1,
        "historical_capture_variants":6,"retained_current_variants":3,"checked_capture_sets":count,
        "upstream_face_generated_source_verified":true,"upstream_face_ptx_hash_verified":true,
        "upstream_cuda_sha256":pins["upstream_cuda_sha256"],"upstream_ptx_sha256":pins["upstream_ptx_sha256"],
        "upstream_hessian_generated_verified":false,"np01_complete":false,"np02_complete":false,
        "equivalence_eligible":false,"np03":"historical-not-rerun","np04":"not-measured",
        "gpu_executions":0,"python_executions":0,"recompilations":0,"remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_FLEX_MATRICES_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
