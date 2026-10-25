//! G6-2 host 单线程性能测量件（只随 unit test 编译；不改产品语义）。
//!
//! 口径：每个 case 先 warmup 次、再计时 reps 次；一次调用返回该次调用的接触点数。
//! 输出写到 `<G6_PERF_OUT 或 <repo>/target/g6-perf/host>/<case>.json`，字段固定。
//! `G6_PERF_QUICK=1` 只做冒烟（warmup=1、reps=5），不产出正式数字。
//! 性能数字不进精度判据、不跨硬件外推。

use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// 一次 case 的测量结果（供测试打印汇总）。
pub(crate) struct CaseTiming {
    /// 全部 `reps` 次样本之和（纳秒）。
    pub(crate) total_ns: f64,
    pub(crate) ncontacts: usize,
    pub(crate) median_ns: f64,
    pub(crate) p95_ns: f64,
    pub(crate) contacts_per_s: f64,
}

/// warmup / reps；`G6_PERF_QUICK=1` 时退化为冒烟参数。
#[derive(Clone, Copy)]
struct Plan {
    warmup: usize,
    reps: usize,
}

impl Plan {
    fn from_env() -> Self {
        if std::env::var("G6_PERF_QUICK").as_deref() == Ok("1") {
            Self { warmup: 1, reps: 5 }
        } else {
            Self {
                warmup: 20,
                reps: 200,
            }
        }
    }
}

/// 计时一个 case：`call` 每次返回该次调用的接触点数（各次应一致）。
pub(crate) fn measure<F>(case: &str, mut call: F) -> CaseTiming
where
    F: FnMut() -> usize,
{
    let plan = Plan::from_env();
    for _ in 0..plan.warmup {
        black_box(call());
    }
    let mut samples = Vec::with_capacity(plan.reps);
    let mut ncontacts = 0;
    for _ in 0..plan.reps {
        let start = Instant::now();
        ncontacts = black_box(call());
        samples.push(elapsed_ns(start.elapsed()));
    }
    let total_ns = samples.iter().sum::<u64>();
    let sorted = sorted_samples(&samples);
    let median_ns = median(&sorted);
    let p95_ns = p95(&sorted);
    let contacts_per_s = if total_ns == 0 {
        0.0
    } else {
        ncontacts as f64 * plan.reps as f64 * 1.0e9 / total_ns as f64
    };
    let path = write_json(
        case,
        plan,
        &samples,
        median_ns,
        p95_ns,
        ncontacts,
        contacts_per_s,
    );
    println!(
        "[G6-2] case={case} warmup={} reps={} ncontacts={ncontacts} median_ns={median_ns:.3} p95_ns={p95_ns:.3} contacts_per_s={contacts_per_s:.3} -> {}",
        plan.warmup,
        plan.reps,
        path.display()
    );
    CaseTiming {
        total_ns: total_ns as f64,
        ncontacts,
        median_ns,
        p95_ns,
        contacts_per_s,
    }
}

fn elapsed_ns(elapsed: Duration) -> u64 {
    elapsed.as_nanos().try_into().unwrap_or(u64::MAX)
}

fn sorted_samples(samples: &[u64]) -> Vec<u64> {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted
}

/// 中位数：奇数取中间值，偶数取中间两值均值。
fn median(sorted: &[u64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] as f64 + sorted[middle] as f64) / 2.0
    } else {
        sorted[middle] as f64
    }
}

/// p95：nearest-rank（`ceil(0.95 × n)`）。
fn p95(sorted: &[u64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (sorted.len() * 95).div_ceil(100).max(1);
    sorted[rank - 1] as f64
}

/// 输出目录：`G6_PERF_OUT`，未设置时 `<repo>/target/g6-perf/host`（gitignored）。
fn out_dir() -> PathBuf {
    match std::env::var_os("G6_PERF_OUT") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("g6-perf")
            .join("host"),
    }
}

fn write_json(
    case: &str,
    plan: Plan,
    samples: &[u64],
    median_ns: f64,
    p95_ns: f64,
    ncontacts: usize,
    contacts_per_s: f64,
) -> PathBuf {
    let dir = out_dir();
    std::fs::create_dir_all(&dir).expect("创建 G6 perf 输出目录");
    let path = dir.join(format!("{case}.json"));
    let body = format!(
        "{{\"kind\":\"host\",\"backend\":\"cpu\",\"threads\":1,\"case\":\"{}\",\"warmup\":{},\"reps\":{},\"samples_ns\":[{}],\"median_ns\":{median_ns:.6},\"p95_ns\":{p95_ns:.6},\"ncontacts\":{ncontacts},\"contacts_per_s\":{contacts_per_s:.6},\"note\":\"G6-2 host 单线程吞吐；性能不进精度判据、不跨硬件外推\"}}\n",
        escape(case),
        plan.warmup,
        plan.reps,
        sample_list(samples),
    );
    std::fs::write(&path, body).expect("写出 G6 perf JSON");
    path
}

fn sample_list(samples: &[u64]) -> String {
    let mut out = String::with_capacity(samples.len() * 8);
    for (index, value) in samples.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&value.to_string());
    }
    out
}

/// 手写 JSON 的字符串转义（case 名目前只有小写与连字符，仍按通用规则处理）。
fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out
}
