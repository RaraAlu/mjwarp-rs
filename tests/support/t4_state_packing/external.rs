//! 上游类型声明与设备读取器核验。

use super::*;

pub(super) fn verify_upstream(i: &Inventory, text: &str) -> Check {
    let input = super::super::inventory();
    require(
        digest(text) == input.upstream_types.sha256,
        "upstream type drift",
    )?;
    for s in &i.states {
        for u in &s.upstream {
            let body = match u.owner.as_str() {
                "Model" => between(text, "class Model:", "class Contact:")?,
                "Data" => between(text, "class Data:", "class InverseContext:")?,
                _ => return Err("unreviewed upstream owner".into()),
            };
            let line = format!("  {}: {}", u.name, u.declaration);
            require(
                body.lines().filter(|l| *l == line).count() == 1,
                format!("upstream declaration: {}", u.name),
            )?;
        }
    }
    for anchor in [
        "class vec10f(wp.types.vector(length=10, dtype=float)):",
        "vec10 = vec10f",
    ] {
        require(text.contains(anchor), "upstream inertia type")?;
    }
    Ok(())
}

pub(super) fn reader_anchors(entry: &str) -> &'static [&'static str] {
    match entry {
        "tendon_wake" => &[
            "if(!m[6]||nt==0) return;",
            "const int* limited=num+nt;",
            "any_awake&&limited[t]",
        ],
        "flex_positions" => &["bool centered", "row[8]!=0", "float* result"],
        "flex_invalidate" => &["int* result", "flags[f]=0;"],
        "flex_hessian" => &["const int* valid", "if(flags[f]==1) continue;"],
        _ => &[],
    }
}

pub(super) fn verify_reader(entry: &str, text: &str) -> Check {
    require(
        !reader_anchors(entry).is_empty(),
        "unreviewed device reader",
    )?;
    for anchor in reader_anchors(entry) {
        require(
            text.contains(anchor),
            format!("device reader: {entry}: {anchor}"),
        )?;
    }
    Ok(())
}
