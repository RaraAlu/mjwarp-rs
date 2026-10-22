//! 借用质量矩阵元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::BorrowedMassComputeMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::BorrowedMassValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::borrowed_mass_validate_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 每数据级元数据；跳过标志写入后刷新证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct BorrowedMassMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: BorrowedMassStaticDims,
    worlds: u32,
    revision: MetadataRevision,
    checked: Checked<BorrowedMassValidateMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl BorrowedMassMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: BorrowedMassStaticDims,
        worlds: u32,
    ) -> Result<Self, TransferError> {
        let dims = BorrowedMassValidateDims {
            static_dims,
            worlds,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<BorrowedMassValidateMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(borrowed_mass_validate_launch_error)?;
        Ok(Self {
            buffer,
            values,
            static_dims,
            worlds,
            revision,
            checked,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    pub(super) fn static_dims(&self) -> BorrowedMassStaticDims {
        self.static_dims
    }

    fn base_dims(&self) -> BorrowedMassValidateDims {
        BorrowedMassValidateDims {
            static_dims: self.static_dims,
            worlds: self.worlds,
        }
    }

    /// 写入每个世界的跳过标志并刷新证明。
    pub(super) fn write_skip(&mut self, flags: &[i32]) -> Result<(), TransferError> {
        if flags.len() != self.worlds as usize {
            return Err(InputError::LengthMismatch {
                field: "borrowed_mul_m_skip",
                expected: self.worlds as usize,
                actual: flags.len(),
            }
            .into());
        }
        if let Some(index) = flags.iter().position(|&flag| flag != 0 && flag != 1) {
            return Err(InputError::InvalidTopology {
                field: "borrowed_mul_m_skip",
                index,
                reason: "skip_flag_must_be_boolean",
            }
            .into());
        }
        let so = self
            .static_dims
            .so()
            .map_err(borrowed_mass_validate_launch_error)? as usize;
        self.buffer.write_range(so, flags)?;
        self.values[so..so + flags.len()].copy_from_slice(flags);
        self.revision.invalidate();
        // SAFETY: 跳过标志已同步写入且宿主镜像同步更新；维度未变。
        self.checked = unsafe {
            Checked::<BorrowedMassValidateMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.base_dims(),
                self.revision.clone(),
            )
        }
        .map_err(borrowed_mass_validate_launch_error)?;
        Ok(())
    }

    /// 验证证明：先核对证明未失效，再按实际世界数重建。
    pub(super) fn validate_proof(
        &self,
        dims: BorrowedMassValidateDims,
    ) -> Result<Checked<BorrowedMassValidateMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(borrowed_mass_validate_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；维度与实际跳过标志数一致。
        unsafe {
            Checked::<BorrowedMassValidateMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(borrowed_mass_validate_launch_error)
    }

    /// 计算证明：元数据内容与实际维度一致。
    pub(super) fn compute_proof(
        &self,
        dims: BorrowedMassComputeDims,
    ) -> Result<Checked<BorrowedMassComputeMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base_dims())
            .map_err(borrowed_mass_validate_launch_error)?;
        // SAFETY: 同上；计算维度只影响工作量与输出跨度。
        unsafe {
            Checked::<BorrowedMassComputeMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(borrowed_mass_validate_launch_error)
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
}
