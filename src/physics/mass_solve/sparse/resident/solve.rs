//! 常驻CSR因子与独立RHS流；不读取稠密矩阵来回代。
//! 源算法跟随冻结上游；实际融合产物等价仍未验收。
use super::{SparseMassData, SparseMassFactorOutput, SparseMassFactorWorld, SparseMassPlan};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout, SparseMassInput};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

pub struct SparseMassSolvePlan {
    mass: SparseMassPlan,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
}
pub struct SparseMassSolveData {
    mass: SparseMassData,
    layout: BatchLayout,
    rhs_count: usize,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    rhs: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassSolveOutput {
    factor: SparseMassFactorOutput,
    layout: BatchLayout,
    rhs_count: usize,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassSolveWorld<'a> {
    pub factor: SparseMassFactorWorld<'a>,
    /// 行优先[RHS,nv]，不包含守卫。
    pub solution: &'a [f32],
}
impl SparseMassSolveOutput {
    pub fn input(&self) -> &SparseMassInput {
        self.factor.input()
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    pub fn world(&self, world: usize) -> Result<SparseMassSolveWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(SparseMassSolveWorld {
            factor: self.factor.world(world)?,
            solution: &self.values[4 + r.start..4 + r.end],
        })
    }
}
fn layout(worlds: usize, nv: usize, rhs_count: usize) -> Result<BatchLayout, InputError> {
    let invalid = || InputError::InvalidDimension {
        field: "sparse_solve_layout",
    };
    if rhs_count == 0
        || rhs_count > i32::MAX as usize
        || worlds == 0
        || worlds
            .checked_mul(rhs_count)
            .is_none_or(|v| v > i32::MAX as usize)
    {
        return Err(invalid());
    }
    let stride = nv.checked_mul(rhs_count).ok_or_else(invalid)?;
    let layout = BatchLayout::new(worlds, stride, 4)?;
    BatchLayout::new(
        1,
        layout.total_elements().checked_add(8).ok_or_else(invalid)?,
        4,
    )?;
    Ok(layout)
}
impl SparseMassSolvePlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        input: SparseMassInput,
    ) -> Result<Self, TransferError> {
        let mass = SparseMassPlan::new(session, model, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = mass;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let input = mass.input();
            let offsets = input.qld_level_offsets();
            let mut metadata = vec![(offsets.len() - 1) as i32];
            metadata.extend(input.qld_block_adr());
            let f = input.fields();
            metadata.extend((0..input.nv()).map(|i| f.m_rowadr[i] + f.m_rownnz[i] - 1));
            metadata.extend(offsets);
            metadata.extend(input.qld_all_updates().iter().flatten());
            Ok(Self {
                mass,
                metadata: session.upload(&metadata)?,
                kernels: [
                    SynchronousKernel::compile(session, CUDA, "sparse_solve_validate")?,
                    SynchronousKernel::compile(session, CUDA, "sparse_solve_fused")?,
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
    ) -> Result<SparseMassSolveData, TransferError> {
        let layout = layout(worlds, self.input().nv(), rhs_count)?;
        let mass = self.mass.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (mass, layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
            values.fill(-131072.0);
            values[4..4 + layout.total_elements()].fill(0.0);
            let rhs = self.mass.session.upload(&values)?;
            values[4..4 + layout.total_elements()].fill(f32::NAN);
            Ok(SparseMassSolveData {
                mass,
                layout,
                rhs_count,
                ready: false,
                rhs,
                output: self.mass.session.upload(&values)?,
            })
        }
    }
    pub fn update(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.update_position(data)?;
        self.update_matrix(data)?;
        self.update_gather(data)?;
        self.update_factor(data)?;
        self.update_solve(data)
    }
    pub fn update_position(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_position(&mut data.mass)
    }
    pub fn update_matrix(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_matrix(&mut data.mass)
    }
    pub fn update_gather(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_gather(&mut data.mass)
    }
    pub fn update_factor(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_factor(&mut data.mass)
    }
    pub fn update_solve(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        if !data.mass.gathered || !data.mass.factor.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_mass_factor",
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
                self.input().nc() as u32,
                data.rhs_count as u32,
                data.worlds() as u32,
            ];
            // SAFETY: Checked metadata contains n block flags, n diagonal slots,
            // level offsets and bounded CSR updates. Guarded factor/RHS strides
            // match this plan; each validation thread owns one world status.
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &data.rhs,
                    &data.mass.factor.output,
                    &mut data.mass.status,
                    dimensions,
                )?;
            }
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.mass.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "sparse_solve_status")?;
            for (index, &actual) in status[4..4 + data.worlds()].iter().enumerate() {
                if actual != 0.0 {
                    if actual == 1.0 || actual == 3.0 {
                        data.mass.factor.ready = false;
                    }
                    return Err(ProbeError::Mismatch {
                        index,
                        expected: 0.0,
                        actual,
                    }
                    .into());
                }
            }
            // SAFETY: Every block owns one world/RHS and all 128 threads reach
            // every layer barrier. Only its disjoint output slice changes.
            unsafe {
                self.kernels[1].launch_sparse_solve(
                    &self.metadata,
                    &data.rhs,
                    &data.mass.factor.output,
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
impl SparseMassSolveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.rhs_count
    }
    pub fn nv(&self) -> usize {
        self.mass.factor.input.nv()
    }
    fn after_qpos(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.mass.write_qpos(values);
        self.after_qpos(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.mass.write_world_qpos(world, values);
        self.after_qpos(result)
    }
    fn check_rhs(values: &[f32], expected: usize) -> Result<(), InputError> {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "sparse_solve_rhs",
                expected,
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "sparse_solve_rhs",
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
        crate::physics::check_device_values(&values, "sparse_solve_output")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassSolveOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_mass_solve",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // 只读取缓存因子；旧CSR与CRB不再决定解的寿命。
            // 守卫、全部因子、原主元与逆对角仍须有效。
            if !self.mass.factor.ready {
                return Err(TransferError::StageNotReady {
                    stage: "sparse_mass_factor",
                });
            }
            let f = &self.mass.factor;
            let mut values =
                crate::runtime::host_staging::<f32>(f.output_layout.total_elements() + 8)?;
            f.output.read_range_into(0, &mut values)?;
            crate::physics::check_device_values(&values, "sparse_solve_cached_factor")?;
            let fields = f.input.fields();
            for world in 0..self.worlds() {
                let start = 4 + f.output_layout.world_elements(world)?.start;
                for dof in 0..self.nv() {
                    let value =
                        values[start + (fields.m_rowadr[dof] + fields.m_rownnz[dof] - 1) as usize];
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof,
                            value: f64::from(value),
                        });
                    }
                }
            }
            let factor = SparseMassFactorOutput {
                input: f.input.clone(),
                layout: f.output_layout,
                values,
            };
            for world in 0..self.worlds() {
                for (dof, &value) in factor.world(world)?.diagonal_inverse.iter().enumerate() {
                    if value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof,
                            value: f64::from(value),
                        });
                    }
                }
            }
            let mut rhs = crate::runtime::host_staging::<f32>(self.layout.total_elements() + 8)?;
            self.rhs.read_range_into(0, &mut rhs)?;
            crate::physics::check_device_values(&rhs, "sparse_solve_rhs")?;
            Ok(SparseMassSolveOutput {
                factor,
                layout: self.layout,
                rhs_count: self.rhs_count,
                values: self.checked_solution()?,
            })
        }
    }
}

