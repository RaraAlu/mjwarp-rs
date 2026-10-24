//! render context 宿主侧参数解析（渲染/BVH S6a）。
//!
//! 语义翻译冻结上游 `render_util.create_render_context`（MJWarp 3.15，`71da24d`）的
//! 宿主段：28 个公开参数的默认值/`None` 推导、相机筛选与分辨率、输出开关、输出偏移与
//! 缓冲大小、`znear/zfar`、samples/AA 组合限制、splat/几何组/灯光/天空盒派生，以及
//! 写前拒绝面。相机选择与分辨率/输出开关复用 S5 已验收的 [`crate::render::camera`]。
//!
//! 本片只做宿主解析（无 GPU、无资源所有权）：设备缓冲与生命周期在 S6b，渲染内核在 S7b。
//! 模型输入取 S1 已验收的 [`RenderFields`]（28 参数之外的模型字段全部来自它）。

use crate::model::render::RenderFields;
use crate::render::CameraError;
use crate::render::camera::{
    CameraActive, CameraModel, CameraPlan, CameraRequest, Projection, RenderTargets,
    resolve_cameras,
};

mod model;
#[cfg(test)]
mod tests;

#[cfg(feature = "cuda-probe")]
pub mod lifecycle;
pub mod output;
#[cfg(feature = "cuda-probe")]
pub mod owner;
pub mod params;

#[cfg(feature = "cuda-probe")]
pub use lifecycle::{LifecycleError, OutputResources, OutputValues, RenderContext, release_stats};
pub use output::OutputLayout;
#[cfg(feature = "cuda-probe")]
pub use owner::{
    LeafPose, MeshResources, MeshState, OwnerError, SceneLeaf, SceneResources, SceneState,
    TextureResources, device_memory,
};
pub use params::{CamRes, ContextParams, RenderFlagSpec, SplatInputs};

use model::{headlight, lights, near_far, skybox, squash_geometry};

/// 上游 `mjtLightType.mjLIGHT_SPOT`。
const LIGHT_SPOT: i32 = 0;

/// render context 参数解析错误；全部在分配/发射前给出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextError {
    /// 相机选择/分辨率/输出开关/samples 组合错误（S5 错误字段）。
    Camera(CameraError),
    MissingField {
        field: &'static str,
    },
    FieldKind {
        field: &'static str,
    },
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 提供了部分 splat 参数（上游要求六个参数同时提供）。
    SplatAttributeMismatch,
    /// splat 形状不一致（四个数组长度须相等）。
    SplatShape {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 提供了 `splat_position` 但缺少其它 splat 数组。
    SplatMissing {
        field: &'static str,
    },
    /// splat 组 id 长度须等于 `nworld`。
    SplatGroupCount {
        expected: usize,
        actual: usize,
    },
    /// 背景色分量乘以 255 后超出 [0, 255] 或非有限。
    BackgroundColor {
        channel: usize,
    },
    CountOverflow {
        field: &'static str,
    },
}

impl From<CameraError> for ContextError {
    fn from(error: CameraError) -> Self {
        Self::Camera(error)
    }
}

impl std::error::Error for ContextError {}

impl std::fmt::Display for ContextError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Camera(error) => write!(formatter, "{error}"),
            Self::MissingField { field } => write!(formatter, "缺少渲染字段 {field}"),
            Self::FieldKind { field } => write!(formatter, "渲染字段 {field} 类型不符"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(formatter, "{field} 长度 {actual} != {expected}"),
            Self::SplatAttributeMismatch => {
                write!(formatter, "splat 属性、偏移与组 id 必须同时提供")
            }
            Self::SplatShape {
                field,
                expected,
                actual,
            } => write!(formatter, "splat {field} 长度 {actual} != {expected}"),
            Self::SplatMissing { field } => write!(formatter, "缺少 splat 数组 {field}"),
            Self::SplatGroupCount { expected, actual } => {
                write!(
                    formatter,
                    "splat_group_id 长度 {actual} != nworld {expected}"
                )
            }
            Self::BackgroundColor { channel } => {
                write!(formatter, "background_color 通道 {channel} 乘 255 后越界")
            }
            Self::CountOverflow { field } => write!(formatter, "{field} 计数溢出"),
        }
    }
}

/// 相机投影路径（与上游 `_build_rays` 的分支选择一致）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraBranch {
    /// `sensorsize[1] != 0`：用 `cam_intrinsic` 内参分支。
    Intrinsic,
    /// 否则用 `cam_fovy` 分支。
    Fovy,
}

/// 单相机解析值（对应上游 S0b `context.json.camera_paths`）。
#[derive(Clone, Debug, PartialEq)]
pub struct CameraPath {
    pub camera_id: i32,
    pub projection: i32,
    pub resolution: [u32; 2],
    pub sensorsize: [f32; 2],
    pub intrinsic: [f32; 4],
    pub fovy_degrees: f64,
    pub branch: CameraBranch,
}

