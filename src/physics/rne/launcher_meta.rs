//! RNE元数据长度、校验与四类规格。

#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RNE_BACKWARD_SLOTS;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RNE_CACC_SLOTS;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RNE_CFRC_SLOTS;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RNE_QFRC_SLOTS;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneBackwardDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneCaccDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneCfrcDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneQfrcDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::launcher_abi::RneStaticDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_metadata_elements(nbody: u32, nv: u32) -> Result<u64, LauncherError> {
    if nbody > i32::MAX as u32 || nv > i32::MAX as u32 {
        return Err(LauncherError::Overflow("rne_metadata_signed_dimensions"));
    }
    u64::from(nbody)
        .checked_mul(4)
        .and_then(|value| value.checked_add(u64::from(nv)))
        .and_then(|value| value.checked_add(5))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("rne_metadata_elements"))
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn validate_rne_metadata(
    values: &[i32],
    dims: RneStaticDims,
) -> Result<(), LauncherError> {
    if dims.nbody == 0 || dims.depth >= dims.nbody {
        return Err(LauncherError::InvalidMetadata("RNE静态维度无效"));
    }
    let expected = rne_metadata_elements(dims.nbody, dims.nv)?;
    if dims.metadata_elements != expected
        || u64::try_from(values.len()).ok() != Some(dims.metadata_elements)
    {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度无效"));
    }
    let nbody =
        usize::try_from(dims.nbody).map_err(|_| LauncherError::Overflow("rne_metadata_nbody"))?;
    let nv = usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("rne_metadata_nv"))?;
    if values[0] != dims.nbody as i32 || values[1] != dims.nv as i32 {
        return Err(LauncherError::InvalidMetadata("RNE元数据维度不匹配"));
    }
    if values[2..5]
        .iter()
        .any(|&bits| !f32::from_bits(bits as u32).is_finite())
    {
        return Err(LauncherError::InvalidMetadata("RNE重力不是有限值"));
    }

    let parent_start = 5;
    let count_start = parent_start + nbody;
    let address_start = count_start + nbody;
    let body_start = address_start + nbody;
    let depth_start = body_start + nv;
    let parent = &values[parent_start..count_start];
    let count = &values[count_start..address_start];
    let address = &values[address_start..body_start];
    let body = &values[body_start..depth_start];
    let depth = &values[depth_start..];
    if parent[0] != 0 || count[0] != 0 || address[0] != 0 || depth[0] != 0 {
        return Err(LauncherError::InvalidMetadata("RNE根节点元数据无效"));
    }
    let mut covered = vec![false; nv];
    let mut max_depth = 0usize;
    for index in 1..nbody {
        let parent_index = usize::try_from(parent[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE父节点为负"))?;
        if parent_index >= index {
            return Err(LauncherError::InvalidMetadata("RNE父节点顺序无效"));
        }
        let dof_count = usize::try_from(count[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE自由度数量为负"))?;
        let dof_start = usize::try_from(address[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE自由度地址为负"))?;
        let dof_end = dof_start
            .checked_add(dof_count)
            .filter(|&end| end <= nv)
            .ok_or(LauncherError::InvalidMetadata("RNE自由度范围越界"))?;
        for dof in dof_start..dof_end {
            if body[dof] != index as i32 || covered[dof] {
                return Err(LauncherError::InvalidMetadata("RNE自由度归属无效"));
            }
            covered[dof] = true;
        }
        let expected_depth = depth[parent_index]
            .checked_add(1)
            .ok_or(LauncherError::Overflow("rne_tree_depth"))?;
        if depth[index] != expected_depth {
            return Err(LauncherError::InvalidMetadata("RNE树深度无效"));
        }
        max_depth = max_depth.max(
            usize::try_from(depth[index])
                .map_err(|_| LauncherError::InvalidMetadata("RNE树深度为负"))?,
        );
    }
    if covered.contains(&false) {
        return Err(LauncherError::InvalidMetadata("RNE自由度映射不完整"));
    }
    for &body_index in body {
        let body_index = usize::try_from(body_index)
            .map_err(|_| LauncherError::InvalidMetadata("RNE自由度体索引为负"))?;
        if body_index >= nbody {
            return Err(LauncherError::InvalidMetadata("RNE自由度体索引越界"));
        }
    }
    if max_depth != dims.depth as usize {
        return Err(LauncherError::InvalidMetadata("RNE最大树深度不匹配"));
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_cacc_lengths(dims: &RneCaccDims) -> Result<[u64; 6], LauncherError> {
    let static_dims = dims.static_dims;
    if dims.flag > 1 || dims.worlds == 0 || dims.worlds > u32::MAX - 255 || static_dims.nbody == 0 {
        return Err(LauncherError::InvalidArray("RNE加速度维度无效"));
    }
    if static_dims.metadata_elements != rne_metadata_elements(static_dims.nbody, static_dims.nv)? {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度不匹配"));
    }
    let guarded = |width: u64, field| {
        u64::from(dims.worlds)
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let nbody = u64::from(static_dims.nbody);
    let nv = u64::from(static_dims.nv);
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_cacc_com"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_cacc_velocity"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_cacc_output"))?;
    Ok([
        static_dims.metadata_elements,
        guarded(nv, "rne_cacc_qvel")?,
        guarded(nv, "rne_cacc_qacc")?,
        guarded(com_width, "rne_cacc_com")?,
        guarded(velocity_width, "rne_cacc_velocity")?,
        guarded(output_width, "rne_cacc_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneCaccSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_cacc私有ABI。
unsafe impl KernelAbi for RneCaccSpec {
    const NAME: &'static str = "rne_cacc";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_CACC_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneCaccSpec {
    type Dims = RneCaccDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_cacc_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_cacc_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_cfrc_lengths(dims: &RneCfrcDims) -> Result<[u64; 4], LauncherError> {
    let [x, y, z, worlds] = dims.abi;
    let static_dims = dims.static_dims;
    if [x, y, z] != [0; 3] || worlds == 0 || worlds > u32::MAX - 255 || static_dims.nbody == 0 {
        return Err(LauncherError::InvalidArray("RNE合力维度无效"));
    }
    if static_dims.metadata_elements != rne_metadata_elements(static_dims.nbody, static_dims.nv)? {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度不匹配"));
    }
    let guarded = |width: u64, field| {
        u64::from(worlds)
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let nbody = u64::from(static_dims.nbody);
    let nv = u64::from(static_dims.nv);
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_cfrc_com"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_cfrc_velocity"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_cfrc_output"))?;
    Ok([
        static_dims.metadata_elements,
        guarded(com_width, "rne_cfrc_com")?,
        guarded(velocity_width, "rne_cfrc_velocity")?,
        guarded(output_width, "rne_cfrc_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneCfrcSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_cfrc私有ABI。
unsafe impl KernelAbi for RneCfrcSpec {
    const NAME: &'static str = "rne_cfrc";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_CFRC_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneCfrcSpec {
    type Dims = RneCfrcDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_cfrc_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_cfrc_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_qfrc_lengths(dims: &RneQfrcDims) -> Result<[u64; 4], LauncherError> {
    let [x, y, z, worlds] = dims.abi;
    let static_dims = dims.static_dims;
    if [x, y, z] != [0; 3] || worlds == 0 || worlds > u32::MAX - 255 || static_dims.nbody == 0 {
        return Err(LauncherError::InvalidArray("RNE广义力维度无效"));
    }
    let backward_work = u64::from(worlds)
        .checked_mul(u64::from(static_dims.nbody))
        .ok_or(LauncherError::Overflow("rne_backward_work"))?;
    if backward_work > u64::from(u32::MAX - 255) {
        return Err(LauncherError::Overflow("rne_backward_work"));
    }
    if static_dims.metadata_elements != rne_metadata_elements(static_dims.nbody, static_dims.nv)? {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度不匹配"));
    }
    let guarded = |width: u64, field| {
        u64::from(worlds)
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let nbody = u64::from(static_dims.nbody);
    let nv = u64::from(static_dims.nv);
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_qfrc_com"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_qfrc_velocity"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_qfrc_output"))?;
    Ok([
        static_dims.metadata_elements,
        guarded(com_width, "rne_qfrc_com")?,
        guarded(velocity_width, "rne_qfrc_velocity")?,
        guarded(output_width, "rne_qfrc_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneQfrcSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_qfrc私有ABI。
unsafe impl KernelAbi for RneQfrcSpec {
    const NAME: &'static str = "rne_qfrc";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_QFRC_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneQfrcSpec {
    type Dims = RneQfrcDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_qfrc_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_qfrc_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn rne_backward_lengths(dims: &RneBackwardDims) -> Result<[u64; 4], LauncherError> {
    let [nbody_abi, level, zero, work_abi] = dims.abi;
    let static_dims = dims.static_dims;
    if dims.worlds == 0
        || static_dims.nbody == 0
        || nbody_abi != static_dims.nbody
        || level == 0
        || level > static_dims.depth
        || zero != 0
    {
        return Err(LauncherError::InvalidArray("RNE反向层级维度无效"));
    }
    if static_dims.metadata_elements != rne_metadata_elements(static_dims.nbody, static_dims.nv)? {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度不匹配"));
    }
    let work = u64::from(dims.worlds)
        .checked_mul(u64::from(static_dims.nbody))
        .ok_or(LauncherError::Overflow("rne_backward_work"))?;
    if work > u64::from(u32::MAX - 255) {
        return Err(LauncherError::Overflow("rne_backward_work"));
    }
    if u64::from(work_abi) != work {
        return Err(LauncherError::InvalidArray("RNE反向工作量无效"));
    }
    let guarded = |width: u64, field| {
        u64::from(dims.worlds)
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let nbody = u64::from(static_dims.nbody);
    let nv = u64::from(static_dims.nv);
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_backward_com"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_backward_velocity"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_backward_output"))?;
    Ok([
        static_dims.metadata_elements,
        guarded(com_width, "rne_backward_com")?,
        guarded(velocity_width, "rne_backward_velocity")?,
        guarded(output_width, "rne_backward_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct RneBackwardSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_backward私有ABI。
unsafe impl KernelAbi for RneBackwardSpec {
    const NAME: &'static str = "rne_backward";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_BACKWARD_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneBackwardSpec {
    type Dims = RneBackwardDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_backward_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_backward_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}