// Frozen smooth.py _solve_LD_sparse_fused (71da24d), Apache-2.0.
// Copyright 2025 The Newton Developers; licenses/Apache-2.0.txt.
// Compact slots use the frozen independent D*y branch.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void sparse_solve_validate(const int* meta,const float* rhs,
 const float* factor,float* status,unsigned n,unsigned nc,unsigned nr,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long fs=1ull*nc+n,rs=1ull*n*nr;
 bool f=true,r=true;
 for(unsigned j=0;j<4;++j) {
   f=f && factor[j]==-131072.0f && factor[4+fs*worlds+j]==-131072.0f;
   r=r && rhs[j]==-131072.0f && rhs[4+rs*worlds+j]==-131072.0f;
 }
 const float* L=factor+4+fs*w;const float* b=rhs+4+rs*w;
 for(unsigned long long j=0;j<fs;++j)f=f && isfinite(L[j]);
 const int* diag=meta+1+n;
 for(unsigned i=0;i<n;++i)f=f && L[diag[i]]>0.0f && L[nc+i]>0.0f;
 for(unsigned long long j=0;j<rs;++j)r=r && isfinite(b[j]);
 status[4+w]=(f?0.0f:1.0f)+(r?0.0f:2.0f);
}
extern "C" __global__ void sparse_solve_fused(const int* meta,const float* rhs,
 const float* factor,float* output,unsigned n,unsigned nc,unsigned nr,unsigned worlds) {
 unsigned group=blockIdx.x,w=group/nr,tid=threadIdx.x;
 const int* flags=meta+1;const int* offsets=flags+2*n;const int* updates=offsets+meta[0]+1;
 const float* L=factor+4+(unsigned long long)w*(1ull*nc+n);const float* D=L+nc;
 const float* b=rhs+4+(unsigned long long)group*n;float* x=output+4+(unsigned long long)group*n;
 for(unsigned i=tid;i<n;i+=128) x[i]=flags[i]==-1?b[i]:__fmul_rn(D[i],b[i]);
 __syncthreads();
 for(int level=meta[0]-1;level>=0;--level) {
   for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
     int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];
     float value=__fmul_rn(L[a],x[k]),ignored;
     asm volatile("atom.global.add.f32 %0, [%1], %2;" : "=f"(ignored) : "l"(x+i),"f"(-value) : "memory");
   }
   __syncthreads();
 }
 for(unsigned i=tid;i<n;i+=128)if(flags[i]==-1)x[i]=__fmul_rn(x[i],D[i]);
 __syncthreads();
 for(int level=0;level<meta[0];++level) {
   for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
     int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];
     float value=__fmul_rn(L[a],x[i]),ignored;
     asm volatile("atom.global.add.f32 %0, [%1], %2;" : "=f"(ignored) : "l"(x+k),"f"(-value) : "memory");
   }
   __syncthreads();
 }
}
"#;

