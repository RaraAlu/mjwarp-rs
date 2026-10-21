//! 内部组、输出字段与边界校验。

use super::super::{Check, digest, require};
use super::types::{Inventory, kernel};
use super::{
    BOUNDARIES, EXTERNAL, OUTPUTS, PHYSICS, REMAINING, RESIDENT, SOURCES, STEPS, TRANSFER,
};
use std::collections::BTreeSet;

pub(super) fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, s)| (*p == path).then_some(*s))
        .ok_or_else(|| format!("source: {path}"))
}

pub(super) fn anchor(text: &str, expected: &str) -> Check {
    require(
        !expected.trim().is_empty() && text.contains(expected),
        format!("anchor: {expected}"),
    )
}

pub(super) fn body<'a>(text: &'a str, symbol: &str) -> Result<&'a str, String> {
    text.split_once(&format!("def {symbol}("))
        .map(|(_, s)| s.split("\n\n\n").next().unwrap())
        .ok_or_else(|| format!("symbol: {symbol}"))
}

pub(super) fn validate_steps(i: &Inventory, text: &str) -> Check {
    require(i.steps.len() == STEPS.len(), "internal coverage")?;
    for (s, expected) in i.steps.iter().zip(STEPS) {
        require(
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                s.id,
                s.classification,
                s.upstream_source,
                s.symbol,
                s.upstream_type,
                s.product.len(),
                s.upstream.len()
            ) == *expected
                && s.scalar == "f32",
            "internal identity/type/boundary",
        )?;
        for a in &s.product {
            anchor(text, a)?;
        }
        require(
            s.upstream.iter().all(|a| !a.trim().is_empty()),
            "empty upstream anchor",
        )?;
    }
    for a in [
        "struct V { float x,y,z; };",
        "struct Q { float w,x,y,z; };",
        "__device__ V add(V a,V b) { return {a.x+b.x,a.y+b.y,a.z+b.z}; }",
        "__device__ V scale(V a,float s) { return {a.x*s,a.y*s,a.z*s}; }",
    ] {
        anchor(text, a)?;
    }
    Ok(())
}

pub(super) fn validate_outputs(i: &Inventory) -> Check {
    require(i.outputs.len() == OUTPUTS.len(), "output coverage")?;
    for (o, expected) in i.outputs.iter().zip(OUTPUTS) {
        require(
            format!(
                "{}|{}|{}|{}|{}|{}",
                o.name, o.upstream_type, o.pointer, o.write, o.view, o.symbol
            ) == *expected
                && o.scalar == "f32",
            "output identity/type/view",
        )?;
        anchor(kernel(), &o.pointer)?;
        anchor(kernel(), &o.write)?;
        anchor(PHYSICS, &format!("pub {}: &'a [f32]", o.name))?;
        anchor(PHYSICS, &format!("{}: {},", o.name, o.view))?;
        require(
            super::super::upstream_scalar(&o.upstream_declaration)? == "f32",
            "output upstream type",
        )?;
        require(!o.upstream_write.trim().is_empty(), "empty output writer")?;
    }
    for a in [
        "let stride = packed(&[(nbody, 28), (njnt, 6)])?;",
        "let v = &self.values[r.start + 4..r.end + 4];",
        "__device__ void store_q(float* p,Q q) { p[0]=q.w;p[1]=q.x;p[2]=q.y;p[3]=q.z; }",
    ] {
        anchor(PHYSICS, a)?;
    }
    anchor(RESIDENT, "rigid: TransferBuffer<f32>")?;
    anchor(
        RESIDENT,
        "let rigid = read_output(&self.rigid, lengths[0], \"resident_rigid_output\")?;",
    )?;
    let read = super::super::between(RESIDENT, "fn read_output<", "\n}")?;
    for a in [
        "Result<Vec<T>, TransferError>",
        "host_staging::<T>(length)",
        "buffer.read_range_into(0, &mut values)?;",
        "Ok(values)",
    ] {
        anchor(read, a)?;
    }
    anchor(TRANSFER, "target.copy_from_slice(staging.values());")?;
    // f64仅用于只读有限值诊断，不回写物理结果。
    let check = super::super::between(PHYSICS, "fn check_device_values<", "\n}")?;
    anchor(check, "values: &[T]")?;
    anchor(check, "if !(*value).into().is_finite()")?;
    Ok(())
}

pub(super) fn validate_boundaries(i: &Inventory, text: &str) -> Check {
    let normalized = text.replace("\r\n", "\n");
    let text = normalized.as_str();
    require(
        i.boundaries.iter().map(String::as_str).eq(BOUNDARIES),
        "scope boundaries",
    )?;
    let z = &i.zero_quaternion;
    require(
        z.status == "matched-generated-artifact"
            && z.product_return == [0, 0, 0, 1]
            && z.warp_header_return == [0, 0, 0, 1]
            && z.warp_doc_claim == "zero quaternion"
            && z.upstream_generated_verified
            && !i.equivalence_eligible,
        "zero quaternion evidence boundary",
    )?;
    anchor(
        text,
        "#ifdef MJWARP_G01_EQUIVALENT\n  if(n==0.0f) return {0,0,0,1};\n#endif",
    )?;
    anchor(text, "if(!b) { p=load_v(xp); r=load_q(xq); }")?;
    anchor(text, "for(unsigned b=0;b<nb;++b)")?;
    anchor(RESIDENT, "\"#define MJWARP_G01_EQUIVALENT\\n\"")?;
    anchor(RESIDENT, "data.rigid.write_range(start, &[0.0; 3])?;")?;
    // 完整入口仅检查长度与有限值，不排除零四元数。
    let check = super::super::between(RESIDENT, "fn check_g01_qpos(", "\n}")?;
    anchor(
        check,
        "check_finite_field(\"kinematics_qpos\", expected, qpos)?;",
    )?;
    require(
        !check.contains("norm_squared"),
        "zero input boundary changed",
    )?;
    Ok(())
}

pub(super) fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1
            && i.scope == "g01-rigid-internal-and-output"
            && i.product_revision == "dcb1fae771c45ca5df823d0b2517f04724f625b2"
            && i.upstream_revision == super::super::REVISION,
        "scope/revision",
    )?;
    require(
        !i.np01_complete
            && !i.np02_complete
            && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "unaccepted scope",
    )?;
    require(
        i.product_sources.len() == SOURCES.len()
            && i.product_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == SOURCES.iter().map(|(p, _)| *p).collect(),
        "product coverage",
    )?;
    for s in &i.product_sources {
        require(digest(source(&s.path)?) == s.sha256, "product drift")?;
    }
    require(
        i.external_sources.len() == EXTERNAL.len()
            && i.external_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == EXTERNAL.iter().copied().collect(),
        "external coverage",
    )?;
    validate_steps(i, kernel())?;
    validate_outputs(i)?;
    validate_boundaries(i, kernel())
}
