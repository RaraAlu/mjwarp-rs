//! M08@P7 局部重置字段扩展：ctrl/act/qfrc_applied/xfrc_applied 与 mocap。
//!
//! 对齐冻结上游 `mujoco_warp.reset_data`/`reset_data_keyframe` 的 P7 字段切片：
//! 掩码内世界 ctrl/act/qfrc_applied/xfrc_applied 置零，
//! mocap_pos/mocap_quat 复位为模型 mocap 身体的 `body_pos`/`body_quat`；
//! 关键帧路径写入 `key_act/key_ctrl/key_mpos/key_mquat` 与 time/qpos/qvel。
//!
//! 未验收边界：G24 历史（reset_history 等）、eq_active（依赖等式约束字段）、
//! qacc_warmstart/act_dot/sensordata/userdata/能量与睡眠字段、完整 Data 与
//! 设备内核均未包含。

use super::{KeyframeSelection, ResetError, ResetMask};

/// 扩展状态的维度（世界×分量）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtendedLayout {
    pub nworld: usize,
    pub nq: usize,
    pub nv: usize,
    pub nu: usize,
    pub na: usize,
    pub nmocap: usize,
    pub nbody: usize,
}

/// 扩展世界批量状态（全部 f32）。
#[derive(Clone, Debug, PartialEq)]
pub struct ExtendedState {
    layout: ExtendedLayout,
    time: Vec<f32>,
    qpos: Vec<f32>,
    qvel: Vec<f32>,
    ctrl: Vec<f32>,
    act: Vec<f32>,
    qfrc_applied: Vec<f32>,
    xfrc_applied: Vec<f32>,
    mocap_pos: Vec<f32>,
    mocap_quat: Vec<f32>,
}

impl ExtendedState {
    /// 新建全零扩展状态。
    pub fn new(layout: ExtendedLayout) -> Result<Self, ResetError> {
        let empty = |field: &'static str, count: usize| -> Result<Vec<f32>, ResetError> {
            if count == 0 {
                return Ok(Vec::new());
            }
            let _ = field;
            Ok(vec![0.0; count])
        };
        let sizes = dims(&layout)?;
        Ok(Self {
            layout,
            time: empty("time", sizes.0)?,
            qpos: empty("qpos", sizes.1)?,
            qvel: empty("qvel", sizes.2)?,
            ctrl: empty("ctrl", sizes.3)?,
            act: empty("act", sizes.4)?,
            qfrc_applied: empty("qfrc_applied", sizes.5)?,
            xfrc_applied: empty("xfrc_applied", sizes.6)?,
            mocap_pos: empty("mocap_pos", sizes.7)?,
            mocap_quat: empty("mocap_quat", sizes.8)?,
        })
    }

    /// 由批量数组构造状态，逐字段校验长度。
    #[allow(clippy::too_many_arguments)]
    pub fn from_batches(
        layout: ExtendedLayout,
        batches: ExtendedBatches<'_>,
    ) -> Result<Self, ResetError> {
        let sizes = dims(&layout)?;
        let mut state = Self::new(layout)?;
        for (field, actual, expected) in [
            ("time", batches.time.len(), sizes.0),
            ("qpos", batches.qpos.len(), sizes.1),
            ("qvel", batches.qvel.len(), sizes.2),
            ("ctrl", batches.ctrl.len(), sizes.3),
            ("act", batches.act.len(), sizes.4),
            ("qfrc_applied", batches.qfrc_applied.len(), sizes.5),
            ("xfrc_applied", batches.xfrc_applied.len(), sizes.6),
            ("mocap_pos", batches.mocap_pos.len(), sizes.7),
            ("mocap_quat", batches.mocap_quat.len(), sizes.8),
        ] {
            if actual != expected {
                return Err(ResetError::LengthMismatch {
                    field,
                    expected,
                    actual,
                });
            }
        }
        state.time.copy_from_slice(batches.time);
        state.qpos.copy_from_slice(batches.qpos);
        state.qvel.copy_from_slice(batches.qvel);
        state.ctrl.copy_from_slice(batches.ctrl);
        state.act.copy_from_slice(batches.act);
        state.qfrc_applied.copy_from_slice(batches.qfrc_applied);
        state.xfrc_applied.copy_from_slice(batches.xfrc_applied);
        state.mocap_pos.copy_from_slice(batches.mocap_pos);
        state.mocap_quat.copy_from_slice(batches.mocap_quat);
        Ok(state)
    }

    pub fn layout(&self) -> ExtendedLayout {
        self.layout
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

    pub fn ctrl(&self) -> &[f32] {
        &self.ctrl
    }

    pub fn act(&self) -> &[f32] {
        &self.act
    }

    pub fn qfrc_applied(&self) -> &[f32] {
        &self.qfrc_applied
    }

    pub fn xfrc_applied(&self) -> &[f32] {
        &self.xfrc_applied
    }

    pub fn mocap_pos(&self) -> &[f32] {
        &self.mocap_pos
    }

    pub fn mocap_quat(&self) -> &[f32] {
        &self.mocap_quat
    }
}

