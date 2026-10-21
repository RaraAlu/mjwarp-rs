//! 质量求解因子格式与独立树分块。

#[cfg(any(feature = "cuda-probe", test))]
use super::*;

/// 每个独立运动树的因子格式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MassFactorKind {
    /// 单自由度只存质量倒数，因子区域为零。
    Diagonal,
    /// 上三角U满足M = U^T U；倒数区域为零。
    Cholesky,
    /// 下三角L与对角D满足M = L^T D L。
    ReverseLdl,
}

/// 自由度区间共用同一因子格式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MassFactorBlock {
    pub start: usize,
    pub size: usize,
    pub kind: MassFactorKind,
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn factor_blocks(parents: &[i32]) -> Vec<MassFactorBlock> {
    let mut blocks = Vec::<MassFactorBlock>::new();
    for (dof, &parent) in parents.iter().enumerate() {
        if parent < 0 {
            blocks.push(MassFactorBlock {
                start: dof,
                size: 0,
                kind: MassFactorKind::Diagonal,
            });
        }
        // 模型校验保证先祖编号更小。
        // 后续区间检查再拒绝交错树。
        blocks.last_mut().expect("checked DOF tree").size += 1;
    }
    for block in &mut blocks {
        block.kind = match block.size {
            1 => MassFactorKind::Diagonal,
            2..=64 => MassFactorKind::Cholesky,
            _ => MassFactorKind::ReverseLdl,
        };
    }
    blocks
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn check_block_intervals(
    parents: &[i32],
    blocks: &[MassFactorBlock],
) -> Result<(), InputError> {
    for block in blocks {
        if parents[block.start..block.start + block.size]
            .iter()
            .any(|&p| p >= 0 && (p as usize) < block.start)
        {
            return Err(InputError::InvalidDimension {
                field: "mass_solve_interleaved_trees",
            });
        }
    }
    Ok(())
}
