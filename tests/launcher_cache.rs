//! M2两层缓存的集成测试。
//! 宿主用例不需要GPU；GPU用例默认忽略。
//! 物理源码只读提取，不改内核字符串。
#![cfg(feature = "cuda-probe")]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use cudarc::driver::CudaContext;
use mjwarp_rs::runtime::launcher::{
    ArtifactCache, ArtifactKey, CompileRecipe, LauncherError, ModuleCache, NvrtcSource,
    RecipePurpose,
};
use sha2::{Digest, Sha256};

/// 治理复核时捕获的L1实际编译摘要：入口|源码|PTX。
const L1_DIGESTS: &str = include_str!(
    "../docs/development-evidence/toolchain-evidence-revalidation/gpu-candidate-digests.txt"
);

/// 以原始字符串常量定义内核的物理文件。
const PHYSICS_FILES: [&str; 22] = [
    include_str!("../src/physics/mod.rs"),
    include_str!("../src/physics/applied_force.rs"),
    include_str!("../src/physics/com_velocity.rs"),
    include_str!("../src/physics/flex_edge_passive.rs"),
    include_str!("../src/physics/fluid_force.rs"),
    include_str!("../src/physics/gravity_compensation.rs"),
    include_str!("../src/physics/mass_matrix.rs"),
    include_str!("../src/physics/mass_matrix/multiply.rs"),
    include_str!("../src/physics/mass_matrix/multiply/borrowed.rs"),
    include_str!("../src/physics/mass_matrix/resident.rs"),
    include_str!("../src/physics/mass_solve.rs"),
    include_str!("../src/physics/mass_solve/sparse.rs"),
    include_str!("../src/physics/mass_solve/sparse/dense.rs"),
    include_str!("../src/physics/mass_solve/sparse/mixed.rs"),
    include_str!("../src/physics/mass_solve/sparse/resident.rs"),
    include_str!("../src/physics/passive_force.rs"),
    include_str!("../src/physics/resident.rs"),
    include_str!("../src/physics/rne_bias.rs"),
    include_str!("../src/physics/smooth_force.rs"),
    include_str!("../src/physics/smooth_force/joint_passive.rs"),
    include_str!("../src/physics/tendon_passive.rs"),
    include_str!("../src/physics/tendon_velocity.rs"),
];
const RNE_CU: &str = include_str!("../src/physics/rne.cu");

const NAMES: [&str; 32] = [
    "physics_00",
    "physics_01",
    "physics_02",
    "physics_03",
    "physics_04",
    "physics_05",
    "physics_06",
    "physics_07",
    "physics_08",
    "physics_09",
    "physics_10",
    "physics_11",
    "physics_12",
    "physics_13",
    "physics_14",
    "physics_15",
    "physics_16",
    "physics_17",
    "physics_18",
    "physics_19",
    "physics_20",
    "physics_21",
    "physics_22",
    "physics_23",
    "physics_24",
    "physics_25",
    "physics_26",
    "physics_27",
    "physics_28",
    "physics_29",
    "physics_30",
    "physics_31",
];

fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

struct L1Kernel {
    source: String,
    ptx_sha256: String,
    entries: Vec<&'static str>,
}

