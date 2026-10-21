//! 休眠复制内核的类型化包装。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceSleep {
    pub(super) metadata: SleepCopyMetadataBuffer,
    pub(super) parameters: TransferBuffer<f32>,
    pub(super) copy_kernel: SleepCopyKernel,
    pub(super) wake_kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
pub(super) const SLEEP_COPY_METADATA_KIND: ProofKind = ProofKind("resident.sleep_copy.metadata.v1");

#[cfg(feature = "cuda-probe")]
pub(super) const SLEEP_COPY_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SLEEP_COPY_METADATA_KIND,
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
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Write,
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

#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SleepCopyStaticDims {
    pub(super) ntree: u32,
    pub(super) metadata_elements: u64,
}

#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SleepCopyDims {
    pub(super) abi: [u32; 4],
    pub(super) ntree: u32,
    pub(super) metadata_elements: u64,
    pub(super) parameter_elements: u64,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SleepCopySpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结sleep_copy ABI。
unsafe impl KernelAbi for SleepCopySpec {
    const NAME: &'static str = "sleep_copy";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLEEP_COPY_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SleepCopySpec {
    type Dims = SleepCopyDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let state = sleep_copy_state_elements(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: dims.metadata_elements,
            },
            SlotRequirement {
                slot: 1,
                min_elements: dims.parameter_elements,
            },
            SlotRequirement {
                slot: 2,
                min_elements: state,
            },
            SlotRequirement {
                slot: 3,
                min_elements: state,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        sleep_copy_state_elements(dims)?;
        LaunchDims::new((dims.abi[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sleep_copy_state_elements(dims: &SleepCopyDims) -> Result<u64, LauncherError> {
    let [a, b, c, worlds] = dims.abi;
    if [a, b, c] != [0; 3]
        || worlds == 0
        || worlds > u32::MAX - 255
        || dims.ntree > i32::MAX as u32
        || dims.metadata_elements == 0
        || dims.parameter_elements == 0
    {
        return Err(LauncherError::InvalidArray("休眠复制维度无效"));
    }
    u64::from(dims.ntree)
        .checked_mul(2)
        .and_then(|value| value.checked_add(3))
        .and_then(|value| value.checked_mul(u64::from(worlds)))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("sleep_copy_state"))
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sleep_copy_plan(
    dims: &SleepCopyDims,
    bindings: &[SlotBinding],
    proof: CheckedBinding<'_, SleepCopyDims>,
) -> Result<(), LauncherError> {
    let exact_bytes = [
        sleep_copy_buffer_bytes(dims, 0)?,
        sleep_copy_buffer_bytes(dims, 1)?,
        sleep_copy_buffer_bytes(dims, 2)?,
        sleep_copy_buffer_bytes(dims, 3)?,
    ];
    Kernel::<SleepCopySpec>::new()?.plan(dims, bindings, &[proof])?;
    for (slot, required) in exact_bytes.into_iter().enumerate() {
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        if binding.span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: binding.span.bytes(),
            });
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sleep_copy_buffer_bytes(
    dims: &SleepCopyDims,
    slot: usize,
) -> Result<u64, LauncherError> {
    let elements = match slot {
        0 => dims.metadata_elements,
        1 => dims.parameter_elements,
        2 | 3 => sleep_copy_state_elements(dims)?,
        _ => return Err(LauncherError::UnexpectedBinding { slot }),
    };
    elements
        .checked_mul(4)
        .ok_or(LauncherError::Overflow(match slot {
            0 => "sleep_copy_metadata_bytes",
            1 => "sleep_copy_parameter_bytes",
            2 => "sleep_copy_state_bytes",
            3 => "sleep_copy_result_bytes",
            _ => unreachable!(),
        }))
}

#[cfg(feature = "cuda-probe")]
pub(super) fn validate_sleep_copy_metadata(
    values: &[i32],
    dims: SleepCopyStaticDims,
) -> Result<(), LauncherError> {
    if u64::try_from(values.len()).ok() != Some(dims.metadata_elements) {
        return Err(LauncherError::InvalidMetadata("休眠复制元数据长度无效"));
    }
    let Some(&ntree) = values.first() else {
        return Err(LauncherError::InvalidMetadata("休眠复制元数据为空"));
    };
    if ntree < 0 || ntree as u32 != dims.ntree {
        return Err(LauncherError::InvalidMetadata("休眠复制树数量不匹配"));
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
pub(super) struct StaticSleepCopyMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSleepCopyMetadata {
    type Dims = SleepCopyStaticDims;

    const KIND: ProofKind = SLEEP_COPY_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sleep_copy_metadata(values, *dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SleepCopyMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SleepCopyMetadata {
    type Dims = SleepCopyDims;

    const KIND: ProofKind = SLEEP_COPY_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_sleep_copy_metadata(
            values,
            SleepCopyStaticDims {
                ntree: dims.ntree,
                metadata_elements: dims.metadata_elements,
            },
        )?;
        sleep_copy_state_elements(dims).map(|_| ())
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn sleep_copy_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata
        | LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_) => InputError::InvalidTopology {
            field: "sleep_copy_metadata",
            index: 0,
            reason: "invalid_or_stale_metadata",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "sleep_copy_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SleepCopyMetadataBuffer {
    pub(super) buffer: TransferBuffer<i32>,
    pub(super) values: Vec<i32>,
    pub(super) dims: SleepCopyStaticDims,
    pub(super) revision: MetadataRevision,
    pub(super) checked: Checked<StaticSleepCopyMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SleepCopyMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
    ) -> Result<Self, TransferError> {
        let ntree = values
            .first()
            .copied()
            .ok_or(InputError::InvalidDimension {
                field: "sleep_copy_metadata",
            })?;
        let ntree = u32::try_from(ntree).map_err(|_| InputError::InvalidDimension {
            field: "sleep_copy_ntree",
        })?;
        let dims = SleepCopyStaticDims {
            ntree,
            metadata_elements: u64::try_from(values.len()).map_err(|_| InputError::Overflow {
                field: "sleep_copy_metadata",
            })?,
        };
        validate_sleep_copy_metadata(&values, dims).map_err(sleep_copy_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定实际设备缓冲。
        let checked = unsafe {
            Checked::<StaticSleepCopyMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(sleep_copy_launch_error)?;
        Ok(Self {
            buffer,
            values,
            dims,
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn proof(
        &self,
        worlds: u32,
        parameter_elements: u64,
    ) -> Result<Checked<SleepCopyMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.dims)
            .map_err(sleep_copy_launch_error)?;
        let dims = SleepCopyDims {
            abi: [0, 0, 0, worlds],
            ntree: self.dims.ntree,
            metadata_elements: self.dims.metadata_elements,
            parameter_elements,
        };
        // SAFETY: 私有包装持有实际缓冲、完整镜像和写入代际。
        unsafe {
            Checked::<SleepCopyMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sleep_copy_launch_error)
    }

    #[cfg(test)]
    pub(super) fn overwrite_for_test(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    pub(super) fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入已同步；跨度与镜像重新绑定。
        self.checked = unsafe {
            Checked::<StaticSleepCopyMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.dims,
                self.revision.clone(),
            )
        }
        .map_err(sleep_copy_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SleepCopyKernel {
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl SleepCopyKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::sleep::TENDON_WAKE_CUDA,
                "sleep_copy",
            )?,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &SleepCopyMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        state: &TransferBuffer<i32>,
        result: &mut TransferBuffer<i32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let parameter_elements =
            u64::try_from(parameters.len()).map_err(|_| InputError::Overflow {
                field: "sleep_copy_parameters",
            })?;
        let proof = metadata.proof(worlds, parameter_elements)?;
        let dims = *proof.dims();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: parameters.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: state.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 3,
                span: result.launcher_span()?,
                elem: ElemType::I32,
            },
        ];
        sleep_copy_plan(&dims, &bindings, proof.binding(0)).map_err(sleep_copy_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), parameters, state, result, dims.abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentSleep {
    pub(super) state: TransferBuffer<i32>,
    pub(super) scratch: TransferBuffer<i32>,
}
