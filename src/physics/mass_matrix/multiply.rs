//! 常驻质量矩阵乘法；不执行forward、分解或回代。
mod borrowed;
mod cuda;
mod data;
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
#[cfg(test)]
#[path = "multiply/tests/launcher.rs"]
mod tests_launcher;

pub use borrowed::schedule_energy_chain;
pub use borrowed::{
    MassMatrixEnergyData, MassMatrixEnergyPlan, MassMatrixEnergySnapshot, MassMatrixEnergyView,
};
pub use borrowed::{MassMatrixMultiplyData, MassMatrixMultiplyPlan, MassMatrixMultiplyView};

#[cfg(feature = "cuda-probe")]
use self::kernels::{MassMultiplyComputeKernel, MassMultiplyValidateKernel};
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::MassMultiplyMetadataBuffer;
use crate::diagnostics::InputError;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
use crate::model::{BatchLayout, SparseMassInput};
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
use std::sync::Arc;

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
    validate: MassMultiplyValidateKernel,
    #[cfg(feature = "cuda-probe")]
    compute: MassMultiplyComputeKernel,
}
pub struct MassMultiplyData {
    input: Arc<SparseMassInput>,
    format: MassMultiplyFormat,
    matrix_layout: BatchLayout,
    vector_layout: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    metadata: MassMultiplyMetadataBuffer,
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
