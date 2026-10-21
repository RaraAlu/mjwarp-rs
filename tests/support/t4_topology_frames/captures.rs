//! 历史捕获核验。

use super::super::require;
use super::{contains, reader};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn verify_captures(root: &Path) -> Result<(usize, usize), String> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let artifacts = manifest["artifacts"].as_array().ok_or("artifacts")?;
    let selected = [
        ("rigid_kinematics", "rigid", 3),
        ("rigid_com_position", "com", 2),
        ("attached_kinematics", "attached", 3),
        ("initialize_attached", "attached", 2),
        ("camlight_rigid", "camlight", 1),
        ("camlight_com", "camlight", 1),
    ];
    let mut total = 0;
    let mut resident = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        for (entry, group, count) in selected {
            let matches: Vec<_> = artifacts.iter().filter(|a| a["entry"] == entry).collect();
            require(matches.len() == count, "capture coverage")?;
            let mut hashes = BTreeSet::new();
            let mut active = 0;
            for a in matches {
                let hash = a["ptx_sha256"].as_str().ok_or("hash")?;
                require(
                    hash.len() == 64
                        && hash.bytes().all(|b| b.is_ascii_hexdigit())
                        && hashes.insert(hash),
                    "capture path/duplicate",
                )?;
                let mut text = String::new();
                for (ext, key) in [
                    ("cu", "source_sha256"),
                    ("ptx", "ptx_sha256"),
                    ("json", "receipt_sha256"),
                ] {
                    let bytes =
                        std::fs::read(root.join(mode).join(format!("{entry}-{hash}.{ext}")))
                            .map_err(|e| e.to_string())?;
                    require(
                        format!("{:x}", Sha256::digest(&bytes))
                            == a[key].as_str().ok_or("artifact hash")?,
                        "capture drift",
                    )?;
                    if ext == "cu" {
                        text = String::from_utf8(bytes)
                            .map_err(|e| e.to_string())?
                            .replace("\r\n", "\n");
                    } else if ext == "json" {
                        let r: serde_json::Value =
                            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                        require(
                            r["entry"] == entry
                                && r["passed"] == true
                                && r["precisionGate"] == "f32-physics"
                                && r["fmad"] == true
                                && r["useFastMath"] == false,
                            "precision receipt",
                        )?;
                    }
                }
                let current = reader(group)?.replace("\r\n", "\n");
                // 旧刚体产物只核对历史分支；新产物另记定向实证。
                let captured = if group == "rigid" {
                    super::super::rigid_internal::historical_rigid_kernel(&current)?
                } else {
                    current
                };
                contains(&text, &captured)?;
                let uses_resident = text.contains("#define MJWARP_FIELD_BATCHES")
                    && (group == "com"
                        || group == "camlight"
                        || text.contains("#define MJWARP_MOCAP"));
                if uses_resident {
                    active += 1;
                    resident += 1;
                }
                total += 1;
            }
            require(active > 0, "missing active resident topology")?;
        }
    }
    Ok((total, resident))
}
