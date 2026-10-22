//! 流体被动力四入口的槽位、证明与精确跨度。

#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, Checked, ElemType, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotDesc,
    SlotKind, SlotRole,
};

/// 元数据证明名称。
#[cfg(feature = "cuda-probe")]
pub(super) const PROOF: ProofKind = ProofKind("physics.fluid_force.metadata.v1");

#[cfg(feature = "cuda-probe")]
const fn read_slot(index: usize) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
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

/// 冻结ABI：元数据、五个输入与输出，后接四个标量。
#[cfg(feature = "cuda-probe")]
pub(super) const SLOTS: [SlotDesc; 11] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing { proof: PROOF },
    },
    read_slot(1),
    read_slot(2),
    read_slot(3),
    read_slot(4),
    read_slot(5),
    SlotDesc {
        index: 6,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    scalar_slot(7),
    scalar_slot(8),
    scalar_slot(9),
    scalar_slot(10),
];

/// 入口种类；决定输入槽与输出槽的精确宽度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FluidForceEntry {
    Validate,
    Body,
    Project,
    Clear,
}

/// 与世界数无关的静态维度。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FluidForceStaticDims {
    pub(super) nb: u32,
    pub(super) nj: u32,
    pub(super) nv: u32,
    pub(super) ng: u32,
    pub(super) ns: u32,
    pub(super) parameter_len: u32,
    /// 是否存在椭球流体体；决定第五槽是附着位姿还是参数占位。
    pub(super) attached: bool,
}

#[cfg(feature = "cuda-probe")]
impl FluidForceStaticDims {
    /// 由刚体、关节、自由度、几何、site 与参数计数构造。
    pub(super) fn new(
        nb: usize,
        nj: usize,
        nv: usize,
        ng: usize,
        ns: usize,
        parameter_len: usize,
        attached: bool,
    ) -> Result<Self, LauncherError> {
        let value = |field: &'static str, raw: usize| {
            u32::try_from(raw).map_err(|_| LauncherError::Overflow(field))
        };
        let dims = Self {
            nb: value("fluid_force_nb", nb)?,
            nj: value("fluid_force_nj", nj)?,
            nv: value("fluid_force_nv", nv)?,
            ng: value("fluid_force_ng", ng)?,
            ns: value("fluid_force_ns", ns)?,
            parameter_len: value("fluid_force_parameters", parameter_len)?,
            attached,
        };
        if dims.nb < 2 || dims.nv == 0 {
            return Err(LauncherError::InvalidArray("流体被动力维度无效"));
        }
        dims.metadata_elements()?;
        dims.parameters_elements()?;
        Ok(dims)
    }

    /// 元数据元素数：首尾守卫、父表、根表、自由度体、几何体、几何类型与椭球标记。
    pub(super) fn metadata_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.nb)
            .checked_mul(3)
            .and_then(|value| value.checked_add(u64::from(self.nv)))
            .and_then(|value| value.checked_add(2 * u64::from(self.ng)))
            .and_then(|value| value.checked_add(8))
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("fluid_force_metadata"))
    }

    /// 参数元素数：首尾守卫与冻结批量。
    pub(super) fn parameters_elements(self) -> Result<u64, LauncherError> {
        u64::from(self.parameter_len)
            .checked_add(8)
            .filter(|&value| value <= i32::MAX as u64)
            .ok_or(LauncherError::Overflow("fluid_force_parameters"))
    }

    /// 刚体输入每世界元素数。
    pub(super) fn rigid_stride(self) -> u64 {
        u64::from(self.nb) * 28 + u64::from(self.nj) * 6
    }

    /// 质心输入每世界元素数。
    pub(super) fn com_stride(self) -> u64 {
        u64::from(self.nb) * 14 + u64::from(self.nv) * 6
    }

    /// 空间速度每世界元素数。
    pub(super) fn velocity_stride(self) -> u64 {
        u64::from(self.nb) * 6 + u64::from(self.nv) * 6
    }

    /// 附着位姿每世界元素数。
    pub(super) fn attached_stride(self) -> Result<u64, LauncherError> {
        self.ng
            .checked_add(self.ns)
            .and_then(|count| count.checked_mul(12))
            .map(u64::from)
            .ok_or(LauncherError::Overflow("fluid_force_attached"))
    }

    /// 刚体空间力每世界元素数。
    pub(super) fn body_stride(self) -> u64 {
        u64::from(self.nb) * 6
    }
}

/// 流体被动力发射维度；宽度随入口变化。
#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FluidForceDims {
    pub(super) static_dims: FluidForceStaticDims,
    pub(super) worlds: u32,
    pub(super) abi: [u32; 4],
    pub(super) entry: FluidForceEntry,
    /// 清零入口的工作量；其它入口为零。
    pub(super) clear_work: u32,
}

#[cfg(feature = "cuda-probe")]
fn world_count(worlds: u32) -> Result<u64, LauncherError> {
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("流体被动力世界数无效"));
    }
    Ok(u64::from(worlds))
}

#[cfg(feature = "cuda-probe")]
fn guarded(worlds: u32, width: u64) -> Result<u64, LauncherError> {
    world_count(worlds)?
        .checked_mul(width)
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("fluid_force_guarded"))
}

