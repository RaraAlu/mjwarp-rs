//! 原生入口的静态精度映射。
//! 不执行GPU或原生动态库。
#![cfg(feature = "cuda-probe")]

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::PathBuf};

#[path = "support/t4_state_packing.rs"]
mod state_packing;

const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const FIXTURE: &str = include_str!("../fixtures/precision/g01-inputs.json");
const REMAINING: [&str; 6] = [
    "state-inputs",
    "device-packing",
    "intermediates-and-accumulators",
    "outputs",
    "upstream-generated-kernels",
    "measured-performance",
];

macro_rules! sources {
    ($($path:literal),+ $(,)?) => {
        &[$(($path, include_str!(concat!("../", $path)))),+]
    };
}

const SOURCES: &[(&str, &str)] = sources![
    "native/g01_fields.inc",
    "native/model_probe.cpp",
    "include/mjwarp_native_probe.h",
    "src/io/g01.rs",
    "src/io/kinematic.rs",
    "src/io/inertial.rs",
    "src/io/flex.rs",
    "src/io/fields.rs",
    "src/model/topology.rs",
    "src/model/inertial.rs",
    "src/model/attached.rs",
    "src/model/camlight.rs",
    "src/model/mocap.rs",
    "src/model/tendon.rs",
    "src/model/spatial_tendon.rs",
    "src/model/flex.rs",
    "src/model/sleep.rs",
    "src/model/parameters.rs",
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stamp {
    path: String,
    sha256: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    name: String,
    family: String,
    native_scalar: String,
    host_file: String,
    host_field: String,
    host_type: String,
    upstream_declaration: String,
    comparison: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    upstream_revision: String,
    product_revision: String,
    warp_version: String,
    product_sources: Vec<Stamp>,
    upstream_types: Stamp,
    warp_types: Stamp,
    np01_complete: bool,
    remaining: Vec<String>,
    fields: Vec<Field>,
}

type Check = Result<(), String>;

fn require(ok: bool, context: impl Into<String>) -> Check {
    if ok { Ok(()) } else { Err(context.into()) }
}

fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.replace("\r\n", "\n")))
}

fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, text)| (*p == path).then_some(*text))
        .ok_or_else(|| format!("unregistered source: {path}"))
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Result<&'a str, String> {
    text.split_once(start)
        .and_then(|(_, tail)| tail.split_once(end).map(|(body, _)| body))
        .ok_or_else(|| format!("missing section: {start}"))
}

// 只解析本仓库的固定声明格式。
// 格式变化必须重新人工审查。
fn input_fields() -> Result<Vec<(String, String, String)>, String> {
    let mut fields = Vec::new();
    for (family, symbol, count) in [
        ("kinematic", "NativeKinematicSnapshot", 12),
        ("inertial", "NativeInertialSnapshot", 9),
        ("flex", "NativeFlexPositionSnapshot", 12),
    ] {
        let text = source(&format!("src/io/{family}.rs"))?;
        let body = between(text, &format!("pub struct {symbol} {{"), "}")?;
        let mut found = 0;
        for line in body.lines().map(str::trim) {
            let Some((name, ty)) = line
                .strip_prefix("pub ")
                .and_then(|s| s.split_once(": Vec<"))
            else {
                continue;
            };
            let ty = ty
                .strip_suffix(">,")
                .ok_or("invalid snapshot declaration")?;
            require(matches!(ty, "f64" | "i32" | "u8"), "snapshot scalar")?;
            fields.push((name.into(), family.into(), ty.into()));
            found += 1;
        }
        require(found == count, format!("snapshot coverage: {family}"))?;
    }
    let text = source("src/io/g01.rs")?;
    let body = between(text, "const FIELDS: [(&str, u32); 45] = [", "];")?;
    for line in body.lines().map(str::trim).filter(|s| !s.is_empty()) {
        let (name, kind) = line
            .strip_prefix("(\"")
            .and_then(|s| s.strip_suffix("),"))
            .and_then(|s| s.split_once("\", "))
            .ok_or("invalid G01 field declaration")?;
        let scalar = match kind {
            "1" => "i32",
            "2" => "f64",
            "3" => "u8",
            _ => return Err("unknown G01 scalar".into()),
        };
        fields.push((name.into(), "g01".into(), scalar.into()));
    }
    require(fields.len() == 78, "native input coverage")?;
    Ok(fields)
}

