//! 渲染相机筛选、分辨率/输出解析与 znear/zfar（渲染/BVH S5）。
//!
//! 语义翻译冻结上游 `render_util.create_render_context` 的相机相关部分（MJWarp 3.15，
//! `mujoco_warp/_src/render_util.py`）：`cam_active` 选择、`cam_res` 解析、`render_rgb/
//! render_depth/render_seg` 的 None/标量/列表推导、`total_rays` 与 `znear/zfar`。
//! 射线本身在 `rays` 子模块（设备内核 `camera/cuda/build_rays.cu`）。
//!
//! 只做 camera-space 语义（局部射线方向/偏移）；`cam_xmat @ dir` 等世界变换属 S7b。

use crate::model::render::{FieldData, RenderFields, RenderObject};

use super::CameraError;

/// MuJoCo 投影类型（`mjtProjection`）：0=透视，1=正交。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Projection {
    Perspective,
    Orthographic,
}

impl Projection {
    /// 从 `mjtProjection` 码解析；其它值拒绝。
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Perspective),
            1 => Some(Self::Orthographic),
            _ => None,
        }
    }

    /// 反向编码。
    pub fn code(self) -> i32 {
        match self {
            Self::Perspective => 0,
            Self::Orthographic => 1,
        }
    }
}

/// MuJoCo `cam_output` 位：RGB=1<<0、DEPTH=1<<1、SEG=1<<4（`mjCAMOUT_*`）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CameraOutput {
    pub rgb: bool,
    pub depth: bool,
    pub seg: bool,
}

impl CameraOutput {
    /// 从 `cam_output` 位掩码解析。
    pub fn from_bits(bits: i32) -> Self {
        Self {
            rgb: bits & (1 << 0) != 0,
            depth: bits & (1 << 1) != 0,
            seg: bits & (1 << 4) != 0,
        }
    }
}

/// 单个相机的射线相关输入（模型值的类型口径与 MuJoCo 一致）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub projection: Projection,
    /// `cam_fovy`：double，内核侧转 f32。
    pub fovy: f64,
    pub sensorsize: [f32; 2],
    pub intrinsic: [f32; 4],
    pub resolution: [u32; 2],
    pub output: CameraOutput,
}

/// 相机模型输入：全部相机 + `stat.extent` + `vis.map.znear/zfar`（后两者原生 float）。
#[derive(Clone, Debug, PartialEq)]
pub struct CameraModel {
    cameras: Vec<Camera>,
    names: Vec<Option<Vec<u8>>>,
    stat_extent: f64,
    vis_znear: f32,
    vis_zfar: f32,
}

impl CameraModel {
    /// 直接构造（测试与已解析输入用）；相机名先用空表。
    pub fn new(cameras: Vec<Camera>, stat_extent: f64, vis_znear: f32, vis_zfar: f32) -> Self {
        let names = vec![None; cameras.len()];
        Self {
            cameras,
            names,
            stat_extent,
            vis_znear,
            vis_zfar,
        }
    }

    /// 附带相机名表（`name_bytes` 语义，供 `cam_active` 名称选择）。
    pub fn with_camera_names(mut self, names: Vec<Option<Vec<u8>>>) -> Self {
        self.names = names;
        self
    }

