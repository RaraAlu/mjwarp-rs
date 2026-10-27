//! 逐相机输出视图（渲染/BVH R08-S1，纯宿主，无 GPU、无窗口）。
//!
//! 语义翻译冻结上游 `render_util.py`（MJWarp 3.15，`71da24d`）的三个输出视图：
//! `get_rgb`（`unpack_rgb_kernel`，L170 起）、`get_depth`（`extract_depth_kernel`，L193 起）、
//! `get_segmentation`（`_extract_seg_kernel`，L246 起）。三者在整缓冲上按
//! [`OutputLayout`] 的 `rgb_adr/depth_adr/seg_adr[camera_index]` 取该相机的区段：
//!
//! - RGB：`b = f32(val & 0xFF) * (1/255)`、`g = f32((val >> 8) & 0xFF) * (1/255)`、
//!   `r = f32((val >> 16) & 0xFF) * (1/255)`，输出 `vec3(r, g, b)`；`(1/255)` 取 S4 冻结的
//!   `float32(1.0/255.0)`（[`INV_255`]，`0x3B80_8081`）。
//! - depth：`clamp(val / depth_scale, 0.0, 1.0)`——**除法在前、clamp 在后**；`depth_scale` 先于
//!   任何查找校验。`-0.0 → +0.0`、`NaN → 0` 由产品 [`clamp_unit`] 实现，属实现行为（对齐 CUDA
//!   `fminf/fmaxf` 顺序），不是上游语义保证；夹具无 `-0`/NaN 样本 → 未覆盖（O3/O7）。
//! - seg：`(object_id, object_type)` 原样复制。
//!
//! `camera_index` 是**激活相机顺序位置**（`cam_active` 选中顺序，即 `OutputLayout` 的 adr
//! 顺序），不是模型相机 id；[`CameraOutputViews::camera_id`] 给出到模型 id 的映射。像素顺序
//! `[y][x]`、`x` 最快，与上游 `array3d[world, y, x]` 的单 world 切片一致。
//!
//! 本片只做单 world（`nworld == 1`）：`nworld > 1` 明确拒绝，多 world 视图归 R10。视图不复制
//! 整缓冲（只有 RGB 解包与 depth 缩放会分配逐相机结果）；越界、通道未启用、缓冲缺失/长度
//! 不符在读取前返回 [`CameraViewError`]，不 panic；`depth()` 的 `depth_scale` 校验优先于
//! 相机/通道/缓冲查找（错误优先级见 [`CameraOutputViews::depth`]）。

use std::ops::Range;

use crate::render::ImageOutput;
use crate::render::camera::CameraPlan;
use crate::render::texture::INV_255;

use crate::render::context::OutputLayout;

/// 逐相机视图错误；全部在读取前给出。
#[derive(Clone, Debug, PartialEq)]
pub enum CameraViewError {
    /// 本片只支持单 world（`nworld > 1` 归 R10）。
    NworldUnsupported { nworld: usize },
    /// 相机数不一致（`plan.resolutions` 与 `layout` 的 adr 表须来自同一次解析）。
    CameraCount { expected: usize, actual: usize },
    /// `plan` 的通道开关表长度与相机数不一致。
    PlanFlags {
        channel: ImageOutput,
        cameras: usize,
        actual: usize,
    },
    /// `layout` 的通道像素总数与 `plan` 的开关/分辨率不一致。
    LayoutPixels {
        channel: ImageOutput,
        expected: usize,
        actual: usize,
    },
    /// `camera_index` 超出激活相机数。
    CameraIndex { index: usize, cameras: usize },
    /// 相机像素数计数溢出。
    PixelOverflow { camera_index: usize },
    /// 该相机的输出通道未启用（`*_adr[camera_index] == -1`）。
    ChannelDisabled {
        channel: ImageOutput,
        camera_index: usize,
    },
    /// 通道已有相机启用，但未提供整缓冲。
    BufferMissing { channel: ImageOutput },
    /// 整缓冲长度与布局不符（单 world 时应等于该通道像素总数）。
    BufferLength {
        channel: ImageOutput,
        expected: usize,
        actual: usize,
    },
    /// 该相机区段末端超出整缓冲（`adr + pixels > len`）。
    BufferRange {
        channel: ImageOutput,
        camera_index: usize,
        end: usize,
        actual: usize,
    },
    /// `depth_scale` 非正或非有限。
    DepthScale { depth_scale: f32 },
}

