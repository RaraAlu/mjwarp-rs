//! 磁盘载荷的编解码与原子写入。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::Ordering;

use super::*;

fn encode(artifact: &Artifact) -> Result<Vec<u8>, LauncherError> {
    let payload = Payload {
        key: artifact.key.clone(),
        ptx: artifact.ptx.clone(),
        ptx_sha256: artifact.ptx_sha256.clone(),
    };
    let canonical = serde_json::to_vec(&payload).map_err(|e| cache_error("payload-json", e))?;
    serde_json::to_vec(&Archive {
        format: FORMAT.into(),
        payload_sha256: digest(&canonical),
        payload,
    })
    .map_err(|e| cache_error("archive-json", e))
}

pub(super) fn decode(bytes: &[u8], expected: &ArtifactKey) -> Result<Payload, LauncherError> {
    let archive: Archive =
        serde_json::from_slice(bytes).map_err(|e| cache_error("archive-json", e))?;
    if archive.format != FORMAT {
        return Err(cache_error("format", &archive.format));
    }
    let canonical =
        serde_json::to_vec(&archive.payload).map_err(|e| cache_error("payload-json", e))?;
    if digest(&canonical) != archive.payload_sha256 {
        return Err(cache_error("checksum", "载荷摘要不匹配"));
    }
    let payload = archive.payload;
    if payload.key != *expected {
        return Err(cache_error("key-mismatch", "磁盘产物键与请求不符"));
    }
    if digest(payload.ptx.as_bytes()) != payload.ptx_sha256 {
        return Err(cache_error("ptx-digest", "PTX摘要不匹配"));
    }
    Ok(payload)
}

pub(super) fn write_disk(directory: &Path, artifact: &Artifact) -> Result<(), LauncherError> {
    let bytes = encode(artifact)?;
    fs::create_dir_all(directory).map_err(|e| cache_error("mkdir", e))?;
    let path = archive_path(directory, &artifact.key_sha256);
    let temp = directory.join(format!(
        ".{}-{}-{}.tmp",
        artifact.key_sha256,
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| cache_error("temp-create", e))?;
    let result = (|| {
        file.write_all(&bytes)
            .map_err(|e| cache_error("write", e))?;
        file.sync_all().map_err(|e| cache_error("sync", e))?;
        drop(file);
        // 同目录替换不先删除旧产物。
        fs::rename(&temp, &path).map_err(|e| cache_error("rename", e))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
