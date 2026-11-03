//! 逐 world 渲染实现 S1（方案 L）：host 逐 world 循环 + 单 world 发射。
//!
//! 机制（证据口径见 `docs/development-evidence/render-bvh/perworld-impl-s1/`）：
//! - **零 `.cu` / 零 ABI 改动**：内核仍是既有 `render_megakernel`，81 槽签名与 `worldid` 语义不变。
//! - 单 world 发射时内核 `worldid` 恒为 0：发射前把目标 world 的 geom 位姿铺到**每个 world 组**
//!   再 `refit_bvh`，于是该 world 的几何处于 root[0]（`cast_ray` 的 `roots[worldid]` 命中目标
//!   world）；相机只喂目标 world 行；rgb/depth/seg 的 per-camera `adr` 平移到目标 world 区段
//!   （`world × pitch + adr`），内核 `worldid×pitch` 恒为 0，最终写到该 world 的全局区段。
//! - 方案 L-2（每 world 另建 context + D2D 拷贝）与 L-3（抽 `megakernel_args.rs`）不采用：
//!   前者违背「零 D2D / 零第二套 context」前提，后者超出本片写入面。
//! - v1 只覆盖 per-world geom 位姿；灯仍模型级单份（v1 不注入 per-world 灯）。
//! - AA：`nsamples > 1` 显式拒绝（`LauncherError::Unsupported`），不静默按单样本渲染。

use crate::model::render::RenderFields;
use crate::render::camera::CameraModel;
use crate::render::context::world::{WorldError, WorldScenePoses};
use crate::render::context::{LeafPose, RenderContext};
use crate::render::kernel::cast::CameraPose;
use crate::render::rays::RayBuffer;
use crate::runtime::launcher::LauncherError;

use super::megakernel::MegakernelRenderer;

/// AA / `spp>1` 守卫：per-world 入口只接受 `nsamples == 1`（纯函数，CPU 可单测）。
pub fn reject_multi_sample(nsamples: usize) -> Result<(), LauncherError> {
    match nsamples {
        1 => Ok(()),
        _ => Err(LauncherError::Unsupported(
            "逐 world 只支持 nsamples=1（AA+per-world 留 L v2/K）",
        )),
    }
}

/// 逐 world 渲染一帧（方案 L v1）。
///
/// 前置：`context.resolved().output.nsamples == 1`；`scene_poses.worlds()` 等于 context 的
/// `nworld`；`scene_poses.leaves()` 等于每 world 的 leaf 数（各 world 的 leaf geom 次序一致）。
/// 成功后 context 的场景状态 == 调用方给定的 per-world 位姿；失败时尽力恢复调用前状态。
pub fn render_per_world(
    renderer: &MegakernelRenderer,
    context: &mut RenderContext,
    fields: &RenderFields,
    model: &CameraModel,
    scene_poses: &WorldScenePoses,
    camera_poses: &[CameraPose],
    rays: &RayBuffer,
) -> Result<(), LauncherError> {
    let resolved = context.resolved();
    reject_multi_sample(resolved.output.nsamples)?;
    let worlds = resolved.nworld;
    if scene_poses.worlds() != worlds {
        return Err(LauncherError::InvalidArray("scene_pose_worlds"));
    }
    let leaf_meta: Vec<(u32, i32)> = context
        .scene()
        .leaves()
        .iter()
        .map(|leaf| (leaf.group, leaf.geom_id))
        .collect();
    let slots = leaf_slots(&leaf_meta, worlds)?;
    if slots.len() / worlds != scene_poses.leaves() {
        return Err(LauncherError::InvalidArray("scene_pose_leaf_count"));
    }
    let original = context.scene().poses().to_vec();
    let mut outcome = Ok(());
    for world in 0..worlds {
        let replicated = replicated_poses(scene_poses, &slots, world);
        if let Err(error) = replicated.and_then(|poses| apply_poses(context, &poses)) {
            outcome = Err(error);
            break;
        }
        if let Err(error) = renderer.render_world(context, fields, model, camera_poses, rays, world)
        {
            outcome = Err(error);
            break;
        }
    }
    // 成功后把 context 同步为调用方给定的 per-world 位姿；失败则恢复调用前位姿（尽力而为）。
    let restore = match outcome.is_ok() {
        true => caller_poses(scene_poses, &slots, &leaf_meta),
        false => Ok(original),
    };
    if outcome.is_ok()
        && let Err(error) = restore.and_then(|poses| apply_poses(context, &poses))
    {
        outcome = Err(error);
    }
    outcome
}