impl std::error::Error for CameraViewError {}

impl std::fmt::Display for CameraViewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NworldUnsupported { nworld } => {
                write!(formatter, "逐相机视图只支持单 world，nworld={nworld}")
            }
            Self::CameraCount { expected, actual } => {
                write!(formatter, "相机数 {actual} != plan 的 {expected}")
            }
            Self::PlanFlags {
                channel,
                cameras,
                actual,
            } => write!(
                formatter,
                "{} 开关表长度 {actual} != 相机数 {cameras}",
                channel_name(*channel)
            ),
            Self::LayoutPixels {
                channel,
                expected,
                actual,
            } => write!(
                formatter,
                "{} 像素总数 {actual} != plan 推导的 {expected}",
                channel_name(*channel)
            ),
            Self::CameraIndex { index, cameras } => {
                write!(formatter, "camera_index {index} 超出相机数 {cameras}")
            }
            Self::PixelOverflow { camera_index } => {
                write!(formatter, "相机 {camera_index} 像素数溢出")
            }
            Self::ChannelDisabled {
                channel,
                camera_index,
            } => write!(
                formatter,
                "相机 {camera_index} 未启用 {} 输出",
                channel_name(*channel)
            ),
            Self::BufferMissing { channel } => {
                write!(formatter, "缺少 {} 整缓冲", channel_name(*channel))
            }
            Self::BufferLength {
                channel,
                expected,
                actual,
            } => write!(
                formatter,
                "{} 整缓冲长度 {actual} != {expected}",
                channel_name(*channel)
            ),
            Self::BufferRange {
                channel,
                camera_index,
                end,
                actual,
            } => write!(
                formatter,
                "相机 {camera_index} 的 {} 区段末端 {end} 超出缓冲 {actual}",
                channel_name(*channel)
            ),
            Self::DepthScale { depth_scale } => {
                write!(formatter, "depth_scale {depth_scale} 须为有限正数")
            }
        }
    }
}

fn channel_name(channel: ImageOutput) -> &'static str {
    match channel {
        ImageOutput::Rgb => "rgb",
        ImageOutput::Depth => "depth",
        ImageOutput::Segmentation => "seg",
    }
}

/// 逐相机图像：分辨率 + `[y][x]` 行主序像素（`x` 最快），单 world。
#[derive(Clone, Debug, PartialEq)]
pub struct CameraImage<T> {
    resolution: [u32; 2],
    pixels: Vec<T>,
}

impl<T> CameraImage<T> {
    fn new(resolution: [u32; 2], pixels: Vec<T>) -> Self {
        Self { resolution, pixels }
    }

    pub fn resolution(&self) -> [u32; 2] {
        self.resolution
    }

    pub fn width(&self) -> u32 {
        self.resolution[0]
    }

    pub fn height(&self) -> u32 {
        self.resolution[1]
    }

    pub fn pixels(&self) -> &[T] {
        &self.pixels
    }

    pub fn into_pixels(self) -> Vec<T> {
        self.pixels
    }

    /// 行主序下标：`y * width + x`；越界返回 `None`。
    pub fn row_major_index(&self, x: u32, y: u32) -> Option<usize> {
        let inside = x < self.resolution[0] && y < self.resolution[1];
        inside.then(|| y as usize * self.resolution[0] as usize + x as usize)
    }

