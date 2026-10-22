//! 直接消费常驻CRB设备矩阵，不复制M。
//! 只读视图同时借用矩阵与向量数据。
mod cuda;
mod data;
mod device_energy;
#[cfg(feature = "cuda-probe")]
mod kernels;
#[cfg(feature = "cuda-probe")]
mod launcher_abi;
#[cfg(feature = "cuda-probe")]
mod launcher_meta;
#[cfg(feature = "cuda-probe")]
mod launcher_specs;
#[cfg(feature = "cuda-probe")]
mod metadata_buffer;
mod plan;
#[cfg(test)]
mod tests;
mod view;

#[cfg(feature = "cuda-probe")]
use self::kernels::{BorrowedMassComputeKernel, BorrowedMassValidateKernel};
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::BorrowedMassMetadataBuffer;
use super::{checked, layouts, metadata};
use crate::diagnostics::InputError;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
use crate::model::{BatchLayout, SparseMassInput};
use crate::physics::{MassMatrixData, MassMatrixPlan};
#[cfg(all(feature = "cuda-probe", test))]
use crate::runtime::launcher::DeviceSpan;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
pub use device_energy::schedule_energy_chain;
pub use device_energy::{
    MassMatrixEnergyData, MassMatrixEnergyPlan, MassMatrixEnergySnapshot, MassMatrixEnergyView,
};
use std::sync::Arc;

#[derive(Debug)]
struct Spec {
    nbody: usize,
    nv: usize,
    meta: Vec<i32>,
}
fn packed(nbody: usize, parents: &[i32], input: &SparseMassInput) -> Result<Spec, InputError> {
    let n = input.nv();
    if parents != input.fields().dof_parentid || n == 0 || nbody == 0 {
        return Err(InputError::InvalidTopology {
            field: "borrowed_mul_m_topology",
            index: 0,
            reason: "source_and_CSR_parents_must_match",
        });
    }
    let stride = n
        .checked_mul(n)
        .and_then(|v| nbody.checked_mul(10).and_then(|b| v.checked_add(b)))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "borrowed_mul_m_stride",
        })?;
    layouts(n, stride, 1)?;
    // A parent match alone permits compact/subset CSR rows. The resident CRB
    // producer writes every ancestor coupling; omitting one would consume a
    // different matrix. This strict borrow accepts only complete ancestry.
    let fields = input.fields();
    for row in 0..n {
        let start = fields.m_rowadr[row] as usize;
        let end = start + fields.m_rownnz[row] as usize;
        let mut ancestor = row as i32;
        for &col in fields.m_colind[start..end].iter().rev() {
            if col != ancestor {
                return Err(InputError::InvalidTopology {
                    field: "borrowed_mul_m_topology",
                    index: row,
                    reason: "source_dense_requires_full_ancestor_CSR",
                });
            }
            ancestor = parents[ancestor as usize];
        }
        if ancestor != -1 {
            return Err(InputError::InvalidTopology {
                field: "borrowed_mul_m_topology",
                index: row,
                reason: "source_dense_requires_full_ancestor_CSR",
            });
        }
    }
    let mut m = metadata(input)?;
    let count = m[n] as usize;
    // Preserve original mirrored-row append order. Only addresses change:
    // each CSR slot now reads its actual lower-triangle dense CRB/M cell.
    for row in 0..n {
        for k in m[row] as usize..m[row + 1] as usize {
            let col = m[n + 1 + k] as usize;
            m[n + 1 + count + k] = (row.max(col) * n + row.min(col)) as i32;
        }
    }
    let mut meta = vec![n as i32, count as i32];
    meta.extend(m);
    Ok(Spec { nbody, nv: n, meta })
}
pub struct MassMatrixMultiplyPlan<'plan> {
    matrix: &'plan MassMatrixPlan,
    spec: Arc<Spec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    validate: BorrowedMassValidateKernel,
    #[cfg(feature = "cuda-probe")]
    compute: BorrowedMassComputeKernel,
}
pub struct MassMatrixMultiplyData {
    spec: Arc<Spec>,
    vector_layout: BatchLayout,
    #[cfg(feature = "cuda-probe")]
    metadata: BorrowedMassMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    vector: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
/// 视图阻止矩阵或向量写入；快照只保存历史结果。
pub struct MassMatrixMultiplyView<'borrow> {
    source: &'borrow MassMatrixData,
    data: &'borrow MassMatrixMultiplyData,
    computed: Vec<bool>,
}
#[cfg(feature = "cuda-probe")]
fn read(
    b: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
    b.read_range_into(0, &mut v)?;
    crate::physics::check_device_values(&v, field)?;
    Ok(v)
}