/// 模型头灯（`vis.headlight`）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Headlight {
    pub active: bool,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

/// 天空盒纹理引用（仅在 `render_skybox=true` 且模型存在天空盒纹理时非空）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skybox {
    pub texture_id: i32,
    pub face_width: u32,
}

/// 解析后的 render context（宿主侧；设备资源在 S6b）。
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedContext {
    pub nworld: usize,
    pub plan: CameraPlan,
    pub output: OutputLayout,
    pub camera_paths: Vec<CameraPath>,
    pub use_textures: bool,
    pub use_fast_math: bool,
    pub use_shadows: bool,
    pub use_ambient_lighting: bool,
    pub use_precomputed_rays: bool,
    pub render_skybox: bool,
    pub skybox: Option<Skybox>,
    pub enable_backface_culling: bool,
    pub shadow_light_fraction: f64,
    pub samples_per_pixel: u32,
    pub enable_vertex_normals: bool,
    pub enable_specular: bool,
    pub enable_emission: bool,
    pub enable_per_light_ambient: bool,
    pub flex_render_smooth: bool,
    pub background_color: [f64; 4],
    pub background_color_packed: u32,
    pub background_color_unpacked: [f64; 4],
    pub enabled_geom_ids: Vec<i32>,
    pub bvh_ngeom: usize,
    pub geom_ray_types: Vec<i32>,
    /// `float(vis.map.znear * stat.extent)`：上游按 f64 计算（f32 输入先升 f64）。
    pub znear: f64,
    /// `float(vis.map.zfar * stat.extent)`。
    pub zfar: f64,
    pub headlight: Headlight,
    pub splat_count: usize,
    /// 启用 geom 的灯光衰减全为默认 `(1,0,0)`（上游 `light_attenuation_is_default`）。
    pub light_attenuation_is_default: bool,
    /// 模型含聚光灯（上游 `has_spot_lights`）。
    pub has_spot_lights: bool,
    /// 激活相机含正交投影（上游 `has_orthographic_camera`）。
    pub has_orthographic_camera: bool,
}

impl ResolvedContext {
    /// 激活相机数（上游 `nrender`）。
    pub fn nrender(&self) -> usize {
        self.plan.cameras.len()
    }

    /// 激活相机 id（上游 `cam_id_map`）。
    pub fn cam_id_map(&self) -> Vec<i32> {
        self.plan
            .cameras
            .iter()
            .map(|camera| *camera as i32)
            .collect()
    }

    /// 激活相机分辨率（上游 `cam_res`）。
    pub fn cam_res(&self) -> &[[u32; 2]] {
        &self.plan.resolutions
    }
}

/// 解析 28 参数与模型字段，得到宿主侧 render context。
pub fn resolve_context(
    fields: &RenderFields,
    params: &ContextParams<'_>,
) -> Result<ResolvedContext, ContextError> {
    let cameras = CameraModel::from_fields(fields)?;
    let requested = params.requested_cameras(cameras.ncam());

    // 上游列表/元组/布尔输入的复制规则由 S5 相机解析实现；这里把输入形态归一成切片。
    let mut tuple_slice = Vec::new();
    let cam_res_slice: Option<&[[u32; 2]]> = match params.cam_res {
        None => None,
        Some(CamRes::List(list)) => Some(list),
        Some(CamRes::Tuple(resolution)) => {
            if requested != 0 {
                tuple_slice.push(resolution);
            }
            Some(tuple_slice.as_slice())
        }
    };
    let mut rgb_slot = Vec::new();
    let mut depth_slot = Vec::new();
    let mut seg_slot = Vec::new();
    let rgb = normalize_flag(params.render_rgb, &mut rgb_slot, requested);
    let depth = normalize_flag(params.render_depth, &mut depth_slot, requested);
    let seg = normalize_flag(params.render_seg, &mut seg_slot, requested);
    let active = clone_active(&params.cam_active);
    let plan = resolve_cameras(
        &cameras,
        CameraRequest {
            active,
            cam_res: cam_res_slice,
            render: RenderTargets { rgb, depth, seg },
            samples_per_pixel: params.samples_per_pixel,
            use_precomputed_rays: params.use_precomputed_rays,
        },
    )?;

    let output = OutputLayout::resolve(&plan)?;
    validate_splats(params)?;
    let squashed = squash_geometry(fields, params.enabled_geom_groups)?;
    let lights = lights(fields)?;
    let headlight = headlight(fields)?;
    let skybox = skybox(fields, params.render_skybox)?;
    let background_color_packed = pack_background(params.background_color)?;
    let (znear, zfar) = near_far(fields)?;
    let camera_paths = camera_paths(&cameras, &plan);
    let has_orthographic_camera = plan.cameras.iter().any(|camera| {
        matches!(
            cameras.camera(*camera).map(|c| c.projection),
            Some(Projection::Orthographic)
        )
    });
    let bvh_ngeom = squashed.ids.len();

    Ok(ResolvedContext {
        nworld: params.nworld,
        plan,
        output,
        camera_paths,
        use_textures: params.use_textures,
        use_fast_math: params.use_fast_math,
        use_shadows: params.use_shadows,
        use_ambient_lighting: params.use_ambient_lighting,
        use_precomputed_rays: params.use_precomputed_rays,
        render_skybox: skybox.is_some(),
        skybox,
        enable_backface_culling: params.enable_backface_culling,
        shadow_light_fraction: params.shadow_light_fraction,
        samples_per_pixel: params.samples_per_pixel,
        enable_vertex_normals: params.enable_vertex_normals,
        enable_specular: params.enable_specular,
        enable_emission: params.enable_emission,
        enable_per_light_ambient: params.enable_per_light_ambient,
        flex_render_smooth: params.flex_render_smooth,
        background_color: params.background_color,
        background_color_packed,
        background_color_unpacked: unpack_background(background_color_packed),
        enabled_geom_ids: squashed.ids,
        bvh_ngeom,
        geom_ray_types: squashed.ray_types,
        znear,
        zfar,
        headlight,
        splat_count: params.splat.position.map_or(0, |position| position.len()),
        light_attenuation_is_default: lights.attenuation_is_default,
        has_spot_lights: lights.has_spot_lights,
        has_orthographic_camera,
    })
}

