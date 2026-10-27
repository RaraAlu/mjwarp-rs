//! 多 world 输出区段（R10-S1 观察收口拆分）：在既有 `OutputLayout` 上按 `nworld` 乘法展开。

use std::ops::Range;

use crate::render::ImageOutput;
use crate::render::camera::CameraPlan;

use super::super::OutputLayout;
use super::errors::{
    WorldError, active_mask_all_true, check_camera_range, check_world_range, pixels_of,
};

/// 多 world 输出区段：在既有 [`OutputLayout`] 上按 `nworld` 乘法展开（world-major）。
///
/// 区段为 `[world * pixels, (world + 1) * pixels)`，各 world 互不重叠；`nworld == 1` 时
/// 与单 world 布局逐位一致（`0..pixels`）。
#[derive(Clone, Debug, PartialEq)]
pub struct WorldLayout {
    plan: CameraPlan,
    layout: OutputLayout,
    worlds: usize,
    active: Vec<bool>,
}

impl WorldLayout {
    /// 按 `plan` 解析单 world 布局后展开到 `nworld` 个 world（全部激活）。
    ///
    /// 错误优先级（观察收口 F3）：`nworld == 0` → `WorldCountZero` **先于** `OutputLayout` 解析；
    /// 极大 `nworld` 走 `try_reserve` → `CountOverflow`（不 capacity-overflow panic，F2）。
    pub fn resolve(plan: &CameraPlan, worlds: usize) -> Result<Self, WorldError> {
        if worlds == 0 {
            return Err(WorldError::WorldCountZero);
        }
        let layout = OutputLayout::resolve(plan)?;
        let active = active_mask_all_true(worlds)?;
        Ok(Self {
            plan: plan.clone(),
            layout,
            worlds,
            active,
        })
    }

    /// 设置激活掩码（长度必须等于 `worlds`）。
    pub fn with_active(mut self, active: Vec<bool>) -> Result<Self, WorldError> {
        if active.len() != self.worlds {
            return Err(WorldError::ActiveMaskMismatch {
                expected: self.worlds,
                actual: active.len(),
            });
        }
        self.active = active;
        Ok(self)
    }

    pub fn worlds(&self) -> usize {
        self.worlds
    }

    pub fn cameras(&self) -> usize {
        self.plan.resolutions.len()
    }

    pub fn layout(&self) -> &OutputLayout {
        &self.layout
    }

    pub fn is_active(&self, world: usize) -> Result<bool, WorldError> {
        check_world_range(world, self.worlds)?;
        Ok(self.active[world])
    }

    /// 单 world 的通道元素数（`OutputLayout` 字段）。
    pub fn channel_pixels(&self, channel: ImageOutput) -> usize {
        match channel {
            ImageOutput::Rgb => self.layout.rgb_pixels,
            ImageOutput::Depth => self.layout.depth_pixels,
            ImageOutput::Segmentation => self.layout.seg_pixels,
        }
    }

    /// 整缓冲元素数（`nworld × 单 world 像素数`）。
    pub fn buffer_elements(&self, channel: ImageOutput) -> Result<usize, WorldError> {
        self.worlds
            .checked_mul(self.channel_pixels(channel))
            .ok_or(WorldError::CountOverflow {
                field: "world_buffer_elements",
            })
    }

    fn check_world(&self, world: usize) -> Result<(), WorldError> {
        check_world_range(world, self.worlds)?;
        if !self.active[world] {
            return Err(WorldError::WorldInactive { world });
        }
        Ok(())
    }

    /// 某 world 在该通道的起始元素下标（world-major）。
    pub fn world_offset(&self, channel: ImageOutput, world: usize) -> Result<usize, WorldError> {
        self.check_world(world)?;
        world
            .checked_mul(self.channel_pixels(channel))
            .ok_or(WorldError::CountOverflow {
                field: "world_offset",
            })
    }

    /// 某 world 在该通道的元素区段（长度 = 单 world 像素数）。
    pub fn world_range(
        &self,
        channel: ImageOutput,
        world: usize,
    ) -> Result<Range<usize>, WorldError> {
        let start = self.world_offset(channel, world)?;
        let end =
            start
                .checked_add(self.channel_pixels(channel))
                .ok_or(WorldError::CountOverflow {
                    field: "world_range",
                })?;
        Ok(start..end)
    }

    /// 从整缓冲切出某 world 的区段（校验缓冲长度；不复制、不越界读邻 world）。
    pub fn world_view<'a, T>(
        &self,
        channel: ImageOutput,
        world: usize,
        data: &'a [T],
    ) -> Result<&'a [T], WorldError> {
        let range = self.world_range(channel, world)?;
        if range.end > data.len() {
            return Err(WorldError::BufferRange {
                world,
                end: range.end,
                actual: data.len(),
            });
        }
        Ok(&data[range])
    }

    /// 某 world 内某相机在该通道的元素区段（相机区段在 world 偏移之后）。
    pub fn camera_range(
        &self,
        channel: ImageOutput,
        world: usize,
        camera: usize,
    ) -> Result<Range<usize>, WorldError> {
        let base = self.world_offset(channel, world)?;
        check_camera_range(camera, self.cameras())?;
        let address = match channel {
            ImageOutput::Rgb => self.layout.rgb_adr[camera],
            ImageOutput::Depth => self.layout.depth_adr[camera],
            ImageOutput::Segmentation => self.layout.seg_adr[camera],
        };
        if address < 0 {
            return Err(WorldError::ChannelDisabled {
                channel,
                camera_index: camera,
            });
        }
        let start = base
            .checked_add(address as usize)
            .ok_or(WorldError::CountOverflow {
                field: "camera_range",
            })?;
        let end = start
            .checked_add(pixels_of(self.plan.resolutions[camera])?)
            .ok_or(WorldError::CountOverflow {
                field: "camera_range",
            })?;
        Ok(start..end)
    }
}
