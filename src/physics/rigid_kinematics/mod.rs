//! 刚体运动学入口的类型化安全包装。
//! 三处调用共用冻结ABI与元数据证明。
//! 内核源码仍由调用方编译，PTX与浮点顺序不变。
//! 设计见`docs/unified-launcher.md`。

#[cfg(feature = "cuda-probe")]
use self::metadata::{
    PROOF, RigidDims, RigidMetadata, RigidStaticDims, StaticRigidMetadata, rigid_lengths,
    validate_metadata,
};
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::{InputError, TransferError};
#[cfg(feature = "cuda-probe")]
use crate::runtime::TransferSession;
#[cfg(all(feature = "cuda-probe", test))]
use crate::runtime::launcher::DeviceSpan;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataRevision, SlotBinding, SlotDesc, SlotKind, SlotRequirement,
    SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

mod metadata;
#[cfg(test)]
mod tests;

/// 刚体运动学的三种发射变体。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RigidVariant {
    /// 普通探针：无批量字段，状态只含qpos。
    Probe,
    /// 严格常驻计划：批量字段与mocap映射。
    Batch,
    /// G01等价计划：布局同`Batch`，内核回读body0。
    Equivalent,
}

impl RigidVariant {
    pub(super) const fn batched(self) -> bool {
        !matches!(self, Self::Probe)
    }
}

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

#[cfg(feature = "cuda-probe")]
/// 冻结ABI：四个指针后接四个标量。
const SLOTS: [SlotDesc; 8] = [
    META_SLOT,
    read_slot(1, ElemType::F32),
    read_slot(2, ElemType::F32),
    // 内核回读已写入的父体位姿；G01等价变体读取body0。
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

#[cfg(feature = "cuda-probe")]
struct RigidSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结rigid_kinematics私有ABI。
unsafe impl KernelAbi for RigidSpec {
    const NAME: &'static str = "rigid_kinematics";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for RigidSpec {
    type Dims = RigidDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(rigid_lengths(dims)?
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        rigid_lengths(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
/// 在最小跨度之外要求精确跨度，再核对证明与别名。
fn rigid_plan(
    dims: &RigidDims,
    bindings: &[SlotBinding; 4],
    proof: CheckedBinding<'_, RigidDims>,
) -> Result<(), LauncherError> {
    let lengths = rigid_lengths(dims)?;
    Kernel::<RigidSpec>::new()?.plan(dims, bindings, &[proof])?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let required = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("rigid_kinematics_bytes"))?;
        if bindings[slot].span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: bindings[slot].span.bytes(),
            });
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
fn rigid_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "rigid_kinematics_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "rigid_kinematics_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "rigid_kinematics_abi",
        }
        .into(),
    }
}

/// 设备拓扑、宿主镜像、参数与写入代际。
#[cfg(feature = "cuda-probe")]
pub(super) struct RigidModel {
    session: TransferSession,
    static_dims: RigidStaticDims,
    pub(super) metadata: TransferBuffer<i32>,
    metadata_values: Vec<i32>,
    pub(super) parameters: TransferBuffer<f32>,
    revision: MetadataRevision,
    checked: Checked<StaticRigidMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl RigidModel {
    /// 先在宿主校验元数据与参数，再上传。
    pub(super) fn upload(
        session: &TransferSession,
        metadata: Vec<i32>,
        parameters: Vec<f32>,
        variant: RigidVariant,
        abi: [u32; 4],
    ) -> Result<Self, TransferError> {
        let [nq, nb, nj, nmocap] = abi;
        if nb == 0 {
            return Err(InputError::InvalidDimension {
                field: "rigid_kinematics_nbody",
            }
            .into());
        }
        let static_dims = RigidStaticDims {
            variant,
            nq,
            nb,
            nj,
            nmocap,
            metadata_elements: u64::try_from(metadata.len()).map_err(|_| InputError::Overflow {
                field: "rigid_kinematics_metadata",
            })?,
            parameter_elements: u64::try_from(parameters.len()).map_err(|_| {
                InputError::Overflow {
                    field: "rigid_kinematics_parameters",
                }
            })?,
        };
        validate_metadata(&metadata, &static_dims).map_err(rigid_launch_error)?;
        let buffer = session.upload(&metadata)?;
        let parameters = session.upload(&parameters)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<StaticRigidMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &metadata,
                static_dims,
                revision.clone(),
            )
        }
        .map_err(rigid_launch_error)?;
        Ok(Self {
            session: session.clone(),
            static_dims,
            metadata: buffer,
            metadata_values: metadata,
            parameters,
            revision,
            checked,
        })
    }

    fn dims(&self, worlds: u32) -> Result<RigidDims, TransferError> {
        let dims = RigidDims {
            static_dims: self.static_dims,
            abi: [
                self.static_dims.nq,
                self.static_dims.nb,
                self.static_dims.nj,
                worlds,
            ],
        };
        rigid_lengths(&dims).map_err(rigid_launch_error)?;
        Ok(dims)
    }

    fn proof(&self, dims: &RigidDims) -> Result<Checked<RigidMetadata>, TransferError> {
        let span = self.metadata.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.static_dims)
            .map_err(rigid_launch_error)?;
        // SAFETY: 私有包装持有实际缓冲、完整镜像与写入代际。
        unsafe {
            Checked::<RigidMetadata>::assume_uploaded(
                span,
                &self.metadata_values,
                *dims,
                self.revision.clone(),
            )
        }
        .map_err(rigid_launch_error)
    }

    /// 真实写入设备元数据并使证明失效。
    #[cfg(test)]
    pub(super) fn overwrite_for_test(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.metadata.write_range(index, &[value])
    }

    /// 写回原镜像并重新绑定证明。
    #[cfg(test)]
    pub(super) fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.metadata.write_range(0, &self.metadata_values)?;
        // SAFETY: 恢复写入已同步；跨度与镜像重新绑定。
        self.checked = unsafe {
            Checked::<StaticRigidMetadata>::assume_uploaded(
                self.metadata.launcher_span()?,
                &self.metadata_values,
                self.static_dims,
                self.revision.clone(),
            )
        }
        .map_err(rigid_launch_error)?;
        Ok(())
    }
}