    /// 按 `(x, y)` 取像素；越界返回 `None`。
    pub fn get(&self, x: u32, y: u32) -> Option<&T> {
        self.pixels.get(self.row_major_index(x, y)?)
    }
}

/// 整缓冲输入（单 world）：`None` 表示调用方没有提供该通道。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WholeOutputData<'a> {
    pub rgb: Option<&'a [u32]>,
    pub depth: Option<&'a [f32]>,
    pub segmentation: Option<&'a [[i32; 2]]>,
}

/// `plan.resolutions` + `OutputLayout` 上的逐相机输出视图。
#[derive(Clone, Copy, Debug)]
pub struct CameraOutputViews<'a> {
    plan: &'a CameraPlan,
    layout: &'a OutputLayout,
    data: WholeOutputData<'a>,
}

impl<'a> CameraOutputViews<'a> {
    /// 校验 `plan`/`layout`/整缓冲自洽后建立视图。
    ///
    /// `plan` 与 `layout` 必须来自同一次解析（`OutputLayout::resolve`）；`data` 的各通道
    /// 长度必须等于布局的该通道像素总数（单 world）。
    pub fn new(
        plan: &'a CameraPlan,
        layout: &'a OutputLayout,
        nworld: usize,
        data: WholeOutputData<'a>,
    ) -> Result<Self, CameraViewError> {
        if nworld != 1 {
            return Err(CameraViewError::NworldUnsupported { nworld });
        }
        let cameras = plan.resolutions.len();
        for actual in [
            layout.rgb_adr.len(),
            layout.depth_adr.len(),
            layout.seg_adr.len(),
        ] {
            if actual != cameras {
                return Err(CameraViewError::CameraCount {
                    expected: cameras,
                    actual,
                });
            }
        }
        for (channel, flags) in [
            (ImageOutput::Rgb, &plan.render_rgb),
            (ImageOutput::Depth, &plan.render_depth),
            (ImageOutput::Segmentation, &plan.render_seg),
        ] {
            if flags.len() != cameras {
                return Err(CameraViewError::PlanFlags {
                    channel,
                    cameras,
                    actual: flags.len(),
                });
            }
        }
        let views = Self { plan, layout, data };
        for channel in [
            ImageOutput::Rgb,
            ImageOutput::Depth,
            ImageOutput::Segmentation,
        ] {
            let expected = plan_pixels(plan, channel)?;
            let actual = views.layout_pixels(channel);
            if actual != expected {
                return Err(CameraViewError::LayoutPixels {
                    channel,
                    expected,
                    actual,
                });
            }
        }
        for channel in [
            ImageOutput::Rgb,
            ImageOutput::Depth,
            ImageOutput::Segmentation,
        ] {
            if views.has_data(channel) {
                views.buffer_len(channel)?;
            }
        }
        Ok(views)
    }

    /// 激活相机数（`camera_index` 的上界）。
    pub fn camera_count(&self) -> usize {
        self.plan.resolutions.len()
    }

    /// `camera_index` → 模型相机 id（`cam_active` 选中顺序，不是相机 id 顺序）。
    pub fn camera_id(&self, camera_index: usize) -> Result<i32, CameraViewError> {
        self.check_index(camera_index)?;
        Ok(self.plan.cameras[camera_index] as i32)
    }

    /// 该相机的分辨率（逐相机，允许各相机不同宽高）。
    pub fn resolution(&self, camera_index: usize) -> Result<[u32; 2], CameraViewError> {
        self.check_index(camera_index)?;
        Ok(self.plan.resolutions[camera_index])
    }