/// 把 `leaf_meta` 压成「每个 leaf 在其 world 组内的槽位」；并校验各 world 的 leaf 数/geom 次序一致。
///
/// `leaf_meta[i] = (world 分组, MuJoCo geom_id)`。任一 world 缺 leaf、分组越界或同槽位 geom 不一致
/// 都返回 `InvalidArray`（可诊断，不 panic）。
fn leaf_slots(leaf_meta: &[(u32, i32)], worlds: usize) -> Result<Vec<usize>, LauncherError> {
    if worlds == 0 {
        return Err(LauncherError::InvalidArray("per_world_worlds"));
    }
    if leaf_meta.is_empty() || !leaf_meta.len().is_multiple_of(worlds) {
        return Err(LauncherError::InvalidArray("per_world_leaf_count"));
    }
    let per_world = leaf_meta.len() / worlds;
    let mut counts = vec![0usize; worlds];
    let mut slots = Vec::with_capacity(leaf_meta.len());
    let mut geoms: Vec<Option<i32>> = vec![None; per_world];
    for &(group, geom_id) in leaf_meta {
        let world = group as usize;
        if world >= worlds {
            return Err(LauncherError::InvalidArray("per_world_leaf_group"));
        }
        let slot = counts[world];
        if slot >= per_world {
            return Err(LauncherError::InvalidArray("per_world_leaf_count"));
        }
        match geoms[slot] {
            None => geoms[slot] = Some(geom_id),
            Some(expected) if expected != geom_id => {
                return Err(LauncherError::InvalidArray("per_world_leaf_geom_order"));
            }
            Some(_) => {}
        }
        slots.push(slot);
        counts[world] += 1;
    }
    if counts.iter().any(|&count| count != per_world) {
        return Err(LauncherError::InvalidArray("per_world_leaf_count"));
    }
    Ok(slots)
}

/// 单 world 发射用：把 `world` 的每个槽位位姿铺到全部 world 组（root[0] 即目标 world）。
fn replicated_poses(
    scene_poses: &WorldScenePoses,
    slots: &[usize],
    world: usize,
) -> Result<Vec<LeafPose>, LauncherError> {
    slots
        .iter()
        .map(|&slot| scene_pose(scene_poses, world, slot))
        .collect()
}

/// 同步 context 用：每个 leaf 取其所属 world 的位姿（world-major 输入口径）。
fn caller_poses(
    scene_poses: &WorldScenePoses,
    slots: &[usize],
    leaf_meta: &[(u32, i32)],
) -> Result<Vec<LeafPose>, LauncherError> {
    slots
        .iter()
        .zip(leaf_meta)
        .map(|(&slot, &(world, _))| scene_pose(scene_poses, world as usize, slot))
        .collect()
}

fn scene_pose(
    scene_poses: &WorldScenePoses,
    world: usize,
    slot: usize,
) -> Result<LeafPose, LauncherError> {
    let pose = scene_poses
        .get(world, slot)
        .map_err(|error: WorldError| LauncherError::Driver(format!("per-world poses: {error}")))?;
    Ok(LeafPose {
        xpos: pose.xpos,
        xmat: pose.xmat,
    })
}

fn apply_poses(context: &mut RenderContext, poses: &[LeafPose]) -> Result<(), LauncherError> {
    context
        .update_poses(poses)
        .map_err(|error| LauncherError::Driver(format!("per-world update_poses: {error}")))?;
    context
        .refit_bvh()
        .map_err(|error| LauncherError::Driver(format!("per-world refit_bvh: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::context::world::ScenePose;

    fn scene_pose(seed: f32) -> ScenePose {
        ScenePose {
            xpos: [seed, seed + 0.25, seed + 0.5],
            xmat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn rejects_multi_sample_per_world_entry() {
        assert!(reject_multi_sample(1).is_ok());
        for nsamples in [0, 2, 4] {
            assert!(matches!(
                reject_multi_sample(nsamples),
                Err(LauncherError::Unsupported(_))
            ));
        }
    }

    #[test]
    fn splits_leaf_slots_per_world_and_requires_equal_geom_order() {
        let meta = [(0, 7), (0, 9), (1, 7), (1, 9), (2, 7), (2, 9)];
        assert_eq!(leaf_slots(&meta, 3).unwrap(), vec![0, 1, 0, 1, 0, 1]);
        assert!(leaf_slots(&[], 3).is_err());
        assert!(leaf_slots(&meta[..4], 3).is_err());
        let bad_group = [(0, 7), (1, 7), (3, 7)];
        assert!(leaf_slots(&bad_group, 3).is_err());
        let bad_order = [(0, 7), (0, 9), (1, 9), (1, 7)];
        assert!(matches!(
            leaf_slots(&bad_order, 2),
            Err(LauncherError::InvalidArray("per_world_leaf_geom_order"))
        ));
    }

    #[test]
    fn replicates_target_world_poses_and_restores_caller_mapping() {
        let poses = WorldScenePoses::new(
            2,
            2,
            vec![
                scene_pose(0.0),
                scene_pose(1.0),
                scene_pose(10.0),
                scene_pose(11.0),
            ],
        )
        .unwrap();
        let slots = [0usize, 1];
        let replicated = replicated_poses(&poses, &slots, 1).unwrap();
        assert_eq!(replicated[0].xpos, [10.0, 10.25, 10.5]);
        assert_eq!(replicated[1].xpos, [11.0, 11.25, 11.5]);
        let caller = caller_poses(&poses, &slots, &[(0, 7), (1, 9)]).unwrap();
        assert_eq!(caller[0].xpos, [0.0, 0.25, 0.5]);
        assert_eq!(caller[1].xpos, [11.0, 11.25, 11.5]);
    }
}
