//! 静态输入映射回归测试。

use super::external::verify_external;
use super::validate::validate;
use super::{FIXTURE, Inventory, SOURCES, digest, inventory};
use std::path::PathBuf;

#[test]
fn maps_every_native_input_without_claiming_full_precision_acceptance() {
    validate(&inventory()).unwrap();
    let i = inventory();
    assert_eq!(
        i.fields.iter().filter(|f| f.native_scalar == "f64").count(),
        32
    );
    assert_eq!(
        i.fields
            .iter()
            .filter(|f| f.comparison == "explicit-adapter")
            .count(),
        3
    );
}

#[test]
fn rejects_each_missing_duplicate_or_reordered_field() {
    for index in 0..78 {
        let mut i = inventory();
        i.fields.remove(index);
        assert!(validate(&i).is_err());
        let mut i = inventory();
        i.fields[index] = i.fields[(index + 1) % 78].clone();
        assert!(validate(&i).is_err());
        let mut i = inventory();
        i.fields.swap(index, (index + 1) % 78);
        assert!(validate(&i).is_err());
    }
}

#[test]
fn rejects_each_abi_host_upstream_or_adapter_misstatement() {
    for index in 0..78 {
        for mutation in 0..6 {
            let mut i = inventory();
            let f = &mut i.fields[index];
            match mutation {
                0 => f.native_scalar = "f16".into(),
                1 => f.host_type = "Vec<f64>".into(),
                2 => f.upstream_declaration = "array(\"nv\", wp.float64)".into(),
                3 => f.comparison = "equivalent".into(),
                4 => f.host_field = "not_the_same_field".into(),
                _ => f.host_file = "../outside.rs".into(),
            }
            assert!(validate(&i).is_err(), "field {index}, mutation {mutation}");
        }
    }
}

#[test]
fn rejects_missing_duplicate_changed_or_unregistered_sources() {
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources.remove(index);
        assert!(validate(&i).is_err());
        let mut i = inventory();
        i.product_sources.push(i.product_sources[index].clone());
        assert!(validate(&i).is_err());
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
        let mut i = inventory();
        i.product_sources[index].path = "../outside.rs".into();
        assert!(validate(&i).is_err());
    }
}

#[test]
fn rejects_version_drift_and_premature_completion() {
    for mutation in 0..8 {
        let mut i = inventory();
        match mutation {
            0 => i.np01_complete = true,
            1 => i.remaining.clear(),
            2 => i.scope = "all-physics".into(),
            3 => i.upstream_revision = "0".repeat(40),
            4 => i.product_revision = "0".repeat(40),
            5 => i.warp_version = "unknown".into(),
            6 => i.upstream_types.sha256 = "0".repeat(64),
            _ => i.warp_types.path = "../types.py".into(),
        }
        assert!(validate(&i).is_err());
    }
}

#[test]
fn rejects_unknown_json_fields_and_preserves_lf_hashes() {
    let mut v: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    v["approved"] = true.into();
    assert!(serde_json::from_value::<Inventory>(v).is_err());
    let mut v: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    v["fields"][0]["approved"] = true.into();
    assert!(serde_json::from_value::<Inventory>(v).is_err());
    assert_eq!(digest("one\r\ntwo\r\n"), digest("one\ntwo\n"));
    assert_ne!(digest("one\n"), digest("two\n"));
}

#[test]
fn quantizes_native_values_once_and_rejects_nonfinite_transactionally() {
    use mjwarp_rs::io::convert_f64_to_f32_into;
    let half_ulp = 2.0_f64.powi(-24);
    let source = [
        0.0,
        -0.0,
        1.0 + half_ulp,
        1.0 + 3.0 * half_ulp,
        f64::from(f32::from_bits(1)),
        f64::MIN_POSITIVE,
        f64::from(f32::MAX),
    ];
    let mut target = [0.0; 7];
    convert_f64_to_f32_into(&source, &mut target).unwrap();
    assert_eq!(
        target.map(f32::to_bits),
        [0, 0x8000_0000, 0x3f80_0000, 0x3f80_0002, 1, 0, 0x7f7f_ffff]
    );
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
        let mut target = [7.0, 8.0];
        assert!(convert_f64_to_f32_into(&[1.0, invalid], &mut target).is_err());
        assert_eq!(target, [7.0, 8.0]);
    }
}

#[test]
#[ignore = "requires frozen upstream and Warp source roots; never executes them"]
fn audit_frozen_input_declarations_without_gpu_or_python() {
    let i = inventory();
    validate(&i).unwrap();
    let upstream_root =
        PathBuf::from(std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream root"));
    let warp_root =
        PathBuf::from(std::env::var_os("MJWARP_FROZEN_WARP_ROOT").expect("set Warp root"));
    let upstream = std::fs::read_to_string(upstream_root.join(&i.upstream_types.path)).unwrap();
    let warp = std::fs::read_to_string(warp_root.join(&i.warp_types.path)).unwrap();
    verify_external(&i, &upstream, &warp).unwrap();
    assert!(verify_external(&i, &(upstream.clone() + "\n"), &warp).is_err());
    assert!(verify_external(&i, &upstream, &(warp.clone() + "\n")).is_err());
    // 即使修改者重算哈希，也须拒绝声明错配。
    let changed = upstream.replace(
        "  body_mass: array(\"*\", \"nbody\", float)",
        "  body_mass: array(\"*\", \"nbody\", wp.float64)",
    );
    let mut changed_inventory = i.clone();
    changed_inventory.upstream_types.sha256 = digest(&changed);
    assert!(verify_external(&changed_inventory, &changed, &warp).is_err());
    let changed = warp.replace("vec3 = vec3f", "vec3 = vec3d");
    let mut changed_inventory = i.clone();
    changed_inventory.warp_types.sha256 = digest(&changed);
    assert!(verify_external(&changed_inventory, &upstream, &changed).is_err());
    let report = serde_json::json!({
        "schema": 1, "scope": i.scope, "inventory_sha256": digest(FIXTURE),
        "upstream_revision": i.upstream_revision, "product_revision": i.product_revision,
        "warp_version": i.warp_version, "upstream_types_sha256": i.upstream_types.sha256,
        "warp_types_sha256": i.warp_types.sha256,
        "fields_checked": 78, "native_f64_to_f32": 32, "native_i32": 44,
        "native_u8": 2, "host_scalar_matches": 75, "explicit_adapters": 3,
        "np01_complete": false, "gpu_executions": 0, "python_executions": 0,
        "remaining": i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_INPUT_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
