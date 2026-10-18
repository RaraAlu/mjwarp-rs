//! 常驻RNE加速度辅助。
//! 不替代完整G02或等价调度。

use super::{ComVelocityData, ComVelocityPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout};
use crate::runtime::TransferSession;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 复用空间速度底座的严格辅助。
/// 共享重力；不转换完整Option。
pub struct RnePlan {
    velocity: ComVelocityPlan,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    depth: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: RneMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    backward_kernel: RneBackwardKernel,
    #[cfg(feature = "cuda-probe")]
    cacc_kernel: RneCaccKernel,
    #[cfg(feature = "cuda-probe")]
    cfrc_kernel: RneCfrcKernel,
    #[cfg(feature = "cuda-probe")]
    qfrc_kernel: RneQfrcKernel,
}

/// 独占qacc与RNE工作区。
pub struct RneData {
    velocity: ComVelocityData,
    layout: RneLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    flg_acc: bool,
    #[cfg(feature = "cuda-probe")]
    qacc: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

#[derive(Clone, Debug)]
pub struct RneSnapshot {
    layout: RneLayout,
    flg_acc: bool,
    values: Vec<f32>,
}

/// 空间向量采用角量在前。
#[derive(Clone, Copy, Debug)]
pub struct RneWorld<'a> {
    pub cacc: &'a [f32],
    pub cfrc_int: &'a [f32],
    /// 跟随上游字段名。
    /// flg_acc=true时包含加速度。
    pub qfrc_bias: &'a [f32],
}

#[derive(Clone, Copy, Debug)]
struct RneLayout {
    output: BatchLayout,
    qacc: BatchLayout,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}

impl RneLayout {
    fn new(worlds: usize, nbody: usize, nv: usize) -> Result<Self, InputError> {
        let overflow = || InputError::Overflow {
            field: "rne_layout",
        };
        if worlds == 0 || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "rne_dimensions",
            });
        }
        worlds
            .checked_mul(nbody)
            .filter(|&v| v <= (u32::MAX - 255) as usize)
            .ok_or_else(overflow)?;
        let width = nbody
            .checked_mul(12)
            .and_then(|n| n.checked_add(nv))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let metadata = nbody
            .checked_mul(4)
            .and_then(|n| n.checked_add(nv))
            .and_then(|n| n.checked_add(5))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, metadata, 4)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        let qacc = BatchLayout::new(worlds, nv, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        let qa = qacc.total_elements().checked_add(8).ok_or_else(overflow)?;
        BatchLayout::new(1, qa, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = guarded;
        Ok(Self {
            output,
            qacc,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

#[cfg(any(feature = "cuda-probe", test))]
const RNE_CACC_METADATA_KIND: ProofKind = ProofKind("rne.cacc.metadata.v1");

#[cfg(any(feature = "cuda-probe", test))]
const RNE_QFRC_METADATA_KIND: ProofKind = ProofKind("rne.qfrc.metadata.v1");

#[cfg(any(feature = "cuda-probe", test))]
const RNE_BACKWARD_METADATA_KIND: ProofKind = ProofKind("rne.backward.metadata.v1");

#[cfg(any(feature = "cuda-probe", test))]
const RNE_CACC_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RNE_CACC_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(any(feature = "cuda-probe", test))]
const RNE_CFRC_METADATA_KIND: ProofKind = ProofKind("rne.cfrc.metadata.v1");

#[cfg(any(feature = "cuda-probe", test))]
const RNE_CFRC_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RNE_CFRC_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(any(feature = "cuda-probe", test))]
const RNE_QFRC_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RNE_QFRC_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(any(feature = "cuda-probe", test))]
const RNE_BACKWARD_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: RNE_BACKWARD_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RneStaticDims {
    nbody: u32,
    nv: u32,
    depth: u32,
    metadata_elements: u64,
}

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RneCaccDims {
    flag: u32,
    worlds: u32,
    static_dims: RneStaticDims,
}

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RneQfrcDims {
    abi: [u32; 4],
    static_dims: RneStaticDims,
}

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RneCfrcDims {
    abi: [u32; 4],
    static_dims: RneStaticDims,
}

#[cfg(any(feature = "cuda-probe", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RneBackwardDims {
    abi: [u32; 4],
    worlds: u32,
    static_dims: RneStaticDims,
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_metadata_elements(nbody: u32, nv: u32) -> Result<u64, LauncherError> {
    if nbody > i32::MAX as u32 || nv > i32::MAX as u32 {
        return Err(LauncherError::Overflow("rne_metadata_signed_dimensions"));
    }
    u64::from(nbody)
        .checked_mul(4)
        .and_then(|value| value.checked_add(u64::from(nv)))
        .and_then(|value| value.checked_add(5))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("rne_metadata_elements"))
}