fn upstream_scalar(declaration: &str) -> Result<&'static str, String> {
    let ty = if declaration == "int" {
        "int"
    } else {
        declaration
            .strip_prefix("array(")
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.rsplit_once(", ").map(|(_, ty)| ty))
            .ok_or("invalid upstream declaration")?
    };
    match ty {
        "float" | "wp.vec2" | "wp.vec3" | "wp.quat" | "wp.mat33" => Ok("f32"),
        "int" | "wp.vec2i" | "wp.vec3i" => Ok("i32"),
        "bool" => Ok("bool"),
        _ => Err(format!("unreviewed upstream scalar: {ty}")),
    }
}

fn verify_field(field: &Field, g01_id: Option<usize>) -> Check {
    let io = source(&format!("src/io/{}.rs", field.family))?;
    let name = &field.name;
    let host = source(&field.host_file)?;
    require(
        host.lines().any(|s| {
            s.trim().strip_prefix("pub ").unwrap_or(s.trim())
                == format!("{}: {},", field.host_field, field.host_type)
        }),
        format!("host declaration: {name}"),
    )?;
    let scalar = match field.host_type.as_str() {
        "Vec<f32>" | "ParameterBatch" => "f32",
        "Vec<i32>" => "i32",
        "Vec<bool>" | "bool" => "bool",
        _ => return Err(format!("unreviewed host scalar: {name}")),
    };
    if field.host_type == "ParameterBatch" {
        require(
            source("src/model/parameters.rs")?.contains("values: Vec<f32>,"),
            "batch scalar",
        )?;
    }
    let upstream = upstream_scalar(&field.upstream_declaration)?;
    let adapter = matches!(
        name.as_str(),
        "tendon_limited" | "enableflags" | "disableflags"
    );
    require(
        field.comparison
            == if adapter {
                "explicit-adapter"
            } else {
                "scalar-match"
            },
        "adapter classification",
    )?;
    require(
        if adapter {
            scalar == "bool" && upstream == "i32"
        } else {
            scalar == upstream
        },
        format!("scalar mapping: {name}"),
    )?;
    let host_name = match name.as_str() {
        "enableflags" => "sleep_enabled".into(),
        "disableflags" => "island_disabled".into(),
        _ => name.replace("_J_", "_j_"),
    };
    require(
        field.host_field == host_name,
        format!("host identity: {name}"),
    )?;
    if let Some(id) = g01_id {
        let (kind, method) = match field.native_scalar.as_str() {
            "i32" => (1, "ints"),
            "f64" => (2, "nums"),
            "u8" => (3, "bools"),
            _ => return Err("unknown native kind".into()),
        };
        let selector = source("native/g01_fields.inc")?;
        let prefix = format!("case {id}: source = ");
        let line = selector
            .lines()
            .map(str::trim)
            .find(|s| s.starts_with(&prefix))
            .ok_or("missing selector")?;
        let source_name = if matches!(name.as_str(), "enableflags" | "disableflags") {
            format!("&m->opt.{name};")
        } else {
            format!("m->{name};")
        };
        require(
            line.starts_with(&format!("{prefix}{source_name}"))
                && line.ends_with(&format!("kind = {kind}; break;")),
            format!("native selector: {name}"),
        )?;
        require(
            io.matches(&format!("self.{method}({id})?")).count() == 1,
            format!("conversion route: {name}"),
        )?;
    } else {
        let c_type = match field.native_scalar.as_str() {
            "f64" => "double",
            "i32" => "int32_t",
            "u8" => "uint8_t",
            _ => return Err("unknown native ABI".into()),
        };
        let header = source("include/mjwarp_native_probe.h")?;
        require(
            header.contains(&format!("{c_type}* {name};")),
            format!("native ABI: {name}"),
        )?;
        require(
            source("native/model_probe.cpp")?.lines().any(|s| {
                s.contains(&format!("memcpy(t->{name}, m->{name},"))
                    && s.contains(&format!("sizeof({c_type})"))
            }),
            format!("native copy: {name}"),
        )?;
        match scalar {
            "f32" => require(
                io.contains(&format!("{name}: convert(\"{name}\", &self.{name})?")),
                format!("float conversion: {name}"),
            )?,
            "i32" => require(
                io.contains(&format!("{name}: self.{name},")),
                format!("integer conversion: {name}"),
            )?,
            "bool" => require(
                io.contains(
                    "flex_centered: self.flex_centered.into_iter().map(|v| v == 1).collect()",
                ),
                "boolean conversion",
            )?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn validate(inventory: &Inventory) -> Check {
    require(
        inventory.schema == 1 && inventory.scope == "g01-native-model-ingress-only",
        "schema/scope",
    )?;
    require(
        inventory.upstream_revision == REVISION && inventory.warp_version == "1.15.0",
        "upstream version",
    )?;
    require(
        inventory.product_revision == "62b92e88100d20939c1a5c0a96f5f5f20626e3d2",
        "product baseline",
    )?;
    require(
        !inventory.np01_complete && inventory.remaining.iter().map(String::as_str).eq(REMAINING),
        "incomplete boundary",
    )?;
    let paths: BTreeSet<_> = inventory
        .product_sources
        .iter()
        .map(|s| s.path.as_str())
        .collect();
    require(
        paths == SOURCES.iter().map(|(p, _)| *p).collect()
            && paths.len() == inventory.product_sources.len(),
        "source coverage",
    )?;
    for stamp in &inventory.product_sources {
        require(
            digest(source(&stamp.path)?) == stamp.sha256,
            format!("source drift: {}", stamp.path),
        )?;
    }
    let kernels: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    require(
        inventory.upstream_types.path == "_src/types.py"
            && inventory.warp_types.path == "_src/types.py",
        "external path",
    )?;
    let frozen = kernels["upstream_sources"]
        .as_array()
        .ok_or("kernel inventory")?
        .iter()
        .find(|s| s["path"] == "_src/types.py")
        .ok_or("frozen types")?;
    require(
        frozen["sha256"] == inventory.upstream_types.sha256,
        "upstream source baseline",
    )?;
    require(
        inventory.warp_types.sha256.len() == 64
            && inventory
                .warp_types
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit()),
        "Warp digest",
    )?;
    let expected = input_fields()?;
    require(inventory.fields.len() == expected.len(), "field coverage")?;
    let mut seen = BTreeSet::new();
    for (index, (field, (name, family, abi))) in inventory.fields.iter().zip(expected).enumerate() {
        require(seen.insert(&field.name), "duplicate field")?;
        require(
            (
                field.name.as_str(),
                field.family.as_str(),
                field.native_scalar.as_str(),
            ) == (name.as_str(), family.as_str(), abi.as_str()),
            "field identity/ABI",
        )?;
        verify_field(field, (index >= 33).then(|| index - 33))?;
    }
    let g01 = source("src/io/g01.rs")?;
    for anchor in [
        "Num(Vec<f64>)",
        "Int(Vec<i32>)",
        "Bool(Vec<u8>)",
        "convert(FIELDS[id].0, v)",
        "0 => Ok(false)",
        "1 => Ok(true)",
        "sleep_enabled: enable[0] & (1 << 4) != 0",
        "island_disabled: disable[0] & (1 << 18) != 0",
    ] {
        require(g01.contains(anchor), format!("G01 conversion: {anchor}"))?;
    }
    require(
        source("src/io/fields.rs")?.contains("*output = value as f32;"),
        "quantization",
    )?;
    Ok(())
}

fn verify_external(inventory: &Inventory, upstream: &str, warp: &str) -> Check {
    require(
        digest(upstream) == inventory.upstream_types.sha256,
        "upstream source drift",
    )?;
    require(
        digest(warp) == inventory.warp_types.sha256,
        "Warp source drift",
    )?;
    let model = between(upstream, "class Model:", "class Contact:")?;
    let option = between(upstream, "class Option:", "class Statistic:")?;
    for field in &inventory.fields {
        let text = if matches!(field.name.as_str(), "enableflags" | "disableflags") {
            option
        } else {
            model
        };
        let declaration = format!("  {}: {}", field.name, field.upstream_declaration);
        require(
            text.lines().filter(|line| *line == declaration).count() == 1,
            format!("upstream declaration: {}", field.name),
        )?;
    }
    require(
        upstream.contains("arr = wp.array(ndim=len(shape), dtype=dtype)"),
        "upstream array adapter",
    )?;
    let warp = warp.replace("\r\n", "\n");
    for anchor in [
        "if dtype is int:\n            dtype = int32\n        elif dtype is float:\n            dtype = float32",
        "class vec2f(vector(length=2, dtype=float32)):",
        "class vec3f(vector(length=3, dtype=float32)):",
        "class quatf(quaternion(dtype=float32)):",
        "class mat33f(matrix(shape=(3, 3), dtype=float32)):",
        "class vec2i(vector(length=2, dtype=int32)):",
        "class vec3i(vector(length=3, dtype=int32)):",
        "vec2 = vec2f",
        "vec3 = vec3f",
        "quat = quatf",
        "mat33 = mat33f",
    ] {
        require(
            warp.contains(anchor),
            format!("Warp scalar definition: {anchor}"),
        )?;
    }
    Ok(())
}

fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}

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
