//! 常驻Sparse、Small与Compact混合分解；Tile仍拒绝。
//! 合并qLD前缀与CSR尾；不计算CPU物理结果。
use super::{SparseMassFactorData, SparseMassFactorPlan, SparseMassSmallData, SparseMassSmallPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, SparseMassInput, SparseMassRoute};
use crate::runtime::TransferSession;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    SlotBinding, SlotDesc, SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;
mod solve;
pub use solve::{
    SparseMassMixedResidentData, SparseMassMixedResidentPlan, SparseMassMixedSolveData,
    SparseMassMixedSolveOutput, SparseMassMixedSolvePlan, SparseMassMixedSolveWorld,
};

#[cfg(any(feature = "cuda-probe", test))]
const MIXED_FACTOR_MERGE_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::Value,
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

#[cfg(any(feature = "cuda-probe", test))]
struct MixedFactorMergeSpec;

#[cfg(any(feature = "cuda-probe", test))]
// SAFETY: 声明逐项匹配冻结混合因子合并ABI。
unsafe impl KernelAbi for MixedFactorMergeSpec {
    const NAME: &'static str = "mixed_factor_merge";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &MIXED_FACTOR_MERGE_SLOTS;
}

#[cfg(any(feature = "cuda-probe", test))]
impl KernelSpec for MixedFactorMergeSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let lengths = mixed_factor_merge_lengths(dims)?;
        Ok(lengths
            .into_iter()
            .enumerate()
            .map(|(slot, min_elements)| SlotRequirement { slot, min_elements })
            .collect())
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        mixed_factor_merge_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn mixed_factor_merge_lengths(dims: &[u32; 4]) -> Result<[u64; 4], LauncherError> {
    let [prefix, nc, nv, work] = *dims;
    let tail = nc
        .checked_add(nv)
        .ok_or(LauncherError::Overflow("mixed_factor_merge_tail"))?;
    let stride = prefix
        .checked_add(tail)
        .filter(|&value| value != 0 && value <= i32::MAX as u32)
        .ok_or(LauncherError::InvalidArray("混合因子合并步长无效"))?;
    if nc == 0 || nv == 0 || work == 0 || work > u32::MAX - 255 || !work.is_multiple_of(stride) {
        return Err(LauncherError::InvalidArray("混合因子合并维度无效"));
    }
    let worlds = u64::from(work / stride);
    let guarded = |elements: u64, field| {
        elements
            .checked_add(8)
            .ok_or(LauncherError::Overflow(field))
    };
    Ok([
        1,
        guarded(
            worlds
                .checked_mul(u64::from(prefix))
                .ok_or(LauncherError::Overflow("mixed_factor_merge_small"))?,
            "mixed_factor_merge_small",
        )?,
        guarded(
            worlds
                .checked_mul(u64::from(tail))
                .ok_or(LauncherError::Overflow("mixed_factor_merge_sparse"))?,
            "mixed_factor_merge_sparse",
        )?,
        guarded(u64::from(work), "mixed_factor_merge_output")?,
    ])
}

#[cfg(any(feature = "cuda-probe", test))]
fn mixed_factor_merge_plan(
    expected: [u32; 3],
    dims: &[u32; 4],
    bindings: &[SlotBinding; 4],
) -> Result<LaunchDims, LauncherError> {
    if dims[..3] != expected {
        return Err(LauncherError::InvalidArray("混合因子合并静态维度不匹配"));
    }
    let lengths = mixed_factor_merge_lengths(dims)?;
    for (slot, elements) in lengths.into_iter().enumerate() {
        let bytes = elements
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("mixed_factor_merge_bytes"))?;
        if bindings[slot].span.bytes() != bytes {
            return Err(LauncherError::InvalidArray("混合因子合并跨度不符"));
        }
    }
    let plan = Kernel::<MixedFactorMergeSpec>::new()?.plan(dims, bindings, &[])?;
    Ok(plan.dims())
}

