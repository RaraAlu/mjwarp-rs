//! G24@P7 环形历史缓冲局部重置：执行器与传感器初始状态。
//!
//! 对齐冻结上游 `history._reset_actuator_history_kernel` 与
//! `history._reset_sensor_history_kernel` 的按世界掩码重置。

use super::*;

fn checked_timestep(timestep: f32) -> Result<(), HistoryError> {
    if timestep.is_finite() && timestep > 0.0 {
        Ok(())
    } else {
        Err(HistoryError::InvalidTimestep { timestep })
    }
}

impl HistoryBuffer {
    /// 执行器历史局部重置（上游 `_reset_actuator_history_kernel`）。
    pub fn reset_actuator(
        &mut self,
        mask: Option<&[bool]>,
        timestep: f32,
        so3_quat: bool,
    ) -> Result<(), HistoryError> {
        checked_timestep(timestep)?;
        let selected = self.selected_worlds(mask)?;
        let samples = self.layout.samples();
        let channels = self.layout.channels();
        let values = self.layout.values_offset();
        let region = self.layout.region_elements();
        for world in selected {
            let start = world * region;
            self.words[start] = 0.0;
            self.words[start + 1] = (samples - 1) as f32;
            let times = &mut self.words[start + DelayBufferLayout::HEADER..start + values];
            for (index, time) in times.iter_mut().enumerate() {
                *time = -((samples - index) as f32) * timestep;
            }
            self.words[start + values..start + region].fill(0.0);
            if so3_quat {
                for index in 0..samples {
                    self.words[start + values + index * channels] = 1.0;
                }
            }
        }
        Ok(())
    }

    /// 传感器历史局部重置（上游 `_reset_sensor_history_kernel`）。
    ///
    /// `interval = [period, phase]`；`period > 0` 走 interval 时间戳分支，
    /// 否则走 delay 分支（`user = -dt`）。
    pub fn reset_sensor(
        &mut self,
        mask: Option<&[bool]>,
        timestep: f32,
        interval: [f32; 2],
    ) -> Result<(), HistoryError> {
        checked_timestep(timestep)?;
        let [period, phase] = interval;
        for (index, value) in interval.iter().enumerate() {
            if !value.is_finite() {
                return Err(HistoryError::NonFinite {
                    field: "interval",
                    index,
                });
            }
        }
        let selected = self.selected_worlds(mask)?;
        let samples = self.layout.samples();
        let values = self.layout.values_offset();
        let region = self.layout.region_elements();
        for world in selected {
            let start = world * region;
            if period > 0.0 {
                let base = if phase != 0.0 { phase } else { -period };
                self.words[start] = base;
                let times = &mut self.words[start + DelayBufferLayout::HEADER..start + values];
                for (index, time) in times.iter_mut().enumerate() {
                    let continuous = base - ((samples - 1 - index) as f32) * period;
                    *time = (continuous / timestep).ceil() * timestep;
                }
            } else {
                self.words[start] = -timestep;
                let times = &mut self.words[start + DelayBufferLayout::HEADER..start + values];
                for (index, time) in times.iter_mut().enumerate() {
                    *time = -((samples - index) as f32) * timestep;
                }
            }
            self.words[start + 1] = (samples - 1) as f32;
            self.words[start + values..start + region].fill(0.0);
        }
        Ok(())
    }

    fn selected_worlds(&self, mask: Option<&[bool]>) -> Result<Vec<usize>, HistoryError> {
        let Some(mask) = mask else {
            return Ok((0..self.worlds).collect());
        };
        if mask.len() != self.worlds {
            return Err(HistoryError::LengthMismatch {
                field: "reset",
                expected: self.worlds,
                actual: mask.len(),
            });
        }
        Ok((0..self.worlds).filter(|world| mask[*world]).collect())
    }
}
