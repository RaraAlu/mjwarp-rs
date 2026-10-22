//! 常驻质量分解元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassFactorDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassSolveDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentMassFactorMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentMassSolveMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::resident_mass_factor_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 计划级块表；只在构造时上传，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentMassMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: ResidentMassStaticDims,
    revision: MetadataRevision,
    checked: Checked<ResidentMassFactorMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMassMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: ResidentMassStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = ResidentMassFactorDims {
            static_dims,
            worlds: 1,
            rhs_count: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<ResidentMassFactorMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(resident_mass_factor_launch_error)?;
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

    pub(super) fn static_dims(&self) -> ResidentMassStaticDims {
        self.static_dims
    }

    fn base_dims(&self) -> ResidentMassFactorDims {
        ResidentMassFactorDims {
            static_dims: self.static_dims,
            worlds: 1,
            rhs_count: 1,
        }
    }

    /// 分解证明：先核对证明未失效，再按实际维度重建。
    pub(super) fn factor_proof(
        &self,
        dims: ResidentMassFactorDims,
    ) -> Result<Checked<ResidentMassFactorMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(resident_mass_factor_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际维度只影响跨度。
        unsafe {
            Checked::<ResidentMassFactorMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(resident_mass_factor_launch_error)
    }

    /// 回代证明：同一块表按回代维度重建。
    pub(super) fn solve_proof(
        &self,
        dims: ResidentMassSolveDims,
    ) -> Result<Checked<ResidentMassSolveMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(resident_mass_factor_launch_error)?;
        // SAFETY: 同上；回代维度只影响右端项与解区跨度。
        unsafe {
            Checked::<ResidentMassSolveMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(resident_mass_factor_launch_error)
    }
}
