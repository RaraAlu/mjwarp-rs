//! 内部组与输出字段校验。

use super::routes::validate_routes;
use super::{
    Check, EXTERNAL, Inventory, OUTPUTS, PHYSICS, REMAINING, SOURCES, STEPS, contains, digest,
    require, source, template,
};
use std::collections::BTreeSet;

pub(super) fn validate_steps(i: &Inventory) -> Check {
    require(i.steps.len() == STEPS.len(), "step count")?;
    for (s, expected) in i.steps.iter().zip(STEPS) {
        require(
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}",
                s.id,
                s.file,
                s.classification,
                s.upstream_file,
                s.symbol,
                s.upstream_type,
                s.product.len(),
                s.upstream.len()
            ) == *expected
                && s.scalar == "f32",
            "step type/boundary",
        )?;
        for a in &s.product {
            contains(template(&s.file)?, a)?;
        }
        require(
            s.upstream.iter().all(|s| !s.trim().is_empty()),
            "empty upstream step",
        )?;
    }
    contains(PHYSICS, "struct V { float x,y,z; };")?;
    contains(PHYSICS, "struct Q { float w,x,y,z; };")?;
    Ok(())
}
pub(super) fn validate_outputs(i: &Inventory) -> Check {
    require(i.outputs.len() == OUTPUTS.len(), "output count")?;
    for (o, expected) in i.outputs.iter().zip(OUTPUTS) {
        require(
            format!("{}:{}:{}:{}", o.name, o.file, o.upstream_type, o.view) == *expected
                && o.scalar == "f32",
            "output type/view",
        )?;
        contains(template(&o.file)?, &o.write)?;
        contains(source(&o.file)?, &format!("pub {}: &'a [f32]", o.name))?;
        contains(source(&o.file)?, &format!("{}: {},", o.name, o.view))?;
        require(
            super::super::upstream_scalar(&o.upstream_declaration)? == "f32",
            "output upstream scalar",
        )?;
    }
    Ok(())
}
pub(super) fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1
            && i.scope == "g01-topology-and-frame-precision"
            && i.product_revision == "dcb1fae771c45ca5df823d0b2517f04724f625b2"
            && i.upstream_revision == super::super::REVISION,
        "revision/scope",
    )?;
    require(
        !i.np01_complete
            && !i.np02_complete
            && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "unaccepted boundary",
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
    for s in &i.product_sources {
        require(digest(source(&s.path)?) == s.sha256, "product source drift")?;
    }
    require(
        i.external_sources.len() == EXTERNAL.len()
            && i.external_sources
                .iter()
                .map(|s| (s.root.as_str(), s.path.as_str()))
                .collect::<BTreeSet<_>>()
                == EXTERNAL.iter().copied().collect(),
        "external coverage",
    )?;
    validate_routes(i)?;
    validate_steps(i)?;
    validate_outputs(i)
}
