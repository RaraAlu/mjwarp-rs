//! G05@P7 执行器阶段切片：关节传动（transmission）的宿主侧实现。
//!
//! 对齐冻结上游 `mujoco_warp.transmission`（U068；G05/G06；P4/P7）的
//! hinge/slide 关节子集：`actuator_length = qpos[jnt_qposadr] * gear[0]`、
//! 逐执行器 moment 行（nnz=1、colind=jnt_dofadr、moment=gear[0]）与
//! `actuator_velocity = moment·qvel`（单行累加）。
//!
//! 输出可直接作为 `fwd_actuation`（U017，P4 切片）的冻结输入，形成
//! transmission→velocity→actuation 端到端链。
//!
//! 未验收边界：free/ball/JOINTINPARENT 关节、tendon/site/body 传动、
//! transmission-body-moment、设备内核与完整 forward 均未包含。

use super::ActuationError;

/// 传动类型（对齐 `mjtTrn`）。
pub const TRN_JOINT: i32 = 0;
/// 关节类型（对齐 `mjtJoint`）。
pub const JOINT_SLIDE: i32 = 2;
pub const JOINT_HINGE: i32 = 3;

/// 关节传动模型数组。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointActuatorModel<'a> {
    pub nv: usize,
    pub trntype: &'a [i32],
    pub trnid: &'a [i32],
    pub gear: &'a [f32],
    pub jnt_type: &'a [i32],
    pub jnt_qposadr: &'a [i32],
    pub jnt_dofadr: &'a [i32],
}

/// 冻结关节数据（qpos/qvel）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointTransmissionData<'a> {
    pub nworld: usize,
    pub nq: usize,
    pub qpos: &'a [f32],
    pub qvel: &'a [f32],
}

/// 关节传动输出（世界×分量展开；moment 按每世界 CSR 展开）。
#[derive(Clone, Debug, PartialEq)]
pub struct JointTransmission {
    actuator_length: Vec<f32>,
    actuator_velocity: Vec<f32>,
    rownnz: Vec<i32>,
    rowadr: Vec<i32>,
    colind: Vec<i32>,
    moment: Vec<f32>,
    moment_width: usize,
}

impl JointTransmission {
    pub fn actuator_length(&self) -> &[f32] {
        &self.actuator_length
    }

    pub fn actuator_velocity(&self) -> &[f32] {
        &self.actuator_velocity
    }

    pub fn rownnz(&self) -> &[i32] {
        &self.rownnz
    }

    pub fn rowadr(&self) -> &[i32] {
        &self.rowadr
    }

    pub fn colind(&self) -> &[i32] {
        &self.colind
    }

    pub fn moment(&self) -> &[f32] {
        &self.moment
    }

    pub fn moment_width(&self) -> usize {
        self.moment_width
    }
}

/// 计算 hinge/slide 关节传动的 length/moment/velocity。
pub fn transmission(
    model: JointActuatorModel<'_>,
    data: JointTransmissionData<'_>,
) -> Result<JointTransmission, ActuationError> {
    let nact = model.trntype.len();
    if nact == 0 {
        return Err(ActuationError::InvalidDimension { field: "nactuator" });
    }
    if data.nworld == 0 || data.nq == 0 || model.nv == 0 {
        return Err(ActuationError::InvalidDimension { field: "nworld" });
    }
    for (field, actual, expected) in [
        ("actuator_trnid", model.trnid.len(), nact * 2),
        ("actuator_gear", model.gear.len(), nact * 6),
        ("actuator_trntype", model.trntype.len(), nact),
        ("jnt_qposadr", model.jnt_qposadr.len(), model.jnt_type.len()),
        ("jnt_dofadr", model.jnt_dofadr.len(), model.jnt_type.len()),
        ("qpos", data.qpos.len(), data.nworld * data.nq),
        ("qvel", data.qvel.len(), data.nworld * model.nv),
    ] {
        if actual != expected {
            return Err(ActuationError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    let moment_width = nact;
    let mut actuator_length = vec![0.0_f32; data.nworld * nact];
    let mut actuator_velocity = vec![0.0_f32; data.nworld * nact];
    let mut rownnz = vec![0_i32; data.nworld * nact];
    let mut rowadr = vec![0_i32; data.nworld * nact];
    let mut colind = vec![0_i32; data.nworld * moment_width];
    let mut moment = vec![0.0_f32; data.nworld * moment_width];
    for world in 0..data.nworld {
        for (running, uid) in (0..nact).enumerate() {
            let trntype = model.trntype[uid];
            if trntype != TRN_JOINT {
                return Err(ActuationError::Unsupported {
                    field: "actuator_trntype",
                    index: uid,
                    value: trntype,
                });
            }
            let jntid = model.trnid[uid * 2];
            if jntid < 0 || jntid as usize >= model.jnt_type.len() {
                return Err(ActuationError::InvalidIndex {
                    field: "actuator_trnid",
                    index: uid,
                    limit: model.jnt_type.len(),
                });
            }
            let jnt = jntid as usize;
            let jnt_type = model.jnt_type[jnt];
            if !matches!(jnt_type, JOINT_SLIDE | JOINT_HINGE) {
                return Err(ActuationError::Unsupported {
                    field: "jnt_type",
                    index: uid,
                    value: jnt_type,
                });
            }
            let qadr = model.jnt_qposadr[jnt];
            let vadr = model.jnt_dofadr[jnt];
            if qadr < 0 || qadr as usize >= data.nq {
                return Err(ActuationError::InvalidIndex {
                    field: "jnt_qposadr",
                    index: uid,
                    limit: data.nq,
                });
            }
            if vadr < 0 || vadr as usize >= model.nv {
                return Err(ActuationError::InvalidIndex {
                    field: "jnt_dofadr",
                    index: uid,
                    limit: model.nv,
                });
            }
            let gear = model.gear[uid * 6];
            actuator_length[world * nact + uid] = data.qpos[world * data.nq + qadr as usize] * gear;
            rownnz[world * nact + uid] = 1;
            rowadr[world * nact + uid] = running as i32;
            colind[world * moment_width + running] = vadr;
            moment[world * moment_width + running] = gear;
            let mut velocity = 0.0_f32;
            velocity += moment[world * moment_width + running]
                * data.qvel[world * model.nv + vadr as usize];
            actuator_velocity[world * nact + uid] = velocity;
        }
    }
    Ok(JointTransmission {
        actuator_length,
        actuator_velocity,
        rownnz,
        rowadr,
        colind,
        moment,
        moment_width,
    })
}

#[cfg(test)]
mod tests;
