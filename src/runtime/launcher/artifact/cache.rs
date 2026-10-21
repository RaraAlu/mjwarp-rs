//! 进程内缓存与磁盘回退；同键并发只编译一次。

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::disk::{decode, write_disk};
use super::nvrtc::{nvrtc_compile, nvrtc_version};
use super::*;

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

    pub(super) fn lookup(
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

    pub(super) fn resolve(
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
