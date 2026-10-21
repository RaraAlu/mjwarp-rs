//! 冻结上游与 Warp 类型源码核验。

use super::{Check, Inventory, between, digest, require};

pub(crate) fn verify_external(inventory: &Inventory, upstream: &str, warp: &str) -> Check {
    require(
        digest(upstream) == inventory.upstream_types.sha256,
        "upstream source drift",
    )?;
    require(
        digest(warp) == inventory.warp_types.sha256,
        "Warp source drift",
    )?;
    let model = between(upstream, "class Model:", "class Contact:")?;
    let option = between(upstream, "class Option:", "class Statistic:")?;
    for field in &inventory.fields {
        let text = if matches!(field.name.as_str(), "enableflags" | "disableflags") {
            option
        } else {
            model
        };
        let declaration = format!("  {}: {}", field.name, field.upstream_declaration);
        require(
            text.lines().filter(|line| *line == declaration).count() == 1,
            format!("upstream declaration: {}", field.name),
        )?;
    }
    require(
        upstream.contains("arr = wp.array(ndim=len(shape), dtype=dtype)"),
        "upstream array adapter",
    )?;
    let warp = warp.replace("\r\n", "\n");
    for anchor in [
        "if dtype is int:\n            dtype = int32\n        elif dtype is float:\n            dtype = float32",
        "class vec2f(vector(length=2, dtype=float32)):",
        "class vec3f(vector(length=3, dtype=float32)):",
        "class quatf(quaternion(dtype=float32)):",
        "class mat33f(matrix(shape=(3, 3), dtype=float32)):",
        "class vec2i(vector(length=2, dtype=int32)):",
        "class vec3i(vector(length=3, dtype=int32)):",
        "vec2 = vec2f",
        "vec3 = vec3f",
        "quat = quatf",
        "mat33 = mat33f",
    ] {
        require(
            warp.contains(anchor),
            format!("Warp scalar definition: {anchor}"),
        )?;
    }
    Ok(())
}
