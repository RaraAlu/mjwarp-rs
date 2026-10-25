#![cfg(feature = "cuda-probe")]
#![allow(dead_code)]
//! 渲染/BVH perf：独占 GPU 窗口的相对性能基准（渲染/BVH perf）。
//!
//! 口径（任务 JSON + Arch 2026-10-25）：
//! - 独占 GPU 窗口；warmup ≥5、迭代 ≥30；
//! - 两个配置：单帧（spp=1，120,000 射线）与 AA（spp=2 → nsamples=4，480,000 射线）；
//! - 每配置报 min/median/p95 **并保留每次迭代原始毫秒序列**（`s8perf[raw]` JSON 行，
//!   由运行日志派生 `raw-timings.json`，摘要与原始序列同源）；
//! - 计时口径：`launch_ms` = 仅 `renderer.render` 调用；`frame_ms` = 发射 + 输出读回同步
//!   （读回是 CUDA 同步点，确保内核真正执行完成；主指标用 `frame_ms`）；
//! - 只给单场景单机相对基准（AA/单帧比值、每 120,000 射线 ms），**不宣称绝对性能**、
//!   不与上游/其它硬件比较；不改渲染语义与已验收数值；
//! - 位姿用产品 camlight 输出（与 S8 交付配置一致；数值 0 位差已由 S8/QA 验收），
//!   射线沿用 S5 预计算缓冲。
//!
//! GPU 用例默认 ignore（perf 需独占窗口）：
//! cargo test --locked --features cuda-probe --test render_perf -- --ignored --test-threads=1 --nocapture

#[path = "support/render_bvh/e2e.rs"]
mod e2e;
#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/s7b3.rs"]
mod s7b3;

use std::time::Instant;

use mjwarp_rs::model::render::RenderFields;
use mjwarp_rs::render::CameraModel;
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::{ImageOutput, OutputValues, RayBuffer, RenderContext, device_memory};
use mjwarp_rs::runtime::TransferSession;

/// 预热次数（Arch 口径：≥5）。
const WARMUP: usize = 5;
/// 测量迭代数（Arch 口径：≥30）。
const ITERATIONS: usize = 30;
/// 单帧射线数（两相机 320×240 + 240×180）。
const RAYS_PER_FRAME: usize = 120_000;

fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
    sorted[index]
}

/// `(min, median, p95)`；输入原样保留（调用方在需要时自行保存原始序列）。
fn summarize(values: &[f64]) -> (f64, f64, f64) {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    (
        sorted[0],
        percentile(&sorted, 0.5),
        percentile(&sorted, 0.95),
    )
}

/// 读回 rgb 输出作为 CUDA 同步点（确保本帧内核执行完毕）。
fn sync(context: &RenderContext) {
    match context.outputs().read(ImageOutput::Rgb).expect("同步读回") {
        OutputValues::Rgb(values) => std::hint::black_box(values),
        other => panic!("rgb 类型 {other:?}"),
    };
}

struct FrameMeasure {
    label: &'static str,
    rays: usize,
    launch_ms: Vec<f64>,
    frame_ms: Vec<f64>,
}

/// 测一个配置：warmup 5 次（不计入统计），再迭代 30 次记录原始序列。
#[allow(clippy::too_many_arguments)] // 测量辅助：输入/输出显式传参，避免隐藏状态影响计时。
fn measure_mode(
    label: &'static str,
    rays: usize,
    renderer: &MegakernelRenderer,
    context: &RenderContext,
    fields: &RenderFields,
    model: &CameraModel,
    poses: &[CameraPose],
    ray_buffer: &RayBuffer,
) -> FrameMeasure {
    for _ in 0..WARMUP {
        renderer
            .render(context, fields, model, poses, ray_buffer)
            .expect("warmup 渲染");
        sync(context);
    }
    let mut launch_ms = Vec::with_capacity(ITERATIONS);
    let mut frame_ms = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let started = Instant::now();
        renderer
            .render(context, fields, model, poses, ray_buffer)
            .expect("计时渲染");
        let launch_elapsed = started.elapsed();
        sync(context);
        launch_ms.push(launch_elapsed.as_secs_f64() * 1000.0);
        frame_ms.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let (launch_min, launch_median, launch_p95) = summarize(&launch_ms);
    let (frame_min, frame_median, frame_p95) = summarize(&frame_ms);
    println!(
        "s8perf[stat] {label}: rays={rays} warmup={WARMUP} iterations={ITERATIONS} \
         launch(min,median,p95)=({launch_min:.3},{launch_median:.3},{launch_p95:.3})ms \
         frame(min,median,p95)=({frame_min:.3},{frame_median:.3},{frame_p95:.3})ms"
    );
    FrameMeasure {
        label,
        rays,
        launch_ms,
        frame_ms,
    }
}

