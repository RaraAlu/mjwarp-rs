//! 非紧凑Small前缀U；不生成Sparse/Compact因子或Tile替代。
use super::{SparseMassDenseData, SparseMassDensePlan};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, SparseMassInput, SparseMassRoute};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc, SlotKind, SlotRequirement,
    SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
const SMALL_FACTOR_METADATA_KIND: ProofKind = ProofKind("small_factor_metadata");

#[cfg(feature = "cuda-probe")]
struct StaticSmallFactorMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for StaticSmallFactorMetadata {
    type Dims = u32;

    const KIND: ProofKind = SMALL_FACTOR_METADATA_KIND;

    fn validate(values: &[i32], prefix: &u32) -> Result<(), LauncherError> {
        if values == [0] {
            return Ok(());
        }
        if *prefix == 0 || *prefix > i32::MAX as u32 {
            return Err(LauncherError::InvalidMetadata("Small前缀无效"));
        }
        if values.is_empty() || !values.len().is_multiple_of(3) {
            return Err(LauncherError::InvalidMetadata("Small元数据长度无效"));
        }
        let mut ranges = Vec::with_capacity(values.len() / 3);
        for triple in values.as_chunks::<3>().0 {
            let _dof = u32::try_from(triple[0])
                .map_err(|_| LauncherError::InvalidMetadata("Small自由度无效"))?;
            let address = u32::try_from(triple[1])
                .map_err(|_| LauncherError::InvalidMetadata("Small地址无效"))?;
            let size = u32::try_from(triple[2])
                .map_err(|_| LauncherError::InvalidMetadata("Small尺寸无效"))?;
            if !(2..=6).contains(&size) {
                return Err(LauncherError::InvalidMetadata("Small尺寸越界"));
            }
            let end = address
                .checked_add(
                    size.checked_mul(size)
                        .ok_or(LauncherError::Overflow("small_factor_block"))?,
                )
                .filter(|&end| end <= *prefix)
                .ok_or(LauncherError::InvalidMetadata("Small块越界"))?;
            if ranges
                .iter()
                .any(|&(start, stop)| address < stop && start < end)
            {
                return Err(LauncherError::InvalidMetadata("Small块重叠"));
            }
            ranges.push((address, end));
        }
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
struct SmallFactorLaunchMetadata;

#[cfg(feature = "cuda-probe")]
impl MetadataProof for SmallFactorLaunchMetadata {
    type Dims = [u32; 4];

    const KIND: ProofKind = SMALL_FACTOR_METADATA_KIND;

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        small_factor_lengths(dims)?;
        StaticSmallFactorMetadata::validate(values, &dims[0])?;
        let first =
            usize::try_from(dims[1]).map_err(|_| LauncherError::Overflow("small_factor_first"))?;
        let count =
            usize::try_from(dims[2]).map_err(|_| LauncherError::Overflow("small_factor_count"))?;
        let end = first
            .checked_add(count)
            .and_then(|value| value.checked_mul(3))
            .ok_or(LauncherError::Overflow("small_factor_metadata"))?;
        if end > values.len() {
            return Err(LauncherError::InvalidMetadata("Small分组越界"));
        }
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
const SMALL_FACTOR_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: SMALL_FACTOR_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Write,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(feature = "cuda-probe")]
struct SmallFactorSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结Small因子ABI。
unsafe impl KernelAbi for SmallFactorSpec {
    const NAME: &'static str = "small_prefix_factor";
    const BLOCK: BlockRule = BlockRule::Fixed(64);
    const SLOTS: &'static [SlotDesc] = &SMALL_FACTOR_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for SmallFactorSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (metadata, buffers) = small_factor_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: metadata,
            },
            SlotRequirement {
                slot: 1,
                min_elements: buffers,
            },
            SlotRequirement {
                slot: 2,
                min_elements: buffers,
            },
            SlotRequirement {
                slot: 3,
                min_elements: buffers,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        small_factor_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(64), 1, 1), (64, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn small_factor_lengths(dims: &[u32; 4]) -> Result<(u64, u64), LauncherError> {
    let [prefix, first, count, work] = *dims;
    if prefix == 0
        || prefix > i32::MAX as u32
        || count == 0
        || work == 0
        || work > u32::MAX - 63
        || !work.is_multiple_of(count)
    {
        return Err(LauncherError::InvalidArray("Small因子维度无效"));
    }
    let metadata = u64::from(first)
        .checked_add(u64::from(count))
        .and_then(|value| value.checked_mul(3))
        .filter(|&value| value <= i32::MAX as u64)
        .ok_or(LauncherError::Overflow("small_factor_metadata"))?;
    let buffers = u64::from(work / count)
        .checked_mul(u64::from(prefix))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("small_factor_buffers"))?;
    Ok((metadata, buffers))
}

#[cfg(feature = "cuda-probe")]
fn small_factor_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "small_factor_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "small_factor_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
struct SmallFactorMetadataBuffer {
    buffer: TransferBuffer<i32>,
    values: Vec<i32>,
    prefix: u32,
    revision: MetadataRevision,
    checked: Checked<StaticSmallFactorMetadata>,
}

#[cfg(feature = "cuda-probe")]
impl SmallFactorMetadataBuffer {
    fn upload(
        session: &TransferSession,
        values: Vec<i32>,
        prefix: u32,
    ) -> Result<Self, TransferError> {
        StaticSmallFactorMetadata::validate(&values, &prefix).map_err(small_factor_launch_error)?;
        let buffer = session.upload(&values)?;
        let revision = MetadataRevision::new();
        // SAFETY: 上传同步完成且跨度来自该缓冲。
        let checked = unsafe {
            Checked::<StaticSmallFactorMetadata>::assume_uploaded(
                buffer.launcher_span()?,
                &values,
                prefix,
                revision.clone(),
            )
        }
        .map_err(small_factor_launch_error)?;
        Ok(Self {
            buffer,
            values,
            prefix,
            revision,
            checked,
        })
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn prefix(&self) -> u32 {
        self.prefix
    }

    fn as_buffer(&self) -> &TransferBuffer<i32> {
        &self.buffer
    }

    fn group_size(&self, first: u32, count: u32) -> Result<u32, TransferError> {
        let first = usize::try_from(first).map_err(|_| InputError::InvalidDimension {
            field: "small_factor_group",
        })?;
        let count = usize::try_from(count).map_err(|_| InputError::InvalidDimension {
            field: "small_factor_group",
        })?;
        let end = first.checked_add(count).ok_or(InputError::Overflow {
            field: "small_factor_group",
        })?;
        if end > self.values.len() / 3 {
            return Err(InputError::InvalidDimension {
                field: "small_factor_group",
            }
            .into());
        }
        let mut sizes = self
            .values
            .as_chunks::<3>()
            .0
            .iter()
            .skip(first)
            .take(end - first)
            .map(|triple| triple[2]);
        let size = sizes.next().ok_or(InputError::InvalidDimension {
            field: "small_factor_group",
        })?;
        if sizes.any(|value| value != size) {
            return Err(InputError::InvalidTopology {
                field: "small_factor_group",
                index: first,
                reason: "mixed_static_size",
            }
            .into());
        }
        u32::try_from(size).map_err(|_| {
            InputError::InvalidDimension {
                field: "small_factor_group",
            }
            .into()
        })
    }

    fn proof(&self, dims: [u32; 4]) -> Result<Checked<SmallFactorLaunchMetadata>, TransferError> {
        let span = self.buffer.launcher_span()?;
        self.checked
            .binding(0)
            .validate_for(span, &self.prefix)
            .map_err(small_factor_launch_error)?;
        // SAFETY: 独占包装保留上传值和代际。
        unsafe {
            Checked::<SmallFactorLaunchMetadata>::assume_uploaded(
                span,
                &self.values,
                dims,
                self.revision.clone(),
            )
        }
        .map_err(small_factor_launch_error)
    }

    #[cfg(test)]
    fn write_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(index, &[value])
    }

    #[cfg(test)]
    fn restore_for_test(&mut self) -> Result<(), TransferError> {
        self.revision.invalidate();
        self.buffer.write_range(0, &self.values)?;
        // SAFETY: 恢复写入同步完成且跨度未变。
        self.checked = unsafe {
            Checked::<StaticSmallFactorMetadata>::assume_uploaded(
                self.buffer.launcher_span()?,
                &self.values,
                self.prefix,
                self.revision.clone(),
            )
        }
        .map_err(small_factor_launch_error)?;
        Ok(())
    }
}

#[cfg(feature = "cuda-probe")]
struct SmallFactorKernel {
    kernel: SynchronousKernel,
    size: u32,
    prefix: u32,
    first: u32,
    count: u32,
}

#[cfg(feature = "cuda-probe")]
impl SmallFactorKernel {
    fn compile(
        session: &TransferSession,
        metadata: &SmallFactorMetadataBuffer,
        size: u32,
        first: u32,
        count: u32,
    ) -> Result<Self, TransferError> {
        if metadata.group_size(first, count)? != size || !(2..=6).contains(&size) {
            return Err(InputError::InvalidDimension {
                field: "small_factor_static_group",
            }
            .into());
        }
        small_factor_lengths(&[metadata.prefix(), first, count, count])
            .map_err(small_factor_launch_error)?;
        let entry = match size {
            2 => "small_prefix_factor_2",
            3 => "small_prefix_factor_3",
            4 => "small_prefix_factor_4",
            5 => "small_prefix_factor_5",
            _ => "small_prefix_factor_6",
        };
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, entry)?,
            size,
            prefix: metadata.prefix(),
            first,
            count,
        })
    }

    fn launch(
        &self,
        metadata: &SmallFactorMetadataBuffer,
        matrix: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        if metadata.prefix() != self.prefix
            || metadata.group_size(self.first, self.count)? != self.size
        {
            return Err(InputError::InvalidDimension {
                field: "small_factor_static_group",
            }
            .into());
        }
        let work = worlds.checked_mul(self.count).ok_or(InputError::Overflow {
            field: "small_factor_work",
        })?;
        let dims = [self.prefix, self.first, self.count, work];
        let (metadata_len, buffer_len) =
            small_factor_lengths(&dims).map_err(small_factor_launch_error)?;
        if u64::try_from(metadata.len())
            .ok()
            .is_none_or(|len| len < metadata_len)
            || u64::try_from(matrix.len()).ok() != Some(buffer_len)
            || u64::try_from(unused.len()).ok() != Some(buffer_len)
            || u64::try_from(output.len()).ok() != Some(buffer_len)
        {
            return Err(InputError::InvalidDimension {
                field: "small_factor_abi",
            }
            .into());
        }
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: matrix.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<SmallFactorSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(small_factor_launch_error)?;
        // SAFETY: 类型化计划核对全部实际跨度。
        unsafe {
            self.kernel
                .launch_small_factor(metadata.as_buffer(), matrix, unused, output, dims)
        }
    }
}

