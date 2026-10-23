//! G16@P6 积分器切片：Euler（半隐式）与 RK4 的状态推进算术。
//!
//! 对齐冻结上游 `mujoco_warp._src.forward`（U014/U015 家族；G16；P6，提交
//! `71da24d956…`，Apache-2.0）的 `_next_position`/`_next_velocity`/`_advance`
//! 与 `rungekutta4` 组合：
//!
//! - `next_position`：free 关节平移 + 四元数积分（`math.quat_integrate`）、
//!   ball 四元数积分、hinge/slide 标量积分；`qvel_scale` 与上游一致。
//! - `next_velocity`：`qvel + qacc_scale * qacc * timestep`。
//! - `euler`：半隐式——先推进速度，再用新速度推进位置，`time += timestep`。
//! - `rungekutta4`：`_rk_accumulate` 的 `1/6,1/3,1/3,1/6` 组合与 `_advance`；
//!   **4 级导数由冻结输入提供**——本片不实现 `forward()` 动力学链。
//!
//! 未验收边界：activation/mocap 推进、`_next_time` 的 overflow/efc 统计、
//! 阻尼隐式分支（`EULERDAMP`/`DAMPER`）、`forward()` 逐级求导、DISCRETE/
//! IMPLICIT 积分器、设备内核与图执行。

use std::error::Error;
use std::fmt;

/// 上游 `mjtJoint` 的 free/ball 编号；hinge/slide 等单自由度关节走标量分支。
pub const JOINT_FREE: i32 = 0;
pub const JOINT_BALL: i32 = 1;

/// RK4 组合权重（上游 `B` 表）。
pub const RK4_WEIGHTS: [f32; 4] = [
    (1.0_f64 / 6.0) as f32,
    (1.0_f64 / 3.0) as f32,
    (1.0_f64 / 3.0) as f32,
    (1.0_f64 / 6.0) as f32,
];

/// 关节布局（上游 `jnt_type`/`jnt_qposadr`/`jnt_dofadr`）。
#[derive(Clone, Copy, Debug)]
pub struct JointLayout<'a> {
    pub nq: usize,
    pub nv: usize,
    pub jnt_type: &'a [i32],
    pub jnt_qposadr: &'a [i32],
    pub jnt_dofadr: &'a [i32],
}

/// 冻结状态（每世界 `nq`/`nv` 展平）。
#[derive(Clone, Copy, Debug)]
pub struct IntegratorState<'a> {
    pub nworld: usize,
    pub qpos: &'a [f32],
    pub qvel: &'a [f32],
}

/// 单步结果。
#[derive(Clone, Debug, PartialEq)]
pub struct IntegratorStep {
    pub qpos: Vec<f32>,
    pub qvel: Vec<f32>,
    pub time: f32,
}

/// RK4 单级导数（冻结输入：该级 `qvel` 与 `qacc`）。
#[derive(Clone, Copy, Debug)]
pub struct Rk4Stage<'a> {
    pub qvel: &'a [f32],
    pub qacc: &'a [f32],
}

/// 积分器错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntegratorError {
    InvalidLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    InvalidIndex {
        field: &'static str,
        index: usize,
        limit: usize,
    },
}

impl fmt::Display for IntegratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "长度不匹配：{field} 期望 {expected}，实际 {actual}"
            ),
            Self::InvalidIndex {
                field,
                index,
                limit,
            } => write!(formatter, "索引越界：{field}[{index}]，上限 {limit}"),
        }
    }
}

impl Error for IntegratorError {}

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn normalize4(quat: [f32; 4]) -> [f32; 4] {
    let norm =
        (quat[0] * quat[0] + quat[1] * quat[1] + quat[2] * quat[2] + quat[3] * quat[3]).sqrt();
    [
        quat[0] / norm,
        quat[1] / norm,
        quat[2] / norm,
        quat[3] / norm,
    ]
}

fn mul_quat(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    [
        left[0] * right[0] - left[1] * right[1] - left[2] * right[2] - left[3] * right[3],
        left[0] * right[1] + left[1] * right[0] + left[2] * right[3] - left[3] * right[2],
        left[0] * right[2] - left[1] * right[3] + left[2] * right[0] + left[3] * right[1],
        left[0] * right[3] + left[1] * right[2] - left[2] * right[1] + left[3] * right[0],
    ]
}

