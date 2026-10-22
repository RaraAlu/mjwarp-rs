//! 借用质量矩阵两类同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.borrowed_mass.metadata.v1");

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
/// 第三槽是验证状态或计算结果，内核只写不读。
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
pub(super) struct BorrowedMassStaticDims {
    pub(super) nbody: u32,
    pub(super) n: u32,
    pub(super) count: u32,
}

#[cfg(feature = "cuda-probe")]
impl BorrowedMassStaticDims {
    pub(super) fn new(nbody: usize, n: usize, count: usize) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nbody: value("borrowed_mass_nbody", nbody)?,
            n: value("borrowed_mass_n", n)?,
            count: value("borrowed_mass_count", count)?,
        };
        if dims.nbody == 0 || dims.n == 0 {
            return Err(LauncherError::InvalidArray("借用质量矩阵维度无效"));
        }
        dims.so()?;
        dims.stride()?;
        Ok(dims)
    }

    /// 每世界矩阵元素数：十个刚体前缀加稠密下三角。
    pub(super) fn stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(10)
            .and_then(|value| {
                u64::from(self.n)
                    .checked_mul(u64::from(self.n))
                    .and_then(|dense| value.checked_add(dense))
            })
            .ok_or(LauncherError::Overflow("borrowed_mass_stride"))
    }

    /// 跳过标志偏移：四个守卫、两个前缀与行偏移、列与地址之后。
    pub(super) fn so(self) -> Result<u32, LauncherError> {
        u64::from(self.n)
            .checked_add(7)
            .and_then(|value| {
                u64::from(self.count)
                    .checked_mul(2)
                    .and_then(|count| value.checked_add(count))
            })
            .filter(|&value| value <= i32::MAX as u64)
            .map(|value| value as u32)
            .ok_or(LauncherError::Overflow("borrowed_mass_metadata"))
    }

    /// 某世界数下的元数据元素数：跳过标志后再接四个守卫。
    pub(super) fn metadata_elements(self, worlds: u32) -> Result<u64, LauncherError> {
        if self.n == 0 || self.nbody == 0 {
            return Err(LauncherError::InvalidArray("借用质量矩阵维度无效"));
        }
        u64::from(worlds)
            .checked_add(u64::from(self.so()?))
            .and_then(|value| value.checked_add(4))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("borrowed_mass_metadata"))
    }

    /// 验证内核的四个标量：刚体数、自由度、跳过偏移与世界数。
    pub(super) fn validate_abi(self, worlds: u32) -> Result<[u32; 4], LauncherError> {
        Ok([self.nbody, self.n, self.so()?, worlds])
    }

    /// 计算内核的四个标量：末位是世界数乘自由度。
    pub(super) fn compute_abi(self, worlds: u32) -> Result<[u32; 4], LauncherError> {
        let work = u64::from(worlds)
            .checked_mul(u64::from(self.n))
            .filter(|&value| value <= u32::MAX as u64)
            .ok_or(LauncherError::Overflow("borrowed_mass_work"))?;
        Ok([self.nbody, self.n, self.so()?, work as u32])
    }
}

/// 验证发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BorrowedMassValidateDims {
    pub(super) static_dims: BorrowedMassStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl BorrowedMassValidateDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        self.static_dims.validate_abi(self.worlds)
    }
}

/// 计算发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BorrowedMassComputeDims {
    pub(super) static_dims: BorrowedMassStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl BorrowedMassComputeDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        self.static_dims.compute_abi(self.worlds)
    }
}
