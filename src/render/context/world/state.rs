//! per-world 相机位姿、分辨率表与状态容器（R10-S1 观察收口拆分）。
//!
//! 均为「同模型多 state」语义：世界维只增加状态副本，不解析第二个模型。

use crate::render::camera::CameraPlan;

use super::errors::{
    WorldError, active_mask_all_true, check_camera_range, check_world_range, checked_index,
    checked_world_states,
};

/// 单个 `(world, camera)` 相机位姿（f32 位型与内核 `CameraPose` 的 `xpos`/`xmat` 一致）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPose {
    pub xpos: [f32; 3],
    /// 行主序 `cam_xmat`。
    pub xmat: [f32; 9],
}

/// `nworld × ncam` 相机位姿；world-major（`world * ncam + camera`）。
#[derive(Clone, Debug, PartialEq)]
pub struct WorldCameraPoses {
    worlds: usize,
    cameras: usize,
    poses: Vec<CameraPose>,
}

impl WorldCameraPoses {
    /// 校验 `worlds >= 1` 且 `poses.len() == worlds × cameras`。
    pub fn new(worlds: usize, cameras: usize, poses: Vec<CameraPose>) -> Result<Self, WorldError> {
        let expected = checked_world_states(worlds, cameras)?;
        if poses.len() != expected {
            return Err(WorldError::CameraCountMismatch {
                expected,
                actual: poses.len(),
            });
        }
        Ok(Self {
            worlds,
            cameras,
            poses,
        })
    }

    /// `(world, camera)` 的位姿（拷贝）。
    pub fn get(&self, world: usize, camera: usize) -> Result<CameraPose, WorldError> {
        check_world_range(world, self.worlds)?;
        check_camera_range(camera, self.cameras)?;
        let index = checked_index(world, camera, self.cameras)?;
        Ok(self.poses[index])
    }

    /// 某 world 的相机位姿切片（长度 = `ncam`；`ncam == 0` 时为空切片）。
    pub fn world_slice(&self, world: usize) -> Result<&[CameraPose], WorldError> {
        check_world_range(world, self.worlds)?;
        let start = checked_index(world, 0, self.cameras)?;
        let end = start
            .checked_add(self.cameras)
            .ok_or(WorldError::CountOverflow {
                field: "world_camera_poses",
            })?;
        Ok(&self.poses[start..end])
    }
}

/// per-world 相机分辨率表（`nworld × ncam`）；同模型多 state 时逐 world 须与 `plan` 一致。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldResolutions {
    worlds: usize,
    cameras: usize,
    resolutions: Vec<[u32; 2]>,
}

impl WorldResolutions {
    /// 校验外部输入的 `nworld × ncam` 分辨率表逐项等于 `plan.resolutions`。
    ///
    /// 空相机选择（`cameras == 0`）直接返回空表：只接受空输入，不做 `chunks(0)`（拆分后的 F1 修复）。
    pub fn new(
        plan: &CameraPlan,
        worlds: usize,
        resolutions: Vec<[u32; 2]>,
    ) -> Result<Self, WorldError> {
        let expected = checked_world_states(worlds, plan.resolutions.len())?;
        if resolutions.len() != expected {
            return Err(WorldError::CameraCountMismatch {
                expected,
                actual: resolutions.len(),
            });
        }
        if plan.resolutions.is_empty() {
            return Ok(Self {
                worlds,
                cameras: 0,
                resolutions,
            });
        }
        for (world, chunk) in resolutions.chunks(plan.resolutions.len()).enumerate() {
            for (camera, actual) in chunk.iter().enumerate() {
                let expected = plan.resolutions[camera];
                if *actual != expected {
                    return Err(WorldError::ResolutionMismatch {
                        world,
                        camera,
                        expected,
                        actual: *actual,
                    });
                }
            }
        }
        Ok(Self {
            worlds,
            cameras: plan.resolutions.len(),
            resolutions,
        })
    }

    pub fn get(&self, world: usize, camera: usize) -> Result<[u32; 2], WorldError> {
        check_world_range(world, self.worlds)?;
        check_camera_range(camera, self.cameras)?;
        Ok(self.resolutions[checked_index(world, camera, self.cameras)?])
    }
}

/// per-world 模型/场景状态容器（同模型多 state：每 world 一份状态，可缺省）。
///
/// 读取顺序固定：world 下标 → 激活位 → 状态是否存在；任何一步不满足都返回可诊断错误。
#[derive(Clone, Debug, PartialEq)]
pub struct WorldStates<T> {
    active: Vec<bool>,
    states: Vec<Option<T>>,
}

impl<T> WorldStates<T> {
    /// 全部 world 激活、状态全为空；极大 `worlds` 返回 `CountOverflow`（不 panic）。
    pub fn new(worlds: usize) -> Result<Self, WorldError> {
        if worlds == 0 {
            return Err(WorldError::WorldCountZero);
        }
        let active = active_mask_all_true(worlds)?;
        Self::assemble(active)
    }

    /// 指定激活掩码（长度必须等于 `worlds`）。
    pub fn with_active(worlds: usize, active: Vec<bool>) -> Result<Self, WorldError> {
        if worlds == 0 {
            return Err(WorldError::WorldCountZero);
        }
        if active.len() != worlds {
            return Err(WorldError::ActiveMaskMismatch {
                expected: worlds,
                actual: active.len(),
            });
        }
        Self::assemble(active)
    }

    fn assemble(active: Vec<bool>) -> Result<Self, WorldError> {
        let worlds = active.len();
        if worlds == 0 {
            return Err(WorldError::WorldCountZero);
        }
        let mut states = Vec::new();
        states
            .try_reserve_exact(worlds)
            .map_err(|_| WorldError::CountOverflow { field: "worlds" })?;
        states.resize_with(worlds, || None);
        Ok(Self { active, states })
    }

    pub fn worlds(&self) -> usize {
        self.states.len()
    }

    pub fn set_active(&mut self, world: usize, active: bool) -> Result<(), WorldError> {
        check_world_range(world, self.worlds())?;
        self.active[world] = active;
        Ok(())
    }

    /// 写入某 world 的状态（world 须存在且激活）。
    pub fn insert(&mut self, world: usize, state: T) -> Result<Option<T>, WorldError> {
        check_world_range(world, self.worlds())?;
        if !self.active[world] {
            return Err(WorldError::WorldInactive { world });
        }
        Ok(self.states[world].replace(state))
    }

    /// 读取某 world 的状态（越界/未激活/缺失分别报错）。
    pub fn state(&self, world: usize) -> Result<&T, WorldError> {
        check_world_range(world, self.worlds())?;
        if !self.active[world] {
            return Err(WorldError::WorldInactive { world });
        }
        self.states[world]
            .as_ref()
            .ok_or(WorldError::StateMissing { world })
    }
}
