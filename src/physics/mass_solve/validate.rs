//! 刚体质量求解元数据校验与打包。

#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::RigidMassSolveDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::metadata::rigid_mass_solve_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn validate_rigid_mass_solve_metadata(
    values: &[i32],
    dims: RigidMassSolveDims,
) -> Result<(), LauncherError> {
    rigid_mass_solve_lengths(&dims)?;
    let count = values
        .first()
        .copied()
        .filter(|&value| value >= 0)
        .ok_or(LauncherError::InvalidMetadata("质量求解块数量无效"))? as usize;
    let expected = count
        .checked_mul(3)
        .and_then(|value| value.checked_add(1))
        .ok_or(LauncherError::Overflow("rigid_mass_solve_metadata"))?;
    if values.len() != expected || values.len() as u64 != dims.metadata_elements {
        return Err(LauncherError::InvalidMetadata("质量求解块表长度无效"));
    }
    let mut cursor = 0u32;
    for block in 0..count {
        let base = 1 + 3 * block;
        let start = u32::try_from(values[base])
            .map_err(|_| LauncherError::InvalidMetadata("质量求解块起点无效"))?;
        let size = u32::try_from(values[base + 1])
            .ok()
            .filter(|&value| value > 0)
            .ok_or(LauncherError::InvalidMetadata("质量求解块长度无效"))?;
        let kind = values[base + 2];
        if !matches!(kind, 0..=2) || (kind == 0 && size != 1) || start != cursor {
            return Err(LauncherError::InvalidMetadata("质量求解块路由无效"));
        }
        cursor = start
            .checked_add(size)
            .filter(|&value| value <= dims.abi[1])
            .ok_or(LauncherError::InvalidMetadata("质量求解块越界"))?;
    }
    if cursor != dims.abi[1] {
        return Err(LauncherError::InvalidMetadata("质量求解块未完整覆盖"));
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn pack_rigid_mass_solve_blocks(
    blocks: &[MassFactorBlock],
    nv: usize,
) -> Result<Vec<i32>, LauncherError> {
    let count = i32::try_from(blocks.len())
        .map_err(|_| LauncherError::Overflow("rigid_mass_solve_block_count"))?;
    let nv_u32 = u32::try_from(nv).map_err(|_| LauncherError::Overflow("rigid_mass_solve_nv"))?;
    let mut packed = Vec::with_capacity(
        blocks
            .len()
            .checked_mul(3)
            .and_then(|value| value.checked_add(1))
            .ok_or(LauncherError::Overflow("rigid_mass_solve_metadata"))?,
    );
    packed.push(count);
    for block in blocks {
        let kind = match block.kind {
            MassFactorKind::Diagonal => 0,
            MassFactorKind::Cholesky => 1,
            MassFactorKind::ReverseLdl => 2,
        };
        packed.extend([
            i32::try_from(block.start)
                .map_err(|_| LauncherError::Overflow("rigid_mass_solve_block_start"))?,
            i32::try_from(block.size)
                .map_err(|_| LauncherError::Overflow("rigid_mass_solve_block_size"))?,
            kind,
        ]);
    }
    let dims = RigidMassSolveDims {
        abi: [0, nv_u32, 1, 1],
        metadata_elements: packed.len() as u64,
    };
    validate_rigid_mass_solve_metadata(&packed, dims)?;
    Ok(packed)
}
