//! 冻结上游源码的外部核验。

use super::super::{Check, between, digest, require};
use super::types::Inventory;
use super::validate::{anchor, body};
use std::collections::BTreeMap;

pub(super) type Texts = BTreeMap<String, String>;

fn ext<'a>(texts: &'a Texts, path: &str) -> Result<&'a str, String> {
    texts
        .get(path)
        .map(String::as_str)
        .ok_or_else(|| format!("external: {path}"))
}

pub(super) fn verify_external(i: &Inventory, texts: &Texts) -> Check {
    for s in &i.external_sources {
        require(digest(ext(texts, &s.path)?) == s.sha256, "external drift")?;
    }
    let types = ext(texts, "upstream/_src/types.py")?;
    super::super::verify_external(
        &super::super::inventory(),
        types,
        ext(texts, "warp/_src/types.py")?,
    )?;
    for s in &i.steps {
        let text = ext(texts, &s.upstream_source)?;
        let section = if s.upstream_source.ends_with(".py") {
            body(text, &s.symbol)?
        } else {
            text
        };
        for a in &s.upstream {
            anchor(if a.starts_with("def ") { text } else { section }, a)?;
        }
    }
    let smooth = ext(texts, "upstream/_src/smooth.py")?;
    let data = types.split_once("class Data:").ok_or("Data class")?.1;
    for o in &i.outputs {
        anchor(data, &format!("  {}: {}\n", o.name, o.upstream_declaration))?;
        let section = body(smooth, &o.symbol)?;
        anchor(
            section,
            &format!("{}_out: wp.array2d[{}]", o.name, o.upstream_type),
        )?;
        anchor(section, &o.upstream_write)?;
    }
    // 原生头文件与注册文档相互矛盾，不能只取文档。
    let quat = ext(texts, "warp/native/quat.h")?;
    let normalize = between(quat, "quat_t<Type> normalize(const quat_t<Type>& q)", "\n}")?;
    anchor(normalize, "if (l > Type(kEps))")?;
    anchor(
        normalize,
        "return quat_t<Type>(Type(0), Type(0), Type(0), Type(1));",
    )?;
    anchor(
        quat,
        "quat_t(Type x = Type(0), Type y = Type(0), Type z = Type(0), Type w = Type(0))",
    )?;
    anchor(ext(texts, "warp/native/builtin.h")?, "#define kEps 0.0f")?;
    let builtin = ext(texts, "warp/_src/builtins.py")?;
    anchor(
        builtin,
        "\"normalize\",\n    input_types={\"a\": quaternion(dtype=Float)},",
    )?;
    anchor(builtin, "the zero quaternion is returned.")?;
    anchor(
        ext(texts, "upstream/_src/io.py")?,
        "ancestor_chain = lambda b: ancestor_chain(mjm.body_parentid[b]) + [b] if b else []",
    )?;
    anchor(
        body(smooth, "_kinematics_branch")?,
        "worldid, branchid = wp.tid()",
    )?;
    anchor(body(smooth, "kinematics")?, "dim=(d.nworld, m.nbranch)")?;
    Ok(())
}
