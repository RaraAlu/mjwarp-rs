//! 重力补偿四类同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.gravity_compensation.metadata.v1");

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

/// 清零内核：元数据、参数、未用状态与输出。
#[cfg(feature = "cuda-probe")]
pub(super) const CLEAR_SLOTS: [SlotDesc; 8] = [
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

/// 五缓冲内核：元数据、参数、刚体、质心与状态/输出。
#[cfg(feature = "cuda-probe")]
pub(super) const FIVE_SLOTS: [SlotDesc; 9] = [
    META_SLOT,
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::F32),
    read_slot(3, ElemType::F32),
    SlotDesc {
        index: 4,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    scalar_slot(5),
    scalar_slot(6),
    scalar_slot(7),
    scalar_slot(8),
];

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GravityStaticDims {
    pub(super) nb: u32,
    pub(super) nj: u32,
    pub(super) nv: u32,
    pub(super) parameter_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl GravityStaticDims {
    pub(super) fn new(
        nb: usize,
        nj: usize,
        nv: usize,
        parameter_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nb: value("gravity_compensation_nb", nb)?,
            nj: value("gravity_compensation_nj", nj)?,
            nv: value("gravity_compensation_nv", nv)?,
            parameter_len: value("gravity_compensation_parameters", parameter_len)?,
        };
        if dims.nb < 2 || dims.nv == 0 {
            return Err(LauncherError::InvalidArray("重力补偿维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：八守卫、根表、祖先表、自由度关节与驱动器标志。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nb < 2 || self.nv == 0 {
            return Err(LauncherError::InvalidArray("重力补偿维度无效"));
        }
        let nb = u64::from(self.nb);
        let nv = u64::from(self.nv);
        nb.checked_mul(nv)
            .and_then(|ancestry| {
                nb.checked_add(ancestry)
                    .and_then(|v| v.checked_add(nv))
                    .and_then(|v| v.checked_add(u64::from(self.nj)))
                    .and_then(|v| v.checked_add(8))
            })
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("gravity_compensation_metadata"))
    }

    /// 参数区元素数：四守卫后接分段参数，尾部四守卫。
    pub(super) fn parameters_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.parameter_len)
            .checked_add(8)
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("gravity_compensation_parameters"))
    }

    /// 刚体输入每世界元素数。
    pub(super) fn rigid_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nb)
            .checked_mul(28)
            .and_then(|value| {
                u64::from(self.nj)
                    .checked_mul(6)
                    .and_then(|joints| value.checked_add(joints))
            })
            .ok_or(LauncherError::Overflow("gravity_compensation_rigid"))
    }

    /// 质心输入每世界元素数。
    pub(super) fn com_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nb)
            .checked_mul(14)
            .and_then(|value| {
                u64::from(self.nv)
                    .checked_mul(6)
                    .and_then(|dofs| value.checked_add(dofs))
            })
            .ok_or(LauncherError::Overflow("gravity_compensation_com"))
    }
}

/// 清零发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GravityClearDims {
    pub(super) static_dims: GravityStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl GravityClearDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        let work = u64::from(self.worlds)
            .checked_mul(u64::from(self.static_dims.nv))
            .filter(|&value| value <= (u32::MAX - 255) as u64)
            .ok_or(LauncherError::Overflow("gravity_compensation_work"))?;
        Ok([
            self.static_dims.nb,
            self.static_dims.nv,
            self.worlds,
            work as u32,
        ])
    }
}

/// 验证发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GravityValidateDims {
    pub(super) static_dims: GravityStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl GravityValidateDims {
    pub(super) fn abi(&self) -> [u32; 4] {
        [
            self.static_dims.nb,
            self.static_dims.nv,
            self.worlds,
            self.worlds,
        ]
    }
}

/// 计算发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GravityComputeDims {
    pub(super) static_dims: GravityStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl GravityComputeDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        let work = u64::from(self.worlds)
            .checked_mul(u64::from(self.static_dims.nb - 1))
            .and_then(|value| value.checked_mul(u64::from(self.static_dims.nv)))
            .filter(|&value| value <= (u32::MAX - 255) as u64)
            .ok_or(LauncherError::Overflow("gravity_compensation_work"))?;
        Ok([
            self.static_dims.nb,
            self.static_dims.nv,
            self.worlds,
            work as u32,
        ])
    }
}

/// 合并发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GravitySumDims {
    pub(super) static_dims: GravityStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl GravitySumDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        let work = u64::from(self.worlds)
            .checked_mul(u64::from(self.static_dims.nv))
            .filter(|&value| value <= (u32::MAX - 255) as u64)
            .ok_or(LauncherError::Overflow("gravity_compensation_work"))?;
        Ok([
            self.static_dims.nb,
            self.static_dims.nv,
            self.worlds,
            work as u32,
        ])
    }
}
