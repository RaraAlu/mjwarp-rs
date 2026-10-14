//! 真正CSR分层分解；不调用稠密辅助。
//! 本轮不实现融合回代与非紧凑混合块。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, SparseMassInput, SparseMassRoute};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

mod resident;
pub use resident::{SparseMassData, SparseMassOutput, SparseMassPlan, SparseMassWorld};

pub struct SparseMassFactorPlan {
    input: Arc<SparseMassInput>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 3],
}
pub struct SparseMassFactorData {
    input: Arc<SparseMassInput>,
    matrix_layout: BatchLayout,
    output_layout: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    matrix: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassFactorOutput {
    input: Arc<SparseMassInput>,
    layout: BatchLayout,
    values: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassFactorWorld<'a> {
    /// 对应上游qLD[qLD_block_total..]。
    /// 它不包含尚未实现的dense前缀。
    pub sparse_factors: &'a [f32],
    pub diagonal_inverse: &'a [f32],
}
impl SparseMassFactorOutput {
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<SparseMassFactorWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        let v = &self.values[4 + r.start..4 + r.end];
        Ok(SparseMassFactorWorld {
            sparse_factors: &v[..self.input.nc()],
            diagonal_inverse: &v[self.input.nc()..],
        })
    }
}
fn layouts(
    input: &SparseMassInput,
    worlds: usize,
) -> Result<(BatchLayout, BatchLayout), InputError> {
    let stride = input
        .nc()
        .checked_add(input.nv())
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "sparse_factor_stride",
        })?;
    // 两种并行内核均采用单一线性工作域。
    let max_updates = input
        .qld_level_offsets()
        .windows(2)
        .map(|p| (p[1] - p[0]) as usize)
        .max()
        .unwrap_or(1);
    if worlds == 0
        || worlds
            .checked_mul(max_updates.max(input.nv()).max(1))
            .is_none_or(|v| v > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "sparse_factor_work_items",
        });
    }
    let m = BatchLayout::new(worlds, input.nc(), 4)?;
    let o = BatchLayout::new(worlds, stride, 4)?;
    for count in [m.total_elements(), o.total_elements()] {
        BatchLayout::new(
            1,
            count.checked_add(8).ok_or(InputError::Overflow {
                field: "sparse_factor_guards",
            })?,
            4,
        )?;
    }
    Ok((m, o))
}
fn matrix_values(layout: BatchLayout, values: &[f32]) -> Result<(), InputError> {
    if values.len() != layout.total_elements() {
        return Err(InputError::LengthMismatch {
            field: "sparse_mass_M",
            expected: layout.total_elements(),
            actual: values.len(),
        });
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: "sparse_mass_M",
                index,
            });
        }
    }
    Ok(())
}
impl SparseMassFactorPlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        // 无稀疏块时，上游不分配CSR因子尾区。
        // 不能用本工作区冒充纯紧凑qLD布局。
        if input.nv() != 0
            && !input
                .blocks()
                .iter()
                .any(|b| b.route == SparseMassRoute::SparseLdl)
        {
            return Err(InputError::InvalidTopology {
                field: "qLD_block_adr",
                index: 0,
                reason: "sparse_factor_region_required",
            }
            .into());
        }
        for block in input.blocks() {
            if !matches!(
                block.route,
                SparseMassRoute::SparseLdl | SparseMassRoute::Compact
            ) {
                return Err(InputError::InvalidTopology {
                    field: "qLD_block_adr",
                    index: block.start,
                    reason: "noncompact_mixed_factor_stage_not_implemented",
                }
                .into());
            }
        }
        let input = Arc::new(input);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, input);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut packed = vec![input.nv() as i32, input.nc() as i32];
            packed.extend(&input.fields().m_rownnz);
            packed.extend(&input.fields().m_rowadr);
            packed.extend(input.qld_all_updates().iter().flatten());
            if packed.len() > i32::MAX as usize {
                return Err(InputError::Overflow {
                    field: "sparse_factor_metadata",
                }
                .into());
            }
            let metadata = session.upload(&packed)?;
            let kernels = [
                SynchronousKernel::compile(session, CUDA, "sparse_mass_copy")?,
                SynchronousKernel::compile(session, CUDA, "sparse_mass_update")?,
                SynchronousKernel::compile(session, CUDA, "sparse_mass_diag")?,
            ];
            Ok(Self {
                input,
                session: session.clone(),
                metadata,
                kernels,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassFactorData, TransferError> {
        let (matrix_layout, output_layout) = layouts(&self.input, worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix_layout, output_layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |layout: BatchLayout, initial| {
                let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
                values.fill(-131072.0);
                values[4..4 + layout.total_elements()].fill(initial);
                self.session.upload(&values)
            };
            Ok(SparseMassFactorData {
                input: self.input.clone(),
                matrix_layout,
                output_layout,
                ready: false,
                matrix: allocate(matrix_layout, 0.0)?,
                output: allocate(output_layout, f32::NAN)?,
            })
        }
    }
    pub fn update_factor(&self, data: &mut SparseMassFactorData) -> Result<(), TransferError> {
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
            let worlds = data.worlds() as u32;
            // SAFETY: Packed metadata and both guarded strides come from checked
            // CSR topology. Inputs stay read-only; only output aliases L_in/L_out
            // inside the shader. Every level synchronizes before the next launch.
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.output,
                    [0, 0, data.output_layout.elements_per_world() as u32, worlds],
                )?;
            }
            for level in (0..self.input.qld_level_offsets().len() - 1).rev() {
                let offsets = self.input.qld_level_offsets();
                let count = (offsets[level + 1] - offsets[level]) as u32;
                // SAFETY: Each update uses proven ancestor-row prefixes. This
                // level owns its multipliers; atomic targets may overlap, exactly
                // as frozen _qLD_acc. Checked work count is worlds*level_size.
                unsafe {
                    self.kernels[1].launch(
                        &self.metadata,
                        &data.matrix,
                        &data.matrix,
                        &mut data.output,
                        [
                            offsets[level] as u32,
                            count,
                            data.output_layout.elements_per_world() as u32,
                            worlds * count,
                        ],
                    )?;
                }
            }
            if self.input.nv() != 0 {
                // SAFETY: One thread writes each diagonal inverse in its own
                // checked world. The factor CSR and diagonal output do not overlap.
                unsafe {
                    self.kernels[2].launch(
                        &self.metadata,
                        &data.matrix,
                        &data.matrix,
                        &mut data.output,
                        [
                            0,
                            0,
                            data.output_layout.elements_per_world() as u32,
                            worlds * self.input.nv() as u32,
                        ],
                    )?;
                }
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
impl SparseMassFactorData {
    pub fn worlds(&self) -> usize {
        self.output_layout.worlds()
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        matrix_values(self.matrix_layout, values)?;
        self.write(0, values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let range = self.matrix_layout.world_elements(world)?;
        matrix_values(BatchLayout::new(1, self.input.nc(), 4)?, values)?;
        self.write(range.start, values)
    }
    fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.matrix.write_range(4 + offset, values)
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut matrix =
            crate::runtime::host_staging::<f32>(self.matrix_layout.total_elements() + 8)?;
        self.matrix.read_range_into(0, &mut matrix)?;
        crate::physics::check_device_values(&matrix, "sparse_mass_M")?;
        let mut values =
            crate::runtime::host_staging::<f32>(self.output_layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut values)?;
        // 先定位主元，再统一检查有限值与守卫。
        for world in 0..self.worlds() {
            let start = 4 + self.output_layout.world_elements(world)?.start;
            for dof in 0..self.input.nv() {
                let f = self.input.fields();
                let diagonal = values[start + (f.m_rowadr[dof] + f.m_rownnz[dof] - 1) as usize];
                if !diagonal.is_finite() || diagonal <= 0.0 {
                    return Err(TransferError::InvalidPivot {
                        world,
                        dof,
                        value: f64::from(diagonal),
                    });
                }
            }
        }
        crate::physics::check_device_values(&values, "sparse_mass_factor")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassFactorOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_mass_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        Ok(SparseMassFactorOutput {
            input: self.input.clone(),
            layout: self.output_layout,
            values: self.checked_values()?,
        })
    }
}

// Frozen smooth.py _qLD_acc/_qLDiag_div (71da24d), Apache-2.0.
// Copyright 2025 The Newton Developers; licenses/Apache-2.0.txt.
// Private ABI replaces array_t descriptors, not CSR or arithmetic semantics.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void sparse_mass_copy(const int* meta,const float* matrix,
 const float* unused,float* out,unsigned first,unsigned count,unsigned stride,unsigned work) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 unsigned nc=meta[1],n=meta[0];float* L=out+4+(unsigned long long)w*stride;
 for(unsigned j=0;j<nc;++j)L[j]=matrix[4+(unsigned long long)w*nc+j];
 for(unsigned i=0;i<n;++i)L[nc+i]=0.0f;
}
extern "C" __global__ void sparse_mass_update(const int* meta,const float* unused,
 const float* unused2,float* out,unsigned first,unsigned count,unsigned stride,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/count,u=first+tid%count,n=meta[0];
 const int* rownnz=meta+2;const int* rowadr=rownnz+n;const int* updates=rowadr+n;
 int i=updates[3*u],k=updates[3*u+1],ki=updates[3*u+2];
 float* L=out+4+(unsigned long long)w*stride;
 float tmp=__fdiv_rn(L[ki],L[rowadr[k]+rownnz[k]-1]);
 for(int j=0;j<rownnz[i];++j) {
   float value=__fmul_rn(L[rowadr[k]+j],tmp);
   float ignored;
   asm volatile("atom.global.add.f32 %0, [%1], %2;" : "=f"(ignored) : "l"(L+rowadr[i]+j),"f"(-value) : "memory");
 }
 L[ki]=tmp;
}
extern "C" __global__ void sparse_mass_diag(const int* meta,const float* unused,
 const float* unused2,float* out,unsigned first,unsigned count,unsigned stride,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned n=meta[0],nc=meta[1],w=tid/n,i=tid%n;
 const int* rownnz=meta+2;const int* rowadr=rownnz+n;
 float* L=out+4+(unsigned long long)w*stride;
 L[nc+i]=__fdiv_rn(1.0f,L[rowadr[i]+rownnz[i]-1]);
}
"#;

