//! per-world L v2 判据实体 support（纯主机；测试期不启动 Python；零 GPU 用于合成自证）。
//!
//! 模型输入：`tests/support/render_bvh/light/model-315.json`（离线 3.15 dumper；provenance 见
//! `model-315.provenance.json`；不进 fixtures）。参考输入（G-L1 后存在）由实体运行时读取。
//!
//! 判据口径（spec v3 §4/§5/§6）：A 两台相机逐 world 严格位级；B 灯字段位型逐位相等；C 参考侧
//! 自算 C1（cam-persp rgb ≥1/255）与 C2（cam-hinge depth 位型 + seg pair），阈值 384/576、
//! 两对 world、两套不互换、无 allowlist/容差。另提供隔离/两跑 sha 助手（不替代 A/B/C）。
#![allow(dead_code)]

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::bvh::{mesh_half_extents, vertices_from_f64};
use mjwarp_rs::render::context::owner::MeshState;
use mjwarp_rs::render::context::{RenderFlagSpec, ResolvedContext};
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::{LightDataPose, WorldLightPoses};
use mjwarp_rs::render::{
    CameraActive, CameraModel, ContextParams, ImageOutput, LeafPose, OutputValues, RayBuffer,
    RayBuilder, RayMode, RenderContext, SceneLeaf, ScenePose, SceneState, TextureRegistry,
};
use mjwarp_rs::runtime::TransferSession;

use crate::{Frame, Reference, ReferenceState};

pub const MODEL: &str = include_str!("light/model-315.json");
pub const WORLDS: usize = 3;
pub const CAMERAS: usize = 3;
pub const ACTIVE_CAMERAS: [bool; 3] = [true, true, false];
pub const ACTIVE_CAMERA_COUNT: usize = 2;
pub const LEAVES: usize = 7;
pub const HEIGHT: usize = 240;
pub const WIDTH: usize = 320;
pub const PIXELS: usize = HEIGHT * WIDTH;
pub const GEOM_GROUPS: [i32; 4] = [0, 1, 2, 3];
pub const REFERENCE_DIR: &str =
    "../fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/";
pub const ACCEPT: usize = 384;
pub const MARGIN: usize = 576;

fn array(value: &serde_json::Value) -> &[serde_json::Value] {
    value.as_array().expect("数组")
}

fn numbers(value: &serde_json::Value) -> Vec<f64> {
    array(value)
        .iter()
        .map(|item| item.as_f64().expect("数值"))
        .collect()
}

pub fn ints(value: &serde_json::Value) -> Vec<i32> {
    numbers(value).iter().map(|item| *item as i32).collect()
}

pub fn model_numbers(document: &serde_json::Value, name: &str) -> Vec<f64> {
    numbers(&document["arrays"][name]["values"])
}

pub fn model_ints(document: &serde_json::Value, name: &str) -> Vec<i32> {
    ints(&document["arrays"][name]["values"])
}

#[rustfmt::skip]
fn field_values(kind: RenderFieldKind, value: &serde_json::Value) -> FieldData {
    let values = array(value);
    match kind {
        RenderFieldKind::Index => FieldData::Index(values.iter().map(|v| v.as_i64().unwrap() as i32).collect()),
        RenderFieldKind::Num => FieldData::Num(values.iter().map(|v| v.as_f64().unwrap()).collect()),
        RenderFieldKind::Flag => FieldData::Flag(values.iter().map(|v| v.as_u64().unwrap() as u8).collect()),
        RenderFieldKind::Size => FieldData::Size(values.iter().map(|v| v.as_i64().unwrap()).collect()),
        RenderFieldKind::Float => FieldData::Float(values.iter().map(|v| v.as_f64().unwrap() as f32).collect()),
    }
}

