//! 连续历史布局与 G24@P7 环形历史缓冲切片。
//!
//! `HistoryLayout` 是既有基础辅助（检查扁平历史值与时间间隔），保持行为不变；
//! 子模块 [`buffer`] 交付 G24@P7 的宿主侧环形索引/插入/读取与局部重置。

use crate::diagnostics::InputError;
use crate::model::BatchLayout;
use std::{mem::size_of, ops::Range};

mod buffer;
pub use buffer::{DelayBufferLayout, HISTORY_EPSILON, HistoryBuffer, HistoryError, HistoryInterp};

#[cfg(test)]
mod tests;

/// 连续历史布局 `[world, sample, channel]`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryLayout {
    batch: BatchLayout,
    samples: usize,
    channels: usize,
}

impl HistoryLayout {
    pub fn new(worlds: usize, samples: usize, channels: usize) -> Result<Self, InputError> {
        if samples == 0 {
            return Err(InputError::InvalidDimension { field: "samples" });
        }
        if channels == 0 {
            return Err(InputError::InvalidDimension { field: "channels" });
        }
        let elements = samples.checked_mul(channels).ok_or(InputError::Overflow {
            field: "history_layout",
        })?;
        Ok(Self {
            batch: BatchLayout::new(worlds, elements, size_of::<f32>())?,
            samples,
            channels,
        })
    }

    pub fn batch(self) -> BatchLayout {
        self.batch
    }

    pub fn samples(self) -> usize {
        self.samples
    }

    pub fn channels(self) -> usize {
        self.channels
    }

    pub fn sample_range(self, world: usize, sample: usize) -> Result<Range<usize>, InputError> {
        let world_range = self.batch.world_elements(world)?;
        if sample >= self.samples {
            return Err(InputError::InvalidIndex {
                field: "sample",
                index: sample,
                limit: self.samples,
            });
        }
        let start = world_range.start + sample * self.channels;
        Ok(start..start + self.channels)
    }

    /// 调用方显式提供原生最小间隔。
    /// 允许间隔恰好等于阈值。
    /// 不改写任何历史槽位。
    pub fn validate_times(self, times: &[f64], minimum_interval: f64) -> Result<(), InputError> {
        if !minimum_interval.is_finite() || minimum_interval <= 0.0 {
            return Err(InputError::InvalidDimension {
                field: "minimum_interval",
            });
        }
        if times.len() != self.samples {
            return Err(InputError::LengthMismatch {
                field: "history_times",
                expected: self.samples,
                actual: times.len(),
            });
        }
        for (index, time) in times.iter().enumerate() {
            if !time.is_finite() {
                return Err(InputError::NonFinite {
                    field: "history_times",
                    index,
                });
            }
        }
        for (index, pair) in times.windows(2).enumerate() {
            let interval = pair[1] - pair[0];
            if !interval.is_finite() || interval < minimum_interval {
                return Err(InputError::InvalidHistoryTime { index: index + 1 });
            }
        }
        Ok(())
    }
}
