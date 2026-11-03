//! 多 world 错误类型与共享校验助手（R10-S1 观察收口拆分）。

use crate::render::ImageOutput;

use super::super::ContextError;

/// 多 world 状态/布局错误；全部在读取或分配前给出。
#[derive(Clone, Debug, PartialEq)]
pub enum WorldError {
    WorldCountZero,
    WorldOutOfRange {
        world: usize,
        worlds: usize,
    },
    WorldInactive {
        world: usize,
    },
    StateMissing {
        world: usize,
    },
    ActiveMaskMismatch {
        expected: usize,
        actual: usize,
    },
    /// per-world 状态条目数不等于 `nworld × ncam`。
    CameraCountMismatch {
        expected: usize,
        actual: usize,
    },
    CameraOutOfRange {
        camera: usize,
        cameras: usize,
    },
    /// per-world 场景位姿条目数不等于 `nworld × nleaf`。
    SceneCountMismatch {
        expected: usize,
        actual: usize,
    },
    /// 场景 leaf 下标越界。
    SceneLeafOutOfRange {
        leaf: usize,
        leaves: usize,
    },
    /// 某 world 某相机的分辨率与 `plan.resolutions` 不一致。
    ResolutionMismatch {
        world: usize,
        camera: usize,
        expected: [u32; 2],
        actual: [u32; 2],
    },
    /// 该相机该通道未启用（`*_adr[camera_index] == -1`）。
    ChannelDisabled {
        channel: ImageOutput,
        camera_index: usize,
    },
    BufferRange {
        world: usize,
        end: usize,
        actual: usize,
    },
    CountOverflow {
        field: &'static str,
    },
    Layout(ContextError),
}

impl From<ContextError> for WorldError {
    fn from(error: ContextError) -> Self {
        Self::Layout(error)
    }
}

impl std::error::Error for WorldError {}

impl std::fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorldCountZero => write!(formatter, "nworld 必须 >= 1"),
            Self::WorldOutOfRange { world, worlds } => {
                write!(formatter, "world {world} 越界（worlds={worlds}）")
            }
            Self::WorldInactive { world } => write!(formatter, "world {world} 未激活"),
            Self::StateMissing { world } => write!(formatter, "world {world} 状态缺失"),
            Self::ActiveMaskMismatch { expected, actual } => {
                write!(formatter, "激活掩码长度 {actual} != nworld {expected}")
            }
            Self::CameraCountMismatch { expected, actual } => write!(
                formatter,
                "per-world 状态条目数 {actual} != nworld×ncam {expected}"
            ),
            Self::CameraOutOfRange { camera, cameras } => {
                write!(formatter, "相机下标 {camera} 越界（cameras={cameras}）")
            }
            Self::SceneCountMismatch { expected, actual } => write!(
                formatter,
                "per-world 场景位姿条目数 {actual} != nworld×nleaf {expected}"
            ),
            Self::SceneLeafOutOfRange { leaf, leaves } => {
                write!(formatter, "场景 leaf 下标 {leaf} 越界（leaves={leaves}）")
            }
            Self::ResolutionMismatch {
                world,
                camera,
                expected,
                actual,
            } => write!(
                formatter,
                "world {world} 相机 {camera} 分辨率 {actual:?} != plan {expected:?}"
            ),
            Self::ChannelDisabled {
                channel,
                camera_index,
            } => write!(
                formatter,
                "相机 {camera_index} 的 {} 通道未启用",
                channel_name(*channel)
            ),
            Self::BufferRange { world, end, actual } => {
                write!(formatter, "world {world} 区段末端 {end} 超出缓冲 {actual}")
            }
            Self::CountOverflow { field } => write!(formatter, "{field} 计数溢出"),
            Self::Layout(error) => write!(formatter, "{error}"),
        }
    }
}

fn channel_name(channel: ImageOutput) -> &'static str {
    match channel {
        ImageOutput::Rgb => "rgb",
        ImageOutput::Depth => "depth",
        ImageOutput::Segmentation => "segmentation",
    }
}

pub(super) fn check_world_range(world: usize, worlds: usize) -> Result<(), WorldError> {
    if world >= worlds {
        return Err(WorldError::WorldOutOfRange { world, worlds });
    }
    Ok(())
}

pub(super) fn check_camera_range(camera: usize, cameras: usize) -> Result<(), WorldError> {
    if camera >= cameras {
        return Err(WorldError::CameraOutOfRange { camera, cameras });
    }
    Ok(())
}

pub(super) fn checked_world_states(worlds: usize, cameras: usize) -> Result<usize, WorldError> {
    if worlds == 0 {
        return Err(WorldError::WorldCountZero);
    }
    worlds
        .checked_mul(cameras)
        .ok_or(WorldError::CountOverflow {
            field: "world_states",
        })
}

pub(super) fn checked_index(
    world: usize,
    camera: usize,
    cameras: usize,
) -> Result<usize, WorldError> {
    world
        .checked_mul(cameras)
        .and_then(|base| base.checked_add(camera))
        .ok_or(WorldError::CountOverflow {
            field: "world_state_index",
        })
}

pub(super) fn pixels_of(resolution: [u32; 2]) -> Result<usize, WorldError> {
    (resolution[0] as usize)
        .checked_mul(resolution[1] as usize)
        .ok_or(WorldError::CountOverflow {
            field: "camera_pixels",
        })
}

/// 全激活掩码；极大 `worlds` 走 `try_reserve_exact` 返回 `CountOverflow`（不 capacity-overflow panic）。
pub(super) fn active_mask_all_true(worlds: usize) -> Result<Vec<bool>, WorldError> {
    let mut mask = Vec::new();
    mask.try_reserve_exact(worlds)
        .map_err(|_| WorldError::CountOverflow { field: "worlds" })?;
    mask.resize(worlds, true);
    Ok(mask)
}