#[cfg(any(feature = "cuda-probe", test))]
fn validate_rne_metadata(values: &[i32], dims: RneStaticDims) -> Result<(), LauncherError> {
    if dims.nbody == 0 || dims.depth >= dims.nbody {
        return Err(LauncherError::InvalidMetadata("RNE静态维度无效"));
    }
    let expected = rne_metadata_elements(dims.nbody, dims.nv)?;
    if dims.metadata_elements != expected
        || u64::try_from(values.len()).ok() != Some(dims.metadata_elements)
    {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度无效"));
    }
    let nbody =
        usize::try_from(dims.nbody).map_err(|_| LauncherError::Overflow("rne_metadata_nbody"))?;
    let nv = usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("rne_metadata_nv"))?;
    if values[0] != dims.nbody as i32 || values[1] != dims.nv as i32 {
        return Err(LauncherError::InvalidMetadata("RNE元数据维度不匹配"));
    }
    if values[2..5]
        .iter()
        .any(|&bits| !f32::from_bits(bits as u32).is_finite())
    {
        return Err(LauncherError::InvalidMetadata("RNE重力不是有限值"));
    }

    let parent_start = 5;
    let count_start = parent_start + nbody;
    let address_start = count_start + nbody;
    let body_start = address_start + nbody;
    let depth_start = body_start + nv;
    let parent = &values[parent_start..count_start];
    let count = &values[count_start..address_start];
    let address = &values[address_start..body_start];
    let body = &values[body_start..depth_start];
    let depth = &values[depth_start..];
    if parent[0] != 0 || count[0] != 0 || address[0] != 0 || depth[0] != 0 {
        return Err(LauncherError::InvalidMetadata("RNE根节点元数据无效"));
    }
    let mut covered = vec![false; nv];
    let mut max_depth = 0usize;
    for index in 1..nbody {
        let parent_index = usize::try_from(parent[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE父节点为负"))?;
        if parent_index >= index {
            return Err(LauncherError::InvalidMetadata("RNE父节点顺序无效"));
        }
        let dof_count = usize::try_from(count[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE自由度数量为负"))?;
        let dof_start = usize::try_from(address[index])
            .map_err(|_| LauncherError::InvalidMetadata("RNE自由度地址为负"))?;
        let dof_end = dof_start
            .checked_add(dof_count)
            .filter(|&end| end <= nv)
            .ok_or(LauncherError::InvalidMetadata("RNE自由度范围越界"))?;
        for dof in dof_start..dof_end {
            if body[dof] != index as i32 || covered[dof] {
                return Err(LauncherError::InvalidMetadata("RNE自由度归属无效"));
            }
            covered[dof] = true;
        }
        let expected_depth = depth[parent_index]
            .checked_add(1)
            .ok_or(LauncherError::Overflow("rne_tree_depth"))?;
        if depth[index] != expected_depth {
            return Err(LauncherError::InvalidMetadata("RNE树深度无效"));
        }
        max_depth = max_depth.max(
            usize::try_from(depth[index])
                .map_err(|_| LauncherError::InvalidMetadata("RNE树深度为负"))?,
        );
    }
    if covered.contains(&false) {
        return Err(LauncherError::InvalidMetadata("RNE自由度映射不完整"));
    }
    for &body_index in body {
        let body_index = usize::try_from(body_index)
            .map_err(|_| LauncherError::InvalidMetadata("RNE自由度体索引为负"))?;
        if body_index >= nbody {
            return Err(LauncherError::InvalidMetadata("RNE自由度体索引越界"));
        }
    }
    if max_depth != dims.depth as usize {
        return Err(LauncherError::InvalidMetadata("RNE最大树深度不匹配"));
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_cacc_lengths(dims: &RneCaccDims) -> Result<[u64; 6], LauncherError> {
    let static_dims = dims.static_dims;
    if dims.flag > 1 || dims.worlds == 0 || dims.worlds > u32::MAX - 255 || static_dims.nbody == 0 {
        return Err(LauncherError::InvalidArray("RNE加速度维度无效"));
    }
    if static_dims.metadata_elements != rne_metadata_elements(static_dims.nbody, static_dims.nv)? {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度不匹配"));
    }
    let guarded = |width: u64, field| {
        u64::from(dims.worlds)
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let nbody = u64::from(static_dims.nbody);
    let nv = u64::from(static_dims.nv);
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_cacc_com"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_cacc_velocity"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_cacc_output"))?;
    Ok([
        static_dims.metadata_elements,
        guarded(nv, "rne_cacc_qvel")?,
        guarded(nv, "rne_cacc_qacc")?,
        guarded(com_width, "rne_cacc_com")?,
        guarded(velocity_width, "rne_cacc_velocity")?,
        guarded(output_width, "rne_cacc_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneCaccSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_cacc私有ABI。
unsafe impl KernelAbi for RneCaccSpec {
    const NAME: &'static str = "rne_cacc";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_CACC_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneCaccSpec {
    type Dims = RneCaccDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_cacc_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_cacc_lengths(dims)?;
        LaunchDims::new((dims.worlds.div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_cfrc_lengths(dims: &RneCfrcDims) -> Result<[u64; 4], LauncherError> {
    let [x, y, z, worlds] = dims.abi;
    let static_dims = dims.static_dims;
    if [x, y, z] != [0; 3] || worlds == 0 || worlds > u32::MAX - 255 || static_dims.nbody == 0 {
        return Err(LauncherError::InvalidArray("RNE合力维度无效"));
    }
    if static_dims.metadata_elements != rne_metadata_elements(static_dims.nbody, static_dims.nv)? {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度不匹配"));
    }
    let guarded = |width: u64, field| {
        u64::from(worlds)
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let nbody = u64::from(static_dims.nbody);
    let nv = u64::from(static_dims.nv);
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_cfrc_com"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_cfrc_velocity"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_cfrc_output"))?;
    Ok([
        static_dims.metadata_elements,
        guarded(com_width, "rne_cfrc_com")?,
        guarded(velocity_width, "rne_cfrc_velocity")?,
        guarded(output_width, "rne_cfrc_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneCfrcSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_cfrc私有ABI。
unsafe impl KernelAbi for RneCfrcSpec {
    const NAME: &'static str = "rne_cfrc";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_CFRC_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneCfrcSpec {
    type Dims = RneCfrcDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_cfrc_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_cfrc_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_qfrc_lengths(dims: &RneQfrcDims) -> Result<[u64; 4], LauncherError> {
    let [x, y, z, worlds] = dims.abi;
    let static_dims = dims.static_dims;
    if [x, y, z] != [0; 3] || worlds == 0 || worlds > u32::MAX - 255 || static_dims.nbody == 0 {
        return Err(LauncherError::InvalidArray("RNE广义力维度无效"));
    }
    let backward_work = u64::from(worlds)
        .checked_mul(u64::from(static_dims.nbody))
        .ok_or(LauncherError::Overflow("rne_backward_work"))?;
    if backward_work > u64::from(u32::MAX - 255) {
        return Err(LauncherError::Overflow("rne_backward_work"));
    }
    if static_dims.metadata_elements != rne_metadata_elements(static_dims.nbody, static_dims.nv)? {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度不匹配"));
    }
    let guarded = |width: u64, field| {
        u64::from(worlds)
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let nbody = u64::from(static_dims.nbody);
    let nv = u64::from(static_dims.nv);
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_qfrc_com"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_qfrc_velocity"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_qfrc_output"))?;
    Ok([
        static_dims.metadata_elements,
        guarded(com_width, "rne_qfrc_com")?,
        guarded(velocity_width, "rne_qfrc_velocity")?,
        guarded(output_width, "rne_qfrc_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneQfrcSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_qfrc私有ABI。
unsafe impl KernelAbi for RneQfrcSpec {
    const NAME: &'static str = "rne_qfrc";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_QFRC_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneQfrcSpec {
    type Dims = RneQfrcDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_qfrc_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_qfrc_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_backward_lengths(dims: &RneBackwardDims) -> Result<[u64; 4], LauncherError> {
    let [nbody_abi, level, zero, work_abi] = dims.abi;
    let static_dims = dims.static_dims;
    if dims.worlds == 0
        || static_dims.nbody == 0
        || nbody_abi != static_dims.nbody
        || level == 0
        || level > static_dims.depth
        || zero != 0
    {
        return Err(LauncherError::InvalidArray("RNE反向层级维度无效"));
    }
    if static_dims.metadata_elements != rne_metadata_elements(static_dims.nbody, static_dims.nv)? {
        return Err(LauncherError::InvalidMetadata("RNE元数据长度不匹配"));
    }
    let work = u64::from(dims.worlds)
        .checked_mul(u64::from(static_dims.nbody))
        .ok_or(LauncherError::Overflow("rne_backward_work"))?;
    if work > u64::from(u32::MAX - 255) {
        return Err(LauncherError::Overflow("rne_backward_work"));
    }
    if u64::from(work_abi) != work {
        return Err(LauncherError::InvalidArray("RNE反向工作量无效"));
    }
    let guarded = |width: u64, field| {
        u64::from(dims.worlds)
            .checked_mul(width)
            .and_then(|value| value.checked_add(8))
            .ok_or(LauncherError::Overflow(field))
    };
    let nbody = u64::from(static_dims.nbody);
    let nv = u64::from(static_dims.nv);
    let com_width = nbody
        .checked_mul(14)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_backward_com"))?;
    let velocity_width = nbody
        .checked_mul(6)
        .and_then(|value| nv.checked_mul(6).and_then(|dof| value.checked_add(dof)))
        .ok_or(LauncherError::Overflow("rne_backward_velocity"))?;
    let output_width = nbody
        .checked_mul(12)
        .and_then(|value| value.checked_add(nv))
        .ok_or(LauncherError::Overflow("rne_backward_output"))?;
    Ok([
        static_dims.metadata_elements,
        guarded(com_width, "rne_backward_com")?,
        guarded(velocity_width, "rne_backward_velocity")?,
        guarded(output_width, "rne_backward_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneBackwardSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐槽匹配冻结rne_backward私有ABI。
unsafe impl KernelAbi for RneBackwardSpec {
    const NAME: &'static str = "rne_backward";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RNE_BACKWARD_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for RneBackwardSpec {
    type Dims = RneBackwardDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rne_backward_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rne_backward_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
struct StaticRneMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticRneMetadata {
    type Dims = RneStaticDims;

    const KIND: ProofKind = RNE_QFRC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, *dims)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneCaccMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneCaccMetadata {
    type Dims = RneCaccDims;

    const KIND: ProofKind = RNE_CACC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, dims.static_dims)?;
        rne_cacc_lengths(dims).map(|_| ())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_cacc_plan(
    dims: &RneCaccDims,
    bindings: &[SlotBinding; 6],
    proofs: &[CheckedBinding<'_, RneCaccDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_cacc_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_cacc_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneCaccSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
fn rne_cacc_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_cacc_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_cacc_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneCfrcMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneCfrcMetadata {
    type Dims = RneCfrcDims;

    const KIND: ProofKind = RNE_CFRC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, dims.static_dims)?;
        rne_cfrc_lengths(dims).map(|_| ())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_cfrc_plan(
    dims: &RneCfrcDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RneCfrcDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_cfrc_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_cfrc_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneCfrcSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
fn rne_cfrc_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_cfrc_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_cfrc_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneQfrcMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneQfrcMetadata {
    type Dims = RneQfrcDims;

    const KIND: ProofKind = RNE_QFRC_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, dims.static_dims)?;
        rne_qfrc_lengths(dims).map(|_| ())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_qfrc_plan(
    dims: &RneQfrcDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RneQfrcDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_qfrc_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_qfrc_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneQfrcSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
fn rne_qfrc_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_qfrc_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_qfrc_abi",
        }
        .into(),
    }
}

#[cfg(any(feature = "cuda-probe", test))]
struct RneBackwardMetadata;

#[cfg(any(feature = "cuda-probe", test))]
impl MetadataProof for RneBackwardMetadata {
    type Dims = RneBackwardDims;

    const KIND: ProofKind = RNE_BACKWARD_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_rne_metadata(values, dims.static_dims)?;
        rne_backward_lengths(dims).map(|_| ())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn rne_backward_plan(
    dims: &RneBackwardDims,
    bindings: &[SlotBinding; 4],
    proofs: &[CheckedBinding<'_, RneBackwardDims>],
) -> Result<LaunchDims, LauncherError> {
    let lengths = rne_backward_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rne_backward_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(Kernel::<RneBackwardSpec>::new()?
        .plan(dims, bindings, proofs)?
        .dims())
}

#[cfg(feature = "cuda-probe")]
fn rne_backward_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "rne_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rne_backward_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rne_backward_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
struct RneMetadataBuffer {
    session: TransferSession,
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    dims: RneStaticDims,
    revision: MetadataRevision,
    checked: Checked<StaticRneMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl RneMetadataBuffer {
    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        nbody: usize,
        nv: usize,
        depth: usize,
    ) -> Result<Self, TransferError> {
        let dims = RneStaticDims {
            nbody: u32::try_from(nbody).map_err(|_| InputError::Overflow {
                field: "rne_metadata_nbody",
            })?,
            nv: u32::try_from(nv).map_err(|_| InputError::Overflow {
                field: "rne_metadata_nv",
            })?,
            depth: u32::try_from(depth).map_err(|_| InputError::Overflow {
                field: "rne_metadata_depth",
            })?,
            metadata_elements: u64::try_from(values.len()).map_err(|_| InputError::Overflow {
                field: "rne_metadata",
            })?,
        };
        validate_rne_metadata(&values, dims).map_err(rne_qfrc_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<StaticRneMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(rne_qfrc_launch_error)?;
        Ok(Self {
            session: session.clone(),
            buffer,
            values,
            dims,
            revision,
            checked,
        })
    }

    fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    fn validate_base(&self) -> Result<crate::runtime::launcher::DeviceSpan, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(rne_qfrc_launch_error)?;
        Ok(span)
    }

    fn cacc_proof(
        &self,
        flg_acc: bool,
        worlds: u32,
    ) -> Result<Checked<RneCaccMetadata>, TransferError> {
        let span = self.validate_base()?;
        let dims = RneCaccDims {
            flag: u32::from(flg_acc),
            worlds,
            static_dims: self.dims,
        };
        // SAFETY: 基础证明已核对真实缓冲、镜像和写入代际。
        unsafe {
            Checked::<RneCaccMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_cacc_launch_error)
    }

    fn cfrc_proof(&self, worlds: u32) -> Result<Checked<RneCfrcMetadata>, TransferError> {
        let span = self.validate_base()?;
        let dims = RneCfrcDims {
            abi: [0, 0, 0, worlds],
            static_dims: self.dims,
        };
        // SAFETY: 基础证明已核对真实缓冲、镜像和写入代际。
        unsafe {
            Checked::<RneCfrcMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_cfrc_launch_error)
    }

    fn qfrc_proof(&self, worlds: u32) -> Result<Checked<RneQfrcMetadata>, TransferError> {
        let span = self.validate_base()?;
        let dims = RneQfrcDims {
            abi: [0, 0, 0, worlds],
            static_dims: self.dims,
        };
        // SAFETY: 私有包装持有实际缓冲、完整镜像和写入代际。
        unsafe {
            Checked::<RneQfrcMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_qfrc_launch_error)
    }

    fn backward_proof(
        &self,
        worlds: u32,
        level: u32,
    ) -> Result<Checked<RneBackwardMetadata>, TransferError> {
        let span = self.validate_base()?;
        let work = worlds
            .checked_mul(self.dims.nbody)
            .ok_or(InputError::Overflow {
                field: "rne_backward_work",
            })?;
        let dims = RneBackwardDims {
            abi: [self.dims.nbody, level, 0, work],
            worlds,
            static_dims: self.dims,
        };
        // SAFETY: 基础证明已核对真实缓冲、镜像和写入代际。
        unsafe {
            Checked::<RneBackwardMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_backward_launch_error)
    }

    #[cfg(test)]
    fn overwrite_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入已同步；跨度与镜像重新绑定。
        self.checked = unsafe {
            Checked::<StaticRneMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(rne_qfrc_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
struct RneCaccLaunch {
    proof: Checked<RneCaccMetadata>,
}

#[cfg(feature = "cuda-probe")]
struct RneCaccKernel {
    session: TransferSession,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneCaccKernel {
    fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_CUDA, "rne_cacc")?,
        })
    }

    fn bindings(
        metadata: &RneMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        qacc: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 6], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: qvel.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: qacc.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: velocity.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 5,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    #[allow(clippy::too_many_arguments)]
    fn preflight(
        &self,
        session: &TransferSession,
        metadata: &RneMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        qacc: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        flg_acc: bool,
        worlds: u32,
    ) -> Result<RneCaccLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.cacc_proof(flg_acc, worlds)?;
        let bindings = Self::bindings(metadata, qvel, qacc, com, velocity, output)?;
        rne_cacc_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_cacc_launch_error)?;
        Ok(RneCaccLaunch { proof })
    }

    #[allow(clippy::too_many_arguments)]
    fn launch(
        &self,
        launch: &RneCaccLaunch,
        metadata: &RneMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        qacc: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, qvel, qacc, com, velocity, output)?;
        rne_cacc_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_cacc_launch_error)?;
        // SAFETY: 类型计划核对证明、六个跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel.launch_rne_cacc(
                metadata.as_buffer(),
                (qvel, qacc, com, velocity),
                output,
                dims.flag != 0,
                dims.worlds,
            )
        }
    }
}

#[cfg(feature = "cuda-probe")]
struct RneCfrcLaunch {
    proof: Checked<RneCfrcMetadata>,
}

#[cfg(feature = "cuda-probe")]
struct RneCfrcKernel {
    session: TransferSession,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneCfrcKernel {
    fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_CUDA, "rne_cfrc")?,
        })
    }

    fn bindings(
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 4], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: velocity.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    fn preflight(
        &self,
        session: &TransferSession,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<RneCfrcLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.cfrc_proof(worlds)?;
        let bindings = Self::bindings(metadata, com, velocity, output)?;
        rne_cfrc_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_cfrc_launch_error)?;
        Ok(RneCfrcLaunch { proof })
    }

    fn launch(
        &self,
        launch: &RneCfrcLaunch,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, com, velocity, output)?;
        rne_cfrc_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_cfrc_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), com, velocity, output, dims.abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
struct RneBackwardLaunch {
    proof: Checked<RneBackwardMetadata>,
}

#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy)]
struct RneBackwardRequest {
    worlds: u32,
    level: u32,
}

#[cfg(feature = "cuda-probe")]
struct RneBackwardKernel {
    session: TransferSession,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneBackwardKernel {
    fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_CUDA, "rne_backward")?,
        })
    }

    fn bindings(
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 4], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: velocity.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    fn preflight(
        &self,
        session: &TransferSession,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        request: RneBackwardRequest,
    ) -> Result<RneBackwardLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.backward_proof(request.worlds, request.level)?;
        let bindings = Self::bindings(metadata, com, velocity, output)?;
        rne_backward_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_backward_launch_error)?;
        Ok(RneBackwardLaunch { proof })
    }

    fn launch(
        &self,
        launch: &RneBackwardLaunch,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, com, velocity, output)?;
        rne_backward_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_backward_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和逐层网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), com, velocity, output, dims.abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
struct RneQfrcLaunch {
    proof: Checked<RneQfrcMetadata>,
}

#[cfg(feature = "cuda-probe")]
struct RneQfrcKernel {
    session: TransferSession,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneQfrcKernel {
    fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_CUDA, "rne_qfrc")?,
        })
    }

    fn bindings(
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 4], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    fn preflight(
        &self,
        session: &TransferSession,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<RneQfrcLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.qfrc_proof(worlds)?;
        let bindings = Self::bindings(metadata, com, unused, output)?;
        rne_qfrc_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_qfrc_launch_error)?;
        Ok(RneQfrcLaunch { proof })
    }

    fn launch(
        &self,
        launch: &RneQfrcLaunch,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, com, unused, output)?;
        rne_qfrc_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_qfrc_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), com, unused, output, dims.abi)
        }
    }
}

impl RnePlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        gravity: [f32; 3],
    ) -> Result<Self, TransferError> {
        check_values("rne_gravity", 3, &gravity)?;
        let k = model.rigid().kinematics();
        let (nbody, nv) = (k.nbody(), k.nv());
        RneLayout::new(1, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let (metadata, depth) = {
            let f = k.fields();
            let mut m = crate::runtime::host_staging::<i32>(5 + 4 * nbody + nv)?;
            m[0] = nbody as i32;
            m[1] = nv as i32;
            for i in 0..3 {
                m[2 + i] = gravity[i].to_bits() as i32;
            }
            m[5..5 + nbody].copy_from_slice(&f.body_parentid);
            for b in 1..nbody {
                let start = f.body_jntadr[b] as usize;
                let count = f.body_jntnum[b] as usize;
                if count != 0 {
                    m[5 + 2 * nbody + b] = f.jnt_dofadr[start];
                    m[5 + nbody + b] = f.jnt_type[start..start + count]
                        .iter()
                        .map(|&t| match t {
                            0 => 6,
                            1 => 3,
                            _ => 1,
                        })
                        .sum();
                }
                m[5 + 3 * nbody + nv + b] = m[5 + 3 * nbody + nv + f.body_parentid[b] as usize] + 1;
            }
            m[5 + 3 * nbody..5 + 3 * nbody + nv]
                .copy_from_slice(&model.rigid().fields().dof_bodyid);
            let depth = *m[5 + 3 * nbody + nv..].iter().max().unwrap() as usize;
            (
                RneMetadataBuffer::upload(session, m, nbody, nv, depth)?,
                depth,
            )
        };
        let velocity = ComVelocityPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let backward_kernel = RneBackwardKernel::compile(session)?;
        #[cfg(feature = "cuda-probe")]
        let cacc_kernel = RneCaccKernel::compile(session)?;
        #[cfg(feature = "cuda-probe")]
        let cfrc_kernel = RneCfrcKernel::compile(session)?;
        #[cfg(feature = "cuda-probe")]
        let qfrc_kernel = RneQfrcKernel::compile(session)?;
        Ok(Self {
            velocity,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            depth,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            backward_kernel,
            #[cfg(feature = "cuda-probe")]
            cacc_kernel,
            #[cfg(feature = "cuda-probe")]
            cfrc_kernel,
            #[cfg(feature = "cuda-probe")]
            qfrc_kernel,
        })
    }

    pub fn device(&self) -> usize {
        self.velocity.device()
    }

    pub fn create_data(&self, worlds: usize) -> Result<RneData, TransferError> {
        let layout = RneLayout::new(worlds, self.nbody, self.nv)?;
        let velocity = self.velocity.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, velocity);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut qacc = crate::runtime::host_staging::<f32>(layout.qacc.total_elements() + 8)?;
            qacc.fill(-131072.0);
            qacc[4..layout.qacc.total_elements() + 4].fill(0.0);
            let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
            values.fill(-131072.0);
            values[4..layout.guarded - 4].fill(f32::NAN);
            Ok(RneData {
                velocity,
                layout,
                ready: false,
                flg_acc: false,
                qacc: self.session.upload(&qacc)?,
                output: self.session.upload(&values)?,
            })
        }
    }

    pub fn update(&self, data: &mut RneData, flg_acc: bool) -> Result<(), TransferError> {
        self.update_velocity(data)?;
        self.update_rne(data, flg_acc)
    }

    pub fn update_velocity(&self, data: &mut RneData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        self.velocity.update(&mut data.velocity)
    }

    /// 只消费已就绪的前置结果。
    /// 本严格辅助不隐式补算前置。
    pub fn update_rne(&self, data: &mut RneData, flg_acc: bool) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = flg_acc;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let inputs = data.velocity.rne_inputs()?;
            let worlds = data.worlds() as u32;
            let cacc_launch = self.cacc_kernel.preflight(
                &self.session,
                &self.metadata,
                inputs.qvel,
                &data.qacc,
                inputs.com,
                inputs.velocity,
                &data.output,
                flg_acc,
                worlds,
            )?;
            let cfrc_launch = self.cfrc_kernel.preflight(
                &self.session,
                &self.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
                worlds,
            )?;
            let qfrc_launch = self.qfrc_kernel.preflight(
                &self.session,
                &self.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
                worlds,
            )?;
            let mut backward_launches = Vec::with_capacity(self.depth);
            for level in (1..=self.depth).rev() {
                backward_launches.push(self.backward_kernel.preflight(
                    &self.session,
                    &self.metadata,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    RneBackwardRequest {
                        worlds,
                        level: level as u32,
                    },
                )?);
            }
            self.cacc_kernel.launch(
                &cacc_launch,
                &self.metadata,
                inputs.qvel,
                &data.qacc,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )?;
            self.cfrc_kernel.launch(
                &cfrc_launch,
                &self.metadata,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )?;
            // 每次同步发射形成反向层级屏障。
            // 同层兄弟仍通过atomicAdd归约。
            for launch in &backward_launches {
                self.backward_kernel.launch(
                    launch,
                    &self.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )?;
            }
            self.qfrc_kernel.launch(
                &qfrc_launch,
                &self.metadata,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )?;
            data.flg_acc = flg_acc;
            data.ready = true;
            Ok(())
        }
    }
}

fn check_values(field: &'static str, expected: usize, values: &[f32]) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite { field, index });
    }
    Ok(())
}

impl RneData {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }

    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }

    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qpos(values);
        self.after_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qpos(world, values);
        self.after_write(result)
    }
    pub fn write_qvel(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qvel(values);
        self.after_write(result)
    }
    pub fn write_world_qvel(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qvel(world, values);
        self.after_write(result)
    }
    pub fn write_qacc(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write_checked_qacc(0, self.layout.qacc.total_elements(), values)
    }
    pub fn write_world_qacc(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.qacc.world_elements(world)?;
        self.write_checked_qacc(range.start, range.len(), values)
    }
    fn write_checked_qacc(
        &mut self,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check_values("rne_qacc", expected, values)?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.qacc.write_range(4 + offset, values)
    }
    pub fn readback(&self) -> Result<RneSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady { stage: "rne" });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.velocity.readback()?;
            let mut qacc =
                crate::runtime::host_staging::<f32>(self.layout.qacc.total_elements() + 8)?;
            self.qacc.read_range_into(0, &mut qacc)?;
            super::check_device_values(&qacc, "rne_qacc")?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            super::check_device_values(&values, "rne_output")?;
            Ok(RneSnapshot {
                layout: self.layout,
                flg_acc: self.flg_acc,
                values,
            })
        }
    }
}

