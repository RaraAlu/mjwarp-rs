//! G27 常量计算的 P7 切片：qpos0 依赖常量的宿主侧实现。
//!
//! 对齐冻结上游 `mujoco_warp.set_const_0`（U052；G27；P2/P7）中
//! 不依赖质量矩阵求解链的部分：`tendon_length0`、
//! `cam_pos0/cam_poscom0/cam_mat0`、`light_pos0/light_poscom0/light_dir0`
//! 与 `meaninertia`。输入是冻结的 qpos0 中间量（ten_length、cam/light
//! 世界位姿、xpos、subtree_com 与 M CSR），本片不重跑
//! kinematics/com/camlight/tendon/crb/factor_m 链。
//!
//! 未验收边界：`dof/body/tendon_invweight0`（依赖 M 求解）、`eq_data0`、
//! 执行器常量（依赖 G05 字段）、`set_const_spring`/全量 `set_const`、
//! 设备内核与链式重算均未包含。

use crate::diagnostics::InputError;

/// qpos0 依赖常量的冻结输入。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QposDependentInput<'a> {
    pub cam_bodyid: &'a [i32],
    pub cam_targetbodyid: &'a [i32],
    pub cam_xpos: &'a [f32],
    pub cam_xmat: &'a [f32],
    pub light_bodyid: &'a [i32],
    pub light_targetbodyid: &'a [i32],
    pub light_xpos: &'a [f32],
    pub light_xdir: &'a [f32],
    pub xpos: &'a [f32],
    pub subtree_com: &'a [f32],
    pub ten_length: &'a [f32],
    pub m_rownnz: &'a [i32],
    pub m_rowadr: &'a [i32],
    pub m_values: &'a [f32],
}

/// qpos0 依赖常量结果。
#[derive(Clone, Debug, PartialEq)]
pub struct QposDependentConstants {
    tendon_length0: Vec<f32>,
    cam_pos0: Vec<f32>,
    cam_poscom0: Vec<f32>,
    cam_mat0: Vec<f32>,
    light_pos0: Vec<f32>,
    light_poscom0: Vec<f32>,
    light_dir0: Vec<f32>,
    meaninertia: f32,
}

impl QposDependentConstants {
    pub fn tendon_length0(&self) -> &[f32] {
        &self.tendon_length0
    }

    pub fn cam_pos0(&self) -> &[f32] {
        &self.cam_pos0
    }

    pub fn cam_poscom0(&self) -> &[f32] {
        &self.cam_poscom0
    }

    pub fn cam_mat0(&self) -> &[f32] {
        &self.cam_mat0
    }

    pub fn light_pos0(&self) -> &[f32] {
        &self.light_pos0
    }

    pub fn light_poscom0(&self) -> &[f32] {
        &self.light_poscom0
    }

    pub fn light_dir0(&self) -> &[f32] {
        &self.light_dir0
    }

    pub fn meaninertia(&self) -> f32 {
        self.meaninertia
    }
}

