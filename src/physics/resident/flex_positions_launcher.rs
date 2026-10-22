//! 常驻柔体失效与位置 FIELD_BATCH 变体的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称；失效与位置共用同一份柔体元数据。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_flex_positions.metadata.v1");

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
const fn meta_slot() -> SlotDesc {
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing { proof: PROOF },
    }
}

#[cfg(feature = "cuda-probe")]
const fn out_slot(index: usize, elem: ElemType) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::DevicePtr { elem },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    }
}

/// flex_invalidate：元数据、参数、刚体输入与i32失效标志，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const INVALIDATE_SLOTS: [SlotDesc; 8] = [
    meta_slot(),
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::F32),
    out_slot(3, ElemType::I32),
    scalar_slot(4),
    scalar_slot(5),
    scalar_slot(6),
    scalar_slot(7),
];

/// flex_positions：元数据、参数、刚体输入与f32节点/顶点输出，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const POSITIONS_SLOTS: [SlotDesc; 8] = [
    meta_slot(),
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::F32),
    out_slot(3, ElemType::F32),
    scalar_slot(4),
    scalar_slot(5),
    scalar_slot(6),
    scalar_slot(7),
];

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlexStaticDims {
    pub(super) nf: u32,
    pub(super) nn: u32,
    pub(super) nv: u32,
    pub(super) nb: u32,
    pub(super) rigid_stride: u32,
    pub(super) parameters_len: u32,
}

#[cfg(feature = "cuda-probe")]
impl FlexStaticDims {
    /// 由柔体、节点、顶点与刚体计数构造。
    pub(super) fn new(
        nf: usize,
        nn: usize,
        nv: usize,
        nb: usize,
        rigid_stride: usize,
        parameters_len: usize,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nf: value("resident_flex_nf", nf)?,
            nn: value("resident_flex_nn", nn)?,
            nv: value("resident_flex_nv", nv)?,
            nb: value("resident_flex_nb", nb)?,
            rigid_stride: value("resident_flex_rigid", rigid_stride)?,
            parameters_len: value("resident_flex_parameters", parameters_len)?,
        };
        if dims.nf == 0 || dims.nb == 0 {
            return Err(LauncherError::InvalidArray("常驻柔体维度无效"));
        }
        let required = dims.parameters_required()?;
        if u64::from(dims.parameters_len) != required {
            return Err(LauncherError::InvalidArray("常驻柔体参数跨度不符"));
        }
        if u64::from(dims.rigid_stride) < 16 * u64::from(dims.nb) {
            return Err(LauncherError::InvalidArray("常驻柔体刚体跨度不足"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：五项头、每柔体九项、节点体与顶点体。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.nf)
            .checked_mul(9)
            .and_then(|rows| rows.checked_add(5))
            .and_then(|value| value.checked_add(u64::from(self.nn)))
            .and_then(|value| value.checked_add(u64::from(self.nv)))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_flex_metadata"))
    }

    /// 参数元素数：顶点、参考顶点与节点坐标。
    pub(super) fn parameters_required(self) -> Result<u64, LauncherError> {
        u64::from(self.nv)
            .checked_mul(6)
            .and_then(|value| {
                u64::from(self.nn)
                    .checked_mul(3)
                    .and_then(|nodes| value.checked_add(nodes))
            })
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_flex_parameters"))
    }

    /// 失效标志每世界元素数。
    pub(super) fn validity_stride(self) -> u64 {
        u64::from(self.nf)
    }

    /// 节点与顶点位置每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nv)
            .checked_mul(3)
            .and_then(|value| {
                u64::from(self.nn)
                    .checked_mul(3)
                    .and_then(|nodes| value.checked_add(nodes))
            })
            .ok_or(LauncherError::Overflow("resident_flex_output"))
    }
}

/// 柔体失效与位置发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlexDims {
    pub(super) static_dims: FlexStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl FlexDims {
    /// 冻结ABI：三个保留零与世界数。
    pub(super) fn abi(self) -> [u32; 4] {
        [0, 0, 0, self.worlds]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻柔体世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_flex_guarded"))
}

/// 失效：刚体输入与i32标志输出。
#[cfg(feature = "cuda-probe")]
pub(super) fn flex_invalidate_lengths(dims: &FlexDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        static_dims.parameters_required()?.max(1),
        guarded(dims.worlds, u64::from(static_dims.rigid_stride))?,
        guarded(dims.worlds, static_dims.validity_stride())?,
    ])
}

