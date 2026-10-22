//! 常驻柔体失效与位置的规格、证明与同步内核包装。

#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldBatchSpec;
#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_launch_dims;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_spans;
#[cfg(feature = "cuda-probe")]
use super::field_batch::launch_field_batch;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::FlexDims;
#[cfg(feature = "cuda-probe")]
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::INVALIDATE_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::POSITIONS_SLOTS;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::PROOF;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::flex_invalidate_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::flex_positions_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::validate_flex_metadata;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexInvalidateSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结flex_invalidate FIELD_BATCH ABI。
unsafe impl KernelAbi for FlexInvalidateSpec {
    const NAME: &'static str = "flex_invalidate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &INVALIDATE_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for FlexInvalidateSpec {
    type Dims = FlexDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for FlexInvalidateSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(flex_invalidate_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexPositionsSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结flex_positions FIELD_BATCH ABI。
unsafe impl KernelAbi for FlexPositionsSpec {
    const NAME: &'static str = "flex_positions";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &POSITIONS_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for FlexPositionsSpec {
    type Dims = FlexDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for FlexPositionsSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(flex_positions_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct FlexMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for FlexMetadata {
    type Dims = FlexDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_flex_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn flex_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_flex_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_flex_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_flex_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_flex_abi",
        }
        .into(),
    }
}

/// 柔体位置元数据缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) type FlexMetadataBuffer = FieldMetadataBuffer<FlexMetadata>;

#[cfg(feature = "cuda-probe")]
fn dims_of(metadata: &FlexMetadataBuffer, worlds: u32) -> FlexDims {
    FlexDims {
        static_dims: metadata.base().static_dims,
        worlds,
    }
}

/// 柔体失效同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FlexInvalidateKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl FlexInvalidateKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::flex::FLEX_POSITION_CUDA,
                "flex_invalidate",
            )?,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &FlexMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        rigid: &TransferBuffer<f32>,
        validity: &mut TransferBuffer<i32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = dims_of(metadata, worlds);
        launch_field_batch::<FlexInvalidateSpec, FlexMetadata, f32, f32, i32>(
            &self.kernel,
            metadata,
            &dims,
            parameters,
            rigid,
            validity,
        )
    }

    #[cfg(test)]
    pub(super) fn from_kernel(kernel: SynchronousKernel) -> Self {
        Self { kernel }
    }
}

/// 柔体位置同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FlexPositionsKernel {
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl FlexPositionsKernel {
    pub(super) fn compile(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            kernel: SynchronousKernel::compile(
                session,
                crate::physics::flex::FLEX_POSITION_CUDA,
                "flex_positions",
            )?,
        })
    }

    pub(super) fn launch(
        &self,
        metadata: &FlexMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        rigid: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = dims_of(metadata, worlds);
        launch_field_batch::<FlexPositionsSpec, FlexMetadata, f32, f32, f32>(
            &self.kernel,
            metadata,
            &dims,
            parameters,
            rigid,
            output,
        )
    }

    #[cfg(test)]
    pub(super) fn from_kernel(kernel: SynchronousKernel) -> Self {
        Self { kernel }
    }
}
