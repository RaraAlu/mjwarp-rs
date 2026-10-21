//! 动能校验与归约内核包装。

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
#[cfg(feature = "cuda-probe")]
use super::metadata::KineticEnergy32Spec;
#[cfg(feature = "cuda-probe")]
use super::metadata::KineticValidationSpec;
#[cfg(feature = "cuda-probe")]
use super::metadata::kinetic_lengths;
#[cfg(feature = "cuda-probe")]
use super::metadata::transfer_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct KineticValidationKernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl KineticValidationKernel {
    pub(in crate::physics) fn compile(
        session: &TransferSession,
        nv: u32,
    ) -> Result<Self, TransferError> {
        let source = KineticEnergy32Kernel::source(nv)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, &source, "kinetic_energy_validate")?,
            nv,
        })
    }

    pub(in crate::physics) fn launch(
        &self,
        metadata: &KineticEnergyMetadataBuffer,
        pair: &TransferBuffer<f32>,
        current: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        if dims[0] != self.nv || metadata.nv != self.nv {
            return Err(InputError::InvalidDimension {
                field: "kinetic_energy_validate_static_nv",
            }
            .into());
        }
        let (pair_len, energy_len) = kinetic_lengths(&dims).map_err(transfer_launch_error)?;
        let status_len = u64::from(dims[1])
            .checked_add(8)
            .ok_or(InputError::Overflow {
                field: "kinetic_energy_status",
            })?;
        if metadata.len() != 1
            || u64::try_from(pair.len()).ok() != Some(pair_len)
            || u64::try_from(current.len()).ok() != Some(energy_len)
            || u64::try_from(status.len()).ok() != Some(status_len)
        {
            return Err(InputError::InvalidDimension {
                field: "kinetic_energy_validate_abi",
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
                span: pair.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: current.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<KineticValidationSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(transfer_launch_error)?;
        // SAFETY: 类型化计划已核对全部真实跨度和元数据代际。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), pair, current, status, dims)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct KineticEnergy32Kernel {
    pub(super) kernel: SynchronousKernel,
    pub(super) nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl KineticEnergy32Kernel {
    pub(in crate::physics) fn source(nv: u32) -> Result<String, TransferError> {
        if !(1..=1024).contains(&nv) {
            return Err(InputError::InvalidDimension {
                field: "kinetic_energy_nv",
            }
            .into());
        }
        Ok(format!("#define ENERGY_NV {nv}\n{CUDA}"))
    }

    pub(in crate::physics) fn compile(
        session: &TransferSession,
        nv: u32,
    ) -> Result<Self, TransferError> {
        let source = Self::source(nv)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, &source, "kinetic_energy_reduce32")?,
            nv,
        })
    }

    pub(in crate::physics) fn launch(
        &self,
        metadata: &KineticEnergyMetadataBuffer,
        pair: &TransferBuffer<f32>,
        current: &TransferBuffer<f32>,
        staged: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        if dims[0] != self.nv {
            return Err(InputError::InvalidDimension {
                field: "kinetic_energy_static_nv",
            }
            .into());
        }
        let (pair_len, energy_len) = kinetic_lengths(&dims).map_err(transfer_launch_error)?;
        if metadata.len() != 1
            || u64::try_from(pair.len()).ok() != Some(pair_len)
            || u64::try_from(current.len()).ok() != Some(energy_len)
            || u64::try_from(staged.len()).ok() != Some(energy_len)
        {
            return Err(InputError::InvalidDimension {
                field: "kinetic_energy32_abi",
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
                span: pair.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: current.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: staged.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<KineticEnergy32Spec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(transfer_launch_error)?;
        // SAFETY: 可信构造绑定冻结源码与静态NV。
        // 类型化计划已核对全部真实跨度。
        unsafe {
            self.kernel
                .launch_kinetic_energy32(metadata.as_buffer(), pair, current, staged, dims)
        }
    }
}
