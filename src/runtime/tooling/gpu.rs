//! A10@P11 CUDA 事件计时与设备显存采样（`cuda-probe` 门控；需 NVIDIA 驱动）。
//!
//! - **事件计时**：在会话流上用 `cuEventRecord` 夹住调用方阶段闭包，
//!   `cuEventElapsedTime` 读取毫秒后换算为纳秒整数；每次重复独立起止。
//! - **显存采样**：`cuMemGetInfo` 读取设备空闲/总量，按「已用 = 总量 - 空闲」
//!   统计基线、峰值与结束值，采样点包括温机前后、每次重复后与阶段结束。
//! - **固定条件**：温机与重复次数由 `BenchmarkPlan` 固定并写入记录。
//!
//! 未验收边界：显存是采样口径（不含内核执行内部的瞬时分配；WDDM 下其他进程
//! 占用会进入空闲读数）；事件计时只覆盖设备流上已提交的工作；不含跨设备/
//! 跨平台对照，也不构成性能验收门槛。

use std::sync::Arc;

use cudarc::driver::{CudaContext, CudaFunction, CudaStream, DriverError, result, sys};
use cudarc::nvrtc::Ptx;

use super::ToolingError;
use super::perf::{BenchmarkPlan, EnvironmentFacts, MemoryStats, PhaseTiming};

fn cuda(stage: &'static str) -> impl FnOnce(DriverError) -> ToolingError {
    move |error| ToolingError::Cuda {
        stage,
        code: error.0 as u32,
    }
}

/// 设备环境事实（会话创建时一次性采集）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuDeviceFacts {
    pub ordinal: usize,
    pub name: String,
    pub compute_capability: (i32, i32),
    pub driver_api_version: i32,
}

/// 设备显存快照（字节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceMemorySample {
    pub total_bytes: u64,
    pub free_bytes: u64,
}

impl DeviceMemorySample {
    /// 已用字节（驱动报告的全局设备占用，含本进程与其他进程）。
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.free_bytes)
    }
}

/// 独立 GPU 计时会话：自带上下文、流与设备事实。
///
/// 一次测量独占一个上下文与一条流；调用方在流上提交被计时的阶段。
pub struct GpuTimingSession {
    context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    facts: GpuDeviceFacts,
}

impl GpuTimingSession {
    /// 创建会话：核对驱动库、设备序号、驱动 API 版本并采集设备事实。
    pub fn new(ordinal: usize) -> Result<Self, ToolingError> {
        // SAFETY: 只探测 NVIDIA 驱动库是否存在，不加载用户代码、不解引用用户指针。
        if !unsafe { sys::is_culib_present() } {
            return Err(ToolingError::Cuda {
                stage: "driver-unavailable",
                code: 0,
            });
        }
        let mut driver_api_version = 0;
        // SAFETY: 驱动写入有效 i32 地址；变量在调用期间保持存活。
        unsafe { sys::cuDriverGetVersion(&mut driver_api_version) }
            .result()
            .map_err(cuda("driver-version"))?;
        let available = CudaContext::device_count().map_err(cuda("device-count"))?;
        if ordinal as i32 >= available {
            return Err(ToolingError::DeviceUnavailable {
                requested: ordinal,
                available,
            });
        }
        let context = CudaContext::new(ordinal).map_err(cuda("context"))?;
        let name = context.name().map_err(cuda("device-name"))?;
        let (major, minor) = context
            .compute_capability()
            .map_err(cuda("compute-capability"))?;
        let stream = context.new_stream().map_err(cuda("stream"))?;
        Ok(Self {
            context,
            stream,
            facts: GpuDeviceFacts {
                ordinal,
                name,
                compute_capability: (major, minor),
                driver_api_version,
            },
        })
    }

    pub fn facts(&self) -> &GpuDeviceFacts {
        &self.facts
    }

    /// 记录用环境事实（平台 + 构建档位 + 设备）。
    pub fn environment(&self) -> EnvironmentFacts {
        EnvironmentFacts::capture(
            self.facts.name.clone(),
            self.facts.compute_capability,
            self.facts.driver_api_version,
        )
    }

