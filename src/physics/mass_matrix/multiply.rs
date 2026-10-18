//! 常驻质量矩阵乘法；不执行forward、分解或回代。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, SparseMassInput};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;
mod borrowed;
pub use borrowed::{
    MassMatrixEnergyData, MassMatrixEnergyPlan, MassMatrixEnergySnapshot, MassMatrixEnergyView,
};
pub use borrowed::{MassMatrixMultiplyData, MassMatrixMultiplyPlan, MassMatrixMultiplyView};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MassMultiplyFormat {
    Sparse,
    Dense,
}
pub struct MassMultiplyPlan {
    input: Arc<SparseMassInput>,
    format: MassMultiplyFormat,
    metadata: Vec<i32>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
}
pub struct MassMultiplyData {
    input: Arc<SparseMassInput>,
    format: MassMultiplyFormat,
    matrix_layout: BatchLayout,
    vector_layout: BatchLayout,
    #[cfg(feature = "cuda-probe")]
    skip_offset: usize,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    matrix: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    alternate: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    vector: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct MassMultiplyOutput {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl MassMultiplyOutput {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
// 逐条复刻冻结io.py的追加顺序；不重新排序或累加。
fn metadata(input: &SparseMassInput) -> Result<Vec<i32>, InputError> {
    let n = input.nv();
    let mut rows = vec![Vec::new(); n];
    let f = input.fields();
    for i in 0..n {
        let start = f.m_rowadr[i] as usize;
        for k in start..start + f.m_rownnz[i] as usize {
            let j = f.m_colind[k] as usize;
            rows[i].push((j as i32, k as i32));
            if i != j {
                rows[j].push((i as i32, k as i32));
            }
        }
    }
    let count = rows.iter().try_fold(0usize, |count, row| {
        count.checked_add(row.len()).ok_or(InputError::Overflow {
            field: "mul_m_metadata",
        })
    })?;
    let size = n
        .checked_add(1)
        .and_then(|v| count.checked_mul(2).and_then(|count| v.checked_add(count)))
        .ok_or(InputError::Overflow {
            field: "mul_m_metadata",
        })?;
    if size > i32::MAX as usize {
        return Err(InputError::Overflow {
            field: "mul_m_metadata",
        });
    }
    let mut out = vec![0];
    let mut col = Vec::with_capacity(count);
    let mut addr = Vec::with_capacity(count);
    for row in rows {
        for (j, k) in row {
            col.push(j);
            addr.push(k);
        }
        out.push(col.len() as i32);
    }
    out.extend(col);
    out.extend(addr);
    Ok(out)
}
fn layouts(n: usize, mc: usize, worlds: usize) -> Result<(BatchLayout, BatchLayout), InputError> {
    if n == 0
        || mc > u32::MAX as usize
        || worlds == 0
        || worlds
            .checked_mul(n)
            .is_none_or(|v| v > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "mul_m_work",
        });
    }
    let matrix = BatchLayout::new(worlds, mc, 4)?;
    let vector = BatchLayout::new(worlds, n, 4)?;
    for len in [matrix.total_elements(), vector.total_elements(), worlds] {
        BatchLayout::new(
            1,
            len.checked_add(8).ok_or(InputError::Overflow {
                field: "mul_m_guards",
            })?,
            4,
        )?;
    }
    Ok((matrix, vector))
}
impl MassMultiplyPlan {
    pub fn new(
        session: &TransferSession,
        input: SparseMassInput,
        format: MassMultiplyFormat,
    ) -> Result<Self, TransferError> {
        let meta = metadata(&input)?;
        let mc = match format {
            MassMultiplyFormat::Sparse => input.nc(),
            MassMultiplyFormat::Dense => {
                input
                    .nv()
                    .checked_mul(input.nv())
                    .ok_or(InputError::Overflow {
                        field: "mul_m_dense",
                    })?
            }
        };
        layouts(input.nv(), mc, 1)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, meta);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = format!(
                "#define MASS_NV {}\n#define MASS_DENSE {}\n{}",
                input.nv(),
                usize::from(format == MassMultiplyFormat::Dense),
                CUDA
            );
            Ok(Self {
                input: Arc::new(input),
                format,
                metadata: meta,
                session: session.clone(),
                kernels: [
                    SynchronousKernel::compile(session, &source, "mass_multiply_validate")?,
                    SynchronousKernel::compile(session, &source, "mass_multiply")?,
                ],
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<MassMultiplyData, TransferError> {
        let n = self.input.nv();
        let mc = if self.format == MassMultiplyFormat::Dense {
            n * n
        } else {
            self.input.nc()
        };
        let (matrix_layout, vector_layout) = layouts(n, mc, worlds)?;
        let mut meta = self.metadata.clone();
        if meta
            .len()
            .checked_add(worlds)
            .is_none_or(|v| v > i32::MAX as usize)
        {
            return Err(InputError::Overflow {
                field: "mul_m_skip",
            }
            .into());
        }
        meta.resize(meta.len() + worlds, 0);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix_layout, vector_layout, meta);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |len, initial| {
                let mut v = crate::runtime::host_staging::<f32>(len + 8)?;
                v.fill(-131072.0);
                v[4..4 + len].fill(initial);
                self.session.upload(&v)
            };
            Ok(MassMultiplyData {
                input: self.input.clone(),
                format: self.format,
                matrix_layout,
                vector_layout,
                skip_offset: self.metadata.len(),
                ready: false,
                metadata: self.session.upload(&meta)?,
                matrix: allocate(matrix_layout.total_elements(), 0.0)?,
                alternate: allocate(matrix_layout.total_elements(), f32::NAN)?,
                vector: allocate(vector_layout.total_elements(), 0.0)?,
                output: allocate(vector_layout.total_elements(), 0.0)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut MassMultiplyData) -> Result<(), TransferError> {
        self.update_with(data, None, false)
    }
    /// skip保留已有输出位型；alternate选择独立替代矩阵。
    pub fn update_with(
        &self,
        data: &mut MassMultiplyData,
        skip: Option<&[bool]>,
        alternate: bool,
    ) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) || self.format != data.format {
            return Err(TransferError::ModelMismatch);
        }
        if let Some(skip) = skip
            && skip.len() != data.worlds()
        {
            return Err(InputError::LengthMismatch {
                field: "mul_m_skip",
                expected: data.worlds(),
                actual: skip.len(),
            }
            .into());
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = alternate;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let flags = skip.map_or_else(
                || vec![0; data.worlds()],
                |v| v.iter().map(|&s| i32::from(s)).collect::<Vec<_>>(),
            );
            data.metadata.write_range(data.skip_offset, &flags)?;
            let n = self.input.nv() as u32;
            let mc = data.matrix_layout.elements_per_world() as u32;
            let w = data.worlds() as u32;
            let so = data.skip_offset as u32;
            let matrix = if alternate {
                &data.alternate
            } else {
                &data.matrix
            };
            // SAFETY: Same-plan buffers own checked guarded world strides. The
            // validator only reads inputs and writes one disjoint status/world.
            unsafe {
                self.kernels[0].launch(
                    &data.metadata,
                    matrix,
                    &data.vector,
                    &mut data.status,
                    [n, mc, so, w],
                )?;
            }
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "mul_m_status")?;
            if let Some(index) = status[4..4 + data.worlds()].iter().position(|&v| v != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "mul_m_source",
                    index,
                    reason: "nonfinite_or_guard",
                }
                .into());
            }
            // SAFETY: Metadata mirrors validated CSR addresses, or static dense
            // columns fit n. Each thread owns one output; skip never writes it.
            unsafe {
                self.kernels[1].launch(
                    &data.metadata,
                    matrix,
                    &data.vector,
                    &mut data.output,
                    [n, mc, so, w * n],
                )?;
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
fn checked(values: &[f32], expected: usize, field: &'static str) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite { field, index });
    }
    Ok(())
}
impl MassMultiplyData {
    pub fn worlds(&self) -> usize {
        self.vector_layout.worlds()
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(false, false, 0, self.matrix_layout.total_elements(), values)
    }
    pub fn write_alternate_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(false, true, 0, self.matrix_layout.total_elements(), values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.matrix_layout.world_elements(world)?;
        self.write(false, false, r.start, r.len(), values)
    }
    pub fn write_vector(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(true, false, 0, self.vector_layout.total_elements(), values)
    }
    pub fn write_world_vector(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.vector_layout.world_elements(world)?;
        self.write(true, false, r.start, r.len(), values)
    }
    fn write(
        &mut self,
        vector: bool,
        alternate: bool,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        checked(
            values,
            expected,
            if vector {
                "mul_m_vector"
            } else {
                "mul_m_matrix"
            },
        )?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (alternate, offset);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let buffer = if vector {
                &mut self.vector
            } else if alternate {
                &mut self.alternate
            } else {
                &mut self.matrix
            };
            buffer.write_range(4 + offset, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut v = crate::runtime::host_staging::<f32>(self.vector_layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut v)?;
        crate::physics::check_device_values(&v, "mul_m_result")?;
        Ok(v)
    }
    pub fn readback(&self) -> Result<MassMultiplyOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mass_multiply",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(MassMultiplyOutput {
                layout: self.vector_layout,
                values: self.checked_values()?,
            })
        }
    }
}
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void mass_multiply_validate(const int* meta,const float* M,
 const float* vec,float* status,unsigned n,unsigned mc,unsigned so,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 bool valid=true;for(unsigned j=0;j<4;++j)valid=valid&&M[j]==-131072.0f&&M[4+1ull*worlds*mc+j]==-131072.0f&&vec[j]==-131072.0f&&vec[4+1ull*worlds*n+j]==-131072.0f;
 if(!meta[so+w]){for(unsigned j=0;j<mc;++j)valid=valid&&isfinite(M[4+1ull*w*mc+j]);for(unsigned j=0;j<n;++j)valid=valid&&isfinite(vec[4+1ull*w*n+j]);}
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void mass_multiply(const int* meta,const float* M,
 const float* vec,float* result,unsigned n,unsigned mc,unsigned so,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/n,row=tid%n;if(meta[so+w])return;
 const float* matrix=M+4+1ull*w*mc;const float* v=vec+4+1ull*w*n;float acc=0.0f;
 #if MASS_DENSE
 #pragma unroll
 for(unsigned j=0;j<MASS_NV;++j)acc=acc+matrix[row*MASS_NV+j]*v[j];
 #else
 const int* col=meta+n+1;const int* adr=col+meta[n];
 for(int k=meta[row];k<meta[row+1];++k)acc=acc+matrix[adr[k]]*v[col[k]];
 #endif
 result[4+tid]=acc;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate as engine;
    mod source {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/sparse_mass_model.rs"
        ));
    }
    #[cfg(feature = "cuda-probe")]
    mod mixed {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/mixed_mass_solve.rs"
        ));
    }
    fn sparse() -> SparseMassInput {
        source::sparse(&source::reference()["models"][0])
    }
    #[test]
    fn mirrors_each_csr_address_in_original_append_order() {
        let v = source::reference();
        let i = source::sparse(&v["models"][0]);
        let _ = source::model(&v["models"][0]);
        let _ = source::floats(&v["models"][0]["cases"][0], "qpos");
        let m = metadata(&i).unwrap();
        let n = i.nv();
        let count = m[n] as usize;
        assert_eq!(count, 2 * i.nc() - n);
        for row in 0..n {
            let start = m[row] as usize;
            let end = m[row + 1] as usize;
            let cols = &m[n + 1 + start..n + 1 + end];
            assert_eq!(cols, &(0..n as i32).collect::<Vec<_>>()[..]);
            for k in start..end {
                let col = m[n + 1 + k] as usize;
                let adr = m[n + 1 + count + k] as usize;
                let r = row.max(col);
                assert_eq!(i.fields().m_colind[adr], row.min(col) as i32);
                assert!(adr >= i.fields().m_rowadr[r] as usize);
            }
        }
    }
    #[test]
    fn rejects_empty_overflow_and_invalid_finite_inputs() {
        assert!(layouts(0, 0, 1).is_err());
        assert!(layouts(2, 4, 0).is_err());
        assert!(layouts(390, 4, usize::MAX).is_err());
        if let Some(mc) = 65536usize.checked_mul(65536) {
            assert!(layouts(65536, mc, 1).is_err());
        }
        assert!(checked(&[f32::NAN], 1, "mul_m").is_err());
        assert!(checked(&[], 1, "mul_m").is_err());
        let _ = sparse();
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new mul_m numerical/skip/alternate behavior, no old factor/solve calls"]
    fn gpu_multiplies_dense_and_sparse_with_skip_and_independent_alternate() {
        let v = mixed::reference();
        let session = TransferSession::new(0).unwrap();
        let start = std::time::Instant::now();
        let mut batches = 0usize;
        let mut slots = 0usize;
        let mut max_abs = 0.0f64;
        let mut capacity = 0usize;
        for model in v["models"].as_array().unwrap() {
            let input = mixed::input(model);
            let n = input.nv();
            for format in [MassMultiplyFormat::Sparse, MassMultiplyFormat::Dense] {
                let plan = MassMultiplyPlan::new(&session, input.clone(), format).unwrap();
                for sample in 0..2 {
                    let csr = mixed::matrix(model, sample);
                    let full = full_matrix(&input, &csr);
                    let matrix = if format == MassMultiplyFormat::Sparse {
                        csr
                    } else {
                        full.clone()
                    };
                    let rhs = mixed::rhs(model, sample, 3);
                    let solution = mixed::solution(model, sample, 3);
                    for col in 0..3 {
                        let x = solution[col * n..(col + 1) * n]
                            .iter()
                            .map(|&v| v as f32)
                            .collect::<Vec<_>>();
                        let oracle = (0..n)
                            .map(|i| {
                                (0..n)
                                    .map(|j| f64::from(full[i * n + j]) * f64::from(x[j]))
                                    .sum::<f64>()
                            })
                            .collect::<Vec<_>>();
                        for worlds in [1, 2, 257] {
                            let mut data = plan.create_data(worlds).unwrap();
                            assert!(data.readback().is_err());
                            data.write_matrix(&matrix.repeat(worlds)).unwrap();
                            data.write_vector(&x.repeat(worlds)).unwrap();
                            plan.update(&mut data).unwrap();
                            let out = data.readback().unwrap();
                            for w in 0..worlds {
                                for (i, &a) in out.world(w).unwrap().iter().enumerate() {
                                    close(a, oracle[i]);
                                    close(a, f64::from(rhs[col * n + i]));
                                    max_abs = max_abs.max((f64::from(a) - oracle[i]).abs());
                                    slots += 1;
                                }
                            }
                            assert!(out.world(worlds).is_err());
                            capacity = capacity.max(
                                4 * (2 * data.matrix.len()
                                    + data.vector.len()
                                    + data.output.len()
                                    + data.status.len()
                                    + data.metadata.len()),
                            );
                            batches += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(batches, 72);
        assert_eq!(slots, 1485120);
        for format in [MassMultiplyFormat::Sparse, MassMultiplyFormat::Dense] {
            let plan =
                MassMultiplyPlan::new(&session, mixed::input(&v["models"][0]), format).unwrap();
            let other =
                MassMultiplyPlan::new(&session, mixed::input(&v["models"][0]), format).unwrap();
            let mut data = plan.create_data(257).unwrap();
            let n = 86;
            let csr = mixed::matrix(&v["models"][0], 0);
            let m = if format == MassMultiplyFormat::Sparse {
                csr.clone()
            } else {
                full_matrix(&mixed::input(&v["models"][0]), &csr)
            };
            let x = vec![1.0; n];
            data.write_matrix(&m.repeat(257)).unwrap();
            data.write_vector(&x.repeat(257)).unwrap();
            plan.update(&mut data).unwrap();
            let original = data.readback().unwrap();
            assert!(other.update(&mut data).is_err());
            assert!(data.write_world_vector(257, &x).is_err());
            assert!(data.write_world_vector(0, &x[..n - 1]).is_err());
            assert!(data.write_world_matrix(257, &m).is_err());
            assert!(data.write_alternate_matrix(&[f32::NAN]).is_err());
            assert!(plan.update_with(&mut data, Some(&[true]), false).is_err());
            assert_eq!(
                bits(&data.readback().unwrap().values),
                bits(&original.values)
            );
            let mut skip = vec![true; 257];
            skip[256] = false;
            let mut alternate = m.iter().map(|v| 2.0 * v).collect::<Vec<_>>();
            if format == MassMultiplyFormat::Dense {
                // Cross-tree asymmetric entry exercises the full row-major M API.
                alternate[n - 1] += 0.375;
            }
            data.write_alternate_matrix(&alternate.repeat(257)).unwrap();
            plan.update_with(&mut data, Some(&skip), true).unwrap();
            let alt = data.readback().unwrap();
            for w in 0..256 {
                assert_eq!(
                    bits(alt.world(w).unwrap()),
                    bits(original.world(w).unwrap())
                );
            }
            let full = if format == MassMultiplyFormat::Dense {
                alternate.clone()
            } else {
                full_matrix(&mixed::input(&v["models"][0]), &alternate)
            };
            for (i, &a) in alt.world(256).unwrap().iter().enumerate() {
                close(
                    a,
                    (0..n)
                        .map(|j| f64::from(full[i * n + j]) * f64::from(x[j]))
                        .sum(),
                );
            }
            // Selecting an alternate matrix never overwrites the default input.
            plan.update(&mut data).unwrap();
            assert_eq!(
                bits(&data.readback().unwrap().values),
                bits(&original.values)
            );
            let before_matrix = raw(&data.matrix);
            let mut selected = m.clone();
            selected[0] += 0.125;
            data.write_world_matrix(256, &selected).unwrap();
            let after_matrix = raw(&data.matrix);
            let mc = m.len();
            for i in (0..4 + 256 * mc).chain(4 + 257 * mc..before_matrix.len()) {
                assert_eq!(before_matrix[i].to_bits(), after_matrix[i].to_bits());
            }
            assert_eq!(
                bits(&after_matrix[4 + 256 * mc..4 + 257 * mc]),
                bits(&selected)
            );
            let before = raw(&data.vector);
            data.write_world_vector(256, &vec![0.0; n]).unwrap();
            let after = raw(&data.vector);
            for i in (0..4 + 256 * n).chain(4 + 257 * n..before.len()) {
                assert_eq!(before[i].to_bits(), after[i].to_bits());
            }
            assert!(data.readback().is_err());
            plan.update_with(&mut data, Some(&skip), false).unwrap();
            let snapshot = data.readback().unwrap();
            assert!(
                snapshot
                    .world(256)
                    .unwrap()
                    .iter()
                    .all(|v| v.to_bits() == 0)
            );
            drop(data);
            drop(plan);
            drop(other);
            assert_eq!(
                bits(snapshot.world(0).unwrap()),
                bits(original.world(0).unwrap())
            );
        }
        drop(session);
        let report = serde_json::json!({"batches":batches,"slots":slots,"max_abs":max_abs,"phase_ms":start.elapsed().as_millis(),"owned_capacity_max_bytes":capacity,"abs":2e-5,"rel":2e-5,"native_or_upstream_equivalence":false});
        std::fs::write(
            std::env::var_os("MJWARP_MUL_M_REPORT").unwrap(),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
    }
    #[cfg(feature = "cuda-probe")]
    fn full_matrix(input: &SparseMassInput, csr: &[f32]) -> Vec<f32> {
        let n = input.nv();
        let f = input.fields();
        let mut out = vec![0.0; n * n];
        for i in 0..n {
            let a = f.m_rowadr[i] as usize;
            let row = a..a + f.m_rownnz[i] as usize;
            for (&j, &value) in f.m_colind[row.clone()].iter().zip(&csr[row]) {
                let j = j as usize;
                out[i * n + j] = value;
                out[j * n + i] = value;
            }
        }
        out
    }
    #[cfg(feature = "cuda-probe")]
    fn close(a: f32, b: f64) {
        assert!(
            (f64::from(a) - b).abs() <= 2e-5 + 2e-5 * b.abs(),
            "mul_m {a} vs {b}"
        );
    }
    #[cfg(feature = "cuda-probe")]
    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|v| v.to_bits()).collect()
    }
    #[cfg(feature = "cuda-probe")]
    fn raw(v: &TransferBuffer<f32>) -> Vec<f32> {
        let mut out = vec![0.0; v.len()];
        v.read_range_into(0, &mut out).unwrap();
        out
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new mul_m source/output/status guards, skipped payload and no partial publish"]
    fn gpu_rejects_source_guards_and_nonfinite_without_publishing_partial_result() {
        let session = TransferSession::new(0).unwrap();
        let i = sparse();
        let n = i.nv();
        let nc = i.nc();
        let plan = MassMultiplyPlan::new(&session, i, MassMultiplyFormat::Sparse).unwrap();
        let mut d = plan.create_data(2).unwrap();
        d.write_matrix(&vec![1.0; 2 * nc]).unwrap();
        d.write_vector(&vec![1.0; 2 * n]).unwrap();
        plan.update(&mut d).unwrap();
        let good = d.readback().unwrap();
        for (which, offset, bad) in [
            (0, 0, 0.0),
            (0, 4 + 2 * nc, 0.0),
            (0, 4, f32::NAN),
            (1, 0, 0.0),
            (1, 4 + 2 * n, 0.0),
            (1, 4, f32::INFINITY),
            (2, 0, 0.0),
            (2, 4 + 2 * n, 0.0),
            (3, 0, 0.0),
            (3, 6, 0.0),
        ] {
            let buf = match which {
                0 => &mut d.matrix,
                1 => &mut d.vector,
                2 => &mut d.output,
                _ => &mut d.status,
            };
            let mut old = [0.0];
            buf.read_range_into(offset, &mut old).unwrap();
            buf.write_range(offset, &[bad]).unwrap();
            assert!(plan.update(&mut d).is_err());
            assert!(d.readback().is_err());
            let buf = match which {
                0 => &mut d.matrix,
                1 => &mut d.vector,
                2 => &mut d.output,
                _ => &mut d.status,
            };
            buf.write_range(offset, &old).unwrap();
            plan.update(&mut d).unwrap();
            assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
        }
        d.matrix.write_range(4, &[f32::NAN]).unwrap();
        plan.update_with(&mut d, Some(&[true, false]), false)
            .unwrap();
        assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
        assert!(plan.update(&mut d).is_err());
        d.matrix.write_range(4, &[1.0]).unwrap();
        assert!(plan.update_with(&mut d, None, true).is_err());
        d.write_alternate_matrix(&vec![2.0; 2 * nc]).unwrap();
        plan.update_with(&mut d, None, true).unwrap();
        let alt = d.readback().unwrap();
        for (&a, &b) in alt.world(0).unwrap().iter().zip(good.world(0).unwrap()) {
            assert_eq!(a.to_bits(), (2.0 * b).to_bits());
        }
    }
}
