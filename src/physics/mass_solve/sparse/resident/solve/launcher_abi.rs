//! 常驻稀疏求解两入口的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
#[cfg(feature = "cuda-probe")]
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, Checked, ElemType, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotDesc,
    SlotKind, SlotRole,
};

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.sparse_solve.metadata.v1");

#[cfg(feature = "cuda-probe")]
const fn read_slot(index: usize, elem: ElemType) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::DevicePtr { elem },
        access: Access::Read,
        role: SlotRole::Value,
    }
}

#[cfg(feature = "cuda-probe")]
const fn scalar_slot(index: usize) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    }
}

/// 冻结ABI：元数据、RHS、因子与输出，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing { proof: PROOF },
    },
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::F32),
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    scalar_slot(4),
    scalar_slot(5),
    scalar_slot(6),
    scalar_slot(7),
];

/// 入口种类；决定发射网格与输出宽度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SparseSolveEntry {
    Validate,
    Fused,
}

/// 与发射维度无关的静态元数据形状。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SparseSolveStaticDims {
    pub(super) n: u32,
    pub(super) nc: u32,
    pub(super) levels: u32,
    pub(super) updates: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseSolveStaticDims {
    /// 由自由度、非零计数、层级与更新计数构造。
    pub(super) fn new(
        n: usize,
        nc: usize,
        levels: usize,
        updates: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            n: value("sparse_solve_n", n)?,
            nc: value("sparse_solve_nc", nc)?,
            levels: value("sparse_solve_levels", levels)?,
            updates: value("sparse_solve_updates", updates)?,
        };
        if dims.n == 0 || dims.nc == 0 {
            return Err(LauncherError::InvalidArray("稀疏求解维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：层级计数、块地址、对角槽、层偏移与更新三元组。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.n)
            .checked_mul(2)
            .and_then(|value| value.checked_add(u64::from(self.levels)))
            .and_then(|value| value.checked_add(1))
            .and_then(|value| {
                u64::from(self.updates)
                    .checked_mul(3)
                    .and_then(|updates| value.checked_add(updates))
            })
            .and_then(|value| value.checked_add(1))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("sparse_solve_metadata"))
    }

    /// 因子缓冲每世界元素数。
    pub(super) fn factor_stride(self) -> u64 {
        u64::from(self.nc) + u64::from(self.n)
    }

    /// RHS 每世界元素数。
    pub(super) fn rhs_stride(self, rhs_count: u32) -> Result<u64, LauncherError> {
        u64::from(self.n)
            .checked_mul(u64::from(rhs_count))
            .ok_or(LauncherError::Overflow("sparse_solve_rhs"))
    }
}

/// 稀疏求解发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SparseSolveDims {
    pub(super) static_dims: SparseSolveStaticDims,
    pub(super) worlds: u32,
    pub(super) rhs_count: u32,
    pub(super) entry: SparseSolveEntry,
}

#[cfg(feature = "cuda-probe")]
impl SparseSolveDims {
    /// 冻结ABI：[n, nc, nr, worlds]。
    pub(super) fn abi(self) -> [u32; 4] {
        [
            self.static_dims.n,
            self.static_dims.nc,
            self.rhs_count,
            self.worlds,
        ]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("稀疏求解世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn rhs_count(dims: &SparseSolveDims) -> Result<u32, LauncherError> {
    if dims.rhs_count == 0 || u64::from(dims.worlds) * u64::from(dims.rhs_count) > i32::MAX as u64 {
        return Err(LauncherError::InvalidArray("稀疏求解右端项数无效"));
    }
    Ok(dims.rhs_count)
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("sparse_solve_guarded"))
}

/// 四槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_solve_lengths(dims: &SparseSolveDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    let rhs_count = rhs_count(dims)?;
    let metadata = static_dims.metadata_elements()?;
    let rhs = guarded(dims.worlds, static_dims.rhs_stride(rhs_count)?)?;
    let factor = guarded(dims.worlds, static_dims.factor_stride())?;
    Ok(match dims.entry {
        SparseSolveEntry::Validate => [metadata, rhs, factor, guarded(dims.worlds, 1)?],
        SparseSolveEntry::Fused => [metadata, rhs, factor, rhs],
    })
}

