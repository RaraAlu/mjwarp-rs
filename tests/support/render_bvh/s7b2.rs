#![allow(dead_code)]
//! S7b2 渲染内核验收的共享夹具（纯主机，测试期不启动 Python）。
//!
//! 输入：
//! - `fixtures/render-bvh/scene-a/s7b/rgb.json`：authoring 期由 `convert_rgb.py` 从 S0b
//!   `reference/rgb.npz` 派生（packed u32 ABGR + 解包口径 + `light_data_check` 位型）；
//! - S7b1 的夹具/支持函数（相机 data 位姿、S5 预计算射线、两份参考）继续复用。
//!
//! 解包口径：`float32(byte) * float32(1/255)`（上游 `unpack_rgb_kernel`）；
//! 判据：解包后每通道 ≤1 LSB（1/255）。

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::context::{RenderFlagSpec, ResolvedContext};
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::{LightDataPose, light_data_poses};
use mjwarp_rs::render::{
    CameraModel, ContextParams, ImageOutput, OutputValues, RenderContext, SceneState,
    TextureRegistry, resolve_context,
};
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;

use super::s7b1::{MODEL, REFERENCE, scene_fixture, session as s7b1_session};

/// S7b2 RGB 参考夹具（S0b `PARAMS_28.use_fast_math=True` 派生；跨配方偏差记录用）。
pub const RGB_REFERENCE: &str = include_str!("../../../fixtures/render-bvh/scene-a/s7b/rgb.json");

/// S7b2 RGB 同配方参考（`use_fast_math=False`，与产品 `WARP_1_15_RENDER` 配方一致；
/// `<=1 LSB` 硬指标依据，S7b1 depth 同配方先例的同构口径）。
pub const RGB_FAST_MATH_FALSE_REFERENCE: &str =
    include_str!("../../../fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json");

pub fn array(value: &Value) -> &[Value] {
    value.as_array().expect("数组")
}

pub fn ints(value: &Value) -> Vec<i64> {
    array(value)
        .iter()
        .map(|item| item.as_i64().expect("整数"))
        .collect()
}

pub fn u32s(value: &Value) -> Vec<u32> {
    array(value)
        .iter()
        .map(|item| item.as_u64().expect("u32") as u32)
        .collect()
}

/// 每相机 RGB 参考（packed u32 ABGR，row-major）。
#[derive(Clone, Debug)]
pub struct RgbCamera {
    pub index: usize,
    pub width: usize,
    pub height: usize,
    pub pixels: usize,
    pub packed: Vec<u32>,
}

/// `light_data_check`：pinned 3.15 forward 的 data light 位姿。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightBits {
    pub mode: i32,
    pub bodyid: i32,
    pub xpos: [u32; 3],
    pub xdir: [u32; 3],
}

#[derive(Clone, Debug)]
pub struct RgbReference {
    pub cameras: Vec<RgbCamera>,
    pub lights: Vec<LightBits>,
    pub total_pixels: usize,
    /// 采集脚本记录的跨配方 byte 差直方图 `[0,1,2,3,>3]`（只作偏差记录，不是判据）。
    pub cross_recipe_histograms: Vec<[usize; 5]>,
}

pub fn parse_rgb_reference() -> RgbReference {
    parse_rgb_document(RGB_REFERENCE)
}

pub fn parse_rgb_fast_math_false_reference() -> RgbReference {
    parse_rgb_document(RGB_FAST_MATH_FALSE_REFERENCE)
}