/// 3.15 dump -> 116 字段快照。
#[rustfmt::skip]
pub fn render_fields() -> RenderFields {
    let model: serde_json::Value = serde_json::from_str(MODEL).expect("model-315.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS.iter().map(|spec| field_values(spec.kind, &arrays[spec.name]["values"])).collect();
    RenderFields::new(data)
}

/// 场景几何（7 个 geom 全是 scene leaf；group 由 state 按 world 赋值）。
pub struct SceneGeometry {
    pub leaves: Vec<SceneLeaf>,
    pub meshes: Vec<MeshState>,
}

#[rustfmt::skip]
pub fn scene_geometry() -> SceneGeometry {
    let model: serde_json::Value = serde_json::from_str(MODEL).expect("model-315.json");
    let (geom_type, geom_size, geom_dataid) = (model_ints(&model, "geom_type"),
        model_numbers(&model, "geom_size"), model_ints(&model, "geom_dataid"));
    let (vert, face) = (model_numbers(&model, "mesh_vert"), model_ints(&model, "mesh_face"));
    let (vertadr, vertnum) = (model_ints(&model, "mesh_vertadr"), model_ints(&model, "mesh_vertnum"));
    let (faceadr, facenum) = (model_ints(&model, "mesh_faceadr"), model_ints(&model, "mesh_facenum"));
    let mut meshes = Vec::new();
    for mesh in 0..vertadr.len() {
        let start = vertadr[mesh] as usize * 3;
        let vertices = vertices_from_f64(&vert[start..start + vertnum[mesh] as usize * 3]).expect("网格顶点");
        let face_start = faceadr[mesh] as usize * 3;
        let indices = face[face_start..face_start + facenum[mesh] as usize * 3].iter().map(|v| *v as u32).collect();
        meshes.push(MeshState { vertices, indices });
    }
    let half = meshes.iter().map(|mesh| mesh_half_extents(&mesh.vertices).expect("mesh bounds")).collect::<Vec<_>>();
    let leaves = (0..LEAVES).map(|geom| {
        let mut size = [geom_size[3 * geom] as f32, geom_size[3 * geom + 1] as f32, geom_size[3 * geom + 2] as f32];
        if matches!(geom_type[geom], 7 | 8) { size = half[geom_dataid[geom] as usize]; }
        SceneLeaf { geom_type: geom_type[geom], size, group: 0, geom_id: geom as i32 }
    }).collect();
    SceneGeometry { leaves, meshes }
}

#[rustfmt::skip]
pub fn context_params() -> ContextParams<'static> {
    ContextParams {
        nworld: WORLDS,
        render_rgb: Some(RenderFlagSpec::Bool(true)),
        render_depth: Some(RenderFlagSpec::Bool(true)),
        render_seg: Some(RenderFlagSpec::Bool(true)),
        use_fast_math: false, use_shadows: true,
        enabled_geom_groups: &GEOM_GROUPS,
        cam_active: CameraActive::Mask(&ACTIVE_CAMERAS),
        shadow_light_fraction: 0.3, samples_per_pixel: 1,
        ..Default::default()
    }
}

/// per-world 几何位姿：`state.npz` 的 geom_xpos/xmat（world-major，leaf=geom id 顺序）。
#[rustfmt::skip]
pub fn scene_poses(state: &ReferenceState) -> mjwarp_rs::render::WorldScenePoses {
    let mut poses = Vec::with_capacity(WORLDS * LEAVES);
    for world in 0..WORLDS {
        for leaf in 0..LEAVES {
            let (base, mbase) = ((world * LEAVES + leaf) * 3, (world * LEAVES + leaf) * 9);
            poses.push(ScenePose {
                xpos: std::array::from_fn(|i| state.geom_xpos[base + i]),
                xmat: std::array::from_fn(|i| state.geom_xmat[mbase + i]),
            });
        }
    }
    mjwarp_rs::render::WorldScenePoses::new(WORLDS, LEAVES, poses).expect("per-world 场景位姿")
}

#[rustfmt::skip]
pub fn scene_state(geometry: &SceneGeometry) -> SceneState {
    let (mut leaves, mut poses) = (Vec::with_capacity(WORLDS * LEAVES), Vec::with_capacity(WORLDS * LEAVES));
    for world in 0..WORLDS {
        for leaf in &geometry.leaves {
            leaves.push(SceneLeaf { geom_type: leaf.geom_type, size: leaf.size, group: world as u32, geom_id: leaf.geom_id });
            poses.push(LeafPose { xpos: [0.0; 3], xmat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] });
        }
    }
    SceneState::new(leaves, poses, WORLDS).expect("场景状态")
}

