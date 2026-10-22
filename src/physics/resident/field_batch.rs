//! 常驻FIELD_BATCH内核共享元数据缓冲、计划与四缓冲发射。

#[cfg(feature = "cuda-probe")]
use super::pack::PackedModel;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::runtime::TransferElement;

/// 固定槽位FIELD_BATCH内核的精确跨度与工作标量。
#[cfg(feature = "cuda-probe")]
pub(super) trait FieldBatchSpec: KernelSpec {
    /// 逐槽精确元素数；含每个缓冲两端的四个守卫元素。
    fn exact_lengths(dims: &Self::Dims) -> Result<Vec<u64>, LauncherError>;

    /// 冻结ABI的四个工作标量。
    fn work_dims(dims: &Self::Dims) -> [u32; 4];
}

/// 共享最小跨度：逐槽取精确元素数。
#[cfg(feature = "cuda-probe")]
pub(super) fn field_spans<S: FieldBatchSpec>(
    dims: &S::Dims,
) -> Result<Vec<SlotRequirement>, LauncherError> {
    Ok(S::exact_lengths(dims)?
        .into_iter()
        .enumerate()
        .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
        .collect())
}

/// 共享发射网格：每个世界一个线程，固定256线程块。
#[cfg(feature = "cuda-probe")]
pub(super) fn field_launch_dims<S: FieldBatchSpec>(
    dims: &S::Dims,
) -> Result<LaunchDims, LauncherError> {
    S::exact_lengths(dims)?;
    let work = S::work_dims(dims);
    LaunchDims::new((work[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
}

/// 计划：核对证明、维度、精确跨度、会话与别名。
#[cfg(feature = "cuda-probe")]
pub(super) fn field_plan<S: FieldBatchSpec>(
    dims: &S::Dims,
    bindings: &[SlotBinding],
    proofs: &[CheckedBinding<'_, S::Dims>],
) -> Result<(), LauncherError> {
    Kernel::<S>::new()?.plan(dims, bindings, proofs)?;
    for (slot, elements) in S::exact_lengths(dims)?.into_iter().enumerate() {
        let binding = bindings
            .iter()
            .find(|binding| binding.slot == slot)
            .ok_or(LauncherError::MissingBinding { slot })?;
        let required = elements
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("field_batch_bytes"))?;
        if binding.span.bytes() != required {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: binding.span.bytes(),
            });
        }
    }
    Ok(())
}

/// 槽位声明的元素类型。
#[cfg(feature = "cuda-probe")]
pub(super) fn slot_elem<S: KernelAbi>(slot: usize) -> ElemType {
    S::SLOTS
        .get(slot)
        .and_then(|desc| desc.kind.elem())
        .expect("device buffer slot")
}

/// 计划级元数据缓冲；构造时上传一次，发射前复核证明。
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) struct FieldMetadataBuffer<M: MetadataProof> {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    base: M::Dims,
    revision: MetadataRevision,
    checked: Checked<M>,
    map: fn(LauncherError) -> TransferError,
}

#[cfg(feature = "cuda-probe")]
impl<M: MetadataProof> FieldMetadataBuffer<M> {
    /// 直接消费打包模型：上传元数据并返回参数缓冲。
    pub(super) fn from_packed(
        session: &TransferSession,
        packed: PackedModel,
        base: M::Dims,
        map: fn(LauncherError) -> TransferError,
    ) -> Result<(Self, TransferBuffer<f32>), TransferError> {
        let buffer = Self::upload(session, packed.metadata, base, map)?;
        Ok((buffer, session.upload(&packed.parameters)?))
    }

    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        base: M::Dims,
        map: fn(LauncherError) -> TransferError,
    ) -> Result<Self, TransferError> {
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传已同步；宿主镜像绑定该实际缓冲，内容由证明校验。
        let checked = unsafe {
            Checked::<M>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                base.clone(),
                revision.clone(),
            )
        }
        .map_err(map)?;
        Ok(Self {
            buffer,
            values,
            base,
            revision,
            checked,
            map,
        })
    }

    pub(super) fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    /// 构造时的冻结维度。
    pub(super) fn base(&self) -> &M::Dims {
        &self.base
    }

    /// 元数据槽绑定；地址类证明由`proof`单独核验。
    pub(super) fn binding(&self) -> Result<SlotBinding, TransferError> {
        Ok(SlotBinding {
            slot: 0,
            span: self.buffer.launcher_span()?,
            elem: ElemType::I32,
        })
    }

    /// 发射证明：先核对证明未失效，再按实际世界数重建。
    pub(super) fn proof(&self, dims: M::Dims) -> Result<Checked<M>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.base)
            .map_err(self.map)?;
        // SAFETY: 缓冲、宿主镜像与写入代际一致；实际世界数只影响跨度。
        unsafe { Checked::<M>::assume_uploaded(span, &self.values, dims, self.revision.clone()) }
            .map_err(self.map)
    }
}

/// 四缓冲FIELD_BATCH发射：先做类型计划，再同步发射。
///
/// 内核必须只写输出槽，且不同步访问其他工作。
#[cfg(feature = "cuda-probe")]
#[allow(clippy::too_many_arguments)]
pub(super) fn launch_field_batch<S, M, P, T, O>(
    kernel: &SynchronousKernel,
    metadata: &FieldMetadataBuffer<M>,
    dims: &S::Dims,
    parameters: &TransferBuffer<P>,
    state: &TransferBuffer<T>,
    output: &mut TransferBuffer<O>,
) -> Result<(), TransferError>
where
    S: FieldBatchSpec<Dims = M::Dims>,
    M: MetadataProof,
    P: TransferElement,
    T: TransferElement,
    O: TransferElement,
{
    let map = metadata.map;
    let proof = metadata.proof(dims.clone())?;
    let bindings = [
        metadata.binding()?,
        SlotBinding {
            slot: 1,
            span: parameters.launcher_span()?,
            elem: slot_elem::<S>(1),
        },
        SlotBinding {
            slot: 2,
            span: state.launcher_span()?,
            elem: slot_elem::<S>(2),
        },
        SlotBinding {
            slot: 3,
            span: output.launcher_span()?,
            elem: slot_elem::<S>(3),
        },
    ];
    field_plan::<S>(dims, &bindings, &[proof.binding(0)]).map_err(map)?;
    // SAFETY: 类型计划已核对证明、精确跨度、会话、别名与256线程网格；
    // 内核只写输出槽且同步完成。
    unsafe {
        kernel.launch(
            metadata.as_buffer(),
            parameters,
            state,
            output,
            S::work_dims(dims),
        )
    }
}
