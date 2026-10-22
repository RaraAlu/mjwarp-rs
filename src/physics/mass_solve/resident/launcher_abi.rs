//! 常驻质量分解两类同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_mass.metadata.v1");

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
/// 第三槽是分解或回代结果，内核只写不读。
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
pub(super) struct ResidentMassStaticDims {
    pub(super) nbody: u32,
    pub(super) n: u32,
    pub(super) count: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMassStaticDims {
    pub(super) fn new(nbody: usize, n: usize, count: usize) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nbody: value("resident_mass_nbody", nbody)?,
            n: value("resident_mass_n", n)?,
            count: value("resident_mass_count", count)?,
        };
        if dims.nbody == 0 {
            return Err(LauncherError::InvalidArray("常驻质量分解维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 块表元素数：块数加三元组。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nbody == 0 {
            return Err(LauncherError::InvalidArray("常驻质量分解维度无效"));
        }
        u64::from(self.count)
            .checked_mul(3)
            .and_then(|value| value.checked_add(1))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_mass_metadata"))
    }

    /// 因子区每世界元素数：状态三标量、下三角与对角逆。
    pub(super) fn factor_stride(self) -> Result<u64, LauncherError> {
        let n = u64::from(self.n);
        n.checked_mul(n)
            .and_then(|square| square.checked_add(n.checked_mul(2)?))
            .and_then(|value| value.checked_add(3))
            .ok_or(LauncherError::Overflow("resident_mass_factor"))
    }

    /// 矩阵源每世界元素数：十个刚体前缀加稠密矩阵。
    pub(super) fn matrix_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(10)
            .and_then(|value| {
                u64::from(self.n)
                    .checked_mul(u64::from(self.n))
                    .and_then(|dense| value.checked_add(dense))
            })
            .ok_or(LauncherError::Overflow("resident_mass_matrix"))
    }

    /// 解区每世界元素数：状态三标量、下三角、对角逆与右端项。
    pub(super) fn solve_stride(self, rhs_count: u32) -> Result<u64, LauncherError> {
        let n = u64::from(self.n);
        let base = n
            .checked_mul(n)
            .and_then(|square| square.checked_add(n))
            .and_then(|value| value.checked_add(3))
            .ok_or(LauncherError::Overflow("resident_mass_solve"))?;
        let rhs = u64::from(rhs_count)
            .checked_mul(n)
            .ok_or(LauncherError::Overflow("resident_mass_solve"))?;
        base.checked_add(rhs)
            .ok_or(LauncherError::Overflow("resident_mass_solve"))
    }
}

/// 分解发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResidentMassFactorDims {
    pub(super) static_dims: ResidentMassStaticDims,
    pub(super) worlds: u32,
    pub(super) rhs_count: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMassFactorDims {
    /// 冻结ABI：分解内核的右端项计数固定为一。
    pub(super) fn abi(&self) -> [u32; 4] {
        [self.static_dims.nbody, self.static_dims.n, 1, self.worlds]
    }
}

/// 回代发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResidentMassSolveDims {
    pub(super) static_dims: ResidentMassStaticDims,
    pub(super) worlds: u32,
    pub(super) rhs_count: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMassSolveDims {
    /// 冻结ABI：右端项计数按实际布局。
    pub(super) fn abi(&self) -> [u32; 4] {
        [
            self.static_dims.nbody,
            self.static_dims.n,
            self.rhs_count,
            self.worlds,
        ]
    }
}
