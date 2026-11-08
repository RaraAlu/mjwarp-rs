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
//! - L v2（spec v3 §10.3）：[`render_per_world_with_lights`] 在每次 world 发射前把该 world 的
//!   灯位姿（[`WorldLightPoses`]，f32 位型 = 采集侧 `state.npz` 原值）逐位透传到内核
//!   `light_xpos_in`/`light_xdir_in`；内核灯数组仍 1-D、`.cu`/81 槽零改动。
//! - AA：`nsamples > 1` 显式拒绝（`LauncherError::Unsupported`），不静默按单样本渲染。

use crate::model::render::RenderFields;
use crate::render::camera::CameraModel;
use crate::render::context::world::{WorldError, WorldScenePoses};
use crate::render::context::{LeafPose, RenderContext};
use crate::render::kernel::cast::CameraPose;
use crate::render::rays::RayBuffer;
use crate::runtime::launcher::LauncherError;

use super::megakernel::MegakernelRenderer;
use super::world_light::WorldLightPoses;

/// AA / `spp>1` 守卫：per-world 入口只接受 `nsamples == 1`（纯函数，CPU 可单测）。
pub fn reject_multi_sample(nsamples: usize) -> Result<(), LauncherError> {
    match nsamples {
        1 => Ok(()),
        _ => Err(LauncherError::Unsupported(
            "逐 world 只支持 nsamples=1（AA+per-world 留 L v2/K）",
        )),
    }
}

/// 逐 world 渲染一帧（方案 L v1：只注入 per-world geom 位姿；灯仍模型级单份）。
///
/// 前置：`context.resolved().output.nsamples == 1`；`scene_poses.worlds()` 等于 context 的
/// `nworld`；`scene_poses.leaves()` 等于每 world 的 leaf 数（各 world 的 leaf geom 次序一致）。
/// 成功后 context 的场景状态 == 调用方给定的 per-world 位姿；失败时恢复调用前状态
/// （恢复失败只作附注，首错原样返回）。
pub fn render_per_world(
    renderer: &MegakernelRenderer,
    context: &mut RenderContext,
    fields: &RenderFields,
    model: &CameraModel,
    scene_poses: &WorldScenePoses,
    camera_poses: &[CameraPose],
    rays: &RayBuffer,
) -> Result<(), LauncherError> {
    render_per_world_impl(
        renderer,
        context,
        fields,
        model,
        scene_poses,
        None,
        camera_poses,
        rays,
    )
}

/// 逐 world 渲染一帧 + L v2 **per-world 灯位姿**（spec v3 §10.3；`.cu`/81 槽零改动）。
///
/// 与 [`render_per_world`] 的唯一差别：每次 world 发射前把 `light_poses.world_slice(world)`
/// 的 [`super::megakernel_arrays::LightDataPose`]（`xpos`/`xdir` f32 位型 = 采集侧
/// `state.npz` 的 `light_xpos`/`light_xdir` 原值；不 FK 推导、不 f64 重算、`xdir` 不重归一化）
/// 重传到内核 `light_xpos_in`/`light_xdir_in` 两个 1-D 实参。
///
/// 灯数必须等于 `fields.light_type` 长度，且 `light_poses.worlds()` 必须等于 context 的
/// `nworld`（否则 `InvalidArray`，不 panic）；其余前置/失败恢复语义与 [`render_per_world`] 相同。
// 8 参数：per-world 灯位姿必须显式传入（同 `render_with_flex` 的 8 参数先例）。
#[allow(clippy::too_many_arguments)]
pub fn render_per_world_with_lights(
    renderer: &MegakernelRenderer,
    context: &mut RenderContext,
    fields: &RenderFields,
    model: &CameraModel,
    scene_poses: &WorldScenePoses,
    light_poses: &WorldLightPoses,
    camera_poses: &[CameraPose],
    rays: &RayBuffer,
) -> Result<(), LauncherError> {
    render_per_world_impl(
        renderer,
        context,
        fields,
        model,
        scene_poses,
        Some(light_poses),
        camera_poses,
        rays,
    )
}

/// 两个 per-world 入口的共享实现：`lights = None` 即 v1（geom-only）路径，行为逐字节不变。
#[allow(clippy::too_many_arguments)]
fn render_per_world_impl(
    renderer: &MegakernelRenderer,
    context: &mut RenderContext,
    fields: &RenderFields,
    model: &CameraModel,
    scene_poses: &WorldScenePoses,
    lights: Option<&WorldLightPoses>,
    camera_poses: &[CameraPose],
    rays: &RayBuffer,
) -> Result<(), LauncherError> {
    let resolved = context.resolved();
    reject_multi_sample(resolved.output.nsamples)?;
    let worlds = resolved.nworld;
    if scene_poses.worlds() != worlds {
        return Err(LauncherError::InvalidArray("scene_pose_worlds"));
    }
    if let Some(lights) = lights {
        validate_light_worlds(lights, worlds)?;
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
        let rendered = match lights {
            Some(lights) => renderer.render_world_with_lights(
                context,
                fields,
                model,
                camera_poses,
                rays,
                world,
                lights,
            ),
            None => renderer.render_world(context, fields, model, camera_poses, rays, world),
        };
        if let Err(error) = rendered {
            outcome = Err(error);
            break;
        }
    }
    // 收尾：成功 → 把 context 同步为调用方给定的 per-world 位姿；失败 → 恢复调用前位姿。
    // 两条路径都会 apply（失败路径不再跳过恢复），策略见 `finish_per_world`。
    let restore = match outcome.is_ok() {
        true => caller_poses(scene_poses, &slots, &leaf_meta),
        false => Ok(original),
    };
    finish_per_world(outcome, restore, |poses| apply_poses(context, poses))
}

