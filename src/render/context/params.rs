//! `create_render_context` 的 28 个公开参数（渲染/BVH S6a，纯宿主）。
//!
//! 字段与冻结上游 MJWarp 3.15 `render_util.create_render_context`（`71da24d`）的
//! Python 签名一一对应，默认值也保持上游签名值。`None` 语义在上游表示「按模型推导」：
//! `cam_res=None` → 模型 `cam_resolution`；`render_rgb/depth/seg=None` → 由模型
//! `cam_output` 位推导；`cam_active=None` → 全部相机。列表/元组/布尔三类输入形态
//! 用枚举表达，转换规则在 `context::resolve` 里按上游实现。
//!
//! 本模块只描述输入，不解析、不分配设备资源。

use crate::render::camera::CameraActive;

#[path = "lighting.rs"]
pub mod lighting;

/// R05-S1 地形 hfield 静态参数（文件位于 `context/`，按任务范围由本文件以 `#[path]` 声明）。
#[path = "hfield.rs"]
pub mod hfield;

/// R05 前置：从既有体积通道构建 hfield 输入（文件位于 `context/`，按任务范围以 `#[path]` 声明）。
#[path = "hfield_inputs.rs"]
pub mod hfield_inputs;

/// 上游默认 `enabled_geom_groups=[0, 1, 2]`。
pub const DEFAULT_GEOM_GROUPS: [i32; 3] = [0, 1, 2];

/// `cam_res` 输入：`(width, height)` 元组（全部相机共用）或逐相机列表。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CamRes<'a> {
    /// 上游 `tuple[int, int]`：全部相机同一分辨率（空相机列表时为空）。
    Tuple([u32; 2]),
    /// 上游 `list[tuple[int, int]]`：长度 1 且多相机时复制，否则须等于相机数。
    List(&'a [[u32; 2]]),
}

/// `render_rgb/render_depth/render_seg` 输入：单个布尔（全部相机）或逐相机列表。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderFlagSpec<'a> {
    /// 上游 `bool`：全部相机同一开关。
    Bool(bool),
    /// 上游 `list[bool]`：长度 1 且多相机时复制；长度等于模型相机数时按
    /// `cam_active` 重排；否则须等于激活相机数。
    List(&'a [bool]),
}

/// splat 输入（六个参数须同时提供；本片只做宿主校验与计数）。
#[derive(Clone, Copy, Debug, Default)]
pub struct SplatInputs<'a> {
    /// 世界坐标中心，`(nsplat, 3)`。
    pub position: Option<&'a [[f32; 3]]>,
    /// 旋转四元数 `(w, x, y, z)`，`(nsplat, 4)`。
    pub rotation: Option<&'a [[f32; 4]]>,
    /// 每维标准差，`(nsplat, 3)`。
    pub scale: Option<&'a [[f32; 3]]>,
    /// 颜色与不透明度，`(nsplat, 4)`。
    pub rgba: Option<&'a [[f32; 4]]>,
    /// 每个 splat 在属性数组中的偏移；`None` 表示全部 splat 同组。
    pub adr: Option<&'a [i32]>,
    /// 每个 world 的 splat 组 id，长度 `nworld`；`None` 表示全部用第一组。
    pub group_id: Option<&'a [i32]>,
}

impl SplatInputs<'_> {
    /// 是否提供了任一 splat 参数（用于「须同时提供」校验）。
    pub fn any_present(&self) -> bool {
        self.position.is_some()
            || self.rotation.is_some()
            || self.scale.is_some()
            || self.rgba.is_some()
            || self.adr.is_some()
            || self.group_id.is_some()
    }
}

