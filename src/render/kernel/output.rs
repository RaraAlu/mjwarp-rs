//! S7b1 参考夹具解析与逐像素 diff 报告（渲染/BVH S7b1）。
//!
//! 只读 `fixtures/render-bvh/scene-a/s7b/depth_seg.json`（由
//! `scripts/authoring/render-bvh-s7b/convert_depth_seg.py` 从 S0b `depth.npz`/`seg.npz`
//! CPU 派生）：depth 为 f32 位型 u32 十进制（行主序），seg 为 `(object_id, object_type)`
//! int32 对。本模块只做长度/形状的显式断言与逐像素 diff，不读 npz、不启动 Python。

use serde::Deserialize;

/// 验收的 depth 最大允许差（ULP，按 f32 位型有序化后比较）。
/// B-class premise (reference-premise criterion): the bound below holds only when the
/// comparison uses the same recipe, the same compilation form, the same GPU+driver and the same
/// warp version. If any premise fails the criterion is NOT applicable, and it is never a
/// standalone pass basis (see docs/development-evidence/criteria-purpose-audit/).
pub const MAX_DEPTH_ULP: i64 = 4;

/// 参考夹具形状/长度错误；出现即 fail，不静默跳过。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceError {
    Json(String),
    Length {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    CameraCount {
        expected: usize,
        actual: usize,
    },
    DepthAggregate {
        camera: usize,
        field: &'static str,
    },
    PixelCount {
        expected: usize,
        actual: usize,
    },
    /// 相机 data 位姿证据缺失/索引不连续/复算误差超限。
    CameraData,
}

impl std::fmt::Display for ReferenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(detail) => write!(formatter, "参考夹具 JSON：{detail}"),
            Self::Length {
                field,
                expected,
                actual,
            } => write!(formatter, "参考字段 {field} 长度 {actual} != {expected}"),
            Self::CameraCount { expected, actual } => {
                write!(formatter, "参考相机数 {actual} != {expected}")
            }
            Self::DepthAggregate { camera, field } => {
                write!(formatter, "参考相机 {camera} 的 {field} 与位型不一致")
            }
            Self::PixelCount { expected, actual } => {
                write!(formatter, "参考像素总数 {actual} != total_rays {expected}")
            }
            Self::CameraData => write!(formatter, "参考相机 data 位姿证据不完整或复算误差超限"),
        }
    }
}

impl std::error::Error for ReferenceError {}

#[derive(Deserialize)]
struct RawReference {
    task_id: String,
    encoding: RawEncoding,
    resolved: RawResolved,
    #[serde(default)]
    recipe: Option<RawRecipe>,
    #[serde(default)]
    camera_data_check: Vec<RawCameraData>,
    cameras: Vec<RawCamera>,
}

/// 同配方参考的编译口径（`depth_seg_fast_math_false.json` 才有 `recipe` 段）。
#[derive(Deserialize)]
struct RawRecipe {
    #[serde(default)]
    compile_fast_math: Option<bool>,
}

#[derive(Deserialize)]
struct RawEncoding {
    depth: String,
    seg: String,
}

#[derive(Deserialize)]
struct RawResolved {
    total_rays: usize,
    cam_res: Vec<[u32; 2]>,
    depth_adr: Vec<i32>,
    seg_adr: Vec<i32>,
}

#[derive(Deserialize)]
struct RawCameraData {
    index: usize,
    ray_count: usize,
    mode: i32,
    target_bodyid: i32,
    origin_bits: [u32; 3],
    xmat_bits: [u32; 9],
    #[serde(default)]
    xmat_vs_f64_max_abs_error: f64,
}

#[derive(Deserialize)]
struct RawCamera {
    index: usize,
    width: usize,
    height: usize,
    pixels: usize,
    depth_adr: i32,
    seg_adr: i32,
    depth_zero_pixels: usize,
    depth_min_positive_hex: Option<String>,
    depth_max_hex: String,
    depth_bits: Vec<u32>,
    seg_pairs: Vec<i32>,
    seg_unique: Vec<[i32; 2]>,
}

