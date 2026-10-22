//! 常驻分解与多右端项严格辅助。
//! 完整G03仍待验收。

#[cfg(feature = "cuda-probe")]
use self::kernels::{ResidentMassFactorKernel, ResidentMassSubstituteKernel};
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::ResidentMassMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::MassFactorBlock;
use super::MassSolveLayout;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::{ProbeError, TransferError};
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

/// 独立保留CRB、因子与回代内核。
pub struct MassSolvePlan {
    matrix: MassMatrixPlan,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    blocks: Vec<MassFactorBlock>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: ResidentMassMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    factor: ResidentMassFactorKernel,
    #[cfg(feature = "cuda-probe")]
    substitute: ResidentMassSubstituteKernel,
}

/// RHS写入保留因子，只废弃解。
pub struct MassSolveData {
    matrix: MassMatrixData,
    layout: MassSolveLayout,
    #[cfg(feature = "cuda-probe")]
    factor_layout: MassSolveLayout,
    factor_ready: bool,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    blocks: Vec<MassFactorBlock>,
    #[cfg(feature = "cuda-probe")]
    rhs: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    factor: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

// Retain the existing frozen-derived f32 arithmetic verbatim.
// Copyright 2025 The Newton Developers. Apache-2.0.
// See licenses/Apache-2.0.txt for license and warranty terms.
#[cfg(feature = "cuda-probe")]
fn sources() -> Result<[String; 2], TransferError> {
    split_sources(super::MASS_SOLVE_CUDA)
}

#[cfg(feature = "cuda-probe")]
fn split_sources(source: &str) -> Result<[String; 2], TransferError> {
    const CUT: &str = "  if(n==0) return;\n";
    const ENTRY: &str = "void rigid_mass_solve(";
    const INIT: &str = "  for(unsigned i=0;i<n;i++) inv[i]=0.0f;";
    // Windows检出可能改换行；先统一再核对唯一锚点。
    let source = source.replace("\r\n", "\n");
    if source.matches(CUT).count() != 1
        || source.matches(ENTRY).count() != 1
        || source.matches(INIT).count() != 1
    {
        return Err(ProbeError::InvalidArgument("mass_solve_source_split").into());
    }
    let (factor, solve) = source.split_once(CUT).unwrap();
    let factor = format!(
        "{}\n}}\n",
        factor.replace(ENTRY, "void resident_mass_factor(").replace(
            INIT,
            &format!("{INIT}\n  for(unsigned i=0;i<n;i++) x[i]=0.0f;")
        )
    );
    // Factors use the existing single-RHS layout. Its unused x slot is zero.
    // Solutions have their own variable-RHS layout and never overwrite factors.
    let prefix = r#"
extern "C" __global__ void resident_mass_substitute(const int* blocks,const float* rhs,
    const float* state,float* output,unsigned nb,unsigned n,unsigned nr,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  const float* saved=state+4+(unsigned long long)w*(3ull+n*n+2ull*n);
  float* status=output+4+(unsigned long long)w*(3ull+n*n+n+nr*n);
  for(unsigned i=0;i<3+n*n+n;++i) status[i]=saved[i];
  const float* ld=saved+3;const float* inv=ld+n*n;float* x=status+3+n*n+n;
  if(n==0) return;
"#;
    let old_rhs = "  const float* b=rhs+(unsigned long long)w*nr*n;";
    if solve.matches(old_rhs).count() != 1 {
        return Err(ProbeError::InvalidArgument("mass_solve_rhs_split").into());
    }
    let solve = format!(
        "{prefix}{}",
        solve.replace(
            old_rhs,
            "  const float* b=rhs+4+(unsigned long long)w*nr*n;"
        )
    )
    .replace("\r\n", "\n");
    Ok([factor, solve])
}

#[path = "resident/data.rs"]
mod data;
#[cfg(feature = "cuda-probe")]
#[path = "resident/kernels.rs"]
mod kernels;
#[cfg(feature = "cuda-probe")]
#[path = "resident/launcher_abi.rs"]
mod launcher_abi;
#[cfg(feature = "cuda-probe")]
#[path = "resident/launcher_meta.rs"]
mod launcher_meta;
#[cfg(feature = "cuda-probe")]
#[path = "resident/launcher_specs.rs"]
mod launcher_specs;
#[cfg(feature = "cuda-probe")]
#[path = "resident/metadata_buffer.rs"]
mod metadata_buffer;
#[path = "resident/plan.rs"]
mod plan;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "resident/tests.rs"]
mod tests;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "resident/tests/launcher.rs"]
mod tests_launcher;