pub struct SparseMassSmallPlan {
    source: SparseMassDensePlan,
    #[cfg(feature = "cuda-probe")]
    metadata: SmallFactorMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validator: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    groups: Vec<SmallFactorKernel>,
}
pub struct SparseMassSmallData {
    source: SparseMassDenseData,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    factor: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassSmallOutput {
    input: Arc<SparseMassInput>,
    layout: BatchLayout,
    values: Vec<f32>,
}
impl SparseMassSmallOutput {
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    /// 仅含Small上三角U；低三角保持正零。
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
fn check_routes(input: &SparseMassInput) -> Result<(), InputError> {
    for block in input.blocks() {
        if block.route == SparseMassRoute::TileCholesky {
            return Err(InputError::InvalidTopology {
                field: "small_prefix_route",
                index: block.start,
                reason: "MathDx_tile_factor_not_implemented",
            });
        }
    }
    Ok(())
}
impl SparseMassSmallPlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        check_routes(&input)?;
        let source = SparseMassDensePlan::new(session, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = source;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut meta = Vec::new();
            let mut group_specs = Vec::new();
            for size in 2..=6 {
                let first = meta.len() / 3;
                for block in source
                    .input
                    .blocks()
                    .iter()
                    .filter(|b| b.route == SparseMassRoute::SmallCholesky && b.size == size)
                {
                    meta.extend([
                        block.start as i32,
                        source.input.qld_block_adr()[block.start],
                        size as i32,
                    ]);
                }
                let count = meta.len() / 3 - first;
                if count != 0 {
                    group_specs.push((size as u32, first as u32, count as u32));
                }
            }
            if meta.is_empty() {
                meta.push(0);
            }
            let prefix = u32::try_from(source.input.qld_block_total()).map_err(|_| {
                InputError::Overflow {
                    field: "small_factor_prefix",
                }
            })?;
            let metadata = SmallFactorMetadataBuffer::upload(session, meta, prefix)?;
            let groups = group_specs
                .into_iter()
                .map(|(size, first, count)| {
                    SmallFactorKernel::compile(session, &metadata, size, first, count)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Self {
                source,
                metadata,
                groups,
                validator: SynchronousKernel::compile(session, CUDA, "small_prefix_validate")?,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.source.input()
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassSmallData, TransferError> {
        let source = self.source.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = source;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let len = source.dense_layout.total_elements();
            let mut values = crate::runtime::host_staging::<f32>(len + 8)?;
            values.fill(-131072.0);
            values[4..4 + len].fill(f32::NAN);
            Ok(SparseMassSmallData {
                source,
                ready: false,
                factor: self.source.session.upload(&values)?,
            })
        }
    }
    fn check_data(&self, data: &SparseMassSmallData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.source.input, &data.source.input) {
            return Err(TransferError::ModelMismatch);
        }
        Ok(())
    }
    pub fn update(&self, data: &mut SparseMassSmallData) -> Result<(), TransferError> {
        self.update_gather(data)?;
        self.update_factor(data)
    }
    pub fn update_gather(&self, data: &mut SparseMassSmallData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready = false;
        self.source.update(&mut data.source)
    }
    pub fn update_factor(&self, data: &mut SparseMassSmallData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready = false;
        if !data.source.ready {
            return Err(TransferError::StageNotReady {
                stage: "small_prefix_gather",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let prefix = self.input().qld_block_total() as u32;
            let worlds = data.worlds() as u32;
            // SAFETY: Same-plan dense input has worlds*prefix payload and guards.
            // Validation owns one status slot per world; all inputs are immutable.
            unsafe {
                self.validator.launch(
                    self.metadata.as_buffer(),
                    &data.source.dense,
                    &data.source.dense,
                    &mut data.source.status,
                    [prefix, 0, 0, worlds],
                )?;
            }
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.source.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "small_prefix_status")?;
            for (index, &actual) in status[4..4 + data.worlds()].iter().enumerate() {
                if actual != 1.0 {
                    data.source.ready = false;
                    return Err(ProbeError::Mismatch {
                        index,
                        expected: 1.0,
                        actual,
                    }
                    .into());
                }
            }
            for kernel in &self.groups {
                kernel.launch(
                    &self.metadata,
                    &data.source.dense,
                    &data.source.dense,
                    &mut data.factor,
                    worlds,
                )?;
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
impl SparseMassSmallData {
    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics::mass_solve::sparse) fn test_factor_buffer_mut(
        &mut self,
    ) -> &mut TransferBuffer<f32> {
        &mut self.factor
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics::mass_solve::sparse) fn copy_matrix_from(
        &mut self,
        source: &TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        self.invalidate();
        self.source
            .matrix
            .copy_range_from(0, source, 0, source.len())
    }
    pub(in crate::physics::mass_solve::sparse) fn invalidate(&mut self) {
        self.ready = false;
        self.source.ready = false;
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics::mass_solve::sparse) fn factor_buffer(
        &self,
    ) -> Result<&TransferBuffer<f32>, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_small_factor",
            });
        }
        Ok(&self.factor)
    }
    pub fn worlds(&self) -> usize {
        self.source.worlds()
    }
    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.source.write_matrix(values);
        self.after_write(result)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let result = self.source.write_world_matrix(world, values);
        self.after_write(result)
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let layout = self.source.dense_layout;
        let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
        self.factor.read_range_into(0, &mut values)?;
        for world in 0..self.worlds() {
            let first = 4 + layout.world_elements(world)?.start;
            for block in self
                .source
                .input
                .blocks()
                .iter()
                .filter(|b| b.route == SparseMassRoute::SmallCholesky)
            {
                let address = self.source.input.qld_block_adr()[block.start] as usize;
                for i in 0..block.size {
                    let value = values[first + address + i * block.size + i];
                    if !value.is_finite() || value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof: block.start + i,
                            value: f64::from(value),
                        });
                    }
                }
            }
        }
        crate::physics::check_device_values(&values, "small_prefix_U")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassSmallOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "small_prefix_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(SparseMassSmallOutput {
                input: self.source.input.clone(),
                layout: self.source.dense_layout,
                values: self.checked_values()?,
            })
        }
    }
}

