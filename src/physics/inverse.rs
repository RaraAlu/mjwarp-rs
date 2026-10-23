//! G20@P7 逆向动力学切片：`qfrc_inverse` 组装与 CSR 质量矩阵乘法。
//!
//! 对齐冻结上游 `mujoco_warp._src.inverse`（G20；P7，提交 `71da24d956…`，
//! Apache-2.0）的连续时间分支：
//!
//! ```text
//! qfrc_inverse = qfrc_bias + M @ qacc - qfrc_passive - qfrc_constraint
//! ```
//!
//! `M` 的取值按上游 `d.M`（CSR 行存储）冻结，乘向量使用上游稀疏乘法视角的
//! `M_mulm_rowadr`/`M_mulm_col`/`M_mulm_madr` 收集布局（由 `support.mul_m`
//! 内核直接使用）；`qfrc_bias`/`qfrc_passive`/`qfrc_constraint` 与 `qacc` 同样
//! 冻结，本片不做 `fwd_position`/`fwd_velocity`/`rne`/传感器等上游链。
//!
//! 未验收边界：DISCRETE/IMPLICITFAST 的 `discrete_acc` 与 `_qfrc_inverse_discrete`、
//! `inv_constraint` 逆约束求解、dense 紧凑块分支、`efm_c/efm_ca` 项、GJ/睡眠/岛屿、
//! GPU 内核与图执行。

use std::error::Error;
use std::fmt;

/// 质量矩阵：`d.M` 值 + 上游稀疏乘法（mulm）收集布局。
#[derive(Clone, Copy, Debug)]
pub struct SparseMass<'a> {
    pub nv: usize,
    /// `d.M` 每世界非零个数（`nC`）。
    pub nnz: usize,
    /// `M_mulm_*` 收集项个数。
    pub mulm_nnz: usize,
    /// `M_mulm_rowadr`（`nv + 1` 行指针）。
    pub mulm_rowadr: &'a [i32],
    /// `M_mulm_col`（收集列号）。
    pub mulm_col: &'a [i32],
    /// `M_mulm_madr`（收集矩阵地址）。
    pub mulm_madr: &'a [i32],
    /// `d.M` 值（`nworld × nnz` 展平）。
    pub values: &'a [f32],
}

/// 逆动力学错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InverseError {
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

impl fmt::Display for InverseError {
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

impl Error for InverseError {}

fn validate(mass: &SparseMass<'_>, nworld: usize) -> Result<(), InverseError> {
    for (field, actual, expected) in [
        ("M_mulm_rowadr", mass.mulm_rowadr.len(), mass.nv + 1),
        ("M_mulm_col", mass.mulm_col.len(), mass.mulm_nnz),
        ("M_mulm_madr", mass.mulm_madr.len(), mass.mulm_nnz),
        ("M", mass.values.len(), nworld * mass.nnz),
    ] {
        if actual != expected {
            return Err(InverseError::InvalidLength {
                field,
                expected,
                actual,
            });
        }
    }
    for row in 0..mass.nv {
        let start = mass.mulm_rowadr[row];
        let end = mass.mulm_rowadr[row + 1];
        if start < 0 || end < start || end as usize > mass.mulm_nnz {
            return Err(InverseError::InvalidIndex {
                field: "M_mulm_rowadr",
                index: row,
                limit: mass.mulm_nnz,
            });
        }
    }
    for item in 0..mass.mulm_nnz {
        if mass.mulm_col[item] < 0 || mass.mulm_col[item] as usize >= mass.nv {
            return Err(InverseError::InvalidIndex {
                field: "M_mulm_col",
                index: item,
                limit: mass.nv,
            });
        }
        if mass.mulm_madr[item] < 0 || mass.mulm_madr[item] as usize >= mass.nnz {
            return Err(InverseError::InvalidIndex {
                field: "M_mulm_madr",
                index: item,
                limit: mass.nnz,
            });
        }
    }
    Ok(())
}

/// 上游 `support.mul_m`：逐 DOF 收集 `Ma = M @ qacc`。
pub fn mul_m(mass: &SparseMass<'_>, nworld: usize, qacc: &[f32]) -> Result<Vec<f32>, InverseError> {
    validate(mass, nworld)?;
    let expected = nworld * mass.nv;
    if qacc.len() != expected {
        return Err(InverseError::InvalidLength {
            field: "qacc",
            expected,
            actual: qacc.len(),
        });
    }
    let mut output = vec![0.0_f32; expected];
    for world in 0..nworld {
        let value_base = world * mass.nnz;
        let dof_base = world * mass.nv;
        for dof in 0..mass.nv {
            let start = mass.mulm_rowadr[dof] as usize;
            let end = mass.mulm_rowadr[dof + 1] as usize;
            let mut sum = 0.0_f32;
            for item in start..end {
                // 上游 CUDA 内核的 `acc += M*vec` 可能被收缩为 FMA，宿主侧保留
                // 逐步乘/加（与上游最多 1–2 ulp 舍入差，冻结容差 1e-5）。
                sum += mass.values[value_base + mass.mulm_madr[item] as usize]
                    * qacc[dof_base + mass.mulm_col[item] as usize];
            }
            output[dof_base + dof] = sum;
        }
    }
    Ok(output)
}

/// 连续时间逆向动力学：`qfrc_bias + M@qacc - qfrc_passive - qfrc_constraint`。
pub fn inverse_dynamics(
    mass: &SparseMass<'_>,
    nworld: usize,
    qacc: &[f32],
    qfrc_bias: &[f32],
    qfrc_passive: &[f32],
    qfrc_constraint: &[f32],
) -> Result<Vec<f32>, InverseError> {
    let expected = nworld * mass.nv;
    for (field, values) in [
        ("qfrc_bias", qfrc_bias),
        ("qfrc_passive", qfrc_passive),
        ("qfrc_constraint", qfrc_constraint),
    ] {
        if values.len() != expected {
            return Err(InverseError::InvalidLength {
                field,
                expected,
                actual: values.len(),
            });
        }
    }
    let ma = mul_m(mass, nworld, qacc)?;
    let mut output = vec![0.0_f32; expected];
    for index in 0..expected {
        let mut value = qfrc_bias[index];
        value += ma[index];
        value -= qfrc_passive[index];
        value -= qfrc_constraint[index];
        output[index] = value;
    }
    Ok(output)
}

#[cfg(test)]
mod tests;
