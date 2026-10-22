//! 常驻柔体 Hessian 的规格、证明与同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldBatchSpec;
#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_launch_dims;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_spans;
#[cfg(feature = "cuda-probe")]
use super::field_batch::launch_field_batch;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::FlexHessianDims;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::HESSIAN_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::PROOF;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::VALIDATE_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::flex_hessian_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::flex_hessian_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::validate_flex_hessian_metadata;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexHessianSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结flex_hessian六缓冲ABI。
unsafe impl KernelAbi for FlexHessianSpec {
    const NAME: &'static str = "flex_hessian";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &HESSIAN_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for FlexHessianSpec {
    type Dims = FlexHessianDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for FlexHessianSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(flex_hessian_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexHessianValidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结flex_hessian_validate FIELD_BATCH ABI。
unsafe impl KernelAbi for FlexHessianValidateSpec {
    const NAME: &'static str = "flex_hessian_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &VALIDATE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for FlexHessianValidateSpec {
    type Dims = FlexHessianDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for FlexHessianValidateSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(flex_hessian_validate_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexHessianMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for FlexHessianMetadata {
    type Dims = FlexHessianDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_flex_hessian_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn flex_hessian_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_flex_hessian_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_flex_hessian_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_flex_hessian_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_flex_hessian_abi",
        }
        .into(),
    }
}

/// 柔体Hessian元数据缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) type FlexHessianMetadataBuffer = FieldMetadataBuffer<FlexHessianMetadata>;

#[cfg(feature = "cuda-probe")]
fn dims_of(metadata: &FlexHessianMetadataBuffer, worlds: u32) -> FlexHessianDims {
    FlexHessianDims {
        static_dims: metadata.base().static_dims,
        worlds,
    }
}

/// 柔体Hessian缓存内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FlexHessianKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl FlexHessianKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::flex_hessian::FLEX_HESSIAN_CUDA,
                "flex_hessian",
            )?,
        })
    }

    /// 六缓冲发射：先核对证明、精确跨度、同会话、别名与256线程网格。
    pub(super) fn launch(
        &self,
        metadata: &FlexHessianMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        buffers: (&TransferBuffer<f32>, &TransferBuffer<f32>),
        validity: &TransferBuffer<i32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let (positions, edges) = buffers;
        let dims = dims_of(metadata, worlds);
        let proof = metadata.proof(dims)?;
        let bindings = [
            metadata.binding()?,
            SlotBinding {
                slot: 1,
                span: parameters.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: positions.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: edges.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 4,
                span: validity.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 5,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        field_plan::<FlexHessianSpec>(&dims, &bindings, &[proof.binding(0)])
            .map_err(flex_hessian_launch_error)?;
        // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与256线程网格。
        unsafe {
            self.kernel.launch_flex_hessian(
                (metadata.as_buffer(), parameters),
                positions,
                edges,
                validity,
                output,
                worlds,
            )
        }
    }

    #[cfg(test)]
    pub(super) fn from_kernel(kernel: SynchronousKernel) -> Self {
        Self { kernel }
    }
}

/// 柔体Hessian发布内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FlexHessianValidateKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl FlexHessianValidateKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::flex_hessian::FLEX_HESSIAN_CUDA,
                "flex_hessian_validate",
            )?,
        })
    }

    /// 四缓冲发射：读Hessian缓存，写失效标志。
    pub(super) fn launch(
        &self,
        metadata: &FlexHessianMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        hessian: &TransferBuffer<f32>,
        validity: &mut TransferBuffer<i32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = dims_of(metadata, worlds);
        launch_field_batch::<FlexHessianValidateSpec, FlexHessianMetadata, f32, f32, i32>(
            &self.kernel,
            metadata,
            &dims,
            parameters,
            hessian,
            validity,
        )
    }

    #[cfg(test)]
    pub(super) fn from_kernel(kernel: SynchronousKernel) -> Self {
        Self { kernel }
    }
}
