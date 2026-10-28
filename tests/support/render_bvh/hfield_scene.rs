#![allow(dead_code)]
//! R05-S3b hfield 产品场景装配与参考比较（作者侧测试辅助；GPU 用例在
//! `tests/render_hfield_reference.rs` 且 `#[ignore]`）。
//!
//! - 产品模型字段：`tests/support/render_bvh/hfield/model-312.json` 与
//!   `model-mixed-312.json`（离线由仓库既有 S0a dumper `compile_scene_312.exe`（MuJoCo 3.12 C API）
//!   对两个签字 scene XML 生成；provenance 见 R05-S3b authoring 证据 materials）。
//! - hfield 注入：`HfieldInputs` 取 R05-S3a 证据 `materials/reverse-calc.txt` 的 16 个 raw f32 位型
//!   （`hfield_data` 内存顺序）+ `size=[1.5,1.0,0.25,0.1]`、`nrow=ncol=4`、`adr=[0]`；
//!   只作为模型输入数据，**不是**渲染数值结论。
//! - `SceneLeaf.size`：hfield leaf 必须用 `build_hfield_mesh(...).half()`（S2 O5 约定，不得 host
//!   侧重推）；box leaf 用 `geom_size`。
//! - 判据：复用 S7b1/S7b2 的 diff 原语（RGB 每通道 ≤1 LSB、depth 位级优先（`mismatched==0`）
//!   且 `max_ulp<=4`、seg 全等）；本模块不新增/不放宽任何容差。

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::bvh::{HfieldMeshInput, build_hfield_mesh};
use mjwarp_rs::render::context::RenderFlagSpec;
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::{
    DepthDiff, MAX_DEPTH_ULP, RefCamera, SegDiff, diff_depth, diff_seg,
};
use mjwarp_rs::render::{
    CameraModel, ContextParams, HfieldInputs, ImageOutput, LeafPose, OutputValues, RayBuffer,
    RayBuilder, RayMode, RenderContext, SceneLeaf, SceneState, TextureRegistry, resolve_context,
};
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::hfield::{HfieldGroup, HfieldReference};
use super::s7b2::{RgbStats, diff_rgb};

/// `scene.xml` 的 3.12 模型 dump（dumper sha/命令见证据 materials）。
pub const MODEL_ONLY: &str = include_str!("hfield/model-312.json");
/// `scene-mixed.xml` 的 3.12 模型 dump。
pub const MODEL_MIXED: &str = include_str!("hfield/model-mixed-312.json");

/// R05-S3a `materials/reverse-calc.txt` [1] 记录的 16 个 raw f32 位型（`mjm.hfield_data` 内存顺序）。
///
/// 这是**模型输入数据**（MuJoCo 归一化 + 行序翻转 + 行主序存放后的 flat），不是渲染参考；
/// 用 `build_hfield_mesh` 做结构交叉核对（13 顶点 / 36 索引 / 12 三角形）见测试 host 自检。
pub const HFIELD_DATA_BITS: [u32; 16] = [
    0x3eba_2e8c,
    0x3f13_64d9,
    0x3f49_b26c,
    0x3f80_0000,
    0x3e78_3e10,
    0x3f22_e8ba,
    0x3f2a_aaab,
    0x3f60_f83f,
    0x3df8_3e10,
    0x3eaa_aaab,
    0x3f0b_a2e9,
    0x3f41_f07c,
    0x0000_0000,
    0x3e59_364e,
    0x3ed9_364e,
    0x3f22_e8ba,
];

/// 签字 `scene.xml` 的 hfield 尺寸（`size="1.5 1.0 0.25 0.1"`，f32 口径）。
pub const HFIELD_SIZE: [f32; 4] = [1.5, 1.0, 0.25, 0.1];
/// `nrow`。
pub const HFIELD_NROW: [i32; 1] = [4];
/// `ncol`。
pub const HFIELD_NCOL: [i32; 1] = [4];
/// `hfield_adr`。
pub const HFIELD_ADR: [i32; 1] = [0];

/// 捕获配方对应的 context 参数（与 R05-S3a `PARAMS_28` 对齐：spp=1、预计算射线、
/// 三输出、`use_textures=true`、`use_fast_math=false`、`use_shadows=true`、几何组 0..3）。
pub fn capture_params() -> ContextParams<'static> {
    ContextParams {
        nworld: 1,
        use_fast_math: false,
        use_shadows: true,
        enabled_geom_groups: &GEOM_GROUPS,
        ..ContextParams::default()
    }
    .with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
    )
}

const GEOM_GROUPS: [i32; 4] = [0, 1, 2, 3];
const IDENTITY_XMAT: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// hfield 注入数据（owned，供 `HfieldInputs` 借用）。
pub struct HfieldFixture {
    data: Vec<f32>,
}

