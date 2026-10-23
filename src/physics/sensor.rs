//! G23@P7 传感器切片：`sensor_pos`/`sensor_vel` 的标量与四元数写出。
//!
//! 对齐冻结上游 `mujoco_warp._src.sensor`（G23；P7，提交 `71da24d956…`，
//! Apache-2.0）`_sensor_pos`/`_sensor_vel` 的四个类型子集：
//!
//! - `JOINTPOS`：`qpos[jnt_qposadr[objid]]`（位置阶段、标量）；
//! - `JOINTVEL`：`qvel[jnt_dofadr[objid]]`（速度阶段、标量）；
//! - `BALLQUAT`：`qpos[jnt_qposadr[objid]..+4]` 四元数（位置阶段、4 分量）；
//! - `CLOCK`：`time`（位置阶段、标量）。
//!
//! 写出遵循上游 `_write_scalar`/`_write_vector`：`sensor_adr` 偏移、`cutoff`
//! 截断（`REAL` → clamp(±cutoff)，`POSITIVE` → min(cutoff)），`QUATERNION`
//! 不截断。类型/时序/单位（qpos 长度单位、qvel 速度单位、时间单位）与上游一致。
//!
//! 未验收边界：接触/测距/帧位姿/速度/加速度传感器、tendon/actuator 传感器、
//! 磁力计/相机投影/geomfromto 特例、SENSOR_STAGE 调度（sensor_acc 等阶段）、
//! 世界批量广播语义、GPU 内核与图执行。

use std::error::Error;
use std::fmt;

/// 上游 `mjtSensor` 子集。
pub const SENSOR_JOINTPOS: i32 = 9;
pub const SENSOR_JOINTVEL: i32 = 10;
pub const SENSOR_BALLQUAT: i32 = 18;
pub const SENSOR_CLOCK: i32 = 45;
/// 上游 `mjtDataType` 子集。
pub const DATA_REAL: i32 = 0;
pub const DATA_POSITIVE: i32 = 1;

/// 冻结传感器布局（`mjm` 的 sensor/joint 数组 + `nsensordata`）。
#[derive(Clone, Copy, Debug)]
pub struct SensorLayout<'a> {
    pub nq: usize,
    pub nv: usize,
    pub nsensordata: usize,
    pub sensor_type: &'a [i32],
    pub sensor_datatype: &'a [i32],
    pub sensor_objid: &'a [i32],
    pub sensor_adr: &'a [i32],
    pub sensor_cutoff: &'a [f32],
    pub jnt_qposadr: &'a [i32],
    pub jnt_dofadr: &'a [i32],
}

/// 传感器错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SensorError {
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

