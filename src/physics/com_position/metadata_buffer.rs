//! 质心位置元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComPositionDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComPositionStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ComPositionMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::com_position_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 每次探针级元数据；上传后绑定证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct ComPositionMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: ComPositionStaticDims,
    revision: MetadataRevision,
    checked: Checked<ComPositionMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl ComPositionMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: ComPositionStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = ComPositionDims {
            static_dims,
            worlds: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<ComPositionMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(com_position_launch_error)?;
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

    pub(super) fn static_dims(&self) -> ComPositionStaticDims {
        self.static_dims
    }

    /// 发射证明：先核对证明未失效，再按实际世界数重建。
    pub(super) fn proof(
        &self,
        dims: ComPositionDims,
    ) -> Result<Checked<ComPositionMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(
                span,
                &ComPositionDims {
                    static_dims: self.static_dims,
                    worlds: 1,
                },
            )
            .map_err(com_position_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际世界数只影响跨度。
        unsafe {
            Checked::<ComPositionMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(com_position_launch_error)
    }
}
