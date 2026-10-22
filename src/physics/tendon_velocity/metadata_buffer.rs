//! 肌腱速度元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::TendonVelocityComputeMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::TendonVelocityValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::tendon_velocity_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 计划级元数据；只在构造时上传，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: TendonVelocityStaticDims,
    revision: MetadataRevision,
    checked: Checked<TendonVelocityValidateMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl TendonVelocityMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: TendonVelocityStaticDims,
    ) -> Result<Self, TransferError> {
        let validate_dims = TendonVelocityValidateDims {
            static_dims,
            worlds: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<TendonVelocityValidateMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                validate_dims,
                revision.clone(),
            )
        }
        .map_err(tendon_velocity_validate_launch_error)?;
        Ok(Self {
            buffer,
            values,
            static_dims,
            revision,
            checked,
        })
    }

    /// 下游被动力计划共享的元数据宿主镜像。
    pub(in crate::physics) fn passive_metadata(&self) -> &[i32] {
        &self.values
    }
    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn static_dims(&self) -> TendonVelocityStaticDims {
        self.static_dims
    }

    /// 验证证明：先核对证明未失效，再按实际世界数重建。
    pub(super) fn validate_proof(
        &self,
        dims: TendonVelocityValidateDims,
    ) -> Result<Checked<TendonVelocityValidateMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(
                span,
                &TendonVelocityValidateDims {
                    static_dims: self.static_dims,
                    worlds: 1,
                },
            )
            .map_err(tendon_velocity_validate_launch_error)?;
        // SAFETY: 缓冲、宿主镜像、写入代际与静态维度一致；实际世界数只影响跨度。
        unsafe {
            Checked::<TendonVelocityValidateMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(tendon_velocity_validate_launch_error)
    }

    /// 计算证明：元数据内容与静态维度一致。
    pub(super) fn compute_proof(
        &self,
        dims: TendonVelocityComputeDims,
    ) -> Result<Checked<TendonVelocityComputeMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(
                span,
                &TendonVelocityValidateDims {
                    static_dims: self.static_dims,
                    worlds: 1,
                },
            )
            .map_err(tendon_velocity_validate_launch_error)?;
        // SAFETY: 同上；计算维度只影响工作量与输出跨度。
        unsafe {
            Checked::<TendonVelocityComputeMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(tendon_velocity_validate_launch_error)
    }

    /// 测试用：改写设备元数据并让证明失效。
    #[cfg(test)]
    pub(super) fn overwrite_for_test(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    /// 测试用：绕过代际直接写设备值，供GPU验证器故障测试。
    #[cfg(test)]
    pub(super) fn write_range_for_test(
        &mut self,
        start: usize,
        values: &[i32],
    ) -> Result<(), TransferError> {
        self.buffer.write_range(start, values)
    }

    /// 测试用：读取设备元数据。
    #[cfg(test)]
    pub(super) fn read_range_for_test(
        &self,
        start: usize,
        values: &mut [i32],
    ) -> Result<(), TransferError> {
        self.buffer.read_range_into(start, values)
    }

    /// 测试用：元数据元素数。
    #[cfg(test)]
    pub(super) fn len_for_test(&self) -> usize {
        self.buffer.len()
    }

    /// 测试用：写回原镜像并重新绑定证明。
    #[cfg(test)]
    pub(super) fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        let validate_dims = TendonVelocityValidateDims {
            static_dims: self.static_dims,
            worlds: 1,
        };
        // SAFETY: 恢复写入已同步；跨度与镜像重新绑定。
        self.checked = unsafe {
            Checked::<TendonVelocityValidateMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                validate_dims,
                self.revision.clone(),
            )
        }
        .map_err(tendon_velocity_validate_launch_error)?;
        Ok(())
    }
}
