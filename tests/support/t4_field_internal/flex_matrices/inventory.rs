use super::checks::*;
use super::*;

pub(super) fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}
pub(super) fn anchor(text: &str, needle: &str) -> Check {
    require(
        !needle.trim().is_empty() && text.contains(needle),
        format!("anchor: {needle}"),
    )
}
pub(super) fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, t)| (*p == path).then_some(*t))
        .ok_or_else(|| path.into())
}
pub(super) fn body<'a>(text: &'a str, symbol: &str) -> Result<&'a str, String> {
    if symbol.is_empty() {
        return Ok(text);
    }
    text.split_once(&format!("def {symbol}("))
        .map(|(_, s)| s.split("\n\n\n").next().unwrap())
        .ok_or_else(|| format!("symbol: {symbol}"))
}
pub(super) fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1
            && i.scope == "flex-face-hessian-internal"
            && i.product_revision == "bdcc9d97482d629c40bcabfa52c0c159590bcd4d"
            && i.upstream_revision == super::super::super::REVISION
            && !i.np01_complete
            && !i.np02_complete
            && !i.equivalence_eligible
            && i.remaining
                .iter()
                .map(String::as_str)
                .eq(REMAINING.iter().copied()),
        "boundary",
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
        let text = if s.path == "src/physics/flex_face.rs" {
            super::super::face_lineage::historical(source(&s.path)?)?
        } else {
            source(&s.path)?.to_owned()
        };
        require(digest(&text) == s.sha256, "product drift")?;
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
    require(i.groups.len() == GROUPS.len(), "group denominator")?;
    for (g, expected) in i.groups.iter().zip(GROUPS) {
        require(
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}",
                g.id,
                g.classification,
                g.scalar,
                g.source,
                g.external,
                g.symbol,
                g.product.len(),
                g.upstream.len()
            ) == *expected,
            "group type/identity",
        )?;
        for a in &g.product {
            let text = if g.source == "src/physics/flex_face.rs" {
                super::super::face_lineage::historical(source(&g.source)?)?
            } else {
                source(&g.source)?.to_owned()
            };
            anchor(&text, a)?;
        }
        require(
            g.upstream.iter().all(|s| !s.trim().is_empty()),
            "empty upstream anchors",
        )?;
    }
    require(i.outputs.len() == OUTPUTS.len(), "output denominator")?;
    for (o, expected) in i.outputs.iter().zip(OUTPUTS) {
        require(
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                o.name,
                o.scalar,
                o.source,
                o.declaration,
                o.pointer,
                o.write,
                o.view,
                o.external,
                o.symbol,
                o.upstream_write
            ) == *expected,
            "output route",
        )?;
        for a in [
            &o.pointer,
            &o.write,
            &format!("pub {}: &'a [f32]", o.name),
            &format!("{}: {},", o.name, o.view),
        ] {
            anchor(source(&o.source)?, a)?;
        }
    }
    boundaries()
}