#[cfg(feature = "cuda-probe")]
fn mixed_factor_merge_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::Overflow(_) => InputError::Overflow {
            field: "mixed_factor_merge_abi",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "mixed_factor_merge_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
struct MixedFactorMergeKernel {
    kernel: SynchronousKernel,
    prefix: u32,
    nc: u32,
    nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl MixedFactorMergeKernel {
    fn compile(session: &TransferSession, input: &SparseMassInput) -> Result<Self, TransferError> {
        let prefix = u32::try_from(input.qld_block_total()).map_err(|_| InputError::Overflow {
            field: "mixed_factor_merge_prefix",
        })?;
        let nc = u32::try_from(input.nc()).map_err(|_| InputError::Overflow {
            field: "mixed_factor_merge_nc",
        })?;
        let nv = u32::try_from(input.nv()).map_err(|_| InputError::Overflow {
            field: "mixed_factor_merge_nv",
        })?;
        let work = prefix
            .checked_add(nc)
            .and_then(|value| value.checked_add(nv))
            .ok_or(InputError::Overflow {
                field: "mixed_factor_merge_stride",
            })?;
        mixed_factor_merge_lengths(&[prefix, nc, nv, work])
            .map_err(mixed_factor_merge_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "mixed_factor_merge")?,
            prefix,
            nc,
            nv,
        })
    }

    fn launch(
        &self,
        metadata: &TransferBuffer<i32>,
        small: &TransferBuffer<f32>,
        sparse: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "mixed_factor_merge_worlds",
        })?;
        let stride = self
            .prefix
            .checked_add(self.nc)
            .and_then(|value| value.checked_add(self.nv))
            .ok_or(InputError::Overflow {
                field: "mixed_factor_merge_stride",
            })?;
        let work = worlds.checked_mul(stride).ok_or(InputError::Overflow {
            field: "mixed_factor_merge_work",
        })?;
        let dims = [self.prefix, self.nc, self.nv, work];
        let lengths = mixed_factor_merge_lengths(&dims).map_err(mixed_factor_merge_launch_error)?;
        if [metadata.len(), small.len(), sparse.len(), output.len()]
            .into_iter()
            .zip(lengths)
            .any(|(actual, expected)| u64::try_from(actual).ok() != Some(expected))
        {
            return Err(InputError::InvalidDimension {
                field: "mixed_factor_merge_abi",
            }
            .into());
        }
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: small.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: sparse.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: output.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        mixed_factor_merge_plan([self.prefix, self.nc, self.nv], &dims, &bindings)
            .map_err(mixed_factor_merge_launch_error)?;
        // SAFETY: 类型化计划核对实际跨度、会话和写入别名。
        unsafe { self.kernel.launch(metadata, small, sparse, output, dims) }
    }
}

