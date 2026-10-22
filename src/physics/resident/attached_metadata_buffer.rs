//! 常驻附着运动学元数据缓冲与证明。

#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::ResidentAttachedDims;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::ResidentAttachedStaticDims;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_meta::ResidentAttachedMetadata;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_meta::resident_attached_launch_error;
#[cfg(feature = "cuda-probe")]
use super::pack::PackedModel;
#[cfg(feature = "cuda-probe")]
use super::*;

/// 计划级元数据；只在构造时上传，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct ResidentAttachedMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    static_dims: ResidentAttachedStaticDims,
    revision: MetadataRevision,
    checked: Checked<ResidentAttachedMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl ResidentAttachedMetadataBuffer {
    /// 直接消费打包模型：上传元数据并返回参数缓冲。
    pub(super) fn from_packed(
        session: &TransferSession,
        packed: PackedModel,
        nb: usize,
        nj: usize,
        ng: usize,
        ns: usize,
    ) -> Result<(Self, TransferBuffer<f32>), TransferError> {
        let static_dims = ResidentAttachedStaticDims::new(nb, nj, ng, ns, packed.parameters.len())
            .map_err(resident_attached_launch_error)?;
        let buffer = Self::upload(session, packed.metadata, static_dims)?;
        Ok((buffer, session.upload(&packed.parameters)?))
    }

    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        static_dims: ResidentAttachedStaticDims,
    ) -> Result<Self, TransferError> {
        let dims = ResidentAttachedDims {
            static_dims,
            worlds: 1,
        };
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<ResidentAttachedMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                dims,
                revision.clone(),
            )
        }
        .map_err(resident_attached_launch_error)?;
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

    pub(super) fn static_dims(&self) -> ResidentAttachedStaticDims {
        self.static_dims
    }

    /// 发射证明：先核对证明未失效，再按实际世界数重建。
    pub(super) fn proof(
        &self,
        dims: ResidentAttachedDims,
    ) -> Result<Checked<ResidentAttachedMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(
                span,
                &ResidentAttachedDims {
                    static_dims: self.static_dims,
                    worlds: 1,
                },
            )
            .map_err(resident_attached_launch_error)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际世界数只影响跨度。
        unsafe {
            Checked::<ResidentAttachedMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(resident_attached_launch_error)
    }
}
