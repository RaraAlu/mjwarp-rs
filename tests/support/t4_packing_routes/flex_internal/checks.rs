use super::super::super::{Check, between, digest, require};
use super::*;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}
pub(super) fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, t)| (*p == path).then_some(*t))
        .ok_or_else(|| path.into())
}
pub(super) fn anchor(text: &str, needle: &str) -> Check {
    require(
        !needle.trim().is_empty() && text.contains(needle),
        format!("anchor: {needle}"),
    )
}
pub(super) fn body<'a>(text: &'a str, symbol: &str) -> Result<&'a str, String> {
    text.split_once(&format!("def {symbol}("))
        .map(|(_, s)| s.split("\n\n\n").next().unwrap())
        .ok_or_else(|| symbol.into())
}
pub(super) fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1
            && i.scope == "g01-flex-position-edge-internal-output"
            && i.product_revision == "e272b1474391c4b875ad89b70a7e865a33eec914"
            && i.upstream_revision == super::super::super::REVISION
            && !i.np01_complete
            && !i.np02_complete
            && !i.equivalence_eligible
            && i.remaining
                .iter()
                .map(String::as_str)
                .eq(REMAINING.iter().copied()),
        "scope/acceptance",
    )?;
    require(
        i.product_sources.len() == SOURCES.len()
            && i.product_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == SOURCES.iter().map(|(p, _)| *p).collect(),
        "product denominator",
    )?;
    for s in &i.product_sources {
        require(digest(source(&s.path)?) == s.sha256, "product source drift")?;
    }
    require(
        i.external_sources.len() == EXTERNAL.len()
            && i.external_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == EXTERNAL.iter().copied().collect(),
        "external denominator",
    )?;
    require(i.steps.len() == STEPS.len(), "step denominator")?;
    for (s, expected) in i.steps.iter().zip(STEPS) {
        require(
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}",
                s.id,
                s.classification,
                s.scalar,
                s.source,
                s.external,
                s.symbol,
                s.upstream_type,
                s.product.len(),
                s.upstream.len()
            ) == *expected,
            "step identity/type/boundary",
        )?;
        let text = source(&s.source)?;
        let kernel = between(
            text,
            if s.source.ends_with("flex.rs") {
                "const FLEX_POSITION_CUDA: &str = r#\""
            } else {
                "const FLEX_EDGE_CUDA: &str = r#\""
            },
            "\"#;",
        )?;
        for a in &s.product {
            anchor(kernel, a)?;
        }
        require(
            s.upstream.iter().all(|a| !a.trim().is_empty()),
            "empty upstream anchor",
        )?;
    }
    require(i.outputs.len() == OUTPUTS.len(), "output denominator")?;
    for (o, expected) in i.outputs.iter().zip(OUTPUTS) {
        require(
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                o.name,
                o.upstream_name,
                o.scalar,
                o.upstream_type,
                o.source,
                o.symbol,
                o.declaration,
                o.pointer,
                o.write,
                o.view,
                o.upstream_write
            ) == *expected,
            "output identity/type/route",
        )?;
        let text = source(&o.source)?;
        for a in [
            &o.pointer,
            &o.write,
            &format!("pub {}: &'a [{}]", o.name, o.scalar),
            &format!("{}: {},", o.name, o.view),
        ] {
            anchor(text, a)?;
        }
        require(
            super::super::super::upstream_scalar(&o.declaration)? == o.scalar,
            "output scalar",
        )?;
    }
    boundaries()
}
pub(super) fn boundaries() -> Check {
    for (text, anchors) in [
        (
            FLEX,
            &[
                "pub(super) values: Vec<f32>",
                "pub(super) validity: Vec<bool>",
                "let v = &self.values[r.start + 4..r.end + 4];",
            ][..],
        ),
        (
            EDGE,
            &[
                "pub(super) values: Vec<f32>",
                "let v = &self.values[r.start + 4..r.end + 4];",
                "const float* qv=qvel+4+(unsigned long long)w*nv;",
                "while(b>0) {if(b==owner) return true;b=parent[b];}return false;",
            ][..],
        ),
        (
            RESIDENT,
            &[
                "flex: Option<TransferBuffer<f32>>",
                "flex_validity: Option<TransferBuffer<i32>>",
                "values: read_output(out, layout.guarded, \"resident_flex_position_output\")?",
                "\"resident_flex_edge_output\"",
                "if value != 0 && value != 1",
                "decoded.extend(values.iter().map(|&v| v == 1));",
            ][..],
        ),
        (TRANSFER, &["target.copy_from_slice(staging.values());"][..]),
        (
            MODEL,
            &[
                "\"interpolated_vertex_has_sparse_row\"",
                "\"endpoint_dof_union_mismatch\"",
                "// Native nJfe can reserve unused tail slots when passive forces are off.",
                "if !scaled.is_finite() || scaled < i32::MIN as f32 || scaled >= 2147483648.0",
            ][..],
        ),
    ] {
        for a in anchors {
            anchor(text, a)?;
        }
    }
    // 同一世界先失效，再算节点/顶点，最后计算边。
    let pos = RESIDENT.find("\"flex_invalidate\"").ok_or("invalidation")?;
    let nodes = RESIDENT[pos..]
        .find("\"flex_positions\"")
        .ok_or("positions")?
        + pos;
    require(RESIDENT[nodes..].contains("\"flex_edges\""), "stage order")?;
    Ok(())
}
pub(super) type Texts = BTreeMap<String, String>;
pub(super) fn verify_external(i: &Inventory, texts: &Texts) -> Check {
    let get = |p: &str| texts.get(p).ok_or_else(|| format!("external: {p}"));
    for s in &i.external_sources {
        require(digest(get(&s.path)?) == s.sha256, "external drift")?;
    }
    super::super::super::verify_external(
        &super::super::super::inventory(),
        get("upstream/_src/types.py")?,
        get("warp/_src/types.py")?,
    )?;
    // 六维速度也须追到f32定义，不能只核对三维别名。
    for a in [
        "class spatial_vectorf(vector(length=6, dtype=float32)):",
        "spatial_vector = spatial_vectorf",
    ] {
        anchor(get("warp/_src/types.py")?, a)?;
    }
    for a in [
        "qvel_in: wp.array2d[float]",
        "cdof_in: wp.array2d[wp.spatial_vector]",
        "body_isdofancestor: wp.array2d[int]",
    ] {
        anchor(body(get("upstream/_src/smooth.py")?, "_flex_edges")?, a)?;
    }
    for s in &i.steps {
        let section = body(get(&s.external)?, &s.symbol)?;
        for a in &s.upstream {
            anchor(section, a)?;
        }
    }
    let smooth = get("upstream/_src/smooth.py")?;
    let data = get("upstream/_src/types.py")?
        .split_once("class Data:")
        .ok_or("Data")?
        .1;
    for o in &i.outputs {
        anchor(data, &format!("  {}: {}\n", o.upstream_name, o.declaration))?;
        let section = body(smooth, &o.symbol)?;
        anchor(section, &o.upstream_write)?;
        if o.scalar != "bool" {
            anchor(
                section,
                &format!("{}_out: wp.array2d[{}]", o.upstream_name, o.upstream_type),
            )?;
        }
    }
    for a in ["if i == 0:", "return 1.0 - s", "return s"] {
        anchor(body(get("upstream/_src/support.py")?, "_phi")?, a)?;
    }
    let edges = body(smooth, "_flex_edges")?;
    require(!edges.contains("zero_()"), "upstream pool-clear assumption")?;
    // 私有祖先遍历与上游预计算掩码分开记账。
    anchor(edges, "if body_isdofancestor[b1, dofid] != 0:")?;
    anchor(edges, "if body_isdofancestor[b2, dofid] != 0:")?;
    let flex = body(smooth, "flex")?;
    let invalidate = flex
        .find("d.flex_hessian_valid.zero_()")
        .ok_or("invalidate")?;
    let nodes = flex.find("_flex_nodes,").ok_or("nodes")?;
    let vertices = flex.find("_flex_vertices,").ok_or("vertices")?;
    let edges = flex.find("_flex_edges,").ok_or("edges")?;
    require(
        invalidate < nodes && nodes < vertices && vertices < edges,
        "upstream stage order",
    )
}
pub(super) fn verify_captures(history: &Path, current: &Path) -> Result<usize, String> {
    let original: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/precision/t4-kernels.json"
    ))
    .unwrap();
    let now: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/precision/t4-current-kernels.json"
    ))
    .unwrap();
    let entries = ["flex_invalidate", "flex_positions", "flex_edges"];
    let rows: Vec<_> = original["artifacts"]
        .as_array()
        .ok_or("artifacts")?
        .iter()
        .filter(|a| entries.contains(&a["entry"].as_str().unwrap_or("")))
        .collect();
    require(
        rows.len() == 3
            && rows
                .iter()
                .map(|a| a["entry"].as_str().unwrap())
                .collect::<BTreeSet<_>>()
                == entries.into_iter().collect(),
        "capture denominator",
    )?;
    let mut count = 0;
    for directory in [
        history.join("debug-ptx"),
        history.join("release-ptx"),
        current.to_path_buf(),
    ] {
        for row in &rows {
            require(
                now["artifacts"]
                    .as_array()
                    .ok_or("current artifacts")?
                    .iter()
                    .filter(|a| *a == *row)
                    .count()
                    == 1,
                "current lineage changed",
            )?;
            let entry = row["entry"].as_str().ok_or("entry")?;
            let hash = row["ptx_sha256"].as_str().ok_or("hash")?;
            require(
                hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                "path hash",
            )?;
            for (ext, key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let bytes = std::fs::read(directory.join(format!("{entry}-{hash}.{ext}")))
                    .map_err(|e| e.to_string())?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == row[key].as_str().ok_or("hash")?,
                    "artifact drift",
                )?;
                if ext == "cu" {
                    let text = String::from_utf8(bytes)
                        .map_err(|e| e.to_string())?
                        .replace("\r\n", "\n");
                    let (source, constant) = if entry == "flex_edges" {
                        (EDGE, "FLEX_EDGE_CUDA")
                    } else {
                        (FLEX, "FLEX_POSITION_CUDA")
                    };
                    let normalized = source.replace("\r\n", "\n");
                    require(
                        text == between(
                            &normalized,
                            &format!("const {constant}: &str = r#\""),
                            "\"#;",
                        )?,
                        "current product template",
                    )?;
                } else if ext == "json" {
                    let r: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        r["entry"] == entry
                            && r["passed"] == true
                            && r["fmad"] == true
                            && r["useFastMath"] == false
                            && r["precisionGate"] == "f32-physics",
                        "receipt options",
                    )?;
                }
            }
            count += 1;
        }
    }
    Ok(count)
}
