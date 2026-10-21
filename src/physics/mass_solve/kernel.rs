//! 质量求解内核包装与发射。

#[cfg(feature = "cuda-probe")]
use super::buffer::RigidMassSolveMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::errors::rigid_mass_solve_launch_error;
#[cfg(feature = "cuda-probe")]
use super::proof::RigidMassSolveMetadata;
#[cfg(feature = "cuda-probe")]
use super::proof::rigid_mass_solve_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct RigidMassSolveLaunch {
    pub(super) proof: Checked<RigidMassSolveMetadata>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RigidMassSolveKernel {
    pub(super) session: TransferSession,
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RigidMassSolveKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, MASS_SOLVE_CUDA, "rigid_mass_solve")?,
        })
    }

    pub(super) fn bindings(
        metadata: &RigidMassSolveMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
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
                span: rhs.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: state.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    pub(super) fn preflight(
        &self,
        session: &TransferSession,
        metadata: &RigidMassSolveMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<RigidMassSolveLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.proof()?;
        let bindings = Self::bindings(metadata, rhs, state, output)?;
        rigid_mass_solve_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rigid_mass_solve_launch_error)?;
        Ok(RigidMassSolveLaunch { proof })
    }

    pub(super) fn launch(
        &self,
        launch: &RigidMassSolveLaunch,
        metadata: &RigidMassSolveMetadataBuffer,
        rhs: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, rhs, state, output)?;
        rigid_mass_solve_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rigid_mass_solve_launch_error)?;
        // SAFETY: 类型计划核对块表、精确跨度、会话、别名和网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), rhs, state, output, dims.abi)
        }
    }
}