/// 已校验的参考夹具。
#[derive(Clone, Debug)]
pub struct DepthSegReference {
    pub task_id: String,
    pub depth_encoding: String,
    pub seg_encoding: String,
    pub total_rays: usize,
    pub cam_res: Vec<[u32; 2]>,
    pub depth_adr: Vec<i32>,
    pub seg_adr: Vec<i32>,
    /// `Some(false)` 表示同配方（`use_fast_math=False`）参考；S0b 冻结参考为 `None`。
    pub recipe_fast_math: Option<bool>,
    /// 相机 data 位姿（上游 `d.cam_xpos`/`d.cam_xmat` f32 位型）；顺序即激活相机顺序。
    pub camera_data: Vec<CameraData>,
    pub cameras: Vec<RefCamera>,
}

/// 单个相机的 data 位姿证据（由 pinned 3.15 `mjwarp.forward` 读取）。
///
/// 相机 1 是 TARGETBODY 相机：data `cam_xmat` 是跟踪 look-at 的产物，不是
/// `quat_to_mat(cam_quat)`；因此渲染内核的相机位姿必须按上游 data 语义显式传入。
#[derive(Clone, Debug, PartialEq)]
pub struct CameraData {
    pub index: usize,
    pub ray_count: usize,
    /// `mjCAMLIGHT_*` 模式：0 fixed、3 target body（本夹具只出现这两种）。
    pub mode: i32,
    pub target_bodyid: i32,
    pub origin_bits: [u32; 3],
    /// 行主序 f32 位型（u32 十进制）。
    pub xmat_bits: [u32; 9],
    /// 转换脚本用 f64 `quat_to_mat`/look-at 复算的 xmat 最大绝对误差。
    pub xmat_vs_f64_max_abs_error: f64,
}

/// 单个相机的全图参考。
#[derive(Clone, Debug)]
pub struct RefCamera {
    pub index: usize,
    pub width: usize,
    pub height: usize,
    pub pixels: usize,
    pub depth_adr: i32,
    pub seg_adr: i32,
    pub depth_zero_pixels: usize,
    pub depth_max_hex: String,
    /// 行主序 f32 位型（u32 十进制）。
    pub depth_bits: Vec<u32>,
    /// 行主序 `(object_id, object_type)` 对（长度 2*pixels）。
    pub seg_pairs: Vec<[i32; 2]>,
    pub seg_unique: Vec<[i32; 2]>,
}

impl RefCamera {
    /// f32 位型解码为 f32 值。
    pub fn depth_values(&self) -> Vec<f32> {
        self.depth_bits
            .iter()
            .map(|bits| f32::from_bits(*bits))
            .collect()
    }
}

