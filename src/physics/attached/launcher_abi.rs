//! 附着运动学同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.attached.metadata.v1");

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
/// 第三槽是附着结果，内核只写不读。
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
pub(super) struct AttachedStaticDims {
    pub(super) nbody: u32,
    pub(super) njnt: u32,
    pub(super) ngeom: u32,
    pub(super) nsite: u32,
}

#[cfg(feature = "cuda-probe")]
impl AttachedStaticDims {
    pub(super) fn new(
        nbody: usize,
        njnt: usize,
        ngeom: usize,
        nsite: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nbody: value("attached_nbody", nbody)?,
            njnt: value("attached_njnt", njnt)?,
            ngeom: value("attached_ngeom", ngeom)?,
            nsite: value("attached_nsite", nsite)?,
        };
        if dims.nbody == 0 {
            return Err(LauncherError::InvalidArray("附着运动学维度无效"));
        }
        dims.metadata_elements()?;
        dims.body_stride()?;
        dims.output_stride()?;
        Ok(dims)
    }

    /// 元数据元素数：刚体跨度加几何与site体索引。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nbody == 0 {
            return Err(LauncherError::InvalidArray("附着运动学维度无效"));
        }
        let count = u64::from(self.ngeom)
            .checked_add(u64::from(self.nsite))
            .and_then(|value| value.checked_add(1))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("attached_metadata"))?;
        Ok(count)
    }

    /// 参数元素数：七项几何/site位姿，空集合保留一项。
    pub(super) fn parameters_elements(self) -> u64 {
        let count = u64::from(self.ngeom) + u64::from(self.nsite);
        (count * 7).max(1)
    }

    /// 刚体运动学输入每世界元素数，必须与元数据首项一致。
    pub(super) fn body_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(28)
            .and_then(|value| {
                u64::from(self.njnt)
                    .checked_mul(6)
                    .and_then(|joints| value.checked_add(joints))
            })
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("attached_body"))
    }

    /// 附着结果每世界元素数：几何与site各十二项。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.ngeom)
            .checked_add(u64::from(self.nsite))
            .and_then(|count| count.checked_mul(12))
            .ok_or(LauncherError::Overflow("attached_output"))
    }
}

/// 附着运动学发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AttachedDims {
    pub(super) static_dims: AttachedStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl AttachedDims {
    /// 冻结ABI：刚体数、几何数、site数与世界数。
    pub(super) fn abi(&self) -> [u32; 4] {
        [
            self.static_dims.nbody,
            self.static_dims.ngeom,
            self.static_dims.nsite,
            self.worlds,
        ]
    }
}
