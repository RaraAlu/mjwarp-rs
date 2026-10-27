//! R10-S1：多 world 渲染状态与布局（宿主侧，无 GPU、无窗口）。
//!
//! 冻结上游 `render_util.create_render_context`（`71da24d`）的状态带前导 `nworld` 维：
//! 相机位姿 `cam_xpos[nworld, ncam, 3]` / `cam_xmat[nworld, ncam, 9]`，输出缓冲形状
//! `[nworld, pixels]`（world-major）。本模块把宿主侧状态从单 world 泛化：**同一个模型**复制到
//! `nworld` 个 world（多 state），world 维只增加状态副本、**不解析第二个模型**；每 world 的
//! 输出区段由既有 [`OutputLayout`] 按 `nworld` 乘法展开、互不重叠。
//!
//! 本片只做宿主侧：per-world 相机位姿、分辨率表、模型/场景状态容器与输出区段；world 越界/
//! 未激活/状态缺失/相机数或分辨率与 `nworld` 不匹配/`nworld == 0` 都在读写前返回
//! [`WorldError`]，不 panic。内核/设备源不变（R10-S2 才接 GPU）。

use std::ops::Range;

use crate::render::ImageOutput;
use crate::render::camera::CameraPlan;

use super::{ContextError, OutputLayout};

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

fn check_world_range(world: usize, worlds: usize) -> Result<(), WorldError> {
    if world >= worlds {
        return Err(WorldError::WorldOutOfRange { world, worlds });
    }
    Ok(())
}

fn check_camera_range(camera: usize, cameras: usize) -> Result<(), WorldError> {
    if camera >= cameras {
        return Err(WorldError::CameraOutOfRange { camera, cameras });
    }
    Ok(())
}

fn checked_world_states(worlds: usize, cameras: usize) -> Result<usize, WorldError> {
    if worlds == 0 {
        return Err(WorldError::WorldCountZero);
    }
    worlds
        .checked_mul(cameras)
        .ok_or(WorldError::CountOverflow {
            field: "world_states",
        })
}

fn pixels_of(resolution: [u32; 2]) -> Result<usize, WorldError> {
    (resolution[0] as usize)
        .checked_mul(resolution[1] as usize)
        .ok_or(WorldError::CountOverflow {
            field: "camera_pixels",
        })
}

fn checked_index(world: usize, camera: usize, cameras: usize) -> Result<usize, WorldError> {
    world
        .checked_mul(cameras)
        .and_then(|base| base.checked_add(camera))
        .ok_or(WorldError::CountOverflow {
            field: "world_state_index",
        })
}

/// 单个 `(world, camera)` 相机位姿（f32 位型与内核 `CameraPose` 的 `xpos`/`xmat` 一致）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPose {
    pub xpos: [f32; 3],
    /// 行主序 `cam_xmat`。
    pub xmat: [f32; 9],
}

