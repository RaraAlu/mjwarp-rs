//! A10@P11 CUDA 事件计时与设备显存峰值采样探针（ignored，需 NVIDIA 驱动）。
//!
//! 固定负载：仓库内置 `affine_probe` PTX（797 B，SHA-256 锁定）+ 固定元素数
//! `1 << 20` 的 f32 输入/输出；内核对每个元素写 `2 * x + 1`（二进制分数下逐位
//! 精确），探针下载输出并逐元素核对，确保被计时的是真实设备工作而不是空转。
//! 温机 10 次、重复 50 次固定写入记录；同一负载测两阶段用于核对重复性
//! （只作一致性核对，不构成性能门槛）。
//!
//! 记录写到 `MJWARP_A10P11_RECORD` 指定的 JSON；未设置时写到
//! `target/a10-p11-record/`（gitignored）并打印路径与摘要。
//! 不启动 Python、不调用 warp.dll、不改产品物理路径与既有测速语义。
#![cfg(feature = "cuda-probe")]

use cudarc::driver::{
    CudaFunction, CudaSlice, CudaStream, DriverError, LaunchConfig, PushKernelArg,
};
use mjwarp_rs::runtime::tooling::{
    BenchmarkPlan, GpuTimingSession, MemoryCheckpoint, PerformanceRecord, PerformanceReport,
    TASK_ID,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;

const KERNEL_PTX: &str = include_str!("../src/runtime/probe.ptx");
const KERNEL_SHA256: &str = "520b9ac7ee24cd1f284ec12f55102124871b17e5e5b0fc2a5dd1b00f19c833d1";
const KERNEL_ENTRY: &str = "affine_probe";
const ELEMENTS: usize = 1 << 20;
const BLOCK_THREADS: u32 = 128;
const WARMUP: u32 = 10;
const REPETITIONS: u32 = 50;
const PHASES: [&str; 2] = ["affine_probe#1", "affine_probe#2"];
const RECORD_ENV: &str = "MJWARP_A10P11_RECORD";

fn input_values() -> Vec<f32> {
    // 二进制分数样本：`* 2 + 1` 在 f32 上精确，可用逐位相等核对。
    (0..ELEMENTS)
        .map(|index| ((index % 31) as f32 - 15.0) * 0.25)
        .collect()
}

fn launch(
    stream: &Arc<CudaStream>,
    function: &CudaFunction,
    input: &CudaSlice<f32>,
    output: &mut CudaSlice<f32>,
) -> Result<(), DriverError> {
    let elements = ELEMENTS as u32;
    let config = LaunchConfig {
        grid_dim: (elements.div_ceil(BLOCK_THREADS), 1, 1),
        block_dim: (BLOCK_THREADS, 1, 1),
        shared_mem_bytes: 0,
    };
    // SAFETY: 固定 ABI（双指针 + u32 元素数）；输入只读、输出独占且容量为
    // ELEMENTS；affine_probe 以元素索引守卫在界内写；函数、模块与缓冲在本次
    // 发射期间存活，且只向会话流提交。
    unsafe {
        stream
            .launch_builder(function)
            .arg(input)
            .arg(output)
            .arg(&elements)
            .launch(config)
            .map(|_| ())
    }
}

fn record_path(profile: &str) -> PathBuf {
    if let Some(path) = std::env::var_os(RECORD_ENV) {
        return PathBuf::from(path);
    }
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/a10-p11-record");
    directory.join(format!("a10-p11-{}-{profile}.json", std::process::id()))
}

#[test]
#[ignore = "需要 NVIDIA 驱动与独占 GPU 窗口"]
fn records_cuda_event_timing_and_device_memory_peak() {
    let digest = format!("{:x}", Sha256::digest(KERNEL_PTX.as_bytes()));
    assert_eq!(digest, KERNEL_SHA256, "固定负载 PTX 身份不符");
    let session = GpuTimingSession::new(0).expect("创建 GPU 计时会话失败（需要 NVIDIA 驱动）");
    let plan = BenchmarkPlan::new(WARMUP, REPETITIONS).expect("固定测速计划不合法");
    let input_host = input_values();
    let zero_host = vec![0_f32; ELEMENTS];

    // 工作负载级显存检查点：空闲 → 加载内核 → 分配固定缓冲 → 释放。
    let idle = session.sample_memory().expect("空闲显存采样失败");
    let function = session
        .load_kernel(KERNEL_PTX, KERNEL_ENTRY)
        .expect("加载固定负载内核失败");
    let module_loaded = session.sample_memory().expect("加载后显存采样失败");
    let input = session
        .stream()
        .clone_htod(&input_host)
        .expect("上传固定输入失败");
    let mut output = session
        .stream()
        .clone_htod(&zero_host)
        .expect("分配固定输出失败");
    session.stream().synchronize().expect("缓冲上传同步失败");
    let allocated = session.sample_memory().expect("分配后显存采样失败");

    let mut report = PerformanceReport::new(
        TASK_ID,
        format!("{KERNEL_ENTRY}/{ELEMENTS}"),
        session.environment(),
        plan,
    );
    for phase in PHASES {
        let (timing, memory) = session
            .time_phase(&plan, |stream| {
                launch(stream, &function, &input, &mut output)
            })
            .unwrap_or_else(|error| panic!("阶段 {phase} 计时失败：{error}"));
        assert_eq!(
            timing.stats.count, REPETITIONS as usize,
            "{phase} 样本数不符"
        );
        assert!(timing.stats.min_ns > 0, "{phase} 事件计时必须为正");
        assert!(
            timing.stats.min_ns <= timing.stats.max_ns,
            "{phase} 最小/最大样本倒置"
        );
        report.push_record(
            PerformanceRecord::new(phase, timing.samples_ns.clone(), memory)
                .unwrap_or_else(|error| panic!("阶段 {phase} 记录不合法：{error}")),
        );
    }

    // 核对被计时的工作真实执行：输出必须等于 2 * 输入 + 1（逐位相等）。
    let output_host = session.stream().clone_dtoh(&output).expect("下载输出失败");
    session.stream().synchronize().expect("输出下载同步失败");
    for (index, (&actual, &source)) in output_host.iter().zip(&input_host).enumerate() {
        assert_eq!(
            actual,
            source * 2.0 + 1.0,
            "固定负载输出不符（索引 {index}）"
        );
    }

    drop(output);
    drop(input);
    session.stream().synchronize().expect("释放同步失败");
    let released = session.sample_memory().expect("释放后显存采样失败");
    for (label, sample) in [
        ("idle", idle),
        ("kernel-loaded", module_loaded),
        ("buffers-allocated", allocated),
        ("buffers-released", released),
    ] {
        assert!(
            sample.used_bytes() <= sample.total_bytes,
            "检查点 {label} 已用显存超过总量"
        );
        report.push_checkpoint(MemoryCheckpoint {
            label: label.to_owned(),
            used_bytes: sample.used_bytes(),
            total_bytes: sample.total_bytes,
        });
    }

    // 同负载两阶段中位数只做数量级一致性核对（不是性能门槛）。
    let (first, second) = (
        report.records[0].timing.stats.median_ns,
        report.records[1].timing.stats.median_ns,
    );
    assert!(first > 0.0 && second > 0.0, "两阶段中位数必须为正");
    let ratio = first / second;
    assert!(
        (0.2..=5.0).contains(&ratio),
        "同负载两阶段中位数差异过大：{ratio}"
    );

    report
        .push_note("固定负载是仓库既有 affine_probe 微基准，不是物理阶段；本片不做物理性能门槛。");
    report.push_note("事件计时只覆盖会话流上已提交的设备工作；不含主机侧提交开销。");
    report.push_note("显存为 cuMemGetInfo 采样口径：不含内核执行内部的瞬时分配，WDDM 下其他进程占用会进入空闲读数。");
    report.push_note("仅 Windows / 单张 RTX 4070 Ti SUPER 单设备实测；不含 Linux 与跨设备对照。");
    report.push_note("GPU 非独占：桌面图形进程常驻，显存与耗时读数含其波动。");

    let json = report.to_json().expect("性能记录不合法");
    let path = record_path(report.environment.profile);
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory).expect("创建记录目录失败");
    }
    let value: serde_json::Value = serde_json::from_str(&json).expect("性能记录 JSON 自解析失败");
    let pretty = serde_json::to_string_pretty(&value).expect("性能记录美化序列化失败");
    std::fs::write(&path, format!("{pretty}\n")).expect("写性能记录失败");
    println!(
        "a10-p11 profile={} workload={} median#1={:.3}us median#2={:.3}us peak_device={}MiB record={}",
        report.environment.profile,
        report.workload,
        first / 1000.0,
        second / 1000.0,
        report.records[0].memory.peak_used_bytes / (1024 * 1024),
        path.display()
    );
}