/// `create_render_context` 的 28 个公开参数（上游签名顺序与默认值）。
pub struct ContextParams<'a> {
    pub nworld: usize,
    pub cam_res: Option<CamRes<'a>>,
    pub render_rgb: Option<RenderFlagSpec<'a>>,
    pub render_depth: Option<RenderFlagSpec<'a>>,
    pub render_seg: Option<RenderFlagSpec<'a>>,
    pub use_textures: bool,
    pub use_fast_math: bool,
    pub use_shadows: bool,
    pub use_ambient_lighting: bool,
    pub enabled_geom_groups: &'a [i32],
    pub cam_active: CameraActive<'a>,
    pub background_color: [f64; 4],
    pub flex_render_smooth: bool,
    pub use_precomputed_rays: bool,
    pub render_skybox: bool,
    pub enable_backface_culling: bool,
    /// 阴影遮挡时的可见度系数（上游 `shadow_light_fraction`，签名默认 0.3）：
    /// 仅当 `use_shadows` 且该灯 `light_castshadow` 时生效；上游不做范围校验，
    /// 本片原值透传、不新增拒绝（见 [`lighting::resolve_lighting`]）。
    pub shadow_light_fraction: f64,
    pub samples_per_pixel: u32,
    pub enable_vertex_normals: bool,
    pub enable_specular: bool,
    pub enable_emission: bool,
    pub enable_per_light_ambient: bool,
    pub splat: SplatInputs<'a>,
}

impl Default for ContextParams<'_> {
    /// 上游 `create_render_context` 的签名默认值。
    fn default() -> Self {
        Self {
            nworld: 1,
            cam_res: None,
            render_rgb: None,
            render_depth: None,
            render_seg: None,
            use_textures: true,
            use_fast_math: true,
            use_shadows: false,
            use_ambient_lighting: true,
            enabled_geom_groups: &DEFAULT_GEOM_GROUPS,
            cam_active: CameraActive::All,
            background_color: [0.0, 0.0, 0.0, 1.0],
            flex_render_smooth: true,
            use_precomputed_rays: true,
            render_skybox: false,
            enable_backface_culling: true,
            shadow_light_fraction: 0.3,
            samples_per_pixel: 1,
            enable_vertex_normals: true,
            enable_specular: true,
            enable_emission: true,
            enable_per_light_ambient: true,
            splat: SplatInputs::default(),
        }
    }
}

impl<'a> ContextParams<'a> {
    /// 默认参数上设置相机选择（测试与调用方构造用）。
    pub fn with_active(mut self, active: CameraActive<'a>) -> Self {
        self.cam_active = active;
        self
    }

    /// 默认参数上设置 `cam_res`。
    pub fn with_cam_res(mut self, cam_res: Option<CamRes<'a>>) -> Self {
        self.cam_res = cam_res;
        self
    }

    /// 默认参数上设置渲染输出开关。
    pub fn with_render_targets(
        mut self,
        rgb: Option<RenderFlagSpec<'a>>,
        depth: Option<RenderFlagSpec<'a>>,
        seg: Option<RenderFlagSpec<'a>>,
    ) -> Self {
        self.render_rgb = rgb;
        self.render_depth = depth;
        self.render_seg = seg;
        self
    }

    /// 相机数（按 `cam_active` 的形态；用于元组/单元素输入的复制规则）。
    pub fn requested_cameras(&self, model_ncam: usize) -> usize {
        match self.cam_active {
            CameraActive::All => model_ncam,
            CameraActive::Empty => 0,
            CameraActive::Mask(mask) => mask.iter().filter(|enabled| **enabled).count(),
            CameraActive::Names(names) => names.len(),
            CameraActive::Indices(indices) => indices.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_upstream_create_render_context_signature() {
        let params = ContextParams::default();
        assert_eq!(params.nworld, 1);
        assert!(params.cam_res.is_none());
        assert!(params.render_rgb.is_none() && params.render_depth.is_none());
        assert!(params.render_seg.is_none() && matches!(params.cam_active, CameraActive::All));
        assert!(params.use_textures && params.use_fast_math && !params.use_shadows);
        assert!(params.use_ambient_lighting && params.use_precomputed_rays);
        assert_eq!(params.enabled_geom_groups, &[0, 1, 2]);
        assert_eq!(params.background_color, [0.0, 0.0, 0.0, 1.0]);
        assert!(params.flex_render_smooth && !params.render_skybox);
        assert!(params.enable_backface_culling);
        assert_eq!(params.shadow_light_fraction, 0.3);
        assert_eq!(params.samples_per_pixel, 1);
        assert!(params.enable_vertex_normals && params.enable_specular);
        assert!(params.enable_emission && params.enable_per_light_ambient);
        assert!(!params.splat.any_present());
    }
}
