//! 拓扑路由与宿主读写顺序校验。

use super::{
    Check, Inventory, MOCAP, RESIDENT, ROUTES, between, compact, contains, reader, require, writer,
};
use std::collections::BTreeMap;

pub(super) fn validate_routes(i: &Inventory) -> Check {
    require(i.routes.len() == ROUTES.len(), "route count")?;
    let mut cursors = BTreeMap::new();
    for (r, expected) in i.routes.iter().zip(ROUTES) {
        require(
            format!(
                "{}:{}:{}:{}",
                r.group,
                r.id,
                r.kind,
                r.upstream
                    .iter()
                    .map(|u| u.name.as_str())
                    .collect::<Vec<_>>()
                    .join("+")
            ) == *expected
                && r.scalar == "i32",
            "route identity/type",
        )?;
        let host = compact(writer(&r.group)?);
        let write = compact(&r.write);
        require(!write.is_empty(), "empty writer")?;
        let cursor = cursors.entry(r.group.as_str()).or_insert(0);
        let index = host[*cursor..].find(&write).ok_or("writer order")? + *cursor;
        *cursor = index + write.len();
        contains(reader(&r.group)?, &r.read)?;
        for u in &r.upstream {
            require(
                super::super::upstream_scalar(&u.declaration)? == "i32",
                "integer source",
            )?;
        }
    }
    // 普通字段复制与私有编码分别守卫。
    let pack = between(RESIDENT, "fn pack_model(", "// Descriptors hold")?;
    for anchor in [
        "host_staging::<i32>(metadata_len)",
        "for t in topology {",
        "metadata[cursor..cursor + t.len()].copy_from_slice(t);",
        "cursor += t.len();",
    ] {
        contains(pack, anchor)?;
    }
    contains(
        MOCAP,
        "let nmocap = body_mocapid.iter().filter(|&&id| id >= 0).count();",
    )?;
    contains(
        &compact(MOCAP),
        "static_body[body]=static_body[f.body_parentid[body]asusize]&&f.body_jntnum[body]==0&&id==-1;",
    )?;
    contains(MOCAP, ".map(|&body| static_body[body as usize])")?;
    contains(RESIDENT, "static_geom.into_iter().map(i32::from).collect()")?;
    contains(
        reader("attached")?,
        "const int* static_geom=desc; desc+=ng;",
    )?;
    contains(reader("rigid")?, "desc=mocap+nb;")?;
    Ok(())
}
