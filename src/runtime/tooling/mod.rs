//! A10@P10 记录与性能工具切片：Rust 无头测速与状态导出（宿主侧）。
//!
//! 对齐矩阵 A10「Rust无头测速与状态导出」的宿主侧最小可交付：
//!
//! - **无头测速**：`BenchmarkRecorder` 归一化记录各阶段样本（纳秒整数），
//!   `report()` 产出计数/总计/最小/最大/中位数与均值（f64 仅作诊断统计）；
//!   `measure` 在调用方闭包上做宿主侧计时（无 GPU 内核、无窗口、无 Python）。
//! - **状态导出**：`StateExport`（`time`/`qpos`/`qvel`，f32）导出确定性 JSON
//!   文本，`export_report_json` 导出测速报告；数值用 f32 最短往返格式。
//!
//! 未验收边界：GPU 事件计时/CUDA 图耗时、显存统计、真实仿真批次的性能门槛
//! （需独占 GPU 的 NP04 测量）、CSV/二进制导出格式、与 T4/Linux 的跨平台
//! 性能对照。

use std::error::Error;
use std::fmt;
use std::time::Instant;

/// 一个阶段的样本（纳秒）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseSamples {
    pub name: &'static str,
    pub samples_ns: Vec<u64>,
}

/// 阶段统计（均值/中位数为诊断用 f64）。
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseSummary {
    pub name: &'static str,
    pub count: usize,
    pub total_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub mean_ns: f64,
    pub median_ns: f64,
}

/// 测速报告。
#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkReport {
    pub phases: Vec<PhaseSummary>,
    pub total_ns: u64,
    pub total_samples: usize,
}

/// 状态导出值（f32 物理字段）。
#[derive(Clone, Copy, Debug)]
pub struct StateExport<'a> {
    pub time: f32,
    pub nworld: usize,
    pub qpos: &'a [f32],
    pub qvel: &'a [f32],
}

/// 工具错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolingError {
    EmptyPhase { name: &'static str },
    ZeroIterations,
    InvalidState { field: &'static str, index: usize },
    NonFinite { field: &'static str, index: usize },
}

impl fmt::Display for ToolingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPhase { name } => write!(formatter, "阶段名为空：{name}"),
            Self::ZeroIterations => write!(formatter, "测速迭代次数必须大于 0"),
            Self::InvalidState { field, index } => {
                write!(formatter, "状态长度不匹配：{field}[{index}]")
            }
            Self::NonFinite { field, index } => {
                write!(formatter, "状态含非有限值：{field}[{index}]")
            }
        }
    }
}

impl Error for ToolingError {}

/// 无头测速记录器（宿主侧，无 GPU/窗口/Python）。
#[derive(Clone, Debug, Default)]
pub struct BenchmarkRecorder {
    phases: Vec<PhaseSamples>,
}

impl BenchmarkRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加一次阶段样本（纳秒）。
    pub fn record(&mut self, name: &'static str, nanos: u64) -> Result<(), ToolingError> {
        if name.is_empty() {
            return Err(ToolingError::EmptyPhase { name });
        }
        if let Some(phase) = self.phases.iter_mut().find(|phase| phase.name == name) {
            phase.samples_ns.push(nanos);
            return Ok(());
        }
        self.phases.push(PhaseSamples {
            name,
            samples_ns: vec![nanos],
        });
        Ok(())
    }

    /// 统计报告：按首次出现顺序输出阶段。
    pub fn report(&self) -> BenchmarkReport {
        let mut phases = Vec::with_capacity(self.phases.len());
        let mut total_ns = 0_u64;
        let mut total_samples = 0_usize;
        for phase in &self.phases {
            let mut samples = phase.samples_ns.clone();
            samples.sort_unstable();
            let count = samples.len();
            let sum: u64 = samples.iter().sum();
            let min_ns = samples.first().copied().unwrap_or(0);
            let max_ns = samples.last().copied().unwrap_or(0);
            let mean_ns = if count > 0 {
                sum as f64 / count as f64
            } else {
                0.0
            };
            let median_ns = if count == 0 {
                0.0
            } else if count % 2 == 1 {
                samples[count / 2] as f64
            } else {
                (samples[count / 2 - 1] as f64 + samples[count / 2] as f64) / 2.0
            };
            total_ns += sum;
            total_samples += count;
            phases.push(PhaseSummary {
                name: phase.name,
                count,
                total_ns: sum,
                min_ns,
                max_ns,
                mean_ns,
                median_ns,
            });
        }
        BenchmarkReport {
            phases,
            total_ns,
            total_samples,
        }
    }
}

/// 在宿主闭包上无头测速：返回总纳秒（迭代次数须大于 0）。
pub fn measure<F: FnMut()>(iterations: usize, mut operation: F) -> Result<u64, ToolingError> {
    if iterations == 0 {
        return Err(ToolingError::ZeroIterations);
    }
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    Ok(start.elapsed().as_nanos() as u64)
}

fn push_f32_array(output: &mut String, values: &[f32]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{value}"));
    }
    output.push(']');
}

/// 状态导出为确定性 JSON 文本（f32 最短往返格式）。
pub fn export_state_json(state: &StateExport<'_>) -> Result<String, ToolingError> {
    if !state.time.is_finite() {
        return Err(ToolingError::NonFinite {
            field: "time",
            index: 0,
        });
    }
    if state.nworld == 0 {
        return Err(ToolingError::InvalidState {
            field: "nworld",
            index: 0,
        });
    }
    for (field, values) in [("qpos", state.qpos), ("qvel", state.qvel)] {
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(ToolingError::NonFinite { field, index });
            }
        }
    }
    if state.qpos.len() != state.qvel.len() {
        return Err(ToolingError::InvalidState {
            field: "qpos/qvel",
            index: state.qpos.len(),
        });
    }
    let mut output = String::new();
    output.push_str("{\"time\":");
    output.push_str(&format!("{}", state.time));
    output.push_str(",\"nworld\":");
    output.push_str(&format!("{}", state.nworld));
    output.push_str(",\"qpos\":");
    push_f32_array(&mut output, state.qpos);
    output.push_str(",\"qvel\":");
    push_f32_array(&mut output, state.qvel);
    output.push('}');
    Ok(output)
}

/// 测速报告导出为确定性 JSON 文本。
pub fn export_report_json(report: &BenchmarkReport) -> String {
    let mut output = String::new();
    output.push_str("{\"total_samples\":");
    output.push_str(&format!("{}", report.total_samples));
    output.push_str(",\"total_ns\":");
    output.push_str(&format!("{}", report.total_ns));
    output.push_str(",\"phases\":[");
    for (index, phase) in report.phases.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"name\":\"{}\",\"count\":{},\"total_ns\":{},\"min_ns\":{},\"max_ns\":{},\
             \"mean_ns\":{},\"median_ns\":{}}}",
            phase.name,
            phase.count,
            phase.total_ns,
            phase.min_ns,
            phase.max_ns,
            phase.mean_ns,
            phase.median_ns
        ));
    }
    output.push_str("]}");
    output
}

#[cfg(test)]
mod tests;
