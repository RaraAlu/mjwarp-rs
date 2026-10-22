//! 常驻CRB元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbAccumulateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbInitDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbMatrixDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentCrbAccumulateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentCrbInitMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentCrbMatrixMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::resident_crb_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 计划级拓扑元数据；只在构造时上传，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentCrbMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: ResidentCrbStaticDims,
    revision: MetadataRevision,
    checked: Checked<ResidentCrbInitMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl ResidentCrbMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: ResidentCrbStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = ResidentCrbInitDims {
            static_dims,
            worlds: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<ResidentCrbInitMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(resident_crb_launch_error)?;
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

    pub(super) fn static_dims(&self) -> ResidentCrbStaticDims {
        self.static_dims
    }

    fn base_dims(&self) -> ResidentCrbInitDims {
        ResidentCrbInitDims {
            static_dims: self.static_dims,
            worlds: 1,
        }
    }

    /// 初始化证明：先核对证明未失效，再按实际世界数重建。
    pub(super) fn init_proof(
        &self,
        dims: ResidentCrbInitDims,
    ) -> Result<Checked<ResidentCrbInitMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(resident_crb_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际世界数只影响跨度。
        unsafe {
            Checked::<ResidentCrbInitMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(resident_crb_launch_error)
    }

    /// 累加证明：同一拓扑表按层级与工作量重建。
    pub(super) fn accumulate_proof(
        &self,
        dims: ResidentCrbAccumulateDims,
    ) -> Result<Checked<ResidentCrbAccumulateMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(resident_crb_launch_error)?;
        // SAFETY: 同上；层级与工作量只影响发射网格。
        unsafe {
            Checked::<ResidentCrbAccumulateMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(resident_crb_launch_error)
    }

    /// 矩阵证明：同一拓扑表按世界与自由度重建。
    pub(super) fn matrix_proof(
        &self,
        dims: ResidentCrbMatrixDims,
    ) -> Result<Checked<ResidentCrbMatrixMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(resident_crb_launch_error)?;
        // SAFETY: 同上；矩阵工作量只影响发射网格。
        unsafe {
            Checked::<ResidentCrbMatrixMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(resident_crb_launch_error)
    }
}
