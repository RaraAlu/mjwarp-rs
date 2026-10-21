//! 逐字段 ABI/宿主/上游声明核验。

use super::{Check, Field, require, source, upstream_scalar};

pub(super) fn verify_field(field: &Field, g01_id: Option<usize>) -> Check {
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
