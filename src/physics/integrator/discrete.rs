//! G18@P6 离散积分切片：DISCRETE 步进映射与 `discrete_acc` 恒等分支。
//!
//! 对齐冻结上游 `mujoco_warp._src.forward`（G18；P6，提交 `71da24d956…`，
//! Apache-2.0）：
//!
//! - `discrete(m, d)` = `_advance(m, d, d.qacc)`：先用 qacc 推进速度，再用新速度
//!   推进位置（quat 积分），`time += timestep`；与半隐式 Euler 的直接分支相同。
//! - `inverse.discrete_acc`：DISCRETE 积分器（以及禁用 `EULERDAMP` 的 EULER）为
//!   恒等映射；EULER 带阻尼时需要 `(M + dt·diag(damping))` 有效度量的求解分支。
//!
//! 未验收边界：`(M + dt·diag(damping))` 有效度量求解分支、`IMPLICITFAST` 的
//! `deriv_smooth_vel` + factor-solve、`_launch_discrete_free_gyro` 自由体陀螺修正、
//! 约束求解（`solver.solve`）与约束一致性、`check_discrete` 配置校验、GPU 内核。

use super::{IntegratorError, IntegratorState, IntegratorStep, JointLayout, euler};

/// `discrete_acc` 的映射分支。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscreteAccBranch {
    /// DISCRETE 积分器或禁用 `EULERDAMP` 的 EULER：`qacc` 直通。
    Identity,
    /// EULER 带阻尼：需要有效度量 `(M + dt·diag(damping))` 的求解分支。
    DampedEuler,
}

/// 上游 `forward.discrete`：DISCRETE 单步映射（`_advance(m, d, d.qacc)`）。
pub fn discrete(
    layout: &JointLayout<'_>,
    state: IntegratorState<'_>,
    qacc: &[f32],
    timestep: f32,
    time: f32,
) -> Result<IntegratorStep, IntegratorError> {
    euler(layout, state, qacc, timestep, time)
}

/// 上游 `inverse.discrete_acc` 的宿主侧可用分支。
///
/// 恒等分支返回 `qacc` 的拷贝（与上游 `wp.copy(qacc, d.qacc)` 等价）；
/// 有效度量求解分支未实现，显式返回 `UnsupportedDiscreteAcc`。
pub fn discrete_acc(branch: DiscreteAccBranch, qacc: &[f32]) -> Result<Vec<f32>, IntegratorError> {
    match branch {
        DiscreteAccBranch::Identity => Ok(qacc.to_vec()),
        DiscreteAccBranch::DampedEuler => Err(IntegratorError::UnsupportedDiscreteAcc),
    }
}

#[cfg(test)]
mod tests;
