//! RNE四类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_plans::RneBackwardMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::RneCaccMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::RneCfrcMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::RneQfrcMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_backward_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_backward_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_cacc_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_cacc_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_cfrc_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_cfrc_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_qfrc_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_qfrc_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::RneMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct RneCaccLaunch {
    pub(super) proof: Checked<RneCaccMetadata>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RneCaccKernel {
    pub(super) session: TransferSession,
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneCaccKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_CUDA, "rne_cacc")?,
        })
    }

    pub(super) fn bindings(
        metadata: &RneMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        qacc: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
    ) -> Result<[SlotBinding; 6], TransferError> {
        Ok([
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: qvel.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: qacc.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: velocity.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 5,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ])
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn preflight(
        &self,
        session: &TransferSession,
        metadata: &RneMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        qacc: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        flg_acc: bool,
        worlds: u32,
    ) -> Result<RneCaccLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.cacc_proof(flg_acc, worlds)?;
        let bindings = Self::bindings(metadata, qvel, qacc, com, velocity, output)?;
        rne_cacc_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_cacc_launch_error)?;
        Ok(RneCaccLaunch { proof })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn launch(
        &self,
        launch: &RneCaccLaunch,
        metadata: &RneMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        qacc: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, qvel, qacc, com, velocity, output)?;
        rne_cacc_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_cacc_launch_error)?;
        // SAFETY: 类型计划核对证明、六个跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel.launch_rne_cacc(
                metadata.as_buffer(),
                (qvel, qacc, com, velocity),
                output,
                dims.flag != 0,
                dims.worlds,
            )
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RneCfrcLaunch {
    pub(super) proof: Checked<RneCfrcMetadata>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RneCfrcKernel {
    pub(super) session: TransferSession,
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneCfrcKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_CUDA, "rne_cfrc")?,
        })
    }

    pub(super) fn bindings(
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
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
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: velocity.launcher_span()?,
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
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<RneCfrcLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.cfrc_proof(worlds)?;
        let bindings = Self::bindings(metadata, com, velocity, output)?;
        rne_cfrc_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_cfrc_launch_error)?;
        Ok(RneCfrcLaunch { proof })
    }

    pub(super) fn launch(
        &self,
        launch: &RneCfrcLaunch,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, com, velocity, output)?;
        rne_cfrc_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_cfrc_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), com, velocity, output, dims.abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RneBackwardLaunch {
    pub(super) proof: Checked<RneBackwardMetadata>,
}

#[cfg(feature = "cuda-probe")]
#[derive(Clone, Copy)]
pub(super) struct RneBackwardRequest {
    pub(super) worlds: u32,
    pub(super) level: u32,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RneBackwardKernel {
    pub(super) session: TransferSession,
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneBackwardKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_CUDA, "rne_backward")?,
        })
    }

    pub(super) fn bindings(
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
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
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: velocity.launcher_span()?,
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
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        request: RneBackwardRequest,
    ) -> Result<RneBackwardLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.backward_proof(request.worlds, request.level)?;
        let bindings = Self::bindings(metadata, com, velocity, output)?;
        rne_backward_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_backward_launch_error)?;
        Ok(RneBackwardLaunch { proof })
    }

    pub(super) fn launch(
        &self,
        launch: &RneBackwardLaunch,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, com, velocity, output)?;
        rne_backward_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_backward_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和逐层网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), com, velocity, output, dims.abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RneQfrcLaunch {
    pub(super) proof: Checked<RneQfrcMetadata>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct RneQfrcKernel {
    pub(super) session: TransferSession,
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl RneQfrcKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, RNE_CUDA, "rne_qfrc")?,
        })
    }

    pub(super) fn bindings(
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
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
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused.launcher_span()?,
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
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<RneQfrcLaunch, TransferError> {
        if !self.session.same_session(session) || !metadata.session.same_session(session) {
            return Err(TransferError::SessionMismatch);
        }
        let proof = metadata.qfrc_proof(worlds)?;
        let bindings = Self::bindings(metadata, com, unused, output)?;
        rne_qfrc_plan(proof.dims(), &bindings, &[proof.binding(0)])
            .map_err(rne_qfrc_launch_error)?;
        Ok(RneQfrcLaunch { proof })
    }

    pub(super) fn launch(
        &self,
        launch: &RneQfrcLaunch,
        metadata: &RneMetadataBuffer,
        com: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        let dims = *launch.proof.dims();
        let bindings = Self::bindings(metadata, com, unused, output)?;
        rne_qfrc_plan(&dims, &bindings, &[launch.proof.binding(0)])
            .map_err(rne_qfrc_launch_error)?;
        // SAFETY: 类型计划核对证明、跨度、会话、别名和256线程网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), com, unused, output, dims.abi)
        }
    }
}
