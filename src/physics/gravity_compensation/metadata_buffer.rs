//! 重力补偿元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityClearDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravitySumDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::GravityClearMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::GravityComputeMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::GravitySumMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::GravityValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::gravity_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 计划级元数据；只在构造时上传，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct GravityMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: GravityStaticDims,
    revision: MetadataRevision,
    checked: Checked<GravityClearMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl GravityMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: GravityStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = GravityClearDims {
            static_dims,
            worlds: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<GravityClearMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(gravity_launch_error)?;
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

    /// 测试用：绕过代际直接写设备元数据，供GPU验证器故障测试。
    #[cfg(test)]
    pub(super) fn write_range_for_test(
        &mut self,
        start: usize,
        values: &[i32],
    ) -> Result<(), TransferError> {
        self.buffer.write_range(start, values)
    }

    pub(super) fn static_dims(&self) -> GravityStaticDims {
        self.static_dims
    }

    fn base_dims(&self) -> GravityClearDims {
        GravityClearDims {
            static_dims: self.static_dims,
            worlds: 1,
        }
    }

    /// 清零证明。
    pub(super) fn clear_proof(
        &self,
        dims: GravityClearDims,
    ) -> Result<Checked<GravityClearMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(gravity_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际世界数只影响跨度。
        unsafe {
            Checked::<GravityClearMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(gravity_launch_error)
    }

    /// 验证证明。
    pub(super) fn validate_proof(
        &self,
        dims: GravityValidateDims,
    ) -> Result<Checked<GravityValidateMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(gravity_launch_error)?;
        // SAFETY: 同上；验证发射的工作量为世界数。
        unsafe {
            Checked::<GravityValidateMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(gravity_launch_error)
    }

    /// 计算证明。
    pub(super) fn compute_proof(
        &self,
        dims: GravityComputeDims,
    ) -> Result<Checked<GravityComputeMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(gravity_launch_error)?;
        // SAFETY: 同上；计算发射的工作量为世界乘刚体乘自由度。
        unsafe {
            Checked::<GravityComputeMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(gravity_launch_error)
    }

    /// 合并证明。
    pub(super) fn sum_proof(
        &self,
        dims: GravitySumDims,
    ) -> Result<Checked<GravitySumMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(gravity_launch_error)?;
        // SAFETY: 同上；合并发射的工作量为世界乘自由度。
        unsafe {
            Checked::<GravitySumMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(gravity_launch_error)
    }
}