impl fmt::Display for SensorError {
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

impl Error for SensorError {}

fn validate(layout: &SensorLayout<'_>, nworld: usize) -> Result<(), SensorError> {
    let nsensor = layout.sensor_type.len();
    for (field, actual) in [
        ("sensor_datatype", layout.sensor_datatype.len()),
        ("sensor_objid", layout.sensor_objid.len()),
        ("sensor_adr", layout.sensor_adr.len()),
        ("sensor_cutoff", layout.sensor_cutoff.len()),
    ] {
        if actual != nsensor {
            return Err(SensorError::InvalidLength {
                field,
                expected: nsensor,
                actual,
            });
        }
    }
    let _ = nworld;
    for sensor in 0..nsensor {
        let adr = layout.sensor_adr[sensor];
        let dim = if layout.sensor_type[sensor] == SENSOR_BALLQUAT {
            4
        } else {
            1
        };
        if adr < 0 || adr as usize + dim > layout.nsensordata {
            return Err(SensorError::InvalidIndex {
                field: "sensor_adr",
                index: sensor,
                limit: layout.nsensordata,
            });
        }
    }
    Ok(())
}

fn write_scalar(
    layout: &SensorLayout<'_>,
    output: &mut [f32],
    world: usize,
    sensor: usize,
    value: f32,
) {
    let adr = layout.sensor_adr[sensor] as usize;
    let cutoff = layout.sensor_cutoff[sensor];
    let index = world * layout.nsensordata + adr;
    let datatype = layout.sensor_datatype[sensor];
    if cutoff > 0.0 {
        if datatype == DATA_REAL {
            output[index] = value.max(-cutoff).min(cutoff);
            return;
        }
        if datatype == DATA_POSITIVE {
            output[index] = value.min(cutoff);
            return;
        }
    }
    output[index] = value;
}

fn write_vector(
    layout: &SensorLayout<'_>,
    output: &mut [f32],
    world: usize,
    sensor: usize,
    values: &[f32],
) {
    let adr = layout.sensor_adr[sensor] as usize;
    let cutoff = layout.sensor_cutoff[sensor];
    let datatype = layout.sensor_datatype[sensor];
    for (offset, value) in values.iter().enumerate() {
        let index = world * layout.nsensordata + adr + offset;
        if cutoff > 0.0 {
            if datatype == DATA_REAL {
                output[index] = value.max(-cutoff).min(cutoff);
                continue;
            }
            if datatype == DATA_POSITIVE {
                output[index] = value.min(cutoff);
                continue;
            }
        }
        output[index] = *value;
    }
}

fn joint_qposadr(layout: &SensorLayout<'_>, sensor: usize) -> Result<usize, SensorError> {
    let objid = layout.sensor_objid[sensor];
    if objid < 0 || objid as usize >= layout.jnt_qposadr.len() {
        return Err(SensorError::InvalidIndex {
            field: "sensor_objid",
            index: sensor,
            limit: layout.jnt_qposadr.len(),
        });
    }
    Ok(layout.jnt_qposadr[objid as usize] as usize)
}

fn joint_dofadr(layout: &SensorLayout<'_>, sensor: usize) -> Result<usize, SensorError> {
    let objid = layout.sensor_objid[sensor];
    if objid < 0 || objid as usize >= layout.jnt_dofadr.len() {
        return Err(SensorError::InvalidIndex {
            field: "sensor_objid",
            index: sensor,
            limit: layout.jnt_dofadr.len(),
        });
    }
    Ok(layout.jnt_dofadr[objid as usize] as usize)
}

/// 位置阶段传感器：JOINTPOS/BALLQUAT/CLOCK。
pub fn sensor_pos(
    layout: &SensorLayout<'_>,
    nworld: usize,
    qpos: &[f32],
    time: &[f32],
) -> Result<Vec<f32>, SensorError> {
    validate(layout, nworld)?;
    for (field, actual, expected) in [
        ("qpos", qpos.len(), nworld * layout.nq),
        ("time", time.len(), nworld),
    ] {
        if actual != expected {
            return Err(SensorError::InvalidLength {
                field,
                expected,
                actual,
            });
        }
    }
    let mut output = vec![0.0_f32; nworld * layout.nsensordata];
    for world in 0..nworld {
        for sensor in 0..layout.sensor_type.len() {
            match layout.sensor_type[sensor] {
                SENSOR_JOINTPOS => {
                    let adr = joint_qposadr(layout, sensor)?;
                    if adr >= layout.nq {
                        return Err(SensorError::InvalidIndex {
                            field: "jnt_qposadr",
                            index: sensor,
                            limit: layout.nq,
                        });
                    }
                    write_scalar(
                        layout,
                        &mut output,
                        world,
                        sensor,
                        qpos[world * layout.nq + adr],
                    );
                }
                SENSOR_BALLQUAT => {
                    let adr = joint_qposadr(layout, sensor)?;
                    if adr + 4 > layout.nq {
                        return Err(SensorError::InvalidIndex {
                            field: "jnt_qposadr",
                            index: sensor,
                            limit: layout.nq,
                        });
                    }
                    let quat = &qpos[world * layout.nq + adr..world * layout.nq + adr + 4];
                    write_vector(layout, &mut output, world, sensor, quat);
                }
                SENSOR_CLOCK => write_scalar(layout, &mut output, world, sensor, time[world]),
                _ => {
                    // 非位置阶段传感器由其它阶段写出；本片按上游分阶段调度跳过。
                }
            }
        }
    }
    Ok(output)
}

/// 速度阶段传感器：JOINTVEL。
pub fn sensor_vel(
    layout: &SensorLayout<'_>,
    nworld: usize,
    qvel: &[f32],
) -> Result<Vec<f32>, SensorError> {
    validate(layout, nworld)?;
    let expected = nworld * layout.nv;
    if qvel.len() != expected {
        return Err(SensorError::InvalidLength {
            field: "qvel",
            expected,
            actual: qvel.len(),
        });
    }
    let mut output = vec![0.0_f32; nworld * layout.nsensordata];
    for world in 0..nworld {
        for sensor in 0..layout.sensor_type.len() {
            match layout.sensor_type[sensor] {
                SENSOR_JOINTVEL => {
                    let adr = joint_dofadr(layout, sensor)?;
                    if adr >= layout.nv {
                        return Err(SensorError::InvalidIndex {
                            field: "jnt_dofadr",
                            index: sensor,
                            limit: layout.nv,
                        });
                    }
                    write_scalar(
                        layout,
                        &mut output,
                        world,
                        sensor,
                        qvel[world * layout.nv + adr],
                    );
                }
                _ => {
                    // 速度阶段不写位置类传感器（保持 0，对齐上游分阶段写出）。
                }
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests;
