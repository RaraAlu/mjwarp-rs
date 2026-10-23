//! M08 局部重置阶段切片的宿主侧实现。
//!
//! 对齐冻结上游 `mujoco_warp.reset_data`（U035）与
//! `mujoco_warp.reset_data_keyframe`（U036）在 P3 阶段的最小语义：
//! 世界掩码隔离（掩码内世界 `time=0`、`qpos=qpos0`、`qvel=0`）与
//! 关键帧重置（全局整数或逐世界整数数组；越界世界不重置）。
//!
//! 本片只提供世界批量状态 `time/qpos/qvel`，不实现完整 Data、
//! 不发射 GPU 内核、不接设备缓冲。
//!
//! 未验收边界：`act`、`ctrl`、`qfrc_applied`、`xfrc_applied`、
//! `mocap_pos/quat`、接触/约束/能量/睡眠字段与 G24 历史
//! （`reset_history`）均未包含；动态时钟以外的历史与传感器
//! 语义未实现。

use std::error::Error;
use std::fmt;

/// 重置操作的输入错误；对齐上游 `ValueError` 的校验语义。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResetError {
    /// 维度为 0。
    InvalidDimension { field: &'static str },
    /// 字段长度与维度不一致。
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 全局关键帧索引越界（上游对整数 key 抛 `ValueError`）。
    KeyframeOutOfRange { key: i32, nkey: usize },
}

impl fmt::Display for ResetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimension { field } => write!(formatter, "维度必须为正：{field}"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "长度不匹配：{field} 期望 {expected}，实际 {actual}"
            ),
            Self::KeyframeOutOfRange { key, nkey } => {
                write!(formatter, "关键帧索引越界：{key} 不在 [0, {nkey})")
            }
        }
    }
}

impl Error for ResetError {}

/// 最小世界批量状态：`time`（每世界）、`qpos` 与 `qvel`（世界×分量，f32）。
#[derive(Clone, Debug, PartialEq)]
pub struct WorldState {
    nworld: usize,
    nq: usize,
    nv: usize,
    time: Vec<f32>,
    qpos: Vec<f32>,
    qvel: Vec<f32>,
}

impl WorldState {
    /// 新建全零状态。
    pub fn new(nworld: usize, nq: usize, nv: usize) -> Result<Self, ResetError> {
        Self::from_batches(
            nworld,
            nq,
            nv,
            &vec![0.0; nworld],
            &vec![0.0; nworld * nq],
            &vec![0.0; nworld * nv],
        )
    }

    /// 由已有批量数组构造状态，逐字段校验长度。
    pub fn from_batches(
        nworld: usize,
        nq: usize,
        nv: usize,
        time: &[f32],
        qpos: &[f32],
        qvel: &[f32],
    ) -> Result<Self, ResetError> {
        if nworld == 0 {
            return Err(ResetError::InvalidDimension { field: "nworld" });
        }
        let expected_qpos = nworld
            .checked_mul(nq)
            .ok_or(ResetError::InvalidDimension { field: "nq" })?;
        let expected_qvel = nworld
            .checked_mul(nv)
            .ok_or(ResetError::InvalidDimension { field: "nv" })?;
        for (field, actual, expected) in [
            ("time", time.len(), nworld),
            ("qpos", qpos.len(), expected_qpos),
            ("qvel", qvel.len(), expected_qvel),
        ] {
            if actual != expected {
                return Err(ResetError::LengthMismatch {
                    field,
                    expected,
                    actual,
                });
            }
        }
        Ok(Self {
            nworld,
            nq,
            nv,
            time: time.to_vec(),
            qpos: qpos.to_vec(),
            qvel: qvel.to_vec(),
        })
    }

    pub fn nworld(&self) -> usize {
        self.nworld
    }

    pub fn nq(&self) -> usize {
        self.nq
    }

    pub fn nv(&self) -> usize {
        self.nv
    }

    pub fn time(&self) -> &[f32] {
        &self.time
    }

    pub fn qpos(&self) -> &[f32] {
        &self.qpos
    }

    pub fn qvel(&self) -> &[f32] {
        &self.qvel
    }
}

/// 世界掩码：`All` 重置全部世界；`Worlds` 按布尔重置；
/// `Flags` 按整数非零重置（对齐上游布尔/整数掩码两种 dtype）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetMask<'a> {
    All,
    Worlds(&'a [bool]),
    Flags(&'a [i32]),
}

impl ResetMask<'_> {
    fn resets(&self, world: usize) -> Result<bool, ResetError> {
        Ok(match self {
            Self::All => true,
            Self::Worlds(values) => values[world],
            Self::Flags(values) => values[world] != 0,
        })
    }
}

