//! 冻结清单、输入、输出与位模式校验。

use crate::diagnostics::ProbeError;

use super::util::{artifact, checked_product, digest};
use super::*;

pub(super) fn validate_receipt(receipt: &FrozenWarpComReceipt) -> Result<(), ProbeError> {
    let valid = receipt.schema == 1
        && receipt.evidence_kind == "frozen-upstream-generated-com-cdof"
        && receipt.upstream_revision == REVISION
        && receipt.warp_version == WARP_VERSION
        && receipt.module == "mujoco_warp._src.smooth"
        && receipt.smooth_source_sha256 == SMOOTH_SHA256
        && receipt.types_source_sha256 == TYPES_SHA256
        && receipt.target_arch == 89
        && receipt.mode == "release"
        && !receipt.fast_math
        && receipt.fuse_fp
        && receipt
            .kernel_sequence
            .iter()
            .map(String::as_str)
            .eq(ENTRIES)
        && receipt.input_file == "input.json"
        && receipt.input_sha256 == INPUT_SHA256
        && receipt.output_file == "output.json"
        && receipt.output_sha256 == OUTPUT_SHA256
        && receipt.raw_bits_file == "raw-bits.json"
        && receipt.raw_bits_sha256 == RAW_SHA256
        && receipt.cu_file == "smooth.cu"
        && receipt.cu_sha256 == CU_SHA256
        && receipt.meta_file == "smooth.meta"
        && receipt.meta_sha256 == META_SHA256
        && receipt.ptx_file == "smooth.ptx"
        && receipt.ptx_sha256 == PTX_SHA256
        && receipt.generator_file == "generate.py"
        && receipt.generator_sha256 == GENERATOR_SHA256
        && receipt.authoring_file == "authoring.json"
        && receipt.authoring_sha256 == AUTHORING_SHA256
        && receipt.generation_log == "generation.log"
        && receipt.generation_log_sha256 == GENERATION_LOG_SHA256
        && receipt.first_attempt_log == "generation-attempt-1.log"
        && receipt.first_attempt_log_sha256 == FIRST_LOG_SHA256
        && receipt.generation_executions_file == "generation-executions.json"
        && receipt.generation_executions_sha256 == EXECUTIONS_SHA256
        && receipt.python_version == "3.12.13"
        && receipt.python_executable_sha256 == PYTHON_SHA256
        && receipt.warp_dll_sha256 == WARP_DLL_SHA256
        && receipt.cuda_toolkit == "12.9"
        && receipt.cuda_driver_api_version == "13.2"
        && receipt.nvidia_driver_version == "596.36"
        && receipt.device == "NVIDIA GeForce RTX 4070 Ti SUPER"
        && receipt.compiler_options_provenance == "pinned_static_defaults_not_runtime_capture"
        && receipt.generation_attempts == 2
        && !receipt.candidate_intermediate_inputs
        && receipt.static_kinematics_inputs
        && !receipt.full_qpos_chain_independent
        && receipt.authoring_only
        && !receipt.product_python_dependency;
    if valid {
        Ok(())
    } else {
        Err(artifact("receipt", "frozen identity mismatch"))
    }
}

