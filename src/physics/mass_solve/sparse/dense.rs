//! 非紧凑块的GPU稠密化输入；不计算或发布Cholesky因子。
//! 只复制f32位型，结构缺项写入正零。
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, SparseMassInput};
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

mod small;
pub use small::{SparseMassSmallData, SparseMassSmallOutput, SparseMassSmallPlan};

// 子模块：纯移动自原单文件，行为不变。
mod data;
mod gather_meta;
mod kernels;
mod plan;
mod validation_meta;
#[cfg(feature = "cuda-probe")]
use kernels::{DenseGatherKernel, DenseGatherMetadataBuffer, DenseValidationKernel};
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "dense/tests/gpu_gather.rs"]
mod tests_gpu_gather;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "dense/tests/gpu_guards.rs"]
mod tests_gpu_guards;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "dense/tests/gpu_validation.rs"]
mod tests_gpu_validation;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "dense/tests/plans.rs"]
mod tests_plans;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "dense/tests/support.rs"]
mod tests_support;

pub struct SparseMassDensePlan {
    input: Arc<SparseMassInput>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: DenseGatherMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    validator: DenseValidationKernel,
    #[cfg(feature = "cuda-probe")]
    gather: DenseGatherKernel,
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
