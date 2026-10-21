//! 历史捕获变体与环境审计。

use super::super::require;
use super::lineage::historical_rigid_kernel;
use super::types::kernel;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn capture_variant(text: &str) -> Result<(bool, bool), String> {
    let has = |s: &str| text.lines().any(|l| l == s);
    let batch = has("#define MJWARP_FIELD_BATCHES");
    let mocap = has("#define MJWARP_MOCAP");
    let equivalent = has("#define MJWARP_G01_EQUIVALENT");
    require(
        batch == mocap && (!equivalent || batch),
        "kernel macro combination",
    )?;
    Ok((batch, equivalent))
}

pub(super) fn verify_captures(root: &Path) -> Result<usize, String> {
    let m: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let rows: Vec<_> = m["artifacts"]
        .as_array()
        .ok_or("artifacts")?
        .iter()
        .filter(|a| a["entry"] == "rigid_kinematics")
        .collect();
    require(rows.len() == 3, "rigid variants")?;
    let mut count = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        let mut variants = BTreeSet::new();
        for a in &rows {
            let h = a["ptx_sha256"].as_str().ok_or("hash")?;
            require(
                h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()),
                "capture path",
            )?;
            let mut text = String::new();
            for (ext, key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let bytes =
                    std::fs::read(root.join(mode).join(format!("rigid_kinematics-{h}.{ext}")))
                        .map_err(|e| e.to_string())?;
                require(
                    format!("{:x}", Sha256::digest(&bytes))
                        == a[key].as_str().ok_or("artifact hash")?,
                    "artifact drift",
                )?;
                if ext == "cu" {
                    text = String::from_utf8(bytes)
                        .map_err(|e| e.to_string())?
                        .replace("\r\n", "\n");
                } else if ext == "json" {
                    let r: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        r["entry"] == "rigid_kinematics"
                            && r["passed"] == true
                            && r["precisionGate"] == "f32-physics"
                            && r["fmad"] == true
                            && r["useFastMath"] == false,
                        "precision receipt",
                    )?;
                }
            }
            super::validate::anchor(
                &text,
                &historical_rigid_kernel(kernel())?.replace("\r\n", "\n"),
            )?;
            require(
                variants.insert(capture_variant(&text)?),
                "duplicate variant",
            )?;
            count += 1;
        }
        require(
            variants
                == [(false, false), (true, false), (true, true)]
                    .into_iter()
                    .collect(),
            "active variant coverage",
        )?;
    }
    Ok(count)
}
