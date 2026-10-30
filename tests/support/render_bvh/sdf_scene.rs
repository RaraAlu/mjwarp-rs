#![allow(dead_code)]
//! U4 SDF 产品场景装配（作者侧测试辅助；GPU 用例在
//! `tests/render_sdf_reference.rs` 且 `#[ignore]`）。
//!
//! - 产品模型字段：`tests/support/render_bvh/sdf/model-312.json` 与
//!   `model-mixed-312.json`（离线由 S0a dumper `compile_scene_312.exe`（MuJoCo 3.12.0 C API）
//!   对两个签字 scene XML 生成，U4a 证据 `sdf-model-dump/`）。
//! - SDF 的渲染语义：上游把 SDF（gtype==8）当底层 mesh 渲染 ⇒ 本场景把 mesh 0 作为
//!   `MeshState` 交给 context，SDF leaf 的 `size` 用 **mesh bounds 半尺寸**
//!   （`mesh_half_extents`，与上游 `build_mesh_bvh` 同口径），**不是** `geom_size`。
//! - 本模块只负责装配/发射/读回；比较与判据断言在 `sdf_compare.rs`。
//! - 不改既有 support 文件、不改判据/容差/参考/fixtures。

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::bvh::mesh_half_extents;
use mjwarp_rs::render::context::RenderFlagSpec;
use mjwarp_rs::render::context::owner::MeshState;
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::{
    CameraModel, ContextParams, ImageOutput, LeafPose, OutputValues, RayBuffer, RayBuilder,
    RayMode, RenderContext, SceneLeaf, SceneState, TextureRegistry, resolve_context,
};
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::sdf::SdfGroup;

/// `scene.xml` 的 3.12 模型 dump（U4a 产物；sha 在 `render_sdf_reference.rs` host 用例里 pin）。
pub const MODEL_MESH: &str = include_str!("sdf/model-312.json");
/// `scene-mixed.xml` 的 3.12 模型 dump。
pub const MODEL_MIXED: &str = include_str!("sdf/model-mixed-312.json");

const GEOM_GROUPS: [i32; 4] = [0, 1, 2, 3];
const IDENTITY_XMAT: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// 捕获配方对应的 context 参数（与 U3 `PARAMS_28` 对齐：spp=1、预计算射线、
/// 三输出、`use_textures=true`、`use_fast_math=false`、`use_shadows` 由臂决定、几何组 0..3）。
pub fn capture_params(use_shadows: bool) -> ContextParams<'static> {
    ContextParams {
        nworld: 1,
        use_fast_math: false,
        use_shadows,
        enabled_geom_groups: &GEOM_GROUPS,
        ..ContextParams::default()
    }
    .with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
    )
}