// Frozen smooth.py:1206-1247, actual Small2 PTX: sqrt.rn/rcp.rn/mul/sub.
// Copyright 2025 The Newton Developers; licenses/Apache-2.0.txt.
// Keep mul/sub permission to contract; do not force early FMA or explicit .rn.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
__device__ float small_schur(float value,float a,float b) {
 float result;
 asm("{ .reg .f32 product; mul.f32 product,%2,%3; sub.f32 %0,%1,product; }"
     : "=f"(result) : "f"(value),"f"(a),"f"(b));
 return result;
}
extern "C" __global__ void small_prefix_validate(const int* meta,const float* matrix,
 const float* unused,float* status,unsigned prefix,unsigned a,unsigned b,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 bool valid=true;unsigned long long total=(unsigned long long)prefix*worlds;
 for(unsigned j=0;j<4;++j)valid=valid && matrix[j]==-131072.0f && matrix[4+total+j]==-131072.0f;
 const float* m=matrix+4+(unsigned long long)w*prefix;
 for(unsigned j=0;j<prefix;++j)valid=valid && isfinite(m[j]);
 status[4+w]=valid?1.0f:0.0f;
}
template<int N> __device__ void small_factor(const int* meta,const float* matrix,float* output,
 unsigned prefix,unsigned first,unsigned count,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/count,block=first+tid%count;int address=meta[3*block+1];
 const float* m=matrix+4+(unsigned long long)w*prefix+address;
 float* U=output+4+(unsigned long long)w*prefix+address;
 for(int j=0;j<N*N;++j)U[j]=0.0f;
 for(int i=0;i<N;++i) {
   float pivot=m[i*N+i];
   for(int k=0;k<i;++k)pivot=small_schur(pivot,U[k*N+i],U[k*N+i]);
   if(!(pivot>0.0f) || !isfinite(pivot)){U[i*N+i]=pivot;return;}
   float diagonal,reciprocal;
   asm("sqrt.rn.f32 %0,%1;":"=f"(diagonal):"f"(pivot));
   U[i*N+i]=diagonal;
   asm("rcp.rn.f32 %0,%1;":"=f"(reciprocal):"f"(diagonal));
   for(int j=i+1;j<N;++j) {
     float value=m[i*N+j];
     for(int k=0;k<i;++k)value=small_schur(value,U[k*N+i],U[k*N+j]);
     float result;asm("mul.f32 %0,%1,%2;":"=f"(result):"f"(value),"f"(reciprocal));
     U[i*N+j]=result;
   }
 }
}
#define SMALL_ENTRY(N) extern "C" __global__ void small_prefix_factor_##N(const int* meta,const float* matrix,\
 const float* unused,float* output,unsigned prefix,unsigned first,unsigned count,unsigned work) {\
 small_factor<N>(meta,matrix,output,prefix,first,count,work);}
