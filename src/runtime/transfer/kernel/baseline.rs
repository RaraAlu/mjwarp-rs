//! 编译基线样本、计数与输出。

use super::*;

/// 模块加载耗时与前后空闲显存。
/// 显存项只在基线模式填写。
pub(super) struct LoadSample {
    pub(super) elapsed: Duration,
    pub(super) before: Option<(usize, usize)>,
    pub(super) after: Option<(usize, usize)>,
}

/// 一次编译与加载的基线样本。
pub(super) struct CompileSample<'a> {
    pub(super) entry: &'static str,
    pub(super) source: &'a str,
    pub(super) ptx: &'a str,
    pub(super) require_f32: bool,
    pub(super) compile: Duration,
    pub(super) check: Duration,
    pub(super) load: LoadSample,
}

// 本进程内各源码摘要的编译次数。
// 次数大于一即为可被进程内缓存消除的重复编译。
pub(super) static COMPILE_OCCURRENCES: Mutex<Option<HashMap<String, u32>>> = Mutex::new(None);

pub(super) fn next_occurrence(counts: &mut HashMap<String, u32>, source_sha256: &str) -> u32 {
    let count = counts.entry(source_sha256.to_owned()).or_insert(0);
    *count += 1;
    *count
}

pub(super) fn compile_baseline_record(
    sample: &CompileSample<'_>,
    source_sha256: &str,
    occurrence: u32,
    sequence: u32,
    nvrtc: (i32, i32),
) -> serde_json::Value {
    use sha2::{Digest, Sha256};
    let ptx_sha256 = format!("{:x}", Sha256::digest(sample.ptx.as_bytes()));
    let free = |memory: Option<(usize, usize)>| memory.map(|(free, _)| free);
    let total = sample
        .load
        .before
        .or(sample.load.after)
        .map(|(_, total)| total);
    serde_json::json!({
        "schema": 1,
        "kind": "nvrtc-compile-load",
        "pid": std::process::id(),
        "sequence": sequence,
        "entry": sample.entry,
        "sourceSha256": source_sha256,
        "ptxSha256": ptx_sha256,
        "sourceBytes": sample.source.len(),
        "ptxBytes": sample.ptx.len(),
        "occurrence": occurrence,
        "requireF32": sample.require_f32,
        "nvrtcVersion": [nvrtc.0, nvrtc.1],
        "fmad": PHYSICS_FMAD,
        "useFastMath": false,
        "options": ["--std=c++17", "--gpu-architecture=compute_70"],
        "compileNs": sample.compile.as_nanos() as u64,
        "checkNs": sample.check.as_nanos() as u64,
        "loadNs": sample.load.elapsed.as_nanos() as u64,
        "deviceFreeBeforeLoad": free(sample.load.before),
        "deviceFreeAfterLoad": free(sample.load.after),
        "deviceTotal": total,
    })
}

/// 每个进程写一个JSON行文件，避免并发交错。
pub(super) fn record_compile_baseline(
    directory: &Path,
    sample: &CompileSample<'_>,
) -> Result<(), ProbeError> {
    use sha2::{Digest, Sha256};
    let failure = |detail: String| ProbeError::Compilation {
        stage: "launcher-compile-baseline",
        detail: format!("{}: {detail}", sample.entry),
    };
    let (mut major, mut minor) = (0, 0);
    // SAFETY: Compilation already loaded NVRTC; both output pointers are valid.
    unsafe { sys::nvrtcVersion(&mut major, &mut minor) }
        .result()
        .map_err(|error| failure(format!("NVRTC version: {error:?}")))?;
    let source_sha256 = format!("{:x}", Sha256::digest(sample.source.as_bytes()));
    let (occurrence, sequence) = {
        let mut guard = COMPILE_OCCURRENCES
            .lock()
            .map_err(|_| failure("occurrence lock poisoned".into()))?;
        let counts = guard.get_or_insert_with(HashMap::new);
        let occurrence = next_occurrence(counts, &source_sha256);
        (occurrence, counts.values().sum::<u32>())
    };
    let record =
        compile_baseline_record(sample, &source_sha256, occurrence, sequence, (major, minor));
    append_compile_baseline(directory, &record)
        .map(|_| ())
        .map_err(|error| failure(error.to_string()))
}

pub(super) fn append_compile_baseline(
    directory: &Path,
    record: &serde_json::Value,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use std::io::Write;
    let path = directory.join(format!("compile-{}.jsonl", std::process::id()));
    std::fs::create_dir_all(directory)?;
    let mut line = serde_json::to_vec(record)?;
    line.push(b'\n');
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?
        .write_all(&line)?;
    Ok(path)
}