/// 计算 qpos0 依赖常量（上游 `set_const_0` 的子集）。
pub fn set_const_0(input: QposDependentInput<'_>) -> Result<QposDependentConstants, InputError> {
    let nbody = input.xpos.len() / 3;
    if nbody == 0 || !input.xpos.len().is_multiple_of(3) {
        return Err(InputError::InvalidDimension { field: "xpos" });
    }
    check_len("subtree_com", input.subtree_com.len(), nbody * 3)?;
    let ncam = input.cam_bodyid.len();
    for (field, actual, expected) in [
        ("cam_bodyid", input.cam_bodyid.len(), ncam),
        ("cam_targetbodyid", input.cam_targetbodyid.len(), ncam),
        ("cam_xpos", input.cam_xpos.len(), ncam * 3),
        ("cam_xmat", input.cam_xmat.len(), ncam * 9),
    ] {
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    let nlight = input.light_bodyid.len();
    for (field, actual, expected) in [
        ("light_bodyid", input.light_bodyid.len(), nlight),
        ("light_targetbodyid", input.light_targetbodyid.len(), nlight),
        ("light_xpos", input.light_xpos.len(), nlight * 3),
        ("light_xdir", input.light_xdir.len(), nlight * 3),
    ] {
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    let nv = input.m_rownnz.len();
    check_len("m_rowadr", input.m_rowadr.len(), nv)?;
    for row in 0..nv {
        let rownnz = input.m_rownnz[row];
        let rowadr = input.m_rowadr[row];
        if rownnz < 0 || rowadr < 0 || rowadr + rownnz > input.m_values.len() as i32 {
            return Err(InputError::InvalidIndex {
                field: "m_rowadr",
                index: row,
                limit: input.m_values.len(),
            });
        }
    }

    let mut cam_pos0 = Vec::with_capacity(ncam * 3);
    let mut cam_poscom0 = Vec::with_capacity(ncam * 3);
    let mut cam_mat0 = Vec::with_capacity(ncam * 9);
    for cam in 0..ncam {
        let body = body_index(input.cam_bodyid[cam], nbody, "cam_bodyid", cam)?;
        for axis in 0..3 {
            cam_pos0.push(input.cam_xpos[cam * 3 + axis] - input.xpos[body * 3 + axis]);
        }
        let com_body = match input.cam_targetbodyid[cam] {
            -1 => body,
            target => body_index(target, nbody, "cam_targetbodyid", cam)?,
        };
        for axis in 0..3 {
            cam_poscom0
                .push(input.cam_xpos[cam * 3 + axis] - input.subtree_com[com_body * 3 + axis]);
        }
        cam_mat0.extend_from_slice(&input.cam_xmat[cam * 9..cam * 9 + 9]);
    }

    let mut light_pos0 = Vec::with_capacity(nlight * 3);
    let mut light_poscom0 = Vec::with_capacity(nlight * 3);
    let mut light_dir0 = Vec::with_capacity(nlight * 3);
    for light in 0..nlight {
        let body = body_index(input.light_bodyid[light], nbody, "light_bodyid", light)?;
        for axis in 0..3 {
            light_pos0.push(input.light_xpos[light * 3 + axis] - input.xpos[body * 3 + axis]);
        }
        let com_body = match input.light_targetbodyid[light] {
            -1 => body,
            target => body_index(target, nbody, "light_targetbodyid", light)?,
        };
        for axis in 0..3 {
            light_poscom0
                .push(input.light_xpos[light * 3 + axis] - input.subtree_com[com_body * 3 + axis]);
            light_dir0.push(input.light_xdir[light * 3 + axis]);
        }
    }

    let meaninertia = mean_inertia(nv, input.m_rownnz, input.m_rowadr, input.m_values);
    Ok(QposDependentConstants {
        tendon_length0: input.ten_length.to_vec(),
        cam_pos0,
        cam_poscom0,
        cam_mat0,
        light_pos0,
        light_poscom0,
        light_dir0,
        meaninertia,
    })
}

fn mean_inertia(nv: usize, rownnz: &[i32], rowadr: &[i32], values: &[f32]) -> f32 {
    if nv == 0 {
        return 1.0;
    }
    let mut total = 0.0_f32;
    for row in 0..nv {
        let address = rowadr[row] as usize + rownnz[row] as usize - 1;
        total += values[address];
    }
    total / nv as f32
}

fn body_index(
    value: i32,
    nbody: usize,
    field: &'static str,
    index: usize,
) -> Result<usize, InputError> {
    if value < 0 || value as usize >= nbody {
        return Err(InputError::InvalidTopology {
            field,
            index,
            reason: "身体索引越界",
        });
    }
    Ok(value as usize)
}

fn check_len(field: &'static str, actual: usize, expected: usize) -> Result<(), InputError> {
    if actual != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
