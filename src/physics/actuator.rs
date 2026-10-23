//! G05 执行器阶段切片的宿主侧实现：`fwd_actuation`。
//!
//! 对齐冻结上游 `mujoco_warp.fwd_actuation`（U017；G05；P4/P7）的
//! joint 传动子集：ctrl 限幅、dyntype none/integrator/filter/filterexact
//! 的激活状态推进与限幅、gaintype fixed/affine、biastype none/affine、
//! 力限幅，输出 `act_dot`、`actuator_force` 与按 CSR moment 累加的
//! `qfrc_actuator`。
//!
//! 本片不实现 transmission（U068，G06）、执行器模型字段转换与设备内核；
//! `actuator_length`/`actuator_velocity`/moment CSR 作为冻结输入传入。
//!
//! 未验收边界：muscle/dcmotor/pid/user 四类 dyntype、muscle/dcmotor/so3/pid/user
//! gain/bias 类型与 tendon/site/body 传动全部显式拒绝；完整 G05、G06
//! 传动与 dynamic 迭代未包含。

use std::error::Error;
use std::fmt;

/// 执行器动力学类型（对齐 `mjtDyn`）。
pub const DYN_NONE: i32 = 0;
pub const DYN_INTEGRATOR: i32 = 1;
pub const DYN_FILTER: i32 = 2;
pub const DYN_FILTEREXACT: i32 = 3;
/// 执行器增益类型（对齐 `mjtGain`）。
pub const GAIN_FIXED: i32 = 0;
pub const GAIN_AFFINE: i32 = 1;
/// 执行器偏置类型（对齐 `mjtBias`）。
pub const BIAS_NONE: i32 = 0;
pub const BIAS_AFFINE: i32 = 1;
/// 上游 `mjMINVAL`。
const MJ_MINVAL: f32 = 1.0e-15;

/// 执行器输入错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActuationError {
    InvalidDimension {
        field: &'static str,
    },
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    Unsupported {
        field: &'static str,
        index: usize,
        value: i32,
    },
    InvalidIndex {
        field: &'static str,
        index: usize,
        limit: usize,
    },
}

impl fmt::Display for ActuationError {
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
            Self::Unsupported {
                field,
                index,
                value,
            } => write!(formatter, "不支持的类型：{field}[{index}]={value}"),
            Self::InvalidIndex {
                field,
                index,
                limit,
            } => write!(formatter, "索引越界：{field}[{index}] >= {limit}"),
        }
    }
}

impl Error for ActuationError {}

/// 执行器模型数组（joint 传动切片；每执行器 10/2 个参数槽）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActuatorModel<'a> {
    pub timestep: f32,
    pub disable_clampctrl: bool,
    pub dyntype: &'a [i32],
    pub gaintype: &'a [i32],
    pub biastype: &'a [i32],
    pub ctrladr: &'a [i32],
    pub ctrlnum: &'a [i32],
    pub actadr: &'a [i32],
    pub actnum: &'a [i32],
    pub dynprm: &'a [f32],
    pub gainprm: &'a [f32],
    pub biasprm: &'a [f32],
    pub actlimited: &'a [bool],
    pub actrange: &'a [f32],
    pub actearly: &'a [bool],
    pub forcelimited: &'a [bool],
    pub forcerange: &'a [f32],
    pub ctrllimited: &'a [bool],
    pub ctrlrange: &'a [f32],
}

/// 冻结数据数组（由上游 transmission/fwd_velocity 产出）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActuationData<'a> {
    pub nworld: usize,
    pub nv: usize,
    pub nu: usize,
    pub na: usize,
    pub ctrl: &'a [f32],
    pub act: &'a [f32],
    pub actuator_length: &'a [f32],
    pub actuator_velocity: &'a [f32],
    pub moment_width: usize,
    pub moment_rownnz: &'a [i32],
    pub moment_rowadr: &'a [i32],
    pub moment_colind: &'a [i32],
    pub actuator_moment: &'a [f32],
}

/// 执行器阶段输出（世界×分量展开）。
#[derive(Clone, Debug, PartialEq)]
pub struct ActuationOutput {
    act_dot: Vec<f32>,
    actuator_force: Vec<f32>,
    qfrc_actuator: Vec<f32>,
}

impl ActuationOutput {
    pub fn act_dot(&self) -> &[f32] {
        &self.act_dot
    }

