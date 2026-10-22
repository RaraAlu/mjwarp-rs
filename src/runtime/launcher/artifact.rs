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

const KEY_SCHEMA: u32 = 2;
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

/// 头文件相对路径：允许子目录，拒绝绝对路径、`..`、反斜杠与其他分隔符。
fn valid_header_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 256 || name.contains('\0') || name.contains('\\') {
        return false;
    }
    name.split('/').all(|segment| {
        !segment.is_empty()
            && segment != "."
            && segment != ".."
            && segment
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'+'))
    })
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
        if !valid_header_name(&name) {
            return Err(LauncherError::InvalidSource("头文件名非法"));
        }
        if !valid_text(&content) {
            return Err(LauncherError::InvalidSource("头文件内容为空、过大或含NUL"));
        }
        Ok(Self { name, content })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

/// NVRTC编译来源；`name`同时作程序名。
#[derive(Clone, Debug)]
pub struct NvrtcSource {
    name: &'static str,
    source: Cow<'static, str>,
    headers: Vec<HeaderFile>,
    include_paths: Vec<String>,
    external_headers: Vec<HeaderDigest>,
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
            include_paths: Vec::new(),
            external_headers: Vec::new(),
        })
    }

    pub fn with_header(mut self, header: HeaderFile) -> Result<Self, LauncherError> {
        if self.headers.iter().any(|h| h.name == header.name) {
            return Err(LauncherError::InvalidSource("头文件重名"));
        }
        self.headers.push(header);
        Ok(self)
    }

    /// 逐字节读取目录下的 `*.h`（递归，保留 `/` 相对路径）作为编译期头文件。
    pub fn with_headers_from_dir(mut self, directory: &Path) -> Result<Self, LauncherError> {
        fn walk(current: &Path, found: &mut Vec<PathBuf>) -> Result<(), LauncherError> {
            let entries = std::fs::read_dir(current)
                .map_err(|error| LauncherError::Compiler(format!("头文件目录：{error}")))?;
            let mut paths = entries
                .map(|entry| entry.map(|value| value.path()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| LauncherError::Compiler(format!("头文件目录项：{error}")))?;
            paths.sort();
            for path in paths {
                if path.is_dir() {
                    walk(&path, found)?;
                } else if path.extension().and_then(|value| value.to_str()) == Some("h") {
                    found.push(path);
                }
            }
            Ok(())
        }
        let mut files = Vec::new();
        walk(directory, &mut files)?;
        if files.is_empty() {
            return Err(LauncherError::InvalidSource("头文件目录为空"));
        }
        for path in files {
            let relative = path
                .strip_prefix(directory)
                .map_err(|_| LauncherError::InvalidSource("头文件路径越界"))?;
            let name = relative
                .to_str()
                .ok_or(LauncherError::InvalidSource("头文件路径非UTF-8"))?
                .replace('\\', "/");
            let content = std::fs::read_to_string(&path)
                .map_err(|error| LauncherError::Compiler(format!("头文件读取：{error}")))?;
            self = self.with_header(HeaderFile::new(name, content)?)?;
        }
        Ok(self)
    }

    /// 外部 include 目录（如 CUDA SDK 头），只作编译选项并进入产物键。
    pub fn with_include_path(mut self, path: impl Into<String>) -> Result<Self, LauncherError> {
        let path = path.into();
        if path.is_empty() || path.len() > 1024 || path.contains('\0') {
            return Err(LauncherError::InvalidSource("include 路径非法"));
        }
        if !self.include_paths.contains(&path) {
            self.include_paths.push(path);
        }
        Ok(self)
    }

    /// 固定一个外部头文件的摘要（内容不进内存树，由 include path 提供）。
    pub fn with_external_header(mut self, path: &Path) -> Result<Self, LauncherError> {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(LauncherError::InvalidSource("外部头文件名非法"))?;
        if !valid_header_name(name) {
            return Err(LauncherError::InvalidSource("外部头文件名非法"));
        }
        if self.external_headers.iter().any(|h| h.name == name) {
            return Err(LauncherError::InvalidSource("外部头文件重名"));
        }
        let bytes = std::fs::read(path)
            .map_err(|error| LauncherError::Compiler(format!("外部头文件读取：{error}")))?;
        self.external_headers.push(HeaderDigest {
            name: name.to_owned(),
            sha256: digest(&bytes),
        });
        Ok(self)
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn headers(&self) -> &[HeaderFile] {
        &self.headers
    }

    pub fn include_paths(&self) -> &[String] {
        &self.include_paths
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
    include_paths: Vec<String>,
    external_headers: Vec<HeaderDigest>,
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
            include_paths: source.include_paths.clone(),
            external_headers: source.external_headers.clone(),
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
            include_paths: Vec::new(),
            external_headers: Vec::new(),
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