/// 读取一组的 3.12 产品模型字段（`RENDER_FIELDS` 全字段）。
pub fn fields(group: SdfGroup) -> RenderFields {
    let text = match group {
        SdfGroup::Mesh => MODEL_MESH,
        SdfGroup::Occluder => MODEL_MIXED,
    };
    let model: Value = serde_json::from_str(text).expect("model-312.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| field_values(spec.kind, &arrays[spec.name]["values"]))
        .collect();
    RenderFields::new(data)
}

fn field_values(kind: RenderFieldKind, value: &Value) -> FieldData {
    let values = value.as_array().expect("字段数组");
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

/// `model-312.json` 里 mesh 0 的顶点（f32）与三角形索引（u32）。
pub fn mesh_state(group: SdfGroup) -> MeshState {
    let text = match group {
        SdfGroup::Mesh => MODEL_MESH,
        SdfGroup::Occluder => MODEL_MIXED,
    };
    let model: Value = serde_json::from_str(text).expect("model-312.json");
    let arrays = &model["arrays"];
    let vert = arrays["mesh_vert"]["values"].as_array().expect("mesh_vert");
    assert_eq!(vert.len() % 3, 0, "mesh_vert 长度");
    let vertices: Vec<[f32; 3]> = vert
        .chunks(3)
        .map(|chunk| {
            [
                chunk[0].as_f64().expect("vertex") as f32,
                chunk[1].as_f64().expect("vertex") as f32,
                chunk[2].as_f64().expect("vertex") as f32,
            ]
        })
        .collect();
    let face = arrays["mesh_face"]["values"].as_array().expect("mesh_face");
    assert_eq!(face.len() % 3, 0, "mesh_face 长度");
    let indices: Vec<u32> = face
        .iter()
        .map(|item| item.as_i64().expect("face index") as u32)
        .collect();
    MeshState { vertices, indices }
}

/// SDF 场景的 mesh bounds 半尺寸（leaf `size` 的真实来源）。
pub fn sdf_half(group: SdfGroup) -> [f32; 3] {
    mesh_half_extents(&mesh_state(group).vertices).expect("mesh half extents")
}

/// 产品一帧输出（与 `OutputLayout` 的 adr/pixels 对齐）。
#[derive(Clone, Debug)]
pub struct Rendered {
    pub rgb: Vec<u32>,
    pub depth: Vec<f32>,
    pub seg: Vec<[i32; 2]>,
}

/// 产品 SDF 场景（context + 相机 + 预计算射线）。
pub struct SdfScene {
    pub group: SdfGroup,
    pub use_shadows: bool,
    pub fields: RenderFields,
    pub context: RenderContext,
    pub model: CameraModel,
    pub poses: Vec<CameraPose>,
    pub rays: RayBuffer,
}

impl SdfScene {
    /// 装配一组产品场景：`resolve_context` → mesh 注入 → `SceneState` → `RenderContext::create`。
    pub fn new(session: &TransferSession, group: SdfGroup, use_shadows: bool) -> Self {
        let fields = fields(group);
        let params = capture_params(use_shadows);
        let resolved = resolve_context(&fields, &params).expect("context 解析");
        let mesh = mesh_state(group);
        let half = mesh_half_extents(&mesh.vertices).expect("mesh half extents");
        let state = scene_state(&fields, &resolved, group, half);
        let textures = TextureRegistry::from_fields(&fields, true).expect("纹理注册表");
        let context = RenderContext::create(session, resolved, state, vec![mesh], &textures)
            .expect("context 创建");
        let model = CameraModel::from_fields(&fields).expect("相机模型");
        let poses = CameraPose::from_fields(&fields).expect("相机位姿");
        let rays = RayBuilder::new(session)
            .expect("build_rays 编译")
            .build(&model, &context.resolved().plan, RayMode::Precomputed)
            .expect("S5 预计算射线");
        Self {
            group,
            use_shadows,
            fields,
            context,
            model,
            poses,
            rays,
        }
    }
}

/// 场景 leaf/pose：顺序 = `resolved.enabled_geom_ids`；SDF（8）`size` 来自 mesh bounds。
fn scene_state(
    fields: &RenderFields,
    resolved: &mjwarp_rs::render::ResolvedContext,
    group: SdfGroup,
    half: [f32; 3],
) -> SceneState {
    let geom_type = index_field(fields, "geom_type");
    let geom_dataid = index_field(fields, "geom_dataid");
    let geom_size = num_field(fields, "geom_size");
    let geom_pos = num_field(fields, "geom_pos");
    let geom_quat = num_field(fields, "geom_quat");
    let mut leaves = Vec::new();
    let mut poses = Vec::new();
    for geom in &resolved.enabled_geom_ids {
        let index = *geom as usize;
        let kind = geom_type[index];
        let dataid = geom_dataid[index];
        // 世界系静态场景：单位四元数；世界位姿 = geom_pos。
        assert_eq!(
            [
                geom_quat[4 * index],
                geom_quat[4 * index + 1],
                geom_quat[4 * index + 2],
                geom_quat[4 * index + 3],
            ],
            [1.0, 0.0, 0.0, 0.0],
            "geom {index} local quat 必须为单位"
        );
        let size = match kind {
            // mjGEOM_PLANE / mjGEOM_SPHERE / mjGEOM_BOX：geom_size 三元组
            0 | 2 | 6 => [
                geom_size[3 * index] as f32,
                geom_size[3 * index + 1] as f32,
                geom_size[3 * index + 2] as f32,
            ],
            // mjGEOM_SDF：底层 mesh，size = mesh bounds 半尺寸（**不是** geom_size）
            8 => {
                assert_eq!(dataid, 0, "SDF geom 必须引用 mesh id 0");
                assert!(group.expected_geoms() >= 3, "SDF 组至少 3 geom");
                half
            }
            other => panic!("未预期的 geom_type {other}（SDF 场景只有 plane/SDF/sphere/box）"),
        };
        leaves.push(SceneLeaf {
            geom_type: kind,
            size,
            group: 0,
            geom_id: *geom,
        });
        poses.push(LeafPose {
            xpos: [
                geom_pos[3 * index] as f32,
                geom_pos[3 * index + 1] as f32,
                geom_pos[3 * index + 2] as f32,
            ],
            xmat: IDENTITY_XMAT,
        });
    }
    assert_eq!(
        leaves.len(),
        group.expected_geoms(),
        "{} enabled geom 数",
        group.label()
    );
    SceneState::new(leaves, poses, 1).expect("场景状态")
}

/// 渲染一帧并读回 rgb/depth/seg。
pub fn fire(renderer: &MegakernelRenderer, scene: &SdfScene) -> Rendered {
    renderer
        .render(
            &scene.context,
            &scene.fields,
            &scene.model,
            &scene.poses,
            &scene.rays,
        )
        .expect("渲染发射");
    Rendered {
        rgb: read_rgb(&scene.context),
        depth: read_depth(&scene.context),
        seg: read_seg(&scene.context),
    }
}

pub fn read_rgb(context: &RenderContext) -> Vec<u32> {
    match context.outputs().read(ImageOutput::Rgb).expect("rgb 读回") {
        OutputValues::Rgb(values) => values,
        other => panic!("rgb 类型 {other:?}"),
    }
}

pub fn read_depth(context: &RenderContext) -> Vec<f32> {
    match context
        .outputs()
        .read(ImageOutput::Depth)
        .expect("depth 读回")
    {
        OutputValues::Depth(values) => values,
        other => panic!("depth 类型 {other:?}"),
    }
}

pub fn read_seg(context: &RenderContext) -> Vec<[i32; 2]> {
    match context
        .outputs()
        .read(ImageOutput::Segmentation)
        .expect("seg 读回")
    {
        OutputValues::Seg(values) => values,
        other => panic!("seg 类型 {other:?}"),
    }
}

/// SHA-256（host 用例 pin 模型 dump 用）。
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn index_field<'a>(fields: &'a RenderFields, name: &str) -> &'a [i32] {
    match fields.field(name) {
        Some(FieldData::Index(values)) => values,
        _ => panic!("渲染字段 {name} 缺失或类型不是 Index"),
    }
}

fn num_field<'a>(fields: &'a RenderFields, name: &str) -> &'a [f64] {
    match fields.field(name) {
        Some(FieldData::Num(values)) => values,
        _ => panic!("渲染字段 {name} 缺失或类型不是 Num"),
    }
}
