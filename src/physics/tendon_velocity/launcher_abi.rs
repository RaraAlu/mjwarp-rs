//! 肌腱速度两类同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.tendon_velocity.metadata.v1");

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
/// 第三槽是验证状态或计算暂存，内核只写不读。
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
pub(super) struct TendonVelocityStaticDims {
    pub(super) nv: u32,
    pub(super) nt: u32,
    pub(super) nnz: u32,
    pub(super) stride: u32,
}

#[cfg(feature = "cuda-probe")]
impl TendonVelocityStaticDims {
    pub(super) fn new(
        nv: usize,
        nt: usize,
        nnz: usize,
        stride: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        Ok(Self {
            nv: value("tendon_velocity_nv", nv)?,
            nt: value("tendon_velocity_nt", nt)?,
            nnz: value("tendon_velocity_nnz", nnz)?,
            stride: value("tendon_velocity_stride", stride)?,
        })
    }

    /// 验证内核的四个标量：前两位保留零，后两位是世界数与工作量。
    pub(super) fn validate_abi(self, worlds: u32) -> [u32; 4] {
        [0, 0, worlds, worlds]
    }

    /// 计算内核的四个标量：工作量按世界数乘肌腱数。
    pub(super) fn compute_abi(self, worlds: u32) -> Result<[u32; 4], LauncherError> {
        let work = worlds
            .checked_mul(self.nt)
            .ok_or(LauncherError::Overflow("tendon_velocity_work"))?;
        Ok([0, 0, worlds, work])
    }
}

/// 验证发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TendonVelocityValidateDims {
    pub(super) static_dims: TendonVelocityStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl TendonVelocityValidateDims {
    pub(super) fn abi(&self) -> [u32; 4] {
        self.static_dims.validate_abi(self.worlds)
    }
}

/// 计算发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TendonVelocityComputeDims {
    pub(super) static_dims: TendonVelocityStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl TendonVelocityComputeDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        self.static_dims.compute_abi(self.worlds)
    }
}
