//! 消费同代合并因子，求解Sparse、Small与Compact。
//! RHS独立批量；Tile与CRB桥接仍不支持。
use super::{
    SparseMassMixedFactorData, SparseMassMixedFactorOutput, SparseMassMixedFactorPlan,
    SparseMassMixedFactorWorld,
};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
#[cfg(any(feature = "cuda-probe", test))]
use crate::model::SparseMassRoute;
use crate::model::{BatchLayout, SparseMassInput};
use crate::runtime::TransferSession;
mod resident;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
pub use resident::{SparseMassMixedResidentData, SparseMassMixedResidentPlan};

pub struct SparseMassMixedSolvePlan {
    mass: SparseMassMixedFactorPlan,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
}
pub struct SparseMassMixedSolveData {
    mass: SparseMassMixedFactorData,
    layout: BatchLayout,
    rhs_count: usize,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    rhs: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassMixedSolveOutput {
    factor: SparseMassMixedFactorOutput,
    layout: BatchLayout,
    rhs_count: usize,
    rhs: Vec<f32>,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassMixedSolveWorld<'a> {
    pub factor: SparseMassMixedFactorWorld<'a>,
    /// 每世界按RHS、DOF连续排列。
    pub rhs: &'a [f32],
    pub solution: &'a [f32],
}
impl SparseMassMixedSolveOutput {
    pub fn input(&self) -> &SparseMassInput {
        self.factor.input()
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    pub fn world(&self, world: usize) -> Result<SparseMassMixedSolveWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(SparseMassMixedSolveWorld {
            factor: self.factor.world(world)?,
            rhs: &self.rhs[4 + r.start..4 + r.end],
            solution: &self.values[4 + r.start..4 + r.end],
        })
    }
}
fn layout(worlds: usize, nv: usize, nr: usize) -> Result<BatchLayout, InputError> {
    let invalid = || InputError::InvalidDimension {
        field: "mixed_solve_layout",
    };
    if nv == 0
        || nr == 0
        || nr > i32::MAX as usize
        || worlds == 0
        || worlds.checked_mul(nr).is_none_or(|v| v > i32::MAX as usize)
    {
        return Err(invalid());
    }
    let layout = BatchLayout::new(worlds, nv.checked_mul(nr).ok_or_else(invalid)?, 4)?;
    BatchLayout::new(
        1,
        layout.total_elements().checked_add(8).ok_or_else(invalid)?,
        4,
    )?;
    Ok(layout)
}
#[cfg(any(feature = "cuda-probe", test))]
fn metadata(input: &SparseMassInput) -> Result<Vec<i32>, TransferError> {
    let n = input.nv();
    let offsets = input.qld_level_offsets();
    let small: Vec<_> = input
        .blocks()
        .iter()
        .filter(|b| b.route == SparseMassRoute::SmallCholesky)
        .collect();
    let length = 4usize
        .checked_add(2 * n)
        .and_then(|v| v.checked_add(offsets.len()))
        .and_then(|v| v.checked_add(3 * input.qld_all_updates().len()))
        .and_then(|v| v.checked_add(3 * small.len()))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "mixed_solve_metadata",
        })?;
    let mut packed = crate::runtime::host_staging::<i32>(length)?;
    packed.clear();
    packed.extend([
        (offsets.len() - 1) as i32,
        input.qld_block_total() as i32,
        input.nc() as i32,
        small.len() as i32,
    ]);
    packed.extend(input.qld_block_adr());
    let f = input.fields();
    let mut diagonal = vec![0i32; n];
    for block in input.blocks() {
        for i in 0..block.size {
            let dof = block.start + i;
            diagonal[dof] = if block.route == SparseMassRoute::SmallCholesky {
                input.qld_block_adr()[dof] + (i * block.size + i) as i32
            } else {
                (input.qld_block_total() + f.m_rowadr[dof] as usize + f.m_rownnz[dof] as usize - 1)
                    as i32
            };
        }
    }
    packed.extend(diagonal);
    packed.extend(offsets);
    packed.extend(input.qld_all_updates().iter().flatten());
    for block in small {
        packed.extend([
            block.start as i32,
            block.size as i32,
            input.qld_block_adr()[block.start],
        ]);
    }
    Ok(packed)
}
impl SparseMassMixedSolvePlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        let mass = SparseMassMixedFactorPlan::new(session, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = mass;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(Self {
                metadata: session.upload(&metadata(mass.input())?)?,
                mass,
                kernels: [
                    SynchronousKernel::compile(session, CUDA, "mixed_solve_validate")?,
                    SynchronousKernel::compile(session, CUDA, "mixed_solve_fused")?,
                ],
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.mass.input()
    }
    pub fn create_data(
        &self,
        worlds: usize,
        rhs_count: usize,
    ) -> Result<SparseMassMixedSolveData, TransferError> {
        let layout = layout(worlds, self.input().nv(), rhs_count)?;
        let mass = self.mass.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (mass, layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, initial: f32| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(initial);
                self.mass.session.upload(&values)
            };
            Ok(SparseMassMixedSolveData {
                mass,
                layout,
                rhs_count,
                ready: false,
                rhs: allocate(layout.total_elements(), 0.0)?,
                output: allocate(layout.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.update_gather(data)?;
        self.update_factor(data)?;
        self.update_solve(data)
    }
    pub fn update_gather(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_gather(&mut data.mass)
    }
    pub fn update_factor(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_factor(&mut data.mass)
    }
    pub fn update_solve(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        if !data.mass.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let dimensions = [
                self.input().nv() as u32,
                data.mass.layout.elements_per_world() as u32,
                data.rhs_count as u32,
                data.worlds() as u32,
            ];
            // SAFETY: Same-plan merged factor/RHS/status have checked guarded
            // strides. Packed diagonal indices and routes bound all reads.
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &data.rhs,
                    &data.mass.output,
                    &mut data.status,
                    dimensions,
                )?;
            }
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "mixed_solve_status")?;
            for (world, &value) in status[4..4 + data.worlds()].iter().enumerate() {
                if value != 0.0 {
                    if value == 1.0 || value == 3.0 {
                        data.mass.ready = false;
                    }
                    return Err(InputError::InvalidTopology {
                        field: "mixed_solve_sources",
                        index: world,
                        reason: "invalid_cached_factor_or_rhs",
                    }
                    .into());
                }
            }
            // SAFETY: Metadata lists disjoint Small blocks and Sparse-only
            // updates; each 128-thread block owns one world/RHS solution.
            unsafe {
                self.kernels[1].launch_mixed_solve(
                    &self.metadata,
                    &data.rhs,
                    &data.mass.output,
                    &mut data.output,
                    dimensions,
                )?;
            }
            data.checked_solution()?;
            data.ready = true;
            Ok(())
        }
    }
}
impl SparseMassMixedSolveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    fn after_matrix(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.mass.write_matrix(values);
        self.after_matrix(result)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let result = self.mass.write_world_matrix(world, values);
        self.after_matrix(result)
    }
    fn check_rhs(values: &[f32], expected: usize) -> Result<(), InputError> {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "mixed_solve_rhs",
                expected,
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "mixed_solve_rhs",
                    index,
                });
            }
        }
        Ok(())
    }
    pub fn write_rhs(&mut self, values: &[f32]) -> Result<(), TransferError> {
        Self::check_rhs(values, self.layout.total_elements())?;
        self.write(0, values)
    }
    pub fn write_world_rhs(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let r = self.layout.world_elements(world)?;
        Self::check_rhs(values, r.len())?;
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
            self.rhs.write_range(4 + offset, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_solution(&self) -> Result<Vec<f32>, TransferError> {
        let mut values = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut values)?;
        crate::physics::check_device_values(&values, "mixed_solve_solution")?;
        Ok(values)
    }
    #[cfg(feature = "cuda-probe")]
    fn cached_factor(&self) -> Result<SparseMassMixedFactorOutput, TransferError> {
        if !self.mass.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve_factor",
            });
        }
        let values = self.mass.checked_values()?;
        let input = &self.mass.sparse.input;
        let f = input.fields();
        for world in 0..self.worlds() {
            let first = 4 + self.mass.layout.world_elements(world)?.start + input.qld_block_total();
            for block in input
                .blocks()
                .iter()
                .filter(|b| b.route != SparseMassRoute::SmallCholesky)
            {
                for dof in block.start..block.start + block.size {
                    let value = values[first + (f.m_rowadr[dof] + f.m_rownnz[dof] - 1) as usize];
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof,
                            value: f64::from(value),
                        });
                    }
                }
            }
        }
        Ok(SparseMassMixedFactorOutput {
            input: input.clone(),
            layout: self.mass.layout,
            values,
        })
    }
    pub fn readback(&self) -> Result<SparseMassMixedSolveOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let factor = self.cached_factor()?;
            let mut rhs = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
            self.rhs.read_range_into(0, &mut rhs)?;
            crate::physics::check_device_values(&rhs, "mixed_solve_rhs")?;
            Ok(SparseMassMixedSolveOutput {
                factor,
                layout: self.layout,
                rhs_count: self.rhs_count,
                rhs,
                values: self.checked_solution()?,
            })
        }
    }
}

