//! 常驻附着运动学两类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::ResidentAttachedDims;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_meta::resident_attached_launch_error;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_specs::resident_attached_plan;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_specs::resident_initialize_attached_plan;
#[cfg(feature = "cuda-probe")]
use super::attached_metadata_buffer::ResidentAttachedMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn bindings<'a>(
    metadata: &'a ResidentAttachedMetadataBuffer,
    parameters: &'a TransferBuffer<f32>,
    body: &'a TransferBuffer<f32>,
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
            span: parameters.launcher_span()?,
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: body.launcher_span()?,
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
pub(in crate::physics) struct ResidentAttachedKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentAttachedKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "attached_kinematics")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ResidentAttachedMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        body: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = ResidentAttachedDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.proof(dims)?;
        resident_attached_plan(
            &dims,
            &bindings(metadata, parameters, body, output)?,
            &[proof.binding(0)],
        )
        .map_err(resident_attached_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), parameters, body, output, dims.abi())
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct ResidentInitializeAttachedKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentInitializeAttachedKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "initialize_attached")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ResidentAttachedMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        body: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = ResidentAttachedDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.proof(dims)?;
        resident_initialize_attached_plan(
            &dims,
            &bindings(metadata, parameters, body, output)?,
            &[proof.binding(0)],
        )
        .map_err(resident_attached_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), parameters, body, output, dims.abi())
        }
    }
}