/// per-world 灯位姿：`state.npz` 的 light_xpos/light_xdir 逐位透传（不 FK/不 f64/不重归一化）。
#[rustfmt::skip]
pub fn world_light_poses(state: &ReferenceState) -> WorldLightPoses {
    let mut poses = Vec::with_capacity(WORLDS * CAMERAS);
    for world in 0..WORLDS {
        for light in 0..CAMERAS {
            let base = (world * 3 + light) * 3;
            poses.push(LightDataPose {
                xpos: std::array::from_fn(|i| state.light_xpos[base + i]),
                xdir: std::array::from_fn(|i| state.light_xdir[base + i]),
            });
        }
    }
    WorldLightPoses::new(WORLDS, 3, poses).expect("per-world 灯位姿")
}

pub fn camera_poses(fields: &RenderFields) -> Vec<CameraPose> {
    CameraPose::from_fields(fields).expect("相机位姿")
}

pub fn camera_model(fields: &RenderFields) -> CameraModel {
    CameraModel::from_fields(fields).expect("相机模型")
}

#[rustfmt::skip]
pub fn precomputed_rays(session: &TransferSession, fields: &RenderFields, resolved: &ResolvedContext) -> RayBuffer {
    RayBuilder::new(session).expect("build_rays 编译")
        .build(&camera_model(fields), &resolved.plan, RayMode::Precomputed).expect("预计算射线")
}

#[rustfmt::skip]
pub fn create_context(session: &TransferSession, fields: &RenderFields, resolved: ResolvedContext,
                      geometry: &SceneGeometry) -> RenderContext {
    let textures = TextureRegistry::from_fields(fields, true).expect("纹理注册表");
    RenderContext::create(session, resolved, scene_state(geometry), geometry.meshes.clone(), &textures)
        .expect("context 创建")
}

/// 读回产品输出并按 `[world][cam]` 切片（cam 顺序 = 启用相机顺序）。
#[rustfmt::skip]
pub fn rendered_frames(context: &RenderContext, resolved: &ResolvedContext) -> Vec<Vec<Frame>> {
    let rgb = match context.outputs().read(ImageOutput::Rgb).expect("rgb 读回") {
        OutputValues::Rgb(values) => values, other => panic!("rgb 类型 {other:?}") };
    let depth = match context.outputs().read(ImageOutput::Depth).expect("depth 读回") {
        OutputValues::Depth(values) => values, other => panic!("depth 类型 {other:?}") };
    let seg = match context.outputs().read(ImageOutput::Segmentation).expect("seg 读回") {
        OutputValues::Seg(values) => values, other => panic!("seg 类型 {other:?}") };
    let layout = &resolved.output;
    (0..WORLDS).map(|world| (0..ACTIVE_CAMERA_COUNT).map(|cam| {
        let pixels = (resolved.cam_res()[cam][0] * resolved.cam_res()[cam][1]) as usize;
        let (rgb_start, depth_start) = (world * layout.rgb_pixels + layout.rgb_adr[cam].max(0) as usize,
                                        world * layout.depth_pixels + layout.depth_adr[cam].max(0) as usize);
        let seg_start = world * layout.seg_slots + layout.seg_adr[cam].max(0) as usize;
        Frame { rgb: rgb[rgb_start..rgb_start + pixels].to_vec(),
                depth: depth[depth_start..depth_start + pixels].to_vec(),
                seg: seg[seg_start..seg_start + pixels].to_vec() }
    }).collect()).collect()
}

