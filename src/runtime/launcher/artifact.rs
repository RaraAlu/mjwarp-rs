//! W03：编译产物缓存。
//! 产物键覆盖源码、头文件、配方、目标与NVRTC版本。
//! 产物不绑定上下文；同键并发只编译一次。
//! 磁盘层沿用原子写入与损坏处理，载荷格式另定。
//! 磁盘载入后重新核验源码键、PTX摘要与精度。

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts, sys};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{CompileRecipe, LauncherError, PrecisionGate, RecipePurpose};
use crate::runtime::transfer::kernel::check_physics_ptx;

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

/// 进程内产物缓存；可选磁盘层。
pub struct ArtifactCache {
    directory: Option<PathBuf>,
    slots: Mutex<HashMap<String, Slot>>,
    memory_hits: AtomicU64,
    disk_hits: AtomicU64,
    compiles: AtomicU64,
}

impl ArtifactCache {
    pub fn in_memory() -> Self {
        Self::build(None)
    }

    /// 磁盘层目录；产物按键摘要命名。
    pub fn with_directory(directory: impl Into<PathBuf>) -> Self {
        Self::build(Some(directory.into()))
    }

    fn build(directory: Option<PathBuf>) -> Self {
        Self {
            directory,
            slots: Mutex::new(HashMap::new()),
            memory_hits: AtomicU64::new(0),
            disk_hits: AtomicU64::new(0),
            compiles: AtomicU64::new(0),
        }
    }

    /// 内存与磁盘均未命中才调用NVRTC。
    /// 磁盘产物损坏时报告错误，不静默重编。
    pub fn compile(
        &self,
        source: &NvrtcSource,
        recipe: &CompileRecipe,
        purpose: RecipePurpose,
    ) -> Result<Arc<Artifact>, LauncherError> {
        recipe.admit(purpose)?;
        let key = ArtifactKey::new(source, recipe, nvrtc_version()?);
        self.resolve(key, recipe, source.name, Mode::Normal, || {
            nvrtc_compile(source, recipe)
        })
    }

    /// 显式重编并原子替换磁盘产物。
    /// 编译或核验失败时保留旧产物。
    pub fn refresh(
        &self,
        source: &NvrtcSource,
        recipe: &CompileRecipe,
        purpose: RecipePurpose,
    ) -> Result<Arc<Artifact>, LauncherError> {
        recipe.admit(purpose)?;
        let key = ArtifactKey::new(source, recipe, nvrtc_version()?);
        self.resolve(key, recipe, source.name, Mode::Refresh, || {
            nvrtc_compile(source, recipe)
        })
    }

    /// 只查缓存，绝不编译；供捕获期间使用。
    pub fn cached(
        &self,
        source: &NvrtcSource,
        recipe: &CompileRecipe,
        purpose: RecipePurpose,
    ) -> Result<Option<Arc<Artifact>>, LauncherError> {
        recipe.admit(purpose)?;
        let key = ArtifactKey::new(source, recipe, nvrtc_version()?);
        self.lookup(&key, recipe, source.name)
    }

    /// 移除内存条目；磁盘产物保留。
    pub fn invalidate(&self, key: &ArtifactKey) -> bool {
        lock(&self.slots).remove(&key.sha256()).is_some()
    }

    pub fn clear(&self) {
        lock(&self.slots).clear();
    }

    pub fn stats(&self) -> ArtifactStats {
        ArtifactStats {
            memory_hits: self.memory_hits.load(Ordering::Relaxed),
            disk_hits: self.disk_hits.load(Ordering::Relaxed),
            compiles: self.compiles.load(Ordering::Relaxed),
        }
    }

