//! 渲染/BVH S5 验收：相机筛选/分辨率/输出解析与相机空间射线逐位对照。
//!
//! 参考：`fixtures/render-bvh/scene-a/s5/camera_rays.json`，由离线脚本
//! `scripts/authoring/render-bvh-s5/capture_s5_camera_rays.py` 在 GPU 上从冻结上游
//! RenderContext 读取 `rc.ray`/`rc.ray_offset` f32 位型捕获（相机空间 dir/offset，
//! 布局 `[sample][camera][row][column]`）。产品测试只读位型，不启动 Python。
//! AA 图像级验收延后 S7b/S8（receipt 记录 deferral）；本片只验射线层。
//!
//! 主机测试默认执行；GPU 用例默认 ignore：
//! cargo test --locked --features cuda-probe --test render_camera_rays -- --ignored --test-threads=1

use std::collections::BTreeMap;

#[cfg(feature = "cuda-probe")]
use mjwarp_rs::render::RayMode;
use mjwarp_rs::render::{
    Camera, CameraActive, CameraError, CameraModel, CameraOutput, CameraPlan, CameraRequest,
    Projection, RayBuffer, RenderTargets, planned_ray_count, resolve_cameras,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const FIXTURE: &str = include_str!("../fixtures/render-bvh/scene-a/s5/camera_rays.json");
const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[derive(Clone, Copy, Deserialize)]
#[serde(transparent)]
struct F32Bits(u32);

impl F32Bits {
    fn value(self) -> f32 {
        f32::from_bits(self.0)
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(transparent)]
struct F64Bits(u64);

impl F64Bits {
    fn value(self) -> f64 {
        f64::from_bits(self.0)
    }
}

#[derive(Deserialize, Clone)]
#[serde(tag = "kind", content = "values", rename_all = "lowercase")]
enum ActiveSpec {
    All,
    Empty,
    Mask(Vec<bool>),
    Names(Vec<String>),
    Indices(Vec<i32>),
}

impl ActiveSpec {
    fn request(&self) -> CameraActive<'_> {
        match self {
            Self::All => CameraActive::All,
            Self::Empty => CameraActive::Empty,
            Self::Mask(values) => CameraActive::Mask(values),
            Self::Names(values) => CameraActive::Names(values),
            Self::Indices(values) => CameraActive::Indices(values),
        }
    }
}

#[derive(Deserialize)]
struct Fixture {
    models: BTreeMap<String, FixtureModel>,
    contexts: BTreeMap<String, FixtureContext>,
    dynamic_equalities: BTreeMap<String, String>,
    rejections: Vec<Rejection>,
    determinism: Determinism,
    scope: Scope,
}

#[derive(Deserialize)]
struct Determinism {
    bitwise_identical: bool,
}

#[derive(Deserialize)]
struct Scope {
    aa_image: String,
}

#[derive(Deserialize)]
struct FixtureModel {
    cameras: Vec<FixtureCamera>,
    names: Vec<Option<String>>,
    stat_extent_bits: F64Bits,
    vis_znear_bits: F32Bits,
    vis_zfar_bits: F32Bits,
}

#[derive(Deserialize)]
struct FixtureCamera {
    projection: i32,
    fovy_bits: F64Bits,
    sensorsize_bits: [F32Bits; 2],
    intrinsic_bits: [F32Bits; 4],
    resolution: [u32; 2],
    output: i32,
}

#[derive(Deserialize)]
struct FixtureContext {
    model: String,
    request: FixtureRequest,
    expected: Expected,
}

#[derive(Deserialize)]
struct FixtureRequest {
    active: ActiveSpec,
    #[serde(default)]
    cam_res: Option<Vec<[u32; 2]>>,
    #[serde(default)]
    render_rgb: Option<Vec<bool>>,
    #[serde(default)]
    render_depth: Option<Vec<bool>>,
    #[serde(default)]
    render_seg: Option<Vec<bool>>,
    samples_per_pixel: u32,
    use_precomputed_rays: bool,
    mode: String,
}

impl FixtureRequest {
    fn request(&self) -> CameraRequest<'_> {
        CameraRequest {
            active: self.active.request(),
            cam_res: self.cam_res.as_deref(),
            render: RenderTargets {
                rgb: self.render_rgb.as_deref(),
                depth: self.render_depth.as_deref(),
                seg: self.render_seg.as_deref(),
            },
            samples_per_pixel: self.samples_per_pixel,
            use_precomputed_rays: self.use_precomputed_rays,
        }
    }
}

