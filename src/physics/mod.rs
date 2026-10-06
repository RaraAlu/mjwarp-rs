//! 历史字段的宿主布局与校验。
//! 不执行物理阶段或时间插值。

use std::ops::Range;

use crate::diagnostics::InputError;
use crate::model::BatchLayout;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_flattened_history_samples() {
        // MC-38/MC-39：仅检查宿主范围。
        let layout = HistoryLayout::new(2, 3, 4).unwrap();
        assert_eq!(layout.samples(), 3);
        assert_eq!(layout.channels(), 4);
        assert_eq!(layout.batch().total_bytes(), 96);
        assert_eq!(layout.sample_range(1, 2).unwrap(), 20..24);
    }

    #[test]
    fn rejects_history_dimensions_and_indices() {
        for dimensions in [(0, 1, 1), (1, 0, 1), (1, 1, 0), (1, usize::MAX, 2)] {
            assert!(HistoryLayout::new(dimensions.0, dimensions.1, dimensions.2).is_err());
        }
        let layout = HistoryLayout::new(2, 3, 4).unwrap();
        assert!(layout.sample_range(2, 0).is_err());
        assert_eq!(
            layout.sample_range(0, 3),
            Err(InputError::InvalidIndex {
                field: "sample",
                index: 3,
                limit: 3
            })
        );
    }

    #[test]
    fn accepts_exact_minimum_interval_and_single_sample() {
        // MC-38：间隔不能小于原生阈值。
        let layout = HistoryLayout::new(2, 3, 1).unwrap();
        layout.validate_times(&[-0.25, 0.0, 0.5], 0.25).unwrap();
        HistoryLayout::new(1, 1, 1)
            .unwrap()
            .validate_times(&[0.0], 0.25)
            .unwrap();
    }

    #[test]
    fn rejects_short_duplicate_decreasing_and_overflowing_intervals() {
        let layout = HistoryLayout::new(1, 2, 1).unwrap();
        for times in [[0.0, 0.125], [1.0, 1.0], [1.0, 0.0], [-f64::MAX, f64::MAX]] {
            assert_eq!(
                layout.validate_times(&times, 0.25),
                Err(InputError::InvalidHistoryTime { index: 1 })
            );
        }
    }

    #[test]
    fn rejects_invalid_times_length_and_threshold() {
        let layout = HistoryLayout::new(1, 2, 1).unwrap();
        assert_eq!(
            layout.validate_times(&[0.0], 0.25),
            Err(InputError::LengthMismatch {
                field: "history_times",
                expected: 2,
                actual: 1
            })
        );
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                layout.validate_times(&[0.0, invalid], 0.25),
                Err(InputError::NonFinite {
                    field: "history_times",
                    index: 1
                })
            );
        }
        for threshold in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(layout.validate_times(&[0.0, 1.0], threshold).is_err());
        }
    }
}
