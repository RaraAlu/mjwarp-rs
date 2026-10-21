use super::super::super::face_lineage;
use super::super::super::{Check, between, digest, require};
use super::*;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

pub(super) fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}
pub(super) fn source(path: &str) -> Result<&'static str, String> {
    if path == "src/physics/resident.rs" {
        return Ok(RESIDENT);
    }
    SOURCES
        .iter()
        .find_map(|(p, s)| (*p == path).then_some(*s))
        .ok_or_else(|| format!("unreviewed source: {path}"))
}
// 固定格式的源码守卫，不冒充语法解析器。
// 来源哈希同时保护完整函数。
pub(super) fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
pub(super) fn pack_body<'a>(resident: &'a str, name: &str) -> Result<&'a str, String> {
    between(resident, &format!("fn {name}("), "\n}")
}
pub(super) fn cuda(path: &str) -> Result<&'static str, String> {
    between(source(path)?, ": &str = r#\"", "\"#;")
}
pub(super) fn validate_reader(route: &Route, text: &str) -> Check {
    require(!route.read.is_empty(), "missing reader")?;
    if route.kind == "unused-i32" {
        require(
            route.id == "flex_elemadr"
                && route.read == "unused:r[6]"
                && !compact(text)
                    .match_indices("r[6]")
                    .any(|(index, _)| index == 0 || !text_before_access(text, index)),
            "unused metadata boundary",
        )
    } else {
        require(
            compact(text).contains(&compact(&route.read)),
            format!("reader: {}", route.id),
        )
    }
}
// 排除upper[6]等不同标识符。
pub(super) fn text_before_access(text: &str, compact_index: usize) -> bool {
    let before = compact(text).as_bytes()[compact_index - 1];
    before.is_ascii_alphanumeric() || before == b'_'
}
pub(super) fn validate_routes(p: &Packer, expected: &str, resident: &str, reader: &str) -> Check {
    let expected: Vec<_> = expected.split(',').collect();
    require(p.routes.len() == expected.len(), "route coverage")?;
    let body = compact(pack_body(resident, &p.name)?);
    for (r, expected) in p.routes.iter().zip(expected) {
        require(
            format!(
                "{}:{}:{}",
                r.id,
                r.kind,
                r.upstream
                    .iter()
                    .map(|u| u.name.as_str())
                    .collect::<Vec<_>>()
                    .join("+")
            ) == expected,
            "route identity or boundary",
        )?;
        let scalar = if matches!(r.kind.as_str(), "copy-f32" | "f32-precompute" | "sentinel") {
            "f32"
        } else {
            "i32"
        };
        require(r.scalar == scalar, "storage scalar")?;
        require(
            !r.write.trim().is_empty() && body.contains(&compact(&r.write)),
            format!("writer: {}", r.id),
        )?;
        validate_reader(r, reader)?;
        require(
            r.upstream
                .iter()
                .map(|u| u.name.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                == r.upstream.len(),
            "duplicate upstream fields",
        )?;
        if matches!(
            r.kind.as_str(),
            "copy-f32" | "copy-i32" | "bool-i32" | "unused-i32"
        ) {
            require(
                r.upstream.len() == 1 && r.upstream[0].name == r.id,
                "direct field identity",
            )?;
        }
        for u in &r.upstream {
            let upstream_scalar = super::super::super::upstream_scalar(&u.declaration)?;
            if matches!(r.kind.as_str(), "copy-f32" | "copy-i32" | "unused-i32") {
                require(upstream_scalar == scalar, "direct scalar")?;
            } else if r.kind == "bool-i32" {
                // tendon_limited在上游采用整数。
                // flex_centered在上游采用布尔。
                require(
                    upstream_scalar
                        == if r.id == "tendon_limited" {
                            "i32"
                        } else {
                            "bool"
                        },
                    "boolean representation",
                )?;
            }
        }
    }
    Ok(())
}
pub(super) fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1 && i.scope == "g01-remaining-eight-packing-routes",
        "scope",
    )?;
    require(
        i.product_revision == "9db581d509adbaf487cbe0498e0b7c2b9a5caba1"
            && i.upstream_revision == super::super::super::REVISION,
        "revision",
    )?;
    require(
        !i.np01_complete && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "remaining work",
    )?;
    require(
        i.product_sources.len() == SOURCES.len()
            && i.product_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == SOURCES.iter().map(|(p, _)| *p).collect(),
        "source coverage",
    )?;
    for stamp in &i.product_sources {
        let text = if stamp.path == "src/physics/flex_face.rs" {
            face_lineage::historical(source(&stamp.path)?)?
        } else {
            source(&stamp.path)?.to_owned()
        };
        require(
            digest(&text) == stamp.sha256,
            format!("source drift: {}", stamp.path),
        )?;
    }
    require(i.packers.len() == 8, "packer coverage")?;
    for (p, &(name, entry, file, routes)) in i.packers.iter().zip(PACKERS) {
        require(
            (p.name.as_str(), p.entry.as_str(), p.file.as_str()) == (name, entry, file),
            "packer identity",
        )?;
        validate_routes(p, routes, &RESIDENT.replace("\r\n", "\n"), cuda(file)?)?;
    }
    // 派生路线需绑定真正的类型来源。
    for (path, anchors) in [
        (
            "src/model/flex_hessian.rs",
            vec![
                "pub flexedge_length0: Vec<f32>",
                "pub flex_stiffness: Vec<f32>",
                "positions.flex_interp[f] == 0",
                "&& !self.flex_rigid[f]",
                "&& self.flex_dim[f] >= 2",
                "&& self.flex_stiffnessadr[f] >= 0",
                "&& self.flex_stiffness[self.flex_stiffnessadr[f] as usize] != 0.0",
            ],
        ),
        (
            "src/model/sleep.rs",
            vec![
                "pub tendon_limited: Vec<bool>",
                "pub tendon_range: ParameterBatch",
                "pub tendon_margin: ParameterBatch",
                "pub wrap_treeid: Vec<i32>",
                "fields.body_treeid[body as usize]",
            ],
        ),
        (
            "src/model/flex.rs",
            vec![
                "pub flex_centered: Vec<bool>",
                "pub flex_node: Vec<f32>",
                "pub flex_vert: Vec<f32>",
                "pub flex_vert0: Vec<f32>",
            ],
        ),
        (
            "src/model/spatial_tendon.rs",
            vec!["pub wrap_prm: Vec<f32>", "pub geom_size: ParameterBatch"],
        ),
        ("src/model/fixed_tendon.rs", vec!["pub wrap_prm: Vec<f32>"]),
    ] {
        for anchor in anchors {
            require(source(path)?.contains(anchor), "host representation")?;
        }
    }
    let spatial = compact(pack_body(
        &RESIDENT.replace("\r\n", "\n"),
        "pack_spatial_tendon",
    )?);
    for anchor in [
        "host_staging::<f32>(param_count.max(1))",
        "let mut scale = 1.0;",
        "scale = 1.0 / fields.wrap_prm[i];",
        "*value = scale;",
        "fields.wrap_prm[i].round() as i32",
    ] {
        require(spatial.contains(&compact(anchor)), "derived precision")?;
    }
    verify_fixed(cuda("src/physics/fixed_tendon.rs")?)
}
pub(super) fn verify_fixed(text: &str) -> Check {
    require(
        digest(text) == digest(cuda("src/physics/fixed_tendon.rs")?),
        "fixed template drift",
    )?;
    for &(product, _) in FIXED_INTERNAL {
        require(text.contains(product), "fixed local or accumulator")?;
    }
    Ok(())
}
pub(super) fn verify_upstream(i: &Inventory, types: &str, warp: &str, smooth: &str) -> Check {
    super::super::super::verify_external(&super::super::super::inventory(), types, warp)?;
    require(
        digest(smooth) == "6268f0e1b6cb4c9ec6bbd3aa29828dfb36224efb2a3a3e1f54e5f87b6623fdef",
        "upstream smooth drift",
    )?;
    let model = between(types, "class Model:", "class Contact:")?;
    for p in &i.packers {
        for r in &p.routes {
            for u in &r.upstream {
                let line = format!("  {}: {}", u.name, u.declaration);
                require(
                    model.lines().filter(|l| *l == line).count() == 1,
                    format!("upstream field: {}", u.name),
                )?;
            }
        }
    }
    let body = between(smooth, "def _joint_tendon(", "\n\n\n")?;
    for &(_, upstream) in FIXED_INTERNAL {
        require(body.contains(upstream), "upstream fixed expression")?;
    }
    require(
        body.contains("wrap_prm: wp.array[float],") && body.contains("qpos_in: wp.array2d[float],"),
        "fixed input scalars",
    )?;
    Ok(())
}
pub(super) fn verify_captures(i: &Inventory, root: &Path) -> Result<usize, String> {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/precision/t4-kernels.json"
    ))
    .map_err(|e| e.to_string())?;
    let artifacts = manifest["artifacts"]
        .as_array()
        .ok_or("capture inventory")?;
    let mut total = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        for p in &i.packers {
            let found: Vec<_> = artifacts.iter().filter(|a| a["entry"] == p.entry).collect();
            require(found.len() == 1, "missing or duplicate capture")?;
            let a = found[0];
            let ptx = a["ptx_sha256"].as_str().ok_or("PTX hash")?;
            require(
                ptx.len() == 64 && ptx.bytes().all(|b| b.is_ascii_hexdigit()),
                "capture path",
            )?;
            let mut text = String::new();
            for (ext, key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let path = root.join(mode).join(format!("{}-{ptx}.{ext}", p.entry));
                let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == a[key].as_str().ok_or("hash")?,
                    "artifact drift",
                )?;
                if ext == "cu" {
                    text = std::str::from_utf8(&bytes)
                        .map_err(|e| e.to_string())?
                        .replace("\r\n", "\n");
                }
                if ext == "json" {
                    let receipt: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        receipt["entry"] == p.entry
                            && receipt["passed"] == true
                            && receipt["precisionGate"] == "f32-physics"
                            && receipt["fmad"] == true
                            && receipt["useFastMath"] == false,
                        "precision receipt",
                    )?;
                }
            }
            require(
                text.contains(&cuda(&p.file)?.replace("\r\n", "\n")),
                "captured template",
            )?;
            for r in &p.routes {
                validate_reader(r, &text)?;
            }
            total += 1;
        }
    }
    Ok(total)
}
