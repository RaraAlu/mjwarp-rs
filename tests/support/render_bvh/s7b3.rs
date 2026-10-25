#![allow(dead_code)]
//! S7b3 flag 变体矩阵与 AA 的共享夹具（纯主机，测试期不启动 Python）。
//!
//! 输入（authoring 期由 `capture_flags_aa.py` 以 pinned 3.15 + `use_fast_math=False`
//! 采集，`fixtures/render-bvh/scene-a/s7b/flags/`）：
//! - `<variant>_rgb.json`：两相机 packed u32 ABGR 全图（≤1 LSB 硬指标依据）；
//! - `<variant>_depth_seg.json`（仅 culling_off / aa2）：depth f32 位型 + seg 对
//!   （depth ≤4 ULP、seg 全等依据）；
//! - `index.json`：变体清单、override、文件 sha256、两次捕获位等、「变体 vs base」差异统计。
//!
//! 变体一处用运行时读文件（6 组夹具约 13 MB，避免 `include_str!` 把测试二进制撑大）；
//! 只读 JSON 位型，不引入 zip/inflate、不启动 Python。

use std::path::PathBuf;

use mjwarp_rs::model::render::RenderFields;
use mjwarp_rs::render::camera::CameraModel;
use mjwarp_rs::render::context::{RenderContext, RenderFlagSpec, ResolvedContext};
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::{DepthSegReference, parse_reference};
use mjwarp_rs::render::{
    ContextParams, RayBuffer, RayBuilder, RayMode, SceneState, TextureRegistry, resolve_context,
};
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;

use super::s7b1;
use super::s7b2::{self, RgbCamera};

/// scene-a 的 `enabled_geom_groups`（S0b `PARAMS_28`：含 group 3 的网格 geom）。
const DEFAULT_GEOM_GROUPS: [i32; 4] = [0, 1, 2, 3];

/// flag 变体名（与 authoring `VARIANTS` 顺序一致）。
pub const VARIANTS: [&str; 5] = [
    "textures_off",
    "vertex_normals_off",
    "specular_off",
    "emission_off",
    "culling_off",
];

/// AA 变体名（`samples_per_pixel=2` → nsamples=4）。
pub const AA_VARIANT: &str = "aa2";

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/render-bvh/scene-a/s7b/flags")
}

/// 变体夹具两相机 RGB 参考（packed u32 ABGR，row-major）。
#[derive(Clone, Debug)]
pub struct VariantRgb {
    pub variant: String,
    pub cameras: Vec<RgbCamera>,
}

/// 变体夹具 depth/seg 参考（仅 culling_off / aa2）。
#[derive(Clone, Debug)]
pub struct VariantDepthSeg {
    pub variant: String,
    pub reference: DepthSegReference,
}

fn read_text(name: &str) -> String {
    let path = fixtures_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读取夹具 {} 失败: {error}", path.display()))
}

fn read_json(name: &str) -> Value {
    let text = read_text(name);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("解析 {name} 失败: {error}"))
}

/// 变体 RGB 参考；夹具缺失直接 panic（不静默跳过）。
pub fn variant_rgb(name: &str) -> VariantRgb {
    let document = read_json(&format!("{name}_rgb.json"));
    assert_eq!(
        document["variant"].as_str(),
        Some(name),
        "{name} 夹具的 variant 字段"
    );
    let cameras = s7b2::array(&document["cameras"])
        .iter()
        .map(|camera| RgbCamera {
            index: camera["index"].as_u64().expect("index") as usize,
            width: camera["width"].as_u64().expect("width") as usize,
            height: camera["height"].as_u64().expect("height") as usize,
            pixels: camera["pixels"].as_u64().expect("pixels") as usize,
            packed: s7b2::u32s(&camera["rgb_bits_packed"]),
        })
        .collect::<Vec<_>>();
    assert_eq!(cameras.len(), 2, "{name} 两相机");
    for camera in &cameras {
        assert_eq!(
            camera.packed.len(),
            camera.pixels,
            "{name} cam{} 位型长度",
            camera.index
        );
    }
    assert_eq!(
        cameras.iter().map(|camera| camera.pixels).sum::<usize>(),
        120_000,
        "{name} 总像素"
    );
    VariantRgb {
        variant: name.to_string(),
        cameras,
    }
}

/// 变体 depth/seg 参考；不存在的变体返回 `None`（调用方按夹具设计断言）。
pub fn variant_depth_seg(name: &str) -> Option<VariantDepthSeg> {
    let path = fixtures_dir().join(format!("{name}_depth_seg.json"));
    if !path.exists() {
        return None;
    }
    let text = read_text(&format!("{name}_depth_seg.json"));
    let document: Value = serde_json::from_str(&text).expect("depth/seg 夹具 JSON");
    assert_eq!(
        document["variant"].as_str(),
        Some(name),
        "{name} depth/seg 夹具的 variant 字段"
    );
    let reference = parse_reference(&text).expect("深度/分割参考解析");
    Some(VariantDepthSeg {
        variant: name.to_string(),
        reference,
    })
}

