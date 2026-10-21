//! 状态打包回归测试。

use super::captures::verify_captures;
use super::external::{reader_anchors, verify_reader, verify_upstream};
use super::validate::validate;
use super::*;
use std::path::Path;

#[test]
fn covers_state_declarations_and_all_ten_packing_functions() {
    validate(&inventory()).unwrap();
    // 编译器核对公开状态类型。
    // 本测试不构造设备会话。
    let s = mjwarp_rs::physics::G01State::default();
    let _: Vec<f32> = s.qpos;
    let _: Vec<f32> = s.qvel;
    let _: Vec<f32> = s.mocap_pos;
    let _: Vec<f32> = s.mocap_quat;
    let _: Option<Vec<bool>> = s.flex_hessian_valid;
}
#[test]
fn rejects_each_missing_duplicated_or_mistyped_state() {
    for index in 0..15 {
        for mutation in 0..6 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.states.remove(index);
                }
                1 => i.states[index] = i.states[(index + 1) % 15].clone(),
                2 => i.states[index].host_type = "Vec<f64>".into(),
                3 => i.states[index].device_type = "TransferBuffer<f64>".into(),
                4 => i.states[index].classification = "equivalent".into(),
                _ => i.states[index].device_field = "wrong_buffer".into(),
            }
            assert!(validate(&i).is_err(), "state {index}, mutation {mutation}");
        }
    }
}
#[test]
fn rejects_packing_source_or_coverage_drift() {
    for index in 0..10 {
        let mut i = inventory();
        i.packers.remove(index);
        assert!(validate(&i).is_err());
        let mut i = inventory();
        i.packers[index].parameters = "f64".into();
        assert!(validate(&i).is_err());
    }
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    let mut i = inventory();
    i.np01_complete = true;
    assert!(validate(&i).is_err());
    let mut i = inventory();
    i.remaining.clear();
    assert!(validate(&i).is_err());
}
#[test]
fn rejects_mislabelled_cache_components_and_device_readers() {
    for index in 0..15 {
        let mut i = inventory();
        if let Some(first) = i.states[index].upstream.first_mut() {
            first.name = "qacc".into();
            assert!(validate(&i).is_err());
        }
    }
    for (entry, path) in [
        ("tendon_wake", "src/physics/sleep.rs"),
        ("flex_positions", "src/physics/flex.rs"),
        ("flex_invalidate", "src/physics/flex.rs"),
        ("flex_hessian", "src/physics/flex_hessian.rs"),
    ] {
        let text = source(path).unwrap();
        verify_reader(entry, text).unwrap();
        for anchor in reader_anchors(entry) {
            assert!(verify_reader(entry, &text.replace(anchor, "changed_reader")).is_err());
        }
    }
}
#[test]
#[ignore = "reads frozen types and existing CUDA captures, with no GPU execution"]
fn audit_state_packing_sources_and_captured_readers() {
    let i = inventory();
    validate(&i).unwrap();
    let root = std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream root");
    let upstream = std::fs::read_to_string(Path::new(&root).join("_src/types.py")).unwrap();
    verify_upstream(&i, &upstream).unwrap();
    let warp_root = std::env::var_os("MJWARP_FROZEN_WARP_ROOT").expect("set Warp root");
    let warp = std::fs::read_to_string(Path::new(&warp_root).join("_src/types.py")).unwrap();
    super::super::verify_external(&super::super::inventory(), &upstream, &warp).unwrap();
    assert!(warp.contains("class spatial_vectorf(vector(length=6, dtype=float32)):"));
    assert!(warp.contains("spatial_vector = spatial_vectorf"));
    assert!(
        warp.replace("\r\n", "\n")
            .contains("elif dtype is float:\n        dtype = float32")
    );
    assert!(verify_upstream(&i, &(upstream.clone() + "\n")).is_err());
    let mut changed = i.clone();
    changed.states[0].upstream[0].declaration = "array(\"nworld\", \"nv\", float)".into();
    assert!(verify_upstream(&changed, &upstream).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("set capture root");
    let captures = verify_captures(Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1, "scope":i.scope, "inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision, "upstream_revision":i.upstream_revision,
        "state_declarations":15, "upstream_fields":25, "packing_functions":10,
        "captured_cuda_readers":captures, "gpu_executions":0, "python_executions":0,
        "np01_complete":false, "remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_STATE_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