/// 解析并校验参考夹具：长度必须与 `width*height`、`total_rays` 严格一致。
pub fn parse_reference(text: &str) -> Result<DepthSegReference, ReferenceError> {
    let raw: RawReference =
        serde_json::from_str(text).map_err(|error| ReferenceError::Json(error.to_string()))?;
    if raw.cameras.len() != raw.resolved.cam_res.len() && !raw.cameras.is_empty() {
        return Err(ReferenceError::CameraCount {
            expected: raw.resolved.cam_res.len(),
            actual: raw.cameras.len(),
        });
    }
    let mut cameras = Vec::with_capacity(raw.cameras.len());
    let mut total = 0usize;
    for camera in raw.cameras {
        let pixels = camera.width * camera.height;
        if camera.pixels != pixels {
            return Err(ReferenceError::Length {
                field: "pixels",
                expected: pixels,
                actual: camera.pixels,
            });
        }
        if camera.depth_bits.len() != pixels {
            return Err(ReferenceError::Length {
                field: "depth_bits",
                expected: pixels,
                actual: camera.depth_bits.len(),
            });
        }
        if camera.seg_pairs.len() != 2 * pixels {
            return Err(ReferenceError::Length {
                field: "seg_pairs",
                expected: 2 * pixels,
                actual: camera.seg_pairs.len(),
            });
        }
        let zeros = camera.depth_bits.iter().filter(|bits| **bits == 0).count();
        if zeros != camera.depth_zero_pixels {
            return Err(ReferenceError::DepthAggregate {
                camera: camera.index,
                field: "depth_zero_pixels",
            });
        }
        let minimum_positive = camera
            .depth_bits
            .iter()
            .copied()
            .filter(|bits| *bits != 0)
            .min();
        if camera
            .depth_min_positive_hex
            .as_deref()
            .map(str::to_ascii_lowercase)
            != minimum_positive.map(|bits| format!("0x{bits:08x}"))
        {
            return Err(ReferenceError::DepthAggregate {
                camera: camera.index,
                field: "depth_min_positive_hex",
            });
        }
        let maximum = camera.depth_bits.iter().copied().max().unwrap_or(0);
        if camera.depth_max_hex.to_ascii_lowercase() != format!("0x{maximum:08x}") {
            return Err(ReferenceError::DepthAggregate {
                camera: camera.index,
                field: "depth_max_hex",
            });
        }
        total += pixels;
        cameras.push(RefCamera {
            index: camera.index,
            width: camera.width,
            height: camera.height,
            pixels,
            depth_adr: camera.depth_adr,
            seg_adr: camera.seg_adr,
            depth_zero_pixels: camera.depth_zero_pixels,
            depth_max_hex: camera.depth_max_hex,
            depth_bits: camera.depth_bits,
            seg_pairs: {
                let (pairs, _) = camera.seg_pairs.as_chunks::<2>();
                pairs.to_vec()
            },
            seg_unique: camera.seg_unique,
        });
    }
    if total != raw.resolved.total_rays {
        return Err(ReferenceError::PixelCount {
            expected: raw.resolved.total_rays,
            actual: total,
        });
    }
    // 相机 data 位姿：每相机必须有一条记录，索引覆盖 `0..cameras.len()`，且
    // 转换脚本的 f64 复算误差必须在 1e-6 内（TARGETBODY look-at / fixed quat）。
    let mut camera_data = raw.camera_data_check;
    camera_data.sort_by_key(|check| check.index);
    if camera_data.len() != cameras.len()
        || camera_data
            .iter()
            .enumerate()
            .any(|(index, check)| check.index != index)
    {
        return Err(ReferenceError::CameraData);
    }
    for check in &camera_data {
        // 非有限值也拒绝（`!(finite && <=)` 而不是 `> 1e-6`，避免 NaN 绕过）。
        if !(check.xmat_vs_f64_max_abs_error.is_finite()
            && check.xmat_vs_f64_max_abs_error <= 1.0e-6)
        {
            return Err(ReferenceError::CameraData);
        }
        if !matches!(check.mode, 0 | 3) {
            return Err(ReferenceError::CameraData);
        }
    }
    Ok(DepthSegReference {
        task_id: raw.task_id,
        depth_encoding: raw.encoding.depth,
        seg_encoding: raw.encoding.seg,
        total_rays: raw.resolved.total_rays,
        cam_res: raw.resolved.cam_res.clone(),
        depth_adr: raw.resolved.depth_adr.clone(),
        seg_adr: raw.resolved.seg_adr.clone(),
        recipe_fast_math: raw
            .recipe
            .as_ref()
            .and_then(|recipe| recipe.compile_fast_math),
        camera_data: camera_data
            .into_iter()
            .map(|check| CameraData {
                index: check.index,
                ray_count: check.ray_count,
                mode: check.mode,
                target_bodyid: check.target_bodyid,
                origin_bits: check.origin_bits,
                xmat_bits: check.xmat_bits,
                xmat_vs_f64_max_abs_error: check.xmat_vs_f64_max_abs_error,
            })
            .collect(),
        cameras,
    })
}

