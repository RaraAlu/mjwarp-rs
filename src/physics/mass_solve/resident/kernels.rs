//! 常驻质量分解两类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassFactorDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassSolveDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::resident_mass_factor_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::resident_mass_solve_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_mass_factor_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_mass_solve_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::ResidentMassMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMassFactorKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMassFactorKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "resident_mass_factor")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ResidentMassMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        matrix: &TransferBuffer<f32>,
        factor: &mut TransferBuffer<f32>,
        worlds: u32,
        rhs_count: u32,
    ) -> Result<(), TransferError> {
        let dims = ResidentMassFactorDims {
            static_dims: metadata.static_dims(),
            worlds,
            rhs_count,
        };
        let proof = metadata.factor_proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: rhs.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: matrix.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: factor.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        resident_mass_factor_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(resident_mass_factor_launch_error)?;
        let abi = dims.abi();
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), rhs, matrix, factor, abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMassSubstituteKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMassSubstituteKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "resident_mass_substitute")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ResidentMassMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
        rhs_count: u32,
    ) -> Result<(), TransferError> {
        let dims = ResidentMassSolveDims {
            static_dims: metadata.static_dims(),
            worlds,
            rhs_count,
        };
        let proof = metadata.solve_proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: rhs.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: factor.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        resident_mass_solve_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(resident_mass_solve_launch_error)?;
        let abi = dims.abi();
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), rhs, factor, output, abi)
        }
    }
}
