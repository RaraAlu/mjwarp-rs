//! 质量矩阵乘法两类同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.mass_multiply.metadata.v1");

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
pub(super) struct MassMultiplyStaticDims {
    pub(super) n: u32,
    pub(super) mc: u32,
    pub(super) count: u32,
}

#[cfg(feature = "cuda-probe")]
impl MassMultiplyStaticDims {
    pub(super) fn new(n: usize, mc: usize, count: usize) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            n: value("mass_multiply_n", n)?,
            mc: value("mass_multiply_mc", mc)?,
            count: value("mass_multiply_count", count)?,
        };
        if dims.n == 0 || dims.mc == 0 {
            return Err(LauncherError::InvalidArray("质量矩阵乘法维度无效"));
        }
        dims.so()?;
        Ok(dims)
    }

    /// 元数据初始元素数：行偏移、列索引与地址。
    pub(super) fn so(self) -> Result<u32, LauncherError> {
        u64::from(self.n)
            .checked_add(1)
            .and_then(|value| {
                u64::from(self.count)
                    .checked_mul(2)
                    .and_then(|count| value.checked_add(count))
            })
            .filter(|&value| value <= i32::MAX as u64)
            .map(|value| value as u32)
            .ok_or(LauncherError::Overflow("mass_multiply_metadata"))
    }

    /// 某世界数下的元数据元素数：初始段后接每世界一个跳过标志。
    pub(super) fn metadata_elements(self, worlds: u32) -> Result<u64, LauncherError> {
        if self.n == 0 || self.mc == 0 {
            return Err(LauncherError::InvalidArray("质量矩阵乘法维度无效"));
        }
        u64::from(worlds)
            .checked_add(u64::from(self.so()?))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("mass_multiply_metadata"))
    }

    /// 验证内核的四个标量：维度、跳过偏移与世界数。
    pub(super) fn validate_abi(self, worlds: u32) -> Result<[u32; 4], LauncherError> {
        Ok([self.n, self.mc, self.so()?, worlds])
    }

    /// 计算内核的四个标量：末位是世界数乘自由度。
    pub(super) fn compute_abi(self, worlds: u32) -> Result<[u32; 4], LauncherError> {
        let work = u64::from(worlds)
            .checked_mul(u64::from(self.n))
            .filter(|&value| value <= u32::MAX as u64)
            .ok_or(LauncherError::Overflow("mass_multiply_work"))?;
        Ok([self.n, self.mc, self.so()?, work as u32])
    }
}

/// 验证发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MassMultiplyValidateDims {
    pub(super) static_dims: MassMultiplyStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl MassMultiplyValidateDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        self.static_dims.validate_abi(self.worlds)
    }
}

/// 计算发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MassMultiplyComputeDims {
    pub(super) static_dims: MassMultiplyStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl MassMultiplyComputeDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        self.static_dims.compute_abi(self.worlds)
    }
}