    pub fn context(&self) -> &Arc<CudaContext> {
        &self.context
    }

    /// 阶段闭包提交工作所在的流。
    pub fn stream(&self) -> &Arc<CudaStream> {
        &self.stream
    }

    /// 加载固定 PTX 内核（调用方保证 PTX 与入口匹配其固定输入）。
    pub fn load_kernel(
        &self,
        ptx: &str,
        entry: &'static str,
    ) -> Result<CudaFunction, ToolingError> {
        let module = self
            .context
            .load_module(Ptx::from_src(ptx))
            .map_err(cuda("module-load"))?;
        module.load_function(entry).map_err(cuda("function-load"))
    }

    /// 采样一次设备显存。
    pub fn sample_memory(&self) -> Result<DeviceMemorySample, ToolingError> {
        self.context
            .bind_to_thread()
            .map_err(cuda("bind-context"))?;
        let (free, total) = result::mem_get_info().map_err(cuda("mem-get-info"))?;
        Ok(DeviceMemorySample {
            total_bytes: total as u64,
            free_bytes: free as u64,
        })
    }

    /// 按固定计划计时一个 GPU 阶段，并统计该阶段的显存采样。
    ///
    /// 先执行 `plan.warmup` 次温机并同步，再对 `plan.repetitions` 次重复各自
    /// 记录起止事件；每次重复后与阶段结束各采样一次显存。闭包只在
    /// `cuda-probe` 已确认的会话流上提交工作。
    pub fn time_phase<F>(
        &self,
        plan: &BenchmarkPlan,
        mut launch: F,
    ) -> Result<(PhaseTiming, MemoryStats), ToolingError>
    where
        F: FnMut(&Arc<CudaStream>) -> Result<(), DriverError>,
    {
        plan.validate()?;
        let start_event = self
            .context
            .new_event(Some(sys::CUevent_flags::CU_EVENT_DEFAULT))
            .map_err(cuda("event-create"))?;
        let end_event = self
            .context
            .new_event(Some(sys::CUevent_flags::CU_EVENT_DEFAULT))
            .map_err(cuda("event-create"))?;
        let baseline = self.sample_memory()?;
        let mut used_samples = vec![baseline.used_bytes()];
        for _ in 0..plan.warmup {
            launch(&self.stream).map_err(cuda("warmup-launch"))?;
        }
        self.stream.synchronize().map_err(cuda("warmup-wait"))?;
        used_samples.push(self.sample_memory()?.used_bytes());
        let mut samples_ns = Vec::with_capacity(plan.repetitions as usize);
        for _ in 0..plan.repetitions {
            start_event
                .record(&self.stream)
                .map_err(cuda("event-record"))?;
            launch(&self.stream).map_err(cuda("phase-launch"))?;
            end_event
                .record(&self.stream)
                .map_err(cuda("event-record"))?;
            let elapsed_ms = start_event
                .elapsed_ms(&end_event)
                .map_err(cuda("event-elapsed"))?;
            samples_ns.push(elapsed_ms_to_nanos(elapsed_ms)?);
            used_samples.push(self.sample_memory()?.used_bytes());
        }
        self.stream.synchronize().map_err(cuda("phase-wait"))?;
        used_samples.push(self.sample_memory()?.used_bytes());
        let timing = PhaseTiming::from_samples(samples_ns)?;
        let end_used_bytes = used_samples.last().copied().unwrap_or(0);
        let peak_used_bytes = used_samples
            .iter()
            .copied()
            .max()
            .unwrap_or(baseline.used_bytes());
        let memory = MemoryStats {
            total_bytes: baseline.total_bytes,
            baseline_used_bytes: baseline.used_bytes(),
            peak_used_bytes,
            end_used_bytes,
            samples: used_samples.len() as u32,
        };
        Ok((timing, memory))
    }
}

fn elapsed_ms_to_nanos(millis: f32) -> Result<u64, ToolingError> {
    let nanos = f64::from(millis) * 1_000_000.0;
    if !nanos.is_finite() || nanos < 0.0 {
        return Err(ToolingError::NonFinite {
            field: "event.elapsed_ms",
            index: 0,
        });
    }
    Ok(nanos.round() as u64)
}