SMALL_ENTRY(2)
SMALL_ENTRY(3)
SMALL_ENTRY(4)
SMALL_ENTRY(5)
SMALL_ENTRY(6)
"#;

#[cfg(all(test, feature = "cuda-probe"))]
mod tests {
    use super::*;
    use crate as engine;
    mod input {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/small_mass_prefix.rs"
        ));
    }
    #[test]
    fn host_metadata_allows_empty_small_set_and_rejects_bad_ranges() {
        assert!(StaticSmallFactorMetadata::validate(&[0], &0).is_ok());
        assert!(StaticSmallFactorMetadata::validate(&[0, 0, 2], &0).is_err());
        assert!(StaticSmallFactorMetadata::validate(&[0, 2, 2], &4).is_err());
        assert!(StaticSmallFactorMetadata::validate(&[0, 0, 2, 2, 2, 2], &8).is_err());
    }

    #[test]
    fn typed_small_factor_plan_rejects_capacity_alias_session_and_stale_proof() {
        use crate::runtime::launcher::DeviceSpan;

        let dims = [4, 0, 1, 257];
        let (metadata_len, buffer_len) = small_factor_lengths(&dims).unwrap();
        let metadata_span = DeviceSpan::new(7, 0x1000, metadata_len * 4).unwrap();
        let revision = MetadataRevision::new();
        // SAFETY: 本测试绑定等价合成跨度。
        let metadata = unsafe {
            Checked::<SmallFactorLaunchMetadata>::assume_uploaded(
                metadata_span,
                &[0, 0, 2],
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata_span,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(7, 0x10000, buffer_len * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(7, 0x30000, buffer_len * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(7, 0x50000, buffer_len * 4).unwrap(),
                elem: ElemType::F32,
            },
        ];
        let kernel = Kernel::<SmallFactorSpec>::new().unwrap();
        let plan = kernel
            .plan(&dims, &bindings, &[metadata.binding(0)])
            .unwrap();
        assert_eq!(plan.dims().grid(), (5, 1, 1));
        assert_eq!(plan.dims().block(), (64, 1, 1));

        let mut capacity = bindings;
        capacity[3].span = DeviceSpan::new(7, 0x50000, buffer_len * 4 - 4).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &capacity, &[metadata.binding(0)]),
            Err(LauncherError::Capacity { slot: 3, .. })
        ));

        let mut alias = bindings;
        alias[3].span = alias[1].span;
        assert!(matches!(
            kernel.plan(&dims, &alias, &[metadata.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3,
            })
        ));

        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x30000, buffer_len * 4).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &foreign, &[metadata.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));

        revision.invalidate();
        assert!(matches!(
            kernel.plan(&dims, &bindings, &[metadata.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));
    }

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn gpu_small_factor_wrapper_binds_real_spans_and_metadata() {
        let session = TransferSession::new(0).unwrap();
        let foreign = TransferSession::new(0).unwrap();
        let mut metadata = SmallFactorMetadataBuffer::upload(&session, vec![0, 0, 2], 4).unwrap();
        let wrong_metadata = SmallFactorMetadataBuffer::upload(&session, vec![0, 0, 2], 5).unwrap();
        let kernel = SmallFactorKernel::compile(&session, &metadata, 2, 0, 1).unwrap();
        let worlds = 257u32;
        let mut matrix_values = vec![-131072.0; worlds as usize * 4 + 8];
        for world in 0..worlds as usize {
            matrix_values[4 + 4 * world..8 + 4 * world].copy_from_slice(&[4.0, 0.0, 0.0, 9.0]);
        }
        let output_values = vec![-131072.0; matrix_values.len()];
        let matrix = session.upload(&matrix_values).unwrap();
        let mut output = session.upload(&output_values).unwrap();

        kernel
            .launch(&metadata, &matrix, &matrix, &mut output, worlds)
            .unwrap();
        let mut result = vec![0.0; output.len()];
        output.read_range_into(0, &mut result).unwrap();
        assert_eq!(&result[..4], &output_values[..4]);
        assert_eq!(
            &result[result.len() - 4..],
            &output_values[output_values.len() - 4..]
        );
        for world in 0..worlds as usize {
            assert_eq!(&result[4 + 4 * world..8 + 4 * world], &[2.0, 0.0, 0.0, 3.0]);
        }

        let short_matrix = session
            .upload(&matrix_values[..matrix_values.len() - 1])
            .unwrap();
        assert!(
            kernel
                .launch(&metadata, &short_matrix, &short_matrix, &mut output, worlds,)
                .is_err()
        );
        let foreign_matrix = foreign.upload(&matrix_values).unwrap();
        assert!(matches!(
            kernel.launch(
                &metadata,
                &foreign_matrix,
                &foreign_matrix,
                &mut output,
                worlds,
            ),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            kernel.launch(&wrong_metadata, &matrix, &matrix, &mut output, worlds,),
            Err(TransferError::Input(InputError::InvalidDimension {
                field: "small_factor_static_group"
            }))
        ));

        metadata.write_for_test(0, -1).unwrap();
        assert!(matches!(
            kernel.launch(&metadata, &matrix, &matrix, &mut output, worlds),
            Err(TransferError::Input(InputError::InvalidTopology {
                field: "small_factor_metadata",
                reason: "stale_proof",
                ..
            }))
        ));
        let mut after = vec![0.0; output.len()];
        output.read_range_into(0, &mut after).unwrap();
        assert_eq!(result, after);
        metadata.restore_for_test().unwrap();
        kernel
            .launch(&metadata, &matrix, &matrix, &mut output, worlds)
            .unwrap();
    }

    #[test]
    fn routes_keep_mathdx_tile_rejected_without_reclassifying_small() {
        assert!(check_routes(&input::mixed_input()).is_ok());
        assert!(check_routes(&input::sparse(&input::base()["models"][3])).is_err());
    }
    #[test]
    #[ignore = "requires NVIDIA/NVRTC; new Small prefix cached-input/factor fault purpose"]
    fn cache_ignores_old_csr_and_guarded_factor_stages_reject_faults() {
        let start = std::time::Instant::now();
        let session = TransferSession::new(0).unwrap();
        let mut plan = SparseMassSmallPlan::new(&session, input::mixed_input()).unwrap();
        let mut data = plan.create_data(2).unwrap();
        let matrix = input::mixed_matrix(0);
        data.write_matrix(&matrix.repeat(2)).unwrap();
        plan.update(&mut data).unwrap();
        let mut stable = vec![0.0; data.factor.len()];
        data.factor.read_range_into(0, &mut stable).unwrap();
        plan.metadata.write_for_test(0, -1).unwrap();
        assert!(matches!(
            plan.update_factor(&mut data),
            Err(TransferError::Input(InputError::InvalidTopology {
                field: "small_factor_metadata",
                reason: "stale_proof",
                ..
            }))
        ));
        let mut rejected = vec![0.0; data.factor.len()];
        data.factor.read_range_into(0, &mut rejected).unwrap();
        assert_eq!(stable, rejected);
        plan.metadata.restore_for_test().unwrap();
        plan.update_factor(&mut data).unwrap();
        let before = data.readback().unwrap();
        data.source.matrix.write_range(4, &[f32::NAN]).unwrap();
        plan.update_factor(&mut data).unwrap();
        let after = data.readback().unwrap();
        for w in 0..2 {
            assert_eq!(
                after
                    .world(w)
                    .unwrap()
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                before
                    .world(w)
                    .unwrap()
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>()
            );
        }
        assert!(plan.update_gather(&mut data).is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.source.matrix.write_range(4, &[matrix[0]]).unwrap();
        plan.update(&mut data).unwrap();
        let tail = 4 + data.source.dense_layout.total_elements();
        for (offset, bad) in [(0, 0.0), (tail, 0.0), (4, f32::NAN)] {
            let mut saved = [0.0];
            data.source
                .dense
                .read_range_into(offset, &mut saved)
                .unwrap();
            data.source.dense.write_range(offset, &[bad]).unwrap();
            assert!(plan.update_factor(&mut data).is_err());
            assert!(!data.source.ready);
            assert!(data.readback().is_err());
            data.source.dense.write_range(offset, &saved).unwrap();
            plan.update(&mut data).unwrap();
        }
        for offset in [0, tail] {
            data.factor.write_range(offset, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_factor(&mut data).is_err());
            assert!(data.readback().is_err());
            data.factor.write_range(offset, &[-131072.0]).unwrap();
            plan.update_factor(&mut data).unwrap();
        }
        for value in [f32::NAN, -1.0] {
            data.factor.write_range(4, &[value]).unwrap();
            assert!(data.readback().is_err());
            plan.update_factor(&mut data).unwrap();
        }
        for offset in [0, data.worlds() + 4] {
            data.source.status.write_range(offset, &[0.0]).unwrap();
            assert!(plan.update_factor(&mut data).is_err());
            assert!(data.readback().is_err());
            data.source
                .status
                .write_range(offset, &[-131072.0])
                .unwrap();
            plan.update_factor(&mut data).unwrap();
        }
        assert!(data.readback().is_ok());
        println!(
            "small_factor_private metadata_faults=1 recovered=true phase_ms={} factor_capacity_bytes={}",
            start.elapsed().as_millis(),
            data.factor.len() * std::mem::size_of::<f32>()
        );
    }
}