    fn lookup(
        &self,
        key: &ArtifactKey,
        recipe: &CompileRecipe,
        program: &'static str,
    ) -> Result<Option<Arc<Artifact>>, LauncherError> {
        let hash = key.sha256();
        let slot = lock(&self.slots).get(&hash).cloned();
        if let Some(Ok(artifact)) = slot.as_ref().and_then(|slot| slot.get()) {
            self.memory_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(Arc::clone(artifact)));
        }
        let Some(artifact) = self.read_disk(key, &hash, recipe, program)? else {
            return Ok(None);
        };
        self.disk_hits.fetch_add(1, Ordering::Relaxed);
        let artifact = Arc::new(artifact);
        let slot = Slot::default();
        let _ = slot.set(Ok(Arc::clone(&artifact)));
        lock(&self.slots).entry(hash).or_insert(slot);
        Ok(Some(artifact))
    }

    fn resolve(
        &self,
        key: ArtifactKey,
        recipe: &CompileRecipe,
        program: &'static str,
        mode: Mode,
        compile: impl FnOnce() -> Result<String, LauncherError>,
    ) -> Result<Arc<Artifact>, LauncherError> {
        let hash = key.sha256();
        let slot = {
            let mut slots = lock(&self.slots);
            if mode == Mode::Refresh {
                slots.remove(&hash);
            }
            Arc::clone(slots.entry(hash.clone()).or_default())
        };
        let mut produced = false;
        let result = slot
            .get_or_init(|| {
                produced = true;
                self.produce(key, &hash, recipe, program, mode, compile)
            })
            .clone();
        if !produced && result.is_ok() {
            self.memory_hits.fetch_add(1, Ordering::Relaxed);
        }
        if result.is_err() {
            // 失败不缓存；下次调用重新尝试。
            let mut slots = lock(&self.slots);
            if slots.get(&hash).is_some_and(|s| Arc::ptr_eq(s, &slot)) {
                slots.remove(&hash);
            }
        }
        result
    }

    fn produce(
        &self,
        key: ArtifactKey,
        hash: &str,
        recipe: &CompileRecipe,
        program: &'static str,
        mode: Mode,
        compile: impl FnOnce() -> Result<String, LauncherError>,
    ) -> Result<Arc<Artifact>, LauncherError> {
        if mode == Mode::Normal
            && let Some(artifact) = self.read_disk(&key, hash, recipe, program)?
        {
            self.disk_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Arc::new(artifact));
        }
        let ptx = compile()?;
        self.compiles.fetch_add(1, Ordering::Relaxed);
        let artifact = Artifact::verified(key, hash.to_owned(), recipe, program, ptx)?;
        if let Some(directory) = &self.directory {
            write_disk(directory, &artifact)?;
        }
        Ok(Arc::new(artifact))
    }

    fn read_disk(
        &self,
        key: &ArtifactKey,
        hash: &str,
        recipe: &CompileRecipe,
        program: &'static str,
    ) -> Result<Option<Artifact>, LauncherError> {
        let Some(directory) = &self.directory else {
            return Ok(None);
        };
        let path = archive_path(directory, hash);
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(cache_error("read", format!("{}: {e}", path.display()))),
        };
        let mut bytes = Vec::new();
        file.take(MAX_ARCHIVE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| cache_error("read", e))?;
        if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
            return Err(cache_error("archive-size", "产物超过上限"));
        }
        let payload = decode(&bytes, key)?;
        Artifact::verified(payload.key, hash.to_owned(), recipe, program, payload.ptx).map(Some)
    }
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