// Frozen smooth.py3314–3553; Apache-2.0, Copyright2025 The Newton Developers.
// Private ABI adds RHS grouping, not a different scalar or sparse algorithm.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
__device__ float mixed_schur(float value,float a,float b) {
 float result;asm("{ .reg .f32 product; mul.f32 product,%2,%3; sub.f32 %0,%1,product; }"
 : "=f"(result):"f"(value),"f"(a),"f"(b));return result;
}
template<int N> __device__ void mixed_small_solve(const float* F,const float* b,float* x,int start,int address) {
 for(int i=0;i<N;++i) {
  float value=b[start+i];for(int k=0;k<i;++k)value=mixed_schur(value,F[address+k*N+i],x[start+k]);
  x[start+i]=__fdiv_rn(value,F[address+i*N+i]);
 }
 for(int i=N-1;i>=0;--i) {
  float value=x[start+i];for(int k=i+1;k<N;++k)value=mixed_schur(value,F[address+i*N+k],x[start+k]);
  x[start+i]=__fdiv_rn(value,F[address+i*N+i]);
 }
}
extern "C" __global__ void mixed_solve_validate(const int* meta,const float* rhs,const float* factor,
 float* status,unsigned n,unsigned stride,unsigned nr,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long rs=1ull*n*nr;bool f=true,r=true;
 for(unsigned j=0;j<4;++j) {
  f=f && factor[j]==-131072.0f && factor[4+1ull*stride*worlds+j]==-131072.0f;
  r=r && rhs[j]==-131072.0f && rhs[4+rs*worlds+j]==-131072.0f;
 }
 const float* F=factor+4+1ull*w*stride;const int* diag=meta+4+n;
 for(unsigned j=0;j<stride;++j)f=f && isfinite(F[j]);
 for(unsigned i=0;i<n;++i)f=f && F[diag[i]]>0.0f && F[meta[1]+meta[2]+i]>0.0f;
 const float* b=rhs+4+rs*w;for(unsigned long long j=0;j<rs;++j)r=r && isfinite(b[j]);
 status[4+w]=(f?0.0f:1.0f)+(r?0.0f:2.0f);
}
extern "C" __global__ void mixed_solve_fused(const int* meta,const float* rhs,const float* factor,
 float* output,unsigned n,unsigned stride,unsigned nr,unsigned worlds) {
 unsigned group=blockIdx.x,w=group/nr,tid=threadIdx.x;
 const int* flags=meta+4;const int* offsets=flags+2*n;const int* updates=offsets+meta[0]+1;
 const int* small=updates+3*offsets[meta[0]];
 const float* F=factor+4+1ull*w*stride;const float* L=F+meta[1];const float* D=L+meta[2];
 const float* b=rhs+4+1ull*group*n;float* x=output+4+1ull*group*n;
 for(unsigned i=tid;i<n;i+=128) {
  if(flags[i]==-1)x[i]=b[i];else if(flags[i]==-2)x[i]=__fmul_rn(D[i],b[i]);
 }
 for(unsigned block=tid;block<(unsigned)meta[3];block+=128) {
  int start=small[3*block],size=small[3*block+1],a=small[3*block+2];
  switch(size) {
   case 2:mixed_small_solve<2>(F,b,x,start,a);break;
   case 3:mixed_small_solve<3>(F,b,x,start,a);break;
   case 4:mixed_small_solve<4>(F,b,x,start,a);break;
   case 5:mixed_small_solve<5>(F,b,x,start,a);break;
   case 6:mixed_small_solve<6>(F,b,x,start,a);break;
  }
 }
 __syncthreads();
 for(int level=meta[0]-1;level>=0;--level) {
  for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
   int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];float value=__fmul_rn(L[a],x[k]),ignored;
   asm volatile("atom.global.add.f32 %0, [%1], %2;":"=f"(ignored):"l"(x+i),"f"(-value):"memory");
  }
  __syncthreads();
 }
 for(unsigned i=tid;i<n;i+=128)if(flags[i]==-1)x[i]=__fmul_rn(x[i],D[i]);
 __syncthreads();
 for(int level=0;level<meta[0];++level) {
  for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
   int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];float value=__fmul_rn(L[a],x[i]),ignored;
   asm volatile("atom.global.add.f32 %0, [%1], %2;":"=f"(ignored):"l"(x+k),"f"(-value):"memory");
  }
  __syncthreads();
 }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cuda-probe")]
    use crate as engine;
    #[cfg(feature = "cuda-probe")]
    mod samples {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/mixed_mass_solve.rs"
        ));
    }
    fn input() -> SparseMassInput {
        let refs: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/mixed-mass-solve/reference.json"
        )))
        .unwrap();
        let f = &refs["models"][0]["fields"];
        let ints = |key: &str| {
            f[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i32)
                .collect()
        };
        SparseMassInput::new(crate::model::SparseMassFields {
            dof_parentid: ints("dof_parentid"),
            m_rownnz: ints("m_rownnz"),
            m_rowadr: ints("m_rowadr"),
            m_colind: ints("m_colind"),
            tree_dofadr: ints("tree_dofadr"),
            tree_dofnum: ints("tree_dofnum"),
        })
        .unwrap()
    }
    #[test]
    fn mixed_rhs_layout_rejects_zero_and_overflow_without_allocation() {
        for (w, n, r) in [
            (0, 86, 3),
            (1, 0, 3),
            (1, 86, 0),
            (usize::MAX, 86, 1),
            (1, usize::MAX, 3),
        ] {
            assert!(layout(w, n, r).is_err());
        }
        assert_eq!(layout(257, 86, 3).unwrap().total_elements(), 66306);
    }
    #[test]
    fn packed_route_diagonals_and_small_groups_keep_distinct_regions() {
        let input = input();
        let m = metadata(&input).unwrap();
        let n = input.nv();
        let flags = &m[4..4 + n];
        assert_eq!(&m[..4], &[64, 90, 2201, 5]);
        assert!(flags[..65].iter().all(|&v| v == -1));
        assert_eq!(flags[85], -2);
        let diag = &m[4 + n..4 + 2 * n];
        assert_eq!(diag[65], 0);
        assert_eq!(diag[66], 3);
        assert_eq!(diag[85], 2290);
        let offsets = &m[4 + 2 * n..4 + 2 * n + 65];
        let count = offsets[64] as usize;
        let updates = &m[4 + 2 * n + 65..4 + 2 * n + 65 + 3 * count];
        assert!(updates.chunks_exact(3).all(|u| u[0] < 65 && u[1] < 65));
        assert_eq!(&m[m.len() - 15..m.len() - 12], &[65, 2, 0]);
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new mixed consumer cached-only factor/source/guard failure purpose"]
    fn cached_merged_factor_ignores_old_sources_and_rejects_each_consumer_fault() {
        let refs = samples::reference();
        let model = &refs["models"][0];
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassMixedSolvePlan::new(&session, samples::input(model)).unwrap();
        let mut data = plan.create_data(2, 3).unwrap();
        data.write_matrix(&samples::matrix(model, 0).repeat(2))
            .unwrap();
        let rhs = samples::rhs(model, 0, 3);
        data.write_rhs(&rhs.repeat(2)).unwrap();
        plan.update(&mut data).unwrap();
        let before = data.readback().unwrap();
        let expected = samples::solution(model, 0, 3);
        let mut factor = vec![0.0; data.mass.output.len()];
        data.mass.output.read_range_into(0, &mut factor).unwrap();
        // Both stale primary and producer CSR can be invalid; the published
        // merged factor alone owns this solve's lifetime and physics inputs.
        let mut a = [0.0];
        let mut b = [0.0];
        data.mass.matrix.read_range_into(4, &mut a).unwrap();
        data.mass.sparse.matrix.read_range_into(4, &mut b).unwrap();
        data.mass.matrix.write_range(4, &[f32::NAN]).unwrap();
        data.mass.sparse.matrix.write_range(4, &[f32::NAN]).unwrap();
        data.write_rhs(&rhs.repeat(2)).unwrap();
        plan.update_solve(&mut data).unwrap();
        let after = data.readback().unwrap();
        let mut unchanged = vec![0.0; factor.len()];
        data.mass.output.read_range_into(0, &mut unchanged).unwrap();
        assert_eq!(
            factor.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            unchanged.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        for w in 0..2 {
            for (&actual, &reference) in after.world(w).unwrap().solution.iter().zip(&expected) {
                assert!((f64::from(actual) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
            }
        }
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        assert!(plan.update_gather(&mut data).is_err());
        data.mass.matrix.write_range(4, &a).unwrap();
        data.mass.sparse.matrix.write_range(4, &b).unwrap();
        plan.update(&mut data).unwrap();
        // A poisoned old Small producer must not invalidate a completed merge.
        data.mass
            .small
            .test_factor_buffer_mut()
            .write_range(0, &[0.0])
            .unwrap();
        data.write_rhs(&rhs.repeat(2)).unwrap();
        plan.update_solve(&mut data).unwrap();
        assert!(data.readback().is_ok());
        data.mass
            .small
            .test_factor_buffer_mut()
            .write_range(0, &[-131072.0])
            .unwrap();
        let prefix = plan.input().qld_block_total();
        let inverse = 4 + prefix + plan.input().nc();
        for (offset, bad) in [
            (0, 0.0),
            (factor.len() - 4, 0.0),
            (4, -1.0),
            (5, f32::NAN),
            (4 + prefix, -1.0),
            (inverse, -1.0),
            (inverse + 65, f32::INFINITY),
        ] {
            data.mass.output.write_range(offset, &[bad]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(!data.mass.ready);
            data.mass
                .output
                .write_range(offset, &[factor[offset]])
                .unwrap();
            plan.update_factor(&mut data).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        let len = data.rhs.len();
        for (offset, bad) in [(0, 0.0), (len - 4, 0.0), (4, f32::NAN)] {
            let mut saved = [0.0];
            data.rhs.read_range_into(offset, &mut saved).unwrap();
            data.rhs.write_range(offset, &[bad]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.mass.ready);
            data.rhs.write_range(offset, &saved).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        for offset in [0, len - 4] {
            data.output.write_range(offset, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.mass.ready);
            data.output.write_range(offset, &[-131072.0]).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        data.output.write_range(4, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        plan.update_solve(&mut data).unwrap();
        for offset in [0, data.worlds() + 4] {
            data.status.write_range(offset, &[0.0]).unwrap();
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.mass.ready);
            assert!(data.readback().is_err());
            data.status.write_range(offset, &[-131072.0]).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        let mut raw = vec![0.0; len];
        data.rhs.read_range_into(0, &mut raw).unwrap();
        data.write_world_rhs(1, &samples::rhs(model, 1, 3)).unwrap();
        let mut next = vec![0.0; len];
        data.rhs.read_range_into(0, &mut next).unwrap();
        for i in (0..4 + 258).chain(4 + 516..len) {
            assert_eq!(raw[i].to_bits(), next[i].to_bits());
        }
        plan.update_solve(&mut data).unwrap();
        assert!(data.readback().is_ok());
        assert_eq!(before.rhs_count(), 3);
    }
}