pub struct SparseMassMixedFactorPlan {
    sparse: SparseMassFactorPlan,
    small: SparseMassSmallPlan,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    merge: MixedFactorMergeKernel,
}
pub struct SparseMassMixedFactorData {
    sparse: SparseMassFactorData,
    small: SparseMassSmallData,
    layout: BatchLayout,
    gathered: bool,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    matrix: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassMixedFactorOutput {
    input: Arc<SparseMassInput>,
    layout: BatchLayout,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassMixedFactorWorld<'a> {
    /// Small上三角U前缀，下三角保持正零。
    pub dense_upper: &'a [f32],
    /// 原CSR尾；非紧凑Small行仍保留原M。
    pub sparse_factors: &'a [f32],
    /// 冻结稀疏阶段处理全部nv的倒数。
    /// Small使用原M对角，不是U对角。
    pub diagonal_inverse: &'a [f32],
}
impl SparseMassMixedFactorOutput {
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<SparseMassMixedFactorWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        let v = &self.values[4 + r.start..4 + r.end];
        let prefix = self.input.qld_block_total();
        let tail = prefix + self.input.nc();
        Ok(SparseMassMixedFactorWorld {
            dense_upper: &v[..prefix],
            sparse_factors: &v[prefix..tail],
            diagonal_inverse: &v[tail..],
        })
    }
}
fn routes(input: &SparseMassInput) -> Result<(), InputError> {
    for block in input.blocks() {
        if block.route == SparseMassRoute::TileCholesky {
            return Err(InputError::InvalidTopology {
                field: "mixed_factor_route",
                index: block.start,
                reason: "MathDx_product_precision_admission_missing",
            });
        }
    }
    if !input
        .blocks()
        .iter()
        .any(|b| b.route == SparseMassRoute::SparseLdl)
    {
        return Err(InputError::InvalidTopology {
            field: "mixed_factor_route",
            index: 0,
            reason: "sparse_CSR_tail_required",
        });
    }
    Ok(())
}
fn layout(input: &SparseMassInput, worlds: usize) -> Result<BatchLayout, InputError> {
    let stride = input
        .qld_block_total()
        .checked_add(input.nc())
        .and_then(|v| v.checked_add(input.nv()))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "mixed_factor_stride",
        })?;
    let layout = BatchLayout::new(worlds, stride, 4)?;
    if worlds == 0 || layout.total_elements() > (u32::MAX - 255) as usize {
        return Err(InputError::InvalidDimension {
            field: "mixed_factor_work",
        });
    }
    BatchLayout::new(
        1,
        layout
            .total_elements()
            .checked_add(8)
            .ok_or(InputError::Overflow {
                field: "mixed_factor_guards",
            })?,
        4,
    )?;
    Ok(layout)
}
impl SparseMassMixedFactorPlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        routes(&input)?;
        let small = SparseMassSmallPlan::new(session, input.clone())?;
        let sparse = SparseMassFactorPlan::from_input(session, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (small, sparse);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let metadata = session.upload(&[0i32])?;
            let merge = MixedFactorMergeKernel::compile(session, sparse.input())?;
            Ok(Self {
                sparse,
                small,
                session: session.clone(),
                metadata,
                merge,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.sparse.input()
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassMixedFactorData, TransferError> {
        let layout = layout(self.input(), worlds)?;
        let sparse = self.sparse.create_data(worlds)?;
        let small = self.small.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, sparse, small);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
            values.fill(-131072.0);
            values[4..4 + layout.total_elements()].fill(f32::NAN);
            let mut matrix =
                crate::runtime::host_staging::<f32>(sparse.matrix_layout.total_elements() + 8)?;
            matrix.fill(-131072.0);
            matrix[4..4 + sparse.matrix_layout.total_elements()].fill(0.0);
            Ok(SparseMassMixedFactorData {
                sparse,
                small,
                layout,
                gathered: false,
                ready: false,
                matrix: self.session.upload(&matrix)?,
                output: self.session.upload(&values)?,
            })
        }
    }
    fn check_data(&self, data: &SparseMassMixedFactorData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.sparse.input, &data.sparse.input) {
            return Err(TransferError::ModelMismatch);
        }
        Ok(())
    }
    pub fn update(&self, data: &mut SparseMassMixedFactorData) -> Result<(), TransferError> {
        self.update_gather(data)?;
        self.update_factor(data)
    }
    pub fn update_gather(&self, data: &mut SparseMassMixedFactorData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // Freeze the main CSR once for both producers. Later main-buffer
            // changes cannot mix a new sparse tail with an old Small prefix.
            // Both copies include guards and finish before either factor stage.
            data.sparse
                .matrix
                .copy_range_from(0, &data.matrix, 0, data.matrix.len())?;
            data.small.copy_matrix_from(&data.sparse.matrix)?;
            self.small.update_gather(&mut data.small)?;
            data.gathered = true;
            Ok(())
        }
    }
    pub fn update_factor(&self, data: &mut SparseMassMixedFactorData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready = false;
        if !data.gathered {
            return Err(TransferError::StageNotReady {
                stage: "mixed_factor_gather",
            });
        }
        self.small.update_factor(&mut data.small)?;
        self.sparse.update_factor(&mut data.sparse)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.merge.launch(
                &self.metadata,
                data.small.factor_buffer()?,
                &data.sparse.output,
                &mut data.output,
                data.layout.worlds(),
            )?;
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
impl SparseMassMixedFactorData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    fn invalidate(&mut self) {
        self.ready = false;
        self.gathered = false;
        self.sparse.ready = false;
        self.small.invalidate();
    }
    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.invalidate();
        }
        result
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        super::matrix_values(self.sparse.matrix_layout, values)?;
        self.write(0, values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let range = self.sparse.matrix_layout.world_elements(world)?;
        super::matrix_values(BatchLayout::new(1, self.sparse.input.nc(), 4)?, values)?;
        self.write(range.start, values)
    }
    fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        #[cfg(feature = "cuda-probe")]
        let result = self.matrix.write_range(4 + offset, values);
        #[cfg(not(feature = "cuda-probe"))]
        let result = {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        };
        self.after_write(result)
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut values = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut values)?;
        crate::physics::check_device_values(&values, "mixed_factor_output")?;
        let input = &self.sparse.input;
        let prefix = input.qld_block_total();
        for world in 0..self.worlds() {
            let first = 4 + self.layout.world_elements(world)?.start;
            for block in input
                .blocks()
                .iter()
                .filter(|b| b.route == SparseMassRoute::SmallCholesky)
            {
                let address = input.qld_block_adr()[block.start] as usize;
                for i in 0..block.size {
                    let value = values[first + address + i * block.size + i];
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof: block.start + i,
                            value: f64::from(value),
                        });
                    }
                }
            }
            for dof in 0..input.nv() {
                let value = values[first + prefix + input.nc() + dof];
                if value <= 0.0 {
                    return Err(TransferError::InvalidPivot {
                        world,
                        dof,
                        value: f64::from(value),
                    });
                }
            }
        }
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassMixedFactorOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.small.readback()?;
            self.sparse.readback()?;
            Ok(SparseMassMixedFactorOutput {
                input: self.sparse.input.clone(),
                layout: self.layout,
                values: self.checked_values()?,
            })
        }
    }
}
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void mixed_factor_merge(const int* metadata,const float* small,
 const float* sparse,float* output,unsigned prefix,unsigned nc,unsigned nv,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned stride=prefix+nc+nv,w=tid/stride,i=tid%stride;
 output[4+tid]=i<prefix?small[4+(unsigned long long)w*prefix+i]:
     sparse[4+(unsigned long long)w*(nc+nv)+i-prefix];
}
"#;

