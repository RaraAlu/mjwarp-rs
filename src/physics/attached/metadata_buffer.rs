//! 附着运动学元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::AttachedDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::AttachedStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::AttachedMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::attached_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 每次探针级元数据；上传后绑定证明。
#[cfg(feature = "cuda-probe")]
pub(super) struct AttachedMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: AttachedStaticDims,
    revision: MetadataRevision,
    checked: Checked<AttachedMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl AttachedMetadataBuffer {
    pub(super) fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: AttachedStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = AttachedDims {
            static_dims,
            worlds: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<AttachedMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(attached_launch_error)?;
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

    pub(super) fn static_dims(&self) -> AttachedStaticDims {
        self.static_dims
    }

    /// 发射证明：先核对证明未失效，再按实际世界数重建。
    pub(super) fn proof(
        &self,
        dims: AttachedDims,
    ) -> Result<Checked<AttachedMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(
                span,
                &AttachedDims {
                    static_dims: self.static_dims,
                    worlds: 1,
                },
            )
            .map_err(attached_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际世界数只影响跨度。
        unsafe {
            Checked::<AttachedMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(attached_launch_error)
    }
}