pub(super) fn validate_input(input: &FrozenWarpComInput) -> Result<(), ProbeError> {
    if input.schema != 1
        || input.scope != "gravity-upstream-independent-com-cdof"
        || input.upstream_revision != REVISION
        || input.warp_version != WARP_VERSION
        || input.worlds != 7
        || input.nbody != 5
        || input.njnt != 4
        || input.nv != 4
        || input.candidate_intermediate_inputs
    {
        return Err(artifact("input", "frozen identity mismatch"));
    }
    let exact = [
        (input.body_parentid.len(), input.nbody),
        (input.body_rootid.len(), input.nbody),
        (input.jnt_type.len(), input.njnt),
        (input.jnt_dofadr.len(), input.njnt),
        (input.jnt_bodyid.len(), input.njnt),
        (input.xipos_row.len(), checked_product(input.nbody, 3)?),
        (input.xmat_row.len(), checked_product(input.nbody, 9)?),
        (input.xanchor_row.len(), checked_product(input.njnt, 3)?),
        (input.xaxis_row.len(), checked_product(input.njnt, 3)?),
    ];
    if exact.iter().any(|&(actual, expected)| actual != expected)
        || input.body_mass.is_empty()
        || input.body_mass.len() != input.body_subtreemass.len()
        || input
            .body_mass
            .iter()
            .chain(&input.body_subtreemass)
            .any(|row| row.len() != input.nbody)
    {
        return Err(artifact("input", "field length mismatch"));
    }
    if input
        .body_mass
        .iter()
        .chain(&input.body_subtreemass)
        .flat_map(|row| row.iter())
        .chain(&input.xipos_row)
        .chain(&input.xmat_row)
        .chain(&input.xanchor_row)
        .chain(&input.xaxis_row)
        .any(|value| !value.is_finite())
    {
        return Err(artifact("input", "non-finite value"));
    }
    if input.body_parentid[0] != 0
        || input.body_rootid[0] != 0
        || input
            .body_parentid
            .iter()
            .chain(&input.body_rootid)
            .any(|&value| value < 0 || value as usize >= input.nbody)
        || input
            .jnt_bodyid
            .iter()
            .any(|&value| value <= 0 || value as usize >= input.nbody)
    {
        return Err(artifact("input", "topology value mismatch"));
    }
    let mut bodies = input
        .body_tree_levels
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    bodies.sort_unstable();
    if bodies != (1..input.nbody as i32).collect::<Vec<_>>() {
        return Err(artifact("input", "body tree mismatch"));
    }
    for ((&joint_type, &dof_address), &body) in input
        .jnt_type
        .iter()
        .zip(&input.jnt_dofadr)
        .zip(&input.jnt_bodyid)
    {
        let width = match joint_type {
            0 => 6,
            1 => 3,
            2 | 3 => 1,
            _ => return Err(artifact("input", "joint type mismatch")),
        };
        if dof_address < 0 || (dof_address as usize).saturating_add(width) > input.nv || body <= 0 {
            return Err(artifact("input", "joint mapping mismatch"));
        }
    }
    Ok(())
}

pub(super) fn validate_output(
    output: &FrozenOutputFile,
    input: &FrozenWarpComInput,
) -> Result<(), ProbeError> {
    if output.schema != 1
        || output.evidence_kind != "frozen-upstream-generated-com-cdof"
        || output.upstream_revision != REVISION
        || output.warp_version != WARP_VERSION
        || output.input_sha256 != INPUT_SHA256
        || output.worlds != input.worlds
        || output.nbody != input.nbody
        || output.njnt != input.njnt
        || output.nv != input.nv
        || output.candidate_intermediate_inputs
        || output.subtree_com.len() != checked_product(input.worlds, input.nbody * 3)?
        || output.cdof.len() != checked_product(input.worlds, input.nv * 6)?
    {
        return Err(artifact("output", "shape or identity mismatch"));
    }
    Ok(())
}

pub(super) fn validate_raw(
    raw: &RawBitsFile,
    output: &FrozenOutputFile,
) -> Result<(Vec<u32>, Vec<u32>), ProbeError> {
    if raw.schema != 1
        || raw.provenance != "direct_upstream_device_readback"
        || raw.upstream_revision != REVISION
        || raw.fields.len() != 2
    {
        return Err(artifact("raw bits", "identity mismatch"));
    }
    let subtree = validate_raw_field(&raw.fields[0], "subtree_com", &output.subtree_com)?;
    let cdof = validate_raw_field(&raw.fields[1], "cdof", &output.cdof)?;
    Ok((subtree, cdof))
}

pub(super) fn validate_raw_field(
    field: &RawField,
    expected_name: &str,
    values: &[f32],
) -> Result<Vec<u32>, ProbeError> {
    if field.field != expected_name
        || field.count != values.len()
        || field.bits.len() != values.len()
    {
        return Err(artifact("raw bits", "field shape mismatch"));
    }
    let bits = field
        .bits
        .iter()
        .map(|value| {
            u32::from_str_radix(value, 16).map_err(|_| artifact("raw bits", "invalid hex"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if bits
        .iter()
        .zip(values)
        .any(|(&actual, expected)| actual != expected.to_bits())
    {
        return Err(artifact("raw bits", "value mismatch"));
    }
    let bytes = bits
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>();
    if digest(&bytes) != field.little_endian_sha256 {
        return Err(artifact("raw bits", "byte hash mismatch"));
    }
    Ok(bits)
}
