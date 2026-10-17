//! M0编译开销基线记录器测试。
//! 宿主测试不加载NVRTC；GPU测试默认忽略。
use super::{
    COMPILE_OCCURRENCES, CompileSample, LoadSample, SynchronousKernel, append_compile_baseline,
    compile_baseline_record, next_occurrence,
};
use crate::runtime::transfer::TransferSession;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

struct Temp(PathBuf);

impl Temp {
    fn new(tag: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "mjwarp-compile-baseline-{tag}-{}-{nanos}",
            std::process::id()
        ));
        Self(path)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn sample<'a>(source: &'a str, ptx: &'a str, memory: bool) -> CompileSample<'a> {
    let memory = |free| memory.then_some((free, 16 << 30));
    CompileSample {
        entry: "baseline_probe",
        source,
        ptx,
        require_f32: true,
        compile: Duration::from_micros(1500),
        check: Duration::from_micros(20),
        load: LoadSample {
            elapsed: Duration::from_micros(300),
            before: memory(8 << 30),
            after: memory((8 << 30) - 4096),
        },
    }
}

fn read_lines(path: &Path) -> Vec<serde_json::Value> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn counts_repeated_compiles_per_source_digest() {
    let mut counts = HashMap::new();
    assert_eq!(next_occurrence(&mut counts, "a"), 1);
    assert_eq!(next_occurrence(&mut counts, "b"), 1);
    assert_eq!(next_occurrence(&mut counts, "a"), 2);
    assert_eq!(next_occurrence(&mut counts, "a"), 3);
    assert_eq!(counts.values().sum::<u32>(), 4);
}

#[test]
fn records_digests_recipe_timings_and_device_memory() {
    let record = compile_baseline_record(&sample("src", "ptx", true), "abc", 2, 7, (12, 8));
    assert_eq!(record["schema"], 1);
    assert_eq!(record["kind"], "nvrtc-compile-load");
    assert_eq!(record["entry"], "baseline_probe");
    assert_eq!(record["sourceSha256"], "abc");
    // SHA-256("ptx")。
    assert_eq!(
        record["ptxSha256"],
        "d9657e50116f295ed2619730493b11018e73fa798b576fffa7f01973aba8d929"
    );
    assert_eq!(record["sourceBytes"], 3);
    assert_eq!(record["ptxBytes"], 3);
    assert_eq!(record["occurrence"], 2);
    assert_eq!(record["sequence"], 7);
    assert_eq!(record["nvrtcVersion"], serde_json::json!([12, 8]));
    assert_eq!(record["fmad"], true);
    assert_eq!(record["useFastMath"], false);
    assert_eq!(
        record["options"],
        serde_json::json!(["--std=c++17", "--gpu-architecture=compute_70"])
    );
    assert_eq!(record["compileNs"], 1_500_000);
    assert_eq!(record["checkNs"], 20_000);
    assert_eq!(record["loadNs"], 300_000);
    assert_eq!(record["deviceFreeBeforeLoad"], 8u64 << 30);
    assert_eq!(record["deviceFreeAfterLoad"], (8u64 << 30) - 4096);
    assert_eq!(record["deviceTotal"], 16u64 << 30);
}

#[test]
fn leaves_device_memory_null_without_queries() {
    let record = compile_baseline_record(&sample("src", "ptx", false), "abc", 1, 1, (12, 8));
    assert!(record["deviceFreeBeforeLoad"].is_null());
    assert!(record["deviceFreeAfterLoad"].is_null());
    assert!(record["deviceTotal"].is_null());
}

#[test]
fn appends_one_json_line_per_record_in_a_process_file() {
    let temp = Temp::new("append");
    let first = compile_baseline_record(&sample("a", "p", true), "a", 1, 1, (12, 8));
    let second = compile_baseline_record(&sample("a", "p", true), "a", 2, 2, (12, 8));
    let path = append_compile_baseline(&temp.0, &first).unwrap();
    assert_eq!(append_compile_baseline(&temp.0, &second).unwrap(), path);
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        format!("compile-{}.jsonl", std::process::id())
    );
    let lines = read_lines(&path);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["occurrence"], 1);
    assert_eq!(lines[1]["occurrence"], 2);
}

#[test]
fn rejects_unwritable_baseline_directory() {
    let temp = Temp::new("blocked");
    std::fs::write(&temp.0, b"file, not directory").unwrap();
    let record = compile_baseline_record(&sample("a", "p", true), "a", 1, 1, (12, 8));
    assert!(append_compile_baseline(&temp.0, &record).is_err());
    std::fs::remove_file(&temp.0).unwrap();
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC; compiles one probe kernel three times"]
fn gpu_baseline_records_cold_and_repeat_compiles_only_when_enabled() {
    let session = TransferSession::new(0).unwrap();
    let temp = Temp::new("gpu");
    // 唯一注释令源码摘要不与其他测试共享。
    let source = format!(
        "// baseline {:?}\nextern \"C\" __global__ void baseline_probe(float* out) {{ out[threadIdx.x] = 1.5f; }}",
        temp.0
    );
    let disabled =
        SynchronousKernel::compile_measured(&session, &source, "baseline_probe", true, None);
    assert!(disabled.is_ok());
    assert!(!temp.0.exists());
    for _ in 0..2 {
        SynchronousKernel::compile_measured(
            &session,
            &source,
            "baseline_probe",
            true,
            Some(&temp.0),
        )
        .unwrap();
    }
    let path = temp.0.join(format!("compile-{}.jsonl", std::process::id()));
    let lines = read_lines(&path);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["sourceSha256"], lines[1]["sourceSha256"]);
    assert_eq!(lines[0]["ptxSha256"], lines[1]["ptxSha256"]);
    // 禁用模式不计入进程内次数。
    assert_eq!(lines[0]["occurrence"], 1);
    assert_eq!(lines[1]["occurrence"], 2);
    for line in &lines {
        assert!(line["compileNs"].as_u64().unwrap() > 0);
        assert!(line["loadNs"].as_u64().unwrap() > 0);
        assert!(line["deviceFreeBeforeLoad"].as_u64().is_some());
        assert!(line["deviceFreeAfterLoad"].as_u64().is_some());
        assert!(line["deviceTotal"].as_u64().unwrap() > 0);
        assert!(line["nvrtcVersion"][0].as_i64().unwrap() >= 12);
    }
    let guard = COMPILE_OCCURRENCES.lock().unwrap();
    let key = lines[0]["sourceSha256"].as_str().unwrap();
    assert_eq!(guard.as_ref().unwrap()[key], 2);
}
