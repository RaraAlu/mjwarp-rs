//! 真正CSR分层分解；不调用稠密辅助。
//! 子模块提供融合回代；非紧凑混合块仍待实现。
#[cfg(feature = "cuda-probe")]
use self::kernels::SparseMassCopyKernel;
#[cfg(feature = "cuda-probe")]
use self::kernels::SparseMassDiagKernel;
#[cfg(feature = "cuda-probe")]
use self::kernels::SparseMassUpdateKernel;
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::SparseMassCopyMetadataBuffer;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, SparseMassInput, SparseMassRoute};
use crate::runtime::TransferSession;
#[cfg(any(feature = "cuda-probe", test))]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

use std::sync::Arc;

mod dense;
mod mixed;
pub use dense::{
    SparseMassDenseData, SparseMassDenseOutput, SparseMassDensePlan, SparseMassSmallData,
    SparseMassSmallOutput, SparseMassSmallPlan,
};
pub use mixed::{
    SparseMassMixedFactorData, SparseMassMixedFactorOutput, SparseMassMixedFactorPlan,
    SparseMassMixedFactorWorld, SparseMassMixedResidentData, SparseMassMixedResidentPlan,
    SparseMassMixedSolveData, SparseMassMixedSolveOutput, SparseMassMixedSolvePlan,
    SparseMassMixedSolveWorld,
};

mod resident;
pub use resident::{
    SparseMassData, SparseMassOutput, SparseMassPlan, SparseMassSolveData, SparseMassSolveOutput,
    SparseMassSolvePlan, SparseMassSolveWorld, SparseMassWorld,
};

// 子模块：纯移动自原单文件，行为不变。
mod factor;
mod kernels;
mod launcher_abi;
mod launcher_diag;
mod launcher_specs;
mod metadata_buffer;
#[cfg(test)]
#[path = "sparse/tests/plans.rs"]
mod tests_plans;
#[cfg(test)]
#[path = "sparse/tests/runs.rs"]
mod tests_runs;
#[cfg(test)]
#[path = "sparse/tests/support.rs"]
mod tests_support;

pub struct SparseMassFactorPlan {
    input: Arc<SparseMassInput>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: SparseMassCopyMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    copy: SparseMassCopyKernel,
    #[cfg(feature = "cuda-probe")]
    update: SparseMassUpdateKernel,
    #[cfg(feature = "cuda-probe")]
    diag: SparseMassDiagKernel,
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