impl RneSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn includes_acceleration(&self) -> bool {
        self.flg_acc
    }
    pub fn world(&self, world: usize) -> Result<RneWorld<'_>, InputError> {
        let range = self.layout.output.world_elements(world)?;
        let v = &self.values[4 + range.start..4 + range.end];
        Ok(RneWorld {
            cacc: &v[..6 * self.nbody()],
            cfrc_int: &v[6 * self.nbody()..12 * self.nbody()],
            qfrc_bias: &v[12 * self.nbody()..],
        })
    }
}

// Adapted from frozen smooth.py RNE and math.py spatial algebra.
// Copyright 2025 The Newton Developers. Apache-2.0.
// See licenses/Apache-2.0.txt for the license and warranty disclaimer.
#[cfg(feature = "cuda-probe")]
const RNE_CUDA: &str = include_str!("rne.cu");

#[cfg(all(test, feature = "cuda-probe"))]
#[path = "../../tests/support/rne_acceleration_frozen.rs"]
mod frozen;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_acceleration_layout_and_input_boundaries() {
        for (w, b, v) in [
            (0, 1, 0),
            (1, 0, 0),
            (usize::MAX, 1, 0),
            (1, usize::MAX, 0),
            (1, 1, usize::MAX),
            (u32::MAX as usize, 2, 0),
        ] {
            assert!(RneLayout::new(w, b, v).is_err());
        }
        let layout = RneLayout::new(513, 1, 0).unwrap();
        assert_eq!(layout.output.total_elements(), 513 * 12);
        assert_eq!(layout.qacc.total_elements(), 0);
        assert!(check_values("test", 2, &[0.0]).is_err());
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(check_values("test", 1, &[value]).is_err());
        }
        assert!(check_values("test", 0, &[]).is_ok());
    }

    #[test]
    fn rne_cacc_plan_checks_metadata_spans_aliases_and_overflow() {
        use crate::runtime::launcher::DeviceSpan;

        let bind = |slot, context, addr, bytes, elem| SlotBinding {
            slot,
            span: DeviceSpan::new(context, addr, bytes).unwrap(),
            elem,
        };
        let static_dims = RneStaticDims {
            nbody: 2,
            nv: 1,
            depth: 1,
            metadata_elements: 14,
        };
        let dims = RneCaccDims {
            flag: 1,
            worlds: 2,
            static_dims,
        };
        let values = vec![2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1];
        assert_eq!(rne_cacc_lengths(&dims).unwrap(), [14, 10, 10, 76, 44, 58]);
        let revision = MetadataRevision::new();
        let metadata_span = DeviceSpan::new(11, 0x1000, 56).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof = unsafe {
            Checked::<RneCaccMetadata>::assume_uploaded(
                metadata_span,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            bind(0, 11, 0x1000, 56, ElemType::I32),
            bind(1, 11, 0x2000, 40, ElemType::F32),
            bind(2, 11, 0x2000, 40, ElemType::F32),
            bind(3, 11, 0x3000, 304, ElemType::F32),
            bind(4, 11, 0x4000, 176, ElemType::F32),
            bind(5, 11, 0x5000, 232, ElemType::F32),
        ];
        let launch = rne_cacc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
        assert_eq!(launch.grid(), (1, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));
        assert_eq!(launch.shared_bytes(), 0);
        assert!(matches!(
            rne_cacc_plan(&dims, &bindings, &[]),
            Err(LauncherError::MissingMetadataProof { slot: 0 })
        ));

        let addresses = [0x1000, 0x2000, 0x2000, 0x3000, 0x4000, 0x5000];
        let bytes = [56, 40, 40, 304, 176, 232];
        for slot in 0..6 {
            let mut short = bindings;
            short[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] - 4).unwrap();
            assert!(matches!(
                rne_cacc_plan(&dims, &short, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));

            let mut long = bindings;
            long[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] + 4).unwrap();
            assert!(matches!(
                rne_cacc_plan(&dims, &long, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));
        }

        let mut foreign = bindings;
        foreign[4].span = DeviceSpan::new(12, 0x4000, 176).unwrap();
        assert!(matches!(
            rne_cacc_plan(&dims, &foreign, &[proof.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 4 })
        ));
        let mut alias = bindings;
        alias[5].span = DeviceSpan::new(11, 0x3000, 232).unwrap();
        assert!(matches!(
            rne_cacc_plan(&dims, &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 3,
                second: 5
            })
        ));

        revision.invalidate();
        assert!(matches!(
            rne_cacc_plan(&dims, &bindings, &[proof.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));
        for invalid in [
            RneCaccDims { flag: 2, ..dims },
            RneCaccDims { worlds: 0, ..dims },
            RneCaccDims {
                worlds: u32::MAX - 254,
                ..dims
            },
        ] {
            assert!(rne_cacc_lengths(&invalid).is_err());
        }
        let signed = RneCaccDims {
            flag: 0,
            worlds: 1,
            static_dims: RneStaticDims {
                nbody: i32::MAX as u32 + 1,
                nv: 0,
                depth: 1,
                metadata_elements: 0,
            },
        };
        assert!(rne_cacc_lengths(&signed).is_err());
        let wide = RneCaccDims {
            flag: 0,
            worlds: u32::MAX - 255,
            static_dims: RneStaticDims {
                nbody: 400_000_000,
                nv: 0,
                depth: 1,
                metadata_elements: 1_600_000_005,
            },
        };
        assert!(matches!(
            rne_cacc_lengths(&wide),
            Err(LauncherError::Overflow("rne_cacc_com"))
        ));

        for (index, value) in [(2, f32::NAN.to_bits() as i32), (6, 1), (11, 2), (12, 2)] {
            let mut invalid = values.clone();
            invalid[index] = value;
            assert!(validate_rne_metadata(&invalid, static_dims).is_err());
        }
        let mut incomplete = values;
        incomplete[8] = 0;
        assert!(validate_rne_metadata(&incomplete, static_dims).is_err());

        let tail = RneCaccDims {
            worlds: 513,
            ..dims
        };
        let tail_launch = RneCaccSpec::launch_dims(&tail).unwrap();
        assert_eq!(tail_launch.grid(), (3, 1, 1));
    }

    #[test]
    fn rne_cacc_plan_accepts_zero_dof_boundary() {
        use crate::runtime::launcher::DeviceSpan;

        let dims = RneCaccDims {
            flag: 0,
            worlds: 2,
            static_dims: RneStaticDims {
                nbody: 2,
                nv: 0,
                depth: 1,
                metadata_elements: 13,
            },
        };
        let values = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let revision = MetadataRevision::new();
        let span = DeviceSpan::new(13, 0x1000, 52).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof =
            unsafe { Checked::<RneCaccMetadata>::assume_uploaded(span, &values, dims, revision) }
                .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(13, 0x2000, 32).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(13, 0x2000, 32).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(13, 0x3000, 256).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: DeviceSpan::new(13, 0x4000, 128).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 5,
                span: DeviceSpan::new(13, 0x5000, 224).unwrap(),
                elem: ElemType::F32,
            },
        ];
        assert_eq!(rne_cacc_lengths(&dims).unwrap(), [13, 8, 8, 64, 32, 56]);
        rne_cacc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    }

    #[test]
    fn rne_cfrc_plan_checks_metadata_spans_aliases_and_overflow() {
        use crate::runtime::launcher::DeviceSpan;

        let bind = |slot, context, addr, bytes, elem| SlotBinding {
            slot,
            span: DeviceSpan::new(context, addr, bytes).unwrap(),
            elem,
        };
        let static_dims = RneStaticDims {
            nbody: 2,
            nv: 1,
            depth: 1,
            metadata_elements: 14,
        };
        let dims = RneCfrcDims {
            abi: [0, 0, 0, 2],
            static_dims,
        };
        let values = vec![2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1];
        assert_eq!(rne_cfrc_lengths(&dims).unwrap(), [14, 76, 44, 58]);
        let revision = MetadataRevision::new();
        let metadata_span = DeviceSpan::new(11, 0x1000, 56).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof = unsafe {
            Checked::<RneCfrcMetadata>::assume_uploaded(
                metadata_span,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            bind(0, 11, 0x1000, 56, ElemType::I32),
            bind(1, 11, 0x2000, 304, ElemType::F32),
            bind(2, 11, 0x2000, 176, ElemType::F32),
            bind(3, 11, 0x4000, 232, ElemType::F32),
        ];
        let launch = rne_cfrc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
        assert_eq!(launch.grid(), (1, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));
        assert_eq!(launch.shared_bytes(), 0);
        assert!(matches!(
            rne_cfrc_plan(&dims, &bindings, &[]),
            Err(LauncherError::MissingMetadataProof { slot: 0 })
        ));

        let addresses = [0x1000, 0x2000, 0x2000, 0x4000];
        let bytes = [56, 304, 176, 232];
        for slot in 0..4 {
            let mut short = bindings;
            short[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] - 4).unwrap();
            assert!(matches!(
                rne_cfrc_plan(&dims, &short, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));

            let mut long = bindings;
            long[slot].span = DeviceSpan::new(11, addresses[slot], bytes[slot] + 4).unwrap();
            assert!(matches!(
                rne_cfrc_plan(&dims, &long, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));
        }

        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(12, 0x2000, 176).unwrap();
        assert!(matches!(
            rne_cfrc_plan(&dims, &foreign, &[proof.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));
        let mut alias = bindings;
        alias[3].span = DeviceSpan::new(11, 0x2000, 232).unwrap();
        assert!(matches!(
            rne_cfrc_plan(&dims, &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3
            })
        ));

        revision.invalidate();
        assert!(matches!(
            rne_cfrc_plan(&dims, &bindings, &[proof.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));
        for invalid in [
            RneCfrcDims {
                abi: [1, 0, 0, 2],
                ..dims
            },
            RneCfrcDims {
                abi: [0, 0, 0, 0],
                ..dims
            },
            RneCfrcDims {
                abi: [0, 0, 0, u32::MAX - 254],
                ..dims
            },
        ] {
            assert!(rne_cfrc_lengths(&invalid).is_err());
        }
        let signed = RneCfrcDims {
            abi: [0, 0, 0, 1],
            static_dims: RneStaticDims {
                nbody: i32::MAX as u32 + 1,
                nv: 0,
                depth: 1,
                metadata_elements: 0,
            },
        };
        assert!(rne_cfrc_lengths(&signed).is_err());
        let wide = RneCfrcDims {
            abi: [0, 0, 0, u32::MAX - 255],
            static_dims: RneStaticDims {
                nbody: 400_000_000,
                nv: 0,
                depth: 1,
                metadata_elements: 1_600_000_005,
            },
        };
        assert!(matches!(
            rne_cfrc_lengths(&wide),
            Err(LauncherError::Overflow("rne_cfrc_com"))
        ));

        for (index, value) in [(2, f32::NAN.to_bits() as i32), (6, 1), (11, 2), (12, 2)] {
            let mut invalid = values.clone();
            invalid[index] = value;
            assert!(validate_rne_metadata(&invalid, static_dims).is_err());
        }
        let mut incomplete = values;
        incomplete[8] = 0;
        assert!(validate_rne_metadata(&incomplete, static_dims).is_err());
    }

    #[test]
    fn rne_cfrc_plan_accepts_zero_dof_boundary() {
        use crate::runtime::launcher::DeviceSpan;

        let dims = RneCfrcDims {
            abi: [0, 0, 0, 2],
            static_dims: RneStaticDims {
                nbody: 2,
                nv: 0,
                depth: 1,
                metadata_elements: 13,
            },
        };
        let values = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let revision = MetadataRevision::new();
        let span = DeviceSpan::new(13, 0x1000, 52).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof =
            unsafe { Checked::<RneCfrcMetadata>::assume_uploaded(span, &values, dims, revision) }
                .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(13, 0x2000, 256).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(13, 0x2000, 128).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(13, 0x3000, 224).unwrap(),
                elem: ElemType::F32,
            },
        ];
        assert_eq!(rne_cfrc_lengths(&dims).unwrap(), [13, 64, 32, 56]);
        rne_cfrc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    }

    #[test]
    fn rne_qfrc_plan_checks_metadata_spans_aliases_and_overflow() {
        use crate::runtime::launcher::DeviceSpan;

        let bind = |slot, context, addr, bytes, elem| SlotBinding {
            slot,
            span: DeviceSpan::new(context, addr, bytes).unwrap(),
            elem,
        };
        let static_dims = RneStaticDims {
            nbody: 2,
            nv: 1,
            depth: 1,
            metadata_elements: 14,
        };
        let dims = RneQfrcDims {
            abi: [0, 0, 0, 2],
            static_dims,
        };
        let values = vec![2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1];
        assert_eq!(rne_qfrc_lengths(&dims).unwrap(), [14, 76, 44, 58]);
        let revision = MetadataRevision::new();
        let metadata_span = DeviceSpan::new(7, 0x1000, 56).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof = unsafe {
            Checked::<RneQfrcMetadata>::assume_uploaded(
                metadata_span,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            bind(0, 7, 0x1000, 56, ElemType::I32),
            bind(1, 7, 0x2000, 304, ElemType::F32),
            bind(2, 7, 0x2000, 176, ElemType::F32),
            bind(3, 7, 0x4000, 232, ElemType::F32),
        ];
        let launch = rne_qfrc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
        assert_eq!(launch.grid(), (1, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));
        assert_eq!(launch.shared_bytes(), 0);
        assert!(matches!(
            rne_qfrc_plan(&dims, &bindings, &[]),
            Err(LauncherError::MissingMetadataProof { slot: 0 })
        ));

        let addresses = [0x1000, 0x2000, 0x2000, 0x4000];
        let bytes = [56, 304, 176, 232];
        for slot in 0..4 {
            let mut short = bindings;
            short[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] - 4).unwrap();
            assert!(matches!(
                rne_qfrc_plan(&dims, &short, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));

            let mut long = bindings;
            long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
            assert!(matches!(
                rne_qfrc_plan(&dims, &long, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));
        }

        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x2000, 176).unwrap();
        assert!(matches!(
            rne_qfrc_plan(&dims, &foreign, &[proof.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));
        let mut alias = bindings;
        alias[3].span = DeviceSpan::new(7, 0x2000, 232).unwrap();
        assert!(matches!(
            rne_qfrc_plan(&dims, &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3
            })
        ));

        revision.invalidate();
        assert!(matches!(
            rne_qfrc_plan(&dims, &bindings, &[proof.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));
        for invalid in [
            RneQfrcDims {
                abi: [1, 0, 0, 2],
                ..dims
            },
            RneQfrcDims {
                abi: [0, 0, 0, 0],
                ..dims
            },
            RneQfrcDims {
                abi: [0, 0, 0, u32::MAX - 255],
                ..dims
            },
        ] {
            assert!(rne_qfrc_lengths(&invalid).is_err());
        }
        assert!(rne_metadata_elements(i32::MAX as u32 + 1, 0).is_err());

        for (index, value) in [(2, f32::NAN.to_bits() as i32), (6, 1), (11, 2), (12, 2)] {
            let mut invalid = values.clone();
            invalid[index] = value;
            assert!(validate_rne_metadata(&invalid, static_dims).is_err());
        }
        let mut incomplete = values;
        incomplete[8] = 0;
        assert!(validate_rne_metadata(&incomplete, static_dims).is_err());
    }

    #[test]
    fn rne_qfrc_plan_accepts_zero_dof_boundary() {
        use crate::runtime::launcher::DeviceSpan;

        let dims = RneQfrcDims {
            abi: [0, 0, 0, 2],
            static_dims: RneStaticDims {
                nbody: 2,
                nv: 0,
                depth: 1,
                metadata_elements: 13,
            },
        };
        let values = vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let revision = MetadataRevision::new();
        let span = DeviceSpan::new(9, 0x1000, 52).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof =
            unsafe { Checked::<RneQfrcMetadata>::assume_uploaded(span, &values, dims, revision) }
                .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(9, 0x2000, 256).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(9, 0x2000, 128).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(9, 0x3000, 224).unwrap(),
                elem: ElemType::F32,
            },
        ];
        assert_eq!(rne_qfrc_lengths(&dims).unwrap(), [13, 64, 32, 56]);
        rne_qfrc_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    }

    #[test]
    fn rne_backward_plan_checks_levels_spans_aliases_and_overflow() {
        use crate::runtime::launcher::DeviceSpan;

        let bind = |slot, context, addr, bytes, elem| SlotBinding {
            slot,
            span: DeviceSpan::new(context, addr, bytes).unwrap(),
            elem,
        };
        let static_dims = RneStaticDims {
            nbody: 4,
            nv: 0,
            depth: 3,
            metadata_elements: 21,
        };
        let dims = RneBackwardDims {
            abi: [4, 3, 0, 8],
            worlds: 2,
            static_dims,
        };
        let values = vec![
            4, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3,
        ];
        assert_eq!(rne_backward_lengths(&dims).unwrap(), [21, 120, 56, 104]);
        let revision = MetadataRevision::new();
        let metadata_span = DeviceSpan::new(17, 0x1000, 84).unwrap();
        // SAFETY: 测试跨度代表同一份宿主快照。
        let proof = unsafe {
            Checked::<RneBackwardMetadata>::assume_uploaded(
                metadata_span,
                &values,
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            bind(0, 17, 0x1000, 84, ElemType::I32),
            bind(1, 17, 0x2000, 480, ElemType::F32),
            bind(2, 17, 0x3000, 224, ElemType::F32),
            bind(3, 17, 0x4000, 416, ElemType::F32),
        ];
        let launch = rne_backward_plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
        assert_eq!(launch.grid(), (1, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));
        assert_eq!(launch.shared_bytes(), 0);
        assert!(matches!(
            rne_backward_plan(&dims, &bindings, &[]),
            Err(LauncherError::MissingMetadataProof { slot: 0 })
        ));

        let addresses = [0x1000, 0x2000, 0x3000, 0x4000];
        let bytes = [84, 480, 224, 416];
        for slot in 0..4 {
            let mut short = bindings;
            short[slot].span = DeviceSpan::new(17, addresses[slot], bytes[slot] - 4).unwrap();
            assert!(matches!(
                rne_backward_plan(&dims, &short, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));

            let mut long = bindings;
            long[slot].span = DeviceSpan::new(17, addresses[slot], bytes[slot] + 4).unwrap();
            assert!(matches!(
                rne_backward_plan(&dims, &long, &[proof.binding(0)]),
                Err(LauncherError::Capacity { slot: bad, .. }) if bad == slot
            ));
        }

        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(18, 0x3000, 224).unwrap();
        assert!(matches!(
            rne_backward_plan(&dims, &foreign, &[proof.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));
        let mut alias = bindings;
        alias[3].span = DeviceSpan::new(17, 0x2000, 416).unwrap();
        assert!(matches!(
            rne_backward_plan(&dims, &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3
            })
        ));

        revision.invalidate();
        assert!(matches!(
            rne_backward_plan(&dims, &bindings, &[proof.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));
        for invalid in [
            RneBackwardDims {
                abi: [4, 0, 0, 8],
                ..dims
            },
            RneBackwardDims {
                abi: [4, 4, 0, 8],
                ..dims
            },
            RneBackwardDims {
                abi: [3, 3, 0, 8],
                ..dims
            },
            RneBackwardDims {
                abi: [4, 3, 1, 8],
                ..dims
            },
            RneBackwardDims {
                abi: [4, 3, 0, 7],
                ..dims
            },
            RneBackwardDims { worlds: 0, ..dims },
        ] {
            assert!(rne_backward_lengths(&invalid).is_err());
        }
        let overflow = RneBackwardDims {
            abi: [500_000_000, 1, 0, 0],
            worlds: 9,
            static_dims: RneStaticDims {
                nbody: 500_000_000,
                nv: 0,
                depth: 1,
                metadata_elements: 2_000_000_005,
            },
        };
        assert!(matches!(
            rne_backward_lengths(&overflow),
            Err(LauncherError::Overflow("rne_backward_work"))
        ));
        let max_work = u32::MAX - 255;
        let max_grid = RneBackwardDims {
            abi: [2, 1, 0, max_work],
            worlds: max_work / 2,
            static_dims: RneStaticDims {
                nbody: 2,
                nv: 0,
                depth: 1,
                metadata_elements: 13,
            },
        };
        assert!(rne_backward_lengths(&max_grid).is_ok());
        assert_eq!(
            RneBackwardSpec::launch_dims(&max_grid).unwrap().grid(),
            (16_777_215, 1, 1)
        );
        let beyond_grid = RneBackwardDims {
            abi: [2, 1, 0, max_work + 2],
            worlds: max_work / 2 + 1,
            ..max_grid
        };
        assert!(matches!(
            rne_backward_lengths(&beyond_grid),
            Err(LauncherError::Overflow("rne_backward_work"))
        ));
        let byte_overflow = RneBackwardDims {
            abi: [2, 1, 0, max_work],
            worlds: max_work / 2,
            static_dims: RneStaticDims {
                nbody: 2,
                nv: 600_000_000,
                depth: 1,
                metadata_elements: 600_000_013,
            },
        };
        assert!(rne_backward_lengths(&byte_overflow).is_ok());
        let byte_bindings = [
            bind(0, 17, 0x1000, 2_400_000_052, ElemType::I32),
            bind(1, 17, 0x1_0000_0000, 4, ElemType::F32),
            bind(2, 17, 0x2_0000_0000, 4, ElemType::F32),
            bind(3, 17, 0x3_0000_0000, 4, ElemType::F32),
        ];
        assert!(matches!(
            rne_backward_plan(&byte_overflow, &byte_bindings, &[]),
            Err(LauncherError::Overflow("rne_backward_bytes"))
        ));
        let root_only = RneBackwardDims {
            abi: [1, 0, 0, 1],
            worlds: 1,
            static_dims: RneStaticDims {
                nbody: 1,
                nv: 0,
                depth: 0,
                metadata_elements: 9,
            },
        };
        assert!(rne_backward_lengths(&root_only).is_err());
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn runs_rne_backward_wrapper_in_reverse_levels_and_recovers() {
        let session = TransferSession::new(0).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/rne-bias/mixed-joints.json")).unwrap(),
        )
        .unwrap();
        let model = reference_model(&fixture);
        let mut plan = RnePlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
        assert_eq!((plan.nbody, plan.nv, plan.depth), (6, 11, 4));
        let parents = [0usize, 0, 1, 2, 3, 0];
        let depths = [0usize, 1, 2, 3, 4, 1];
        assert_eq!(
            plan.metadata.values[5..5 + plan.nbody],
            parents.map(|value| value as i32)
        );
        const WORLDS: usize = 43;
        let mut data = plan.create_data(WORLDS).unwrap();
        data.write_qvel(&vec![0.0; WORLDS * plan.nv]).unwrap();
        plan.update_velocity(&mut data).unwrap();

        let width = 12 * plan.nbody + plan.nv;
        let mut initial = vec![0.0; data.output.len()];
        initial[..4].fill(-131072.0);
        let guard_end = initial.len() - 4;
        initial[guard_end..].fill(-131072.0);
        for world in 0..WORLDS {
            let force = 4 + world * width + 6 * plan.nbody;
            for body in 0..plan.nbody {
                for lane in 0..6 {
                    initial[force + 6 * body + lane] =
                        (world * 1_000 + body * 16 + lane + 1) as f32;
                }
            }
        }
        let mut expected = initial.clone();
        for level in (1..=plan.depth).rev() {
            for body in 1..plan.nbody {
                if depths[body] != level {
                    continue;
                }
                for world in 0..WORLDS {
                    let force = 4 + world * width + 6 * plan.nbody;
                    for lane in 0..6 {
                        let child = expected[force + 6 * body + lane];
                        expected[force + 6 * parents[body] + lane] += child;
                    }
                }
            }
        }

        let bits = |values: &[f32]| {
            values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        };
        let (raw, typed) = {
            let inputs = data.velocity.rne_inputs().unwrap();
            data.output.write_range(0, &initial).unwrap();
            // SAFETY: 复用产品计划的真实缓冲和冻结私有ABI。
            unsafe {
                for level in (1..=plan.depth).rev() {
                    plan.backward_kernel
                        .kernel
                        .launch(
                            plan.metadata.as_buffer(),
                            inputs.com,
                            inputs.velocity,
                            &mut data.output,
                            [
                                plan.nbody as u32,
                                level as u32,
                                0,
                                (WORLDS * plan.nbody) as u32,
                            ],
                        )
                        .unwrap();
                }
            }
            let mut raw = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut raw).unwrap();

            data.output.write_range(0, &initial).unwrap();
            let mut launches = Vec::with_capacity(plan.depth);
            for level in (1..=plan.depth).rev() {
                launches.push(
                    plan.backward_kernel
                        .preflight(
                            &plan.session,
                            &plan.metadata,
                            inputs.com,
                            inputs.velocity,
                            &data.output,
                            RneBackwardRequest {
                                worlds: WORLDS as u32,
                                level: level as u32,
                            },
                        )
                        .unwrap(),
                );
            }
            for launch in &launches {
                plan.backward_kernel
                    .launch(
                        launch,
                        &plan.metadata,
                        inputs.com,
                        inputs.velocity,
                        &mut data.output,
                    )
                    .unwrap();
            }
            let mut typed = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut typed).unwrap();

            let short_com = session.upload(&vec![0.0; inputs.com.len() - 1]).unwrap();
            assert!(
                plan.backward_kernel
                    .preflight(
                        &plan.session,
                        &plan.metadata,
                        &short_com,
                        inputs.velocity,
                        &data.output,
                        RneBackwardRequest {
                            worlds: WORLDS as u32,
                            level: plan.depth as u32,
                        },
                    )
                    .is_err()
            );
            let proof = plan
                .metadata
                .backward_proof(WORLDS as u32, plan.depth as u32)
                .unwrap();
            let mut alias = RneBackwardKernel::bindings(
                &plan.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
            )
            .unwrap();
            let output_bytes = rne_backward_lengths(proof.dims()).unwrap()[3] * 4;
            alias[3].span = alias[1].span.view(0, output_bytes).unwrap();
            assert!(matches!(
                rne_backward_plan(proof.dims(), &alias, &[proof.binding(0)]),
                Err(LauncherError::Alias {
                    first: 1,
                    second: 3
                })
            ));

            let stale_launch = plan
                .backward_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    RneBackwardRequest {
                        worlds: WORLDS as u32,
                        level: plan.depth as u32,
                    },
                )
                .unwrap();
            let mut before_stale = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut before_stale).unwrap();
            plan.metadata.overwrite_for_test(0, 1).unwrap();
            assert!(
                plan.backward_kernel
                    .launch(
                        &stale_launch,
                        &plan.metadata,
                        inputs.com,
                        inputs.velocity,
                        &mut data.output,
                    )
                    .is_err()
            );
            let mut rejected_stale = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut rejected_stale).unwrap();
            assert_eq!(bits(&before_stale), bits(&rejected_stale));
            plan.metadata.restore_for_test().unwrap();
            let recovered_launch = plan
                .backward_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    RneBackwardRequest {
                        worlds: WORLDS as u32,
                        level: plan.depth as u32,
                    },
                )
                .unwrap();
            data.output.write_range(0, &initial).unwrap();
            plan.backward_kernel
                .launch(
                    &recovered_launch,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )
                .unwrap();
            (raw, typed)
        };
        assert_eq!(bits(&raw), bits(&typed));
        assert_eq!(bits(&raw), bits(&expected));

        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();
        let mut before = vec![0.0; data.output.len()];
        before[..4].fill(-131072.0);
        let end = before.len() - 4;
        before[end..].fill(-131072.0);
        for (index, value) in before[4..end].iter_mut().enumerate() {
            *value = -4096.0 + (index % 31) as f32 * 0.125;
        }
        data.output.write_range(0, &before).unwrap();

        let foreign_session = TransferSession::new(0).unwrap();
        let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
        if let Some(root) = capture.as_ref() {
            let foreign = std::path::PathBuf::from(root).join("foreign-session");
            assert!(!foreign.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
        }
        let foreign_kernel = RneBackwardKernel::compile(&foreign_session).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        } else {
            // SAFETY: 本测试单线程运行，调用前也没有该变量。
            unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
        }
        let correct_kernel = std::mem::replace(&mut plan.backward_kernel, foreign_kernel);
        assert!(matches!(
            plan.update_rne(&mut data, true),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne" })
        ));
        let mut rejected = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut rejected).unwrap();
        assert_eq!(bits(&before), bits(&rejected));
        let foreign_kernel = std::mem::replace(&mut plan.backward_kernel, correct_kernel);
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();

        use crate::model::{
            InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
        };
        let root_model = KinematicModelInput::new(
            0,
            KinematicFields {
                body_parentid: vec![0],
                body_jntadr: vec![-1],
                body_jntnum: vec![0],
                body_pos: vec![0.0; 3],
                body_quat: vec![1.0, 0.0, 0.0, 0.0],
                ..Default::default()
            },
        )
        .unwrap();
        let root_model = InertialModelInput::new(
            root_model,
            InertialFields {
                body_ipos: vec![0.0; 3],
                body_iquat: vec![1.0, 0.0, 0.0, 0.0],
                body_mass: vec![0.0],
                body_inertia: vec![0.0; 3],
                ..Default::default()
            },
        )
        .unwrap();
        let root_model =
            AttachedModelInput::new(root_model, crate::model::AttachedFields::default()).unwrap();
        if let Some(root) = capture.as_ref() {
            let root_only = std::path::PathBuf::from(root).join("root-only");
            assert!(!root_only.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &root_only) };
        }
        let mut root_plan = RnePlan::new(&session, root_model, [0.0; 3]).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        } else {
            // SAFETY: 本测试单线程运行，调用前也没有该变量。
            unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
        }
        assert_eq!(root_plan.depth, 0);
        root_plan.backward_kernel = foreign_kernel;
        let mut root_data = root_plan.create_data(2).unwrap();
        root_plan.update(&mut root_data, false).unwrap();
        root_data.readback().unwrap();
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn runs_rne_cacc_wrapper_and_recovers_after_real_metadata_write() {
        let session = TransferSession::new(0).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/rne-bias/mixed-joints.json")).unwrap(),
        )
        .unwrap();
        let model = reference_model(&fixture);
        assert_eq!(
            model.rigid().kinematics().fields().body_parentid,
            [0, 0, 1, 2, 3, 0]
        );
        let mut plan = RnePlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
        assert_eq!((plan.nbody, plan.nv), (6, 11));
        const WORLDS: usize = 513;
        let mut data = plan.create_data(WORLDS).unwrap();
        let qvel: Vec<_> = (0..WORLDS * plan.nv)
            .map(|index| (index as f32 - 2821.0) * 0.000_125)
            .collect();
        let qacc: Vec<_> = (0..WORLDS * plan.nv)
            .map(|index| (2821.0 - index as f32) * 0.000_25)
            .collect();
        data.write_qvel(&qvel).unwrap();
        data.write_qacc(&qacc).unwrap();
        plan.update_velocity(&mut data).unwrap();
        let width = 12 * plan.nbody + plan.nv;
        let cacc_width = 6 * plan.nbody;
        let sentinel = -777.25f32;
        let mut initial = vec![sentinel; data.output.len()];
        initial[..4].fill(-131072.0);
        let end = initial.len() - 4;
        initial[end..].fill(-131072.0);

        for flg_acc in [false, true] {
            let (raw, typed) = {
                let inputs = data.velocity.rne_inputs().unwrap();
                data.output.write_range(0, &initial).unwrap();
                // SAFETY: 复用产品计划的同一实际缓冲和冻结私有ABI。
                unsafe {
                    plan.cacc_kernel.kernel.launch_rne_cacc(
                        plan.metadata.as_buffer(),
                        (inputs.qvel, &data.qacc, inputs.com, inputs.velocity),
                        &mut data.output,
                        flg_acc,
                        WORLDS as u32,
                    )
                }
                .unwrap();
                let mut raw = vec![0.0; data.output.len()];
                data.output.read_range_into(0, &mut raw).unwrap();

                data.output.write_range(0, &initial).unwrap();
                let launch = plan
                    .cacc_kernel
                    .preflight(
                        &plan.session,
                        &plan.metadata,
                        inputs.qvel,
                        &data.qacc,
                        inputs.com,
                        inputs.velocity,
                        &data.output,
                        flg_acc,
                        WORLDS as u32,
                    )
                    .unwrap();
                plan.cacc_kernel
                    .launch(
                        &launch,
                        &plan.metadata,
                        inputs.qvel,
                        &data.qacc,
                        inputs.com,
                        inputs.velocity,
                        &mut data.output,
                    )
                    .unwrap();
                let mut typed = vec![0.0; data.output.len()];
                data.output.read_range_into(0, &mut typed).unwrap();
                (raw, typed)
            };
            assert_eq!(
                raw.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
                typed
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            );
            assert_eq!(&raw[..4], &initial[..4]);
            assert_eq!(&raw[raw.len() - 4..], &initial[initial.len() - 4..]);
            for world in 0..WORLDS {
                let start = 4 + world * width;
                assert!(
                    raw[start..start + cacc_width]
                        .iter()
                        .all(|value| value.to_bits() != sentinel.to_bits())
                );
                assert!(
                    raw[start + cacc_width..start + width]
                        .iter()
                        .all(|value| value.to_bits() == sentinel.to_bits())
                );
            }
        }

        {
            let inputs = data.velocity.rne_inputs().unwrap();
            let short_qacc = session.upload(&vec![0.0; data.qacc.len() - 1]).unwrap();
            assert!(
                plan.cacc_kernel
                    .preflight(
                        &plan.session,
                        &plan.metadata,
                        inputs.qvel,
                        &short_qacc,
                        inputs.com,
                        inputs.velocity,
                        &data.output,
                        true,
                        WORLDS as u32,
                    )
                    .is_err()
            );
            let proof = plan.metadata.cacc_proof(true, WORLDS as u32).unwrap();
            let mut alias = RneCaccKernel::bindings(
                &plan.metadata,
                inputs.qvel,
                &data.qacc,
                inputs.com,
                inputs.velocity,
                &data.output,
            )
            .unwrap();
            let output_bytes = rne_cacc_lengths(proof.dims()).unwrap()[5] * 4;
            alias[5].span = alias[3].span.view(0, output_bytes).unwrap();
            assert!(matches!(
                rne_cacc_plan(proof.dims(), &alias, &[proof.binding(0)]),
                Err(LauncherError::Alias {
                    first: 3,
                    second: 5
                })
            ));
        }

        let sentinel_workspace = |data: &mut RneData, seed: f32| {
            let mut values = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut values).unwrap();
            let end = values.len() - 4;
            for (index, value) in values[4..end].iter_mut().enumerate() {
                *value = seed + (index % 31) as f32 * 0.125;
            }
            data.output.write_range(0, &values).unwrap();
            values
        };
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();
        let before = sentinel_workspace(&mut data, -4096.0);
        plan.metadata.overwrite_for_test(0, 1).unwrap();
        assert!(plan.update_rne(&mut data, true).is_err());
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne" })
        ));
        let mut rejected = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut rejected).unwrap();
        assert_eq!(
            before
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            rejected
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.metadata.restore_for_test().unwrap();
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();

        let before_session = sentinel_workspace(&mut data, -8192.0);
        let foreign_session = TransferSession::new(0).unwrap();
        let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
        if let Some(root) = capture.as_ref() {
            let foreign = std::path::PathBuf::from(root).join("foreign-session");
            assert!(!foreign.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
        }
        let foreign_kernel = RneCaccKernel::compile(&foreign_session).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        } else {
            // SAFETY: 本测试单线程运行，调用前也没有该变量。
            unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
        }
        let correct_kernel = std::mem::replace(&mut plan.cacc_kernel, foreign_kernel);
        assert!(matches!(
            plan.update_rne(&mut data, true),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne" })
        ));
        let mut rejected_session = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut rejected_session)
            .unwrap();
        assert_eq!(
            before_session
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            rejected_session
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.cacc_kernel = correct_kernel;
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();

        let zero = super::super::tests::test_model(None);
        let zero = AttachedModelInput::new(zero, crate::model::AttachedFields::default()).unwrap();
        if let Some(root) = capture.as_ref() {
            let zero_capture = std::path::PathBuf::from(root).join("zero-dof");
            assert!(!zero_capture.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &zero_capture) };
        }
        let zero_plan = RnePlan::new(&session, zero, [0.0; 3]).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        }
        let mut zero_data = zero_plan.create_data(2).unwrap();
        zero_plan.update(&mut zero_data, false).unwrap();
        let zero_output = zero_data.readback().unwrap();
        for world in 0..2 {
            assert!(zero_output.world(world).unwrap().qfrc_bias.is_empty());
        }
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn runs_rne_cfrc_wrapper_and_recovers_after_real_metadata_write() {
        let session = TransferSession::new(0).unwrap();
        let model = super::super::tests::test_model(Some(3));
        let model =
            AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
        let mut plan = RnePlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
        const WORLDS: usize = 513;
        let mut data = plan.create_data(WORLDS).unwrap();
        let qvel: Vec<_> = (0..WORLDS)
            .map(|world| (world as f32 - 256.0) * 0.000_125)
            .collect();
        let qacc: Vec<_> = (0..WORLDS)
            .map(|world| (256.0 - world as f32) * 0.000_25)
            .collect();
        data.write_qvel(&qvel).unwrap();
        data.write_qacc(&qacc).unwrap();
        plan.update_velocity(&mut data).unwrap();
        let width = 12 * plan.nbody + plan.nv;
        let cfrc_offset = 6 * plan.nbody;
        let sentinel = -777.25f32;
        let mut initial = vec![sentinel; data.output.len()];
        initial[..4].fill(-131072.0);
        let end = initial.len() - 4;
        initial[end..].fill(-131072.0);

        let (raw, typed) = {
            let inputs = data.velocity.rne_inputs().unwrap();
            data.output.write_range(0, &initial).unwrap();
            // SAFETY: 先用冻结旧ABI生成相同cacc前缀。
            unsafe {
                plan.cacc_kernel.kernel.launch_rne_cacc(
                    plan.metadata.as_buffer(),
                    (inputs.qvel, &data.qacc, inputs.com, inputs.velocity),
                    &mut data.output,
                    true,
                    WORLDS as u32,
                )
            }
            .unwrap();
            let mut cacc_ready = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut cacc_ready).unwrap();

            data.output.write_range(0, &cacc_ready).unwrap();
            // SAFETY: 复用产品计划的同一实际缓冲和冻结私有ABI。
            unsafe {
                plan.cfrc_kernel.kernel.launch(
                    plan.metadata.as_buffer(),
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                    [0, 0, 0, WORLDS as u32],
                )
            }
            .unwrap();
            let mut raw = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut raw).unwrap();

            data.output.write_range(0, &cacc_ready).unwrap();
            let launch = plan
                .cfrc_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    WORLDS as u32,
                )
                .unwrap();
            plan.cfrc_kernel
                .launch(
                    &launch,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )
                .unwrap();
            let mut typed = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut typed).unwrap();

            let short_velocity = session
                .upload(&vec![0.0; inputs.velocity.len() - 1])
                .unwrap();
            assert!(
                plan.cfrc_kernel
                    .preflight(
                        &plan.session,
                        &plan.metadata,
                        inputs.com,
                        &short_velocity,
                        &data.output,
                        WORLDS as u32,
                    )
                    .is_err()
            );
            let proof = plan.metadata.cfrc_proof(WORLDS as u32).unwrap();
            let mut alias =
                RneCfrcKernel::bindings(&plan.metadata, inputs.com, inputs.velocity, &data.output)
                    .unwrap();
            let output_bytes = rne_cfrc_lengths(proof.dims()).unwrap()[3] * 4;
            alias[3].span = alias[1].span.view(0, output_bytes).unwrap();
            assert!(matches!(
                rne_cfrc_plan(proof.dims(), &alias, &[proof.binding(0)]),
                Err(LauncherError::Alias {
                    first: 1,
                    second: 3
                })
            ));
            (raw, typed)
        };
        assert_eq!(
            raw.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            typed
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        for world in 0..WORLDS {
            for lane in 0..6 * plan.nbody {
                assert_ne!(
                    raw[4 + world * width + cfrc_offset + lane].to_bits(),
                    sentinel.to_bits()
                );
            }
        }

        let sentinel_workspace = |data: &mut RneData, seed: f32| {
            let mut values = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut values).unwrap();
            let end = values.len() - 4;
            for (index, value) in values[4..end].iter_mut().enumerate() {
                *value = seed + (index % 31) as f32 * 0.125;
            }
            data.output.write_range(0, &values).unwrap();
            values
        };
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();
        let before = sentinel_workspace(&mut data, -4096.0);
        plan.metadata.overwrite_for_test(0, 1).unwrap();
        assert!(plan.update_rne(&mut data, true).is_err());
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne" })
        ));
        let mut rejected = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut rejected).unwrap();
        assert_eq!(
            before
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            rejected
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.metadata.restore_for_test().unwrap();
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();

        let before_session = sentinel_workspace(&mut data, -8192.0);
        let foreign_session = TransferSession::new(0).unwrap();
        let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
        if let Some(root) = capture.as_ref() {
            let foreign = std::path::PathBuf::from(root).join("foreign-session");
            assert!(!foreign.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
        }
        let foreign_kernel = RneCfrcKernel::compile(&foreign_session).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        } else {
            // SAFETY: 本测试单线程运行，调用前也没有该变量。
            unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
        }
        let correct_kernel = std::mem::replace(&mut plan.cfrc_kernel, foreign_kernel);
        assert!(matches!(
            plan.update_rne(&mut data, true),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne" })
        ));
        let mut rejected_session = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut rejected_session)
            .unwrap();
        assert_eq!(
            before_session
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            rejected_session
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.cfrc_kernel = correct_kernel;
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();

        let zero = super::super::tests::test_model(None);
        let zero = AttachedModelInput::new(zero, crate::model::AttachedFields::default()).unwrap();
        if let Some(root) = capture.as_ref() {
            let zero_capture = std::path::PathBuf::from(root).join("zero-dof");
            assert!(!zero_capture.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &zero_capture) };
        }
        let zero_plan = RnePlan::new(&session, zero, [0.0; 3]).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        }
        let mut zero_data = zero_plan.create_data(2).unwrap();
        zero_plan.update(&mut zero_data, false).unwrap();
        let zero_output = zero_data.readback().unwrap();
        for world in 0..2 {
            assert!(zero_output.world(world).unwrap().qfrc_bias.is_empty());
        }
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn runs_rne_qfrc_wrapper_and_recovers_after_real_metadata_write() {
        let session = TransferSession::new(0).unwrap();
        let model = super::super::tests::test_model(Some(3));
        let model =
            AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
        let mut plan = RnePlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
        const WORLDS: usize = 513;
        let mut data = plan.create_data(WORLDS).unwrap();
        let qvel: Vec<_> = (0..WORLDS)
            .map(|world| (world as f32 - 256.0) * 0.000_125)
            .collect();
        let qacc: Vec<_> = (0..WORLDS)
            .map(|world| (256.0 - world as f32) * 0.000_25)
            .collect();
        data.write_qvel(&qvel).unwrap();
        data.write_qacc(&qacc).unwrap();
        plan.update(&mut data, true).unwrap();
        let mut initial = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut initial).unwrap();
        let width = 12 * plan.nbody + plan.nv;
        let qfrc_offset = 12 * plan.nbody;
        let sentinel = -777.25f32;
        for world in 0..WORLDS {
            initial[4 + world * width + qfrc_offset] = sentinel;
        }

        let (raw, typed) = {
            let inputs = data.velocity.rne_inputs().unwrap();
            data.output.write_range(0, &initial).unwrap();
            // SAFETY: 复用产品计划的同一实际缓冲和冻结私有ABI。
            unsafe {
                plan.qfrc_kernel.kernel.launch(
                    plan.metadata.as_buffer(),
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                    [0, 0, 0, WORLDS as u32],
                )
            }
            .unwrap();
            let mut raw = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut raw).unwrap();

            data.output.write_range(0, &initial).unwrap();
            let launch = plan
                .qfrc_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    WORLDS as u32,
                )
                .unwrap();
            plan.qfrc_kernel
                .launch(
                    &launch,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )
                .unwrap();
            let mut typed = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut typed).unwrap();

            let short_com = session.upload(&vec![0.0; inputs.com.len() - 1]).unwrap();
            assert!(
                plan.qfrc_kernel
                    .preflight(
                        &plan.session,
                        &plan.metadata,
                        &short_com,
                        inputs.velocity,
                        &data.output,
                        WORLDS as u32,
                    )
                    .is_err()
            );
            let proof = plan.metadata.qfrc_proof(WORLDS as u32).unwrap();
            let mut alias =
                RneQfrcKernel::bindings(&plan.metadata, inputs.com, inputs.velocity, &data.output)
                    .unwrap();
            let output_bytes = rne_qfrc_lengths(proof.dims()).unwrap()[3] * 4;
            alias[3].span = alias[1].span.view(0, output_bytes).unwrap();
            assert!(matches!(
                rne_qfrc_plan(proof.dims(), &alias, &[proof.binding(0)]),
                Err(LauncherError::Alias {
                    first: 1,
                    second: 3
                })
            ));
            (raw, typed)
        };
        assert_eq!(
            raw.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            typed
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        for world in 0..WORLDS {
            assert_ne!(
                raw[4 + world * width + qfrc_offset].to_bits(),
                sentinel.to_bits()
            );
        }

        let sentinel_workspace = |data: &mut RneData, seed: f32| {
            let mut values = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut values).unwrap();
            let end = values.len() - 4;
            for (index, value) in values[4..end].iter_mut().enumerate() {
                *value = seed + (index % 31) as f32 * 0.125;
            }
            data.output.write_range(0, &values).unwrap();
            values
        };
        let before = sentinel_workspace(&mut data, -4096.0);
        plan.metadata.overwrite_for_test(0, 1).unwrap();
        assert!(plan.update_rne(&mut data, true).is_err());
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne" })
        ));
        let mut rejected = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut rejected).unwrap();
        assert_eq!(
            before.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            rejected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        plan.metadata.restore_for_test().unwrap();
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();

        let before_session = sentinel_workspace(&mut data, -8192.0);
        let foreign_session = TransferSession::new(0).unwrap();
        let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
        if let Some(root) = capture.as_ref() {
            let foreign = std::path::PathBuf::from(root).join("foreign-session");
            assert!(!foreign.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
        }
        let foreign_kernel = RneQfrcKernel::compile(&foreign_session).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        } else {
            // SAFETY: 本测试单线程运行，调用前也没有该变量。
            unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
        }
        let correct_kernel = std::mem::replace(&mut plan.qfrc_kernel, foreign_kernel);
        assert!(matches!(
            plan.update_rne(&mut data, true),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rne" })
        ));
        let mut rejected_session = vec![0.0; data.output.len()];
        data.output
            .read_range_into(0, &mut rejected_session)
            .unwrap();
        assert_eq!(
            before_session
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            rejected_session
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        plan.qfrc_kernel = correct_kernel;
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();

        let zero = super::super::tests::test_model(None);
        let zero = AttachedModelInput::new(zero, crate::model::AttachedFields::default()).unwrap();
        if let Some(root) = capture.as_ref() {
            let zero_capture = std::path::PathBuf::from(root).join("zero-dof");
            assert!(!zero_capture.exists());
            // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &zero_capture) };
        }
        let zero_plan = RnePlan::new(&session, zero, [0.0; 3]).unwrap();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        }
        let mut zero_data = zero_plan.create_data(2).unwrap();
        zero_plan.update(&mut zero_data, false).unwrap();
        let zero_output = zero_data.readback().unwrap();
        for world in 0..2 {
            assert!(zero_output.world(world).unwrap().qfrc_bias.is_empty());
        }
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn checks_acceleration_guards_and_unpublished_nonfinite_workspaces() {
        let model = super::super::tests::test_model(Some(3));
        let model =
            AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
        let session = TransferSession::new(0).unwrap();
        let plan = RnePlan::new(&session, model, [0.0; 3]).unwrap();
        let mut data = plan.create_data(2).unwrap();
        data.write_qacc(&[0.25, -0.5]).unwrap();
        plan.update(&mut data, true).unwrap();
        for index in [0, data.layout.guarded - 1] {
            data.output.write_range(index, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            data.output.write_range(index, &[-131072.0]).unwrap();
        }
        for index in [4, 4 + 6 * data.layout.nbody, 4 + 12 * data.layout.nbody] {
            data.output.write_range(index, &[f32::NAN]).unwrap();
            assert!(data.readback().is_err());
            plan.update_rne(&mut data, true).unwrap();
        }
        for index in [0, data.layout.qacc.total_elements() + 7] {
            data.qacc.write_range(index, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            data.qacc.write_range(index, &[-131072.0]).unwrap();
        }
        data.qacc.write_range(4, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        data.write_world_qacc(0, &[0.25]).unwrap();
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and pinned frozen smooth PTX"]
    fn diagnoses_new_qacc_stage_against_frozen_ptx_on_identical_device_inputs() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let base: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/rne-bias/mixed-joints.json")).unwrap(),
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/rne-acceleration/mixed-joints.json"))
                .unwrap(),
        )
        .unwrap();
        let model = crate::physics::rne::tests::reference_model(&base);
        let session = TransferSession::new(0).unwrap();
        let mut rows = Vec::new();
        for (state, g, gravity) in [(3, 0, [0.0, 0.0, -9.81]), (7, 2, [1.25, -2.0, 3.5])] {
            let plan = RnePlan::new(&session, model.clone(), gravity).unwrap();
            let mut data = plan.create_data(1).unwrap();
            let values = |key: &str| {
                v["cases"][state][key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n.as_f64().unwrap() as f32)
                    .collect::<Vec<_>>()
            };
            data.write_qpos(&values("qpos")).unwrap();
            data.write_qvel(&values("qvel")).unwrap();
            data.write_qacc(&values("qacc")).unwrap();
            plan.update_velocity(&mut data).unwrap();
            for flag in [false, true] {
                plan.update_rne(&mut data, flag).unwrap();
                let candidate = data.readback().unwrap();
                let inputs = data.velocity.rne_inputs().unwrap();
                let mut m = vec![0; 5 + 4 * plan.nbody + plan.nv];
                plan.metadata
                    .as_buffer()
                    .read_range_into(0, &mut m)
                    .unwrap();
                let mut qvel = vec![0.0; plan.nv];
                let mut qacc = vec![0.0; plan.nv];
                let mut com = vec![0.0; 14 * plan.nbody + 6 * plan.nv];
                let mut velocity = vec![0.0; 6 * plan.nbody + 6 * plan.nv];
                inputs.qvel.read_range_into(4, &mut qvel).unwrap();
                data.qacc.read_range_into(4, &mut qacc).unwrap();
                inputs.com.read_range_into(4, &mut com).unwrap();
                inputs.velocity.read_range_into(4, &mut velocity).unwrap();
                let frozen = frozen::run(&m, &qvel, &qacc, &com, &velocity, flag);
                let actual = &candidate.values[4..candidate.values.len() - 4];
                let differences: Vec<_> = actual.iter().zip(&frozen).enumerate().filter(|(_, (a, b))| a.to_bits() != b.to_bits())
                .map(|(i, (&a, &b))| serde_json::json!({"index":i,"candidate":a,"frozen_stage":b,"candidate_bits":a.to_bits(),"frozen_bits":b.to_bits()})).collect();
                let native = &v["cases"][state]["qfrc"][usize::from(flag)][g];
                rows.push(serde_json::json!({"state":state,"gravity":g,"flag":flag,"differences":differences,"candidate_qfrc":candidate.world(0).unwrap().qfrc_bias,
                "frozen_stage_qfrc":&frozen[12 * plan.nbody..],"native_qfrc":native}));
            }
        }
        let result = serde_json::json!({"scope":"new qacc first-difference; frozen RNE consumes identical product-prefix f32 inputs; not full upstream pipeline or admission", "model":"mixed-joints","rows":rows});
        std::fs::write(
            root.join("target/rne-acceleration-validation/frozen-first-difference.json"),
            serde_json::to_string_pretty(&result).unwrap(),
        )
        .unwrap();
        eprintln!("new qacc frozen stage diagnostic: {}", result);
        assert!(
            rows.iter()
                .all(|row| row["differences"].as_array().unwrap().is_empty()),
            "local RNE stage bit differences remain"
        );
    }

    #[cfg(feature = "cuda-probe")]
    fn reference_model(v: &serde_json::Value) -> AttachedModelInput {
        use crate::model::{
            InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
        };
        let m = &v["model"];
        let f = |key: &str| {
            m[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect()
        };
        let i = |key: &str| {
            m[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
                .collect()
        };
        let k = KinematicModelInput::new(
            v["nv"].as_u64().unwrap() as usize,
            KinematicFields {
                qpos0: f("qpos0"),
                body_parentid: i("body_parentid"),
                body_jntadr: i("body_jntadr"),
                body_jntnum: i("body_jntnum"),
                body_pos: f("body_pos"),
                body_quat: f("body_quat"),
                jnt_type: i("jnt_type"),
                jnt_bodyid: i("jnt_bodyid"),
                jnt_qposadr: i("jnt_qposadr"),
                jnt_dofadr: i("jnt_dofadr"),
                jnt_pos: f("jnt_pos"),
                jnt_axis: f("jnt_axis"),
            },
        )
        .unwrap();
        let model = InertialModelInput::new(
            k,
            InertialFields {
                body_ipos: f("body_ipos"),
                body_iquat: f("body_iquat"),
                body_mass: f("body_mass"),
                body_inertia: f("body_inertia"),
                dof_bodyid: i("dof_bodyid"),
                dof_jntid: i("dof_jntid"),
                dof_parentid: i("dof_parentid"),
                dof_armature: f("dof_armature"),
                dof_damping: f("dof_damping"),
            },
        )
        .unwrap();
        AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap()
    }
}
