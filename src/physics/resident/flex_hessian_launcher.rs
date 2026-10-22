//! 常驻柔体 Hessian FIELD_BATCH 变体的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称；缓存与发布两段共用同一份柔体Hessian元数据。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_flex_hessian.metadata.v1");

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

/// flex_hessian：元数据、系数、位置、边长、失效标志与Hessian输出，后接一个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const HESSIAN_SLOTS: [SlotDesc; 7] = [
    meta_slot(),
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::F32),
    read_slot(3, ElemType::F32),
    read_slot(4, ElemType::I32),
    SlotDesc {
        index: 5,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    scalar_slot(6),
];

/// flex_hessian_validate：元数据、系数、Hessian读入与i32失效标志，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const VALIDATE_SLOTS: [SlotDesc; 8] = [
    meta_slot(),
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::F32),
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
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
pub(super) struct FlexHessianStaticDims {
    pub(super) nf: u32,
    pub(super) nv: u32,
    pub(super) nn: u32,
    pub(super) ne: u32,
    pub(super) pos_stride: u32,
    pub(super) edge_stride: u32,
    pub(super) parameters_len: u32,
    pub(super) metadata_elements: u64,
}

/// 构造静态维度所需的宿主计数。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug)]
pub(super) struct FlexHessianDimsInput {
    pub(super) nf: usize,
    pub(super) nv: usize,
    pub(super) nn: usize,
    pub(super) ne: usize,
    pub(super) pos_stride: usize,
    pub(super) edge_stride: usize,
    pub(super) parameters_len: usize,
    pub(super) metadata_elements: usize,
}

#[cfg(feature = "cuda-probe")]
impl FlexHessianStaticDims {
    /// 由柔体、节点、边与两处布局跨度构造。
    pub(super) fn new(input: FlexHessianDimsInput) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nf: value("resident_flex_hessian_nf", input.nf)?,
            nv: value("resident_flex_hessian_nv", input.nv)?,
            nn: value("resident_flex_hessian_nn", input.nn)?,
            ne: value("resident_flex_hessian_ne", input.ne)?,
            pos_stride: value("resident_flex_hessian_pos", input.pos_stride)?,
            edge_stride: value("resident_flex_hessian_edge", input.edge_stride)?,
            parameters_len: value("resident_flex_hessian_parameters", input.parameters_len)?,
            metadata_elements: u64::try_from(input.metadata_elements)
                .map_err(|_| LauncherError::Overflow("resident_flex_hessian_metadata"))?,
        };
        let expected_pos = u64::from(dims.nn)
            .checked_mul(3)
            .and_then(|nodes| {
                u64::from(dims.nv)
                    .checked_mul(3)
                    .and_then(|verts| nodes.checked_add(verts))
            })
            .ok_or(LauncherError::Overflow("resident_flex_hessian_pos"))?;
        if dims.nf == 0
            || dims.nv == 0
            || dims.ne == 0
            || expected_pos != u64::from(dims.pos_stride)
        {
            return Err(LauncherError::InvalidArray("常驻柔体Hessian维度无效"));
        }
        if u64::from(dims.edge_stride) < 2 * u64::from(dims.ne) {
            return Err(LauncherError::InvalidArray("常驻柔体Hessian边长跨度不足"));
        }
        Ok(dims)
    }

    /// Hessian输出每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nv)
            .checked_mul(6)
            .and_then(|diag| {
                u64::from(self.ne)
                    .checked_mul(9)
                    .and_then(|blocks| diag.checked_add(blocks))
            })
            .ok_or(LauncherError::Overflow("resident_flex_hessian_output"))
    }

    /// 失效标志每世界元素数。
    pub(super) fn validity_stride(self) -> u64 {
        u64::from(self.nf)
    }
}

/// 柔体Hessian发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlexHessianDims {
    pub(super) static_dims: FlexHessianStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl FlexHessianDims {
    /// 冻结ABI：三个保留零与世界数。
    pub(super) fn abi(self) -> [u32; 4] {
        [0, 0, 0, self.worlds]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻柔体Hessian世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_flex_hessian_guarded"))
}

/// 六槽精确元素数：元数据、系数、位置、边长、失效标志与输出。
#[cfg(feature = "cuda-probe")]
pub(super) fn flex_hessian_lengths(dims: &FlexHessianDims) -> Result<[u64; 6], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements,
        u64::from(static_dims.parameters_len).max(1),
        guarded(dims.worlds, u64::from(static_dims.pos_stride))?,
        guarded(dims.worlds, u64::from(static_dims.edge_stride))?,
        guarded(dims.worlds, static_dims.validity_stride())?,
        guarded(dims.worlds, static_dims.output_stride()?)?,
    ])
}

/// 四槽精确元素数：元数据、系数、Hessian与失效标志。
#[cfg(feature = "cuda-probe")]
pub(super) fn flex_hessian_validate_lengths(
    dims: &FlexHessianDims,
) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements,
        u64::from(static_dims.parameters_len).max(1),
        guarded(dims.worlds, static_dims.output_stride()?)?,
        guarded(dims.worlds, static_dims.validity_stride())?,
    ])
}

