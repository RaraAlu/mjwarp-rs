//! render context 的输出偏移与缓冲大小（渲染/BVH S6a，纯宿主）。
//!
//! 语义翻译冻结上游 `render_util.create_render_context`（MJWarp 3.15，`71da24d`）的
//! 输出布局段：逐相机累加 `rgb_adr/depth_adr/seg_adr`（未启用为 -1），`total_rays`
//! 为全部激活相机像素数之和，`nsamples = samples_per_pixel²`，射线数组长度为
//! `total_rays * nsamples`；`seg_data` 宽度为 `max(seg_pixels, 1)`，`aa_accum` 宽度
//! 在 `nsamples > 1` 时是 `rgb_pixels`、否则为 1。全部计数用 `checked_*` 防溢出。

use crate::render::camera::CameraPlan;

use super::ContextError;

/// 输出缓冲布局（设备分配在 S6b）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputLayout {
    /// 全部激活相机像素数之和。
    pub total_rays: usize,
    pub samples_per_pixel: u32,
    /// `samples_per_pixel²`。
    pub nsamples: usize,
    /// 射线数组长度（`total_rays * nsamples`）。
    pub ray_count: usize,
    /// 每相机 RGB 起始下标；未启用为 -1。
    pub rgb_adr: Vec<i32>,
    pub depth_adr: Vec<i32>,
    pub seg_adr: Vec<i32>,
    /// 启用 RGB 的相机像素总数（`rgb_data` 宽度）。
    pub rgb_pixels: usize,
    pub depth_pixels: usize,
    pub seg_pixels: usize,
    /// `seg_data` 宽度：上游 `max(seg_pixels, 1)`。
    pub seg_slots: usize,
    /// `aa_accum` 宽度：`nsamples > 1 ? rgb_pixels : 1`。
    pub aa_accum_width: usize,
}

impl OutputLayout {
    /// 按 `CameraPlan` 的相机顺序与输出开关计算偏移/大小。
    pub fn resolve(plan: &CameraPlan) -> Result<Self, ContextError> {
        let cameras = plan.resolutions.len();
        let mut rgb_adr = vec![-1i32; cameras];
        let mut depth_adr = vec![-1i32; cameras];
        let mut seg_adr = vec![-1i32; cameras];
        let mut rgb_pixels = 0usize;
        let mut depth_pixels = 0usize;
        let mut seg_pixels = 0usize;
        let mut total_rays = 0usize;
        for (position, resolution) in plan.resolutions.iter().enumerate() {
            let pixels = (resolution[0] as usize)
                .checked_mul(resolution[1] as usize)
                .ok_or(ContextError::CountOverflow {
                    field: "camera_pixels",
                })?;
            total_rays = total_rays
                .checked_add(pixels)
                .ok_or(ContextError::CountOverflow {
                    field: "total_rays",
                })?;
            if plan.render_rgb[position] {
                rgb_adr[position] = i32::try_from(rgb_pixels)
                    .map_err(|_| ContextError::CountOverflow { field: "rgb_adr" })?;
                rgb_pixels = rgb_pixels
                    .checked_add(pixels)
                    .ok_or(ContextError::CountOverflow {
                        field: "rgb_pixels",
                    })?;
            }
            if plan.render_depth[position] {
                depth_adr[position] = i32::try_from(depth_pixels)
                    .map_err(|_| ContextError::CountOverflow { field: "depth_adr" })?;
                depth_pixels =
                    depth_pixels
                        .checked_add(pixels)
                        .ok_or(ContextError::CountOverflow {
                            field: "depth_pixels",
                        })?;
            }
            if plan.render_seg[position] {
                seg_adr[position] = i32::try_from(seg_pixels)
                    .map_err(|_| ContextError::CountOverflow { field: "seg_adr" })?;
                seg_pixels = seg_pixels
                    .checked_add(pixels)
                    .ok_or(ContextError::CountOverflow {
                        field: "seg_pixels",
                    })?;
            }
        }
        let nsamples = plan.samples;
        let ray_count = total_rays
            .checked_mul(nsamples)
            .ok_or(ContextError::CountOverflow { field: "ray_count" })?;
        Ok(Self {
            total_rays,
            samples_per_pixel: plan.samples_per_pixel,
            nsamples,
            ray_count,
            rgb_adr,
            depth_adr,
            seg_adr,
            rgb_pixels,
            depth_pixels,
            seg_pixels,
            seg_slots: seg_pixels.max(1),
            aa_accum_width: if nsamples > 1 { rgb_pixels } else { 1 },
        })
    }

    fn bytes(
        elements: usize,
        element_bytes: usize,
        nworld: usize,
        field: &'static str,
    ) -> Result<usize, ContextError> {
        elements
            .checked_mul(nworld)
            .and_then(|total| total.checked_mul(element_bytes))
            .ok_or(ContextError::CountOverflow { field })
    }

    /// `rgb_data` 字节数：`nworld * rgb_pixels * 4`（u32 packed RGBA）。
    pub fn rgb_buffer_bytes(&self, nworld: usize) -> Result<usize, ContextError> {
        Self::bytes(self.rgb_pixels, 4, nworld, "rgb_bytes")
    }

    /// `depth_data` 字节数：`nworld * depth_pixels * 4`（f32）。
    pub fn depth_buffer_bytes(&self, nworld: usize) -> Result<usize, ContextError> {
        Self::bytes(self.depth_pixels, 4, nworld, "depth_bytes")
    }

    /// `seg_data` 字节数：`nworld * seg_slots * 8`（vec2i）。
    pub fn seg_buffer_bytes(&self, nworld: usize) -> Result<usize, ContextError> {
        Self::bytes(self.seg_slots, 8, nworld, "seg_bytes")
    }

    /// `ray` 字节数：`ray_count * 12`（vec3 f32，不含 world 维）。
    pub fn ray_buffer_bytes(&self) -> Result<usize, ContextError> {
        self.ray_count
            .checked_mul(12)
            .ok_or(ContextError::CountOverflow { field: "ray_bytes" })
    }

    /// `ray_offset` 字节数（与 `ray` 相同）。
    pub fn ray_offset_buffer_bytes(&self) -> Result<usize, ContextError> {
        self.ray_buffer_bytes()
    }

    /// `aa_accum` 字节数：`nworld * aa_accum_width * 12`（vec3 f32）。
    pub fn aa_accum_bytes(&self, nworld: usize) -> Result<usize, ContextError> {
        Self::bytes(self.aa_accum_width, 12, nworld, "aa_accum_bytes")
    }
}