fn decode(bytes: &[u8], expected: &ArtifactKey) -> Result<Payload, LauncherError> {
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

fn write_disk(directory: &Path, artifact: &Artifact) -> Result<(), LauncherError> {
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

/// 进程只加载一份NVRTC；成功结果缓存，失败每次重查。
fn nvrtc_version() -> Result<(i32, i32), LauncherError> {
    static VERSION: OnceLock<(i32, i32)> = OnceLock::new();
    if let Some(version) = VERSION.get() {
        return Ok(*version);
    }
    let version = query_nvrtc_version()?;
    Ok(*VERSION.get_or_init(|| version))
}

fn query_nvrtc_version() -> Result<(i32, i32), LauncherError> {
    std::panic::catch_unwind(|| {
        // SAFETY: Only probes whether the compiler library can be loaded.
        if !unsafe { sys::is_culib_present() } {
            return Err(LauncherError::Compiler("NVRTC不可用".into()));
        }
        let (mut major, mut minor) = (0, 0);
        // SAFETY: The library is loaded; both out-pointers are valid locals.
        unsafe { sys::nvrtcVersion(&mut major, &mut minor) }
            .result()
            .map_err(|e| LauncherError::Compiler(format!("NVRTC版本：{e:?}")))?;
        Ok((major, minor))
    })
    .unwrap_or_else(|_| Err(LauncherError::Compiler("NVRTC加载异常".into())))
}

/// 编译选项只取自配方，与L1现行路径逐项一致。
fn nvrtc_compile(source: &NvrtcSource, recipe: &CompileRecipe) -> Result<String, LauncherError> {
    if !source.headers.is_empty() {
        return Err(LauncherError::Unsupported("内存头文件编译待W15"));
    }
    let options = CompileOptions {
        fmad: Some(recipe.fmad()),
        use_fast_math: Some(recipe.fast_math()),
        options: recipe.nvrtc_options(),
        name: Some(format!("{}.cu", source.name)),
        ..Default::default()
    };
    let text = source.source.as_ref();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compile_ptx_with_opts(text, options)
            .map(|ptx| ptx.to_src())
            .map_err(|e| LauncherError::Compiler(format!("{}: {e:?}", source.name)))
    }))
    .unwrap_or_else(|_| Err(LauncherError::Compiler("NVRTC编译异常".into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::sync::atomic::AtomicUsize;

    const PTX: &str = ".version 8.7\n.target sm_70\n.address_size 64\n";
    const F64_PTX: &str =
        ".version 8.7\n.target sm_70\n.reg .f64 %fd<2>;\nadd.f64 %fd1, %fd0, %fd0;\n";
    const NVRTC: (i32, i32) = (12, 8);

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "mjwarp-artifact-test-{}-{}",
                std::process::id(),
                TEMP_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn source(text: &'static str) -> NvrtcSource {
        NvrtcSource::new("unit_kernel", text).unwrap()
    }

    fn run(
        cache: &ArtifactCache,
        source: &NvrtcSource,
        recipe: &CompileRecipe,
        mode: Mode,
        ptx: Result<&str, LauncherError>,
    ) -> Result<Arc<Artifact>, LauncherError> {
        let key = ArtifactKey::new(source, recipe, NVRTC);
        cache.resolve(key, recipe, source.name, mode, || ptx.map(str::to_owned))
    }

    fn never(
        cache: &ArtifactCache,
        source: &NvrtcSource,
        recipe: &CompileRecipe,
    ) -> Result<Arc<Artifact>, LauncherError> {
        let key = ArtifactKey::new(source, recipe, NVRTC);
        cache.resolve(key, recipe, source.name, Mode::Normal, || {
            panic!("cache hit compiled")
        })
    }

    #[test]
    fn key_covers_source_headers_recipe_target_and_toolchain() {
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        let base = ArtifactKey::new(&source("a"), &physics, NVRTC);
        assert_eq!(base, ArtifactKey::new(&source("a"), &physics, NVRTC));
        assert_eq!(
            base.sha256(),
            ArtifactKey::new(&source("a"), &physics, NVRTC).sha256()
        );
        let header = HeaderFile::new("h.cuh", "x").unwrap();
        let changed = [
            ArtifactKey::new(&source("b"), &physics, NVRTC),
            ArtifactKey::new(&NvrtcSource::new("other", "a").unwrap(), &physics, NVRTC),
            ArtifactKey::new(
                &source("a").with_header(header.clone()).unwrap(),
                &physics,
                NVRTC,
            ),
            ArtifactKey::new(
                &source("a")
                    .with_header(HeaderFile::new("h.cuh", "y").unwrap())
                    .unwrap(),
                &physics,
                NVRTC,
            ),
            ArtifactKey::new(&source("a"), &CompileRecipe::WARP_1_15_DIAGNOSTIC, NVRTC),
            ArtifactKey::new(&source("a"), &CompileRecipe::P1_PROBE, NVRTC),
            ArtifactKey::new(&source("a"), &physics, (12, 9)),
            ArtifactKey::new(&source("a"), &physics, (13, 8)),
        ];
        let mut hashes: Vec<_> = changed.iter().map(ArtifactKey::sha256).collect();
        hashes.push(base.sha256());
        let unique: std::collections::BTreeSet<_> = hashes.iter().collect();
        assert_eq!(unique.len(), hashes.len());
        assert!(base.recipe().contains("arch=--gpu-architecture=compute_70"));
        assert_eq!(base.nvrtc(), NVRTC);
    }

    #[test]
    fn rejects_invalid_sources_and_headers() {
        assert!(NvrtcSource::new("", "a").is_err());
        assert!(NvrtcSource::new("bad name", "a").is_err());
        assert!(NvrtcSource::new("k", "").is_err());
        assert!(NvrtcSource::new("k", "a\0b").is_err());
        assert!(HeaderFile::new("../x.h", "a").is_err());
        assert!(HeaderFile::new("x.h", "").is_err());
        let header = HeaderFile::new("x.h", "a").unwrap();
        let source = source("a").with_header(header.clone()).unwrap();
        assert!(source.clone().with_header(header).is_err());
        assert!(matches!(
            nvrtc_compile(&source, &CompileRecipe::WARP_1_15_DIAGNOSTIC),
            Err(LauncherError::Unsupported(_))
        ));
    }

    #[test]
    fn purpose_admission_runs_before_any_compiler_access() {
        let cache = ArtifactCache::in_memory();
        for (recipe, purpose) in [
            (CompileRecipe::P1_PROBE, RecipePurpose::Physics),
            (CompileRecipe::WARP_1_15_DIAGNOSTIC, RecipePurpose::Physics),
            (CompileRecipe::WARP_1_15_PHYSICS, RecipePurpose::Probe),
        ] {
            for result in [
                cache.compile(&source("a"), &recipe, purpose).map(|_| ()),
                cache.refresh(&source("a"), &recipe, purpose).map(|_| ()),
                cache.cached(&source("a"), &recipe, purpose).map(|_| ()),
            ] {
                assert!(matches!(result, Err(LauncherError::RecipePurpose { .. })));
            }
        }
        assert_eq!(cache.stats(), ArtifactStats::default());
    }

    #[test]
    fn probe_and_physics_recipes_never_share_entries() {
        let directory = Temp::new();
        let cache = ArtifactCache::with_directory(&directory.0);
        let probe = CompileRecipe::P1_PROBE;
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        let artifact = run(&cache, &source("a"), &probe, Mode::Normal, Ok(PTX)).unwrap();
        assert_eq!(artifact.purpose(), RecipePurpose::Probe);
        let key = ArtifactKey::new(&source("a"), &physics, NVRTC);
        assert!(
            cache
                .lookup(&key, &physics, "unit_kernel")
                .unwrap()
                .is_none()
        );
        let compiled = run(&cache, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
        assert_eq!(compiled.purpose(), RecipePurpose::Physics);
        assert_ne!(compiled.key_sha256(), artifact.key_sha256());
        assert_eq!(cache.stats().compiles, 2);
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
    }

    #[test]
    fn memory_and_disk_hits_never_call_the_compiler() {
        let directory = Temp::new();
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        let cache = ArtifactCache::with_directory(&directory.0);
        let first = run(&cache, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
        let hit = never(&cache, &source("a"), &physics).unwrap();
        assert!(Arc::ptr_eq(&first, &hit));
        assert_eq!(
            cache.stats(),
            ArtifactStats {
                memory_hits: 1,
                disk_hits: 0,
                compiles: 1
            }
        );
        // 新实例模拟新进程，只能命中磁盘。
        let restarted = ArtifactCache::with_directory(&directory.0);
        let disk = never(&restarted, &source("a"), &physics).unwrap();
        assert_eq!(disk.ptx_sha256(), first.ptx_sha256());
        assert_eq!(disk.key(), first.key());
        let again = never(&restarted, &source("a"), &physics).unwrap();
        assert!(Arc::ptr_eq(&disk, &again));
        assert_eq!(
            restarted.stats(),
            ArtifactStats {
                memory_hits: 1,
                disk_hits: 1,
                compiles: 0
            }
        );
    }

    #[test]
    fn corruption_is_reported_without_compiling_until_refresh() {
        let directory = Temp::new();
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        let cache = ArtifactCache::with_directory(&directory.0);
        let first = run(&cache, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
        let path = archive_path(&directory.0, first.key_sha256());
        fs::write(&path, b"corrupted").unwrap();
        let restarted = ArtifactCache::with_directory(&directory.0);
        assert!(matches!(
            never(&restarted, &source("a"), &physics),
            Err(LauncherError::Cache {
                stage: "archive-json",
                ..
            })
        ));
        // 失败不进入内存；显式刷新后恢复。
        let refreshed = run(&restarted, &source("a"), &physics, Mode::Refresh, Ok(PTX)).unwrap();
        assert_eq!(refreshed.ptx_sha256(), first.ptx_sha256());
        assert!(
            never(
                &ArtifactCache::with_directory(&directory.0),
                &source("a"),
                &physics
            )
            .is_ok()
        );
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
    }

    fn rewrite(path: &Path, edit: impl FnOnce(&mut Payload)) {
        let mut archive: Archive = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        edit(&mut archive.payload);
        archive.payload_sha256 = digest(&serde_json::to_vec(&archive.payload).unwrap());
        fs::write(path, serde_json::to_vec(&archive).unwrap()).unwrap();
    }

    #[test]
    fn checksum_valid_tampering_is_still_rejected() {
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        for case in 0..4 {
            let directory = Temp::new();
            let cache = ArtifactCache::with_directory(&directory.0);
            let first = run(&cache, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
            let path = archive_path(&directory.0, first.key_sha256());
            rewrite(&path, |payload| match case {
                0 => payload.key.nvrtc = [12, 9],
                1 => payload.ptx.push_str("// edited\n"),
                2 => {
                    payload.ptx = F64_PTX.into();
                    payload.ptx_sha256 = digest(F64_PTX.as_bytes());
                }
                _ => payload.key.recipe = CompileRecipe::P1_PROBE.fingerprint(),
            });
            let result = never(
                &ArtifactCache::with_directory(&directory.0),
                &source("a"),
                &physics,
            );
            match case {
                0 | 3 => assert!(matches!(
                    result,
                    Err(LauncherError::Cache {
                        stage: "key-mismatch",
                        ..
                    })
                )),
                1 => assert!(matches!(
                    result,
                    Err(LauncherError::Cache {
                        stage: "ptx-digest",
                        ..
                    })
                )),
                _ => assert!(matches!(result, Err(LauncherError::Precision(_)))),
            }
        }
        // 未知字段与错误格式同样拒绝。
        let directory = Temp::new();
        let cache = ArtifactCache::with_directory(&directory.0);
        let first = run(&cache, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
        let path = archive_path(&directory.0, first.key_sha256());
        let text = String::from_utf8(fs::read(&path).unwrap()).unwrap();
        fs::write(
            &path,
            text.replacen("{\"format\"", "{\"extra\":1,\"format\"", 1),
        )
        .unwrap();
        assert!(
            never(
                &ArtifactCache::with_directory(&directory.0),
                &source("a"),
                &physics
            )
            .is_err()
        );
        fs::write(&path, text.replace(FORMAT, "other-format")).unwrap();
        assert!(matches!(
            never(
                &ArtifactCache::with_directory(&directory.0),
                &source("a"),
                &physics
            ),
            Err(LauncherError::Cache {
                stage: "format",
                ..
            })
        ));
    }

    #[test]
    fn precision_gate_rejects_compiled_f64_and_caches_nothing() {
        let directory = Temp::new();
        let cache = ArtifactCache::with_directory(&directory.0);
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        assert!(matches!(
            run(&cache, &source("a"), &physics, Mode::Normal, Ok(F64_PTX)),
            Err(LauncherError::Precision(_))
        ));
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 0);
        let key = ArtifactKey::new(&source("a"), &physics, NVRTC);
        assert!(
            cache
                .lookup(&key, &physics, "unit_kernel")
                .unwrap()
                .is_none()
        );
        // 诊断配方不做f32检查，键也不同。
        let diagnostic = CompileRecipe::WARP_1_15_DIAGNOSTIC;
        assert!(run(&cache, &source("a"), &diagnostic, Mode::Normal, Ok(F64_PTX)).is_ok());
    }

    #[test]
    fn failed_compile_is_not_cached_and_failed_refresh_keeps_last_good() {
        let directory = Temp::new();
        let cache = ArtifactCache::with_directory(&directory.0);
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        let failure = || Err(LauncherError::Compiler("injected".into()));
        assert!(run(&cache, &source("a"), &physics, Mode::Normal, failure()).is_err());
        let good = run(&cache, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
        let path = archive_path(&directory.0, good.key_sha256());
        let before = fs::read(&path).unwrap();
        assert!(run(&cache, &source("a"), &physics, Mode::Refresh, failure()).is_err());
        assert!(run(&cache, &source("a"), &physics, Mode::Refresh, Ok(F64_PTX)).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        let names: Vec<_> = fs::read_dir(&directory.0).unwrap().collect();
        assert_eq!(names.len(), 1);
        assert!(
            never(
                &ArtifactCache::with_directory(&directory.0),
                &source("a"),
                &physics
            )
            .is_ok()
        );
    }

    #[test]
    fn concurrent_requests_compile_once() {
        let directory = Temp::new();
        let cache = Arc::new(ArtifactCache::with_directory(&directory.0));
        let calls = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(8));
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let (cache, calls, barrier) = (cache.clone(), calls.clone(), barrier.clone());
                let physics = physics.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    let source = source("a");
                    let key = ArtifactKey::new(&source, &physics, NVRTC);
                    cache
                        .resolve(key, &physics, source.name, Mode::Normal, || {
                            calls.fetch_add(1, Ordering::SeqCst);
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            Ok(PTX.to_owned())
                        })
                        .unwrap()
                })
            })
            .collect();
        let artifacts: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(artifacts.iter().all(|a| Arc::ptr_eq(a, &artifacts[0])));
        let stats = cache.stats();
        assert_eq!(
            (stats.compiles, stats.memory_hits, stats.disk_hits),
            (1, 7, 0)
        );
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
    }

    #[test]
    fn invalidation_drops_memory_but_keeps_disk_and_lookup_never_compiles() {
        let directory = Temp::new();
        let cache = ArtifactCache::with_directory(&directory.0);
        let physics = CompileRecipe::WARP_1_15_PHYSICS;
        let key = ArtifactKey::new(&source("a"), &physics, NVRTC);
        assert!(
            cache
                .lookup(&key, &physics, "unit_kernel")
                .unwrap()
                .is_none()
        );
        let first = run(&cache, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
        assert!(cache.invalidate(first.key()));
        assert!(!cache.invalidate(first.key()));
        let disk = cache
            .lookup(&key, &physics, "unit_kernel")
            .unwrap()
            .unwrap();
        assert!(!Arc::ptr_eq(&disk, &first));
        cache.clear();
        fs::remove_file(archive_path(&directory.0, first.key_sha256())).unwrap();
        assert!(
            cache
                .lookup(&key, &physics, "unit_kernel")
                .unwrap()
                .is_none()
        );
        assert_eq!(cache.stats().compiles, 1);
        // 纯内存缓存失效后只能重编。
        let memory = ArtifactCache::in_memory();
        run(&memory, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
        memory.clear();
        run(&memory, &source("a"), &physics, Mode::Normal, Ok(PTX)).unwrap();
        assert_eq!(memory.stats().compiles, 2);
    }
}