#[cfg(all(test, feature = "cuda-probe"))]
mod tests {
    use super::*;
    use crate as engine;
    mod input {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/sparse_mass_model.rs"
        ));
    }
    #[test]
    fn rejects_zero_or_overflowing_rhs_layouts_without_allocation() {
        for (w, n, r) in [
            (0, 65, 1),
            (1, 65, 0),
            (usize::MAX, 65, 1),
            (1, usize::MAX, 2),
            (2, 0, i32::MAX as usize),
        ] {
            assert!(layout(w, n, r).is_err());
        }
        assert_eq!(layout(2, 0, 3).unwrap().total_elements(), 0);
        assert_eq!(layout(257, 65, 5).unwrap().total_elements(), 83525);
    }
    fn factor_values(data: &SparseMassSolveData) -> Vec<f32> {
        let mut values = vec![0.0; data.mass.factor.output_layout.total_elements() + 8];
        data.mass
            .factor
            .output
            .read_range_into(0, &mut values)
            .unwrap();
        values
    }
    #[test]
    #[ignore = "requires NVIDIA driver/NVRTC; new fused solve source/cache fault purpose"]
    fn cached_solve_ignores_dense_poison_and_rejects_all_guarded_source_faults() {
        let refs = input::reference();
        let m = &refs["models"][0];
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassSolvePlan::new(&session, input::model(m), input::sparse(m)).unwrap();
        let mut data = plan.create_data(2, 3).unwrap();
        data.write_qpos(&input::floats(&m["cases"][0], "qpos").repeat(2))
            .unwrap();
        data.write_rhs(&vec![0.5; 390]).unwrap();
        plan.update(&mut data).unwrap();
        let mut rhs_before = vec![0.0; 398];
        data.rhs.read_range_into(0, &mut rhs_before).unwrap();
        data.write_world_rhs(1, &vec![0.5; 195]).unwrap();
        let mut rhs_after = vec![0.0; 398];
        data.rhs.read_range_into(0, &mut rhs_after).unwrap();
        for i in (0..199).chain(394..398) {
            assert_eq!(rhs_before[i].to_bits(), rhs_after[i].to_bits());
        }
        assert!(
            rhs_after[199..394]
                .iter()
                .all(|v| v.to_bits() == 0.5f32.to_bits())
        );
        plan.update_solve(&mut data).unwrap();
        let before = data.readback().unwrap();
        let saved = factor_values(&data);
        let dense_slot = 4 + 10 * 66 + 1;
        data.mass
            .matrix
            .corrupt_output_for_test(dense_slot, f32::NAN)
            .unwrap();
        data.write_rhs(&vec![0.5; 390]).unwrap();
        plan.update_solve(&mut data).unwrap();
        assert_eq!(
            factor_values(&data)
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            saved.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        let after = data.readback().unwrap();
        for w in 0..2 {
            for (&a, &b) in after
                .world(w)
                .unwrap()
                .solution
                .iter()
                .zip(before.world(w).unwrap().solution)
            {
                assert!((a - b).abs() <= 2e-4 + 2e-4 * b.abs());
            }
        }
        assert!(plan.update_gather(&mut data).is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.readback().is_err());
        plan.update(&mut data).unwrap();
        let mut csr = [0.0];
        data.mass
            .factor
            .matrix
            .read_range_into(4, &mut csr)
            .unwrap();
        data.mass.factor.matrix.write_range(4, &[f32::NAN]).unwrap();
        data.write_rhs(&vec![0.5; 390]).unwrap();
        plan.update_solve(&mut data).unwrap();
        assert!(data.readback().is_ok());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.mass.factor.matrix.write_range(4, &csr).unwrap();
        plan.update_factor(&mut data).unwrap();
        plan.update_solve(&mut data).unwrap();
        let saved = factor_values(&data);
        let inverse = 4 + plan.input().nc();
        for (offset, bad) in [
            (0, 0.0),
            (saved.len() - 4, 0.0),
            (5, f32::NAN),
            (inverse, -1.0),
            (inverse, f32::INFINITY),
        ] {
            data.mass.factor.output.write_range(offset, &[bad]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(!data.mass.factor.ready);
            assert!(data.readback().is_err());
            data.mass
                .factor
                .output
                .write_range(offset, &[saved[offset]])
                .unwrap();
            plan.update_factor(&mut data).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        let len = data.layout.total_elements() + 8;
        for (offset, bad, restore) in [
            (0, 0.0, -131072.0),
            (len - 4, 0.0, -131072.0),
            (4, f32::NAN, 0.5),
        ] {
            data.rhs.write_range(offset, &[bad]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.mass.factor.ready);
            assert!(data.readback().is_err());
            data.rhs.write_range(offset, &[restore]).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        for offset in [0, len - 4] {
            data.output.write_range(offset, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.mass.factor.ready);
            assert!(data.readback().is_err());
            data.output.write_range(offset, &[-131072.0]).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        data.output.write_range(4, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        plan.update_solve(&mut data).unwrap();
        for offset in [0, data.worlds() + 4] {
            data.mass.status.write_range(offset, &[0.0]).unwrap();
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.readback().is_err());
            assert!(data.mass.factor.ready);
            data.mass.status.write_range(offset, &[-131072.0]).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        assert!(data.readback().is_ok());
    }
}
