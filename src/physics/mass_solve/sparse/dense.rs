//! 非紧凑块的GPU稠密化输入；不计算或发布Cholesky因子。
//! 只复制f32位型，结构缺项写入正零。
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, SparseMassInput};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    SlotBinding, SlotDesc, SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

mod small;
pub use small::{SparseMassSmallData, SparseMassSmallOutput, SparseMassSmallPlan};

#[cfg(feature = "cuda-probe")]
const DENSE_VALIDATION_SLOTS: [SlotDesc; 8] = [
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

#[cfg(feature = "cuda-probe")]
struct DenseValidationSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结稠密校验ABI。
unsafe impl KernelAbi for DenseValidationSpec {
    const NAME: &'static str = "sparse_dense_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &DENSE_VALIDATION_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for DenseValidationSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (matrix, status) = dense_validation_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 1,
                min_elements: matrix,
            },
            SlotRequirement {
                slot: 2,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        dense_validation_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn dense_validation_lengths(dims: &[u32; 4]) -> Result<(u64, u64), LauncherError> {
    let [nc, _prefix, reserved, worlds] = *dims;
    if reserved != 0 || worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("稠密校验维度无效"));
    }
    let matrix = u64::from(nc)
        .checked_mul(u64::from(worlds))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("dense_validation_matrix"))?;
    let status = u64::from(worlds)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("dense_validation_status"))?;
    Ok((matrix, status))
}