/// 只取被L1原样编译过的常量；按源码摘要匹配。
fn l1_kernels() -> Vec<L1Kernel> {
    let mut recorded: BTreeMap<&str, (&str, BTreeSet<&'static str>)> = BTreeMap::new();
    for line in L1_DIGESTS.lines().filter(|l| !l.trim().is_empty()) {
        let parts: Vec<&'static str> = line.trim().split('|').collect();
        assert_eq!(parts.len(), 3, "digest row: {line}");
        let row = recorded
            .entry(parts[1])
            .or_insert((parts[2], BTreeSet::new()));
        assert_eq!(row.0, parts[2], "one source maps to one PTX");
        row.1.insert(parts[0]);
    }
    let mut candidates: Vec<String> = vec![RNE_CU.replace("\r\n", "\n")];
    for file in PHYSICS_FILES {
        let file = file.replace("\r\n", "\n");
        for tail in file.split("&str = r#\"").skip(1) {
            if let Some((body, _)) = tail.split_once("\"#;") {
                candidates.push(body.to_owned());
            }
        }
    }
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter_map(|source| {
            let sha = digest(&source);
            let (ptx, entries) = recorded.get(sha.as_str())?;
            seen.insert(sha).then(|| L1Kernel {
                source,
                ptx_sha256: (*ptx).to_owned(),
                entries: entries.iter().copied().collect(),
            })
        })
        .collect()
}

fn sources(kernels: &[L1Kernel]) -> Vec<NvrtcSource> {
    assert!(kernels.len() <= NAMES.len());
    kernels
        .iter()
        .zip(NAMES)
        .map(|(k, name)| NvrtcSource::new(name, k.source.clone()).unwrap())
        .collect()
}

struct Temp(PathBuf);
impl Temp {
    fn new(tag: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "mjwarp-launcher-cache-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const PHYSICS: CompileRecipe = CompileRecipe::WARP_1_15_PHYSICS;

#[test]
fn host_keys_separate_purposes_and_reject_misuse() {
    let source = NvrtcSource::new("host_key", "extern \"C\" __global__ void k() {}").unwrap();
    let physics = ArtifactKey::new(&source, &PHYSICS, (12, 8));
    let probe = ArtifactKey::new(&source, &CompileRecipe::P1_PROBE, (12, 8));
    let toolchain = ArtifactKey::new(&source, &PHYSICS, (12, 9));
    assert_ne!(physics.sha256(), probe.sha256());
    assert_ne!(physics.sha256(), toolchain.sha256());
    assert_eq!(physics.source_sha256(), probe.source_sha256());
    let cache = ArtifactCache::in_memory();
    assert!(matches!(
        cache.compile(&source, &CompileRecipe::P1_PROBE, RecipePurpose::Physics),
        Err(LauncherError::RecipePurpose { .. })
    ));
    assert!(NvrtcSource::new("bad/name", "x").is_err());
}

#[test]
fn host_finds_l1_constants_for_ptx_comparison() {
    let kernels = l1_kernels();
    assert!(kernels.len() >= 10, "matched {}", kernels.len());
    assert!(kernels.iter().all(|k| !k.entries.is_empty()));
}

#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC 12.8"]
fn gpu_physics_artifacts_reproduce_l1_source_and_ptx_digests() {
    let kernels = l1_kernels();
    let sources = sources(&kernels);
    let artifacts = ArtifactCache::in_memory();
    let context = CudaContext::new(0).unwrap();
    let modules = ModuleCache::new(context.clone());
    let mut entries = 0;
    for (kernel, source) in kernels.iter().zip(&sources) {
        let artifact = artifacts
            .compile(source, &PHYSICS, RecipePurpose::Physics)
            .unwrap();
        assert_eq!(artifact.key().nvrtc(), (12, 8));
        assert_eq!(artifact.source_sha256(), digest(&kernel.source));
        assert_eq!(
            artifact.ptx_sha256(),
            kernel.ptx_sha256,
            "{}",
            source.name()
        );
        for entry in &kernel.entries {
            let handle = modules.function(&context, &artifact, entry).unwrap();
            assert_eq!(handle.artifact().ptx_sha256(), kernel.ptx_sha256);
            entries += 1;
        }
    }
    let stats = modules.stats();
    assert_eq!(stats.module_loads as usize, kernels.len());
    assert_eq!(stats.function_loads as usize, entries);
    assert_eq!(artifacts.stats().compiles as usize, kernels.len());
    println!("matched sources={} entries={entries}", kernels.len());
}

#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC 12.8"]
fn gpu_disk_layer_survives_restart_and_reports_corruption() {
    let kernels = l1_kernels();
    let source = &sources(&kernels[..1])[0];
    let directory = Temp::new("disk");
    let first = ArtifactCache::with_directory(&directory.0)
        .compile(source, &PHYSICS, RecipePurpose::Physics)
        .unwrap();
    let restarted = ArtifactCache::with_directory(&directory.0);
    let hit = restarted
        .compile(source, &PHYSICS, RecipePurpose::Physics)
        .unwrap();
    assert_eq!(hit.ptx_sha256(), first.ptx_sha256());
    assert_eq!(
        (restarted.stats().disk_hits, restarted.stats().compiles),
        (1, 0)
    );
    let path = directory.0.join(format!("{}.json", first.key_sha256()));
    std::fs::write(&path, b"{\"format\":\"truncated").unwrap();
    let corrupted = ArtifactCache::with_directory(&directory.0);
    assert!(matches!(
        corrupted.compile(source, &PHYSICS, RecipePurpose::Physics),
        Err(LauncherError::Cache { .. })
    ));
    assert!(
        corrupted
            .cached(source, &PHYSICS, RecipePurpose::Physics)
            .is_err()
    );
    let refreshed = corrupted
        .refresh(source, &PHYSICS, RecipePurpose::Physics)
        .unwrap();
    assert_eq!(refreshed.ptx_sha256(), first.ptx_sha256());
    assert_eq!(corrupted.stats().compiles, 1);
    assert!(
        ArtifactCache::with_directory(&directory.0)
            .cached(source, &PHYSICS, RecipePurpose::Physics)
            .unwrap()
            .is_some()
    );
}

#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC 12.8"]
fn gpu_concurrent_requests_share_one_compile_and_one_module() {
    let kernels = l1_kernels();
    let source = Arc::new(sources(&kernels[..1]).remove(0));
    let entry = kernels[0].entries[0];
    let artifacts = Arc::new(ArtifactCache::in_memory());
    let context = CudaContext::new(0).unwrap();
    let modules = Arc::new(ModuleCache::new(context.clone()));
    let barrier = Arc::new(Barrier::new(8));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let (source, artifacts, modules, barrier, context) = (
                source.clone(),
                artifacts.clone(),
                modules.clone(),
                barrier.clone(),
                context.clone(),
            );
            std::thread::spawn(move || {
                barrier.wait();
                let artifact = artifacts
                    .compile(&source, &PHYSICS, RecipePurpose::Physics)
                    .unwrap();
                modules.function(&context, &artifact, entry).unwrap();
                artifact
            })
        })
        .collect();
    let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert!(results.iter().all(|a| Arc::ptr_eq(a, &results[0])));
    assert_eq!(artifacts.stats().compiles, 1);
    assert_eq!(modules.stats().module_loads, 1);
}

#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC 12.8"]
fn gpu_records_cold_start_and_steady_state_costs() {
    let kernels = l1_kernels();
    let sources = sources(&kernels);
    let entries: usize = kernels.iter().map(|k| k.entries.len()).sum();
    let context = CudaContext::new(0).unwrap();
    let free = || cudarc::driver::result::mem_get_info().unwrap().0;
    let load_all = |artifacts: &ArtifactCache, modules: &ModuleCache| {
        for (kernel, source) in kernels.iter().zip(&sources) {
            let artifact = artifacts
                .compile(source, &PHYSICS, RecipePurpose::Physics)
                .unwrap();
            for entry in &kernel.entries {
                modules.function(&context, &artifact, entry).unwrap();
            }
        }
    };
    // L1现行行为：每个入口各编译、各加载一次。
    let started = Instant::now();
    for (kernel, source) in kernels.iter().zip(&sources) {
        for entry in &kernel.entries {
            let artifacts = ArtifactCache::in_memory();
            let modules = ModuleCache::new(context.clone());
            let artifact = artifacts
                .compile(source, &PHYSICS, RecipePurpose::Physics)
                .unwrap();
            modules.function(&context, &artifact, entry).unwrap();
        }
    }
    let per_entry = started.elapsed();

    let directory = Temp::new("measure");
    let before = free();
    let artifacts = ArtifactCache::with_directory(&directory.0);
    let modules = ModuleCache::new(context.clone());
    let started = Instant::now();
    load_all(&artifacts, &modules);
    let cold = started.elapsed();
    let after = free();

    let restarted = ArtifactCache::with_directory(&directory.0);
    let fresh_modules = ModuleCache::new(context.clone());
    let started = Instant::now();
    load_all(&restarted, &fresh_modules);
    let disk_warm = started.elapsed();
    assert_eq!(restarted.stats().compiles, 0);

    const ROUNDS: u32 = 100;
    let started = Instant::now();
    for _ in 0..ROUNDS {
        load_all(&artifacts, &modules);
    }
    let steady = started.elapsed() / ROUNDS;
    assert_eq!(artifacts.stats().compiles as usize, kernels.len());
    assert_eq!(modules.stats().module_loads as usize, kernels.len());

    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    let report = format!(
        concat!(
            "{{\"schema\":1,\"sources\":{},\"entries\":{},",
            "\"per_entry_compile_ms\":{:.3},\"cold_ms\":{:.3},",
            "\"disk_warm_ms\":{:.3},\"steady_round_ms\":{:.6},",
            "\"steady_rounds\":{},\"module_bytes_free_delta\":{},",
            "\"artifact_stats\":{{\"memory_hits\":{},\"disk_hits\":{},\"compiles\":{}}},",
            "\"module_stats\":{{\"module_loads\":{},\"function_loads\":{},\"hits\":{}}}}}"
        ),
        kernels.len(),
        entries,
        ms(per_entry),
        ms(cold),
        ms(disk_warm),
        ms(steady),
        ROUNDS,
        before as i128 - after as i128,
        artifacts.stats().memory_hits,
        artifacts.stats().disk_hits,
        artifacts.stats().compiles,
        modules.stats().module_loads,
        modules.stats().function_loads,
        modules.stats().hits,
    );
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_LAUNCHER_CACHE_REPORT") {
        std::fs::write(path, &report).unwrap();
    }
}