/// 扩展状态构造所需的批量数组。
#[derive(Clone, Copy, Debug)]
pub struct ExtendedBatches<'a> {
    pub time: &'a [f32],
    pub qpos: &'a [f32],
    pub qvel: &'a [f32],
    pub ctrl: &'a [f32],
    pub act: &'a [f32],
    pub qfrc_applied: &'a [f32],
    pub xfrc_applied: &'a [f32],
    pub mocap_pos: &'a [f32],
    pub mocap_quat: &'a [f32],
}

/// mocap 复位的模型默认值（按身体索引提供，`body_mocapid >= 0` 的身体生效）。
#[derive(Clone, Copy, Debug)]
pub struct MocapDefaults<'a> {
    pub body_mocapid: &'a [i32],
    pub body_pos: &'a [f32],
    pub body_quat: &'a [f32],
}

/// 扩展关键帧模型字段。
#[derive(Clone, Copy, Debug)]
pub struct ExtendedKeyframes<'a> {
    pub nkey: usize,
    pub time: &'a [f32],
    pub qpos: &'a [f32],
    pub qvel: &'a [f32],
    pub act: &'a [f32],
    pub ctrl: &'a [f32],
    pub mpos: &'a [f32],
    pub mquat: &'a [f32],
}

/// 世界掩码重置扩展字段。
pub fn reset_data_extended(
    state: &mut ExtendedState,
    qpos0: &[f32],
    mocap: MocapDefaults<'_>,
    mask: ResetMask<'_>,
) -> Result<(), ResetError> {
    let layout = state.layout;
    if qpos0.len() != layout.nq {
        return Err(ResetError::LengthMismatch {
            field: "qpos0",
            expected: layout.nq,
            actual: qpos0.len(),
        });
    }
    check_mask(mask, layout.nworld)?;
    let defaults = mocap_defaults(&layout, mocap)?;
    for world in 0..layout.nworld {
        if !mask_resets(mask, world)? {
            continue;
        }
        state.time[world] = 0.0;
        let qpos = &mut state.qpos[world * layout.nq..(world + 1) * layout.nq];
        qpos.copy_from_slice(qpos0);
        zero(&mut state.qvel[world * layout.nv..(world + 1) * layout.nv]);
        zero(&mut state.ctrl[world * layout.nu..(world + 1) * layout.nu]);
        zero(&mut state.act[world * layout.na..(world + 1) * layout.na]);
        zero(&mut state.qfrc_applied[world * layout.nv..(world + 1) * layout.nv]);
        zero(&mut state.xfrc_applied[world * layout.nbody * 6..(world + 1) * layout.nbody * 6]);
        for (index, value) in defaults.iter().enumerate() {
            state.mocap_pos[world * layout.nmocap * 3 + index * 3..][..3].copy_from_slice(&value.0);
            state.mocap_quat[world * layout.nmocap * 4 + index * 4..][..4]
                .copy_from_slice(&value.1);
        }
    }
    Ok(())
}

/// 关键帧重置扩展字段；逐世界越界 key 跳过。
pub fn reset_data_keyframe_extended(
    state: &mut ExtendedState,
    keyframes: ExtendedKeyframes<'_>,
    selection: KeyframeSelection<'_>,
) -> Result<(), ResetError> {
    let layout = state.layout;
    let nkey = keyframes.nkey;
    for (field, actual, expected) in [
        ("key_time", keyframes.time.len(), nkey),
        ("key_qpos", keyframes.qpos.len(), nkey * layout.nq),
        ("key_qvel", keyframes.qvel.len(), nkey * layout.nv),
        ("key_act", keyframes.act.len(), nkey * layout.na),
        ("key_ctrl", keyframes.ctrl.len(), nkey * layout.nu),
        ("key_mpos", keyframes.mpos.len(), nkey * layout.nmocap * 3),
        ("key_mquat", keyframes.mquat.len(), nkey * layout.nmocap * 4),
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
            for world in 0..layout.nworld {
                apply_keyframe(state, &keyframes, key as usize, world);
            }
        }
        KeyframeSelection::Worlds(keys) => {
            if keys.len() != layout.nworld {
                return Err(ResetError::LengthMismatch {
                    field: "key",
                    expected: layout.nworld,
                    actual: keys.len(),
                });
            }
            for (world, key) in keys.iter().enumerate() {
                if *key < 0 || *key as usize >= nkey {
                    continue;
                }
                apply_keyframe(state, &keyframes, *key as usize, world);
            }
        }
    }
    Ok(())
}