/// 关键帧选择：`All(key)` 重置全部世界到同一关键帧；
/// `Worlds(keys)` 按世界选择，越界或负索引的世界不重置。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyframeSelection<'a> {
    All(i32),
    Worlds(&'a [i32]),
}

/// 关键帧模型字段（`nkey` 行；qpos/qvel 按行展开）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframes<'a> {
    nkey: usize,
    time: &'a [f32],
    qpos: &'a [f32],
    qvel: &'a [f32],
}

impl<'a> Keyframes<'a> {
    /// 登记关键帧数组；qpos/qvel 的长度在重置时按状态维度校验。
    pub fn new(
        nkey: usize,
        time: &'a [f32],
        qpos: &'a [f32],
        qvel: &'a [f32],
    ) -> Result<Self, ResetError> {
        if time.len() != nkey {
            return Err(ResetError::LengthMismatch {
                field: "key_time",
                expected: nkey,
                actual: time.len(),
            });
        }
        Ok(Self {
            nkey,
            time,
            qpos,
            qvel,
        })
    }
}

/// 世界掩码重置：掩码内世界 `time=0`、`qpos=qpos0`、`qvel=0`。
pub fn reset_data(
    state: &mut WorldState,
    qpos0: &[f32],
    mask: ResetMask<'_>,
) -> Result<(), ResetError> {
    if qpos0.len() != state.nq {
        return Err(ResetError::LengthMismatch {
            field: "qpos0",
            expected: state.nq,
            actual: qpos0.len(),
        });
    }
    let nworld = state.nworld;
    check_mask_length(mask, nworld)?;
    for world in 0..nworld {
        if !mask.resets(world)? {
            continue;
        }
        state.time[world] = 0.0;
        let qpos = &mut state.qpos[world * state.nq..(world + 1) * state.nq];
        qpos.copy_from_slice(qpos0);
        let qvel = &mut state.qvel[world * state.nv..(world + 1) * state.nv];
        qvel.fill(0.0);
    }
    Ok(())
}

/// 关键帧重置：按选择写 `time/qpos/qvel`；越界世界保持原状态。
pub fn reset_data_keyframe(
    state: &mut WorldState,
    keyframes: Keyframes<'_>,
    selection: KeyframeSelection<'_>,
) -> Result<(), ResetError> {
    let nkey = keyframes.nkey;
    let nq = state.nq;
    let nv = state.nv;
    for (field, actual, expected) in [
        ("key_qpos", keyframes.qpos.len(), nkey * nq),
        ("key_qvel", keyframes.qvel.len(), nkey * nv),
    ] {
        if actual != expected {
            return Err(ResetError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    match selection {
        KeyframeSelection::All(key) => {
            if key < 0 || key as usize >= nkey {
                return Err(ResetError::KeyframeOutOfRange { key, nkey });
            }
            for world in 0..state.nworld {
                apply_keyframe(state, keyframes, key as usize, world);
            }
        }
        KeyframeSelection::Worlds(keys) => {
            if keys.len() != state.nworld {
                return Err(ResetError::LengthMismatch {
                    field: "key",
                    expected: state.nworld,
                    actual: keys.len(),
                });
            }
            for (world, key) in keys.iter().enumerate() {
                if *key < 0 || *key as usize >= nkey {
                    continue;
                }
                apply_keyframe(state, keyframes, *key as usize, world);
            }
        }
    }
    Ok(())
}

fn check_mask_length(mask: ResetMask<'_>, nworld: usize) -> Result<(), ResetError> {
    match mask {
        ResetMask::All => Ok(()),
        ResetMask::Worlds(values) if values.len() == nworld => Ok(()),
        ResetMask::Worlds(values) => Err(ResetError::LengthMismatch {
            field: "reset",
            expected: nworld,
            actual: values.len(),
        }),
        ResetMask::Flags(values) if values.len() == nworld => Ok(()),
        ResetMask::Flags(values) => Err(ResetError::LengthMismatch {
            field: "reset",
            expected: nworld,
            actual: values.len(),
        }),
    }
}

fn apply_keyframe(state: &mut WorldState, keyframes: Keyframes<'_>, key: usize, world: usize) {
    let nq = state.nq;
    let nv = state.nv;
    state.time[world] = keyframes.time[key];
    state.qpos[world * nq..(world + 1) * nq]
        .copy_from_slice(&keyframes.qpos[key * nq..(key + 1) * nq]);
    state.qvel[world * nv..(world + 1) * nv]
        .copy_from_slice(&keyframes.qvel[key * nv..(key + 1) * nv]);
}

mod extended;
pub use extended::{
    ExtendedBatches, ExtendedKeyframes, ExtendedLayout, ExtendedState, MocapDefaults,
    reset_data_extended, reset_data_keyframe_extended,
};

#[cfg(test)]
mod tests;