/// 上游 `math.quat_integrate`：轴角增量乘到四元数上（wxyz）。
pub fn quat_integrate(quat: [f32; 4], velocity: [f32; 3], timestep: f32) -> [f32; 4] {
    let norm = dot3(velocity, velocity).sqrt();
    let axis = [velocity[0] / norm, velocity[1] / norm, velocity[2] / norm];
    let angle = timestep * norm;
    let half = angle * 0.5;
    let sin = half.sin();
    let cos = half.cos();
    let delta = [cos, axis[0] * sin, axis[1] * sin, axis[2] * sin];
    normalize4(mul_quat(normalize4(quat), delta))
}

/// 上游 `_next_velocity`：`qvel + qacc_scale * qacc * timestep`。
pub fn next_velocity(
    nworld: usize,
    nv: usize,
    qvel: &[f32],
    qacc: &[f32],
    timestep: f32,
    scale: f32,
) -> Result<Vec<f32>, IntegratorError> {
    let expected = nworld * nv;
    for (field, values) in [("qvel", qvel), ("qacc", qacc)] {
        if values.len() != expected {
            return Err(IntegratorError::InvalidLength {
                field,
                expected,
                actual: values.len(),
            });
        }
    }
    let mut output = Vec::with_capacity(expected);
    for index in 0..expected {
        output.push(qvel[index] + (scale * qacc[index]) * timestep);
    }
    Ok(output)
}

fn validate_layout(layout: &JointLayout<'_>) -> Result<(), IntegratorError> {
    let njnt = layout.jnt_type.len();
    for (field, actual) in [
        ("jnt_qposadr", layout.jnt_qposadr.len()),
        ("jnt_dofadr", layout.jnt_dofadr.len()),
    ] {
        if actual != njnt {
            return Err(IntegratorError::InvalidLength {
                field,
                expected: njnt,
                actual,
            });
        }
    }
    let nq = layout.nq as i64;
    let nv = layout.nv as i64;
    for joint in 0..njnt {
        let qposadr = layout.jnt_qposadr[joint] as i64;
        let dofadr = layout.jnt_dofadr[joint] as i64;
        if qposadr < 0 || qposadr >= nq {
            return Err(IntegratorError::InvalidIndex {
                field: "jnt_qposadr",
                index: joint,
                limit: layout.nq,
            });
        }
        if dofadr < 0 || dofadr >= nv {
            return Err(IntegratorError::InvalidIndex {
                field: "jnt_dofadr",
                index: joint,
                limit: layout.nv,
            });
        }
    }
    Ok(())
}

