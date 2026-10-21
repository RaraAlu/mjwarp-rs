//! 冻结回执与输入形状核验。

use super::{
    CODEGEN_SHA256, CU_SHA256, ENTRY, FrozenWarpGravityInput, FrozenWarpGravityReceipt,
    GENERATION_EXECUTION_SHA256, GENERATION_LOG_SHA256, GENERATOR_SHA256, META_SHA256,
    PASSIVE_SHA256, PTX_SHA256, PYTHON_SHA256, REVISION, SUPPORT_SHA256, WARP_DLL_SHA256,
    WARP_VERSION, artifact, checked_product,
};
use crate::diagnostics::ProbeError;

pub(super) fn validate_receipt(receipt: &FrozenWarpGravityReceipt) -> Result<(), ProbeError> {
    let valid = receipt.schema == 1
        && receipt.evidence_kind == "frozen-generated"
        && receipt.upstream_revision == REVISION
        && receipt.warp_version == WARP_VERSION
        && receipt.module == "mujoco_warp._src.passive"
        && receipt.passive_source_sha256 == PASSIVE_SHA256
        && receipt.support_source_sha256 == SUPPORT_SHA256
        && receipt.codegen_source_sha256 == CODEGEN_SHA256
        && receipt.target_arch == 89
        && receipt.mode == "release"
        && !receipt.fast_math
        && receipt.fuse_fp
        && receipt.entry == ENTRY
        && receipt.launch_rank == 3
        && receipt.array_arguments == 11
        && receipt.python_version == "3.12.13"
        && receipt.python_executable_sha256 == PYTHON_SHA256
        && receipt.generator_file == "generate.py"
        && receipt.generator_sha256 == GENERATOR_SHA256
        && receipt.warp_dll_sha256 == WARP_DLL_SHA256
        && receipt.cuda_toolkit == "12.9"
        && receipt.ptx_compiler == "12.9.86"
        && receipt.generation_invocation_id == "exec-9a825267"
        && receipt.generation_log == "generation.log"
        && receipt.generation_log_sha256 == GENERATION_LOG_SHA256
        && receipt.generation_execution_file == "generation-execution.json"
        && receipt.generation_execution_sha256 == GENERATION_EXECUTION_SHA256
        && receipt.compiler_provenance
            == "reconstructed_from_stdout_ptx_and_pinned_source_defaults"
        && receipt.cu_file == "module.cu"
        && receipt.ptx_file == "module.ptx"
        && receipt.meta_file == "module.meta"
        && receipt.cu_sha256 == CU_SHA256
        && receipt.ptx_sha256 == PTX_SHA256
        && receipt.meta_sha256 == META_SHA256
        && receipt.authoring_only
        && !receipt.product_python_dependency;
    if valid {
        Ok(())
    } else {
        Err(artifact("receipt", "frozen identity mismatch"))
    }
}

pub(super) fn validate_input(input: &FrozenWarpGravityInput) -> Result<(), ProbeError> {
    if input.worlds == 0 || input.nbody < 2 || input.nv == 0 {
        return Err(artifact("input", "zero dimension"));
    }
    let body_dof = checked_product(input.nbody, input.nv)?;
    let world_body = checked_product(input.worlds, input.nbody)?;
    let world_dof = checked_product(input.worlds, input.nv)?;
    let exact = [
        (input.body_parentid.len(), input.nbody),
        (input.body_rootid.len(), input.nbody),
        (input.dof_bodyid.len(), input.nv),
        (input.body_isdofancestor.len(), body_dof),
        (input.xipos.len(), checked_product(world_body, 3)?),
        (input.subtree_com.len(), checked_product(world_body, 3)?),
        (input.cdof.len(), checked_product(world_dof, 6)?),
    ];
    if exact.iter().any(|&(actual, expected)| actual != expected)
        || input.opt_gravity.is_empty()
        || !input.opt_gravity.len().is_multiple_of(3)
        || input.body_mass.is_empty()
        || !input.body_mass.len().is_multiple_of(input.nbody)
        || input.body_gravcomp.is_empty()
        || !input.body_gravcomp.len().is_multiple_of(input.nbody)
    {
        return Err(artifact("input", "field length mismatch"));
    }
    for values in [
        &input.opt_gravity,
        &input.body_mass,
        &input.body_gravcomp,
        &input.xipos,
        &input.subtree_com,
        &input.cdof,
    ] {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(artifact("input", "non-finite value"));
        }
    }
    if input
        .body_parentid
        .iter()
        .chain(&input.body_rootid)
        .any(|&value| value < 0 || value as usize >= input.nbody)
        || input
            .dof_bodyid
            .iter()
            .any(|&value| value < 0 || value as usize >= input.nbody)
        || input
            .body_isdofancestor
            .iter()
            .any(|&value| value != 0 && value != 1)
    {
        return Err(artifact("input", "topology value mismatch"));
    }
    Ok(())
}