/// 独占 GPU 窗口下的相对基准：单帧 vs AA（不声明绝对性能）。
#[test]
#[ignore = "需要独占 GPU + NVRTC（perf 窗口）"]
fn relative_frame_benchmark_with_aa() {
    let started_all = Instant::now();
    let session: TransferSession = s7b2::session();
    let fields = s7b2::render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    // 位姿来源与 S8 交付一致：产品 camlight 输出（数值 0 位差已验收；性能与来源无关）。
    let product = e2e::product_camlight(&session);
    let poses = product.poses;
    let free_before = device_memory().expect("显存查询").0;

    // ---- 单帧（spp=1）----
    let context = s7b2::create_context(&session, &fields, &s7b2::default_params());
    let free_after_create = device_memory().expect("显存查询").0;
    let rays = s7b2::precomputed_rays(&session, &fields);
    assert_eq!(rays.len(), RAYS_PER_FRAME, "单帧射线数");
    let renderer = MegakernelRenderer::new(&session).expect("渲染内核编译");
    let single = measure_mode(
        "single_spp1",
        RAYS_PER_FRAME,
        &renderer,
        &context,
        &fields,
        &model,
        &poses,
        &rays,
    );
    let free_after_single = device_memory().expect("显存查询").0;
    context.destroy();

    // ---- AA（spp=2 → nsamples=4）----
    let aa_context = s7b3::create_variant_context(&session, &fields, s7b3::AA_VARIANT);
    let free_after_aa_create = device_memory().expect("显存查询").0;
    let aa_rays = s7b3::aa_rays(&session, &fields);
    assert_eq!(
        aa_rays.len(),
        4 * RAYS_PER_FRAME,
        "AA 射线条数 = nsamples × total_rays"
    );
    let aa = measure_mode(
        "aa2_spp2",
        4 * RAYS_PER_FRAME,
        &renderer,
        &aa_context,
        &fields,
        &model,
        &poses,
        &aa_rays,
    );
    aa_context.destroy();
    let free_after_destroy = device_memory().expect("显存查询").0;

    // ---- 相对比值（单场景单机；不宣称绝对性能）----
    let single_median = summarize(&single.frame_ms).1;
    let aa_median = summarize(&aa.frame_ms).1;
    let ratio = aa_median / single_median;
    println!(
        "s8perf[ratio] aa_over_single={ratio:.3} single_ms_per_120k_rays={single_median:.3} \
         aa_ms_per_120k_rays={:.3}",
        aa_median / 4.0
    );
    println!(
        "s8perf[mem] free_bytes: before={free_before} after_create={free_after_create} \
         after_single={free_after_single} after_aa_create={free_after_aa_create} \
         after_destroy={free_after_destroy} context_bytes={} aa_context_bytes={}",
        free_before - free_after_create,
        free_after_single - free_after_aa_create
    );

    // ---- 原始毫秒序列（证据 `raw-timings.json` 由此日志派生；摘要同源）----
    for measure in [&single, &aa] {
        let record = serde_json::json!({
            "mode": measure.label,
            "rays_per_frame": measure.rays,
            "warmup": WARMUP,
            "iterations": ITERATIONS,
            "launch_ms": measure.launch_ms,
            "frame_ms": measure.frame_ms,
        });
        println!("s8perf[raw] {record}");
    }
    println!(
        "s8perf[total] elapsed_s={:.3}",
        started_all.elapsed().as_secs_f64()
    );

    // 基本自检：序列长度、有限、正数；配置不变时同一序列可复算摘要。
    for measure in [&single, &aa] {
        assert_eq!(measure.launch_ms.len(), ITERATIONS);
        assert_eq!(measure.frame_ms.len(), ITERATIONS);
        assert!(
            measure
                .frame_ms
                .iter()
                .all(|value| value.is_finite() && *value > 0.0),
            "{} 的 frame_ms 必须为有限正数",
            measure.label
        );
        assert!(
            measure
                .launch_ms
                .iter()
                .all(|value| value.is_finite() && *value > 0.0),
            "{} 的 launch_ms 必须为有限正数",
            measure.label
        );
    }
    println!("s8perf[done] 单场景单机相对基准完成（不声明绝对性能，不跨硬件/上游比较）");
}

