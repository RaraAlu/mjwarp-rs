//! 常驻空间肌腱三项 FIELD_BATCH 变体的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;

/// 元数据证明名称；三项共用同一模型元数据。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.resident_spatial_tendon.metadata.v1");

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

/// site与moment：元数据、缩放、输入与f32结果，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const SLOTS_F32: [SlotDesc; 8] = [
    meta_slot(),
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

/// wrap：读取f32结果，写i32包裹索引，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const SLOTS_I32: [SlotDesc; 8] = [
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
pub(super) struct SpatialTendonStaticDims {
    pub(super) nb: u32,
    pub(super) nv: u32,
    pub(super) nt: u32,
    pub(super) nnz: u32,
    pub(super) nw: u32,
    pub(super) ngeom: u32,
    pub(super) nsite: u32,
    pub(super) attached_stride: u32,
    pub(super) parameters_len: u32,
}

/// 构造静态维度所需的宿主计数。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug)]
pub(super) struct SpatialTendonDimsInput {
    pub(super) nb: usize,
    pub(super) nv: usize,
    pub(super) nt: usize,
    pub(super) nnz: usize,
    pub(super) nw: usize,
    pub(super) ngeom: usize,
    pub(super) nsite: usize,
    pub(super) attached_stride: usize,
    pub(super) parameters_len: usize,
}

#[cfg(feature = "cuda-probe")]
impl SpatialTendonStaticDims {
    pub(super) fn new(input: SpatialTendonDimsInput) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nb: value("resident_spatial_tendon_nb", input.nb)?,
            nv: value("resident_spatial_tendon_nv", input.nv)?,
            nt: value("resident_spatial_tendon_nt", input.nt)?,
            nnz: value("resident_spatial_tendon_nnz", input.nnz)?,
            nw: value("resident_spatial_tendon_nw", input.nw)?,
            ngeom: value("resident_spatial_tendon_ngeom", input.ngeom)?,
            nsite: value("resident_spatial_tendon_nsite", input.nsite)?,
            attached_stride: value("resident_spatial_tendon_attached", input.attached_stride)?,
            parameters_len: value("resident_spatial_tendon_parameters", input.parameters_len)?,
        };
        if dims.nb == 0 {
            return Err(LauncherError::InvalidArray("常驻空间肌腱维度无效"));
        }
        let attached = u64::from(dims.ngeom)
            .checked_add(u64::from(dims.nsite))
            .and_then(|value| value.checked_mul(12))
            .ok_or(LauncherError::Overflow("resident_spatial_tendon_attached"))?;
        if attached != u64::from(dims.attached_stride) {
            return Err(LauncherError::InvalidArray("常驻空间肌腱附着跨度不符"));
        }
        dims.metadata_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：九项头、四块体表、四块肌腱表、四块包裹表与列索引。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        let blocks = [
            u64::from(self.nb).checked_mul(4),
            u64::from(self.nt).checked_mul(4),
            u64::from(self.nw).checked_mul(4),
        ];
        let mut total = 9u64;
        for block in blocks {
            total = total
                .checked_add(
                    block.ok_or(LauncherError::Overflow("resident_spatial_tendon_metadata"))?,
                )
                .ok_or(LauncherError::Overflow("resident_spatial_tendon_metadata"))?;
        }
        total
            .checked_add(u64::from(self.nnz))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("resident_spatial_tendon_metadata"))
    }

    /// 空间结果每世界元素数。
    pub(super) fn output_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nt)
            .checked_add(u64::from(self.nnz))
            .and_then(|value| {
                u64::from(self.nw)
                    .checked_mul(19)
                    .and_then(|wraps| value.checked_add(wraps))
            })
            .ok_or(LauncherError::Overflow("resident_spatial_tendon_output"))
    }

    /// 包裹索引每世界元素数。
    pub(super) fn indices_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nt)
            .checked_add(u64::from(self.nw))
            .and_then(|value| value.checked_mul(2))
            .ok_or(LauncherError::Overflow("resident_spatial_tendon_indices"))
    }

    /// 质心输入每世界元素数。
    pub(super) fn com_stride(self) -> Result<u64, LauncherError> {
        u64::from(self.nb)
            .checked_mul(14)
            .and_then(|value| {
                u64::from(self.nv)
                    .checked_mul(6)
                    .and_then(|dofs| value.checked_add(dofs))
            })
            .ok_or(LauncherError::Overflow("resident_spatial_tendon_com"))
    }
}

