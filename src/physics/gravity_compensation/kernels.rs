//! 重力补偿四类同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityClearDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravitySumDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::gravity_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_clear_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_compute_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_sum_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::GravityMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityClearKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl GravityClearKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "gravity_compensation_clear")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &GravityMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = GravityClearDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.clear_proof(dims)?;
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
                span: parameters.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        gravity_clear_plan(&dims, &bindings, &[proof.binding(0)]).map_err(gravity_launch_error)?;
        let abi = dims.abi().map_err(gravity_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), parameters, parameters, output, abi)
        }
    }
}

#[cfg(feature = "cuda-probe")]
fn five_bindings<'a>(
    metadata: &'a GravityMetadataBuffer,
    parameters: &'a TransferBuffer<f32>,
    rigid: &'a TransferBuffer<f32>,
    com: &'a TransferBuffer<f32>,
    output: &'a TransferBuffer<f32>,
) -> Result<[SlotBinding; 5], TransferError> {
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
            span: rigid.launcher_span()?,
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: com.launcher_span()?,
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 4,
            span: output.launcher_span()?,
            elem: ElemType::F32,
        },
    ])
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityValidateKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl GravityValidateKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "gravity_compensation_validate")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &GravityMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        rigid: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = GravityValidateDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.validate_proof(dims)?;
        gravity_validate_plan(
            &dims,
            &five_bindings(metadata, parameters, rigid, com, status)?,
            &[proof.binding(0)],
        )
        .map_err(gravity_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel.launch_gravity_compensation(
                metadata.as_buffer(),
                parameters,
                rigid,
                com,
                status,
                dims.abi(),
            )
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravityComputeKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl GravityComputeKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "gravity_compensation_compute")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &GravityMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        rigid: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = GravityComputeDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.compute_proof(dims)?;
        let abi = dims.abi().map_err(gravity_launch_error)?;
        gravity_compute_plan(
            &dims,
            &five_bindings(metadata, parameters, rigid, com, output)?,
            &[proof.binding(0)],
        )
        .map_err(gravity_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel.launch_gravity_compensation(
                metadata.as_buffer(),
                parameters,
                rigid,
                com,
                output,
                abi,
            )
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct GravitySumKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl GravitySumKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(session, source, "gravity_compensation_sum")?,
        })
    }

    /// 先核对证明、精确跨度、同会话、别名与256线程网格，再发射。
    pub(super) fn launch(
        &self,
        metadata: &GravityMetadataBuffer,
        parts: &TransferBuffer<f32>,
        gravity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = GravitySumDims {
            static_dims: metadata.static_dims(),
            worlds,
        };
        let proof = metadata.sum_proof(dims)?;
        let abi = dims.abi().map_err(gravity_launch_error)?;
        gravity_sum_plan(
            &dims,
            &five_bindings(metadata, parts, gravity, gravity, output)?,
            &[proof.binding(0)],
        )
        .map_err(gravity_launch_error)?;
        // SAFETY: 类型计划已核对证明、跨度、会话、别名与网格。
        unsafe {
            self.kernel.launch_gravity_compensation(
                metadata.as_buffer(),
                parts,
                gravity,
                gravity,
                output,
                abi,
            )
        }
    }
}