    /// 该相机的 packed RGB 区段（`rgb_data[rgb_adr .. rgb_adr + pixels]`，逐位原样）。
    pub fn rgb_packed(&self, camera_index: usize) -> Result<&'a [u32], CameraViewError> {
        let (range, _resolution) = self.region(ImageOutput::Rgb, camera_index)?;
        match self.data.rgb {
            Some(values) => Ok(&values[range]),
            None => Err(CameraViewError::BufferMissing {
                channel: ImageOutput::Rgb,
            }),
        }
    }

    /// 解包后的 RGB：`vec3(r, g, b)`，逐位复刻上游 `unpack_rgb_kernel`。
    pub fn rgb(&self, camera_index: usize) -> Result<CameraImage<[f32; 3]>, CameraViewError> {
        let packed = self.rgb_packed(camera_index)?;
        let resolution = self.resolution(camera_index)?;
        let pixels = packed.iter().map(|value| unpack_rgb(*value)).collect();
        Ok(CameraImage::new(resolution, pixels))
    }

    /// 该相机的 depth 区段（原始 f32 深度值，未缩放）。
    pub fn depth_raw(&self, camera_index: usize) -> Result<&'a [f32], CameraViewError> {
        let (range, _resolution) = self.region(ImageOutput::Depth, camera_index)?;
        match self.data.depth {
            Some(values) => Ok(&values[range]),
            None => Err(CameraViewError::BufferMissing {
                channel: ImageOutput::Depth,
            }),
        }
    }

    /// 缩放并截断后的深度：`clamp(val / depth_scale, 0.0, 1.0)`（除法在前）。错误优先级（O2）：
    /// `depth_scale` 先于相机/通道/缓冲查找校验，故「通道未启用 + 非法 scale」返回 `DepthScale`。
    pub fn depth(
        &self,
        camera_index: usize,
        depth_scale: f32,
    ) -> Result<CameraImage<f32>, CameraViewError> {
        if !depth_scale.is_finite() || depth_scale <= 0.0 {
            return Err(CameraViewError::DepthScale { depth_scale });
        }
        let raw = self.depth_raw(camera_index)?;
        let resolution = self.resolution(camera_index)?;
        let pixels = raw
            .iter()
            .map(|value| clamp_unit(*value / depth_scale))
            .collect();
        Ok(CameraImage::new(resolution, pixels))
    }

    /// 该相机的分割区段 `(object_id, object_type)` 对，逐对原样。
    pub fn segmentation_raw(&self, camera_index: usize) -> Result<&'a [[i32; 2]], CameraViewError> {
        let (range, _resolution) = self.region(ImageOutput::Segmentation, camera_index)?;
        match self.data.segmentation {
            Some(values) => Ok(&values[range]),
            None => Err(CameraViewError::BufferMissing {
                channel: ImageOutput::Segmentation,
            }),
        }
    }

    /// 分割图（与 [`Self::segmentation_raw`] 同值，附带分辨率信息）。
    pub fn segmentation(
        &self,
        camera_index: usize,
    ) -> Result<CameraImage<[i32; 2]>, CameraViewError> {
        let resolution = self.resolution(camera_index)?;
        let pixels = self.segmentation_raw(camera_index)?.to_vec();
        Ok(CameraImage::new(resolution, pixels))
    }

    fn check_index(&self, camera_index: usize) -> Result<(), CameraViewError> {
        let cameras = self.camera_count();
        (camera_index < cameras)
            .then_some(())
            .ok_or(CameraViewError::CameraIndex {
                index: camera_index,
                cameras,
            })
    }

    fn has_data(&self, channel: ImageOutput) -> bool {
        match channel {
            ImageOutput::Rgb => self.data.rgb.is_some(),
            ImageOutput::Depth => self.data.depth.is_some(),
            ImageOutput::Segmentation => self.data.segmentation.is_some(),
        }
    }

    fn layout_pixels(&self, channel: ImageOutput) -> usize {
        match channel {
            ImageOutput::Rgb => self.layout.rgb_pixels,
            ImageOutput::Depth => self.layout.depth_pixels,
            ImageOutput::Segmentation => self.layout.seg_pixels,
        }
    }

    fn buffer_len(&self, channel: ImageOutput) -> Result<usize, CameraViewError> {
        let expected = self.layout_pixels(channel);
        let actual = match channel {
            ImageOutput::Rgb => self.data.rgb.map(<[u32]>::len),
            ImageOutput::Depth => self.data.depth.map(<[f32]>::len),
            ImageOutput::Segmentation => self.data.segmentation.map(<[[i32; 2]]>::len),
        };
        match actual {
            Some(actual) if actual == expected => Ok(actual),
            Some(actual) => Err(CameraViewError::BufferLength {
                channel,
                expected,
                actual,
            }),
            None if expected == 0 => Ok(0),
            None => Err(CameraViewError::BufferMissing { channel }),
        }
    }

    /// 该通道上该相机的区段 `[adr, adr + pixels)` 与分辨率。
    fn region(
        &self,
        channel: ImageOutput,
        camera_index: usize,
    ) -> Result<(Range<usize>, [u32; 2]), CameraViewError> {
        self.check_index(camera_index)?;
        let resolution = self.plan.resolutions[camera_index];
        let pixels = (resolution[0] as usize)
            .checked_mul(resolution[1] as usize)
            .ok_or(CameraViewError::PixelOverflow { camera_index })?;
        let adr = match channel {
            ImageOutput::Rgb => self.layout.rgb_adr[camera_index],
            ImageOutput::Depth => self.layout.depth_adr[camera_index],
            ImageOutput::Segmentation => self.layout.seg_adr[camera_index],
        };
        if adr < 0 {
            return Err(CameraViewError::ChannelDisabled {
                channel,
                camera_index,
            });
        }
        let start = adr as usize;
        let end = start
            .checked_add(pixels)
            .ok_or(CameraViewError::PixelOverflow { camera_index })?;
        let actual = self.buffer_len(channel)?;
        if end > actual {
            return Err(CameraViewError::BufferRange {
                channel,
                camera_index,
                end,
                actual,
            });
        }
        Ok((start..end, resolution))
    }
}

