//! 回执结构校验。

use super::support::artifact;
use super::*;

pub(super) fn validate_receipt(receipt: &FrozenWarpKinematicsReceipt) -> Result<(), ProbeError> {
    let valid = receipt.schema == 1
        && receipt.evidence_kind == "frozen-upstream-generated-qpos-kinematics-com-cdof"
        && receipt.upstream_revision == REVISION
        && receipt.warp_version == WARP_VERSION
        && receipt.module == "mujoco_warp._src.smooth"
        && receipt.smooth_source_sha256 == SMOOTH_SHA256
        && receipt.types_source_sha256 == TYPES_SHA256
        && receipt.target_arch == 89
        && receipt.mode == "release"
        && !receipt.fast_math
        && receipt.fuse_fp
        && receipt.compiler_options_provenance == "pinned_static_defaults_not_runtime_capture"
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
        && receipt.generation_executions_file == "generation-executions.json"
        && receipt.generation_executions_sha256 == EXECUTIONS_SHA256
        && receipt.prevalidation_readback_file == "attempt-3/raw-readback.json"
        && receipt.prevalidation_readback_sha256 == RAW_SHA256
        && receipt.prevalidation_state_file == "attempt-3/state.json"
        && receipt.prevalidation_state_sha256
            == "29fbc75e4ea3aceb3015b62497b8f5d340fc05f0c81e663c00a81f43b6684e75"
        && receipt.python_version == "3.12.13"
        && receipt.python_executable_sha256 == PYTHON_SHA256
        && receipt.warp_dll_sha256 == WARP_DLL_SHA256
        && receipt.cuda_toolkit == "12.9"
        && receipt.cuda_driver_api_version == "13.2"
        && receipt.nvidia_driver_version == "596.36"
        && receipt.device == "NVIDIA GeForce RTX 4070 Ti SUPER"
        && receipt.generation_attempts == 3
        && !receipt.first_attempt_readback_preserved
        && receipt.second_attempt_output_preserved
        && receipt.third_attempt_prevalidation_readback_preserved
        && receipt.qpos_generated_kinematics
        && receipt.zero_qpos_only
        && !receipt.nonzero_rotation_coverage
        && receipt.legacy_static_fields_only_as_expected_assertions
        && !receipt.candidate_kinematics_inputs
        && receipt.authoring_only
        && !receipt.product_python_dependency;
    if valid {
        Ok(())
    } else {
        Err(artifact("receipt", "frozen identity mismatch"))
    }
}
