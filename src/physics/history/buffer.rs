//! G24@P7 历史与延迟切片：环形缓冲索引、插入、读取与局部重置。
//!
//! 对齐冻结上游 `mujoco_warp._src.history`（G24；P7；提交
//! `71da24d956…`，Apache-2.0）：
//!
//! - `_history_physical_index`：逻辑索引（0=最旧，n-1=最新）到物理槽位
//!   `(cursor + 1 + logical) % n` 的映射；`cursor` 指向最新样本的物理槽位。
//! - `_history_find_index`：环形二分，返回满足 `times[i-1] < t <= times[i]`
//!   的逻辑索引；`t <= times[oldest]` 返回 0，`t > times[newest]` 返回 n。
//! - `_history_insert_scalar`/`_history_insert_vector`：精确匹配替换、
//!   早于最旧替换最旧、新于最新推进 `cursor`、乱序左移插入四条路径。
//! - `_history_read_scalar`/`_history_read_vector`：端点外夹取到最旧/最新、
//!   精确匹配（`|t - times[i]| < 1e-6`）直取、零阶保持（interp=0）与
//!   线性插值（interp=1）。
//! - `_reset_actuator_history_kernel`/`_reset_sensor_history_kernel`：
//!   按世界掩码的局部重置；执行器为 `user=0`、`cursor=n-1`、
//!   `times[j]=-(n-j)*dt`、值清零（SO3+QUAT 每个样本置 1）；传感器为
//!   delay 分支（`user=-dt`、`times[j]=-(n-j)*dt`）与 interval 分支
//!   （`user=t0`、`times[j]=ceil((t0-(n-1-j)*period)/dt)*dt`）。
//!
//! 每世界区域布局与上游 `d.history` 一致：
//! `[user, cursor, times(samples), values(samples*channels)]`。
//!
//! 未验收边界：三次插值（interp=2 Catmull-Rom）、`init_ctrl_history`/
//! `init_sensor_history` 公共初始化入口、interval 运行期门控
//! （`_insert_sensor_history_stage` 的 period 推进）、`apply_sensor_delay`/
//! `read_ctrl`/`read_sensor` 的 forward 调度封装与 sensor 全类型、
//! `d.history` 全量分配与 actuator/sensor registry 组装、GPU 内核与图执行。

use std::{error::Error, fmt, ops::Range};

/// 上游精确匹配阈值（`wp.abs(t - times[i]) < 1e-6`，f32）。
pub const HISTORY_EPSILON: f32 = 1e-6;

/// 历史缓冲每世界区域布局：`[user, cursor, times(n), values(n*channels)]`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DelayBufferLayout {
    samples: usize,
    channels: usize,
}

impl DelayBufferLayout {
    /// 区域头部槽位数：`user` 槽与 `cursor` 槽。
    pub const HEADER: usize = 2;

    /// 校验并构造布局；任一维度为 0 或总长度溢出时拒绝。
    pub fn new(samples: usize, channels: usize) -> Result<Self, HistoryError> {
        if samples == 0 {
            return Err(HistoryError::InvalidDimension { field: "samples" });
        }
        if channels == 0 {
            return Err(HistoryError::InvalidDimension { field: "channels" });
        }
        samples
            .checked_mul(channels)
            .ok_or(HistoryError::InvalidDimension {
                field: "history_values",
            })?;
        Ok(Self { samples, channels })
    }

    /// 每区域样本数 `n`。
    pub fn samples(self) -> usize {
        self.samples
    }

    /// 每样本通道数 `dim`。
    pub fn channels(self) -> usize {
        self.channels
    }

    /// 值区起点（相对区域起点）：`2 + n`。
    pub fn values_offset(self) -> usize {
        Self::HEADER + self.samples
    }

    /// 每世界区域总槽位数：`2 + n + n*channels`。
    pub fn region_elements(self) -> usize {
        self.values_offset() + self.samples * self.channels
    }

    /// 时间戳槽位范围（相对区域起点）。
    pub fn times_range(self) -> Range<usize> {
        Self::HEADER..self.values_offset()
    }
}

/// 读取插值模式（上游 `interp`：0=零阶保持，1=线性）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryInterp {
    /// 零阶保持：返回 `times[i-1]` 处的值。
    ZeroOrderHold,
    /// 线性插值：在 `times[i-1]` 与 `times[i]` 之间做线性插值。
    Linear,
}

impl HistoryInterp {
    /// 把上游整数 `interp` 转换为插值模式；三次插值（2）未实现并显式拒绝。
    pub fn from_raw(interp: i32) -> Result<Self, HistoryError> {
        match interp {
            0 => Ok(Self::ZeroOrderHold),
            1 => Ok(Self::Linear),
            other => Err(HistoryError::UnsupportedInterpolation { interp: other }),
        }
    }
}