impl HfieldFixture {
    /// 由 `HFIELD_DATA_BITS` 还原 f32 高程（位型保持）。
    pub fn new() -> Self {
        Self {
            data: HFIELD_DATA_BITS
                .iter()
                .map(|bits| f32::from_bits(*bits))
                .collect(),
        }
    }

    /// 高程数据切片（f32，行主序）。
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// `HfieldInputs` 借用视图（五字段全给）。
    pub fn inputs(&self) -> HfieldInputs<'_> {
        HfieldInputs {
            size: Some(&HFIELD_SIZE[..]),
            nrow: Some(&HFIELD_NROW[..]),
            ncol: Some(&HFIELD_NCOL[..]),
            adr: Some(&HFIELD_ADR[..]),
            data: Some(&self.data[..]),
        }
    }

    /// 宿主三角化 + mesh BVH（`half` 的真实来源，S2 O5）。
    pub fn mesh(&self) -> mjwarp_rs::render::bvh::HfieldMesh {
        build_hfield_mesh(&HfieldMeshInput {
            nrow: HFIELD_NROW[0],
            ncol: HFIELD_NCOL[0],
            size: [HFIELD_SIZE[0], HFIELD_SIZE[1], HFIELD_SIZE[2]],
            data: &self.data,
        })
        .expect("hfield 三角化")
    }
}

