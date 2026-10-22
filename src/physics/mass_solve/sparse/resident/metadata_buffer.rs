//! 常驻稀疏质量元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentGatherDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::SparseResidentGatherMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::SparseResidentValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::sparse_resident_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 计划级元数据；只在构造时上传，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct SparseResidentMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: SparseResidentStaticDims,
    revision: MetadataRevision,
    checked: Checked<SparseResidentValidateMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SparseResidentMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: SparseResidentStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = SparseResidentValidateDims {
            static_dims,
            worlds: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<SparseResidentValidateMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(sparse_resident_launch_error)?;
        Ok(Self {
            buffer,
            values,
            static_dims,
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn static_dims(&self) -> SparseResidentStaticDims {
        self.static_dims
    }

    fn base_dims(&self) -> SparseResidentValidateDims {
        SparseResidentValidateDims {
            static_dims: self.static_dims,
            worlds: 1,
        }
    }

    /// 验证证明：先核对证明未失效，再按实际世界数重建。
    pub(super) fn validate_proof(
        &self,
        dims: SparseResidentValidateDims,
    ) -> Result<Checked<SparseResidentValidateMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(sparse_resident_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际世界数只影响跨度。
        unsafe {
            Checked::<SparseResidentValidateMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_resident_launch_error)
    }

    /// 采集证明：同一元数据按采集维度重建。
    pub(super) fn gather_proof(
        &self,
        dims: SparseResidentGatherDims,
    ) -> Result<Checked<SparseResidentGatherMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(sparse_resident_launch_error)?;
        // SAFETY: 同上；采集维度只影响工作量。
        unsafe {
            Checked::<SparseResidentGatherMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(sparse_resident_launch_error)
    }
}
