use super::plan::SOURCE_VALIDATE;
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) const SOURCE_VALIDATION_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: KINETIC_METADATA_KIND,
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
pub(super) struct DeviceEnergySourceValidationSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结校验ABI。
unsafe impl KernelAbi for DeviceEnergySourceValidationSpec {
    const NAME: &'static str = "device_energy_source_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SOURCE_VALIDATION_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for DeviceEnergySourceValidationSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (source, status) = source_validation_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 1,
                min_elements: source,
            },
            SlotRequirement {
                slot: 2,
                min_elements: source,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        source_validation_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn source_validation_lengths(dims: &[u32; 4]) -> Result<(u64, u64), LauncherError> {
    let [nv, worlds, reserved, work] = *dims;
    if !(1..=1024).contains(&nv)
        || worlds == 0
        || worlds > i32::MAX as u32
        || reserved != 0
        || work != worlds
    {
        return Err(LauncherError::InvalidArray("设备动能源校验维度无效"));
    }
    let source = u64::from(worlds)
        .checked_mul(u64::from(nv))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("device_energy_source_elements"))?;
    let status = u64::from(worlds)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("device_energy_status_elements"))?;
    Ok((source, status))
}

#[cfg(feature = "cuda-probe")]
pub(super) fn source_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "device_energy_source_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "device_energy_source_validate_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceEnergySourceValidationKernel {
    kernel: SynchronousKernel,
    nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl DeviceEnergySourceValidationKernel {
    pub(super) fn compile(session: &TransferSession, nv: u32) -> Result<Self, TransferError> {
        source_validation_lengths(&[nv, 1, 0, 1]).map_err(source_launch_error)?;
        let source = format!("#define ENERGY_NV {nv}\n{SOURCE_VALIDATE}");
        Ok(Self {
            kernel: SynchronousKernel::compile(session, &source, "device_energy_source_validate")?,
            nv,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &KineticEnergyMetadataBuffer,
        q: &TransferBuffer<f32>,
        mq: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        if dims[0] != self.nv || metadata.nv() != self.nv {
            return Err(InputError::InvalidDimension {
                field: "device_energy_source_validate_static_nv",
            }
            .into());
        }
        let (source_len, status_len) =
            source_validation_lengths(&dims).map_err(source_launch_error)?;
        if metadata.len() != 1
            || u64::try_from(q.len()).ok() != Some(source_len)
            || u64::try_from(mq.len()).ok() != Some(source_len)
            || u64::try_from(status.len()).ok() != Some(status_len)
        {
            return Err(InputError::InvalidDimension {
                field: "device_energy_source_validate_abi",
            }
            .into());
        }
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: q.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: mq.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<DeviceEnergySourceValidationSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(source_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), q, mq, status, dims)
        }
    }
}
