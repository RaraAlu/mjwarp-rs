//! 静态输入清单整体校验。

use super::fields::verify_field;
use super::{
    Check, Inventory, REMAINING, REVISION, SOURCES, digest, input_fields, require, source,
};
use std::collections::BTreeSet;

pub(super) fn validate(inventory: &Inventory) -> Check {
    require(
        inventory.schema == 1 && inventory.scope == "g01-native-model-ingress-only",
        "schema/scope",
    )?;
    require(
        inventory.upstream_revision == REVISION && inventory.warp_version == "1.15.0",
        "upstream version",
    )?;
    require(
        inventory.product_revision == "62b92e88100d20939c1a5c0a96f5f5f20626e3d2",
        "product baseline",
    )?;
    require(
        !inventory.np01_complete && inventory.remaining.iter().map(String::as_str).eq(REMAINING),
        "incomplete boundary",
    )?;
    let paths: BTreeSet<_> = inventory
        .product_sources
        .iter()
        .map(|s| s.path.as_str())
        .collect();
    require(
        paths == SOURCES.iter().map(|(p, _)| *p).collect()
            && paths.len() == inventory.product_sources.len(),
        "source coverage",
    )?;
    for stamp in &inventory.product_sources {
        require(
            digest(source(&stamp.path)?) == stamp.sha256,
            format!("source drift: {}", stamp.path),
        )?;
    }
    let kernels: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    require(
        inventory.upstream_types.path == "_src/types.py"
            && inventory.warp_types.path == "_src/types.py",
        "external path",
    )?;
    let frozen = kernels["upstream_sources"]
        .as_array()
        .ok_or("kernel inventory")?
        .iter()
        .find(|s| s["path"] == "_src/types.py")
        .ok_or("frozen types")?;
    require(
        frozen["sha256"] == inventory.upstream_types.sha256,
        "upstream source baseline",
    )?;
    require(
        inventory.warp_types.sha256.len() == 64
            && inventory
                .warp_types
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit()),
        "Warp digest",
    )?;
    let expected = input_fields()?;
    require(inventory.fields.len() == expected.len(), "field coverage")?;
    let mut seen = BTreeSet::new();
    for (index, (field, (name, family, abi))) in inventory.fields.iter().zip(expected).enumerate() {
        require(seen.insert(&field.name), "duplicate field")?;
        require(
            (
                field.name.as_str(),
                field.family.as_str(),
                field.native_scalar.as_str(),
            ) == (name.as_str(), family.as_str(), abi.as_str()),
            "field identity/ABI",
        )?;
        verify_field(field, (index >= 33).then(|| index - 33))?;
    }
    let g01 = source("src/io/g01.rs")?;
    for anchor in [
        "Num(Vec<f64>)",
        "Int(Vec<i32>)",
        "Bool(Vec<u8>)",
        "convert(FIELDS[id].0, v)",
        "0 => Ok(false)",
        "1 => Ok(true)",
        "sleep_enabled: enable[0] & (1 << 4) != 0",
        "island_disabled: disable[0] & (1 << 18) != 0",
    ] {
        require(g01.contains(anchor), format!("G01 conversion: {anchor}"))?;
    }
    require(
        source("src/io/fields.rs")?.contains("*output = value as f32;"),
        "quantization",
    )?;
    Ok(())
}
