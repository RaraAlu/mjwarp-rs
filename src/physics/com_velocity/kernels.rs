//! 空间速度同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComVelocityDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::com_velocity_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::com_velocity_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::ComVelocityMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct ComVelocityKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ComVelocityKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "com_velocity")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ComVelocityMetadataBuffer,
        qvel: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = ComVelocityDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.proof(dims)?;
        let bindings = [
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
                span: com.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        com_velocity_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(com_velocity_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), qvel, com, output, dims.abi())
        }
    }
}