/// 位置：刚体输入与节点/顶点输出。
#[cfg(feature = "cuda-probe")]
pub(super) fn flex_positions_lengths(dims: &FlexDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        static_dims.parameters_required()?.max(1),
        guarded(dims.worlds, u64::from(static_dims.rigid_stride))?,
        guarded(dims.worlds, static_dims.output_stride()?)?,
    ])
}

/// 校验元数据：五项头、每柔体行区间与插值网格、节点体与顶点体。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_flex_metadata(
    values: &[i32],
    dims: FlexStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("常驻柔体元数据长度无效"));
    }
    let header = [
        dims.nf as i32,
        dims.nn as i32,
        dims.nv as i32,
        dims.nb as i32,
        dims.rigid_stride as i32,
    ];
    if values[..5] != header {
        return Err(LauncherError::InvalidMetadata("常驻柔体头计数不符"));
    }
    let nf = usize::try_from(dims.nf).map_err(|_| LauncherError::Overflow("resident_flex_nf"))?;
    let nn = usize::try_from(dims.nn).map_err(|_| LauncherError::Overflow("resident_flex_nn"))?;
    let nv = u64::from(dims.nv);
    let nodebody = &values[5 + 9 * nf..5 + 9 * nf + nn];
    let vertbody = &values[5 + 9 * nf + nn..];
    for flex in 0..nf {
        let row = &values[5 + 9 * flex..14 + 9 * flex];
        if !(0..=3).contains(&row[0]) {
            return Err(LauncherError::InvalidMetadata("常驻柔体插值类型越界"));
        }
        if row[8] != 0 && row[8] != 1 {
            return Err(LauncherError::InvalidMetadata("常驻柔体居中标志无效"));
        }
        let nodeadr = usize::try_from(row[4])
            .map_err(|_| LauncherError::InvalidMetadata("常驻柔体节点地址为负"))?;
        let nodenum = usize::try_from(row[5])
            .map_err(|_| LauncherError::InvalidMetadata("常驻柔体节点数为负"))?;
        let vertadr = usize::try_from(row[6])
            .map_err(|_| LauncherError::InvalidMetadata("常驻柔体顶点地址为负"))?;
        let vertnum = usize::try_from(row[7])
            .map_err(|_| LauncherError::InvalidMetadata("常驻柔体顶点数为负"))?;
        let node_end = nodeadr
            .checked_add(nodenum)
            .filter(|&end| end <= nn)
            .ok_or(LauncherError::InvalidMetadata("常驻柔体节点区间越界"))?;
        let vert_end = vertadr
            .checked_add(vertnum)
            .filter(|&end| (end as u64) <= nv)
            .ok_or(LauncherError::InvalidMetadata("常驻柔体顶点区间越界"))?;
        // 节点位置对所有柔体类型都读取，节点体必须有效。
        if nodebody[nodeadr..node_end]
            .iter()
            .any(|&body| body < 0 || body as u32 >= dims.nb)
        {
            return Err(LauncherError::InvalidMetadata("常驻柔体节点体越界"));
        }
        if row[0] == 0 {
            // 顶点类型直接读取顶点体；其余类型的顶点由插值生成，顶点体为占位负值。
            if vertbody[vertadr..vert_end]
                .iter()
                .any(|&body| body < 0 || body as u32 >= dims.nb)
            {
                return Err(LauncherError::InvalidMetadata("常驻柔体顶点体越界"));
            }
            continue;
        }
        // 插值类型按cell三维网格读取节点：节点地址必须覆盖(r1+1)*(r2+1)*(r3+1)个节点。
        let cells = [row[1], row[2], row[3]];
        if cells.iter().any(|&width| width < 1) {
            return Err(LauncherError::InvalidMetadata("常驻柔体cell维度无效"));
        }
        let needed = cells
            .into_iter()
            .try_fold(1usize, |product, width| {
                usize::try_from(width)
                    .ok()
                    .and_then(|value| value.checked_add(1))
                    .and_then(|value| product.checked_mul(value))
            })
            .ok_or(LauncherError::Overflow("resident_flex_grid"))?;
        if nodeadr.checked_add(needed).is_none_or(|end| end > nn) {
            return Err(LauncherError::InvalidMetadata("常驻柔体插值网格越界"));
        }
    }
    Ok(())
}