    pub fn actuator_force(&self) -> &[f32] {
        &self.actuator_force
    }

    pub fn qfrc_actuator(&self) -> &[f32] {
        &self.qfrc_actuator
    }
}

/// 计算执行器力、激活导数与广义执行器力（上游 `fwd_actuation` 子集）。
pub fn fwd_actuation(
    model: ActuatorModel<'_>,
    data: ActuationData<'_>,
) -> Result<ActuationOutput, ActuationError> {
    let nact = model.dyntype.len();
    if nact == 0 {
        return Err(ActuationError::InvalidDimension { field: "nactuator" });
    }
    if data.nworld == 0 {
        return Err(ActuationError::InvalidDimension { field: "nworld" });
    }
    validate_model(model, nact)?;
    validate_data(data, nact)?;

    let mut act_dot = vec![0.0_f32; data.nworld * data.na];
    let mut actuator_force = vec![0.0_f32; data.nworld * nact];
    let mut qfrc_actuator = vec![0.0_f32; data.nworld * data.nv];
    for world in 0..data.nworld {
        for uid in 0..nact {
            let force = actuator_force_one(model, data, world, uid, &mut act_dot)?;
            actuator_force[world * nact + uid] = force;
            let rownnz = data.moment_rownnz[world * nact + uid];
            let rowadr = data.moment_rowadr[world * nact + uid];
            for offset in 0..rownnz as usize {
                let column =
                    data.moment_colind[world * data.moment_width + rowadr as usize + offset];
                let moment =
                    data.actuator_moment[world * data.moment_width + rowadr as usize + offset];
                qfrc_actuator[world * data.nv + column as usize] += moment * force;
            }
        }
    }
    Ok(ActuationOutput {
        act_dot,
        actuator_force,
        qfrc_actuator,
    })
}

fn actuator_force_one(
    model: ActuatorModel<'_>,
    data: ActuationData<'_>,
    world: usize,
    uid: usize,
    act_dot: &mut [f32],
) -> Result<f32, ActuationError> {
    let dyntype = model.dyntype[uid];
    if !matches!(
        dyntype,
        DYN_NONE | DYN_INTEGRATOR | DYN_FILTER | DYN_FILTEREXACT
    ) {
        return Err(ActuationError::Unsupported {
            field: "dyntype",
            index: uid,
            value: dyntype,
        });
    }
    let gaintype = model.gaintype[uid];
    if !matches!(gaintype, GAIN_FIXED | GAIN_AFFINE) {
        return Err(ActuationError::Unsupported {
            field: "gaintype",
            index: uid,
            value: gaintype,
        });
    }
    let biastype = model.biastype[uid];
    if !matches!(biastype, BIAS_NONE | BIAS_AFFINE) {
        return Err(ActuationError::Unsupported {
            field: "biastype",
            index: uid,
            value: biastype,
        });
    }

    let ctrlnum = model.ctrlnum[uid];
    let mut ctrl = 0.0_f32;
    if ctrlnum > 0 {
        let ctrladr = model.ctrladr[uid] as usize;
        ctrl = data.ctrl[world * data.nu + ctrladr];
        if model.ctrllimited[uid] && !model.disable_clampctrl {
            ctrl = clamp(ctrl, model.ctrlrange[uid * 2], model.ctrlrange[uid * 2 + 1]);
        }
    }
    let mut ctrl_act = ctrl;

    let actadr = model.actadr[uid];
    let actnum = model.actnum[uid];
    if data.na > 0 && actadr >= 0 {
        if actnum != 1 {
            return Err(ActuationError::Unsupported {
                field: "actnum",
                index: uid,
                value: actnum,
            });
        }
        let act_last = actadr as usize + actnum as usize - 1;
        let dynprm = &model.dynprm[uid * 10..uid * 10 + 10];
        let act_in = data.act[world * data.na + act_last];
        let act_dot_value = match dyntype {
            DYN_INTEGRATOR => ctrl,
            DYN_FILTER | DYN_FILTEREXACT => (ctrl - act_in) / dynprm[0].max(MJ_MINVAL),
            _ => 0.0,
        };
        act_dot[world * data.na + act_last] = act_dot_value;
        if model.actearly[uid] {
            if matches!(dyntype, DYN_INTEGRATOR | DYN_NONE) {
                ctrl_act = next_act(model, uid, dyntype, dynprm, ctrl_act, act_dot_value);
            } else {
                ctrl_act = next_act(model, uid, dyntype, dynprm, act_in, act_dot_value);
            }
        } else {
            ctrl_act = act_in;
        }
    }

    let length = data.actuator_length[world * model.dyntype.len() + uid];
    let velocity = data.actuator_velocity[world * model.dyntype.len() + uid];
    let gainprm = &model.gainprm[uid * 10..uid * 10 + 10];
    let gain = match gaintype {
        GAIN_FIXED => gainprm[0],
        _ => gainprm[0] + gainprm[1] * length + gainprm[2] * velocity,
    };
    let biasprm = &model.biasprm[uid * 10..uid * 10 + 10];
    let bias = match biastype {
        BIAS_NONE => 0.0,
        _ => biasprm[0] + biasprm[1] * length + biasprm[2] * velocity,
    };
    let mut force = gain * ctrl_act + bias;
    if model.forcelimited[uid] {
        force = clamp(
            force,
            model.forcerange[uid * 2],
            model.forcerange[uid * 2 + 1],
        );
    }
    Ok(force)
}