#[cfg(test)]
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
    fn mixed_routes_require_sparse_tail_and_keep_mathdx_rejected() {
        routes(&input::mixed_input()).unwrap();
        assert!(routes(&input::sparse(&input::base()["models"][3])).is_err());
        assert!(routes(&SparseMassInput::new(Default::default()).unwrap()).is_err());
        assert_eq!(input::mixed_matrix(0).len(), input::mixed_input().nc());
        assert_eq!(input::analytic()["cases"].as_array().unwrap().len(), 2);
    }
    #[test]
    fn merged_world_layout_keeps_prefix_tail_and_inverse_distinct() {
        let input = Arc::new(input::mixed_input());
        let layout = layout(&input, 2).unwrap();
        assert_eq!(layout.elements_per_world(), 2377);
        assert!(super::layout(&input, 0).is_err());
        assert!(super::layout(&input, usize::MAX).is_err());
        let output = SparseMassMixedFactorOutput {
            input,
            layout,
            values: (0..layout.total_elements() + 8).map(|i| i as f32).collect(),
        };
        let world = output.world(1).unwrap();
        assert_eq!(
            (
                world.dense_upper.len(),
                world.sparse_factors.len(),
                world.diagonal_inverse.len()
            ),
            (90, 2201, 86)
        );
        assert_eq!(world.dense_upper[0], 2381.0);
        assert_eq!(world.sparse_factors[0], 2471.0);
        assert_eq!(world.diagonal_inverse[0], 4672.0);
        assert!(output.world(2).is_err());
    }
    #[test]
    fn mixed_factor_merge_plan_checks_dims_spans_sessions_and_aliases() {
        use crate::runtime::launcher::DeviceSpan;

        let bind = |slot, context, addr, bytes, elem| SlotBinding {
            slot,
            span: DeviceSpan::new(context, addr, bytes).unwrap(),
            elem,
        };
        let dims = [2, 3, 4, 18];
        assert_eq!(mixed_factor_merge_lengths(&dims).unwrap(), [1, 12, 22, 26]);
        let bindings = [
            bind(0, 7, 0x1000, 4, ElemType::I32),
            bind(1, 7, 0x2000, 48, ElemType::F32),
            bind(2, 7, 0x3000, 88, ElemType::F32),
            bind(3, 7, 0x4000, 104, ElemType::F32),
        ];
        let launch = mixed_factor_merge_plan([2, 3, 4], &dims, &bindings).unwrap();
        assert_eq!(launch.grid(), (1, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));
        assert_eq!(launch.shared_bytes(), 0);
        assert!(mixed_factor_merge_plan([1, 3, 4], &dims, &bindings).is_err());

        let addresses = [0x1000, 0x2000, 0x3000, 0x4000];
        let bytes = [4, 48, 88, 104];
        for slot in 0..4 {
            let mut short = bindings;
            short[slot].span = DeviceSpan::new(
                7,
                addresses[slot],
                if bytes[slot] == 4 { 1 } else { bytes[slot] - 4 },
            )
            .unwrap();
            assert!(mixed_factor_merge_plan([2, 3, 4], &dims, &short).is_err());

            let mut long = bindings;
            long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
            assert!(mixed_factor_merge_plan([2, 3, 4], &dims, &long).is_err());
        }

        let mut foreign = bindings;
        foreign[1].span = DeviceSpan::new(8, 0x2000, 48).unwrap();
        assert!(matches!(
            mixed_factor_merge_plan([2, 3, 4], &dims, &foreign),
            Err(LauncherError::ContextMismatch { slot: 1 })
        ));

        let alias_dims = [0, 3, 4, 14];
        assert_eq!(
            mixed_factor_merge_lengths(&alias_dims).unwrap(),
            [1, 8, 22, 22]
        );
        let zero_prefix = [
            bind(0, 7, 0x1000, 4, ElemType::I32),
            bind(1, 7, 0x2000, 32, ElemType::F32),
            bind(2, 7, 0x3000, 88, ElemType::F32),
            bind(3, 7, 0x4000, 88, ElemType::F32),
        ];
        mixed_factor_merge_plan([0, 3, 4], &alias_dims, &zero_prefix).unwrap();
        let mut alias = zero_prefix;
        alias[3].span = alias[2].span;
        assert!(matches!(
            mixed_factor_merge_plan([0, 3, 4], &alias_dims, &alias),
            Err(LauncherError::Alias {
                first: 2,
                second: 3
            })
        ));
        assert!(mixed_factor_merge_lengths(&[2, 3, 4, 0]).is_err());
        assert!(mixed_factor_merge_lengths(&[2, 3, 4, 17]).is_err());
        assert!(mixed_factor_merge_lengths(&[2, 3, 4, u32::MAX - 254]).is_err());
        assert!(mixed_factor_merge_lengths(&[0, u32::MAX, 1, 1]).is_err());
        assert!(mixed_factor_merge_lengths(&[i32::MAX as u32, 1, 1, i32::MAX as u32 + 2]).is_err());
        assert!(mixed_factor_merge_lengths(&[u32::MAX, 3, 4, 1]).is_err());
    }

    #[cfg(feature = "cuda-probe")]
    fn assert_merge_matches_producers(data: &SparseMassMixedFactorData) {
        let prefix = data.sparse.input.qld_block_total();
        let tail = data.sparse.input.nc() + data.sparse.input.nv();
        let mut small = vec![0.0; data.small.factor_buffer().unwrap().len()];
        let mut sparse = vec![0.0; data.sparse.output.len()];
        let mut merged = vec![0.0; data.output.len()];
        data.small
            .factor_buffer()
            .unwrap()
            .read_range_into(0, &mut small)
            .unwrap();
        data.sparse.output.read_range_into(0, &mut sparse).unwrap();
        data.output.read_range_into(0, &mut merged).unwrap();
        for guard in merged[..4].iter().chain(&merged[merged.len() - 4..]) {
            assert_eq!(guard.to_bits(), (-131072.0f32).to_bits());
        }
        for world in 0..data.worlds() {
            let merged_start = 4 + world * (prefix + tail);
            let small_start = 4 + world * prefix;
            let sparse_start = 4 + world * tail;
            for (actual, expected) in merged[merged_start..merged_start + prefix]
                .iter()
                .zip(&small[small_start..small_start + prefix])
            {
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
            for (actual, expected) in merged[merged_start + prefix..merged_start + prefix + tail]
                .iter()
                .zip(&sparse[sparse_start..sparse_start + tail])
            {
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
        }
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new mixed main-CSR/merged-output guarded cache failure purpose"]
    fn mixed_device_guards_and_cached_stage_failures_reject_and_recover() {
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassMixedFactorPlan::new(&session, input::mixed_input()).unwrap();
        let mut data = plan.create_data(2).unwrap();
        data.write_matrix(&input::mixed_matrix(0).repeat(2))
            .unwrap();
        plan.update(&mut data).unwrap();
        assert_merge_matches_producers(&data);
        let foreign = TransferSession::new(0).unwrap();
        let foreign_small = foreign
            .upload(&vec![0.0f32; data.small.factor_buffer().unwrap().len()])
            .unwrap();
        let worlds = data.worlds();
        assert!(matches!(
            plan.merge.launch(
                &plan.metadata,
                &foreign_small,
                &data.sparse.output,
                &mut data.output,
                worlds,
            ),
            Err(TransferError::SessionMismatch)
        ));
        assert_merge_matches_producers(&data);
        // Poisoning old main M cannot affect either captured producer. A fresh
        // gather must detect it, instead of publishing a cross-generation mix.
        let before = data.readback().unwrap();
        let mut saved = [0.0];
        data.matrix.read_range_into(4, &mut saved).unwrap();
        data.matrix.write_range(4, &[f32::NAN]).unwrap();
        plan.update_factor(&mut data).unwrap();
        let cached = data.readback().unwrap();
        for w in 0..2 {
            assert_eq!(
                cached.world(w).unwrap().dense_upper,
                before.world(w).unwrap().dense_upper
            );
            for (&a, &b) in cached
                .world(w)
                .unwrap()
                .sparse_factors
                .iter()
                .zip(before.world(w).unwrap().sparse_factors)
            {
                assert!((f64::from(a) - f64::from(b)).abs() <= 2e-4 + 2e-4 * f64::from(b).abs());
            }
        }
        assert!(plan.update_gather(&mut data).is_err());
        assert!(!data.ready);
        assert!(data.readback().is_err());
        data.matrix.write_range(4, &saved).unwrap();
        plan.update(&mut data).unwrap();
        assert_merge_matches_producers(&data);
        let end = 4 + data.sparse.matrix_layout.total_elements();
        for offset in [0, end, 4] {
            let mut saved = [0.0];
            data.matrix.read_range_into(offset, &mut saved).unwrap();
            data.matrix.write_range(offset, &[f32::NAN]).unwrap();
            assert!(plan.update_gather(&mut data).is_err());
            assert!(plan.update_factor(&mut data).is_err());
            assert!(!data.ready);
            assert!(data.readback().is_err());
            data.matrix.write_range(offset, &saved).unwrap();
            plan.update(&mut data).unwrap();
            assert_merge_matches_producers(&data);
        }
        let end = 4 + data.layout.total_elements();
        for offset in [0, end] {
            data.output.write_range(offset, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_factor(&mut data).is_err());
            assert!(data.readback().is_err());
            data.output.write_range(offset, &[-131072.0]).unwrap();
            plan.update_factor(&mut data).unwrap();
            assert_merge_matches_producers(&data);
        }
        for (offset, value) in [(4, f32::NAN), (4, -1.0), (4 + 90 + 2201, -1.0)] {
            data.output.write_range(offset, &[value]).unwrap();
            assert!(data.readback().is_err());
            plan.update_factor(&mut data).unwrap();
            assert!(data.readback().is_ok());
        }
        // Cached-factor publication must still validate both actual producer
        // buffers; it cannot trust a good-looking merged snapshot alone.
        data.sparse.output.write_range(0, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        data.sparse.output.write_range(0, &[-131072.0]).unwrap();
        assert!(data.readback().is_ok());
        for offset in [0, 4 + data.small.factor_buffer().unwrap().len() - 8] {
            data.small
                .test_factor_buffer_mut()
                .write_range(offset, &[0.0])
                .unwrap();
            assert!(data.readback().is_err());
            data.small
                .test_factor_buffer_mut()
                .write_range(offset, &[-131072.0])
                .unwrap();
            assert!(data.readback().is_ok());
        }
        // Poison captured Sparse M, not the primary old source. Publication
        // must reject this producer's corrupted snapshot independently.
        data.sparse.matrix.write_range(4, &[f32::NAN]).unwrap();
        assert!(plan.update_factor(&mut data).is_err());
        assert!(!data.ready);
        assert!(data.readback().is_err());
        plan.update(&mut data).unwrap();
        assert_merge_matches_producers(&data);
        let mut raw = vec![0.0; data.matrix.len()];
        data.matrix.read_range_into(0, &mut raw).unwrap();
        data.write_world_matrix(1, &input::mixed_matrix(1)).unwrap();
        let mut next = vec![0.0; raw.len()];
        data.matrix.read_range_into(0, &mut next).unwrap();
        for i in (0..4 + 2201).chain(4 + 2 * 2201..raw.len()) {
            assert_eq!(raw[i].to_bits(), next[i].to_bits());
        }
        plan.update(&mut data).unwrap();
        assert_merge_matches_producers(&data);
        assert!(data.readback().is_ok());
    }
}