/// `index.json`：变体清单与「变体 vs base」差异统计（采集期记录，判据来源）。
pub fn flags_index() -> Value {
    read_json("index.json")
}

/// 变体对应的 context 参数（与 authoring 采集的 override 一一对应）。
pub fn variant_params(name: &str) -> ContextParams<'static> {
    let base = default_params();
    match name {
        "textures_off" => ContextParams {
            use_textures: false,
            ..base
        },
        "vertex_normals_off" => ContextParams {
            enable_vertex_normals: false,
            ..base
        },
        "specular_off" => ContextParams {
            enable_specular: false,
            ..base
        },
        "emission_off" => ContextParams {
            enable_emission: false,
            ..base
        },
        "culling_off" => ContextParams {
            enable_backface_culling: false,
            ..base
        },
        AA_VARIANT => ContextParams {
            samples_per_pixel: 2,
            ..base
        },
        other => panic!("未知变体 {other}"),
    }
}

/// S0b `PARAMS_28` 的 28 参默认值（spp=1、预计算射线、两相机全开）。
pub fn default_params() -> ContextParams<'static> {
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

/// 按变体参数创建 context（与 S7b2 相同的场景/网格/纹理装配）。
pub fn create_variant_context(
    session: &TransferSession,
    fields: &RenderFields,
    variant: &str,
) -> RenderContext {
    create_context_with_params(session, fields, &variant_params(variant))
}

/// 按任意 `ContextParams` 创建 context（合成法线/基线用例复用同一装配）。
pub fn create_context_with_params(
    session: &TransferSession,
    fields: &RenderFields,
    params: &ContextParams<'_>,
) -> RenderContext {
    let resolved = resolve_context(fields, params).expect("context 解析");
    let fixture = s7b1::scene_fixture();
    let state = SceneState::new(fixture.leaves, fixture.poses, params.nworld).expect("场景状态");
    let meshes = fixture.meshes.clone();
    let textures = TextureRegistry::from_fields(fields, params.use_textures).expect("纹理注册表");
    RenderContext::create(session, resolved, state, meshes, &textures).expect("context 创建")
}

/// 以合成 `mesh_normal` 替换渲染字段（其余字段与 `model-312.json` 相同）。
///
/// 用于 `vertex_normals` 分支的判别：scene-a 的 mesh 顶点法线在 cam1 视角下与面法线
/// 差异被 8 位量化吸收（开/关 0 像素差异），换成极端方向后 `vertex_normal` 的
/// `dot(n, face) >= 0.8` 分支必然与纯面法线不同。
pub fn fields_with_mesh_normal(normals: &[f32]) -> RenderFields {
    let base = s7b2::fields_from_model();
    let data = mjwarp_rs::model::render::RENDER_FIELDS
        .iter()
        .enumerate()
        .map(|(index, spec)| match spec.name {
            "mesh_normal" => mjwarp_rs::model::render::FieldData::Float(normals.to_vec()),
            _ => base.get(index).expect("字段按 RENDER_FIELDS 顺序").clone(),
        })
        .collect();
    RenderFields::new(data)
}

/// 默认（spp=1）预计算射线（S5 `RayMode::Precomputed`）。
pub fn precomputed_rays(session: &TransferSession, fields: &RenderFields) -> RayBuffer {
    s7b2::precomputed_rays(session, fields)
}

/// AA（spp=2 → nsamples=4）的预计算子样本射线：布局 `[sample][camera][row][column]`，
/// 与上游 `for sample in range(nsamples)` 的 `ray_base = sample * total_rays` 一致。
pub fn aa_rays(session: &TransferSession, fields: &RenderFields) -> RayBuffer {
    let model = CameraModel::from_fields(fields).expect("相机模型");
    let resolved = resolve_context(fields, &variant_params(AA_VARIANT)).expect("AA context 解析");
    assert_eq!(resolved.plan.samples, 4, "spp=2 → nsamples=4");
    assert_eq!(resolved.plan.samples_per_pixel, 2);
    RayBuilder::new(session)
        .expect("build_rays 编译")
        .build(&model, &resolved.plan, RayMode::Precomputed)
        .expect("AA 子样本射线")
}

/// 相机 data 位姿（与 S7b1/S7b2 相同的显式入参：两相机 data 语义）。
pub fn camera_data_poses(fields: &RenderFields, resolved: &ResolvedContext) -> Vec<CameraPose> {
    let model = CameraModel::from_fields(fields).expect("相机模型");
    let reference = parse_reference(s7b1::RECIPE_REFERENCE).expect("S7b1 同配方参考");
    s7b1::camera_data_poses(&reference, &model, resolved)
}

/// 渲一帧（不做断言）。
pub fn render_variant(
    session: &TransferSession,
    renderer: &MegakernelRenderer,
    context: &RenderContext,
    fields: &RenderFields,
    poses: &[CameraPose],
    rays: &RayBuffer,
) {
    let _ = session;
    let model = CameraModel::from_fields(fields).expect("相机模型");
    renderer
        .render(context, fields, &model, poses, rays)
        .expect("S7b3 渲染发射");
}
