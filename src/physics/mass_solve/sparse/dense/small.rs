//! 非紧凑Small前缀U；不生成Sparse/Compact因子或Tile替代。
use super::{SparseMassDenseData, SparseMassDensePlan};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, SparseMassInput, SparseMassRoute};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

pub struct SparseMassSmallPlan {
    source: SparseMassDensePlan,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    validator: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    groups: Vec<(u32, u32, SynchronousKernel)>,
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
            let mut groups = Vec::new();
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
                    let entry = match size {
                        2 => "small_prefix_factor_2",
                        3 => "small_prefix_factor_3",
                        4 => "small_prefix_factor_4",
                        5 => "small_prefix_factor_5",
                        _ => "small_prefix_factor_6",
                    };
                    groups.push((
                        first as u32,
                        count as u32,
                        SynchronousKernel::compile(session, CUDA, entry)?,
                    ));
                }
            }
            if meta.is_empty() {
                meta.push(0);
            }
            Ok(Self {
                source,
                metadata: session.upload(&meta)?,
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
                    &self.metadata,
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
            for &(first, count, ref kernel) in &self.groups {
                // SAFETY: Static size2..6 groups contain proven, disjoint prefix
                // blocks. Each thread owns one world/block; no thread shares U.
                unsafe {
                    kernel.launch_small_factor(
                        &self.metadata,
                        &data.source.dense,
                        &data.source.dense,
                        &mut data.factor,
                        [prefix, first, count, worlds * count],
                    )?;
                }
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
    fn routes_keep_mathdx_tile_rejected_without_reclassifying_small() {
        assert!(check_routes(&input::mixed_input()).is_ok());
        assert!(check_routes(&input::sparse(&input::base()["models"][3])).is_err());
    }
    #[test]
    #[ignore = "requires NVIDIA/NVRTC; new Small prefix cached-input/factor fault purpose"]
    fn cache_ignores_old_csr_and_guarded_factor_stages_reject_faults() {
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassSmallPlan::new(&session, input::mixed_input()).unwrap();
        let mut data = plan.create_data(2).unwrap();
        let matrix = input::mixed_matrix(0);
        data.write_matrix(&matrix.repeat(2)).unwrap();
        plan.update(&mut data).unwrap();
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
    }
}
