//! 清单结构与映射校验。

use super::*;

pub fn validate_inventory(value: &Inventory) -> Result<(), String> {
    require(
        value.schema == 1 && value.upstream_revision == UPSTREAM,
        "unfrozen upstream/schema",
    )?;
    require(
        value.source_revision.len() == 40
            && value.source_revision.bytes().all(|c| c.is_ascii_hexdigit()),
        "invalid source revision",
    )?;
    require(
        value.scope == "t4-synchronous-physics-kernels",
        "unexpected precision scope",
    )?;
    require(
        !value.mappings.is_empty() && !value.artifacts.is_empty(),
        "empty precision evidence",
    )?;
    require(
        unique(value.mappings.iter().map(|v| v.entry.as_str())),
        "duplicate kernel mapping",
    )?;
    require(
        unique(value.product_sources.iter().map(|v| v.path.as_str())),
        "duplicate product source",
    )?;
    require(
        unique(value.upstream_sources.iter().map(|v| v.path.as_str())),
        "duplicate upstream source",
    )?;
    for stamp in value.product_sources.iter().chain(&value.upstream_sources) {
        require(valid_hash(&stamp.sha256), "invalid source hash")?;
    }
    let excluded: BTreeSet<_> = value
        .excluded_test_entries
        .iter()
        .map(String::as_str)
        .collect();
    require(
        excluded == BTreeSet::from(["adapter_probe", "fma_probe", "hessian_probe"]),
        "unexpected diagnostic exclusions",
    )?;
    for item in &value.mappings {
        require(
            !excluded.contains(item.entry.as_str()),
            "test-only entry in physical scope",
        )?;
        require(
            value.product_sources.iter().any(|s| s.path == item.product),
            "unbound product source",
        )?;
        require(
            value
                .upstream_sources
                .iter()
                .any(|s| s.path == item.upstream_file),
            "unbound upstream source",
        )?;
        require(
            !item.upstream_symbols.is_empty()
                && unique(item.upstream_symbols.iter().map(String::as_str)),
            "missing upstream symbols",
        )?;
        require(
            !item.fields.is_empty()
                && matches!(item.arithmetic.as_str(), "f32" | "integer-only")
                && !item.boundary.is_empty(),
            "incomplete type mapping",
        )?;
        let integer_only = matches!(
            item.entry.as_str(),
            "flex_invalidate" | "flex_hessian_validate" | "sleep_copy" | "tendon_indices"
        );
        require(
            (item.arithmetic == "integer-only") == integer_only,
            "arithmetic classification changed",
        )?;
    }
    let mut product_sources: BTreeSet<_> =
        value.mappings.iter().map(|v| v.product.as_str()).collect();
    product_sources.extend([
        "src/physics/resident.rs",
        "src/physics/tendon_wrap.rs",
        "src/runtime/transfer/kernel.rs",
        "src/runtime/transfer.rs",
        "fixtures/mjwarp-mass-solve/manifest.json",
    ]);
    require(
        product_sources
            == value
                .product_sources
                .iter()
                .map(|v| v.path.as_str())
                .collect(),
        "product source coverage changed",
    )?;
    let mut upstream_sources: BTreeSet<_> = value
        .mappings
        .iter()
        .map(|v| v.upstream_file.as_str())
        .collect();
    upstream_sources.extend([
        "_src/types.py",
        "_src/math.py",
        "_src/util_misc.py",
        "_src/block_cholesky.py",
    ]);
    require(
        upstream_sources
            == value
                .upstream_sources
                .iter()
                .map(|v| v.path.as_str())
                .collect(),
        "upstream source coverage changed",
    )?;
    let mapped: BTreeSet<_> = value.mappings.iter().map(|v| v.entry.as_str()).collect();
    let captured: BTreeSet<_> = value.artifacts.iter().map(|v| v.entry.as_str()).collect();
    require(mapped == captured, "kernel coverage mismatch")?;
    require(
        mapped == BTreeSet::from(ENTRIES) && value.artifacts.len() == 30,
        "frozen coverage changed",
    )?;
    let mut seen = BTreeSet::new();
    for item in &value.artifacts {
        require(
            seen.insert((&item.entry, &item.ptx_sha256)),
            "duplicate artifact",
        )?;
        require(
            [&item.source_sha256, &item.ptx_sha256, &item.receipt_sha256]
                .into_iter()
                .all(|v| valid_hash(v)),
            "invalid artifact hash",
        )?;
    }
    // 本清单只证明内核来源，不冒充完整逐字段审查。
    require(
        !value.remaining.is_empty(),
        "missing precision boundary declaration",
    )
}