fn clone_active<'a>(active: &CameraActive<'a>) -> CameraActive<'a> {
    match active {
        CameraActive::All => CameraActive::All,
        CameraActive::Empty => CameraActive::Empty,
        CameraActive::Mask(mask) => CameraActive::Mask(mask),
        CameraActive::Names(names) => CameraActive::Names(names),
        CameraActive::Indices(indices) => CameraActive::Indices(indices),
    }
}

fn normalize_flag<'a>(
    spec: Option<RenderFlagSpec<'a>>,
    slot: &'a mut Vec<bool>,
    requested: usize,
) -> Option<&'a [bool]> {
    match spec {
        None => None,
        Some(RenderFlagSpec::List(list)) => Some(list),
        Some(RenderFlagSpec::Bool(value)) => {
            if requested == 0 {
                Some(&[])
            } else {
                slot.push(value);
                Some(slot.as_slice())
            }
        }
    }
}

fn validate_splats(params: &ContextParams<'_>) -> Result<(), ContextError> {
    let splat = &params.splat;
    if splat.position.is_none() {
        if splat.any_present() {
            return Err(ContextError::SplatAttributeMismatch);
        }
        return Ok(());
    }
    let count = splat.position.map_or(0, |position| position.len());
    for (field, provided) in [
        ("splat_rotation", splat.rotation.map(|values| values.len())),
        ("splat_scale", splat.scale.map(|values| values.len())),
        ("splat_rgba", splat.rgba.map(|values| values.len())),
    ] {
        let Some(actual) = provided else {
            return Err(ContextError::SplatMissing { field });
        };
        if actual != count {
            return Err(ContextError::SplatShape {
                field,
                expected: count,
                actual,
            });
        }
    }
    if let Some(group_id) = splat.group_id
        && group_id.len() != params.nworld
    {
        return Err(ContextError::SplatGroupCount {
            expected: params.nworld,
            actual: group_id.len(),
        });
    }
    Ok(())
}

fn camera_paths(model: &CameraModel, plan: &CameraPlan) -> Vec<CameraPath> {
    plan.cameras
        .iter()
        .map(|camera_id| {
            let camera = model.camera(*camera_id).expect("相机计划来自同一模型");
            let branch = if camera.sensorsize[1] == 0.0 {
                CameraBranch::Fovy
            } else {
                CameraBranch::Intrinsic
            };
            CameraPath {
                camera_id: *camera_id as i32,
                projection: camera.projection.code(),
                resolution: camera.resolution,
                sensorsize: camera.sensorsize,
                intrinsic: camera.intrinsic,
                fovy_degrees: camera.fovy,
                branch,
            }
        })
        .collect()
}

fn pack_background(color: [f64; 4]) -> Result<u32, ContextError> {
    let mut channels = [0u32; 4];
    for (channel, value) in color.iter().enumerate() {
        let scaled = value * 255.0;
        if !scaled.is_finite() || !(0.0..=255.0).contains(&scaled) {
            return Err(ContextError::BackgroundColor { channel });
        }
        channels[channel] = scaled as u32;
    }
    Ok((channels[3] << 24) | (channels[0] << 16) | (channels[1] << 8) | channels[2])
}

fn unpack_background(packed: u32) -> [f64; 4] {
    [
        f64::from((packed >> 16) & 0xFF),
        f64::from((packed >> 8) & 0xFF),
        f64::from(packed & 0xFF),
        f64::from((packed >> 24) & 0xFF),
    ]
}