/// 校验元数据：头、偏移表、每柔体区间、元素/边索引与21系数区间。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_flex_hessian_metadata(
    values: &[i32],
    dims: FlexHessianStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements) {
        return Err(LauncherError::InvalidMetadata(
            "常驻柔体Hessian元数据长度无效",
        ));
    }
    let header = [
        dims.nf as i32,
        dims.nv as i32,
        dims.ne as i32,
        dims.pos_stride as i32,
        dims.edge_stride as i32,
    ];
    if values[..5] != header {
        return Err(LauncherError::InvalidMetadata("常驻柔体Hessian头计数不符"));
    }
    let nf =
        usize::try_from(dims.nf).map_err(|_| LauncherError::Overflow("resident_hessian_nf"))?;
    let ne = u64::from(dims.ne);
    let nv = u64::from(dims.nv);
    let elem_offset = usize::try_from(values[5])
        .map_err(|_| LauncherError::InvalidMetadata("常驻柔体Hessian元素偏移为负"))?;
    let elemedge_offset = usize::try_from(values[6])
        .map_err(|_| LauncherError::InvalidMetadata("常驻柔体Hessian元素边偏移为负"))?;
    let edge_offset = usize::try_from(values[7])
        .map_err(|_| LauncherError::InvalidMetadata("常驻柔体Hessian边偏移为负"))?;
    if elem_offset != 9 + 11 * nf {
        return Err(LauncherError::InvalidMetadata(
            "常驻柔体Hessian元素偏移不符",
        ));
    }
    if values[8] != 3 * dims.nn as i32 {
        return Err(LauncherError::InvalidMetadata(
            "常驻柔体Hessian节点偏移不符",
        ));
    }
    if elemedge_offset < elem_offset || edge_offset < elemedge_offset {
        return Err(LauncherError::InvalidMetadata("常驻柔体Hessian偏移表逆序"));
    }
    let elem_len = elemedge_offset - elem_offset;
    let elemedge_len = edge_offset - elemedge_offset;
    if u64::try_from(edge_offset)
        .ok()
        .and_then(|offset| offset.checked_add(2 * ne))
        != Some(dims.metadata_elements)
    {
        return Err(LauncherError::InvalidMetadata(
            "常驻柔体Hessian边数据长度不符",
        ));
    }
    for flex in 0..nf {
        let row = &values[9 + 11 * flex..20 + 11 * flex];
        if row[1] != 0 && row[1] != 1 {
            return Err(LauncherError::InvalidMetadata(
                "常驻柔体Hessian激活标志无效",
            ));
        }
        let position = |index: usize| {
            usize::try_from(row[index])
                .map_err(|_| LauncherError::InvalidMetadata("常驻柔体Hessian区间为负"))
        };
        let vertadr = position(2)?;
        let vertnum = position(3)?;
        let edgeadr = position(4)?;
        let edgenum = position(5)?;
        if vertadr
            .checked_add(vertnum)
            .is_none_or(|end| end as u64 > nv)
        {
            return Err(LauncherError::InvalidMetadata(
                "常驻柔体Hessian顶点区间越界",
            ));
        }
        if edgeadr
            .checked_add(edgenum)
            .is_none_or(|end| end as u64 > ne)
        {
            return Err(LauncherError::InvalidMetadata("常驻柔体Hessian边区间越界"));
        }
        // 未激活柔体只清零对角与块，不读取元素/系数；其维度与地址可为占位值。
        if row[1] == 0 {
            continue;
        }
        let dim = row[0];
        if dim != 2 && dim != 3 {
            return Err(LauncherError::InvalidMetadata(
                "常驻柔体Hessian单元维度无效",
            ));
        }
        let width = dim as usize + 1;
        let pairs = if dim == 2 { 3 } else { 6 };
        let elemnum = position(7)?;
        let elemdataadr = position(8)?;
        let elemedgeadr = position(9)?;
        let stiffnessadr = position(10)?;
        elemdataadr
            .checked_add(
                elemnum
                    .checked_mul(width)
                    .ok_or(LauncherError::Overflow("resident_hessian_elements"))?,
            )
            .filter(|&end| end <= elem_len)
            .ok_or(LauncherError::InvalidMetadata(
                "常驻柔体Hessian元素区间越界",
            ))?;
        elemedgeadr
            .checked_add(
                elemnum
                    .checked_mul(pairs)
                    .ok_or(LauncherError::Overflow("resident_hessian_elemedge"))?,
            )
            .filter(|&end| end <= elemedge_len)
            .ok_or(LauncherError::InvalidMetadata(
                "常驻柔体Hessian元素边区间越界",
            ))?;
        let coefficients = elemnum
            .checked_mul(21)
            .and_then(|value| stiffnessadr.checked_add(value))
            .ok_or(LauncherError::Overflow("resident_hessian_stiffness"))?;
        if u64::try_from(coefficients)
            .ok()
            .and_then(|value| value.checked_add(ne))
            .is_none_or(|end| end > u64::from(dims.parameters_len))
        {
            return Err(LauncherError::InvalidMetadata(
                "常驻柔体Hessian系数区间越界",
            ));
        }
        for element in 0..elemnum {
            for local in 0..width {
                let index = elem_offset + elemdataadr + element * width + local;
                let vertex = values[index];
                if vertex < 0 || vertex as usize >= vertnum {
                    return Err(LauncherError::InvalidMetadata(
                        "常驻柔体Hessian单元顶点越界",
                    ));
                }
            }
            for pair in 0..pairs {
                let index = elemedge_offset + elemedgeadr + element * pairs + pair;
                let edge = values[index];
                if edge < 0 || edge as usize >= edgenum {
                    return Err(LauncherError::InvalidMetadata("常驻柔体Hessian单元边越界"));
                }
            }
        }
        for edge in edgeadr..edgeadr + edgenum {
            let endpoints = edge_offset + 2 * edge;
            for endpoint in &values[endpoints..endpoints + 2] {
                if *endpoint < 0 || *endpoint as usize >= vertnum {
                    return Err(LauncherError::InvalidMetadata("常驻柔体Hessian端点越界"));
                }
            }
        }
    }
    Ok(())
}
