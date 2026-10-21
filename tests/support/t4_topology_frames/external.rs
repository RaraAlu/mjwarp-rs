//! 冻结上游源码核验。

use super::super::{Check, between, digest, require};
use super::{Inventory, contains};
use std::collections::BTreeMap;

pub(super) type ExternalText = BTreeMap<(String, String), String>;
fn external<'a>(texts: &'a ExternalText, root: &str, path: &str) -> Result<&'a str, String> {
    texts
        .get(&(root.into(), path.into()))
        .map(String::as_str)
        .ok_or_else(|| format!("external: {root}/{path}"))
}
pub(super) fn verify_external(i: &Inventory, texts: &ExternalText) -> Check {
    for s in &i.external_sources {
        require(
            digest(external(texts, &s.root, &s.path)?) == s.sha256,
            "external drift",
        )?;
    }
    let types = external(texts, "upstream", "_src/types.py")?;
    let warp = external(texts, "warp", "_src/types.py")?;
    super::super::verify_external(&super::super::inventory(), types, warp)?;
    for anchor in [
        "class mat33f(matrix(shape=(3, 3), dtype=float32)):",
        "mat33 = mat33f",
    ] {
        contains(warp, anchor)?;
    }
    let model = between(types, "class Model:", "class Contact:")?;
    for r in &i.routes {
        for u in &r.upstream {
            contains(model, &format!("  {}: {}\n", u.name, u.declaration))?;
        }
    }
    let data = types.split_once("class Data:").ok_or("Data class")?.1;
    for o in &i.outputs {
        contains(data, &format!("  {}: {}\n", o.name, o.upstream_declaration))?;
    }
    for s in &i.steps {
        let text = external(
            texts,
            if s.upstream_file.starts_with("native/") {
                "warp"
            } else {
                "upstream"
            },
            &s.upstream_file,
        )?;
        let body = if s.upstream_file.ends_with(".py") {
            text.split_once(&format!("def {}(", s.symbol))
                .ok_or("upstream symbol")?
                .1
                .split("\n\n\n")
                .next()
                .ok_or("upstream body")?
        } else {
            text
        };
        for a in &s.upstream {
            // 签名锚点包括def，单独在完整来源查找。
            contains(if a.starts_with("def ") { text } else { body }, a)?;
        }
    }
    contains(
        external(texts, "warp", "native/builtin.h")?,
        "#define kEps 0.0f",
    )?;
    contains(
        external(texts, "upstream", "_src/smooth.py")?,
        "if body_weldid[bodyid] == 0 and body_mocapid[body_rootid[bodyid]] == -1:",
    )?;
    Ok(())
}
