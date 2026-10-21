//! P1产物格式与完整性检查。
//! 本模块不认证外部GPU代码。

use super::{
    ArtifactReport, ArtifactStatus, BLOCK_THREADS, ProbeBackend, ProbeConfig, ProbeKernel,
};
use crate::diagnostics::ProbeError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) const CUBECL_REVISION: &str = "1f73b9f63de50a17398c1d5278e2a5f11612c7e1";
const MAX_ARCHIVE_BYTES: usize = 8 * 1024 * 1024;
const MAX_PTX_BYTES: usize = 2 * 1024 * 1024;
const RECIPE: &str = "p1-v1;checked;u32;block=128;cpp17;fmad=false;fastmath=false;nvrtc=12.8;llvm-grid-constants=false;native-ptx=6.0-sm70";
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub(super) fn error(stage: &'static str, detail: impl ToString) -> ProbeError {
    ProbeError::Artifact {
        stage,
        detail: detail.to_string(),
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Key {
    schema: u32,
    backend: String,
    kernel: String,
    source_dependencies_sha256: String,
    recipe: String,
    sm: (i32, i32),
    driver_api: i32,
    os: String,
    arch: String,
}

impl Key {
    pub fn new(config: ProbeConfig, sm: (i32, i32), driver_api: i32) -> Self {
        // 长度前缀分隔各输入。
        // 锁文件覆盖前端依赖版本。
        let mut sources = Sha256::new();
        for source in [
            include_str!("mod.rs"),
            include_str!("cubecl.rs"),
            include_str!("cuda.rs"),
            include_str!("cuda/artifacts.rs"),
            include_str!("probe.ptx"),
            include_str!("cache.rs"),
            include_str!("../../Cargo.toml"),
            include_str!("../../Cargo.lock"),
        ] {
            sources.update((source.len() as u64).to_le_bytes());
            sources.update(source.as_bytes());
        }
        Self {
            schema: 1,
            backend: config.backend.name().into(),
            kernel: config.kernel.name().into(),
            source_dependencies_sha256: format!("{:x}", sources.finalize()),
            recipe: RECIPE.into(),
            sm,
            driver_api,
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
        }
    }

    pub fn hash(&self) -> Result<String, ProbeError> {
        Ok(digest(
            &serde_json::to_vec(self).map_err(|e| error("key-json", e))?,
        ))
    }

    fn path(&self, directory: &Path) -> Result<PathBuf, ProbeError> {
        Ok(directory.join(format!("{}.json", self.hash()?)))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Stage {
    pub entrypoint: String,
    pub abi: String,
    pub block_threads: u32,
    pub shared_memory: u32,
    pub ptx: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Payload {
    pub key: Key,
    pub stages: Vec<Stage>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Archive {
    payload: Payload,
    payload_sha256: String,
}

pub(super) fn entrypoints(config: ProbeConfig) -> Vec<&'static str> {
    if config.backend == ProbeBackend::NativePtx {
        vec!["affine_probe"]
    } else if config.kernel == ProbeKernel::GlobalScan {
        vec![
            config.kernel.entrypoint(),
            "cubecl_scan_totals_probe",
            "cubecl_scan_offsets_probe",
        ]
    } else {
        vec![config.kernel.entrypoint()]
    }
}

pub(super) fn abi(backend: ProbeBackend) -> &'static str {
    match backend {
        ProbeBackend::NativePtx => "ptr64/ptr64/u32",
        _ => "ptr64/ptr64/ptr64-meta-u32x2",
    }
}

impl Payload {
    fn validate(&self, expected: &Key, config: ProbeConfig) -> Result<(), ProbeError> {
        if &self.key != expected {
            return Err(error("key-mismatch", "源码、配方或设备键不匹配"));
        }
        let names = entrypoints(config);
        if self.stages.len() != names.len() {
            return Err(error("stage-count", "缺少或多出内核阶段"));
        }
        for (stage, expected_name) in self.stages.iter().zip(names) {
            if stage.entrypoint != expected_name {
                return Err(error("entrypoint", "重复、乱序或未知入口"));
            }
            if stage.abi != abi(config.backend) || stage.block_threads != BLOCK_THREADS as u32 {
                return Err(error("abi", "参数或线程布局不匹配"));
            }
            if stage.ptx.is_empty() || stage.ptx.len() > MAX_PTX_BYTES || stage.ptx.contains('\0') {
                return Err(error("ptx-size", "PTX为空、过大或含NUL"));
            }
            if stage.shared_memory > 128 * 1024 {
                return Err(error("shared-memory", "共享区超出探针上限"));
            }
            if config.backend == ProbeBackend::NativePtx && stage.shared_memory != 0 {
                return Err(error("shared-memory", "原生探针不使用共享区"));
            }
        }
        debug_assert_eq!(BLOCK_THREADS, 128);
        Ok(())
    }

    fn encode(&self) -> Result<Vec<u8>, ProbeError> {
        let bytes = serde_json::to_vec(self).map_err(|e| error("payload-json", e))?;
        let archive = Archive {
            payload: self.clone(),
            payload_sha256: digest(&bytes),
        };
        let bytes = serde_json::to_vec(&archive).map_err(|e| error("archive-json", e))?;
        if bytes.len() > MAX_ARCHIVE_BYTES {
            return Err(error("archive-size", "产物超过8MiB"));
        }
        Ok(bytes)
    }
}

fn decode(bytes: &[u8], key: &Key, config: ProbeConfig) -> Result<Payload, ProbeError> {
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err(error("archive-size", "产物超过8MiB"));
    }
    let archive: Archive = serde_json::from_slice(bytes).map_err(|e| error("archive-json", e))?;
    let canonical = serde_json::to_vec(&archive.payload).map_err(|e| error("payload-json", e))?;
    if digest(&canonical) != archive.payload_sha256 {
        return Err(error("checksum", "产物摘要不匹配"));
    }
    archive.payload.validate(key, config)?;
    Ok(archive.payload)
}

pub(super) fn read(
    directory: &Path,
    key: &Key,
    config: ProbeConfig,
) -> Result<Option<Payload>, ProbeError> {
    let path = key.path(directory)?;
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(error("read", format!("{}: {e}", path.display()))),
    };
    if file.metadata().map_err(|e| error("metadata", e))?.len() > MAX_ARCHIVE_BYTES as u64 {
        return Err(error("archive-size", "产物超过8MiB"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_ARCHIVE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| error("read", e))?;
    decode(&bytes, key, config).map(Some)
}

fn write(directory: &Path, payload: &Payload) -> Result<(), ProbeError> {
    let bytes = payload.encode()?;
    fs::create_dir_all(directory).map_err(|e| error("mkdir", e))?;
    let path = payload.key.path(directory)?;
    let temp = directory.join(format!(
        ".{}-{}-{}.tmp",
        payload.key.hash()?,
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| error("temp-create", e))?;
    let result = (|| {
        file.write_all(&bytes).map_err(|e| error("write", e))?;
        file.sync_all().map_err(|e| error("sync", e))?;
        drop(file);
        // 同目录替换不先删除旧产物。
        // 本接口不承诺断电持久性。
        fs::rename(&temp, &path).map_err(|e| error("rename", e))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

pub(super) fn report(
    directory: &Path,
    payload: &Payload,
    status: ArtifactStatus,
) -> Result<ArtifactReport, ProbeError> {
    Ok(ArtifactReport {
        path: payload.key.path(directory)?,
        key_sha256: payload.key.hash()?,
        stages: payload.stages.len(),
        status,
    })
}

pub(super) fn build(
    directory: &Path,
    key: Key,
    config: ProbeConfig,
    refresh: bool,
    compile: impl FnOnce() -> Result<Vec<Stage>, ProbeError>,
) -> Result<ArtifactReport, ProbeError> {
    if !refresh && let Some(payload) = read(directory, &key, config)? {
        return report(directory, &payload, ArtifactStatus::Hit);
    }
    let payload = Payload {
        key,
        stages: compile()?,
    };
    payload.validate(&payload.key, config)?;
    write(directory, &payload)?;
    report(directory, &payload, ArtifactStatus::Compiled)
}

#[cfg(test)]
mod tests;
