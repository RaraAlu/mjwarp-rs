#![allow(dead_code)]
//! R10-S2 多 world 端到端测试共享夹具（宿主构造 + 窗口 GPU 用例辅助）。
//!
//! 口径（方案 A，待 Chief 确认）：world0 用既有 nworld=1 冻结参考做数值验收；world1 用确定性
//! 第二 state（同模型多 state），验收 `nworld=2 的 world1 输出 == nworld=1 单独渲染同 state 逐位`
//! 与 world 隔离；不对上游 nworld=2 做数值结论。

use mjwarp_rs::model::render::RenderFields;
use mjwarp_rs::render::context::RenderFlagSpec;
use mjwarp_rs::render::context::owner::MeshState;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::{
    ContextParams, LeafPose, RenderContext, SceneLeaf, SceneState, TextureRegistry, resolve_context,
};
use mjwarp_rs::runtime::TransferSession;
use sha2::{Digest, Sha256};

/// 同模型多 state：把夹具的 leaf/位姿复制到 `nworld` 个 world（同几何；world>0 可整体平移 `world_dx`）。
pub fn state_worlds(
    fixture: &super::s7b1::SceneFixture,
    nworld: usize,
    world_dx: f32,
) -> SceneState {
    let mut leaves: Vec<SceneLeaf> = Vec::new();
    let mut poses: Vec<LeafPose> = Vec::new();
    for world in 0..nworld {
        for (leaf, pose) in fixture.leaves.iter().zip(&fixture.poses) {
            let mut leaf = *leaf;
            leaf.group = world as u32;
            leaves.push(leaf);
            let mut pose = *pose;
            if world > 0 {
                pose.xpos[0] += world_dx;
            }
            poses.push(pose);
        }
    }
    SceneState::new(leaves, poses, nworld).expect("多 world 场景状态")
}

/// 用给定 `StoryState`/网格/纹理创建 context（参数与 s7b1/s7b2 同源，避免重复解析模型）。
pub fn context_with_state(
    session: &TransferSession,
    fields: &RenderFields,
    params: &ContextParams<'_>,
    state: SceneState,
    meshes: Vec<MeshState>,
    use_textures: bool,
) -> RenderContext {
    let resolved = resolve_context(fields, params).expect("context 解析");
    let textures = TextureRegistry::from_fields(fields, use_textures).expect("纹理注册表");
    RenderContext::create(session, resolved, state, meshes, &textures).expect("context 创建")
}

/// depth/seg 口径参数（rgb/depth/seg 全开；其余默认）。
pub fn depth_seg_params(nworld: usize) -> ContextParams<'static> {
    ContextParams {
        nworld,
        ..Default::default()
    }
    .with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
    )
}

const DEFAULT_GEOM_GROUPS: [i32; 4] = [0, 1, 2, 3];

/// RGB（megakernel）口径参数：与 S0b `PARAMS_28` 默认一致，仅替换 `nworld`。
pub fn rgb_params(nworld: usize) -> ContextParams<'static> {
    ContextParams {
        nworld,
        enabled_geom_groups: &DEFAULT_GEOM_GROUPS,
        ..Default::default()
    }
    .with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
    )
}

/// `nworld × ncam` per-world 位姿（world-major）：world0 = `world0`，world1 = `world1`。
pub fn per_world_poses(world0: &[CameraPose], world1: &[CameraPose]) -> Vec<CameraPose> {
    let mut poses = Vec::with_capacity(world0.len() + world1.len());
    poses.extend_from_slice(world0);
    poses.extend_from_slice(world1);
    poses
}

/// 绕世界 z 轴旋转 `quarter_turns × 90°` 的相机位姿（0/±1 精确，位型不变式在测试内断言）。
pub fn rotated(pose: &CameraPose, quarter_turns: usize) -> CameraPose {
    let rz: [[f32; 3]; 3] = match quarter_turns % 4 {
        0 => [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        1 => [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        2 => [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]],
        _ => [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
    };
    let old = pose.xmat;
    let mut xmat = [0.0f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            xmat[3 * row + column] = rz[row][0] * old[column]
                + rz[row][1] * old[3 + column]
                + rz[row][2] * old[6 + column];
        }
    }
    CameraPose {
        xpos: pose.xpos,
        xmat,
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn sha256_depth(values: &[f32]) -> String {
    let mut hasher = Sha256::new();
    for value in values {
        hasher.update(value.to_bits().to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub fn sha256_seg(values: &[[i32; 2]]) -> String {
    let mut hasher = Sha256::new();
    for pair in values {
        hasher.update(pair[0].to_le_bytes());
        hasher.update(pair[1].to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub fn sha256_rgb(values: &[u32]) -> String {
    let mut hasher = Sha256::new();
    for value in values {
        hasher.update(value.to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}