#[cfg(feature = "cuda-probe")]
fn dense_validation_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        _ => InputError::InvalidDimension {
            field: "dense_validation_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
struct DenseValidationKernel {
    kernel: SynchronousKernel,
    nc: u32,
    prefix: u32,
}

#[cfg(feature = "cuda-probe")]
impl DenseValidationKernel {
    fn compile(session: &TransferSession, nc: u32, prefix: u32) -> Result<Self, TransferError> {
        dense_validation_lengths(&[nc, prefix, 0, 1]).map_err(dense_validation_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "sparse_dense_validate")?,
            nc,
            prefix,
        })
    }

    fn launch(
        &self,
        metadata: &TransferBuffer<i32>,
        matrix: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let worlds = u32::try_from(worlds).map_err(|_| InputError::Overflow {
            field: "dense_validation_worlds",
        })?;
        let dims = [self.nc, self.prefix, 0, worlds];
        let (matrix_len, status_len) =
            dense_validation_lengths(&dims).map_err(dense_validation_launch_error)?;
        if u64::try_from(matrix.len()).ok() != Some(matrix_len)
            || u64::try_from(status.len()).ok() != Some(status_len)
        {
            return Err(InputError::InvalidDimension {
                field: "dense_validation_abi",
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
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<DenseValidationSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[]))
            .map_err(dense_validation_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度和固定ABI。
        unsafe { self.kernel.launch(metadata, matrix, unused, status, dims) }
    }
}

pub struct SparseMassDensePlan {
    input: Arc<SparseMassInput>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    validator: DenseValidationKernel,
    #[cfg(feature = "cuda-probe")]
    gather: SynchronousKernel,
}
pub struct SparseMassDenseData {
    input: Arc<SparseMassInput>,
    matrix_layout: BatchLayout,
    dense_layout: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    matrix: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    dense: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassDenseOutput {
    input: Arc<SparseMassInput>,
    layout: BatchLayout,
    values: Vec<f32>,
}
impl SparseMassDenseOutput {
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    /// 对称原矩阵按qLD块地址排列；这些值不是因子。
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
fn layouts(
    input: &SparseMassInput,
    worlds: usize,
) -> Result<(BatchLayout, BatchLayout), InputError> {
    if worlds == 0
        || worlds
            .checked_mul(input.qld_block_total().max(1))
            .is_none_or(|v| v > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "sparse_dense_work_items",
        });
    }
    let matrix = BatchLayout::new(worlds, input.nc(), 4)?;
    let dense = BatchLayout::new(worlds, input.qld_block_total(), 4)?;
    for count in [matrix.total_elements(), dense.total_elements(), worlds] {
        BatchLayout::new(
            1,
            count.checked_add(8).ok_or(InputError::Overflow {
                field: "sparse_dense_guards",
            })?,
            4,
        )?;
    }
    Ok((matrix, dense))
}
impl SparseMassDensePlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        let input = Arc::new(input);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (input, session);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut sources = input.dense_block_sources()?;
            if sources.is_empty() {
                sources.push(0);
            }
            let nc = u32::try_from(input.nc()).map_err(|_| InputError::Overflow {
                field: "dense_validation_nc",
            })?;
            let prefix =
                u32::try_from(input.qld_block_total()).map_err(|_| InputError::Overflow {
                    field: "dense_validation_prefix",
                })?;
            Ok(Self {
                input,
                session: session.clone(),
                metadata: session.upload(&sources)?,
                validator: DenseValidationKernel::compile(session, nc, prefix)?,
                gather: SynchronousKernel::compile(session, CUDA, "sparse_dense_gather")?,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassDenseData, TransferError> {
        let (matrix_layout, dense_layout) = layouts(&self.input, worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix_layout, dense_layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, initial| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(initial);
                self.session.upload(&values)
            };
            Ok(SparseMassDenseData {
                input: self.input.clone(),
                matrix_layout,
                dense_layout,
                ready: false,
                matrix: allocate(matrix_layout.total_elements(), 0.0)?,
                dense: allocate(dense_layout.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut SparseMassDenseData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let n = self.validator.nc;
            let p = self.validator.prefix;
            let worlds = u32::try_from(data.worlds()).map_err(|_| InputError::Overflow {
                field: "sparse_dense_worlds",
            })?;
            self.validator.launch(
                &self.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.status,
                worlds as usize,
            )?;
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "sparse_dense_status")?;
            for (index, &actual) in status[4..4 + data.worlds()].iter().enumerate() {
                if actual != 1.0 {
                    return Err(ProbeError::Mismatch {
                        index,
                        expected: 1.0,
                        actual,
                    }
                    .into());
                }
            }
            if p != 0 {
                // SAFETY: Each index is a checked CSR source slot or the nC
                // sentinel, which writes +0 without reading out of bounds.
                // One thread owns one dense slot; all work finishes synchronously.
                unsafe {
                    self.gather.launch(
                        &self.metadata,
                        &data.matrix,
                        &data.matrix,
                        &mut data.dense,
                        [n, p, 0, worlds * p],
                    )?;
                }
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
impl SparseMassDenseData {
    pub fn worlds(&self) -> usize {
        self.dense_layout.worlds()
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        check_input(values, self.matrix_layout.total_elements())?;
        self.write(0, values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.matrix_layout.world_elements(world)?;
        check_input(values, r.len())?;
        self.write(r.start, values)
    }
    fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.matrix.write_range(4 + offset, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut values =
            crate::runtime::host_staging::<f32>(self.dense_layout.total_elements() + 8)?;
        self.dense.read_range_into(0, &mut values)?;
        crate::physics::check_device_values(&values, "sparse_dense_matrix")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassDenseOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_dense_gather",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(SparseMassDenseOutput {
                input: self.input.clone(),
                layout: self.dense_layout,
                values: self.checked_values()?,
            })
        }
    }
}
fn check_input(values: &[f32], expected: usize) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field: "sparse_dense_matrix",
            expected,
            actual: values.len(),
        });
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: "sparse_dense_matrix",
                index,
            });
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void sparse_dense_validate(const int* sources,const float* matrix,
 const float* unused,float* status,unsigned nc,unsigned prefix,unsigned reserved,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 bool valid=true;unsigned long long total=(unsigned long long)nc*worlds;
 for(unsigned j=0;j<4;++j)valid=valid && matrix[j]==-131072.0f && matrix[4+total+j]==-131072.0f;
 const float* input=matrix+4+(unsigned long long)w*nc;
 for(unsigned j=0;j<nc;++j)valid=valid && isfinite(input[j]);
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void sparse_dense_gather(const int* sources,const float* matrix,
 const float* unused,float* output,unsigned nc,unsigned prefix,unsigned reserved,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/prefix,slot=tid%prefix;int source=sources[slot];
 output[4+tid]=source<(int)nc?matrix[4+(unsigned long long)w*nc+source]:0.0f;
}
"#;

#[cfg(all(test, feature = "cuda-probe"))]
mod tests {
    use super::*;
    use crate as engine;
    mod input {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/sparse_mass_dense.rs"
        ));
    }

    #[test]
    fn typed_dense_validation_plan_tracks_actual_access_spans() {
        use crate::runtime::launcher::DeviceSpan;

        let dims = [3, 0, 0, 257];
        let (matrix_len, status_len) = dense_validation_lengths(&dims).unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: DeviceSpan::new(7, 0x1000, 4).unwrap(),
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(7, 0x2000, matrix_len * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(7, 0x4000, 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(7, 0x6000, status_len * 4).unwrap(),
                elem: ElemType::F32,
            },
        ];
        let kernel = Kernel::<DenseValidationSpec>::new().unwrap();
        let plan = kernel.plan(&dims, &bindings, &[]).unwrap();
        assert_eq!(plan.dims().grid(), (2, 1, 1));
        assert_eq!(plan.dims().block(), (256, 1, 1));

        let mut capacity = bindings;
        capacity[1].span = DeviceSpan::new(7, 0x2000, matrix_len * 4 - 4).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &capacity, &[]),
            Err(LauncherError::Capacity { slot: 1, .. })
        ));

        let mut alias = bindings;
        alias[3].span = alias[1].span;
        assert!(matches!(
            kernel.plan(&dims, &alias, &[]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3,
            })
        ));

        let mut unused_alias = bindings;
        unused_alias[2].span = unused_alias[3].span;
        assert!(matches!(
            kernel.plan(&dims, &unused_alias, &[]),
            Err(LauncherError::Alias {
                first: 2,
                second: 3,
            })
        ));

        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x4000, 4).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &foreign, &[]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));

        assert!(kernel.plan(&[3, 0, 1, 257], &bindings, &[]).is_err());
        assert!(kernel.plan(&[3, 0, 0, 0], &bindings, &[]).is_err());
    }

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn gpu_dense_validation_wrapper_preserves_zero_prefix_and_unused_semantics() {
        let session = TransferSession::new(0).unwrap();
        let foreign = TransferSession::new(0).unwrap();
        let mut metadata = session.upload(&[0i32]).unwrap();
        let unused = session.upload(&[7.0f32]).unwrap();
        let worlds = 257usize;
        let nc = 3usize;
        let mut matrix_values = vec![-131072.0; nc * worlds + 8];
        let matrix_end = matrix_values.len() - 4;
        matrix_values[4..matrix_end].fill(0.25);
        let status_values = vec![-131072.0; worlds + 8];
        let matrix = session.upload(&matrix_values).unwrap();
        let mut status = session.upload(&status_values).unwrap();
        let kernel = DenseValidationKernel::compile(&session, nc as u32, 0).unwrap();

        kernel
            .launch(&metadata, &matrix, &unused, &mut status, worlds)
            .unwrap();
        let mut result = vec![0.0; status.len()];
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(&result[..4], &status_values[..4]);
        assert_eq!(
            &result[result.len() - 4..],
            &status_values[status_values.len() - 4..]
        );
        assert!(result[4..4 + worlds].iter().all(|&value| value == 1.0));
        let stable = result.clone();

        metadata.write_range(0, &[-1]).unwrap();
        kernel
            .launch(&metadata, &matrix, &unused, &mut status, worlds)
            .unwrap();
        let short_matrix = session
            .upload(&matrix_values[..matrix_values.len() - 1])
            .unwrap();
        assert!(
            kernel
                .launch(&metadata, &short_matrix, &unused, &mut status, worlds)
                .is_err()
        );
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);

        let mut short_status = session
            .upload(&status_values[..status_values.len() - 1])
            .unwrap();
        assert!(
            kernel
                .launch(&metadata, &matrix, &unused, &mut short_status, worlds)
                .is_err()
        );
        let mut short_after = vec![0.0; short_status.len()];
        short_status.read_range_into(0, &mut short_after).unwrap();
        assert_eq!(short_after, status_values[..status_values.len() - 1]);

        let wrong_nc_matrix = session
            .upload(&vec![-131072.0f32; (nc + 1) * worlds + 8])
            .unwrap();
        assert!(
            kernel
                .launch(&metadata, &wrong_nc_matrix, &unused, &mut status, worlds,)
                .is_err()
        );
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);

        let foreign_unused = foreign.upload(&[7.0f32]).unwrap();
        assert!(matches!(
            kernel.launch(&metadata, &matrix, &foreign_unused, &mut status, worlds),
            Err(TransferError::SessionMismatch)
        ));
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);
    }

    #[test]
    #[ignore = "requires NVIDIA/NVRTC; dense validation product-call purpose"]
    fn gpu_dense_validation_real_update_rejects_before_gather_and_recovers() {
        let start = std::time::Instant::now();
        let refs = input::reference();
        let model = &refs["models"][3];
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassDensePlan::new(&session, input::native(model)).unwrap();
        let mut data = plan.create_data(2).unwrap();
        let matrix = input::matrix(&model["cases"][0]);
        data.write_matrix(&matrix.repeat(2)).unwrap();
        plan.update(&mut data).unwrap();
        let bits = |values: &[f32]| {
            values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        };
        let mut stable = vec![0.0; data.dense.len()];
        data.dense.read_range_into(0, &mut stable).unwrap();

        data.matrix.write_range(0, &[0.0]).unwrap();
        assert!(plan.update(&mut data).is_err());
        assert!(!data.ready);
        assert!(data.readback().is_err());
        let mut rejected = vec![0.0; data.dense.len()];
        data.dense.read_range_into(0, &mut rejected).unwrap();
        assert_eq!(bits(&stable), bits(&rejected));

        data.matrix.write_range(0, &[-131072.0]).unwrap();
        plan.update(&mut data).unwrap();
        assert!(data.readback().is_ok());
        println!(
            "dense_validation_private worlds=2 recovered=true phase_ms={} status_capacity_bytes={}",
            start.elapsed().as_millis(),
            data.status.len() * std::mem::size_of::<f32>()
        );
    }

    #[test]
    #[ignore = "requires NVIDIA/NVRTC; new dense prefix source/target fault purpose"]
    fn dense_source_guards_unselected_nan_world_writes_and_recovery() {
        let refs = input::reference();
        let m = &refs["models"][3];
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassDensePlan::new(&session, input::native(m)).unwrap();
        let mut data = plan.create_data(2).unwrap();
        let base = input::matrix(&m["cases"][0]);
        let next = input::matrix(&m["cases"][1]);
        data.write_matrix(&base.repeat(2)).unwrap();
        plan.update(&mut data).unwrap();
        let before = data.readback().unwrap();
        for other in [input::fork(7), input::small()] {
            let foreign = SparseMassDensePlan::new(&session, other).unwrap();
            assert!(foreign.update(&mut data).is_err());
            assert_eq!(
                data.readback().unwrap().world(0).unwrap(),
                before.world(0).unwrap()
            );
        }
        assert!(data.write_matrix(&[]).is_err());
        assert!(data.write_world_matrix(2, &base).is_err());
        assert!(data.write_world_matrix(0, &vec![f32::NAN; 2173]).is_err());
        assert!(
            data.write_world_matrix(0, &vec![f32::INFINITY; 2173])
                .is_err()
        );
        assert!(data.readback().is_ok());
        let nc = data.matrix_layout.elements_per_world();
        let tail = 4 + data.matrix_layout.total_elements();
        let mut original = vec![0.0; tail + 4];
        data.matrix.read_range_into(0, &mut original).unwrap();
        data.write_world_matrix(1, &next).unwrap();
        let mut changed = vec![0.0; original.len()];
        data.matrix.read_range_into(0, &mut changed).unwrap();
        for i in (0..4 + nc).chain(tail..tail + 4) {
            assert_eq!(original[i].to_bits(), changed[i].to_bits());
        }
        for (a, b) in changed[4 + nc..tail].iter().zip(&next) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
        plan.update(&mut data).unwrap();
        for (offset, bad) in [(0, 0.0), (tail, 0.0), (4, f32::NAN), (4 + 2145, f32::NAN)] {
            let mut saved = [0.0];
            data.matrix.read_range_into(offset, &mut saved).unwrap();
            data.matrix.write_range(offset, &[bad]).unwrap();
            assert!(plan.update(&mut data).is_err());
            assert!(data.readback().is_err());
            data.matrix.write_range(offset, &saved).unwrap();
            plan.update(&mut data).unwrap();
        }
        for offset in [0, 4 + data.dense_layout.total_elements()] {
            data.dense.write_range(offset, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update(&mut data).is_err());
            assert!(data.readback().is_err());
            data.dense.write_range(offset, &[-131072.0]).unwrap();
            plan.update(&mut data).unwrap();
        }
        data.dense.write_range(4, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        plan.update(&mut data).unwrap();
        for offset in [0, 4 + data.worlds()] {
            data.status.write_range(offset, &[0.0]).unwrap();
            assert!(plan.update(&mut data).is_err());
            assert!(data.readback().is_err());
            data.status.write_range(offset, &[-131072.0]).unwrap();
            plan.update(&mut data).unwrap();
        }
        assert!(data.readback().is_ok());
    }
}