fn next_act(
    model: ActuatorModel<'_>,
    uid: usize,
    dyntype: i32,
    dynprm: &[f32],
    act_in: f32,
    act_dot: f32,
) -> f32 {
    let mut act = if dyntype == DYN_FILTEREXACT {
        let tau = dynprm[0].max(MJ_MINVAL);
        act_in + act_dot * tau * (1.0 - (-model.timestep / tau).exp())
    } else {
        act_in + act_dot * model.timestep
    };
    if model.actlimited[uid] {
        act = clamp(act, model.actrange[uid * 2], model.actrange[uid * 2 + 1]);
    }
    act
}

fn clamp(value: f32, lower: f32, upper: f32) -> f32 {
    value.min(upper).max(lower)
}

fn validate_model(model: ActuatorModel<'_>, nact: usize) -> Result<(), ActuationError> {
    for (field, actual, expected) in [
        ("gaintype", model.gaintype.len(), nact),
        ("biastype", model.biastype.len(), nact),
        ("ctrladr", model.ctrladr.len(), nact),
        ("ctrlnum", model.ctrlnum.len(), nact),
        ("actadr", model.actadr.len(), nact),
        ("actnum", model.actnum.len(), nact),
        ("dynprm", model.dynprm.len(), nact * 10),
        ("gainprm", model.gainprm.len(), nact * 10),
        ("biasprm", model.biasprm.len(), nact * 10),
        ("actlimited", model.actlimited.len(), nact),
        ("actrange", model.actrange.len(), nact * 2),
        ("actearly", model.actearly.len(), nact),
        ("forcelimited", model.forcelimited.len(), nact),
        ("forcerange", model.forcerange.len(), nact * 2),
        ("ctrllimited", model.ctrllimited.len(), nact),
        ("ctrlrange", model.ctrlrange.len(), nact * 2),
    ] {
        if actual != expected {
            return Err(ActuationError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

fn validate_data(data: ActuationData<'_>, nact: usize) -> Result<(), ActuationError> {
    let nworld = data.nworld;
    for (field, actual, expected) in [
        ("ctrl", data.ctrl.len(), nworld * data.nu),
        ("act", data.act.len(), nworld * data.na),
        ("actuator_length", data.actuator_length.len(), nworld * nact),
        (
            "actuator_velocity",
            data.actuator_velocity.len(),
            nworld * nact,
        ),
        ("moment_rownnz", data.moment_rownnz.len(), nworld * nact),
        ("moment_rowadr", data.moment_rowadr.len(), nworld * nact),
        (
            "moment_colind",
            data.moment_colind.len(),
            nworld * data.moment_width,
        ),
        (
            "actuator_moment",
            data.actuator_moment.len(),
            nworld * data.moment_width,
        ),
    ] {
        if actual != expected {
            return Err(ActuationError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    for world in 0..nworld {
        for uid in 0..nact {
            let rownnz = data.moment_rownnz[world * nact + uid];
            let rowadr = data.moment_rowadr[world * nact + uid];
            if rownnz < 0 || rowadr < 0 || (rowadr + rownnz) as usize > data.moment_width {
                return Err(ActuationError::InvalidIndex {
                    field: "moment",
                    index: world * nact + uid,
                    limit: data.moment_width,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