/// `nworld × ncam` 相机位姿；world-major（`world * ncam + camera`）。
///
/// 双 world 语义 = **同模型多 state**：相机下标在所有 world 间一致，各 world 可持不同位姿。
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
    /// 全部 world 激活、状态全为空。
    pub fn new(worlds: usize) -> Result<Self, WorldError> {
        Self::with_active(worlds, vec![true; worlds])
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
        Ok(Self {
            active,
            states: (0..worlds).map(|_| None).collect(),
        })
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

/// 多 world 输出区段：在既有 [`OutputLayout`] 上按 `nworld` 乘法展开（world-major）。
///
/// 区段为 `[world * pixels, (world + 1) * pixels)`，各 world 互不重叠；`nworld == 1` 时
/// 与单 world 布局逐位一致（`0..pixels`）。
#[derive(Clone, Debug, PartialEq)]
pub struct WorldLayout {
    plan: CameraPlan,
    layout: OutputLayout,
    worlds: usize,
    active: Vec<bool>,
}

impl WorldLayout {
    /// 按 `plan` 解析单 world 布局后展开到 `nworld` 个 world（全部激活）。
    pub fn resolve(plan: &CameraPlan, worlds: usize) -> Result<Self, WorldError> {
        let layout = OutputLayout::resolve(plan)?;
        if worlds == 0 {
            return Err(WorldError::WorldCountZero);
        }
        Ok(Self {
            plan: plan.clone(),
            layout,
            worlds,
            active: vec![true; worlds],
        })
    }

    /// 设置激活掩码（长度必须等于 `worlds`）。
    pub fn with_active(mut self, active: Vec<bool>) -> Result<Self, WorldError> {
        if active.len() != self.worlds {
            return Err(WorldError::ActiveMaskMismatch {
                expected: self.worlds,
                actual: active.len(),
            });
        }
        self.active = active;
        Ok(self)
    }

    pub fn worlds(&self) -> usize {
        self.worlds
    }

    pub fn cameras(&self) -> usize {
        self.plan.resolutions.len()
    }

    pub fn layout(&self) -> &OutputLayout {
        &self.layout
    }

    pub fn is_active(&self, world: usize) -> Result<bool, WorldError> {
        check_world_range(world, self.worlds)?;
        Ok(self.active[world])
    }

    /// 单 world 的通道元素数（`OutputLayout` 字段）。
    pub fn channel_pixels(&self, channel: ImageOutput) -> usize {
        match channel {
            ImageOutput::Rgb => self.layout.rgb_pixels,
            ImageOutput::Depth => self.layout.depth_pixels,
            ImageOutput::Segmentation => self.layout.seg_pixels,
        }
    }

    /// 整缓冲元素数（`nworld × 单 world 像素数`）。
    pub fn buffer_elements(&self, channel: ImageOutput) -> Result<usize, WorldError> {
        self.worlds
            .checked_mul(self.channel_pixels(channel))
            .ok_or(WorldError::CountOverflow {
                field: "world_buffer_elements",
            })
    }

    fn check_world(&self, world: usize) -> Result<(), WorldError> {
        check_world_range(world, self.worlds)?;
        if !self.active[world] {
            return Err(WorldError::WorldInactive { world });
        }
        Ok(())
    }

    /// 某 world 在该通道的起始元素下标（world-major）。
    pub fn world_offset(&self, channel: ImageOutput, world: usize) -> Result<usize, WorldError> {
        self.check_world(world)?;
        world
            .checked_mul(self.channel_pixels(channel))
            .ok_or(WorldError::CountOverflow {
                field: "world_offset",
            })
    }

    /// 某 world 在该通道的元素区段（长度 = 单 world 像素数）。
    pub fn world_range(
        &self,
        channel: ImageOutput,
        world: usize,
    ) -> Result<Range<usize>, WorldError> {
        let start = self.world_offset(channel, world)?;
        let end =
            start
                .checked_add(self.channel_pixels(channel))
                .ok_or(WorldError::CountOverflow {
                    field: "world_range",
                })?;
        Ok(start..end)
    }

    /// 从整缓冲切出某 world 的区段（校验缓冲长度；不复制、不越界读邻 world）。
    pub fn world_view<'a, T>(
        &self,
        channel: ImageOutput,
        world: usize,
        data: &'a [T],
    ) -> Result<&'a [T], WorldError> {
        let range = self.world_range(channel, world)?;
        if range.end > data.len() {
            return Err(WorldError::BufferRange {
                world,
                end: range.end,
                actual: data.len(),
            });
        }
        Ok(&data[range])
    }

    /// 某 world 内某相机在该通道的元素区段（相机区段在 world 偏移之后）。
    pub fn camera_range(
        &self,
        channel: ImageOutput,
        world: usize,
        camera: usize,
    ) -> Result<Range<usize>, WorldError> {
        let base = self.world_offset(channel, world)?;
        check_camera_range(camera, self.cameras())?;
        let address = match channel {
            ImageOutput::Rgb => self.layout.rgb_adr[camera],
            ImageOutput::Depth => self.layout.depth_adr[camera],
            ImageOutput::Segmentation => self.layout.seg_adr[camera],
        };
        if address < 0 {
            return Err(WorldError::ChannelDisabled {
                channel,
                camera_index: camera,
            });
        }
        let start = base
            .checked_add(address as usize)
            .ok_or(WorldError::CountOverflow {
                field: "camera_range",
            })?;
        let end = start
            .checked_add(pixels_of(self.plan.resolutions[camera])?)
            .ok_or(WorldError::CountOverflow {
                field: "camera_range",
            })?;
        Ok(start..end)
    }
}