/// 读取一组的 3.12 产品模型字段（116 项 `RENDER_FIELDS`）。
pub fn fields(group: HfieldGroup) -> RenderFields {
    let text = match group {
        HfieldGroup::Only => MODEL_ONLY,
        HfieldGroup::Primitive => MODEL_MIXED,
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

/// 产品一帧输出（与 `OutputLayout` 的 adr/pixels 对齐）。
#[derive(Clone, Debug)]
pub struct Rendered {
    pub rgb: Vec<u32>,
    pub depth: Vec<f32>,
    pub seg: Vec<[i32; 2]>,
}

/// 产品 hfield 场景（context + 相机 + 预计算射线）。
pub struct HfieldScene {
    pub group: HfieldGroup,
    pub fields: RenderFields,
    pub context: RenderContext,
    pub model: CameraModel,
    pub poses: Vec<CameraPose>,
    pub rays: RayBuffer,
}

impl HfieldScene {
    /// 装配一组产品场景：`resolve_context` → 注入 hfield → `SceneState` → `RenderContext::create`。
    pub fn new(session: &TransferSession, group: HfieldGroup) -> Self {
        let fields = fields(group);
        let params = capture_params();
        let mut resolved = resolve_context(&fields, &params).expect("context 解析");
        let fixture = HfieldFixture::new();
        let mesh = fixture.mesh();
        let half = mesh.half();
        resolved.set_hfield(&fixture.inputs()).expect("hfield 注入");
        let state = scene_state(&fields, &resolved, group, half);
        let textures = TextureRegistry::from_fields(&fields, true).expect("纹理注册表");
        let context = RenderContext::create(session, resolved, state, Vec::new(), &textures)
            .expect("context 创建");
        let model = CameraModel::from_fields(&fields).expect("相机模型");
        let poses = CameraPose::from_fields(&fields).expect("相机位姿");
        let rays = RayBuilder::new(session)
            .expect("build_rays 编译")
            .build(&model, &context.resolved().plan, RayMode::Precomputed)
            .expect("S5 预计算射线");
        Self {
            group,
            fields,
            context,
            model,
            poses,
            rays,
        }
    }
}

/// 场景 leaf/pose：顺序 = `resolved.enabled_geom_ids`；hfield `size` 来自 `half`（S2 O5）。
fn scene_state(
    fields: &RenderFields,
    resolved: &mjwarp_rs::render::ResolvedContext,
    group: HfieldGroup,
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
        // 两组场景的 geom 都是世界原局部位姿（quat 单位、local pos 0）；世界位姿来自静态 body。
        assert_eq!(
            [
                geom_pos[3 * index],
                geom_pos[3 * index + 1],
                geom_pos[3 * index + 2]
            ],
            [0.0, 0.0, 0.0],
            "geom {index} local pos 必须为 0"
        );
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
        let (size, xpos) = match kind {
            // mjGEOM_HFIELD
            1 => {
                assert_eq!(dataid, 0, "hfield geom 必须引用 hfield id 0");
                (half, [0.0, 0.0, 0.0])
            }
            // mjGEOM_BOX（仅 mixed 组；世界 body pos 来自签字 XML L55）
            6 => {
                assert_eq!(group, HfieldGroup::Primitive, "box 只在 mixed 组");
                (
                    [
                        geom_size[3 * index] as f32,
                        geom_size[3 * index + 1] as f32,
                        geom_size[3 * index + 2] as f32,
                    ],
                    [0.62, -0.47, 0.45],
                )
            }
            other => panic!("未预期的 geom_type {other}（场景只有 hfield / box）"),
        };
        leaves.push(SceneLeaf {
            geom_type: kind,
            size,
            group: 0,
            geom_id: *geom,
        });
        poses.push(LeafPose {
            xpos,
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
pub fn fire(
    session: &TransferSession,
    renderer: &MegakernelRenderer,
    scene: &HfieldScene,
) -> Rendered {
    let _ = session;
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

/// 对一组参考逐相机比较（复用 S7b1/S7b2 的 diff 口径）。
pub fn compare_group(
    rendered: &Rendered,
    scene: &HfieldScene,
    reference: &HfieldReference,
) -> (Vec<RgbStats>, Vec<DepthDiff>, Vec<SegDiff>) {
    let layout = &scene.context.resolved().output;
    let mut rgb_stats = Vec::with_capacity(reference.rgb.len());
    for camera in &reference.rgb {
        let start = layout.rgb_adr[camera.index] as usize;
        let actual = &rendered.rgb[start..start + camera.pixels];
        rgb_stats.push(diff_rgb(actual, camera));
    }
    let mut depth_stats = Vec::with_capacity(reference.depth_seg.len());
    let mut seg_stats = Vec::with_capacity(reference.depth_seg.len());
    for camera in &reference.depth_seg {
        let depth_start = layout.depth_adr[camera.index] as usize;
        let depth_actual = &rendered.depth[depth_start..depth_start + camera.pixels];
        depth_stats.push(diff_depth(depth_actual, camera).expect("depth diff"));
        let seg_start = layout.seg_adr[camera.index] as usize;
        let seg_actual = &rendered.seg[seg_start..seg_start + camera.pixels];
        seg_stats.push(diff_seg(seg_actual, camera).expect("seg diff"));
    }
    (rgb_stats, depth_stats, seg_stats)
}

/// 判据断言（同 R09-S3b `assert_group_criteria`，不新增/不放宽）：
/// RGB 每通道 ≤1 LSB；depth 位级优先（`mismatched==0`）且 `max_ulp<=4` 且 `beyond_ulp==0`；seg 全等。
pub fn assert_group_criteria(
    label: &str,
    reference: &HfieldReference,
    rgb_stats: &[RgbStats],
    depth_stats: &[DepthDiff],
    seg_stats: &[SegDiff],
) {
    for (camera, stats) in reference.rgb.iter().zip(rgb_stats) {
        assert_eq!(
            stats.beyond_one_lsb_pixels,
            0,
            "{label} cam{} RGB 超 1 LSB：{} / {} 像素，max_byte_diff={} first={:?}",
            camera.index,
            stats.beyond_one_lsb_pixels,
            stats.compared,
            stats.max_byte_diff,
            stats.first
        );
    }
    for (camera, diff) in reference.depth_seg.iter().zip(depth_stats) {
        assert_eq!(
            diff.mismatched, 0,
            "{label} cam{} depth 必须与参考位级相同（first={:?}）",
            camera.index, diff.first
        );
        assert!(
            diff.max_ulp <= MAX_DEPTH_ULP,
            "{label} cam{} depth max_ulp={} 超 {}",
            camera.index,
            diff.max_ulp,
            MAX_DEPTH_ULP
        );
        assert_eq!(
            diff.beyond_ulp, 0,
            "{label} cam{} depth 超 4 ULP 像素 {}",
            camera.index, diff.beyond_ulp
        );
    }
    for (camera, diff) in reference.depth_seg.iter().zip(seg_stats) {
        assert_eq!(
            diff.mismatched, 0,
            "{label} cam{} seg 必须逐像素全等（first={:?}）",
            camera.index, diff.first
        );
    }
}

/// 参考相机深度统计摘要（比较规模/超判据计数/max ULP 打印用）。
pub fn camera_summary(camera: &RefCamera, depth: &DepthDiff, seg: &SegDiff) -> String {
    format!(
        "cam{} pixels={} depth_mismatch={} max_ulp={} beyond_ulp={} seg_mismatch={} \
         depth_zero_pixels={}",
        camera.index,
        camera.pixels,
        depth.mismatched,
        depth.max_ulp,
        depth.beyond_ulp,
        seg.mismatched,
        camera.depth_zero_pixels
    )
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn hash_rgb(values: &[u32]) -> String {
    let mut buffer = Vec::with_capacity(values.len() * 4);
    for value in values {
        buffer.extend_from_slice(&value.to_le_bytes());
    }
    sha256_hex(&buffer)
}

pub fn hash_depth(values: &[f32]) -> String {
    let mut buffer = Vec::with_capacity(values.len() * 4);
    for value in values {
        buffer.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    sha256_hex(&buffer)
}

pub fn hash_seg(values: &[[i32; 2]]) -> String {
    let mut buffer = Vec::with_capacity(values.len() * 8);
    for pair in values {
        buffer.extend_from_slice(&pair[0].to_le_bytes());
        buffer.extend_from_slice(&pair[1].to_le_bytes());
    }
    sha256_hex(&buffer)
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