#[derive(Deserialize)]
struct Expected {
    cameras: Vec<usize>,
    cam_res: Vec<[u32; 2]>,
    render_rgb: Vec<bool>,
    render_depth: Vec<bool>,
    render_seg: Vec<bool>,
    total_rays: usize,
    rgb_pixels: usize,
    samples: usize,
    znear_bits: F32Bits,
    zfar_bits: F32Bits,
    count: usize,
    sha256: String,
    samples_entries: Vec<Entry>,
}

#[derive(Deserialize)]
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
struct Entry {
    camera: usize,
    sample: usize,
    x: u32,
    y: u32,
    index: usize,
    dir_bits: [u32; 3],
    offset_bits: [u32; 3],
}

#[derive(Deserialize)]
struct Rejection {
    model: String,
    kwargs: serde_json::Value,
    product_field: String,
    upstream_type: String,
}

impl Rejection {
    fn resolve(&self, model: &CameraModel) -> Result<CameraPlan, CameraError> {
        let samples_per_pixel = self.kwargs["samples_per_pixel"]
            .as_u64()
            .map_or(1, |value| value as u32);
        let use_precomputed_rays = self.kwargs["use_precomputed_rays"]
            .as_bool()
            .unwrap_or(true);
        let render_rgb = self.kwargs["render_rgb"].as_bool().map(|value| vec![value]);
        let cam_res: Option<Vec<[u32; 2]>> = self.kwargs["cam_res"].as_array().map(|items| {
            items
                .iter()
                .map(|item| {
                    let values: Vec<u32> = item
                        .as_array()
                        .expect("分辨率")
                        .iter()
                        .map(|v| v.as_u64().unwrap() as u32)
                        .collect();
                    [values[0], values[1]]
                })
                .collect()
        });
        let mut mask = Vec::new();
        let mut names = Vec::new();
        let mut indices = Vec::new();
        if let Some(items) = self.kwargs["cam_active"].as_array() {
            for item in items {
                if let Some(value) = item.as_bool() {
                    mask.push(value);
                } else if let Some(value) = item.as_str() {
                    names.push(value.to_string());
                } else if let Some(value) = item.as_i64() {
                    indices.push(value as i32);
                }
            }
        }
        let active = if !names.is_empty() {
            CameraActive::Names(&names)
        } else if !indices.is_empty() {
            CameraActive::Indices(&indices)
        } else if !mask.is_empty() {
            CameraActive::Mask(&mask)
        } else {
            CameraActive::All
        };
        resolve_cameras(
            model,
            CameraRequest {
                active,
                cam_res: cam_res.as_deref(),
                render: RenderTargets {
                    rgb: render_rgb.as_deref(),
                    depth: None,
                    seg: None,
                },
                samples_per_pixel,
                use_precomputed_rays,
            },
        )
    }
}

fn load() -> Fixture {
    serde_json::from_str(FIXTURE).expect("S5 相机射线夹具 JSON")
}

