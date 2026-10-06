//! 单相机批量图像的宿主布局。
//! 不生成像素或创建BVH。

use std::ops::Range;

use crate::diagnostics::InputError;
use crate::model::BatchLayout;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutputChannels {
    pub rgb: bool,
    pub depth: bool,
    pub segmentation: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageOutput {
    Rgb,
    Depth,
    Segmentation,
}

/// 连续布局 `[world, row, column]`。
/// RGB为u32，深度为f32。
/// 分割包含i32的ID与类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageBatchLayout {
    pixels: BatchLayout,
    width: usize,
    height: usize,
    channels: OutputChannels,
}

impl ImageBatchLayout {
    pub fn new(
        worlds: usize,
        width: usize,
        height: usize,
        channels: OutputChannels,
    ) -> Result<Self, InputError> {
        if width == 0 {
            return Err(InputError::InvalidDimension { field: "width" });
        }
        if height == 0 {
            return Err(InputError::InvalidDimension { field: "height" });
        }
        let pixels_per_world = width.checked_mul(height).ok_or(InputError::Overflow {
            field: "image_layout",
        })?;
        let pixels = BatchLayout::new(worlds, pixels_per_world, 1)?;
        if channels.rgb || channels.depth {
            BatchLayout::new(worlds, pixels_per_world, 4)?;
        }
        if channels.segmentation {
            BatchLayout::new(worlds, pixels_per_world, 8)?;
        }
        Ok(Self {
            pixels,
            width,
            height,
            channels,
        })
    }

    pub fn worlds(self) -> usize {
        self.pixels.worlds()
    }

    pub fn width(self) -> usize {
        self.width
    }

    pub fn height(self) -> usize {
        self.height
    }

    pub fn channels(self) -> OutputChannels {
        self.channels
    }

    pub fn pixel_index(self, world: usize, row: usize, column: usize) -> Result<usize, InputError> {
        let world_range = self.pixels.world_elements(world)?;
        for (field, index, limit) in [("row", row, self.height), ("column", column, self.width)] {
            if index >= limit {
                return Err(InputError::InvalidIndex {
                    field,
                    index,
                    limit,
                });
            }
        }
        Ok(world_range.start + row * self.width + column)
    }

    pub fn buffer_bytes(self, output: ImageOutput) -> Result<usize, InputError> {
        let scalars_per_pixel = self.scalars_per_pixel(output)?;
        Ok(self.pixels.total_elements() * scalars_per_pixel * 4)
    }

    /// 返回u32/f32/i32元素范围。
    /// 分割范围包含两个元素。
    pub fn pixel_range(
        self,
        output: ImageOutput,
        world: usize,
        row: usize,
        column: usize,
    ) -> Result<Range<usize>, InputError> {
        let scalars = self.scalars_per_pixel(output)?;
        let start = self.pixel_index(world, row, column)? * scalars;
        Ok(start..start + scalars)
    }

    fn scalars_per_pixel(self, output: ImageOutput) -> Result<usize, InputError> {
        let (enabled, scalars) = match output {
            ImageOutput::Rgb => (self.channels.rgb, 1),
            ImageOutput::Depth => (self.channels.depth, 1),
            ImageOutput::Segmentation => (self.channels.segmentation, 2),
        };
        if !enabled {
            return Err(InputError::DisabledOutput);
        }
        Ok(scalars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: OutputChannels = OutputChannels {
        rgb: true,
        depth: true,
        segmentation: true,
    };

    #[test]
    fn locates_rectangular_batch_pixels_and_segmentation_pairs() {
        let layout = ImageBatchLayout::new(2, 3, 2, ALL).unwrap();
        assert_eq!(
            (layout.worlds(), layout.width(), layout.height()),
            (2, 3, 2)
        );
        assert_eq!(layout.channels(), ALL);
        assert_eq!(layout.pixel_index(1, 1, 2).unwrap(), 11);
        assert_eq!(
            layout.pixel_range(ImageOutput::Rgb, 1, 1, 2).unwrap(),
            11..12
        );
        assert_eq!(
            layout
                .pixel_range(ImageOutput::Segmentation, 1, 1, 2)
                .unwrap(),
            22..24
        );
        assert_eq!(layout.buffer_bytes(ImageOutput::Rgb).unwrap(), 48);
        assert_eq!(layout.buffer_bytes(ImageOutput::Depth).unwrap(), 48);
        assert_eq!(layout.buffer_bytes(ImageOutput::Segmentation).unwrap(), 96);
    }

    #[test]
    fn rejects_disabled_outputs_before_access() {
        let layout = ImageBatchLayout::new(
            1,
            2,
            3,
            OutputChannels {
                depth: true,
                ..OutputChannels::default()
            },
        )
        .unwrap();
        assert_eq!(layout.buffer_bytes(ImageOutput::Depth).unwrap(), 24);
        for output in [ImageOutput::Rgb, ImageOutput::Segmentation] {
            assert_eq!(layout.buffer_bytes(output), Err(InputError::DisabledOutput));
            assert_eq!(
                layout.pixel_range(output, usize::MAX, usize::MAX, usize::MAX),
                Err(InputError::DisabledOutput)
            );
        }
    }

    #[test]
    fn permits_all_disabled_channels_without_buffers() {
        let layout = ImageBatchLayout::new(1, 2, 3, OutputChannels::default()).unwrap();
        assert_eq!(layout.pixel_index(0, 2, 1).unwrap(), 5);
        for output in [
            ImageOutput::Rgb,
            ImageOutput::Depth,
            ImageOutput::Segmentation,
        ] {
            assert_eq!(layout.buffer_bytes(output), Err(InputError::DisabledOutput));
        }
    }

    #[test]
    fn rejects_world_row_and_column_indices() {
        let layout = ImageBatchLayout::new(2, 3, 2, ALL).unwrap();
        for (world, row, column, field, index, limit) in [
            (2, 0, 0, "world", 2, 2),
            (0, 2, 0, "row", 2, 2),
            (0, 0, 3, "column", 3, 3),
        ] {
            assert_eq!(
                layout.pixel_index(world, row, column),
                Err(InputError::InvalidIndex {
                    field,
                    index,
                    limit
                })
            );
        }
    }

    #[test]
    fn rejects_dimensions_and_active_output_capacity_overflow() {
        for (worlds, width, height) in [
            (0, 1, 1),
            (1, 0, 1),
            (1, 1, 0),
            (1, usize::MAX, 2),
            (2, usize::MAX, 1),
            (1, isize::MAX as usize / 4, 1),
        ] {
            assert!(ImageBatchLayout::new(worlds, width, height, ALL).is_err());
        }
        let width = isize::MAX as usize / 4;
        let rgb_only = OutputChannels {
            rgb: true,
            ..OutputChannels::default()
        };
        let layout = ImageBatchLayout::new(1, width, 1, rgb_only).unwrap();
        assert_eq!(layout.buffer_bytes(ImageOutput::Rgb).unwrap(), width * 4);
        assert_eq!(
            layout.buffer_bytes(ImageOutput::Segmentation),
            Err(InputError::DisabledOutput)
        );
    }
}
