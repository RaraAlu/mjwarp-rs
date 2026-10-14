//! 非紧凑块的GPU稠密化输入；不计算或发布Cholesky因子。
//! 只复制f32位型，结构缺项写入正零。
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, SparseMassInput};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

pub struct SparseMassDensePlan {
    input: Arc<SparseMassInput>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
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
            Ok(Self {
                input,
                session: session.clone(),
                metadata: session.upload(&sources)?,
                kernels: [
                    SynchronousKernel::compile(session, CUDA, "sparse_dense_validate")?,
                    SynchronousKernel::compile(session, CUDA, "sparse_dense_gather")?,
                ],
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
            let n = self.input.nc() as u32;
            let p = self.input.qld_block_total() as u32;
            let worlds = data.worlds() as u32;
            // SAFETY: Checked CSR/world strides bound the full guarded source.
            // One thread owns each status slot. Buffers are disjoint, same-plan.
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.status,
                    [n, p, 0, worlds],
                )?;
            }
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
                    self.kernels[1].launch(
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
