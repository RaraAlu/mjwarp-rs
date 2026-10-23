//! G21@P7 平滑速度导数切片：`deriv_smooth_vel`（`M - dt * qDeriv`）。
//!
//! 对齐冻结上游 `mujoco_warp._src.derivative.deriv_smooth_vel`（G21；P7，提交
//! `71da24d956…`，Apache-2.0）的被动阻尼贡献子集：
//!
//! ```text
//! qDeriv = -(线性 + 多项式阻尼对速度的导数)   // 仅对角 dof
//! out    = M - dt * qDeriv                    // 写到 M 结构地址
//! ```
//!
//! 上游 `deriv_smooth_vel` 还叠加执行器/肌腱阻尼/流体力等导数项；本片按冻结
//! 输入只复刻关节阻尼（`dof_damping`/`dof_dampingpoly`）的解析导数与 M 结构
//! 组装，不做 `forward`/`fwd_velocity` 等上游链。
//!
//! 未验收边界：执行器 actuation/moment 导数、肌腱阻尼导数、椭球/盒流体力导数、
//! `has_fluid` 分支、`efm`/隐式积分组合、批量世界广播语义、GPU 内核与图执行。

use std::error::Error;
use std::fmt;

/// 上游 `_poly_force_deriv`：阻尼多项式力对速度的导数（`flg_odd = 1`）。
pub fn poly_force_deriv(linear: f32, poly: [f32; 2], x: f32) -> f32 {
    let x_val = x.abs();
    let mut result = linear;
    result += 2.0 * poly[0] * x_val;
    result += 3.0 * poly[1] * x_val * x_val;
    result
}

/// 冻结导数布局：上游 `M_fullm_i`/`M_fullm_j`/`M_elemid` 与 `dof_damping`。
#[derive(Clone, Copy, Debug)]
pub struct DerivativeLayout<'a> {
    pub timestep: f32,
    pub nv: usize,
    /// `d.M` 每世界非零个数（`nC`）。
    pub nnz: usize,
    pub fullm_i: &'a [i32],
    pub fullm_j: &'a [i32],
    /// `(row, col) -> CSR madr`，`-1` 表示非模式内。
    pub elemid: &'a [i32],
    pub damping_batches: usize,
    pub damping: &'a [f32],
    pub damping_poly_batches: usize,
    pub damping_poly: &'a [[f32; 2]],
    /// `DAMPER` 禁用位未置位时为 true（执行阻尼导数项）。
    pub damper_enabled: bool,
}

/// 导数错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DerivativeError {
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
    InvalidBatch {
        field: &'static str,
        batches: usize,
    },
}

impl fmt::Display for DerivativeError {
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
            Self::InvalidBatch { field, batches } => {
                write!(formatter, "批量不合法：{field} = {batches}")
            }
        }
    }
}

impl Error for DerivativeError {}

fn validate(layout: &DerivativeLayout<'_>, nworld: usize) -> Result<(), DerivativeError> {
    for (field, batches) in [
        ("dof_damping", layout.damping_batches),
        ("dof_dampingpoly", layout.damping_poly_batches),
    ] {
        if batches == 0 {
            return Err(DerivativeError::InvalidBatch { field, batches });
        }
    }
    for (field, actual, expected) in [
        ("fullm_i", layout.fullm_i.len(), layout.fullm_j.len()),
        ("elemid", layout.elemid.len(), layout.nv * layout.nv),
        (
            "dof_damping",
            layout.damping.len(),
            layout.damping_batches * layout.nv,
        ),
        (
            "dof_dampingpoly",
            layout.damping_poly.len(),
            layout.damping_poly_batches * layout.nv,
        ),
    ] {
        if actual != expected {
            return Err(DerivativeError::InvalidLength {
                field,
                expected,
                actual,
            });
        }
    }
    let _ = nworld;
    Ok(())
}

/// 上游 `deriv_smooth_vel` 的被动阻尼子集：返回 `M - dt * qDeriv`（`d.M` 布局）。
pub fn deriv_smooth_vel(
    layout: &DerivativeLayout<'_>,
    nworld: usize,
    mass: &[f32],
    qvel: &[f32],
) -> Result<Vec<f32>, DerivativeError> {
    validate(layout, nworld)?;
    let expected = nworld * layout.nnz;
    for (field, actual) in [("d.M", mass.len()), ("qvel", qvel.len())] {
        let limit = if field == "d.M" {
            expected
        } else {
            nworld * layout.nv
        };
        if actual != limit {
            return Err(DerivativeError::InvalidLength {
                field,
                expected: limit,
                actual,
            });
        }
    }
    let mut output = vec![0.0_f32; expected];
    for element in 0..layout.fullm_i.len() {
        let dofi = layout.fullm_i[element];
        let dofj = layout.fullm_j[element];
        if dofi < 0 || dofj < 0 || dofi as usize >= layout.nv || dofj as usize >= layout.nv {
            return Err(DerivativeError::InvalidIndex {
                field: "fullm",
                index: element,
                limit: layout.nv,
            });
        }
        let dofi = dofi as usize;
        let dofj = dofj as usize;
        let madr = layout.elemid[dofi * layout.nv + dofj];
        if madr < 0 {
            continue;
        }
        let madr = madr as usize;
        if madr >= layout.nnz {
            return Err(DerivativeError::InvalidIndex {
                field: "M_elemid",
                index: madr,
                limit: layout.nnz,
            });
        }
        for world in 0..nworld {
            let mut qderiv = 0.0_f32;
            if layout.damper_enabled && dofi == dofj {
                let damping_batch = world % layout.damping_batches;
                let poly_batch = world % layout.damping_poly_batches;
                let damping = layout.damping[damping_batch * layout.nv + dofi];
                let poly = layout.damping_poly[poly_batch * layout.nv + dofi];
                let velocity = qvel[world * layout.nv + dofi];
                qderiv -= poly_force_deriv(damping, poly, velocity);
            }
            qderiv *= layout.timestep;
            output[world * layout.nnz + madr] = mass[world * layout.nnz + madr] - qderiv;
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests;
