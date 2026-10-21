//! 产品与上游源码核验。

use super::*;

pub fn verify_sources(value: &Inventory, product: &Path, upstream: &Path) -> Result<(), String> {
    validate_inventory(value)?;
    for (root, stamps) in [
        (product, &value.product_sources),
        (upstream, &value.upstream_sources),
    ] {
        for stamp in stamps {
            let bytes = read_under(root, &stamp.path)?;
            let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
            require(
                digest(text.replace("\r\n", "\n").as_bytes()) == stamp.sha256,
                &format!("source drift: {}", stamp.path),
            )?;
        }
    }
    for item in &value.mappings {
        let product =
            String::from_utf8(read_under(product, &item.product)?).map_err(|e| e.to_string())?;
        let upstream = String::from_utf8(read_under(upstream, &item.upstream_file)?)
            .map_err(|e| e.to_string())?;
        require(
            product.contains(&format!("void {}(", item.entry)),
            "missing product entry",
        )?;
        for symbol in &item.upstream_symbols {
            require(
                upstream.contains(&format!("def {symbol}(")),
                &format!("missing upstream symbol: {symbol}"),
            )?;
        }
    }
    Ok(())
}
