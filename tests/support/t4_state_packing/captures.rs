//! 历史捕获核验。

use super::external::{reader_anchors, verify_reader};
use super::*;
use std::path::Path;

pub(super) fn verify_captures(root: &Path) -> Result<usize, String> {
    let kernels: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let artifacts = kernels["artifacts"].as_array().ok_or("capture inventory")?;
    let mut total = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        let mut entries = BTreeSet::new();
        for artifact in artifacts {
            let entry = artifact["entry"].as_str().ok_or("capture entry")?;
            if reader_anchors(entry).is_empty() {
                continue;
            }
            let hash = artifact["source_sha256"].as_str().ok_or("capture hash")?;
            let ptx_hash = artifact["ptx_sha256"].as_str().ok_or("PTX hash")?;
            require(
                [hash, ptx_hash]
                    .iter()
                    .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
                "capture path hash",
            )?;
            // 三类产物统一采用PTX哈希命名。
            // CUDA源码仍按源码哈希核验。
            let stem = format!("{entry}-{ptx_hash}");
            for (extension, expected) in [
                ("ptx", ptx_hash),
                (
                    "json",
                    artifact["receipt_sha256"].as_str().ok_or("receipt hash")?,
                ),
            ] {
                let bytes = std::fs::read(root.join(mode).join(format!("{stem}.{extension}")))
                    .map_err(|e| e.to_string())?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == expected,
                    "capture companion drift",
                )?;
            }
            let path = root.join(mode).join(format!("{stem}.cu"));
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            require(
                format!("{:x}", Sha256::digest(&bytes)) == hash,
                "captured source drift",
            )?;
            let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
            verify_reader(entry, text)?;
            entries.insert(entry);
            total += 1;
        }
        require(entries.len() == 4, "reader capture coverage")?;
    }
    Ok(total)
}
