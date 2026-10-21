//! 拓扑与帧精度回归测试。

use super::captures::verify_captures;
use super::external::{ExternalText, verify_external};
use super::routes::validate_routes;
use super::validate::{validate, validate_outputs, validate_steps};
use super::{FIXTURE, Field, OUTPUTS, ROUTES, SOURCES, STEPS, digest, inventory};
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn covers_topology_internal_groups_and_outputs() {
    validate(&inventory()).unwrap();
    assert_eq!((ROUTES.len(), STEPS.len(), OUTPUTS.len()), (23, 16, 8));
}
#[test]
fn rejects_every_missing_or_mistyped_topology_route() {
    for index in 0..ROUTES.len() {
        for mutation in 0..6 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.routes.remove(index);
                }
                1 => i.routes[index].scalar = "f64".into(),
                2 => i.routes[index].read = "wrong_reader".into(),
                3 => i.routes[index].write = "wrong_writer".into(),
                4 => i.routes[index].upstream.push(Field {
                    name: "bad".into(),
                    declaration: "int".into(),
                }),
                _ => i.routes.swap(index, (index + 1) % ROUTES.len()),
            }
            assert!(validate_routes(&i).is_err(), "{index}/{mutation}");
        }
    }
}
#[test]
fn rejects_internal_and_output_precision_drift() {
    for index in 0..STEPS.len() {
        for mutation in 0..4 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.steps.remove(index);
                }
                1 => i.steps[index].scalar = "f64".into(),
                2 => {
                    i.steps[index].product.pop();
                }
                _ => i.steps[index].classification = "equivalent".into(),
            }
            assert!(validate_steps(&i).is_err());
        }
    }
    for index in 0..OUTPUTS.len() {
        for mutation in 0..4 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.outputs.remove(index);
                }
                1 => i.outputs[index].scalar = "f64".into(),
                2 => i.outputs[index].view = "&v[..]".into(),
                _ => i.outputs[index].write = "wrong_output".into(),
            }
            assert!(validate_outputs(&i).is_err());
        }
    }
}
#[test]
fn rejects_source_gaps_and_premature_acceptance() {
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    let mut i = inventory();
    i.external_sources.pop();
    assert!(validate(&i).is_err());
    let mut i = inventory();
    i.np01_complete = true;
    assert!(validate(&i).is_err());
    let mut i = inventory();
    i.np02_complete = true;
    assert!(validate(&i).is_err());
    let mut i = inventory();
    i.remaining.clear();
    assert!(validate(&i).is_err());
}
#[test]
#[ignore = "reads frozen upstream and existing captures; never executes Python or GPU"]
fn audit_frozen_topology_and_frame_sources() {
    let i = inventory();
    validate(&i).unwrap();
    let mut texts = ExternalText::new();
    for s in &i.external_sources {
        let env = if s.root == "upstream" {
            "MJWARP_FROZEN_UPSTREAM_ROOT"
        } else {
            "MJWARP_FROZEN_WARP_ROOT"
        };
        let root = std::env::var_os(env).expect(env);
        let text = std::fs::read_to_string(Path::new(&root).join(&s.path))
            .unwrap()
            .replace("\r\n", "\n");
        texts.insert((s.root.clone(), s.path.clone()), text);
    }
    verify_external(&i, &texts).unwrap();
    let mut changed = i.clone();
    changed.routes[0].upstream[0].declaration = "array(\"nbody\", float)".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = i.clone();
    changed.steps[0].upstream[0] = "def bad():".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = i.clone();
    changed.outputs[0].upstream_declaration = "array(\"nbody\", float)".into();
    assert!(verify_external(&changed, &texts).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("evidence root");
    let (captures, resident) = verify_captures(Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1,"scope":i.scope,"inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision,"upstream_revision":i.upstream_revision,
        "topology_routes":i.routes.len(),"internal_groups":i.steps.len(),"output_fields":i.outputs.len(),
        "upstream_topology_fields":i.routes.iter().flat_map(|r|r.upstream.iter().map(|u|u.name.as_str())).collect::<BTreeSet<_>>().len(),
        "captured_variants":captures,"resident_variants":resident,
        "np01_complete":false,"np02_complete":false,"np03":"historical-unchanged","np04":"not-measured",
        "gpu_executions":0,"python_executions":0,"remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_TOPOLOGY_FRAMES_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
