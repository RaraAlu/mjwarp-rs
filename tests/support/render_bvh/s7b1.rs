//! S7b1 渲染内核验收的共享夹具（纯主机，测试期不启动 Python）。
//!
//! 输入：
//! - `fixtures/render-bvh/scene-a/model-312.json`（S1 已验收的渲染模型输入）；
//! - `fixtures/render-bvh/scene-a/s2/reference_hits.json`（S2b 已验收的 f32 位姿）；
//! - `fixtures/render-bvh/scene-a/s7b/depth_seg.json`（S7b1 参考：全图 depth/seg 位型 +
//!   相机 data 位姿，authoring 期由 `convert_depth_seg.py` 从 S0b npz 与 pinned 3.15
//!   `mjwarp.forward` 派生）。
//!
//! 提供：场景位姿/网格/相机 context 构造、data 相机位姿解码、S5 预计算射线、输出读回、
//! 参考对照与哈希。渲染内核本身在 `mjwarp_rs::render::kernel`，本模块不访问 GPU。
#![allow(dead_code)]

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::bvh::{mesh_half_extents, vertices_from_f64};
use mjwarp_rs::render::context::owner::MeshState;
use mjwarp_rs::render::context::{RenderFlagSpec, ResolvedContext};
use mjwarp_rs::render::kernel::DepthSegRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::{
    DepthSegReference, MAX_DEPTH_ULP, diff_depth, diff_seg, ulp_distance,
};
use mjwarp_rs::render::{
    CameraModel, ContextParams, ImageOutput, LeafPose, OutputValues, RayBuffer, RayBuilder,
    RayMode, RenderContext, SceneLeaf, SceneState, TextureRegistry, resolve_context,
};
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;

// Env-gated criterion render-sdf-criterion-v2.1 support (see outline.rs).
#[path = "outline.rs"]
pub mod outline;

pub const MODEL: &str = include_str!("../../../fixtures/render-bvh/scene-a/model-312.json");
pub const REFERENCE: &str = include_str!("../../../fixtures/render-bvh/scene-a/s7b/depth_seg.json");
/// 同配方（`use_fast_math=False`）参考：depth ≤4 ULP 的验收口径；S0b fast-math 参考的差异
/// 只作「跨配方已知偏差」记录（Arch 2026-10-25 裁决），不当作通过依据。
pub const RECIPE_REFERENCE: &str =
    include_str!("../../../fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json");
pub const REFERENCE_HITS: &str =
    include_str!("../../../fixtures/render-bvh/scene-a/s2/reference_hits.json");

pub fn array(value: &Value) -> &[Value] {
    value.as_array().expect("数组")
}

pub fn numbers(value: &Value) -> Vec<f64> {
    array(value)
        .iter()
        .map(|item| item.as_f64().expect("数值"))
        .collect()
}

pub fn ints(value: &Value) -> Vec<i32> {
    numbers(value).iter().map(|item| *item as i32).collect()
}

pub fn u32s(value: &Value) -> Vec<u32> {
    numbers(value).iter().map(|item| *item as u32).collect()
}

pub fn model_ints(document: &Value, name: &str) -> Vec<i32> {
    ints(&document["arrays"][name]["values"])
}

pub fn model_numbers(document: &Value, name: &str) -> Vec<f64> {
    numbers(&document["arrays"][name]["values"])
}

/// data 相机位姿解码（上游 `d.cam_xpos`/`d.cam_xmat` f32 位型，按模型相机 id 索引）。
///
/// 相机 1 是 TARGETBODY 相机：`xmat` 是 forward 的跟踪 look-at 产物，不能从 `cam_quat` 重建。
pub fn camera_data_poses(
    reference: &DepthSegReference,
    model: &CameraModel,
    resolved: &ResolvedContext,
) -> Vec<CameraPose> {
    let mut poses = vec![
        CameraPose {
            xpos: [0.0; 3],
            xmat: [0.0; 9],
        };
        model.ncam()
    ];
    for (position, camera_id) in resolved.plan.cameras.iter().enumerate() {
        let data = &reference.camera_data[position];
        poses[*camera_id] = CameraPose {
            xpos: data.origin_bits.map(f32::from_bits),
            xmat: data.xmat_bits.map(f32::from_bits),
        };
    }
    poses
}

pub fn camera_modes(fields: &RenderFields) -> Vec<i32> {
    match fields.field("cam_mode") {
        Some(FieldData::Index(values)) => values.to_vec(),
        _ => Vec::new(),
    }
}