/// 上游 `_next_position`：按关节类型推进 `qpos`（`qvel_scale` 逐关节生效）。
pub fn next_position(
    layout: &JointLayout<'_>,
    nworld: usize,
    qpos: &[f32],
    qvel: &[f32],
    timestep: f32,
    scale: f32,
) -> Result<Vec<f32>, IntegratorError> {
    let nq_expected = nworld * layout.nq;
    if qpos.len() != nq_expected {
        return Err(IntegratorError::InvalidLength {
            field: "qpos",
            expected: nq_expected,
            actual: qpos.len(),
        });
    }
    let nv_expected = nworld * layout.nv;
    if qvel.len() != nv_expected {
        return Err(IntegratorError::InvalidLength {
            field: "qvel",
            expected: nv_expected,
            actual: qvel.len(),
        });
    }
    let mut output = qpos.to_vec();
    validate_layout(layout)?;
    for world in 0..nworld {
        let qpos_base = world * layout.nq;
        let qvel_base = world * layout.nv;
        for joint in 0..layout.jnt_type.len() {
            let joint_type = layout.jnt_type[joint];
            let qposadr = qpos_base + layout.jnt_qposadr[joint] as usize;
            let dofadr = qvel_base + layout.jnt_dofadr[joint] as usize;
            match joint_type {
                JOINT_FREE => {
                    let position = [output[qposadr], output[qposadr + 1], output[qposadr + 2]];
                    let linear = [
                        qvel[dofadr] * scale,
                        qvel[dofadr + 1] * scale,
                        qvel[dofadr + 2] * scale,
                    ];
                    let quat = [
                        output[qposadr + 3],
                        output[qposadr + 4],
                        output[qposadr + 5],
                        output[qposadr + 6],
                    ];
                    let angular = [
                        qvel[dofadr + 3] * scale,
                        qvel[dofadr + 4] * scale,
                        qvel[dofadr + 5] * scale,
                    ];
                    let next = quat_integrate(quat, angular, timestep);
                    for axis in 0..3 {
                        output[qposadr + axis] = position[axis] + timestep * linear[axis];
                    }
                    output[qposadr + 3..qposadr + 7].copy_from_slice(&next);
                }
                JOINT_BALL => {
                    let quat = [
                        output[qposadr],
                        output[qposadr + 1],
                        output[qposadr + 2],
                        output[qposadr + 3],
                    ];
                    let angular = [
                        qvel[dofadr] * scale,
                        qvel[dofadr + 1] * scale,
                        qvel[dofadr + 2] * scale,
                    ];
                    let next = quat_integrate(quat, angular, timestep);
                    output[qposadr..qposadr + 4].copy_from_slice(&next);
                }
                _ => {
                    // 标量关节：上游 `qpos + timestep * qvel * scale`（左结合）。
                    output[qposadr] += timestep * qvel[dofadr] * scale;
                }
            }
        }
    }
    Ok(output)
}

/// 半隐式推进：先 `next_velocity`，再用新速度 `next_position`，`time += timestep`。
pub fn advance(
    layout: &JointLayout<'_>,
    state: IntegratorState<'_>,
    qacc: &[f32],
    qvel_derivative: &[f32],
    timestep: f32,
    time: f32,
) -> Result<IntegratorStep, IntegratorError> {
    let qvel = next_velocity(state.nworld, layout.nv, state.qvel, qacc, timestep, 1.0)?;
    let qpos = next_position(
        layout,
        state.nworld,
        state.qpos,
        qvel_derivative,
        timestep,
        1.0,
    )?;
    Ok(IntegratorStep {
        qpos,
        qvel,
        time: time + timestep,
    })
}

/// Euler 半隐式单步：使用冻结加速度 `qacc`。
pub fn euler(
    layout: &JointLayout<'_>,
    state: IntegratorState<'_>,
    qacc: &[f32],
    timestep: f32,
    time: f32,
) -> Result<IntegratorStep, IntegratorError> {
    let qvel = next_velocity(state.nworld, layout.nv, state.qvel, qacc, timestep, 1.0)?;
    let qpos = next_position(layout, state.nworld, state.qpos, &qvel, timestep, 1.0)?;
    Ok(IntegratorStep {
        qpos,
        qvel,
        time: time + timestep,
    })
}

/// RK4 单步：`1/6,1/3,1/3,1/6` 组合冻结的 4 级导数后做半隐式推进。
pub fn rungekutta4(
    layout: &JointLayout<'_>,
    state: IntegratorState<'_>,
    stages: [Rk4Stage<'_>; 4],
    timestep: f32,
    time: f32,
) -> Result<IntegratorStep, IntegratorError> {
    let nv_expected = state.nworld * layout.nv;
    let mut qvel_rk = vec![0.0_f32; nv_expected];
    let mut qacc_rk = vec![0.0_f32; nv_expected];
    for (stage_index, stage) in stages.iter().enumerate() {
        for (field, values) in [("qvel", stage.qvel), ("qacc", stage.qacc)] {
            if values.len() != nv_expected {
                return Err(IntegratorError::InvalidLength {
                    field,
                    expected: nv_expected,
                    actual: values.len(),
                });
            }
        }
        let weight = RK4_WEIGHTS[stage_index];
        for index in 0..nv_expected {
            qvel_rk[index] += weight * stage.qvel[index];
            qacc_rk[index] += weight * stage.qacc[index];
        }
    }
    advance(layout, state, &qacc_rk, &qvel_rk, timestep, time)
}

#[cfg(test)]
mod tests;
