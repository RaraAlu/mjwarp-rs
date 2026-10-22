//! 常驻CRB三类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbAccumulateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbInitDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbMatrixDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::resident_crb_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_crb_accumulate_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_crb_init_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_crb_matrix_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::ResidentCrbMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn bindings<'a>(
    metadata: &'a ResidentCrbMetadataBuffer,
    armature: &'a TransferBuffer<f32>,
    state: &'a TransferBuffer<f32>,
    output: &'a TransferBuffer<f32>,
) -> Result<[SlotBinding; 4], TransferError> {
    Ok([
        SlotBinding {
            slot: 0,
            span: metadata.as_buffer().launcher_span()?,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: armature.launcher_span()?,
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

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbInitKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCrbInitKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "resident_crb_init")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ResidentCrbMetadataBuffer,
        armature: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = ResidentCrbInitDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.init_proof(dims)?;
        resident_crb_init_plan(
            &dims,
            &bindings(metadata, armature, state, output)?,
            &[proof.binding(0)],
        )
        .map_err(resident_crb_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), armature, state, output, dims.abi())
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbAccumulateKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCrbAccumulateKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "resident_crb_accumulate")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ResidentCrbMetadataBuffer,
        armature: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
        level: u32,
    ) -> Result<(), TransferError> {
        let dims = ResidentCrbAccumulateDims {
            static_dims: metadata.static_dims(),
            worlds,
            level,
        };
        let proof = metadata.accumulate_proof(dims)?;
        let abi = dims.abi().map_err(resident_crb_launch_error)?;
        resident_crb_accumulate_plan(
            &dims,
            &bindings(metadata, armature, state, output)?,
            &[proof.binding(0)],
        )
        .map_err(resident_crb_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), armature, state, output, abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbMatrixKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCrbMatrixKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "resident_crb_matrix")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ResidentCrbMetadataBuffer,
        armature: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = ResidentCrbMatrixDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.matrix_proof(dims)?;
        let abi = dims.abi().map_err(resident_crb_launch_error)?;
        resident_crb_matrix_plan(
            &dims,
            &bindings(metadata, armature, state, output)?,
            &[proof.binding(0)],
        )
        .map_err(resident_crb_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), armature, state, output, abi)
        }
    }
}
