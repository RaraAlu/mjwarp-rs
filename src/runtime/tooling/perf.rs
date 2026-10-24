//! A10@P11 记录与性能工具：固定测速计划、统计、环境事实与确定性 JSON 记录。
//!
//! 本模块是宿主侧纯数据面，不持有 GPU 句柄，默认构建与单元测试都能编译：
//!
//! - `BenchmarkPlan` 固定温机次数与重复次数（两者都必须大于 0）；
//! - `PhaseTiming`/`TimingStats` 汇总单阶段纳秒样本（min/max/均值/中位数）；
//! - `MemoryStats` 记录设备显存采样（总量/基线/峰值/结束/采样数）；
//! - `PerformanceReport` 汇总环境、计划、阶段记录、显存检查点与未测项，
//!   经 `validate` 校验一致后输出确定性 JSON。
//!
//! CUDA 事件计时与显存采样在 `gpu` 子模块（`cuda-probe` 门控）。
//!
//! 未验收边界：不设性能门槛、不做跨平台或跨设备对照；显存是
//! `cuMemGetInfo` 采样口径，不含内核执行内部的瞬时分配。

use super::ToolingError;

/// 本切片的固定任务标识（写入记录与回执）。
pub const TASK_ID: &str = "2026-10-11-roadmap-a10-p11";

/// 性能记录 JSON 的结构版本。
pub const RECORD_SCHEMA: u32 = 1;

/// 固定测速计划：温机次数与重复次数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BenchmarkPlan {
    pub warmup: u32,
    pub repetitions: u32,
}

impl BenchmarkPlan {
    /// 构造并校验计划：温机与重复次数都必须大于 0。
    pub fn new(warmup: u32, repetitions: u32) -> Result<Self, ToolingError> {
        let plan = Self {
            warmup,
            repetitions,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// 零温机或零重复都拒绝（保证记录可复现）。
    pub fn validate(&self) -> Result<(), ToolingError> {
        if self.warmup == 0 {
            return Err(ToolingError::ZeroWarmup);
        }
        if self.repetitions == 0 {
            return Err(ToolingError::ZeroIterations);
        }
        Ok(())
    }
}

/// 单阶段样本统计（均值/中位数为诊断用 f64，不反馈产品物理结果）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimingStats {
    pub count: usize,
    pub total_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub mean_ns: f64,
    pub median_ns: f64,
}

impl TimingStats {
    /// 由纳秒样本聚合统计；空样本拒绝（与 P10 零迭代口径一致）。
    pub fn from_samples(samples_ns: &[u64]) -> Result<Self, ToolingError> {
        if samples_ns.is_empty() {
            return Err(ToolingError::ZeroIterations);
        }
        let mut sorted = samples_ns.to_vec();
        sorted.sort_unstable();
        let count = sorted.len();
        let total_ns: u64 = sorted.iter().sum();
        let mean_ns = total_ns as f64 / count as f64;
        let median_ns = if count % 2 == 1 {
            sorted[count / 2] as f64
        } else {
            (sorted[count / 2 - 1] as f64 + sorted[count / 2] as f64) / 2.0
        };
        Ok(Self {
            count,
            total_ns,
            min_ns: sorted[0],
            max_ns: sorted[count - 1],
            mean_ns,
            median_ns,
        })
    }
}

/// 单阶段计时：保留逐次样本与聚合统计。
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseTiming {
    pub stats: TimingStats,
    pub samples_ns: Vec<u64>,
}

impl PhaseTiming {
    /// 由逐次样本构造，并立即聚合。
    pub fn from_samples(samples_ns: Vec<u64>) -> Result<Self, ToolingError> {
        let stats = TimingStats::from_samples(&samples_ns)?;
        Ok(Self { stats, samples_ns })
    }

    /// 校验统计与样本一致（防止记录被拼接篡改）。
    pub fn validate(&self) -> Result<(), ToolingError> {
        let recomputed = TimingStats::from_samples(&self.samples_ns)?;
        if recomputed != self.stats {
            return Err(ToolingError::InvalidRecord {
                field: "timing.stats",
                index: 0,
            });
        }
        Ok(())
    }
}

/// 设备显存采样统计（字节；采样口径）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryStats {
    pub total_bytes: u64,
    pub baseline_used_bytes: u64,
    pub peak_used_bytes: u64,
    pub end_used_bytes: u64,
    pub samples: u32,
}

