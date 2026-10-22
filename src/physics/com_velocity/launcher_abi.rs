//! 空间速度同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.com_velocity.metadata.v1");

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
/// 第三槽是速度结果，内核只写不读。
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
pub(super) struct ComVelocityStaticDims {
    pub(super) nbody: u32,
    pub(super) njnt: u32,
    pub(super) nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl ComVelocityStaticDims {
    pub(super) fn new(nbody: usize, njnt: usize, nv: usize) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nbody: value("com_velocity_nbody", nbody)?,
            njnt: value("com_velocity_njnt", njnt)?,
            nv: value("com_velocity_nv", nv)?,
        };
        if dims.nbody == 0 {
            return Err(LauncherError::InvalidArray("空间速度维度无效"));
        }
        dims.metadata_elements()?;
        dims.output_stride()?;
        Ok(dims)
    }

    /// 元数据元素数：体父、体关节数与关节表。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nbody == 0 {
            return Err(LauncherError::InvalidArray("空间速度维度无效"));
        }
        u64::from(self.nbody)
            .checked_mul(3)
            .and_then(|value| {
                u64::from(self.njnt)
                    .checked_mul(2)
                    .and_then(|joints| value.checked_add(joints))
            })
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("com_velocity_metadata"))
    }

    /// 质心输入每世界元素数。
    pub(super) fn com_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(14)
            .and_then(|value| {
                u64::from(self.nv)
                    .checked_mul(6)
                    .and_then(|dofs| value.checked_add(dofs))
            })
            .ok_or(LauncherError::Overflow("com_velocity_com"))
    }

    /// 速度输出每世界元素数：每体与每自由度各六项。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_add(u64::from(self.nv))
            .and_then(|count| count.checked_mul(6))
            .ok_or(LauncherError::Overflow("com_velocity_output"))
    }
}

/// 空间速度发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ComVelocityDims {
    pub(super) static_dims: ComVelocityStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl ComVelocityDims {
    /// 冻结ABI：刚体数、关节数、自由度与世界数。
    pub(super) fn abi(&self) -> [u32; 4] {
        [
            self.static_dims.nbody,
            self.static_dims.njnt,
            self.static_dims.nv,
            self.worlds,
        ]
    }
}
