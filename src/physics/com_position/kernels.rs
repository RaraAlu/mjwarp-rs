//! 质心位置同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComPositionDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::com_position_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::com_position_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::ComPositionMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct ComPositionKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ComPositionKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "rigid_com_position")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &ComPositionMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = ComPositionDims {
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
                span: parameters.launcher_span()?,
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
        ];
        com_position_plan(&dims, &bindings, &[proof.binding(0)])
            .map_err(com_position_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), parameters, state, output, dims.abi())
        }
    }
}
