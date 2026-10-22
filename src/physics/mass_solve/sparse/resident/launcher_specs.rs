//! 常驻稀疏质量两类规格、长度与计划。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SLOTS;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentGatherDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentValidateDims;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻稀疏质量世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn checked_buffer(
    base: u64,
    worlds: u64,
    width: u64,
    field: &'static str,
) -> Result<u64, LauncherError> {
    worlds
        .checked_mul(width)
        .and_then(|value| value.checked_add(base))
        .ok_or(LauncherError::Overflow(field))
}

#[cfg(feature = "cuda-probe")]
fn source_elements(
    static_dims: SparseResidentStaticDims,
    worlds: u32,
) -> Result<u64, LauncherError> {
    checked_buffer(
        8,
        world_count(worlds)?,
        static_dims.source_stride()?,
        "resident_sparse_source",
    )
}

/// 验证内核四槽精确元素数；源缓冲同时占用第一、二槽。
#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_resident_validate_lengths(
    dims: &SparseResidentValidateDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let source = source_elements(static_dims, dims.worlds)?;
    let worlds = world_count(dims.worlds)?;
    let status = worlds
        .checked_add(8)
        .ok_or(LauncherError::Overflow("resident_sparse_status"))?;
    Ok([metadata, source, source, status])
}

/// 采集内核四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_resident_gather_lengths(
    dims: &SparseResidentGatherDims,
) -> Result<[u64; 4], LauncherError> {
    dims.abi()?;
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let source = source_elements(static_dims, dims.worlds)?;
    let worlds = world_count(dims.worlds)?;
    let csr = checked_buffer(8, worlds, u64::from(static_dims.nc), "resident_sparse_csr")?;
    Ok([metadata, source, source, csr])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseResidentValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻稀疏验证ABI。
unsafe impl KernelAbi for SparseResidentValidateSpec {
    const NAME: &'static str = "resident_sparse_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SparseResidentValidateSpec {
    type Dims = SparseResidentValidateDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(sparse_resident_validate_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sparse_resident_validate_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseResidentGatherSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结常驻稀疏采集ABI。
unsafe impl KernelAbi for SparseResidentGatherSpec {
    const NAME: &'static str = "resident_sparse_gather";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SparseResidentGatherSpec {
    type Dims = SparseResidentGatherDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(sparse_resident_gather_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sparse_resident_gather_lengths(dims)?;
        let abi = dims.abi()?;
        LaunchDims::new((abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn exact_lengths(bindings: &[SlotBinding], lengths: [u64; 4]) -> Result<(), LauncherError> {
    for (slot, elements) in lengths.into_iter().enumerate() {
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        let required = elements
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("resident_sparse_bytes"))?;
        if binding.span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: binding.span.bytes(),
            });
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_resident_validate_plan(
    dims: &SparseResidentValidateDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, SparseResidentValidateDims>],
) -> Result<(), LauncherError> {
    Kernel::<SparseResidentValidateSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, sparse_resident_validate_lengths(dims)?)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_resident_gather_plan(
    dims: &SparseResidentGatherDims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, SparseResidentGatherDims>],
) -> Result<(), LauncherError> {
    Kernel::<SparseResidentGatherSpec>::new()?.plan(dims, bindings, proofs)?;
    exact_lengths(bindings, sparse_resident_gather_lengths(dims)?)
}