/// 已编译内核与会话绑定。
#[cfg(feature = "cuda-probe")]
pub(super) struct RigidKernel {
    session: TransferSession,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RigidKernel {
    /// 接收调用方已编译的同名内核。
    pub(super) fn new(session: &TransferSession, kernel: SynchronousKernel) -> Self {
        Self {
            session: session.clone(),
            kernel,
        }
    }

    fn bindings(
        model: &RigidModel,
        state: &TransferBuffer<f32>,
        result: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 4], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: model.metadata.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: model.parameters.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: state.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: result.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    /// 先证明、再校验精确跨度与别名，最后发射内核。
    pub(super) fn launch(
        &self,
        model: &RigidModel,
        state: &TransferBuffer<f32>,
        result: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        self.check_session(model)?;
        let dims = model.dims(worlds)?;
        let proof = model.proof(&dims)?;
        let bindings = Self::bindings(model, state, result)?;
        rigid_plan(&dims, &bindings, proof.binding(0)).map_err(rigid_launch_error)?;
        // SAFETY: 类型计划核对父体前序、关节与qpos区间、mocap映射、字段描述符、
        // 精确跨度、同会话、别名与256线程网格。内核只写守卫结果区间；
        // 同步适配器在返回前等待完成。
        unsafe {
            self.kernel.launch(
                &model.metadata,
                &model.parameters,
                state,
                result,
                dims.abi(),
            )
        }
    }

    fn check_session(&self, model: &RigidModel) -> Result<(), TransferError> {
        // 内核与模型同会话；其余缓冲的上下文由联合绑定校验覆盖。
        if self.session.same_session(&model.session) {
            Ok(())
        } else {
            Err(TransferError::SessionMismatch)
        }
    }

    /// 对照用原始发射：跳过类型计划，仍固定同一ABI。
    #[cfg(test)]
    pub(super) fn launch_raw(
        &self,
        model: &RigidModel,
        state: &TransferBuffer<f32>,
        result: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = model.dims(worlds)?;
        // SAFETY: 测试单线程；缓冲、ABI、会话与产品发射完全相同。
        unsafe {
            self.kernel.launch(
                &model.metadata,
                &model.parameters,
                state,
                result,
                dims.abi(),
            )
        }
    }

    /// 测试用：伪造状态槽跨度后复用同一计划校验。
    #[cfg(test)]
    pub(super) fn plan_with_state_span_for_test(
        &self,
        model: &RigidModel,
        state: &TransferBuffer<f32>,
        result: &TransferBuffer<f32>,
        worlds: u32,
        span: DeviceSpan,
    ) -> Result<(), LauncherError> {
        let dims = RigidDims {
            static_dims: model.static_dims,
            abi: [
                model.static_dims.nq,
                model.static_dims.nb,
                model.static_dims.nj,
                worlds,
            ],
        };
        rigid_lengths(&dims)?;
        let metadata = model
            .metadata
            .launcher_span()
            .map_err(|_| LauncherError::InvalidSpan("rigid_kinematics_session"))?;
        model
            .checked
            .binding(0)
            .validate_for(metadata, &model.static_dims)?;
        // SAFETY: 测试单线程；镜像与实际缓冲一致。
        let proof = unsafe {
            Checked::<RigidMetadata>::assume_uploaded(
                metadata,
                &model.metadata_values,
                dims,
                model.revision.clone(),
            )
        }?;
        let mut bindings = Self::bindings(model, state, result)
            .map_err(|_| LauncherError::InvalidSpan("rigid_kinematics_bindings"))?;
        bindings[2].span = span;
        rigid_plan(&dims, &bindings, proof.binding(0))
    }
}