/// 相机 data 位姿证据自检：索引连续、fixed 相机与 f32 quat 路径一致、look-at 复算在限内。
pub fn check_camera_data(reference: &DepthSegReference, fields: &RenderFields) -> (f32, f32) {
    let poses = CameraPose::from_fields(fields).expect("f32 quat 相机位姿");
    let modes = camera_modes(fields);
    let (mut fixed_max, mut data_max) = (0.0f32, 0.0f32);
    for (index, data) in reference.camera_data.iter().enumerate() {
        assert_eq!(data.index, index, "相机 data 索引必须连续");
        assert!(
            data.xmat_vs_f64_max_abs_error <= 1.0e-6,
            "look-at/quat 复算误差超限"
        );
        if modes.get(index) == Some(&0) {
            let pose = poses[index];
            for axis in 0..3 {
                let data_axis = &data.xmat_bits[3 * axis..3 * axis + 3];
                for (column, bits) in data_axis.iter().enumerate() {
                    let expected = f32::from_bits(*bits);
                    fixed_max = fixed_max.max((pose.xmat[3 * axis + column] - expected).abs());
                }
            }
        }
        for bits in data.xmat_bits {
            data_max = data_max.max(f32::from_bits(bits).abs());
        }
    }
    (fixed_max, data_max)
}

fn field_values(kind: RenderFieldKind, value: &Value) -> FieldData {
    let values = array(value);
    match kind {
        RenderFieldKind::Index => FieldData::Index(
            values
                .iter()
                .map(|v| v.as_i64().expect("i32") as i32)
                .collect(),
        ),
        RenderFieldKind::Num => {
            FieldData::Num(values.iter().map(|v| v.as_f64().expect("f64")).collect())
        }
        RenderFieldKind::Flag => FieldData::Flag(
            values
                .iter()
                .map(|v| v.as_u64().expect("u8") as u8)
                .collect(),
        ),
        RenderFieldKind::Size => {
            FieldData::Size(values.iter().map(|v| v.as_i64().expect("i64")).collect())
        }
        RenderFieldKind::Float => FieldData::Float(
            values
                .iter()
                .map(|v| v.as_f64().expect("f32") as f32)
                .collect(),
        ),
    }
}