/// 历史缓冲错误。
#[derive(Clone, Debug, PartialEq)]
pub enum HistoryError {
    /// 维度为 0 或长度溢出。
    InvalidDimension { field: &'static str },
    /// 字段长度与维度不一致。
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 索引越界。
    InvalidIndex {
        field: &'static str,
        index: usize,
        limit: usize,
    },
    /// 数值非有限。
    NonFinite { field: &'static str, index: usize },
    /// `cursor` 槽不在 `[0, samples)`。
    InvalidCursor { cursor: f32, samples: usize },
    /// 未实现的插值模式（三次插值）。
    UnsupportedInterpolation { interp: i32 },
    /// 对多通道缓冲调用标量读取。
    ScalarReadOnVector { channels: usize },
    /// 重置时间步非正或非有限。
    InvalidTimestep { timestep: f32 },
}

impl fmt::Display for HistoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimension { field } => write!(formatter, "维度必须为正：{field}"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "长度不匹配：{field} 期望 {expected}，实际 {actual}"
            ),
            Self::InvalidIndex {
                field,
                index,
                limit,
            } => write!(formatter, "索引越界：{field}[{index}]，上限 {limit}"),
            Self::NonFinite { field, index } => {
                write!(formatter, "数值必须有限：{field}[{index}]")
            }
            Self::InvalidCursor { cursor, samples } => {
                write!(formatter, "历史 cursor 越界：{cursor} 不在 [0, {samples})")
            }
            Self::UnsupportedInterpolation { interp } => {
                write!(
                    formatter,
                    "未实现的历史插值模式：interp={interp}（只支持 0/1）"
                )
            }
            Self::ScalarReadOnVector { channels } => {
                write!(formatter, "标量读取要求单通道缓冲，实际通道数 {channels}")
            }
            Self::InvalidTimestep { timestep } => {
                write!(formatter, "重置时间步必须为正有限数：{timestep}")
            }
        }
    }
}

impl Error for HistoryError {}

fn checked_time(t: f32) -> Result<(), HistoryError> {
    if t.is_finite() {
        Ok(())
    } else {
        Err(HistoryError::NonFinite {
            field: "t",
            index: 0,
        })
    }
}

/// 历史环形缓冲：`worlds` 个独立的每世界区域。
#[derive(Clone, Debug, PartialEq)]
pub struct HistoryBuffer {
    worlds: usize,
    layout: DelayBufferLayout,
    words: Vec<f32>,
}

impl HistoryBuffer {
    /// 构造全零缓冲；调用方须自行执行 `reset_*` 得到上游初始状态。
    pub fn new(worlds: usize, samples: usize, channels: usize) -> Result<Self, HistoryError> {
        if worlds == 0 {
            return Err(HistoryError::InvalidDimension { field: "worlds" });
        }
        let layout = DelayBufferLayout::new(samples, channels)?;
        let region = layout.region_elements();
        let elements = worlds
            .checked_mul(region)
            .ok_or(HistoryError::InvalidDimension {
                field: "history_layout",
            })?;
        Ok(Self {
            worlds,
            layout,
            words: vec![0.0; elements],
        })
    }

    /// 由冻结的扁平字（`[world, region]`）构造缓冲。
    pub fn from_words(
        worlds: usize,
        samples: usize,
        channels: usize,
        words: Vec<f32>,
    ) -> Result<Self, HistoryError> {
        let buffer = Self::new(worlds, samples, channels)?;
        let expected = buffer.words.len();
        if words.len() != expected {
            return Err(HistoryError::LengthMismatch {
                field: "history_words",
                expected,
                actual: words.len(),
            });
        }
        Ok(Self { words, ..buffer })
    }

    /// 世界数。
    pub fn worlds(&self) -> usize {
        self.worlds
    }

    /// 每世界区域布局。
    pub fn layout(&self) -> DelayBufferLayout {
        self.layout
    }

    /// 扁平字（`[world, region]`）。
    pub fn words(&self) -> &[f32] {
        &self.words
    }

    /// 逻辑索引（0=最旧，n-1=最新）到物理槽位的映射。
    pub fn physical_index(cursor: usize, samples: usize, logical: usize) -> usize {
        (cursor + 1 + logical) % samples
    }

    fn region_start(&self, world: usize) -> Result<usize, HistoryError> {
        if world >= self.worlds {
            return Err(HistoryError::InvalidIndex {
                field: "world",
                index: world,
                limit: self.worlds,
            });
        }
        Ok(world * self.layout.region_elements())
    }

    /// 读取 `cursor` 槽（最新样本的物理槽位）。
    pub fn cursor(&self, world: usize) -> Result<usize, HistoryError> {
        let start = self.region_start(world)?;
        let raw = self.words[start + 1];
        if !raw.is_finite() || raw < 0.0 || raw >= self.layout.samples() as f32 {
            return Err(HistoryError::InvalidCursor {
                cursor: raw,
                samples: self.layout.samples(),
            });
        }
        Ok(raw as usize)
    }