    /// 从 S1 已验收的渲染字段构造（字段类型/长度不符拒绝）。
    pub fn from_fields(fields: &RenderFields) -> Result<Self, CameraError> {
        let projection = field_ints(fields, "cam_projection")?;
        let ncam = projection.len();
        let fovy = field_nums(fields, "cam_fovy")?;
        let sensorsize = field_floats(fields, "cam_sensorsize")?;
        let intrinsic = field_floats(fields, "cam_intrinsic")?;
        let resolution = field_ints(fields, "cam_resolution")?;
        let output = field_ints(fields, "cam_output")?;
        for (field, expected, actual) in [
            ("cam_fovy", ncam, fovy.len()),
            ("cam_sensorsize", ncam * 2, sensorsize.len()),
            ("cam_intrinsic", ncam * 4, intrinsic.len()),
            ("cam_resolution", ncam * 2, resolution.len()),
            ("cam_output", ncam, output.len()),
        ] {
            if expected != actual {
                return Err(CameraError::LengthMismatch {
                    field,
                    expected,
                    actual,
                });
            }
        }
        let stat_extent = field_nums(fields, "stat.extent")?;
        if stat_extent.len() != 1 {
            return Err(CameraError::LengthMismatch {
                field: "stat.extent",
                expected: 1,
                actual: stat_extent.len(),
            });
        }
        let vis_znear = field_floats(fields, "vis.map.znear")?;
        let vis_zfar = field_floats(fields, "vis.map.zfar")?;
        if vis_znear.len() != 1 || vis_zfar.len() != 1 {
            return Err(CameraError::LengthMismatch {
                field: "vis.map.znear_zfar",
                expected: 1,
                actual: vis_znear.len().max(vis_zfar.len()),
            });
        }
        let mut cameras = Vec::with_capacity(ncam);
        for camera in 0..ncam {
            let projection = Projection::from_code(projection[camera]).ok_or(
                CameraError::UnsupportedProjection {
                    code: projection[camera],
                },
            )?;
            let width = resolution[2 * camera];
            let height = resolution[2 * camera + 1];
            if width < 0 || height < 0 {
                return Err(CameraError::NegativeResolution { camera });
            }
            cameras.push(Camera {
                projection,
                fovy: fovy[camera],
                sensorsize: [sensorsize[2 * camera], sensorsize[2 * camera + 1]],
                intrinsic: [
                    intrinsic[4 * camera],
                    intrinsic[4 * camera + 1],
                    intrinsic[4 * camera + 2],
                    intrinsic[4 * camera + 3],
                ],
                resolution: [width as u32, height as u32],
                output: CameraOutput::from_bits(output[camera]),
            });
        }
        let names = (0..ncam)
            .map(|camera| {
                fields
                    .name_bytes(RenderObject::Camera, camera as i64)
                    .map(Vec::from)
            })
            .collect();
        Ok(Self {
            cameras,
            names,
            stat_extent: stat_extent[0],
            vis_znear: vis_znear[0],
            vis_zfar: vis_zfar[0],
        })
    }

    /// 相机数。
    pub fn ncam(&self) -> usize {
        self.cameras.len()
    }

    /// 单相机输入。
    pub fn camera(&self, index: usize) -> Option<&Camera> {
        self.cameras.get(index)
    }

    /// `vis.map.znear * stat.extent` 的 f32 结果（上游 `float(...)` 口径）。
    pub fn znear(&self) -> f32 {
        (f64::from(self.vis_znear) * self.stat_extent) as f32
    }

    /// `vis.map.zfar * stat.extent` 的 f32 结果。
    pub fn zfar(&self) -> f32 {
        (f64::from(self.vis_zfar) * self.stat_extent) as f32
    }
}

/// `cam_active` 选择方式（与上游 create_render_context 的输入形态对应）。
pub enum CameraActive<'a> {
    /// `None`：全部相机。
    All,
    /// 显式空列表：不渲染任何相机（合法）。
    Empty,
    /// bool 掩码：长度必须等于模型相机数。
    Mask(&'a [bool]),
    /// 相机名列表：按模型名表解析成相机 id。
    Names(&'a [String]),
    /// 相机 id 列表（`cam_id_map` 的模型侧值）。
    Indices(&'a [i32]),
}

/// `render_rgb/render_depth/render_seg` 输入（None 表示按 `cam_output` 推导）。
#[derive(Clone, Copy, Default)]
pub struct RenderTargets<'a> {
    pub rgb: Option<&'a [bool]>,
    pub depth: Option<&'a [bool]>,
    pub seg: Option<&'a [bool]>,
}

/// 一次相机解析请求。
pub struct CameraRequest<'a> {
    pub active: CameraActive<'a>,
    pub cam_res: Option<&'a [[u32; 2]]>,
    pub render: RenderTargets<'a>,
    pub samples_per_pixel: u32,
    pub use_precomputed_rays: bool,
}

