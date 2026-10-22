//! 常驻稀疏质量两类同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_sparse.metadata.v1");

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

#[cfg(feature = "cuda-probe")]
const META_SLOT: SlotDesc = SlotDesc {
    index: 0,
    kind: SlotKind::DevicePtr {
        elem: ElemType::I32,
    },
    access: Access::Read,
    role: SlotRole::AddressBearing { proof: PROOF },
};

/// 冻结ABI：四个指针后接四个标量。
/// 第一、二槽指向同一只读源缓冲，第三槽是状态或CSR结果。
#[cfg(feature = "cuda-probe")]
pub(super) const SLOTS: [SlotDesc; 8] = [
    META_SLOT,
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

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SparseResidentStaticDims {
    pub(super) nbody: u32,
    pub(super) n: u32,
    pub(super) nc: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseResidentStaticDims {
    pub(super) fn new(nbody: usize, n: usize, nc: usize) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nbody: value("resident_sparse_nbody", nbody)?,
            n: value("resident_sparse_n", n)?,
            nc: value("resident_sparse_nc", nc)?,
        };
        if dims.nbody == 0 {
            return Err(LauncherError::InvalidArray("常驻稀疏质量维度无效"));
        }
        dims.metadata_elements()?;
        dims.source_stride()?;
        Ok(dims)
    }

    /// 元数据元素数：行数、行地址与列索引，零自由度保留一项。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nbody == 0 {
            return Err(LauncherError::InvalidArray("常驻稀疏质量维度无效"));
        }
        let n = u64::from(self.n);
        let elements = n
            .checked_mul(2)
            .and_then(|value| value.checked_add(u64::from(self.nc)))
            .ok_or(LauncherError::Overflow("resident_sparse_metadata"))?;
        let elements = elements.max(1);
        if elements > i32::MAX as u64 {
            return Err(LauncherError::Overflow("resident_sparse_metadata"));
        }
        Ok(elements)
    }

    /// 源矩阵每世界元素数：十个刚体前缀加稠密矩阵。
    pub(super) fn source_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(10)
            .and_then(|value| {
                u64::from(self.n)
                    .checked_mul(u64::from(self.n))
                    .and_then(|dense| value.checked_add(dense))
            })
            .ok_or(LauncherError::Overflow("resident_sparse_source"))
    }
}

/// 验证发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SparseResidentValidateDims {
    pub(super) static_dims: SparseResidentStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseResidentValidateDims {
    pub(super) fn abi(&self) -> [u32; 4] {
        [
            self.static_dims.nbody,
            self.static_dims.n,
            self.static_dims.nc,
            self.worlds,
        ]
    }
}

/// 采集发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SparseResidentGatherDims {
    pub(super) static_dims: SparseResidentStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl SparseResidentGatherDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        let work = u64::from(self.worlds)
            .checked_mul(u64::from(self.static_dims.n))
            .filter(|&value| value <= u32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_sparse_work"))?;
        Ok([
            self.static_dims.nbody,
            self.static_dims.n,
            self.static_dims.nc,
            work as u32,
        ])
    }
}