/// 把 S1 渲染字段夹具还原成 116 字段快照。
pub fn render_fields() -> RenderFields {
    let model: Value = serde_json::from_str(MODEL).expect("model-312.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| field_values(spec.kind, &arrays[spec.name]["values"]))
        .collect();
    RenderFields::new(data)
}

/// 场景参考：S1 模型字段 + S2b `reference_hits.json` 的 initial 位姿（f32 位型）。
pub struct SceneFixture {
    pub leaves: Vec<SceneLeaf>,
    pub poses: Vec<LeafPose>,
    pub meshes: Vec<MeshState>,
}

fn bits3(bits: &[u32], slot: usize) -> [f32; 3] {
    let base = 3 * slot;
    [
        f32::from_bits(bits[base]),
        f32::from_bits(bits[base + 1]),
        f32::from_bits(bits[base + 2]),
    ]
}

fn scene_meshes(model: &Value) -> Vec<MeshState> {
    let mesh_vert = model_numbers(model, "mesh_vert");
    let mesh_face = model_ints(model, "mesh_face");
    let vertadr = model_ints(model, "mesh_vertadr");
    let vertnum = model_ints(model, "mesh_vertnum");
    let faceadr = model_ints(model, "mesh_faceadr");
    let facenum = model_ints(model, "mesh_facenum");
    let mut meshes = Vec::new();
    for mesh in 0..vertadr.len() {
        let start = vertadr[mesh] as usize * 3;
        let vertices = vertices_from_f64(&mesh_vert[start..start + vertnum[mesh] as usize * 3])
            .expect("网格顶点");
        let face_start = faceadr[mesh] as usize * 3;
        let indices = mesh_face[face_start..face_start + facenum[mesh] as usize * 3]
            .iter()
            .map(|value| *value as u32)
            .collect();
        meshes.push(MeshState { vertices, indices });
    }
    meshes
}

pub fn scene_fixture() -> SceneFixture {
    let model: Value = serde_json::from_str(MODEL).expect("model-312.json");
    let reference: Value = serde_json::from_str(REFERENCE_HITS).expect("reference_hits.json");
    let enabled = ints(&reference["enabled_geom_ids"]);
    let xpos_bits = u32s(&reference["cases"]["initial"]["geom_xpos_bits"]);
    let xmat_bits = u32s(&reference["cases"]["initial"]["geom_xmat_bits"]);
    let geom_type = model_ints(&model, "geom_type");
    let geom_size = model_numbers(&model, "geom_size");
    let geom_dataid = model_ints(&model, "geom_dataid");
    let meshes = scene_meshes(&model);
    let mesh_half = meshes
        .iter()
        .map(|mesh| mesh_half_extents(&mesh.vertices).expect("网格 bounds"))
        .collect::<Vec<_>>();
    let mut leaves = Vec::new();
    let mut poses = Vec::new();
    for (slot, geom) in enabled.iter().enumerate() {
        let geom = *geom as usize;
        let mut size = [
            geom_size[3 * geom] as f32,
            geom_size[3 * geom + 1] as f32,
            geom_size[3 * geom + 2] as f32,
        ];
        if matches!(geom_type[geom], 7 | 8) {
            size = mesh_half[geom_dataid[geom] as usize];
        }
        leaves.push(SceneLeaf {
            geom_type: geom_type[geom],
            size,
            group: 0,
            geom_id: geom as i32,
        });
        let mut xmat = [0.0f32; 9];
        for (index, value) in xmat.iter_mut().enumerate() {
            *value = f32::from_bits(xmat_bits[9 * slot + index]);
        }
        poses.push(LeafPose {
            xpos: bits3(&xpos_bits, slot),
            xmat,
        });
    }
    SceneFixture {
        leaves,
        poses,
        meshes,
    }
}

pub fn session() -> TransferSession {
    TransferSession::new(0).expect("S7b1 GPU 验收需要 NVIDIA 驱动与 cuda-probe")
}

fn state_for(fixture: SceneFixture, nworld: usize) -> SceneState {
    let (mut leaves, mut poses) = (fixture.leaves, fixture.poses);
    if nworld > 1 {
        let count = leaves.len();
        for index in 0..count {
            let mut leaf = leaves[index];
            leaf.group = 1;
            leaves.push(leaf);
            let mut pose = poses[index];
            pose.xpos[0] += 1000.0;
            poses.push(pose);
        }
    }
    SceneState::new(leaves, poses, nworld).expect("场景状态")
}

/// 默认 context（两相机、rgb 全开）；`nworld=2` 时 world 1 几何 +1000 x（必定 miss）。
pub fn create_context(
    session: &TransferSession,
    fields: &RenderFields,
    nworld: usize,
    depth: bool,
    seg: bool,
) -> RenderContext {
    let params = ContextParams {
        nworld,
        ..Default::default()
    }
    .with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(depth)),
        Some(RenderFlagSpec::Bool(seg)),
    );
    let resolved = resolve_context(fields, &params).expect("context 解析");
    let fixture = scene_fixture();
    let meshes = fixture.meshes.clone();
    let state = state_for(fixture, nworld);
    let textures = TextureRegistry::from_fields(fields, true).expect("纹理注册表");
    RenderContext::create(session, resolved, state, meshes, &textures).expect("context 创建")
}

pub fn precomputed_rays(session: &TransferSession, fields: &RenderFields) -> RayBuffer {
    let model = CameraModel::from_fields(fields).expect("相机模型");
    let resolved = resolve_context(fields, &ContextParams::default()).expect("context 解析");
    RayBuilder::new(session)
        .expect("build_rays 编译")
        .build(&model, &resolved.plan, RayMode::Precomputed)
        .expect("S5 预计算射线")
}