fn camera_model(fixture: &Fixture, name: &str) -> CameraModel {
    let source = fixture
        .models
        .get(name)
        .unwrap_or_else(|| panic!("缺少模型 {name}"));
    let cameras = source
        .cameras
        .iter()
        .map(|camera| Camera {
            projection: Projection::from_code(camera.projection).expect("投影码"),
            fovy: camera.fovy_bits.value(),
            sensorsize: [
                camera.sensorsize_bits[0].value(),
                camera.sensorsize_bits[1].value(),
            ],
            intrinsic: camera.intrinsic_bits.map(F32Bits::value),
            resolution: camera.resolution,
            output: CameraOutput::from_bits(camera.output),
        })
        .collect();
    CameraModel::new(
        cameras,
        source.stat_extent_bits.value(),
        source.vis_znear_bits.value(),
        source.vis_zfar_bits.value(),
    )
    .with_camera_names(
        source
            .names
            .iter()
            .map(|name| name.as_ref().map(|v| v.as_bytes().to_vec()))
            .collect(),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn product_field(error: &CameraError) -> &'static str {
    match error {
        CameraError::SamplesPerPixelZero => "samples_per_pixel",
        CameraError::SamplesRequirePrecomputedRays { .. } => "precomputed_rays_required",
        CameraError::SamplesRequireRgb => "render_rgb_required",
        CameraError::ResolutionCount { .. } => "cam_res",
        CameraError::ActiveMaskLength { .. } => "cam_active",
        CameraError::UnknownCameraName { .. } => "cam_active_name",
        CameraError::CameraIndexOutOfRange { .. } => "cam_active_index",
        other => panic!("未映射的相机错误 {other:?}"),
    }
}

#[test]
fn fixture_round_trips_models_and_requests() {
    let fixture = load();
    assert!(
        fixture.determinism.bitwise_identical,
        "两次捕获必须逐位一致"
    );
    assert!(
        fixture.scope.aa_image.contains("S7b"),
        "AA 图像级必须标注延后 S7b/S8"
    );
    for (name, context) in &fixture.contexts {
        let model = camera_model(&fixture, &context.model);
        let plan = resolve_cameras(&model, context.request.request())
            .unwrap_or_else(|error| panic!("{name} 解析失败: {error}"));
        let expected = &context.expected;
        assert_eq!(plan.cameras, expected.cameras, "{name} 相机选择");
        assert_eq!(plan.resolutions, expected.cam_res, "{name} 分辨率");
        assert_eq!(plan.render_rgb, expected.render_rgb, "{name} rgb");
        assert_eq!(plan.render_depth, expected.render_depth, "{name} depth");
        assert_eq!(plan.render_seg, expected.render_seg, "{name} seg");
        assert_eq!(plan.total_rays, expected.total_rays, "{name} total_rays");
        assert_eq!(plan.rgb_pixels, expected.rgb_pixels, "{name} rgb_pixels");
        assert_eq!(plan.samples, expected.samples, "{name} samples");
        assert_eq!(plan.znear.to_bits(), expected.znear_bits.0, "{name} znear");
        assert_eq!(plan.zfar.to_bits(), expected.zfar_bits.0, "{name} zfar");
        assert_eq!(
            planned_ray_count(&plan).expect("射线数"),
            expected.count,
            "{name} 条数"
        );
        for entry in &expected.samples_entries {
            assert_eq!(
                plan.ray_index(entry.sample, entry.camera, entry.x, entry.y),
                entry.index,
                "{name} 抽样下标 camera {} sample {}",
                entry.camera,
                entry.sample
            );
        }
    }
}

#[test]
fn empty_camera_lists_are_valid() {
    let fixture = load();
    for name in ["empty_active", "empty_mask"] {
        let context = &fixture.contexts[name];
        let model = camera_model(&fixture, &context.model);
        let plan = resolve_cameras(&model, context.request.request()).expect("空相机列表应合法");
        assert!(plan.cameras.is_empty() && plan.total_rays == 0);
        assert_eq!(planned_ray_count(&plan).unwrap(), 0);
        let buffer = RayBuffer::new(Vec::new(), Vec::new()).expect("空缓冲");
        assert_eq!(sha256_hex(&buffer.canonical_bytes()), EMPTY_SHA256);
        assert_eq!(context.expected.count, 0);
        assert_eq!(context.expected.sha256, EMPTY_SHA256);
    }
}

#[test]
fn dynamic_rays_match_precomputed_bits() {
    let fixture = load();
    for (dynamic, static_name) in &fixture.dynamic_equalities {
        let dynamic_context = &fixture.contexts[dynamic];
        let static_context = &fixture.contexts[static_name];
        assert_eq!(dynamic_context.request.mode, "dynamic");
        assert!(!dynamic_context.request.use_precomputed_rays);
        assert_eq!(
            dynamic_context.expected.sha256,
            static_context.expected.sha256
        );
        assert_eq!(
            dynamic_context.expected.count,
            static_context.expected.count
        );
    }
}

#[test]
fn rejection_matrix_maps_to_product_fields() {
    let fixture = load();
    for rejection in &fixture.rejections {
        let model = camera_model(&fixture, &rejection.model);
        let error = rejection.resolve(&model).err().unwrap_or_else(|| {
            panic!(
                "{} 应被拒绝（上游 {}）",
                rejection.product_field, rejection.upstream_type
            )
        });
        assert_eq!(
            product_field(&error),
            rejection.product_field,
            "上游 {}",
            rejection.upstream_type
        );
    }
}

#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use mjwarp_rs::runtime::TransferSession;

    fn builder() -> mjwarp_rs::render::RayBuilder {
        let session = TransferSession::new(0).expect("S5 GPU 验收需要 NVIDIA 驱动与 cuda-probe");
        mjwarp_rs::render::RayBuilder::new(&session).expect("build_rays.cu 应可在 NVRTC 下编译")
    }

    fn compare_entries(name: &str, buffer: &RayBuffer, entries: &[Entry]) {
        for entry in entries {
            let direction = buffer.directions()[entry.index];
            let offset = buffer.offsets()[entry.index];
            let actual_dir = direction.map(f32::to_bits);
            let actual_offset = offset.map(f32::to_bits);
            assert_eq!(
                actual_dir, entry.dir_bits,
                "{name} dir bits index {}",
                entry.index
            );
            assert_eq!(
                actual_offset, entry.offset_bits,
                "{name} offset bits index {}",
                entry.index
            );
        }
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn camera_rays_match_reference_for_every_context() {
        let builder = builder();
        let fixture = load();
        let mut checked = 0;
        for (name, context) in &fixture.contexts {
            let model = camera_model(&fixture, &context.model);
            let plan = resolve_cameras(&model, context.request.request()).expect("计划解析");
            let mode = if context.request.mode == "dynamic" {
                RayMode::Dynamic
            } else {
                RayMode::Precomputed
            };
            let buffer = builder.build(&model, &plan, mode).expect("射线生成");
            assert_eq!(buffer.len(), context.expected.count, "{name} 射线条数");
            compare_entries(name, &buffer, &context.expected.samples_entries);
            assert_eq!(
                sha256_hex(&buffer.canonical_bytes()),
                context.expected.sha256,
                "{name} 全网格 SHA-256"
            );
            checked += 1;
        }
        assert!(checked >= 15, "覆盖用例不足: {checked}");
        println!("camera-ray contexts checked: {checked}");
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn rejects_invalid_plans_before_allocating() {
        let builder = builder();
        let fixture = load();
        let context = &fixture.contexts["default_all"];
        let model = camera_model(&fixture, &context.model);
        let plan = resolve_cameras(&model, context.request.request()).unwrap();
        let mut broken = plan.clone();
        broken.samples = 2;
        assert!(
            builder
                .build(&model, &broken, RayMode::Precomputed)
                .is_err(),
            "samples 不一致应拒绝"
        );
        let mut broken = plan.clone();
        broken.cameras[0] = model.ncam();
        assert!(
            builder
                .build(&model, &broken, RayMode::Precomputed)
                .is_err(),
            "相机越界应拒绝"
        );
        let mut broken = plan.clone();
        broken.resolutions[0] = [0, 0];
        assert!(
            builder
                .build(&model, &broken, RayMode::Precomputed)
                .is_err(),
            "零分辨率应拒绝"
        );
        let mut broken = plan.clone();
        broken.samples_per_pixel = 2;
        broken.samples = 4;
        assert!(
            builder.build(&model, &broken, RayMode::Dynamic).is_err(),
            "动态模式 spp>1 应拒绝"
        );
        println!("builder rejection surface ok");
    }
}