fn ulp(a: f32, b: f32) -> u32 {
    let map = |value: f32| -> i64 {
        let bits = value.to_bits() as i32;
        if bits < 0 {
            0x8000_0000i64 - i64::from(bits)
        } else {
            i64::from(bits)
        }
    };
    (map(a) - map(b)).unsigned_abs() as u32
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameStats {
    pub rgb_mismatched_pixels: usize,
    pub rgb_max_byte_diff: u8,
    pub depth_mismatched_pixels: usize,
    pub depth_max_ulp: u32,
    pub depth_beyond_ulp: usize,
    pub seg_mismatched_pixels: usize,
}

/// 判据 A：单帧单相机严格位级比较。
#[rustfmt::skip]
pub fn compare_frame(actual: &Frame, expected: &Frame) -> FrameStats {
    assert_eq!(actual.rgb.len(), expected.rgb.len(), "rgb 像素数不同");
    assert_eq!(actual.depth.len(), expected.depth.len(), "depth 像素数不同");
    assert_eq!(actual.seg.len(), expected.seg.len(), "seg 像素数不同");
    let mut stats = FrameStats::default();
    for (left, right) in actual.rgb.iter().zip(&expected.rgb) {
        if left != right {
            stats.rgb_mismatched_pixels += 1;
            for channel in 0..3 {
                let shift = 16 - 8 * channel;
                let (a, b) = (((left >> shift) & 0xFF) as i32, ((right >> shift) & 0xFF) as i32);
                stats.rgb_max_byte_diff = stats.rgb_max_byte_diff.max((a - b).unsigned_abs() as u8);
            }
        }
    }
    for (left, right) in actual.depth.iter().zip(&expected.depth) {
        if left.to_bits() != right.to_bits() {
            stats.depth_mismatched_pixels += 1;
            stats.depth_max_ulp = stats.depth_max_ulp.max(ulp(*left, *right));
            stats.depth_beyond_ulp += 1; // depth 判据要求位级全等（0 ULP）
        }
    }
    stats.seg_mismatched_pixels = actual.seg.iter().zip(&expected.seg).filter(|(l, r)| l != r).count();
    stats
}

/// 判据 A：两台相机各自逐 world（返回 `[world][cam]` 统计）。
#[rustfmt::skip]
pub fn criterion_a(reference: &Reference, product: &[Vec<Frame>]) -> Vec<Vec<FrameStats>> {
    assert_eq!(product.len(), WORLDS);
    (0..WORLDS).map(|world| (0..ACTIVE_CAMERA_COUNT)
        .map(|cam| compare_frame(&product[world][cam], &reference.frames[world][cam])).collect()).collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LightStats {
    pub worlds: usize,
    pub lights: usize,
    pub xpos_mismatched: usize,
    pub xdir_mismatched: usize,
}

/// 判据 B：per-world 灯数据面位型比较（参考 state.npz vs 产品侧 WorldLightPoses）。
#[rustfmt::skip]
pub fn criterion_b(state: &ReferenceState, lights: &WorldLightPoses) -> LightStats {
    assert_eq!((lights.worlds(), lights.lights()), (WORLDS, 3));
    let mut stats = LightStats { worlds: WORLDS, lights: 3, ..LightStats::default() };
    for world in 0..WORLDS {
        for light in 0..3 {
            let base = (world * 3 + light) * 3;
            let pose = lights.get(world, light).expect("灯位姿");
            for axis in 0..3 {
                if pose.xpos[axis].to_bits() != state.light_xpos[base + axis].to_bits() { stats.xpos_mismatched += 1; }
                if pose.xdir[axis].to_bits() != state.light_xdir[base + axis].to_bits() { stats.xdir_mismatched += 1; }
            }
        }
    }
    stats
}

/// 判据 C 计数（参考侧自算；两对 world；相机归属写死）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CCounts {
    pub c1_rgb_changed: [usize; 2],
    pub c2_depth_changed: [usize; 2],
    pub c2_seg_changed: [usize; 2],
    pub diag_campersp_depth: [usize; 2],
    pub diag_campersp_seg: [usize; 2],
    pub diag_camhinge_rgb: [usize; 2],
}

pub const PAIRS: [(usize, usize); 2] = [(0, 1), (1, 2)];

fn rgb_channel_bytes(packed: u32) -> [i32; 3] {
    [
        ((packed >> 16) & 0xFF) as i32,
        ((packed >> 8) & 0xFF) as i32,
        (packed & 0xFF) as i32,
    ]
}

#[rustfmt::skip]
fn rgb_max_channel_delta(left: u32, right: u32) -> i32 {
    rgb_channel_bytes(left).iter().zip(rgb_channel_bytes(right)).map(|(a, b)| (a - b).abs()).max().unwrap_or(0)
}

/// 判据 C：参考侧自算 C1/C2 + 诊断项。
#[rustfmt::skip]
pub fn criterion_c(reference: &Reference) -> CCounts {
    let mut counts = CCounts::default();
    for (pair_index, (left, right)) in PAIRS.iter().enumerate() {
        let (campersp_l, campersp_r) = (&reference.frames[*left][0], &reference.frames[*right][0]);
        let (camhinge_l, camhinge_r) = (&reference.frames[*left][1], &reference.frames[*right][1]);
        for pixel in 0..PIXELS {
            if rgb_max_channel_delta(campersp_l.rgb[pixel], campersp_r.rgb[pixel]) >= 1 { counts.c1_rgb_changed[pair_index] += 1; }
            if campersp_l.depth[pixel].to_bits() != campersp_r.depth[pixel].to_bits() { counts.diag_campersp_depth[pair_index] += 1; }
            if campersp_l.seg[pixel] != campersp_r.seg[pixel] { counts.diag_campersp_seg[pair_index] += 1; }
            if camhinge_l.depth[pixel].to_bits() != camhinge_r.depth[pixel].to_bits() { counts.c2_depth_changed[pair_index] += 1; }
            if camhinge_l.seg[pixel] != camhinge_r.seg[pixel] { counts.c2_seg_changed[pair_index] += 1; }
            if rgb_max_channel_delta(camhinge_l.rgb[pixel], camhinge_r.rgb[pixel]) >= 1 { counts.diag_camhinge_rgb[pair_index] += 1; }
        }
    }
    counts
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    PassMargin,
    Insufficient,
    NonDiscriminating,
}

/// 判定（spec v3 §6）：任一 <384 -> 无判别力；任一 <576 -> 余量不足；否则 pass-margin。
#[rustfmt::skip]
pub fn verdict(counts: &CCounts) -> Verdict {
    let all = [counts.c1_rgb_changed[0], counts.c1_rgb_changed[1], counts.c2_depth_changed[0],
               counts.c2_depth_changed[1], counts.c2_seg_changed[0], counts.c2_seg_changed[1]];
    if all.iter().any(|value| *value < ACCEPT) { Verdict::NonDiscriminating }
    else if all.iter().any(|value| *value < MARGIN) { Verdict::Insufficient }
    else { Verdict::PassMargin }
}

#[rustfmt::skip]
pub fn sha256_frames(frames: &[Vec<Frame>]) -> [String; 3] {
    use sha2::{Digest, Sha256};
    let mut hashes: [String; 3] = Default::default();
    for (index, name) in ["rgb", "depth", "seg"].iter().enumerate() {
        let mut hasher = Sha256::new();
        for frame in frames.iter().flatten() {
            match *name {
                "rgb" => for value in &frame.rgb { hasher.update(value.to_le_bytes()); },
                "depth" => for value in &frame.depth { hasher.update(value.to_bits().to_le_bytes()); },
                _ => for pair in &frame.seg { hasher.update(pair[0].to_le_bytes()); hasher.update(pair[1].to_le_bytes()); },
            }
        }
        hashes[index] = format!("{:x}", hasher.finalize());
    }
    hashes
}

/// world 隔离：`changed[world][cam]` = 该 world 输出是否变化；仅允许 world1 变化。
#[rustfmt::skip]
pub fn isolation_changed(base: &[Vec<Frame>], moved: &[Vec<Frame>]) -> Vec<Vec<bool>> {
    (0..WORLDS).map(|world| (0..ACTIVE_CAMERA_COUNT)
        .map(|cam| base[world][cam] != moved[world][cam]).collect()).collect()
}

#[cfg(test)]
#[path = "light/tests.rs"]
mod tests;