/// 显存工作集：create 后 / 每帧渲染后 free 序列 / destroy 后（S6b 同口径，非泄漏判定）。
///
/// 峰值口径 = `free_before - min(free 序列)`：含 context 占用 + 渲染帧工作集（host 打包
/// 上传的临时缓冲）。destroy 后相对渲染阶段回池量 = context 占用；相对进程起始的差值
/// 属驱动代码缓存/池一次性保留（S6b 先例，非泄漏）。
#[test]
#[ignore = "需要独占 GPU + NVRTC（perf 窗口）"]
fn device_memory_working_set() {
    const FRAMES: usize = 10;
    let session: TransferSession = s7b2::session();
    let fields = s7b2::render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let product = e2e::product_camlight(&session);
    let poses = product.poses;
    let free_before = device_memory().expect("显存查询").0;

    // ---- 单帧（spp=1）----
    let single = {
        let context = s7b2::create_context(&session, &fields, &s7b2::default_params());
        let free_after_create = device_memory().expect("显存查询").0;
        let rays = s7b2::precomputed_rays(&session, &fields);
        let renderer = MegakernelRenderer::new(&session).expect("渲染内核编译");
        let free_after_compile = device_memory().expect("显存查询").0;
        let mut frame_free = Vec::with_capacity(FRAMES);
        for _ in 0..FRAMES {
            renderer
                .render(&context, &fields, &model, &poses, &rays)
                .expect("渲染");
            sync(&context);
            frame_free.push(device_memory().expect("显存查询").0);
        }
        context.destroy();
        let free_after_destroy = device_memory().expect("显存查询").0;
        (
            free_after_create,
            free_after_compile,
            frame_free,
            free_after_destroy,
        )
    };

    // ---- AA（spp=2 → nsamples=4）----
    let aa = {
        let context = s7b3::create_variant_context(&session, &fields, s7b3::AA_VARIANT);
        let free_after_create = device_memory().expect("显存查询").0;
        let rays = s7b3::aa_rays(&session, &fields);
        let renderer = MegakernelRenderer::new(&session).expect("渲染内核编译（模块缓存）");
        let mut frame_free = Vec::with_capacity(FRAMES);
        for _ in 0..FRAMES {
            renderer
                .render(&context, &fields, &model, &poses, &rays)
                .expect("渲染");
            sync(&context);
            frame_free.push(device_memory().expect("显存查询").0);
        }
        context.destroy();
        let free_after_destroy = device_memory().expect("显存查询").0;
        (free_after_create, frame_free, free_after_destroy)
    };
    let free_after_all = device_memory().expect("显存查询").0;

    let single_peak = free_before.saturating_sub(*single.2.iter().min().expect("单帧 free 序列"));
    let aa_peak = free_before.saturating_sub(*aa.1.iter().min().expect("AA free 序列"));
    println!(
        "s8perf[mempeak] single: free_before={free_before} after_create={} after_compile={} \
         peak_bytes={single_peak} frame_free_min={} after_destroy={}",
        single.0,
        single.1,
        single.2.iter().min().expect("min"),
        single.3
    );
    println!(
        "s8perf[mempeak] aa2: after_create={} peak_bytes={aa_peak} frame_free_min={} \
         after_destroy={}",
        aa.0,
        aa.1.iter().min().expect("min"),
        aa.2
    );
    println!("s8perf[mempeak] after_all_destroy={free_after_all}");
    for record in [
        serde_json::json!({
            "mode": "single_spp1",
            "frames": FRAMES,
            "free_before": free_before,
            "free_after_create": single.0,
            "free_after_compile": single.1,
            "frame_free": single.2,
            "free_after_destroy": single.3,
        }),
        serde_json::json!({
            "mode": "aa2_spp2",
            "frames": FRAMES,
            "free_before": free_before,
            "free_after_create": aa.0,
            "frame_free": aa.1,
            "free_after_destroy": aa.2,
        }),
    ] {
        println!("s8perf[memraw] {record}");
    }
    assert_eq!(single.2.len(), FRAMES);
    assert_eq!(aa.1.len(), FRAMES);
    assert!(single_peak > 0 && aa_peak > 0, "显存峰值必须为正");
    println!("s8perf[memdone] 显存工作集完成（相对本机基线，非泄漏判定见 README）");
}