pub fn parse_rgb_document(text: &str) -> RgbReference {
    let document: Value = serde_json::from_str(text).expect("RGB 参考夹具 JSON");
    let cameras = array(&document["cameras"])
        .iter()
        .map(|camera| {
            let packed = u32s(&camera["rgb_bits_packed"]);
            RgbCamera {
                index: camera["index"].as_u64().expect("index") as usize,
                width: camera["width"].as_u64().expect("width") as usize,
                height: camera["height"].as_u64().expect("height") as usize,
                pixels: camera["pixels"].as_u64().expect("pixels") as usize,
                packed,
            }
        })
        .collect::<Vec<_>>();
    // `light_data_check` 只有 S0b 派生夹具带（同配方 RGB 采集不重复灯位型）。
    let lights = document["light_data_check"]
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .map(|light| LightBits {
                    mode: light["mode"].as_i64().expect("mode") as i32,
                    bodyid: light["bodyid"].as_i64().expect("bodyid") as i32,
                    xpos: u32s(&light["xpos_bits"]).try_into().expect("xpos"),
                    xdir: u32s(&light["xdir_bits"]).try_into().expect("xdir"),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let total_pixels = document["resolved"]["total_pixels"]
        .as_u64()
        .expect("total_pixels") as usize;
    let cross_recipe_histograms = document["cross_recipe"]["per_camera"]
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .map(|entry| {
                    let mut histogram = [0usize; 5];
                    for (slot, value) in histogram
                        .iter_mut()
                        .zip(array(&entry["histogram_0_1_2_3_gt3"]))
                    {
                        *slot = value.as_u64().expect("histogram 项") as usize;
                    }
                    histogram
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // 夹具长度不允许静默缺失：位型数组必须与声明像素数一致。
    for camera in &cameras {
        assert_eq!(
            camera.packed.len(),
            camera.pixels,
            "cam{} RGB 位型长度 {} != pixels {}",
            camera.index,
            camera.packed.len(),
            camera.pixels
        );
        assert_eq!(
            camera.pixels,
            camera.width * camera.height,
            "cam{} pixels != width*height",
            camera.index
        );
    }
    assert_eq!(
        cameras.iter().map(|camera| camera.pixels).sum::<usize>(),
        total_pixels,
        "夹具总像素数"
    );
    RgbReference {
        cameras,
        lights,
        total_pixels,
        cross_recipe_histograms,
    }
}

impl RgbReference {
    pub fn camera(&self, index: usize) -> Option<&RgbCamera> {
        self.cameras.iter().find(|camera| camera.index == index)
    }
}

/// 上游 `unpack_rgb_kernel`：`float32(byte) * float32(1/255)`。
pub fn unpack_rgb(packed: u32) -> [f32; 3] {
    let inv = 1.0f32 / 255.0f32;
    [
        ((packed >> 16) & 0xFF) as f32 * inv,
        ((packed >> 8) & 0xFF) as f32 * inv,
        (packed & 0xFF) as f32 * inv,
    ]
}

pub fn packed_byte(packed: u32, channel: usize) -> u8 {
    match channel {
        0 => ((packed >> 16) & 0xFF) as u8,
        1 => ((packed >> 8) & 0xFF) as u8,
        _ => (packed & 0xFF) as u8,
    }
}

/// 逐像素逐通道 byte 差（判据：解包后 ≤1 LSB ⇔ byte 差 ≤1）。
#[derive(Clone, Debug, Default)]
pub struct RgbStats {
    pub compared: usize,
    pub mismatched_pixels: usize,
    pub beyond_one_lsb_pixels: usize,
    pub max_byte_diff: u8,
    /// `(pixel, x, y, channel, expected_byte, actual_byte)`。
    pub first: Option<(usize, usize, usize, usize, u8, u8)>,
}

pub fn diff_rgb(actual: &[u32], camera: &RgbCamera) -> RgbStats {
    assert_eq!(actual.len(), camera.pixels, "RGB 像素数");
    let mut stats = RgbStats {
        compared: actual.len(),
        ..RgbStats::default()
    };
    for (pixel, (actual_pixel, expected_pixel)) in actual.iter().zip(&camera.packed).enumerate() {
        let mut pixel_mismatch = false;
        let mut pixel_beyond = false;
        for channel in 0..3 {
            let expected = packed_byte(*expected_pixel, channel);
            let got = packed_byte(*actual_pixel, channel);
            let diff = expected.abs_diff(got);
            if diff > 0 {
                pixel_mismatch = true;
                stats.max_byte_diff = stats.max_byte_diff.max(diff);
                if diff > 1 {
                    pixel_beyond = true;
                }
                if stats.first.is_none() {
                    stats.first = Some((
                        pixel,
                        pixel % camera.width,
                        pixel / camera.width,
                        channel,
                        expected,
                        got,
                    ));
                }
            }
        }
        if pixel_mismatch {
            stats.mismatched_pixels += 1;
        }
        if pixel_beyond {
            stats.beyond_one_lsb_pixels += 1;
        }
    }
    stats
}

/// RGB byte 差直方图 `[0,1,2,3,>3]` 与按 seg geom 的分布（诊断用）。
pub fn rgb_diff_histogram(
    actual: &[u32],
    camera: &RgbCamera,
    seg_pairs: &[[i32; 2]],
) -> (Vec<usize>, Vec<(i32, usize, u8)>) {
    let mut histogram = vec![0usize; 5];
    let mut by_geom: Vec<(i32, usize, u8)> = Vec::new();
    for (pixel, (actual_pixel, expected_pixel)) in actual.iter().zip(&camera.packed).enumerate() {
        let mut max_diff = 0u8;
        for channel in 0..3 {
            let diff =
                packed_byte(*expected_pixel, channel).abs_diff(packed_byte(*actual_pixel, channel));
            max_diff = max_diff.max(diff);
        }
        let bin = usize::from(max_diff).min(4);
        histogram[bin] += 1;
        if max_diff > 1 {
            let geom = seg_pairs.get(pixel).map(|pair| pair[0]).unwrap_or(-2);
            match by_geom.iter_mut().find(|(id, _, _)| *id == geom) {
                Some((_, count, max)) => {
                    *count += 1;
                    *max = (*max).max(max_diff);
                }
                None => by_geom.push((geom, 1, max_diff)),
            }
        }
    }
    (histogram, by_geom)
}
/// 渲染器返回的 RGB packed u32 输出。
pub fn read_rgb(context: &RenderContext) -> Vec<u32> {
    match context.outputs().read(ImageOutput::Rgb).expect("rgb 读回") {
        OutputValues::Rgb(values) => values,
        other => panic!("rgb 类型 {other:?}"),
    }
}

pub fn fields_from_model() -> RenderFields {
    let model: Value = serde_json::from_str(MODEL).expect("model-312.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| field_values(spec.kind, &arrays[spec.name]["values"]))
        .collect();
    RenderFields::new(data)
}

fn field_values(kind: RenderFieldKind, value: &Value) -> FieldData {
    let values = array(value);
    match kind {
        RenderFieldKind::Index => FieldData::Index(
            values
                .iter()
                .map(|item| item.as_i64().expect("i32") as i32)
                .collect(),
        ),
        RenderFieldKind::Num => FieldData::Num(
            values
                .iter()
                .map(|item| item.as_f64().expect("f64"))
                .collect(),
        ),
        RenderFieldKind::Flag => FieldData::Flag(
            values
                .iter()
                .map(|item| item.as_u64().expect("u8") as u8)
                .collect(),
        ),
        RenderFieldKind::Size => FieldData::Size(
            values
                .iter()
                .map(|item| item.as_i64().expect("i64"))
                .collect(),
        ),
        RenderFieldKind::Float => FieldData::Float(
            values
                .iter()
                .map(|item| item.as_f64().expect("f32") as f32)
                .collect(),
        ),
    }
}

/// 本片数据位姿断言：`light_data_poses`（渲染器实际使用）与夹具位型逐位相等。
pub fn check_light_data(reference: &RgbReference, fields: &RenderFields) -> Vec<LightDataPose> {
    let poses = light_data_poses(fields).expect("light_data_poses");
    assert_eq!(poses.len(), reference.lights.len(), "灯数量");
    for (pose, expected) in poses.iter().zip(&reference.lights) {
        assert_eq!(expected.mode, 0, "固定灯模式");
        assert_eq!(expected.bodyid, 0, "world body");
        assert_eq!(
            pose.xpos.map(f32::to_bits),
            expected.xpos,
            "light_xpos 位型必须与 pinned 3.15 forward 一致"
        );
        assert_eq!(
            pose.xdir.map(f32::to_bits),
            expected.xdir,
            "light_xdir 位型必须与 pinned 3.15 forward 一致"
        );
    }
    poses
}

pub fn session() -> TransferSession {
    s7b1_session()
}

/// 默认 context（与 S0b `PARAMS_28` 默认一致；`overrides` 用于拒绝面/纹理开关）。
pub fn create_context(
    session: &TransferSession,
    fields: &RenderFields,
    overrides: &ContextParams<'_>,
) -> RenderContext {
    let params = overrides;
    let resolved = resolve_context(fields, overrides).expect("context 解析");
    let fixture = scene_fixture();
    let state = SceneState::new(fixture.leaves, fixture.poses, params.nworld).expect("场景状态");
    let meshes = fixture.meshes.clone();
    let textures = TextureRegistry::from_fields(fields, params.use_textures).expect("纹理注册表");
    RenderContext::create(session, resolved, state, meshes, &textures).expect("context 创建")
}

/// S0b `PARAMS_28` 的默认值（`enabled_geom_groups=[0,1,2,3]`，含 group 3 的网格 geom）。
const DEFAULT_GEOM_GROUPS: [i32; 4] = [0, 1, 2, 3];

pub fn default_params<'a>() -> ContextParams<'a> {
    ContextParams {
        nworld: 1,
        enabled_geom_groups: &DEFAULT_GEOM_GROUPS,
        ..Default::default()
    }
    .with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
    )
}
pub fn camera_poses(
    reference: &mjwarp_rs::render::kernel::output::DepthSegReference,
    resolved: &ResolvedContext,
) -> Vec<CameraPose> {
    let model = CameraModel::from_fields(&render_fields()).expect("相机模型");
    super::s7b1::camera_data_poses(reference, &model, resolved)
}

pub fn render_fields() -> RenderFields {
    fields_from_model()
}

pub fn precomputed_rays(
    session: &TransferSession,
    fields: &RenderFields,
) -> mjwarp_rs::render::RayBuffer {
    super::s7b1::precomputed_rays(session, fields)
}

pub fn reference_depth_seg() -> mjwarp_rs::render::kernel::output::DepthSegReference {
    mjwarp_rs::render::kernel::output::parse_reference(REFERENCE).expect("S7b1 参考")
}

pub fn recipe_depth_seg() -> mjwarp_rs::render::kernel::output::DepthSegReference {
    mjwarp_rs::render::kernel::output::parse_reference(super::s7b1::RECIPE_REFERENCE)
        .expect("S7b1 同配方参考")
}

pub fn render(
    session: &TransferSession,
    renderer: &MegakernelRenderer,
    context: &RenderContext,
    fields: &RenderFields,
    model: &CameraModel,
    poses: &[CameraPose],
    rays: &mjwarp_rs::render::RayBuffer,
) {
    let _ = session;
    renderer
        .render(context, fields, model, poses, rays)
        .expect("S7b2 渲染发射");
}