    /// 环形二分：返回满足 `times[i-1] < t <= times[i]` 的逻辑索引。
    ///
    /// `t <= times[oldest]` 返回 0，`t > times[newest]` 返回 `n`。
    pub fn find_index(&self, world: usize, t: f32) -> Result<usize, HistoryError> {
        checked_time(t)?;
        let start = self.region_start(world)?;
        let samples = self.layout.samples();
        let cursor = self.cursor(world)?;
        let times = self.layout.times_range();
        let time_at =
            |logical: usize| start + times.start + Self::physical_index(cursor, samples, logical);
        let oldest = time_at(0);
        let newest = time_at(samples - 1);
        if t <= self.words[oldest] {
            return Ok(0);
        }
        if t > self.words[newest] {
            return Ok(samples);
        }
        let mut lo = 0;
        let mut hi = samples - 1;
        while hi - lo > 1 {
            let mid = (lo + hi) >> 1;
            if self.words[time_at(mid)] < t {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Ok(hi)
    }

    /// 插入标量样本（上游 `_history_insert_scalar`）。
    pub fn insert_scalar(&mut self, world: usize, t: f32, value: f32) -> Result<(), HistoryError> {
        checked_time(t)?;
        if !value.is_finite() {
            return Err(HistoryError::NonFinite {
                field: "value",
                index: 0,
            });
        }
        let start = self.region_start(world)?;
        let samples = self.layout.samples();
        let cursor = self.cursor(world)?;
        let times = self.layout.times_range();
        let values = self.layout.values_offset();
        let index = self.find_index(world, t)?;
        if index < samples {
            let physical = Self::physical_index(cursor, samples, index);
            if (t - self.words[start + times.start + physical]).abs() < HISTORY_EPSILON {
                self.words[start + times.start + physical] = t;
                self.words[start + values + physical] = value;
                return Ok(());
            }
        }
        if index == 0 {
            let physical = Self::physical_index(cursor, samples, 0);
            self.words[start + times.start + physical] = t;
            self.words[start + values + physical] = value;
            return Ok(());
        }
        if index == samples {
            let next = (cursor + 1) % samples;
            self.words[start + 1] = next as f32;
            self.words[start + times.start + next] = t;
            self.words[start + values + next] = value;
            return Ok(());
        }
        for j in 0..index - 1 {
            let source = Self::physical_index(cursor, samples, j + 1);
            let destination = Self::physical_index(cursor, samples, j);
            self.words[start + times.start + destination] =
                self.words[start + times.start + source];
            self.words[start + values + destination] = self.words[start + values + source];
        }
        let slot = Self::physical_index(cursor, samples, index - 1);
        self.words[start + times.start + slot] = t;
        self.words[start + values + slot] = value;
        Ok(())
    }

    /// 插入向量样本（上游 `_history_insert_vector`）。
    pub fn insert_vector(
        &mut self,
        world: usize,
        t: f32,
        values: &[f32],
    ) -> Result<(), HistoryError> {
        checked_time(t)?;
        let channels = self.layout.channels();
        if values.len() != channels {
            return Err(HistoryError::LengthMismatch {
                field: "values",
                expected: channels,
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(HistoryError::NonFinite {
                    field: "values",
                    index,
                });
            }
        }
        let start = self.region_start(world)?;
        let samples = self.layout.samples();
        let cursor = self.cursor(world)?;
        let times = self.layout.times_range();
        let values_offset = self.layout.values_offset();
        let index = self.find_index(world, t)?;
        let mut slot = None;
        if index < samples {
            let physical = Self::physical_index(cursor, samples, index);
            if (t - self.words[start + times.start + physical]).abs() < HISTORY_EPSILON {
                slot = Some(physical);
            }
        }
        let slot = if let Some(slot) = slot {
            slot
        } else if index == 0 {
            let physical = Self::physical_index(cursor, samples, 0);
            self.words[start + times.start + physical] = t;
            physical
        } else if index == samples {
            let next = (cursor + 1) % samples;
            self.words[start + 1] = next as f32;
            self.words[start + times.start + next] = t;
            next
        } else {
            for j in 0..index - 1 {
                let source = Self::physical_index(cursor, samples, j + 1);
                let destination = Self::physical_index(cursor, samples, j);
                self.words[start + times.start + destination] =
                    self.words[start + times.start + source];
                for channel in 0..channels {
                    let value = self.words[start + values_offset + source * channels + channel];
                    self.words[start + values_offset + destination * channels + channel] = value;
                }
            }
            let physical = Self::physical_index(cursor, samples, index - 1);
            self.words[start + times.start + physical] = t;
            physical
        };
        for (channel, value) in values.iter().enumerate() {
            self.words[start + values_offset + slot * channels + channel] = *value;
        }
        Ok(())
    }
}

mod read;
mod reset;