impl MemoryStats {
    /// 校验采样口径自洽：总量非零、至少两个样本、峰值覆盖基线/结束。
    pub fn validate(&self) -> Result<(), ToolingError> {
        if self.total_bytes == 0 {
            return Err(ToolingError::InvalidRecord {
                field: "memory.total_bytes",
                index: 0,
            });
        }
        if self.samples < 2 {
            return Err(ToolingError::InvalidRecord {
                field: "memory.samples",
                index: self.samples as usize,
            });
        }
        if self.baseline_used_bytes > self.total_bytes || self.end_used_bytes > self.total_bytes {
            return Err(ToolingError::InvalidRecord {
                field: "memory.used_bytes",
                index: 0,
            });
        }
        if self.peak_used_bytes < self.baseline_used_bytes
            || self.peak_used_bytes < self.end_used_bytes
        {
            return Err(ToolingError::InvalidRecord {
                field: "memory.peak_used_bytes",
                index: 0,
            });
        }
        Ok(())
    }
}

/// 工作负载级显存检查点（如加载前/分配后/释放后）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryCheckpoint {
    pub label: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

impl MemoryCheckpoint {
    pub fn validate(&self) -> Result<(), ToolingError> {
        if self.label.is_empty() {
            return Err(ToolingError::InvalidRecord {
                field: "checkpoint.label",
                index: 0,
            });
        }
        if self.total_bytes == 0 || self.used_bytes > self.total_bytes {
            return Err(ToolingError::InvalidRecord {
                field: "checkpoint.used_bytes",
                index: 0,
            });
        }
        Ok(())
    }
}

/// 构建档位：与 `cfg!(debug_assertions)` 绑定，debug/release 分开记录。
pub fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

/// 运行环境事实（平台、构建档位与设备）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentFacts {
    pub profile: &'static str,
    pub os: &'static str,
    pub arch: &'static str,
    pub gpu_name: String,
    pub compute_capability: (i32, i32),
    pub driver_api_version: i32,
}

impl EnvironmentFacts {
    /// 由 GPU 会话提供的设备事实补齐平台与构建档位。
    pub fn capture(
        gpu_name: impl Into<String>,
        compute_capability: (i32, i32),
        driver_api_version: i32,
    ) -> Self {
        Self {
            profile: build_profile(),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            gpu_name: gpu_name.into(),
            compute_capability,
            driver_api_version,
        }
    }

    pub fn validate(&self) -> Result<(), ToolingError> {
        if self.profile != "debug" && self.profile != "release" {
            return Err(ToolingError::InvalidRecord {
                field: "environment.profile",
                index: 0,
            });
        }
        if self.gpu_name.is_empty() {
            return Err(ToolingError::InvalidRecord {
                field: "environment.gpu_name",
                index: 0,
            });
        }
        if self.compute_capability.0 <= 0 {
            return Err(ToolingError::InvalidRecord {
                field: "environment.compute_capability",
                index: 0,
            });
        }
        if self.driver_api_version <= 0 {
            return Err(ToolingError::InvalidRecord {
                field: "environment.driver_api_version",
                index: 0,
            });
        }
        Ok(())
    }
}

/// 单阶段性能记录。
#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceRecord {
    pub phase: &'static str,
    pub timing: PhaseTiming,
    pub memory: MemoryStats,
}

impl PerformanceRecord {
    pub fn new(
        phase: &'static str,
        samples_ns: Vec<u64>,
        memory: MemoryStats,
    ) -> Result<Self, ToolingError> {
        if phase.is_empty() {
            return Err(ToolingError::EmptyPhase { name: phase });
        }
        Ok(Self {
            phase,
            timing: PhaseTiming::from_samples(samples_ns)?,
            memory,
        })
    }

    /// 阶段记录必须与固定计划一致（样本数等于重复次数）。
    pub fn validate(&self, plan: &BenchmarkPlan) -> Result<(), ToolingError> {
        if self.phase.is_empty() {
            return Err(ToolingError::EmptyPhase { name: self.phase });
        }
        if self.timing.stats.count != plan.repetitions as usize {
            return Err(ToolingError::InvalidRecord {
                field: "timing.samples_ns",
                index: self.timing.stats.count,
            });
        }
        self.timing.validate()?;
        self.memory.validate()
    }
}

/// 一次完整性能测量报告（环境 + 计划 + 阶段 + 显存检查点 + 未测项）。
#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceReport {
    pub task: String,
    pub workload: String,
    pub environment: EnvironmentFacts,
    pub plan: BenchmarkPlan,
    pub records: Vec<PerformanceRecord>,
    pub checkpoints: Vec<MemoryCheckpoint>,
    pub notes: Vec<String>,
}

