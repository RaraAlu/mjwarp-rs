//! W03：编译产物缓存。
//! 产物键覆盖源码、头文件、配方、目标与NVRTC版本。
//! 产物不绑定上下文；同键并发只编译一次。
//! 磁盘层沿用原子写入与损坏处理，载荷格式另定。
//! 磁盘载入后重新核验源码键、PTX摘要与精度。

mod cache;
mod disk;
mod nvrtc;
#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{CompileRecipe, LauncherError, PrecisionGate, RecipePurpose};
use crate::runtime::transfer::kernel::check_physics_ptx;

pub use cache::ArtifactCache;

const KEY_SCHEMA: u32 = 1;
const FORMAT: &str = "mjwarp-launcher-artifact-v1";
const MAX_TEXT_BYTES: usize = 8 << 20;
const MAX_ARCHIVE_BYTES: u64 = 24 << 20;
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // 锁内只改映射，不执行编译；中毒时映射仍完整。
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

fn valid_text(text: &str) -> bool {
    !text.is_empty() && text.len() <= MAX_TEXT_BYTES && !text.contains('\0')
}

/// 随源码提交的头文件；内容参与产物键。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderFile {
    name: String,
    content: String,
}

impl HeaderFile {
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Result<Self, LauncherError> {
        let (name, content) = (name.into(), content.into());
        if !valid_name(&name) {
            return Err(LauncherError::InvalidSource("头文件名非法"));
        }
        if !valid_text(&content) {
            return Err(LauncherError::InvalidSource("头文件内容为空、过大或含NUL"));
        }
        Ok(Self { name, content })
    }
}

/// NVRTC编译来源；`name`同时作程序名。
#[derive(Clone, Debug)]
pub struct NvrtcSource {
    name: &'static str,
    source: Cow<'static, str>,
    headers: Vec<HeaderFile>,
}

impl NvrtcSource {
    pub fn new(
        name: &'static str,
        source: impl Into<Cow<'static, str>>,
    ) -> Result<Self, LauncherError> {
        let source = source.into();
        if !valid_name(name) {
            return Err(LauncherError::InvalidSource("程序名非法"));
        }
        if !valid_text(&source) {
            return Err(LauncherError::InvalidSource("源码为空、过大或含NUL"));
        }
        Ok(Self {
            name,
            source,
            headers: Vec::new(),
        })
    }

    pub fn with_header(mut self, header: HeaderFile) -> Result<Self, LauncherError> {
        if self.headers.iter().any(|h| h.name == header.name) {
            return Err(LauncherError::InvalidSource("头文件重名"));
        }
        self.headers.push(header);
        Ok(self)
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HeaderDigest {
    name: String,
    sha256: String,
}

/// 产物键；任一字段变化都换键。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactKey {
    schema: u32,
    program: String,
    source_sha256: String,
    headers: Vec<HeaderDigest>,
    recipe: String,
    arch: String,
    nvrtc: [i32; 2],
}

impl ArtifactKey {
    /// `nvrtc`取实际加载库的主次版本。
    pub fn new(source: &NvrtcSource, recipe: &CompileRecipe, nvrtc: (i32, i32)) -> Self {
        Self {
            schema: KEY_SCHEMA,
            program: source.name.to_owned(),
            source_sha256: digest(source.source.as_bytes()),
            headers: source
                .headers
                .iter()
                .map(|h| HeaderDigest {
                    name: h.name.clone(),
                    sha256: digest(h.content.as_bytes()),
                })
                .collect(),
            recipe: recipe.fingerprint(),
            arch: recipe.arch().flag(),
            nvrtc: [nvrtc.0, nvrtc.1],
        }
    }

    /// 键摘要；按固定字段顺序序列化。
    pub fn sha256(&self) -> String {
        digest(&serde_json::to_vec(self).expect("artifact key serializes"))
    }

    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }

    pub fn recipe(&self) -> &str {
        &self.recipe
    }

    pub fn nvrtc(&self) -> (i32, i32) {
        (self.nvrtc[0], self.nvrtc[1])
    }
}

/// 已核验的编译产物。
#[derive(Debug)]
pub struct Artifact {
    key: ArtifactKey,
    key_sha256: String,
    purpose: RecipePurpose,
    ptx: String,
    ptx_sha256: String,
}

impl Artifact {
    /// L2冻结PTX产物：摘要绑定L2既有常量。
    /// 它不进入NVRTC编译缓存，也不套用物理精度门禁。
    pub(crate) fn from_frozen(frozen: &super::FrozenPtx) -> Result<Self, LauncherError> {
        let key = ArtifactKey {
            schema: KEY_SCHEMA,
            program: frozen.name().to_owned(),
            source_sha256: frozen.sha256().to_owned(),
            headers: Vec::new(),
            recipe: "frozen-warp-1.15".to_owned(),
            arch: "frozen".to_owned(),
            nvrtc: [0, 0],
        };
        let ptx = frozen.ptx().to_owned();
        if !valid_text(&ptx) {
            return Err(LauncherError::InvalidSource("冻结PTX为空、过大或含NUL"));
        }
        Ok(Self {
            key_sha256: digest(&serde_json::to_vec(&key).expect("artifact key serializes")),
            key,
            purpose: RecipePurpose::Diagnostic,
            ptx_sha256: digest(ptx.as_bytes()),
            ptx,
        })
    }

    /// 每份产物都经过同一核验，不论来自编译或磁盘。
    fn verified(
        key: ArtifactKey,
        key_sha256: String,
        recipe: &CompileRecipe,
        program: &'static str,
        ptx: String,
    ) -> Result<Self, LauncherError> {
        if !valid_text(&ptx) {
            return Err(LauncherError::Compiler("PTX为空、过大或含NUL".into()));
        }
        if recipe.gate() == PrecisionGate::F32Physics {
            check_physics_ptx(&ptx, program)
                .map_err(|error| LauncherError::Precision(error.to_string()))?;
        }
        Ok(Self {
            ptx_sha256: digest(ptx.as_bytes()),
            key,
            key_sha256,
            purpose: recipe.purpose(),
            ptx,
        })
    }

    pub fn key(&self) -> &ArtifactKey {
        &self.key
    }

    pub fn key_sha256(&self) -> &str {
        &self.key_sha256
    }

    pub fn source_sha256(&self) -> &str {
        &self.key.source_sha256
    }

    pub fn purpose(&self) -> RecipePurpose {
        self.purpose
    }

    pub fn ptx(&self) -> &str {
        &self.ptx
    }

    pub fn ptx_sha256(&self) -> &str {
        &self.ptx_sha256
    }
}

/// 产物缓存计数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArtifactStats {
    pub memory_hits: u64,
    pub disk_hits: u64,
    pub compiles: u64,
}

type Slot = Arc<OnceLock<Result<Arc<Artifact>, LauncherError>>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Refresh,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    key: ArtifactKey,
    ptx: String,
    ptx_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Archive {
    format: String,
    payload: Payload,
    payload_sha256: String,
}

fn cache_error(stage: &'static str, detail: impl ToString) -> LauncherError {
    LauncherError::Cache {
        stage,
        detail: detail.to_string(),
    }
}

fn archive_path(directory: &Path, hash: &str) -> PathBuf {
    directory.join(format!("{hash}.json"))
}