fn apply_keyframe(
    state: &mut ExtendedState,
    keyframes: &ExtendedKeyframes<'_>,
    key: usize,
    world: usize,
) {
    let layout = state.layout;
    state.time[world] = keyframes.time[key];
    let qpos = &mut state.qpos[world * layout.nq..(world + 1) * layout.nq];
    qpos.copy_from_slice(&keyframes.qpos[key * layout.nq..(key + 1) * layout.nq]);
    let qvel = &mut state.qvel[world * layout.nv..(world + 1) * layout.nv];
    qvel.copy_from_slice(&keyframes.qvel[key * layout.nv..(key + 1) * layout.nv]);
    let act = &mut state.act[world * layout.na..(world + 1) * layout.na];
    act.copy_from_slice(&keyframes.act[key * layout.na..(key + 1) * layout.na]);
    let ctrl = &mut state.ctrl[world * layout.nu..(world + 1) * layout.nu];
    ctrl.copy_from_slice(&keyframes.ctrl[key * layout.nu..(key + 1) * layout.nu]);
    let mpos = &mut state.mocap_pos[world * layout.nmocap * 3..(world + 1) * layout.nmocap * 3];
    mpos.copy_from_slice(&keyframes.mpos[key * layout.nmocap * 3..(key + 1) * layout.nmocap * 3]);
    let mquat = &mut state.mocap_quat[world * layout.nmocap * 4..(world + 1) * layout.nmocap * 4];
    mquat.copy_from_slice(&keyframes.mquat[key * layout.nmocap * 4..(key + 1) * layout.nmocap * 4]);
    // 上游 reset_data_keyframe 先按有效 key 世界执行 reset_data（清零其余 Data 字段），
    // 再写入关键帧字段；这里同样清零 qfrc_applied/xfrc_applied。
    zero(&mut state.qfrc_applied[world * layout.nv..(world + 1) * layout.nv]);
    zero(&mut state.xfrc_applied[world * layout.nbody * 6..(world + 1) * layout.nbody * 6]);
}

#[allow(clippy::type_complexity)]
type Sizes = (
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
);

fn dims(layout: &ExtendedLayout) -> Result<Sizes, ResetError> {
    if layout.nworld == 0 {
        return Err(ResetError::InvalidDimension { field: "nworld" });
    }
    let nq = layout.nq;
    let nv = layout.nv;
    let nbody = layout.nbody;
    Ok((
        layout.nworld,
        layout.nworld * nq,
        layout.nworld * nv,
        layout.nworld * layout.nu,
        layout.nworld * layout.na,
        layout.nworld * nv,
        layout.nworld * nbody * 6,
        layout.nworld * layout.nmocap * 3,
        layout.nworld * layout.nmocap * 4,
    ))
}

#[allow(clippy::type_complexity)]
fn mocap_defaults(
    layout: &ExtendedLayout,
    mocap: MocapDefaults<'_>,
) -> Result<Vec<([f32; 3], [f32; 4])>, ResetError> {
    for (field, actual, expected) in [
        ("body_mocapid", mocap.body_mocapid.len(), layout.nbody),
        ("body_pos", mocap.body_pos.len(), layout.nbody * 3),
        ("body_quat", mocap.body_quat.len(), layout.nbody * 4),
    ] {
        if actual != expected {
            return Err(ResetError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    let mut defaults = vec![([0.0; 3], [1.0, 0.0, 0.0, 0.0]); layout.nmocap];
    let mut seen = vec![false; layout.nmocap];
    for body in 0..layout.nbody {
        let mocapid = mocap.body_mocapid[body];
        if mocapid < 0 {
            continue;
        }
        if mocapid as usize >= layout.nmocap {
            return Err(ResetError::InvalidDimension {
                field: "body_mocapid",
            });
        }
        let index = mocapid as usize;
        defaults[index] = (
            [
                mocap.body_pos[body * 3],
                mocap.body_pos[body * 3 + 1],
                mocap.body_pos[body * 3 + 2],
            ],
            [
                mocap.body_quat[body * 4],
                mocap.body_quat[body * 4 + 1],
                mocap.body_quat[body * 4 + 2],
                mocap.body_quat[body * 4 + 3],
            ],
        );
        seen[index] = true;
    }
    if seen.iter().any(|flag| !flag) {
        return Err(ResetError::InvalidDimension {
            field: "body_mocapid",
        });
    }
    Ok(defaults)
}

fn zero(values: &mut [f32]) {
    values.fill(0.0);
}

fn check_mask(mask: ResetMask<'_>, nworld: usize) -> Result<(), ResetError> {
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

fn mask_resets(mask: ResetMask<'_>, world: usize) -> Result<bool, ResetError> {
    Ok(match mask {
        ResetMask::All => true,
        ResetMask::Worlds(values) => values[world],
        ResetMask::Flags(values) => values[world] != 0,
    })
}

#[cfg(test)]
mod tests;