impl PerformanceReport {
    pub fn new(
        task: impl Into<String>,
        workload: impl Into<String>,
        environment: EnvironmentFacts,
        plan: BenchmarkPlan,
    ) -> Self {
        Self {
            task: task.into(),
            workload: workload.into(),
            environment,
            plan,
            records: Vec::new(),
            checkpoints: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn push_record(&mut self, record: PerformanceRecord) {
        self.records.push(record);
    }

    pub fn push_checkpoint(&mut self, checkpoint: MemoryCheckpoint) {
        self.checkpoints.push(checkpoint);
    }

    pub fn push_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// 校验报告可复现所需的最小一致面。
    pub fn validate(&self) -> Result<(), ToolingError> {
        if self.task.is_empty() {
            return Err(ToolingError::InvalidRecord {
                field: "report.task",
                index: 0,
            });
        }
        if self.workload.is_empty() {
            return Err(ToolingError::InvalidRecord {
                field: "report.workload",
                index: 0,
            });
        }
        self.environment.validate()?;
        self.plan.validate()?;
        if self.records.is_empty() {
            return Err(ToolingError::InvalidRecord {
                field: "report.records",
                index: 0,
            });
        }
        for (index, record) in self.records.iter().enumerate() {
            record.validate(&self.plan).map_err(|error| match error {
                ToolingError::InvalidRecord { field, .. } => {
                    ToolingError::InvalidRecord { field, index }
                }
                other => other,
            })?;
        }
        if self.checkpoints.is_empty() {
            return Err(ToolingError::InvalidRecord {
                field: "report.checkpoints",
                index: 0,
            });
        }
        for (index, checkpoint) in self.checkpoints.iter().enumerate() {
            checkpoint.validate().map_err(|error| match error {
                ToolingError::InvalidRecord { field, .. } => {
                    ToolingError::InvalidRecord { field, index }
                }
                other => other,
            })?;
        }
        if self.notes.is_empty() {
            return Err(ToolingError::InvalidRecord {
                field: "report.notes",
                index: 0,
            });
        }
        Ok(())
    }

    /// 确定性 JSON 导出：先校验，再按固定字段顺序输出。
    pub fn to_json(&self) -> Result<String, ToolingError> {
        self.validate()?;
        let mut output = String::new();
        output.push_str("{\"schema\":");
        output.push_str(&RECORD_SCHEMA.to_string());
        output.push_str(",\"task\":");
        push_json_string(&mut output, &self.task);
        output.push_str(",\"workload\":");
        push_json_string(&mut output, &self.workload);
        output.push_str(",\"environment\":{\"profile\":");
        push_json_string(&mut output, self.environment.profile);
        output.push_str(",\"os\":");
        push_json_string(&mut output, self.environment.os);
        output.push_str(",\"arch\":");
        push_json_string(&mut output, self.environment.arch);
        output.push_str(",\"gpu\":");
        push_json_string(&mut output, &self.environment.gpu_name);
        output.push_str(&format!(
            ",\"compute_capability\":[{},{}],\"driver_api_version\":{}}}",
            self.environment.compute_capability.0,
            self.environment.compute_capability.1,
            self.environment.driver_api_version
        ));
        output.push_str(&format!(
            ",\"plan\":{{\"warmup\":{},\"repetitions\":{}}}",
            self.plan.warmup, self.plan.repetitions
        ));
        output.push_str(",\"records\":[");
        for (index, record) in self.records.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str("{\"phase\":");
            push_json_string(&mut output, record.phase);
            output.push_str(&format!(
                ",\"timing\":{{\"count\":{},\"total_ns\":{},\"min_ns\":{},\"max_ns\":{},\
                 \"mean_ns\":{},\"median_ns\":{},\"samples_ns\":[",
                record.timing.stats.count,
                record.timing.stats.total_ns,
                record.timing.stats.min_ns,
                record.timing.stats.max_ns,
                record.timing.stats.mean_ns,
                record.timing.stats.median_ns
            ));
            for (sample_index, sample) in record.timing.samples_ns.iter().enumerate() {
                if sample_index > 0 {
                    output.push(',');
                }
                output.push_str(&sample.to_string());
            }
            output.push_str(&format!(
                "]}},\"memory\":{{\"total_bytes\":{},\"baseline_used_bytes\":{},\
                 \"peak_used_bytes\":{},\"end_used_bytes\":{},\"samples\":{}}}",
                record.memory.total_bytes,
                record.memory.baseline_used_bytes,
                record.memory.peak_used_bytes,
                record.memory.end_used_bytes,
                record.memory.samples
            ));
            output.push('}');
        }
        output.push_str("],\"checkpoints\":[");
        for (index, checkpoint) in self.checkpoints.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str("{\"label\":");
            push_json_string(&mut output, &checkpoint.label);
            output.push_str(&format!(
                ",\"used_bytes\":{},\"total_bytes\":{}}}",
                checkpoint.used_bytes, checkpoint.total_bytes
            ));
        }
        output.push_str("],\"notes\":[");
        for (index, note) in self.notes.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_json_string(&mut output, note);
        }
        output.push_str("]}");
        Ok(output)
    }
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                output.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => output.push(other),
        }
    }
    output.push('"');
}