/// 空间肌腱发射维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SpatialTendonDims {
    pub(super) static_dims: SpatialTendonStaticDims,
    pub(super) worlds: u32,
}

#[cfg(feature = "cuda-probe")]
impl SpatialTendonDims {
    /// 冻结ABI：自由度、两个保留零与世界数。
    pub(super) fn abi(self) -> [u32; 4] {
        [self.static_dims.nv, 0, 0, self.worlds]
    }
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻空间肌腱世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_spatial_tendon_guarded"))
}

/// site：附着输入与空间结果。
#[cfg(feature = "cuda-probe")]
pub(super) fn spatial_site_lengths(dims: &SpatialTendonDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        u64::from(static_dims.parameters_len).max(1),
        guarded(dims.worlds, u64::from(static_dims.attached_stride))?,
        guarded(dims.worlds, static_dims.output_stride()?)?,
    ])
}

/// moment：质心输入与空间结果。
#[cfg(feature = "cuda-probe")]
pub(super) fn spatial_moment_lengths(dims: &SpatialTendonDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        u64::from(static_dims.parameters_len).max(1),
        guarded(dims.worlds, static_dims.com_stride()?)?,
        guarded(dims.worlds, static_dims.output_stride()?)?,
    ])
}

/// wrap：空间结果输入与包裹索引输出。
#[cfg(feature = "cuda-probe")]
pub(super) fn spatial_wrap_lengths(dims: &SpatialTendonDims) -> Result<[u64; 4], LauncherError> {
    let static_dims = dims.static_dims;
    Ok([
        static_dims.metadata_elements()?,
        u64::from(static_dims.parameters_len).max(1),
        guarded(dims.worlds, static_dims.output_stride()?)?,
        guarded(dims.worlds, static_dims.indices_stride()?)?,
    ])
}