/// per-world 发射的收尾策略（纯函数，CPU 单测覆盖；真实施加由 `apply` 回调完成）。
///
/// - 成功路径（`outcome` 为 `Ok`）：施加调用方给定位姿；`restore`/`apply` 的失败即首错，返回它。
/// - 失败路径（`outcome` 为 `Err`）：**仍施加恢复位姿**（调用前状态）；恢复阶段的错误只作附注
///   （本 crate 无集中日志设施）——**首错原样返回**，不 panic、不把恢复错误升为首错。
fn finish_per_world(
    outcome: Result<(), LauncherError>,
    restore: Result<Vec<LeafPose>, LauncherError>,
    apply: impl FnOnce(&[LeafPose]) -> Result<(), LauncherError>,
) -> Result<(), LauncherError> {
    let restore_result = restore.and_then(|poses| apply(&poses));
    match (outcome.is_ok(), restore_result) {
        (true, Err(error)) => Err(error),
        (true, Ok(())) => Ok(()),
        // 失败路径：恢复适用性错误/恢复失败只作附注，首错原样返回。
        (false, _) => outcome,
    }
}

/// per-world 灯入口的 world 数校验（纯函数，CPU 单测覆盖；不发 GPU）。
fn validate_light_worlds(lights: &WorldLightPoses, worlds: usize) -> Result<(), LauncherError> {
    if lights.worlds() != worlds {
        return Err(LauncherError::InvalidArray("per_world_light_worlds"));
    }
    Ok(())
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

    fn light_pose(seed: f32) -> super::super::megakernel_arrays::LightDataPose {
        super::super::megakernel_arrays::LightDataPose {
            xpos: [seed, seed + 1.0, seed + 2.0],
            xdir: [0.0, 0.0, -1.0],
        }
    }

    #[test]
    fn validates_per_world_light_world_count_without_panic() {
        let lights = WorldLightPoses::new(3, 3, vec![light_pose(0.0); 9]).unwrap();
        assert!(matches!(
            validate_light_worlds(&lights, 2),
            Err(LauncherError::InvalidArray("per_world_light_worlds"))
        ));
        assert!(validate_light_worlds(&lights, 3).is_ok());
    }

    fn leaf(seed: f32) -> LeafPose {
        LeafPose {
            xpos: [seed, seed + 1.0, seed + 2.0],
            xmat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }

    // QA 中观察 O1：失败路径必须真的恢复（此前 `outcome.is_ok()` 门控导致恢复从不施加）。
    #[test]
    fn failure_path_restores_original_and_keeps_first_error() {
        let original = vec![leaf(0.0), leaf(1.0)];
        let first = LauncherError::Unsupported("per-world 首错");
        let mut scene = original.clone();
        let mut applied = 0usize;
        let result = finish_per_world(Err(first.clone()), Ok(original.clone()), |poses| {
            applied += 1;
            scene = poses.to_vec();
            Ok(())
        });
        assert_eq!(result, Err(first), "失败路径必须原样返回首错");
        assert_eq!(applied, 1, "失败路径必须执行恢复 apply（不得跳过）");
        assert_eq!(scene, original, "失败路径必须回到调用前位姿");
    }

    #[test]
    fn failure_path_restore_error_does_not_mask_first_error() {
        let first = LauncherError::InvalidArray("per_world_light_count");
        let mut applied = 0usize;
        let result = finish_per_world(Err(first.clone()), Ok(vec![leaf(0.0)]), |_| {
            applied += 1;
            Err(LauncherError::Driver("restore failed".into()))
        });
        assert_eq!(result, Err(first), "恢复失败只作附注，不得掩盖首错");
        assert_eq!(applied, 1, "即使恢复失败也必须尝试恢复");
    }

    #[test]
    fn success_path_applies_caller_poses_unchanged() {
        let caller = vec![leaf(5.0), leaf(6.0)];
        let mut scene = vec![leaf(-1.0)];
        let result = finish_per_world(Ok(()), Ok(caller.clone()), |poses| {
            scene = poses.to_vec();
            Ok(())
        });
        assert_eq!(result, Ok(()));
        assert_eq!(scene, caller, "成功路径必须同步调用方位姿");
    }

    #[test]
    fn success_path_apply_error_is_returned_as_first_error() {
        let error = LauncherError::Driver("sync failed".into());
        let result = finish_per_world(Ok(()), Ok(vec![leaf(0.0)]), |_| Err(error.clone()));
        assert_eq!(result, Err(error), "成功路径施加失败即首错");
    }
}