/// 七槽精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn fluid_force_lengths(dims: &FluidForceDims) -> Result<[u64; 7], LauncherError> {
    let static_dims = dims.static_dims;
    let metadata = static_dims.metadata_elements()?;
    let parameters = static_dims.parameters_elements()?;
    let rigid = guarded(dims.worlds, static_dims.rigid_stride())?;
    let com = guarded(dims.worlds, static_dims.com_stride())?;
    let velocity = guarded(dims.worlds, static_dims.velocity_stride())?;
    let body = guarded(dims.worlds, static_dims.body_stride())?;
    let attached = if static_dims.attached {
        guarded(dims.worlds, static_dims.attached_stride()?)?
    } else {
        // 无椭球体时第五槽退回参数占位，内核不读取。
        parameters
    };
    let status = guarded(dims.worlds, 1)?;
    let qfrc = guarded(dims.worlds, u64::from(static_dims.nv))?;
    Ok(match dims.entry {
        FluidForceEntry::Validate => [metadata, parameters, rigid, com, velocity, attached, status],
        FluidForceEntry::Body => [metadata, parameters, rigid, com, velocity, attached, body],
        FluidForceEntry::Project => [metadata, parameters, rigid, com, body, attached, qfrc],
        FluidForceEntry::Clear => [
            metadata,
            parameters,
            parameters,
            parameters,
            parameters,
            parameters,
            u64::from(dims.clear_work)
                .checked_add(8)
                .ok_or(LauncherError::Overflow("fluid_force_clear"))?,
        ],
    })
}

/// 校验元数据：守卫、父表、根表、自由度体、几何体与椭球标记。
#[cfg(feature = "cuda-probe")]
pub(super) fn validate_fluid_force_metadata(
    values: &[i32],
    dims: FluidForceStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements()?) {
        return Err(LauncherError::InvalidMetadata("流体被动力元数据长度无效"));
    }
    let nb = usize::try_from(dims.nb).map_err(|_| LauncherError::Overflow("fluid_force_nb"))?;
    let nv = usize::try_from(dims.nv).map_err(|_| LauncherError::Overflow("fluid_force_nv"))?;
    let ng = usize::try_from(dims.ng).map_err(|_| LauncherError::Overflow("fluid_force_ng"))?;
    let tail = values.len() - 4;
    if values[..4] != [-131072; 4] || values[tail..] != [-131072; 4] {
        return Err(LauncherError::InvalidMetadata("流体被动力元数据守卫无效"));
    }
    let parent = &values[4..4 + nb];
    let root = &values[4 + nb..4 + 2 * nb];
    let dof_body = &values[4 + 2 * nb..4 + 2 * nb + nv];
    let geom_body = &values[4 + 2 * nb + nv..4 + 2 * nb + nv + ng];
    let geom_type = &values[4 + 2 * nb + nv + ng..4 + 2 * nb + nv + 2 * ng];
    let ellipsoid = &values[4 + 2 * nb + nv + 2 * ng..tail];
    for body in 0..nb {
        let parent_ok = if body == 0 {
            parent[0] == 0
        } else {
            parent[body] >= 0 && (parent[body] as usize) < body
        };
        if !parent_ok || root[body] < 0 || root[body] as usize >= nb {
            return Err(LauncherError::InvalidMetadata("流体被动力体父链或根表无效"));
        }
        let expected = if body == 0 {
            0
        } else if parent[body] == 0 {
            body
        } else {
            let ancestor = root[parent[body] as usize];
            if ancestor < 0 {
                return Err(LauncherError::InvalidMetadata("流体被动力根表无效"));
            }
            ancestor as usize
        };
        if root[body] as usize != expected || (ellipsoid[body] != 0 && ellipsoid[body] != 1) {
            return Err(LauncherError::InvalidMetadata(
                "流体被动力根表或椭球标记无效",
            ));
        }
    }
    if dof_body.iter().any(|&body| body < 0 || body as usize >= nb) {
        return Err(LauncherError::InvalidMetadata("流体被动力自由度体越界"));
    }
    if geom_body
        .iter()
        .any(|&body| body < 0 || body as usize >= nb)
    {
        return Err(LauncherError::InvalidMetadata("流体被动力几何体越界"));
    }
    // 几何类型由内核分支判定；这里只拒绝确实无法映射的值。
    if geom_type.iter().any(|&kind| kind < 0) {
        return Err(LauncherError::InvalidMetadata("流体被动力几何类型为负"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FluidForceMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for FluidForceMetadata {
    type Dims = FluidForceDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_fluid_force_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn fluid_force_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "fluid_force_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "fluid_force_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "fluid_force_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "fluid_force_abi",
        }
        .into(),
    }
}

/// 计划级元数据缓冲；构造时上传一次，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct FluidForceMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: FluidForceStaticDims,
    revision: MetadataRevision,
    checked: Checked<FluidForceMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl FluidForceMetadataBuffer {
    /// 由计划构造期冻结的宿主镜像上传并建立证明。
    pub(super) fn from_values(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: FluidForceStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = Self::base_dims(static_dims);
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<FluidForceMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(fluid_force_launch_error)?;
        Ok(Self {
            buffer,
            values,
            static_dims,
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn static_dims(&self) -> FluidForceStaticDims {
        self.static_dims
    }

    /// 只改设备元数据、保留有效证明，供设备验证器测试读取坏值。
    #[cfg(test)]
    pub(super) fn corrupt_device_for_validator_test(
        &mut self,
        start: usize,
        values: &[i32],
    ) -> Result<(), TransferError> {
        self.buffer.write_range(start, values)
    }

    fn base_dims(static_dims: FluidForceStaticDims) -> FluidForceDims {
        FluidForceDims {
            static_dims,
            worlds: 1,
            abi: [0; 4],
            entry: FluidForceEntry::Clear,
            clear_work: 0,
        }
    }

    /// 发射证明：先核对证明未失效，再按实际维度重建。
    pub(super) fn proof(
        &self,
        dims: FluidForceDims,
    ) -> Result<Checked<FluidForceMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &Self::base_dims(self.static_dims))
            .map_err(fluid_force_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际维度只影响跨度。
        unsafe {
            Checked::<FluidForceMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(fluid_force_launch_error)
    }
}
