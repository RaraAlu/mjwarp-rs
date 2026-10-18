//! W18：受检元数据证明。

use super::{DeviceSpan, LauncherError, ProofKind};
use std::{
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

/// 一类元数据的宿主校验规则。
pub trait MetadataProof {
    type Dims: Clone + Eq;

    const KIND: ProofKind;

    /// 校验索引、范围与维度关系。
    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError>;
}

/// 元数据写入代际。
///
/// 每次写入前必须调用`invalidate`。
#[derive(Clone, Debug, Default)]
pub struct MetadataRevision {
    generation: Arc<AtomicU64>,
}

impl MetadataRevision {
    pub fn new() -> Self {
        Self::default()
    }

    /// 使既有证明全部失效。
    pub fn invalidate(&self) {
        let _ = self
            .generation
            .try_update(Ordering::AcqRel, Ordering::Acquire, |generation| {
                generation.checked_add(1)
            });
    }

    fn current(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
}

/// 与设备区间绑定的元数据证明。
pub struct Checked<M: MetadataProof> {
    span: DeviceSpan,
    dims: M::Dims,
    digest: u64,
    revision: MetadataRevision,
    generation: u64,
    marker: PhantomData<M>,
}

impl<M: MetadataProof> Checked<M> {
    /// 校验宿主副本并绑定设备区间。
    ///
    /// # Safety
    /// `span`内容须与`values`逐字节一致。
    /// 后续写入须先使`revision`失效。
    pub unsafe fn assume_uploaded(
        span: DeviceSpan,
        values: &[i32],
        dims: M::Dims,
        revision: MetadataRevision,
    ) -> Result<Self, LauncherError> {
        let bytes = u64::try_from(values.len())
            .map_err(|_| LauncherError::Overflow("metadata_elements"))?
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("metadata_bytes"))?;
        if bytes == 0 || bytes > span.bytes() {
            return Err(LauncherError::InvalidMetadata("宿主副本与设备跨度不符"));
        }
        M::validate(values, &dims)?;
        let generation = revision.current();
        if generation == u64::MAX {
            return Err(LauncherError::Overflow("metadata_revision"));
        }
        Ok(Self {
            span,
            dims,
            digest: digest(values),
            revision,
            generation,
            marker: PhantomData,
        })
    }

    pub fn span(&self) -> DeviceSpan {
        self.span
    }

    pub fn dims(&self) -> &M::Dims {
        &self.dims
    }

    pub fn digest(&self) -> u64 {
        self.digest
    }

    pub fn binding(&self, slot: usize) -> CheckedBinding<'_, M::Dims> {
        CheckedBinding {
            slot,
            kind: M::KIND,
            span: self.span,
            dims: &self.dims,
            revision: &self.revision,
            generation: self.generation,
        }
    }
}

/// 一个地址类槽位的证明引用。
#[derive(Clone, Copy)]
pub struct CheckedBinding<'a, D: Eq> {
    pub slot: usize,
    kind: ProofKind,
    span: DeviceSpan,
    dims: &'a D,
    revision: &'a MetadataRevision,
    generation: u64,
}

impl<D: Eq> CheckedBinding<'_, D> {
    pub(crate) fn kind(&self) -> ProofKind {
        self.kind
    }

    pub(crate) fn validate_for(&self, span: DeviceSpan, dims: &D) -> Result<(), LauncherError> {
        if self.revision.current() != self.generation {
            return Err(LauncherError::StaleMetadata);
        }
        if self.span != span {
            return Err(LauncherError::MetadataMismatch("设备区间不同"));
        }
        if self.dims != dims {
            return Err(LauncherError::MetadataMismatch("发射维度不同"));
        }
        Ok(())
    }
}

fn digest(values: &[i32]) -> u64 {
    // 固定FNV-1a仅标识宿主快照。
    let mut hash = 0xcbf29ce484222325u64;
    for value in values {
        for byte in value.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}
