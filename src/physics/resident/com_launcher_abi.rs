//! 常驻质心位置（FIELD_BATCH 变体）的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_com.metadata.v1");

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

/// 冻结ABI：元数据、参数、运动学输入与质心输出，后接四个标量。
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
pub(super) struct ResidentComStaticDims {
    pub(super) nb: u32,
    pub(super) nj: u32,
    pub(super) nv: u32,
    pub(super) parameters_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentComStaticDims {
    pub(super) fn new(
        nb: usize,
        nj: usize,
        nv: usize,
        parameters_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nb: value("resident_com_nb", nb)?,
            nj: value("resident_com_nj", nj)?,
            nv: value("resident_com_nv", nv)?,
            parameters_len: value("resident_com_parameters", parameters_len)?,
        };
        if dims.nb == 0 {
            return Err(LauncherError::InvalidArray("常驻质心维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：体父、关节类型、关节体、关节自由度与两对描述符。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nb == 0 {
            return Err(LauncherError::InvalidArray("常驻质心维度无效"));
        }
        u64::from(self.nb)
            .checked_add(
                u64::from(self.nj)
                    .checked_mul(3)
                    .ok_or(LauncherError::Overflow("resident_com_metadata"))?,
            )
            .and_then(|value| value.checked_add(4))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_com_metadata"))
    }

    /// 运动学输入每世界元素数。
    pub(super) fn state_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nb)
            .checked_mul(28)
            .and_then(|value| {
                u64::from(self.nj)
                    .checked_mul(6)
                    .and_then(|joints| value.checked_add(joints))
            })
            .ok_or(LauncherError::Overflow("resident_com_state"))
    }

    /// 质心输出每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nb)
            .checked_mul(14)
            .and_then(|value| {
                u64::from(self.nv)
                    .checked_mul(6)
                    .and_then(|dofs| value.checked_add(dofs))
            })
            .ok_or(LauncherError::Overflow("resident_com_output"))
    }
}

/// 质心发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResidentComDims {
    pub(super) static_dims: ResidentComStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentComDims {
    /// 冻结ABI：刚体数、关节数、自由度与世界数。
    pub(super) fn abi(&self) -> [u32; 4] {
        [
            self.static_dims.nb,
            self.static_dims.nj,
            self.static_dims.nv,
            self.worlds,
        ]
    }
}
