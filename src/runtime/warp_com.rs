//! 冻结Warp质心中间量导入器。
//! 它拒绝候选链反馈的中间量。
mod artifact;
mod format;
#[cfg(test)]
mod tests;
mod util;
mod validate;

use crate::runtime::launcher::FrozenPtx;
use serde::Deserialize;

const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
const SMOOTH_SHA256: &str = "d0952a647a55b4c5f3e89f7224c4842d15c504fe40bafb2fd8bd9b9bd8ef8f4b";
const TYPES_SHA256: &str = "7970e04d40dd15b33171dd1558c369cf45478b6ffdb31d8a805aec93702ec717";
const INPUT_SHA256: &str = "5136b09e1d727b7799d527ebed42b1f7ad05b0a0040928a78a69137b2400ec0c";
const OUTPUT_SHA256: &str = "7abb55179941ad74e26194614967791380b590fe59b88e2bfb6a51bce1b06253";
const RAW_SHA256: &str = "2260e407503d5dc661f9566660b8e368f0228f0a9ad6c7a9f6ccec81b188e8d0";
const CU_SHA256: &str = "844dca14a5932827729313a2500f343bdff950b8cdb6170aed1c5973e0ff290e";
const META_SHA256: &str = "2bb1bcdb12cbcce6143af26da5798c047ed628d4817c4b76358fc646db0e3cc8";
const PTX_SHA256: &str = "4179acd01fe8e7064fa97566cfe4158dd9ad4b89761dcab0da88001d9b521f57";
const GENERATOR_SHA256: &str = "d03c81b8df11fe56d45b78dfed820229c6211f2ec8d405870e51e622d2e540a8";
const AUTHORING_SHA256: &str = "750bef19ce09aaa875bddb282d734b8807d998c2a7764c6a8f567b4688c7250e";
const GENERATION_LOG_SHA256: &str =
    "28a199655053ece2213402bc6aff8bb0ea2709a80128bd5aac5b50c7ce1b8ed7";
const FIRST_LOG_SHA256: &str = "1a8eb689817412f9960e08de1e8e86e87fdc6a1b066f2d7ee2bee07ca07767aa";
const EXECUTIONS_SHA256: &str = "6a1abf5f8618b1a7c048794c5d531739ea3acc68831064631047c67bd6086b46";
const PYTHON_SHA256: &str = "feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77";
const WARP_DLL_SHA256: &str = "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6";

const INIT_ENTRY: &str = "_subtree_com_init_002b02fd_cuda_kernel_forward";
const ACC_ENTRY: &str = "_subtree_com_acc_350f0560_cuda_kernel_forward";
const DIV_ENTRY: &str = "_subtree_div_0ef57c26_cuda_kernel_forward";
const CDOF_ENTRY: &str = "_cdof_3e73eb78_cuda_kernel_forward";
const ENTRIES: [&str; 4] = [INIT_ENTRY, ACC_ENTRY, DIV_ENTRY, CDOF_ENTRY];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenWarpComReceipt {
    pub schema: u32,
    pub evidence_kind: String,
    pub upstream_revision: String,
    pub warp_version: String,
    pub module: String,
    pub smooth_source_sha256: String,
    pub types_source_sha256: String,
    pub target_arch: u32,
    pub mode: String,
    pub fast_math: bool,
    pub fuse_fp: bool,
    pub kernel_sequence: Vec<String>,
    pub input_file: String,
    pub input_sha256: String,
    pub output_file: String,
    pub output_sha256: String,
    pub raw_bits_file: String,
    pub raw_bits_sha256: String,
    pub cu_file: String,
    pub cu_sha256: String,
    pub meta_file: String,
    pub meta_sha256: String,
    pub ptx_file: String,
    pub ptx_sha256: String,
    pub generator_file: String,
    pub generator_sha256: String,
    pub authoring_file: String,
    pub authoring_sha256: String,
    pub generation_log: String,
    pub generation_log_sha256: String,
    pub first_attempt_log: String,
    pub first_attempt_log_sha256: String,
    pub generation_executions_file: String,
    pub generation_executions_sha256: String,
    pub python_version: String,
    pub python_executable_sha256: String,
    pub warp_dll_sha256: String,
    pub cuda_toolkit: String,
    pub cuda_driver_api_version: String,
    pub nvidia_driver_version: String,
    pub device: String,
    pub compiler_options_provenance: String,
    pub generation_attempts: usize,
    pub candidate_intermediate_inputs: bool,
    pub static_kinematics_inputs: bool,
    pub full_qpos_chain_independent: bool,
    pub authoring_only: bool,
    pub product_python_dependency: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenWarpComInput {
    pub schema: u32,
    pub scope: String,
    pub upstream_revision: String,
    pub warp_version: String,
    pub worlds: usize,
    pub nbody: usize,
    pub njnt: usize,
    pub nv: usize,
    pub body_parentid: Vec<i32>,
    pub body_rootid: Vec<i32>,
    pub body_mass: Vec<Vec<f32>>,
    pub body_subtreemass: Vec<Vec<f32>>,
    pub body_tree_levels: Vec<Vec<i32>>,
    pub jnt_type: Vec<i32>,
    pub jnt_dofadr: Vec<i32>,
    pub jnt_bodyid: Vec<i32>,
    pub xipos_row: Vec<f32>,
    pub xmat_row: Vec<f32>,
    pub xanchor_row: Vec<f32>,
    pub xaxis_row: Vec<f32>,
    pub candidate_intermediate_inputs: bool,
}

#[derive(Clone, Debug)]
pub struct FrozenWarpComOutput {
    pub subtree_com: Vec<f32>,
    pub cdof: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct FrozenWarpComReadback {
    subtree_com: Vec<f32>,
    cdof: Vec<f32>,
    subtree_com_bits: Vec<u32>,
    cdof_bits: Vec<u32>,
}

pub struct FrozenWarpComArtifact {
    receipt: FrozenWarpComReceipt,
    input: FrozenWarpComInput,
    readback: FrozenWarpComReadback,
    frozen: FrozenPtx,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenOutputFile {
    schema: u32,
    evidence_kind: String,
    upstream_revision: String,
    warp_version: String,
    input_sha256: String,
    worlds: usize,
    nbody: usize,
    njnt: usize,
    nv: usize,
    subtree_com: Vec<f32>,
    cdof: Vec<f32>,
    candidate_intermediate_inputs: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBitsFile {
    schema: u32,
    provenance: String,
    upstream_revision: String,
    fields: Vec<RawField>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawField {
    field: String,
    count: usize,
    little_endian_sha256: String,
    bits: Vec<String>,
}
