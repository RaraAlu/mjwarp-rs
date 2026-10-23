//! G24@P7 环形历史缓冲读取：零阶保持/线性插值、端点夹取与延迟读取。
//!
//! 对齐冻结上游 `history._history_read_scalar`/`_history_read_vector` 与
//! `_read_ctrl_delayed_kernel`/`_read_sensor_kernel` 的 `t - delay` 读取。

use super::*;

impl HistoryBuffer {
    /// 读取标量样本（上游 `_history_read_scalar`）。
    pub fn read_scalar(
        &self,
        world: usize,
        t: f32,
        interp: HistoryInterp,
    ) -> Result<f32, HistoryError> {
        checked_time(t)?;
        if self.layout.channels() != 1 {
            return Err(HistoryError::ScalarReadOnVector {
                channels: self.layout.channels(),
            });
        }
        let start = self.region_start(world)?;
        let samples = self.layout.samples();
        let cursor = self.cursor(world)?;
        let times = self.layout.times_range();
        let values = self.layout.values_offset();
        let value_at =
            |logical: usize| start + values + Self::physical_index(cursor, samples, logical);
        let time_at =
            |logical: usize| start + times.start + Self::physical_index(cursor, samples, logical);
        let oldest_time = self.words[time_at(0)];
        let newest_time = self.words[time_at(samples - 1)];
        if t <= oldest_time + HISTORY_EPSILON {
            return Ok(self.words[value_at(0)]);
        }
        if t >= newest_time - HISTORY_EPSILON {
            return Ok(self.words[value_at(samples - 1)]);
        }
        let index = self.find_index(world, t)?;
        let physical = Self::physical_index(cursor, samples, index);
        if (t - self.words[start + times.start + physical]).abs() < HISTORY_EPSILON {
            return Ok(self.words[start + values + physical]);
        }
        let low = Self::physical_index(cursor, samples, index - 1);
        if interp == HistoryInterp::ZeroOrderHold {
            return Ok(self.words[start + values + low]);
        }
        let dt = self.words[start + times.start + physical] - self.words[start + times.start + low];
        let alpha = (t - self.words[start + times.start + low]) / dt;
        let low_value = self.words[start + values + low];
        let high_value = self.words[start + values + physical];
        Ok(low_value + alpha * (high_value - low_value))
    }

    /// 读取向量样本（上游 `_history_read_vector`），结果写入 `out`。
    pub fn read_vector(
        &self,
        world: usize,
        t: f32,
        interp: HistoryInterp,
        out: &mut [f32],
    ) -> Result<(), HistoryError> {
        checked_time(t)?;
        let channels = self.layout.channels();
        if out.len() != channels {
            return Err(HistoryError::LengthMismatch {
                field: "out",
                expected: channels,
                actual: out.len(),
            });
        }
        let start = self.region_start(world)?;
        let samples = self.layout.samples();
        let cursor = self.cursor(world)?;
        let times = self.layout.times_range();
        let values = self.layout.values_offset();
        let time_at =
            |logical: usize| start + times.start + Self::physical_index(cursor, samples, logical);
        let value_at = |logical: usize| {
            start + values + Self::physical_index(cursor, samples, logical) * channels
        };
        let oldest_time = self.words[time_at(0)];
        let newest_time = self.words[time_at(samples - 1)];
        if t <= oldest_time + HISTORY_EPSILON {
            out.copy_from_slice(&self.words[value_at(0)..value_at(0) + channels]);
            return Ok(());
        }
        if t >= newest_time - HISTORY_EPSILON {
            out.copy_from_slice(
                &self.words[value_at(samples - 1)..value_at(samples - 1) + channels],
            );
            return Ok(());
        }
        let index = self.find_index(world, t)?;
        let physical = Self::physical_index(cursor, samples, index);
        if (t - self.words[start + times.start + physical]).abs() < HISTORY_EPSILON {
            out.copy_from_slice(&self.words[value_at(index)..value_at(index) + channels]);
            return Ok(());
        }
        let low = Self::physical_index(cursor, samples, index - 1);
        if interp == HistoryInterp::ZeroOrderHold {
            out.copy_from_slice(&self.words[value_at(index - 1)..value_at(index - 1) + channels]);
            return Ok(());
        }
        let dt = self.words[start + times.start + physical] - self.words[start + times.start + low];
        let alpha = (t - self.words[start + times.start + low]) / dt;
        for (channel, value) in out.iter_mut().enumerate() {
            let low_value = self.words[start + values + low * channels + channel];
            let high_value = self.words[start + values + physical * channels + channel];
            *value = low_value + alpha * (high_value - low_value);
        }
        Ok(())
    }

    /// 延迟读取：在上游 `_read_ctrl_delayed_kernel`/`_read_sensor_kernel` 处
    /// 按 `t - delay` 读取。
    pub fn read_delayed(
        &self,
        world: usize,
        t: f32,
        delay: f32,
        interp: HistoryInterp,
        out: &mut [f32],
    ) -> Result<(), HistoryError> {
        if !delay.is_finite() {
            return Err(HistoryError::NonFinite {
                field: "delay",
                index: 0,
            });
        }
        self.read_vector(world, t - delay, interp, out)
    }
}
