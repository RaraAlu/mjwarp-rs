//! per-world 场景 geom 位姿（方案 L v1：只覆盖 `geom_xpos`/`geom_xmat`）。
//!
//! 上游 `put_data` 后每个 world 一份 geom 位姿；本容器按 world-major 保存
//! `nworld × nleaf` 的 `xpos[3]` 与行主序 `xmat[9]`，供 per-world 发射逐 world 喂给
//! `RenderContext::update_poses`（`LeafPose` 的 f32 位型与上游 `geom_xpos/xmat` 一致）。
//!
//! 边界（v1 口径，Chief 2026-10-30 批）：灯位姿仍模型级单份，本容器不承载灯；不解析第二个模型。
//! 越界/计数不匹配在读取前返回 [`WorldError`]，不 panic。

use super::errors::{WorldError, check_world_range, checked_world_states};

/// 单个 `(world, leaf)` 的 geom 位姿（f32 位型与 `LeafPose` 一致）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScenePose {
    pub xpos: [f32; 3],
    /// 行主序 3x3（`geom_xmat` 语义）。
    pub xmat: [f32; 9],
}

/// `nworld × nleaf` 场景位姿；world-major（`world * nleaf + leaf`）。
#[derive(Clone, Debug, PartialEq)]
pub struct WorldScenePoses {
    worlds: usize,
    leaves: usize,
    poses: Vec<ScenePose>,
}

impl WorldScenePoses {
    /// 校验 `worlds >= 1` 且 `poses.len() == worlds × leaves`。
    pub fn new(worlds: usize, leaves: usize, poses: Vec<ScenePose>) -> Result<Self, WorldError> {
        let expected = checked_world_states(worlds, leaves)?;
        if poses.len() != expected {
            return Err(WorldError::SceneCountMismatch {
                expected,
                actual: poses.len(),
            });
        }
        Ok(Self {
            worlds,
            leaves,
            poses,
        })
    }

    pub fn worlds(&self) -> usize {
        self.worlds
    }

    pub fn leaves(&self) -> usize {
        self.leaves
    }

    /// `(world, leaf)` 的位姿（拷贝）。
    pub fn get(&self, world: usize, leaf: usize) -> Result<ScenePose, WorldError> {
        check_world_range(world, self.worlds)?;
        if leaf >= self.leaves {
            return Err(WorldError::SceneLeafOutOfRange {
                leaf,
                leaves: self.leaves,
            });
        }
        Ok(self.poses[self.index(world, leaf)?])
    }

    /// 某 world 的位姿切片（长度 = `leaves`）。
    pub fn world_slice(&self, world: usize) -> Result<&[ScenePose], WorldError> {
        check_world_range(world, self.worlds)?;
        let start = self.index(world, 0)?;
        let end = start
            .checked_add(self.leaves)
            .ok_or(WorldError::CountOverflow {
                field: "world_scene_poses",
            })?;
        Ok(&self.poses[start..end])
    }

    fn index(&self, world: usize, leaf: usize) -> Result<usize, WorldError> {
        world
            .checked_mul(self.leaves)
            .and_then(|base| base.checked_add(leaf))
            .ok_or(WorldError::CountOverflow {
                field: "world_scene_poses",
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(seed: f32) -> ScenePose {
        ScenePose {
            xpos: [seed, seed + 1.0, seed + 2.0],
            xmat: [seed, 0.0, 0.0, 0.0, seed, 0.0, 0.0, 0.0, seed],
        }
    }

    #[test]
    fn keeps_world_major_scene_poses_and_round_trips_bits() {
        let poses: Vec<ScenePose> = (0..12).map(|index| pose(index as f32 * 0.5)).collect();
        let world = WorldScenePoses::new(3, 4, poses.clone()).unwrap();
        assert_eq!(world.worlds(), 3);
        assert_eq!(world.leaves(), 4);
        for (index, expected) in poses.iter().enumerate() {
            let (w, leaf) = (index / 4, index % 4);
            let actual = world.get(w, leaf).unwrap();
            assert_eq!(
                actual.xpos.map(f32::to_bits),
                expected.xpos.map(f32::to_bits)
            );
            assert_eq!(
                actual.xmat.map(f32::to_bits),
                expected.xmat.map(f32::to_bits)
            );
        }
        assert_eq!(world.world_slice(2).unwrap().len(), 4);
    }

    #[test]
    fn rejects_count_mismatch_and_zero_worlds_without_panic() {
        assert!(matches!(
            WorldScenePoses::new(3, 4, vec![pose(0.0); 11]),
            Err(WorldError::SceneCountMismatch {
                expected: 12,
                actual: 11
            })
        ));
        assert!(WorldScenePoses::new(0, 4, Vec::new()).is_err());
        assert!(matches!(
            WorldScenePoses::new(2, 0, vec![pose(0.0); 3]),
            Err(WorldError::SceneCountMismatch { .. })
        ));
    }

    #[test]
    fn rejects_out_of_range_world_and_leaf() {
        let world = WorldScenePoses::new(2, 3, vec![pose(0.0); 6]).unwrap();
        assert!(matches!(
            world.get(2, 0),
            Err(WorldError::WorldOutOfRange {
                world: 2,
                worlds: 2
            })
        ));
        assert!(matches!(
            world.get(0, 3),
            Err(WorldError::SceneLeafOutOfRange { leaf: 3, leaves: 3 })
        ));
        assert!(world.world_slice(2).is_err());
    }
}
