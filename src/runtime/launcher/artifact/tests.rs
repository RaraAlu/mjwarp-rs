//! 产物缓存单元测试。

use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Barrier;
use std::sync::atomic::{AtomicUsize, Ordering};

const PTX: &str = ".version 8.7\n.target sm_70\n.address_size 64\n";
const F64_PTX: &str = ".version 8.7\n.target sm_70\n.reg .f64 %fd<2>;\nadd.f64 %fd1, %fd0, %fd0;\n";
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
    assert!(HeaderFile::new("/x.h", "a").is_err());
    assert!(HeaderFile::new("a/../x.h", "a").is_err());
    assert!(HeaderFile::new("a\\x.h", "a").is_err());
    assert!(HeaderFile::new("a//x.h", "a").is_err());
    assert!(HeaderFile::new("x.h", "").is_err());
    let nested = HeaderFile::new("nanovdb/PNanoVDB.h", "a").unwrap();
    assert_eq!(nested.name(), "nanovdb/PNanoVDB.h");
    assert_eq!(nested.content(), "a");
    let header = HeaderFile::new("x.h", "a").unwrap();
    let source = source("a").with_header(header.clone()).unwrap();
    assert!(source.clone().with_header(header).is_err());
    assert!(source.clone().with_include_path("").is_err());
    assert!(source.with_include_path("a\0b").is_err());
}

#[test]
fn headers_from_directory_are_sorted_and_keep_relative_paths() {
    let directory = Temp::new();
    let nested = directory.0.join("nanovdb");
    fs::create_dir_all(&nested).unwrap();
    fs::write(directory.0.join("b.h"), "b").unwrap();
    fs::write(directory.0.join("a.h"), "a").unwrap();
    fs::write(nested.join("c.h"), "c").unwrap();
    fs::write(directory.0.join("ignored.cuh"), "x").unwrap();
    let loaded = source("a").with_headers_from_dir(&directory.0).unwrap();
    let names: Vec<_> = loaded.headers().iter().map(|h| h.name()).collect();
    assert_eq!(names, vec!["a.h", "b.h", "nanovdb/c.h"]);
    let empty = Temp::new();
    assert!(source("a").with_headers_from_dir(&empty.0).is_err());
}

#[test]
fn key_covers_include_paths_and_external_headers() {
    let physics = CompileRecipe::WARP_1_15_PHYSICS;
    let base = ArtifactKey::new(&source("a"), &physics, NVRTC);
    let with_path = ArtifactKey::new(
        &source("a").with_include_path("C:/cuda/include").unwrap(),
        &physics,
        NVRTC,
    );
    assert_ne!(base.sha256(), with_path.sha256());
    let directory = Temp::new();
    let external = directory.0.join("cuda_fp16.h");
    fs::write(&external, "#pragma once").unwrap();
    let with_external = ArtifactKey::new(
        &source("a").with_external_header(&external).unwrap(),
        &physics,
        NVRTC,
    );
    assert_ne!(base.sha256(), with_external.sha256());
    fs::write(&external, "#pragma once\n// changed").unwrap();
    let changed = ArtifactKey::new(
        &source("a").with_external_header(&external).unwrap(),
        &physics,
        NVRTC,
    );
    assert_ne!(with_external.sha256(), changed.sha256());
    assert!(
        source("a")
            .with_external_header(&directory.0.join("missing.h"))
            .is_err()
    );
}

#[test]
fn materializes_headers_without_mutating_sources() {
    use super::nvrtc::{materialize_headers, remove_headers_dir};
    let headers = [
        HeaderFile::new("a.h", "alpha").unwrap(),
        HeaderFile::new("nanovdb/PNanoVDB.h", "nabo").unwrap(),
    ];
    let directory = materialize_headers(&headers).unwrap();
    assert_eq!(fs::read_to_string(directory.join("a.h")).unwrap(), "alpha");
    assert_eq!(
        fs::read_to_string(directory.join("nanovdb/PNanoVDB.h")).unwrap(),
        "nabo"
    );
    assert_eq!(headers[0].content(), "alpha");
    remove_headers_dir(&directory);
    assert!(!directory.exists());
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
