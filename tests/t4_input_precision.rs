//! 原生入口的静态精度映射。
//! 默认不执行GPU；零分支实证另行显式启用。
#![cfg(feature = "cuda-probe")]

use serde::Deserialize;
use sha2::{Digest, Sha256};

#[path = "support/t4_face_lineage.rs"]
mod face_lineage;

#[path = "support/t4_rotate_lineage.rs"]
mod rotate_lineage;

#[path = "support/t4_state_packing.rs"]
mod state_packing;

#[path = "support/t4_field_internal.rs"]
mod field_internal;

#[path = "support/t4_packing_routes.rs"]
mod packing_routes;

#[path = "support/t4_topology_frames.rs"]
mod topology_frames;

#[path = "support/t4_rigid_internal.rs"]
mod rigid_internal;

#[path = "t4_input_precision/external.rs"]
mod external;
#[path = "t4_input_precision/fields.rs"]
mod fields;
#[path = "t4_input_precision/tests.rs"]
mod tests;
#[path = "t4_input_precision/validate.rs"]
mod validate;

pub(crate) use external::verify_external;
#[path = "support/t4_zero_quaternion.rs"]
mod zero_quaternion;

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

fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}
