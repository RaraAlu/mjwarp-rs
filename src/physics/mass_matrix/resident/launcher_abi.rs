//! 常驻CRB三类同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_crb.metadata.v1");

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
/// 第三槽是质量矩阵输出，内核只写不读。
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
pub(super) struct ResidentCrbStaticDims {
    pub(super) nbody: u32,
    pub(super) nv: u32,
    pub(super) depth: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCrbStaticDims {
    pub(super) fn new(nbody: usize, nv: usize, depth: usize) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nbody: value("resident_crb_nbody", nbody)?,
            nv: value("resident_crb_nv", nv)?,
            depth: value("resident_crb_depth", depth)?,
        };
        if dims.nbody == 0 {
            return Err(LauncherError::InvalidArray("常驻CRB维度无效"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：维度二项、体父、关节体、关节父与深度表。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nbody == 0 {
            return Err(LauncherError::InvalidArray("常驻CRB维度无效"));
        }
        u64::from(self.nbody)
            .checked_mul(2)
            .and_then(|value| {
                u64::from(self.nv)
                    .checked_mul(2)
                    .and_then(|joints| value.checked_add(joints))
            })
            .and_then(|value| value.checked_add(2))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_crb_metadata"))
    }

    /// 骨架参数元素数：空臂ature按一项零点上传。
    pub(super) fn armature_elements(self) -> u64 {
        u64::from(self.nv.max(1))
    }

    /// 质心输入每世界元素数。
    pub(super) fn state_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(14)
            .and_then(|value| {
                u64::from(self.nv)
                    .checked_mul(6)
                    .and_then(|joints| value.checked_add(joints))
            })
            .ok_or(LauncherError::Overflow("resident_crb_state"))
    }

    /// 质量矩阵输出每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(10)
            .and_then(|value| {
                u64::from(self.nv)
                    .checked_mul(u64::from(self.nv))
                    .and_then(|dense| value.checked_add(dense))
            })
            .ok_or(LauncherError::Overflow("resident_crb_output"))
    }
}

/// 初始化发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResidentCrbInitDims {
    pub(super) static_dims: ResidentCrbStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCrbInitDims {
    pub(super) fn abi(&self) -> [u32; 4] {
        [self.static_dims.nbody, self.static_dims.nv, 0, self.worlds]
    }
}

/// 逐层累加发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResidentCrbAccumulateDims {
    pub(super) static_dims: ResidentCrbStaticDims,
    pub(super) worlds: u32,
    pub(super) level: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCrbAccumulateDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        let work = u64::from(self.worlds)
            .checked_mul(u64::from(self.static_dims.nbody))
            .filter(|&value| value <= u32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_crb_accumulate"))?;
        Ok([
            self.static_dims.nbody,
            self.static_dims.nv,
            self.level,
            work as u32,
        ])
    }
}

/// 矩阵发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResidentCrbMatrixDims {
    pub(super) static_dims: ResidentCrbStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCrbMatrixDims {
    pub(super) fn abi(&self) -> Result<[u32; 4], LauncherError> {
        let work = u64::from(self.worlds)
            .checked_mul(u64::from(self.static_dims.nv))
            .filter(|&value| value <= u32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_crb_matrix"))?;
        Ok([self.static_dims.nbody, self.static_dims.nv, 0, work as u32])
    }
}