/// `plan` 的开关/分辨率推导出的通道像素总数（与 `OutputLayout` 对账用）。
fn plan_pixels(plan: &CameraPlan, channel: ImageOutput) -> Result<usize, CameraViewError> {
    let flags = match channel {
        ImageOutput::Rgb => &plan.render_rgb,
        ImageOutput::Depth => &plan.render_depth,
        ImageOutput::Segmentation => &plan.render_seg,
    };
    let mut total = 0usize;
    for (camera_index, (resolution, enabled)) in plan.resolutions.iter().zip(flags).enumerate() {
        if !*enabled {
            continue;
        }
        let pixels = (resolution[0] as usize)
            .checked_mul(resolution[1] as usize)
            .ok_or(CameraViewError::PixelOverflow { camera_index })?;
        total = total
            .checked_add(pixels)
            .ok_or(CameraViewError::PixelOverflow { camera_index })?;
    }
    Ok(total)
}

/// 上游 `unpack_rgb_kernel`：`b = f32(val & 0xFF) * (1/255)`、`g = (val >> 8)`、`r = (val >> 16)`。
fn unpack_rgb(value: u32) -> [f32; 3] {
    let b = (value & 0xFF) as f32 * INV_255;
    let g = ((value >> 8) & 0xFF) as f32 * INV_255;
    let r = ((value >> 16) & 0xFF) as f32 * INV_255;
    [r, g, b]
}

/// 产品侧 clamp：区间压缩到 `[0.0, 1.0]`（上游要求的语义部分）；`NaN → 0.0`、`-0.0 → +0.0` 是
/// 实现行为（对齐 CUDA `fminf/fmaxf` 顺序），不是上游语义保证；夹具无此类样本 → 未覆盖（O3/O7）。
fn clamp_unit(value: f32) -> f32 {
    if value.is_nan() || value <= 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}