/// 校验元数据：层偏移、对角槽与更新三元组边界。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_sparse_solve_metadata(
    values: &[i32],
    dims: SparseSolveStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("稀疏求解元数据长度无效"));
    }
    let n = usize::try_from(dims.n).map_err(|_| LauncherError::Overflow("sparse_solve_n"))?;
    let levels =
        usize::try_from(dims.levels).map_err(|_| LauncherError::Overflow("sparse_solve_levels"))?;
    let updates = usize::try_from(dims.updates)
        .map_err(|_| LauncherError::Overflow("sparse_solve_updates"))?;
    let stride =
        n + usize::try_from(dims.nc).map_err(|_| LauncherError::Overflow("sparse_solve_nc"))?;
    let declared = usize::try_from(values[0])
        .map_err(|_| LauncherError::InvalidMetadata("稀疏求解层级为负"))?;
    if declared != levels {
        return Err(LauncherError::InvalidMetadata("稀疏求解层级计数不符"));
    }
    // 对角槽是因子缓冲的线性索引。
    let diag = &values[1 + n..1 + 2 * n];
    if diag.iter().any(|&slot| slot < 0 || slot as usize >= stride) {
        return Err(LauncherError::InvalidMetadata("稀疏求解对角槽越界"));
    }
    let offsets = &values[1 + 2 * n..1 + 2 * n + levels + 1];
    if offsets[0] != 0 {
        return Err(LauncherError::InvalidMetadata("稀疏求解层偏移起点非零"));
    }
    if offsets.windows(2).any(|pair| pair[0] > pair[1]) {
        return Err(LauncherError::InvalidMetadata("稀疏求解层偏移非单调"));
    }
    if offsets[levels] < 0 || offsets[levels] as usize > updates {
        return Err(LauncherError::InvalidMetadata("稀疏求解层偏移终点越界"));
    }
    // 层偏移之后的尾项不被内核读取，只校验实际使用的三元组。
    let used = offsets[levels] as usize;
    let triples = &values[1 + 2 * n + levels + 1..1 + 2 * n + levels + 1 + 3 * used];
    for index in 0..used {
        let target = triples[3 * index];
        let source = triples[3 * index + 1];
        let slot = triples[3 * index + 2];
        if target < 0
            || target as usize >= n
            || source < 0
            || source as usize >= n
            || slot < 0
            || slot as usize >= stride
        {
            return Err(LauncherError::InvalidMetadata("稀疏求解更新三元组越界"));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SparseSolveMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SparseSolveMetadata {
    type Dims = SparseSolveDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sparse_solve_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sparse_solve_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "sparse_solve_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "sparse_solve_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "sparse_solve_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "sparse_solve_abi",
        }
        .into(),
    }
}

/// 计划级元数据缓冲；构造时上传一次，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct SparseSolveMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: SparseSolveStaticDims,
    revision: MetadataRevision,
    checked: Checked<SparseSolveMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SparseSolveMetadataBuffer {
    /// 由计划构造期冻结的宿主镜像上传并建立证明。
    pub(super) fn from_values(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: SparseSolveStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = Self::base_dims(static_dims);
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<SparseSolveMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(sparse_solve_launch_error)?;
        Ok(Self {
            buffer,
            values,
            static_dims,
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn static_dims(&self) -> SparseSolveStaticDims {
        self.static_dims
    }

    fn base_dims(static_dims: SparseSolveStaticDims) -> SparseSolveDims {
        SparseSolveDims {
            static_dims,
            worlds: 1,
            rhs_count: 1,
            entry: SparseSolveEntry::Validate,
        }
    }

    /// 发射证明：先核对证明未失效，再按实际维度重建。
    pub(super) fn proof(
        &self,
        dims: SparseSolveDims,
    ) -> Result<Checked<SparseSolveMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &Self::base_dims(self.static_dims))
            .map_err(sparse_solve_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际维度只影响跨度。
        unsafe {
            Checked::<SparseSolveMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_solve_launch_error)
    }
}