/// 渲染一帧并读回 depth/seg；`poses` 是上游 data 语义的相机位姿。
pub fn render_and_read(
    renderer: &DepthSegRenderer,
    fields: &RenderFields,
    model: &CameraModel,
    context: &RenderContext,
    poses: &[CameraPose],
    rays: Option<&RayBuffer>,
) -> (Vec<f32>, Vec<[i32; 2]>) {
    renderer
        .render(context, fields, model, poses, rays, true)
        .expect("渲染发射");
    (read_depth(context), read_seg(context))
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

pub fn rgb_of(context: &RenderContext) -> Vec<u32> {
    match context.outputs().read(ImageOutput::Rgb).expect("rgb 读回") {
        OutputValues::Rgb(values) => values,
        other => panic!("rgb 类型 {other:?}"),
    }
}

pub fn output_hash(depth: &[f32], seg: &[[i32; 2]]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut mix = |value: u64| hash = (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3);
    for value in depth {
        mix(u64::from(value.to_bits()));
    }
    for pair in seg {
        mix(pair[0] as u32 as u64);
        mix(pair[1] as u32 as u64);
    }
    hash
}

pub fn bitwise_differences(left: &[f32], right: &[f32]) -> usize {
    left.iter()
        .zip(right)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count()
}

/// 两相机对照冻结参考；返回 (depth max ULP, depth 不匹配数, seg 不匹配数)。
/// 单相机 depth/seg 对照统计（参考 `depth_seg.json`）。
#[derive(Clone, Debug, Default)]
pub struct CameraStats {
    pub index: usize,
    pub depth_mismatch: usize,
    pub max_ulp: i64,
    /// ULP 直方图分箱 `[0, 1, 2, 3, 4, >4]`。
    pub histogram: [usize; 6],
    /// 超 4 ULP 像素按 seg geom id 计数（sphere 掠射放大参考 fast-math 噪声）。
    pub beyond_by_geom: Vec<(i32, usize)>,
}

/// 两相机参考对照统计；断言 seg 逐像素全等（可判定），depth 只统计不设容差。
///
/// `label` 用于输出区分「同配方参考（验收口径）」与「S0b fast-math 参考（跨配方已知偏差）」。
pub fn compare_reference(
    label: &str,
    depth: &[f32],
    seg: &[[i32; 2]],
    reference: &DepthSegReference,
) -> Vec<CameraStats> {
    let mut all = Vec::with_capacity(reference.cameras.len());
    for camera in &reference.cameras {
        let base = camera.depth_adr as usize;
        let seg_base = camera.seg_adr as usize;
        let actual_depth = &depth[base..base + camera.pixels];
        let actual_seg = &seg[seg_base..seg_base + camera.pixels];
        let mut diff = diff_depth(actual_depth, camera).expect("depth diff");
        let seg_diff = diff_seg(actual_seg, camera).expect("seg diff");
        assert_eq!(
            seg_diff.mismatched, 0,
            "相机 {} seg 不匹配（first={:?}）",
            camera.index, seg_diff.first
        );
        let expected = camera.depth_values();
        // criterion v2.1: the recipe comparison may consume the frozen outline list; the
        // cross-recipe (s0b fast-math) comparison stays raw.
        // criterion v2.1: the recipe comparison may consume the frozen outline list; the
        // cross-recipe (s0b fast-math) comparison stays raw.
        let exempted = label == "recipe"
            && outline::adjust_recipe_diff(&mut diff, actual_depth, &expected, camera.index);
        let mut stats = CameraStats {
            index: camera.index,
            depth_mismatch: diff.mismatched,
            max_ulp: diff.max_ulp,
            ..CameraStats::default()
        };
        for pixel in 0..camera.pixels {
            let ulp = ulp_distance(actual_depth[pixel].to_bits(), expected[pixel].to_bits());
            let bin = match ulp {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                _ => 5,
            };
            stats.histogram[bin] += 1;
            if !exempted && ulp > MAX_DEPTH_ULP {
                let geom = camera.seg_pairs[pixel][0];
                match stats.beyond_by_geom.iter_mut().find(|(id, _)| *id == geom) {
                    Some((_, count)) => *count += 1,
                    None => stats.beyond_by_geom.push((geom, 1)),
                }
            }
        }
        println!(
            "s7b1[{label}] cam{}: depth_mismatch={} max_ulp={} ulp_histogram(0,1,2,3,4,>4)={:?} \
             beyond4_by_geom={:?} seg_mismatch={}",
            stats.index,
            stats.depth_mismatch,
            stats.max_ulp,
            stats.histogram,
            stats.beyond_by_geom,
            seg_diff.mismatched
        );
        all.push(stats);
    }
    all
}

/// 单相机深度 ULP 超限像素数（按 geom 白名单累计）。
pub fn beyond_ulp_count(stats: &[CameraStats], geoms: &[i32]) -> usize {
    stats
        .iter()
        .flat_map(|camera| camera.beyond_by_geom.iter())
        .filter(|(geom, _)| geoms.contains(geom))
        .map(|(_, count)| count)
        .sum()
}

/// 全相机最大 ULP。
pub fn max_ulp(stats: &[CameraStats]) -> i64 {
    stats.iter().map(|camera| camera.max_ulp).max().unwrap_or(0)
}

/// 相机 data 位姿证据摘要（写入证据与测试输出）。
pub fn camera_data_errors(reference: &DepthSegReference) -> Vec<f64> {
    reference
        .camera_data
        .iter()
        .map(|data| data.xmat_vs_f64_max_abs_error)
        .collect()
}