/// 解析后的相机计划；只含 S5 射线生成所需的字段（输出偏移属 S6a）。
#[derive(Clone, Debug, PartialEq)]
pub struct CameraPlan {
    /// 选中的模型相机 id（顺序即 `cam_id_map` 顺序）。
    pub cameras: Vec<usize>,
    pub resolutions: Vec<[u32; 2]>,
    pub render_rgb: Vec<bool>,
    pub render_depth: Vec<bool>,
    pub render_seg: Vec<bool>,
    /// 全部相机像素数之和。
    pub total_rays: usize,
    /// `render_rgb` 相机的像素数之和（AA 要求 >0）。
    pub rgb_pixels: usize,
    pub samples_per_pixel: u32,
    /// `samples_per_pixel²`。
    pub samples: usize,
    pub znear: f32,
    pub zfar: f32,
}

impl CameraPlan {
    /// 第 `position` 个相机在 `total_rays` 内的像素偏移。
    pub fn camera_pixel_offset(&self, position: usize) -> usize {
        self.resolutions[..position]
            .iter()
            .map(|resolution| resolution[0] as usize * resolution[1] as usize)
            .sum()
    }

    /// 相机像素数。
    pub fn camera_pixels(&self, position: usize) -> usize {
        let resolution = self.resolutions[position];
        resolution[0] as usize * resolution[1] as usize
    }

    /// 全局射线下标：`[sample][camera][row][column]`，x 最快。
    pub fn ray_index(&self, sample: usize, position: usize, x: u32, y: u32) -> usize {
        let resolution = self.resolutions[position];
        sample * self.total_rays
            + self.camera_pixel_offset(position)
            + y as usize * resolution[0] as usize
            + x as usize
    }
}

/// 解析相机选择、分辨率、输出开关与 `samples_per_pixel`；合法性在分配前检查。
pub fn resolve_cameras(
    model: &CameraModel,
    request: CameraRequest<'_>,
) -> Result<CameraPlan, CameraError> {
    let active = resolve_active(model, request.active)?;
    let resolutions = resolve_resolutions(model, &active, request.cam_res)?;
    let render_rgb = resolve_targets("render_rgb", model, &active, request.render.rgb, |c| {
        c.output.rgb
    })?;
    let render_depth =
        resolve_targets("render_depth", model, &active, request.render.depth, |c| {
            c.output.depth
        })?;
    let render_seg = resolve_targets("render_seg", model, &active, request.render.seg, |c| {
        c.output.seg
    })?;
    let total_rays = resolutions
        .iter()
        .map(|resolution| resolution[0] as usize * resolution[1] as usize)
        .sum();
    let rgb_pixels: usize = resolutions
        .iter()
        .zip(render_rgb.iter())
        .filter(|(_, enabled)| **enabled)
        .map(|(resolution, _)| resolution[0] as usize * resolution[1] as usize)
        .sum();
    let samples_per_pixel = request.samples_per_pixel;
    if samples_per_pixel < 1 {
        return Err(CameraError::SamplesPerPixelZero);
    }
    if samples_per_pixel > 1 && !request.use_precomputed_rays {
        return Err(CameraError::SamplesRequirePrecomputedRays {
            samples: samples_per_pixel,
        });
    }
    if samples_per_pixel > 1 && rgb_pixels == 0 {
        return Err(CameraError::SamplesRequireRgb);
    }
    let samples = (samples_per_pixel as usize)
        .checked_mul(samples_per_pixel as usize)
        .ok_or(CameraError::CountOverflow {
            field: "samples_per_pixel",
        })?;
    Ok(CameraPlan {
        cameras: active,
        resolutions,
        render_rgb,
        render_depth,
        render_seg,
        total_rays,
        rgb_pixels,
        samples_per_pixel,
        samples,
        znear: model.znear(),
        zfar: model.zfar(),
    })
}