#[cfg(all(test, feature = "cuda-probe"))]
mod tests {
    use super::*;
    use crate::model::SparseMassFields;
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC; new sparse buffer fault purpose"]
    fn matrix_write_is_bit_isolated_and_both_guards_reject_corruption() {
        let refs: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sparse-mass-factor/native.json"
        )))
        .unwrap();
        let m = &refs["models"][2];
        let ints = |name: &str| {
            m[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect()
        };
        let input = SparseMassInput::new(SparseMassFields {
            dof_parentid: ints("dof_parentid"),
            m_rownnz: ints("m_rownnz"),
            m_rowadr: ints("m_rowadr"),
            m_colind: ints("m_colind"),
            tree_dofadr: ints("tree_dofadr"),
            tree_dofnum: ints("tree_dofnum"),
        })
        .unwrap();
        let nc = input.nc();
        let values = |case: usize| {
            m["cases"][case]["matrix"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect::<Vec<_>>()
        };
        let session = TransferSession::new(0).unwrap();
        let compact = SparseMassInput::new(SparseMassFields {
            dof_parentid: vec![-1],
            m_rownnz: vec![1],
            m_rowadr: vec![0],
            m_colind: vec![0],
            tree_dofadr: vec![0],
            tree_dofnum: vec![1],
        })
        .unwrap();
        assert!(SparseMassFactorPlan::new(&session, compact).is_err());
        let plan = SparseMassFactorPlan::new(&session, input).unwrap();
        let mut data = plan.create_data(3).unwrap();
        data.write_matrix(&values(0).repeat(3)).unwrap();
        let mut before = vec![0.0; 3 * nc + 8];
        data.matrix.read_range_into(0, &mut before).unwrap();
        data.write_world_matrix(1, &values(1)).unwrap();
        let mut after = vec![0.0; before.len()];
        data.matrix.read_range_into(0, &mut after).unwrap();
        for index in (0..4 + nc).chain(4 + 2 * nc..after.len()) {
            assert_eq!(before[index].to_bits(), after[index].to_bits());
        }
        assert!(
            before[4 + nc..4 + 2 * nc]
                .iter()
                .zip(&after[4 + nc..4 + 2 * nc])
                .any(|(a, b)| a.to_bits() != b.to_bits())
        );
        plan.update_factor(&mut data).unwrap();
        data.output.write_range(0, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        data.output.write_range(0, &[-131072.0]).unwrap();
        data.matrix.write_range(0, &[0.0]).unwrap();
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.matrix.write_range(0, &[-131072.0]).unwrap();
        plan.update_factor(&mut data).unwrap();
        assert!(data.readback().is_ok());
    }
}