impl DepthSegReference {
    pub fn camera(&self, index: usize) -> Option<&RefCamera> {
        self.cameras.get(index)
    }

    pub fn total_pixels(&self) -> usize {
        self.cameras.iter().map(|camera| camera.pixels).sum()
    }
}

/// 首个不匹配位置。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mismatch {
    pub camera: usize,
    pub pixel: usize,
    pub x: usize,
    pub y: usize,
}

/// depth 逐像素差异报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepthDiff {
    pub compared: usize,
    pub mismatched: usize,
    pub beyond_ulp: usize,
    pub max_ulp: i64,
    pub first: Option<Mismatch>,
}

/// seg 逐像素差异报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegDiff {
    pub compared: usize,
    pub mismatched: usize,
    pub first: Option<Mismatch>,
}

/// f32 位型转有序整数，ULP 差 = |a-b|（跨零连续）。
fn ordered(bits: u32) -> i64 {
    if bits & 0x8000_0000 == 0 {
        i64::from(bits)
    } else {
        -i64::from(bits & 0x7fff_ffff)
    }
}

/// 两个 f32 位型的 ULP 距离。
pub fn ulp_distance(left: u32, right: u32) -> i64 {
    (ordered(left) - ordered(right)).abs()
}

/// 逐像素比较 actual（f32 值）与参考位型。
/// B-class premise (reference-premise criterion): the readings produced here are only valid for
/// the same recipe, the same compilation form, the same GPU+driver and the same warp version;
/// when a premise fails the criterion is not applicable and it is never a standalone pass basis
/// (see docs/development-evidence/criteria-purpose-audit/).
pub fn diff_depth(actual: &[f32], reference: &RefCamera) -> Result<DepthDiff, ReferenceError> {
    if actual.len() != reference.depth_bits.len() {
        return Err(ReferenceError::Length {
            field: "actual_depth",
            expected: reference.depth_bits.len(),
            actual: actual.len(),
        });
    }
    let mut diff = DepthDiff {
        compared: 0,
        mismatched: 0,
        beyond_ulp: 0,
        max_ulp: 0,
        first: None,
    };
    for (pixel, (value, expected)) in actual.iter().zip(&reference.depth_bits).enumerate() {
        diff.compared += 1;
        let distance = ulp_distance(value.to_bits(), *expected);
        if distance == 0 {
            continue;
        }
        diff.mismatched += 1;
        if distance > MAX_DEPTH_ULP {
            diff.beyond_ulp += 1;
        }
        if distance > diff.max_ulp {
            diff.max_ulp = distance;
        }
        if diff.first.is_none() {
            diff.first = Some(Mismatch {
                camera: reference.index,
                pixel,
                x: pixel % reference.width,
                y: pixel / reference.width,
            });
        }
    }
    Ok(diff)
}

/// 逐像素比较 seg 对（要求逐像素全等）。
pub fn diff_seg(actual: &[[i32; 2]], reference: &RefCamera) -> Result<SegDiff, ReferenceError> {
    if actual.len() != reference.seg_pairs.len() {
        return Err(ReferenceError::Length {
            field: "actual_seg",
            expected: reference.seg_pairs.len(),
            actual: actual.len(),
        });
    }
    let mut diff = SegDiff {
        compared: 0,
        mismatched: 0,
        first: None,
    };
    for (pixel, (value, expected)) in actual.iter().zip(&reference.seg_pairs).enumerate() {
        diff.compared += 1;
        if value == expected {
            continue;
        }
        diff.mismatched += 1;
        if diff.first.is_none() {
            diff.first = Some(Mismatch {
                camera: reference.index,
                pixel,
                x: pixel % reference.width,
                y: pixel / reference.width,
            });
        }
    }
    Ok(diff)
}