fn resolve_active(
    model: &CameraModel,
    active: CameraActive<'_>,
) -> Result<Vec<usize>, CameraError> {
    let ncam = model.ncam();
    match active {
        CameraActive::All => Ok((0..ncam).collect()),
        CameraActive::Empty => Ok(Vec::new()),
        CameraActive::Mask(mask) => {
            if mask.len() != ncam {
                return Err(CameraError::ActiveMaskLength {
                    expected: ncam,
                    actual: mask.len(),
                });
            }
            Ok(mask
                .iter()
                .enumerate()
                .filter(|(_, enabled)| **enabled)
                .map(|(camera, _)| camera)
                .collect())
        }
        CameraActive::Names(names) => names
            .iter()
            .enumerate()
            .map(|(position, name)| {
                let wanted = name.as_bytes();
                (0..ncam)
                    .find(|camera| model.names[*camera].as_deref() == Some(wanted))
                    .ok_or(CameraError::UnknownCameraName { position })
            })
            .collect(),
        CameraActive::Indices(indices) => indices
            .iter()
            .enumerate()
            .map(|(position, camera)| {
                let camera = *camera;
                if camera < 0 || camera as usize >= ncam {
                    return Err(CameraError::CameraIndexOutOfRange {
                        position,
                        camera,
                        limit: ncam,
                    });
                }
                Ok(camera as usize)
            })
            .collect(),
    }
}

fn resolve_resolutions(
    model: &CameraModel,
    active: &[usize],
    cam_res: Option<&[[u32; 2]]>,
) -> Result<Vec<[u32; 2]>, CameraError> {
    let ncam = active.len();
    let Some(list) = cam_res else {
        return Ok(active
            .iter()
            .map(|camera| model.cameras[*camera].resolution)
            .collect());
    };
    if list.len() == 1 && ncam > 1 {
        return Ok(vec![list[0]; ncam]);
    }
    if list.len() != ncam {
        return Err(CameraError::ResolutionCount {
            expected: ncam,
            actual: list.len(),
        });
    }
    Ok(list.to_vec())
}

fn resolve_targets(
    field: &'static str,
    model: &CameraModel,
    active: &[usize],
    values: Option<&[bool]>,
    from_output: impl Fn(&Camera) -> bool,
) -> Result<Vec<bool>, CameraError> {
    let ncam = active.len();
    let Some(list) = values else {
        return Ok(active
            .iter()
            .map(|camera| from_output(&model.cameras[*camera]))
            .collect());
    };
    let resolved: Vec<bool> = if list.len() == model.ncam() && ncam != model.ncam() {
        active.iter().map(|camera| list[*camera]).collect()
    } else if list.len() == 1 && ncam > 1 {
        vec![list[0]; ncam]
    } else {
        list.to_vec()
    };
    if resolved.len() != ncam {
        return Err(CameraError::RenderFlagLength {
            field,
            expected: ncam,
            actual: resolved.len(),
        });
    }
    Ok(resolved)
}

fn field_ints<'a>(fields: &'a RenderFields, name: &'static str) -> Result<&'a [i32], CameraError> {
    match fields.field(name) {
        Some(FieldData::Index(values)) => Ok(values),
        Some(_) => Err(CameraError::FieldKind { field: name }),
        None => Err(CameraError::MissingField { field: name }),
    }
}

fn field_nums<'a>(fields: &'a RenderFields, name: &'static str) -> Result<&'a [f64], CameraError> {
    match fields.field(name) {
        Some(FieldData::Num(values)) => Ok(values),
        Some(_) => Err(CameraError::FieldKind { field: name }),
        None => Err(CameraError::MissingField { field: name }),
    }
}

fn field_floats<'a>(
    fields: &'a RenderFields,
    name: &'static str,
) -> Result<&'a [f32], CameraError> {
    match fields.field(name) {
        Some(FieldData::Float(values)) => Ok(values),
        Some(_) => Err(CameraError::FieldKind { field: name }),
        None => Err(CameraError::MissingField { field: name }),
    }
}
