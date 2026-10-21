//! 混合求解融合元数据与证明校验。

#[cfg(any(feature = "cuda-probe", test))]
use super::fused_spec::mixed_solve_fused_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) const MIXED_SOLVE_FUSED_METADATA_KIND: ProofKind =
    ProofKind("mixed_solve_fused_metadata");

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MixedSolveFusedDims {
    pub(super) abi: [u32; 4],
    pub(super) metadata_elements: u64,
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct StaticMixedSolveFusedMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for StaticMixedSolveFusedMetadata {
    type Dims = [u32; 2];

    const KIND: ProofKind = MIXED_SOLVE_FUSED_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        let [n, stride] = *dims;
        if n == 0
            || stride == 0
            || n > i32::MAX as u32
            || stride > i32::MAX as u32
            || values.len() > i32::MAX as usize
        {
            return Err(LauncherError::InvalidMetadata("混合融合维度无效"));
        }
        let n = usize::try_from(n).map_err(|_| LauncherError::Overflow("mixed_solve_fused_n"))?;
        let stride = usize::try_from(stride)
            .map_err(|_| LauncherError::Overflow("mixed_solve_fused_stride"))?;
        let header_end = 4usize;
        let flags_end = header_end
            .checked_add(n)
            .ok_or(LauncherError::Overflow("mixed_solve_fused_flags"))?;
        let diagonal_end = flags_end
            .checked_add(n)
            .ok_or(LauncherError::Overflow("mixed_solve_fused_diagonal"))?;
        if values.len() <= diagonal_end {
            return Err(LauncherError::InvalidMetadata("混合融合元数据过短"));
        }
        let levels = usize::try_from(values[0])
            .map_err(|_| LauncherError::InvalidMetadata("混合融合层数为负"))?;
        let prefix = usize::try_from(values[1])
            .map_err(|_| LauncherError::InvalidMetadata("混合融合前缀为负"))?;
        let nc = usize::try_from(values[2])
            .map_err(|_| LauncherError::InvalidMetadata("混合融合稀疏跨度为负"))?;
        let small_count = usize::try_from(values[3])
            .map_err(|_| LauncherError::InvalidMetadata("混合融合Small数量为负"))?;
        let expected_stride = prefix
            .checked_add(nc)
            .and_then(|value| value.checked_add(n))
            .ok_or(LauncherError::Overflow("mixed_solve_fused_stride"))?;
        if expected_stride != stride {
            return Err(LauncherError::InvalidMetadata("混合融合因子分区不符"));
        }
        let offsets_end = diagonal_end
            .checked_add(levels)
            .and_then(|value| value.checked_add(1))
            .ok_or(LauncherError::Overflow("mixed_solve_fused_offsets"))?;
        if values.len() < offsets_end {
            return Err(LauncherError::InvalidMetadata("混合融合层偏移越界"));
        }
        let offsets = &values[diagonal_end..offsets_end];
        if offsets.first() != Some(&0)
            || offsets
                .windows(2)
                .any(|pair| pair[0] < 0 || pair[0] >= pair[1])
        {
            return Err(LauncherError::InvalidMetadata("混合融合层偏移无效"));
        }
        let update_count = usize::try_from(*offsets.last().unwrap_or(&0))
            .map_err(|_| LauncherError::InvalidMetadata("混合融合更新数量为负"))?;
        if (levels == 0) != (update_count == 0) || update_count > i32::MAX as usize / 3 {
            return Err(LauncherError::InvalidMetadata("混合融合更新数量无效"));
        }
        let stored_updates = update_count.max(1);
        let updates_end = offsets_end
            .checked_add(
                stored_updates
                    .checked_mul(3)
                    .ok_or(LauncherError::Overflow("mixed_solve_fused_updates"))?,
            )
            .ok_or(LauncherError::Overflow("mixed_solve_fused_updates"))?;
        let expected_len = updates_end
            .checked_add(
                small_count
                    .checked_mul(3)
                    .ok_or(LauncherError::Overflow("mixed_solve_fused_small"))?,
            )
            .ok_or(LauncherError::Overflow("mixed_solve_fused_metadata"))?;
        if values.len() != expected_len {
            return Err(LauncherError::InvalidMetadata("混合融合元数据长度不符"));
        }
        if update_count == 0 && small_count != 0 {
            return Err(LauncherError::InvalidMetadata("混合融合占位更新偏移Small"));
        }

        let flags = &values[header_end..flags_end];
        for level in 0..levels {
            let first = usize::try_from(offsets[level])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合层起点为负"))?;
            let end = usize::try_from(offsets[level + 1])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合层终点为负"))?;
            if end > update_count {
                return Err(LauncherError::InvalidMetadata("混合融合层终点越界"));
            }
            let mut writes = vec![false; n];
            let mut reads = vec![false; n];
            for update in first..end {
                let base = offsets_end
                    .checked_add(
                        update
                            .checked_mul(3)
                            .ok_or(LauncherError::Overflow("mixed_solve_fused_update_index"))?,
                    )
                    .ok_or(LauncherError::Overflow("mixed_solve_fused_update_index"))?;
                let i = usize::try_from(values[base])
                    .map_err(|_| LauncherError::InvalidMetadata("混合融合写索引为负"))?;
                let k = usize::try_from(values[base + 1])
                    .map_err(|_| LauncherError::InvalidMetadata("混合融合读索引为负"))?;
                let address = usize::try_from(values[base + 2])
                    .map_err(|_| LauncherError::InvalidMetadata("混合融合L索引为负"))?;
                if i >= n || k >= n || address >= nc {
                    return Err(LauncherError::InvalidMetadata("混合融合更新索引越界"));
                }
                if i >= k || flags[i] != -1 || flags[k] != -1 {
                    return Err(LauncherError::InvalidMetadata("混合融合更新路由无效"));
                }
                writes[i] = true;
                reads[k] = true;
            }
            if writes.iter().zip(reads).any(|(write, read)| *write && read) {
                return Err(LauncherError::InvalidMetadata("混合融合同层读写交叉"));
            }
        }
        if !flags.contains(&-1) {
            return Err(LauncherError::InvalidMetadata("混合融合缺少Sparse尾部"));
        }
        let mut small_owned = vec![false; n];
        let mut prefix_ranges = Vec::with_capacity(small_count);
        for block in 0..small_count {
            let base = updates_end
                .checked_add(
                    block
                        .checked_mul(3)
                        .ok_or(LauncherError::Overflow("mixed_solve_fused_small_index"))?,
                )
                .ok_or(LauncherError::Overflow("mixed_solve_fused_small_index"))?;
            let start = usize::try_from(values[base])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合Small起点为负"))?;
            let size = usize::try_from(values[base + 1])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合Small大小为负"))?;
            let address = usize::try_from(values[base + 2])
                .map_err(|_| LauncherError::InvalidMetadata("混合融合Small地址为负"))?;
            if !(2..=6).contains(&size) {
                return Err(LauncherError::InvalidMetadata("混合融合Small大小无效"));
            }
            let end = start
                .checked_add(size)
                .filter(|&value| value <= n)
                .ok_or(LauncherError::InvalidMetadata("混合融合Small区越界"))?;
            let address_end = address
                .checked_add(
                    size.checked_mul(size)
                        .ok_or(LauncherError::Overflow("mixed_solve_fused_small_square"))?,
                )
                .filter(|&value| value <= prefix)
                .ok_or(LauncherError::InvalidMetadata("混合融合Small因子越界"))?;
            for dof in start..end {
                if small_owned[dof] || flags[dof] != values[base + 2] {
                    return Err(LauncherError::InvalidMetadata("混合融合Small分区冲突"));
                }
                small_owned[dof] = true;
            }
            prefix_ranges.push((address, address_end));
        }
        prefix_ranges.sort_unstable();
        let mut next_address = 0usize;
        for (start, end) in prefix_ranges {
            if start != next_address {
                return Err(LauncherError::InvalidMetadata("混合融合Small前缀不连续"));
            }
            next_address = end;
        }
        if next_address != prefix {
            return Err(LauncherError::InvalidMetadata("混合融合Small前缀未覆盖"));
        }
        for (dof, &flag) in flags.iter().enumerate() {
            if small_owned[dof] {
                if flag < 0 {
                    return Err(LauncherError::InvalidMetadata("混合融合Small标记无效"));
                }
            } else if flag != -1 && flag != -2 {
                return Err(LauncherError::InvalidMetadata("混合融合路由标记无效"));
            }
        }
        Ok(())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct MixedSolveFusedLaunchMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for MixedSolveFusedLaunchMetadata {
    type Dims = MixedSolveFusedDims;

    const KIND: ProofKind = MIXED_SOLVE_FUSED_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        mixed_solve_fused_lengths(dims)?;
        if u64::try_from(values.len()).ok() != Some(dims.metadata_elements) {
            return Err(LauncherError::InvalidMetadata("混合融合快照长度不符"));
        }
        StaticMixedSolveFusedMetadata::validate(values, &[dims.abi[0], dims.abi[1]])
    }
}