/// 校验元数据：头一致性、体父链、自由度表、肌腱行与包裹表。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_spatial_tendon_metadata(
    values: &[i32],
    dims: SpatialTendonStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱元数据长度无效"));
    }
    let nb = usize::try_from(dims.nb)
        .map_err(|_| LauncherError::Overflow("resident_spatial_tendon_nb"))?;
    let nt = usize::try_from(dims.nt)
        .map_err(|_| LauncherError::Overflow("resident_spatial_tendon_nt"))?;
    let nw = usize::try_from(dims.nw)
        .map_err(|_| LauncherError::Overflow("resident_spatial_tendon_nw"))?;
    if values[..4]
        != [
            dims.nt as i32,
            dims.nw as i32,
            dims.nnz as i32,
            dims.nb as i32,
        ]
    {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱头计数不符"));
    }
    if values[4] != dims.attached_stride as i32
        || values[5] != 12 * dims.ngeom as i32
        || values[6] != dims.ngeom as i32
    {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱附着跨度不符"));
    }
    if values[7] < 1 || values[8] != dims.nw as i32 {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱参数表头无效"));
    }
    let batches = u64::try_from(values[7])
        .map_err(|_| LauncherError::InvalidMetadata("常驻空间肌腱几何批次为负"))?;
    let mut cursor = 9;
    let parents = &values[cursor..cursor + nb];
    cursor += nb;
    if parents[0] != -1 && parents[0] != 0 {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱根体父引用无效"));
    }
    for (body, &parent) in parents.iter().enumerate().skip(1) {
        if parent < 0 || parent as usize >= body {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱体父链无效"));
        }
    }
    let roots = &values[cursor..cursor + nb];
    cursor += nb;
    if roots[0] != -1 && roots[0] != 0 {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱根体索引无效"));
    }
    if roots
        .iter()
        .enumerate()
        .skip(1)
        .any(|(body, &root)| root < 0 || root as usize > body || root as usize >= nb)
    {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱根体表越界"));
    }
    let dofadr = &values[cursor..cursor + nb];
    cursor += nb;
    let dofnum = &values[cursor..cursor + nb];
    cursor += nb;
    let nv = u64::from(dims.nv);
    for (&address, &count) in dofadr.iter().zip(dofnum) {
        if address < 0 || count < 0 {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱自由度表为负"));
        }
        let end = u64::try_from(address)
            .ok()
            .and_then(|value| u64::try_from(count).ok()?.checked_add(value))
            .ok_or(LauncherError::Overflow("resident_spatial_tendon_dofs"))?;
        if end > nv {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱自由度表越界"));
        }
    }
    let adr = &values[cursor..cursor + nt];
    cursor += nt;
    let num = &values[cursor..cursor + nt];
    cursor += nt;
    let rowadr = &values[cursor..cursor + nt];
    cursor += nt;
    let rownnz = &values[cursor..cursor + nt];
    cursor += nt;
    let mut row = 0usize;
    for tendon in 0..nt {
        if adr[tendon] < 0 || num[tendon] < 0 {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱行区间为负"));
        }
        if adr[tendon] as usize != row {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱行未连续覆盖"));
        }
        row = row
            .checked_add(num[tendon] as usize)
            .ok_or(LauncherError::Overflow("resident_spatial_tendon_rows"))?;
    }
    if row != nw {
        return Err(LauncherError::InvalidMetadata(
            "常驻空间肌腱行数与包裹数不符",
        ));
    }
    for (&address, &count) in rowadr.iter().zip(rownnz) {
        if address < 0 || count < 0 {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱稀疏行为负"));
        }
        let end = u64::try_from(address)
            .ok()
            .and_then(|value| u64::try_from(count).ok()?.checked_add(value))
            .ok_or(LauncherError::Overflow("resident_spatial_tendon_sparse"))?;
        if end > u64::from(dims.nnz) {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱稀疏行越界"));
        }
    }
    let types = &values[cursor..cursor + nw];
    cursor += nw;
    let objects = &values[cursor..cursor + nw];
    cursor += nw;
    let bodies = &values[cursor..cursor + nw];
    cursor += nw;
    let sides = &values[cursor..cursor + nw];
    cursor += nw;
    let colind = &values[cursor..];
    // 几何尺寸参数只在存在球柱/滑轮包裹时被内核索引。
    let geometry = types.iter().any(|&kind| matches!(kind, 4..=5));
    let sizes = if geometry {
        batches
            .checked_mul(3)
            .and_then(|value| value.checked_mul(u64::from(dims.ngeom)))
            .ok_or(LauncherError::Overflow(
                "resident_spatial_tendon_parameters",
            ))?
    } else {
        0
    };
    let required = u64::from(dims.nw)
        .checked_add(sizes)
        .ok_or(LauncherError::Overflow(
            "resident_spatial_tendon_parameters",
        ))?;
    if required > u64::from(dims.parameters_len) {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱参数区间越界"));
    }
    for (wrap, &kind) in types.iter().enumerate() {
        if !(0..=5).contains(&kind) {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱包裹类型越界"));
        }
        let object = objects[wrap];
        if kind == 3 && (object < 0 || object as u32 >= dims.nsite) {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱site索引越界"));
        }
        if matches!(kind, 4..=5) && (object < 0 || object as u32 >= dims.ngeom) {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱几何索引越界"));
        }
        if !matches!(kind, 3..=5) && object != -1 {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱占位索引非空"));
        }
        // 非包裹对象（类型2等）用负值哨兵，内核只在包裹分支读取体索引。
        if matches!(kind, 3..=5) {
            if bodies[wrap] < 0 || bodies[wrap] as u32 >= dims.nb {
                return Err(LauncherError::InvalidMetadata("常驻空间肌腱体索引越界"));
            }
        } else if bodies[wrap] != -1 {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱占位体索引非空"));
        }
        let side = sides[wrap];
        if side < -1 || (side >= 0 && side as u32 >= dims.nsite) {
            return Err(LauncherError::InvalidMetadata("常驻空间肌腱侧点索引越界"));
        }
    }
    if colind
        .iter()
        .any(|&column| column < 0 || column as u32 >= dims.nv)
    {
        return Err(LauncherError::InvalidMetadata("常驻空间肌腱列索引越界"));
    }
    Ok(())
}
