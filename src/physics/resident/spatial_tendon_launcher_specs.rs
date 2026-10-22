//! 常驻空间肌腱三项的规格与同步内核包装。

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
use super::spatial_tendon_launcher::PROOF;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SLOTS_F32;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SLOTS_I32;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SpatialTendonDims;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::spatial_moment_lengths;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::spatial_site_lengths;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::spatial_wrap_lengths;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::validate_spatial_tendon_metadata;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct SpatialSiteSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结spatial_site FIELD_BATCH ABI。
unsafe impl KernelAbi for SpatialSiteSpec {
    const NAME: &'static str = "spatial_site";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS_F32;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SpatialSiteSpec {
    type Dims = SpatialTendonDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for SpatialSiteSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(spatial_site_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SpatialMomentSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结spatial_moment FIELD_BATCH ABI。
unsafe impl KernelAbi for SpatialMomentSpec {
    const NAME: &'static str = "spatial_moment";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS_F32;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SpatialMomentSpec {
    type Dims = SpatialTendonDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for SpatialMomentSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(spatial_moment_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SpatialWrapSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明逐槽匹配冻结spatial_wrap FIELD_BATCH ABI。
unsafe impl KernelAbi for SpatialWrapSpec {
    const NAME: &'static str = "spatial_wrap";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SLOTS_I32;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SpatialWrapSpec {
    type Dims = SpatialTendonDims;

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        field_spans::<Self>(dims)
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        field_launch_dims::<Self>(dims)
    }
}

#[cfg(feature = "cuda-probe")]
impl FieldBatchSpec for SpatialWrapSpec {
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError> {
        Ok(spatial_wrap_lengths(dims)?.to_vec())
    }

    fn work_dims(dims: &Self::Dims) -> [u32; 4] {
        dims.abi()
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) struct SpatialTendonMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SpatialTendonMetadata {
    type Dims = SpatialTendonDims;

    const KIND: ProofKind = PROOF;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        validate_spatial_tendon_metadata(values, dims.static_dims)
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn spatial_tendon_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "resident_spatial_tendon_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        LauncherError::InvalidMetadata(_)
        | LauncherError::MetadataMismatch(_)
        | LauncherError::MissingMetadataProof { .. }
        | LauncherError::DuplicateMetadataProof { .. } => InputError::InvalidTopology {
            field: "resident_spatial_tendon_metadata",
            index: 0,
            reason: "invalid_metadata",
        }
        .into(),
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "resident_spatial_tendon_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "resident_spatial_tendon_abi",
        }
        .into(),
    }
}

/// 空间肌腱元数据缓冲。
/// 空间肌腱三项同步内核包装。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct ResidentSpatialTendonKernel {
    site: SynchronousKernel,
    moment: SynchronousKernel,
    wrap: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
impl ResidentSpatialTendonKernel {
    pub(super) fn compile(session: &TransferSession, source: &str) -> Result<Self, TransferError> {
        Ok(Self {
            site: SynchronousKernel::compile(session, source, "spatial_site")?,
            moment: SynchronousKernel::compile(session, source, "spatial_moment")?,
            wrap: SynchronousKernel::compile(session, source, "spatial_wrap")?,
        })
    }

    /// site段：读取附着结果，写空间肌腱中间量。
    pub(super) fn launch_site(
        &self,
        metadata: &SpatialTendonMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        attached: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = SpatialTendonDims {
            static_dims: metadata.base().static_dims,
            worlds,
        };
        launch_field_batch::<SpatialSiteSpec, SpatialTendonMetadata, f32, f32, f32>(
            &self.site, metadata, &dims, parameters, attached, output,
        )
    }

    /// moment段：读取质心，累加空间力臂。
    pub(super) fn launch_moment(
        &self,
        metadata: &SpatialTendonMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = SpatialTendonDims {
            static_dims: metadata.base().static_dims,
            worlds,
        };
        launch_field_batch::<SpatialMomentSpec, SpatialTendonMetadata, f32, f32, f32>(
            &self.moment,
            metadata,
            &dims,
            parameters,
            com,
            output,
        )
    }

    /// wrap段：读取空间结果，写出包裹索引。
    pub(super) fn launch_wrap(
        &self,
        metadata: &SpatialTendonMetadataBuffer,
        parameters: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<i32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        let dims = SpatialTendonDims {
            static_dims: metadata.base().static_dims,
            worlds,
        };
        launch_field_batch::<SpatialWrapSpec, SpatialTendonMetadata, f32, f32, i32>(
            &self.wrap, metadata, &dims, parameters, state, output,
        )
    }
}

/// 空间肌腱元数据缓冲。
#[cfg(feature = "cuda-probe")]
pub(super) type SpatialTendonMetadataBuffer = FieldMetadataBuffer<SpatialTendonMetadata>;
