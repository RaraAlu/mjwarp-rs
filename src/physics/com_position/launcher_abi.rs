//! 质心位置同步发射的槽位、证明与维度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.com_position.metadata.v1");

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
/// 第三槽是质心输出，内核只写不读。
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
pub(super) struct ComPositionStaticDims {
    pub(super) nbody: u32,
    pub(super) njnt: u32,
    pub(super) nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl ComPositionStaticDims {
    pub(super) fn new(nbody: usize, njnt: usize, nv: usize) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nbody: value("com_position_nbody", nbody)?,
            njnt: value("com_position_njnt", njnt)?,
            nv: value("com_position_nv", nv)?,
        };
        if dims.nbody == 0 {
            return Err(LauncherError::InvalidArray("质心位置维度无效"));
        }
        dims.metadata_elements()?;
        dims.parameters_elements()?;
        dims.state_stride()?;
        dims.output_stride()?;
        Ok(dims)
    }

    /// 元数据元素数：体父与三类关节表。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        if self.nbody == 0 {
            return Err(LauncherError::InvalidArray("质心位置维度无效"));
        }
        u64::from(self.njnt)
            .checked_mul(3)
            .and_then(|value| value.checked_add(u64::from(self.nbody)))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("com_position_metadata"))
    }

    /// 模型参数元素数：每体质量与三项惯量。
    pub(super) fn parameters_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(4)
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("com_position_parameters"))
    }

    /// 运动学输入每世界元素数。
    pub(super) fn state_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(28)
            .and_then(|value| {
                u64::from(self.njnt)
                    .checked_mul(6)
                    .and_then(|joints| value.checked_add(joints))
            })
            .ok_or(LauncherError::Overflow("com_position_state"))
    }

    /// 质心输出每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nbody)
            .checked_mul(14)
            .and_then(|value| {
                u64::from(self.nv)
                    .checked_mul(6)
                    .and_then(|dofs| value.checked_add(dofs))
            })
            .ok_or(LauncherError::Overflow("com_position_output"))
    }
}

/// 质心位置发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ComPositionDims {
    pub(super) static_dims: ComPositionStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl ComPositionDims {
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
