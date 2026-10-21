//! G01公开入口与调用方状态。

use super::checks::check_size;
use super::*;

impl G01SleepState {
    pub(super) fn validate(&self, ntree: usize, nbody: usize, nv: usize) -> Result<(), InputError> {
        SleepTreeState {
            tree_asleep: self.tree_asleep.clone(),
            nbody_awake: self.nbody_awake,
            nv_awake: self.nv_awake,
        }
        .validate(ntree, nbody, nv)?;
        check_size("tree_awake", ntree, self.tree_awake.len())?;
        if self.ntree_awake < 0 || self.ntree_awake as usize > ntree {
            return Err(InputError::InvalidDimension {
                field: "ntree_awake",
            });
        }
        for (index, &value) in self.tree_awake.iter().enumerate() {
            if value != 0 && value != 1 {
                return Err(InputError::InvalidTopology {
                    field: "tree_awake",
                    index,
                    reason: "noncanonical_boolean",
                });
            }
        }
        Ok(())
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn packed(&self) -> Vec<i32> {
        let mut result = self.tree_asleep.clone();
        result.extend_from_slice(&self.tree_awake);
        result.extend_from_slice(&[self.ntree_awake, self.nbody_awake, self.nv_awake]);
        result
    }
}

/// 完整G01入口不要求新鲜度票据。
/// 调用方提供一致的模型与状态。
/// 本入口不执行碰撞或质量阶段。
pub fn fwd_kinematics(
    plan: &KinematicsPlan,
    data: &mut KinematicsData,
) -> Result<(), TransferError> {
    kinematics(plan, data)?;
    com_pos(plan, data)?;
    camlight(plan, data)?;
    flex(plan, data)?;
    tendon(plan, data)?;
    plan.update_tendon_wake(data)
}

/// 重算刚体与附着位姿。
/// 保留世界位姿与静态几何缓存。
pub fn kinematics(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_rigid(data)?;
    plan.update_attached(data)
}

/// 读取已有刚体设备字段。
/// 调用方负责输入的物理一致性。
pub fn com_pos(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_com(data)
}

/// 读取已有刚体与质心字段。
/// 本入口不隐式重算前置阶段。
pub fn camlight(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_camlight(data)
}

/// 重算柔体位置、边与壳体面。
/// 读取已有刚体、质心与qvel。
/// 先清除Hessian有效标志。
pub fn flex(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_flex_positions(data)?;
    plan.update_flex_edges(data)?;
    plan.update_flex_faces(data)
}

/// 重算全局肌腱与包裹字段。
/// 读取已有附着、质心与qpos。
/// 本入口不执行肌腱唤醒。
pub fn tendon(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_tendons(data)
}
