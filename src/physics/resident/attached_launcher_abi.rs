//! 常驻附着运动学（FIELD_BATCH 变体）的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_attached.metadata.v1");

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

/// 冻结ABI：元数据、参数、刚体输入与附着输出，后接四个标量。
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
pub(super) struct ResidentAttachedStaticDims {
    pub(super) nb: u32,
    pub(super) nj: u32,
    pub(super) ng: u32,
    pub(super) ns: u32,
    pub(super) parameters_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentAttachedStaticDims {
    pub(super) fn new(
        nb: usize,
        nj: usize,
        ng: usize,
        ns: usize,
        parameters_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nb: value("resident_attached_nb", nb)?,
            nj: value("resident_attached_nj", nj)?,
            ng: value("resident_attached_ng", ng)?,
            ns: value("resident_attached_ns", ns)?,
            parameters_len: value("resident_attached_parameters", parameters_len)?,
        };
        if dims.nb == 0 {
            return Err(LauncherError::InvalidArray("常驻附着维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：跨度、几何体、site体、静态几何与四对描述符。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nb == 0 {
            return Err(LauncherError::InvalidArray("常驻附着维度无效"));
        }
        u64::from(self.ng)
            .checked_mul(2)
            .and_then(|value| value.checked_add(u64::from(self.ns)))
            .and_then(|value| value.checked_add(9))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_attached_metadata"))
    }

    /// 刚体输入每世界元素数，必须等于元数据首项。
    pub(super) fn body_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nb)
            .checked_mul(28)
            .and_then(|value| {
                u64::from(self.nj)
                    .checked_mul(6)
                    .and_then(|joints| value.checked_add(joints))
            })
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_attached_body"))
    }

    /// 附着输出每世界元素数：几何与site各十二项。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.ng)
            .checked_add(u64::from(self.ns))
            .and_then(|count| count.checked_mul(12))
            .ok_or(LauncherError::Overflow("resident_attached_output"))
    }
}

/// 附着发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResidentAttachedDims {
    pub(super) static_dims: ResidentAttachedStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentAttachedDims {
    /// 冻结ABI：刚体数、几何数、site数与世界数。
    pub(super) fn abi(&self) -> [u32; 4] {
        [
            self.static_dims.nb,
            self.static_dims.ng,
            self.static_dims.ns,
            self.worlds,
        ]
    }
}
