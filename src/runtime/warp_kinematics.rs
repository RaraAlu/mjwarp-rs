//! 冻结Warp刚体运动学链导入器。
//! 它从固定模型与qpos生成重力前置量。

use crate::diagnostics::ProbeError;
use crate::runtime::launcher::{
    FrozenPtx, KernelHandle, LaunchBounds2, LauncherError, ModuleCache, WarpArg, WarpBounds,
    WarpLaunch,
};
use cudarc::driver::{DevicePtr, DevicePtrMut, DriverError};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
const SMOOTH_SHA256: &str = "d0952a647a55b4c5f3e89f7224c4842d15c504fe40bafb2fd8bd9b9bd8ef8f4b";
const TYPES_SHA256: &str = "7970e04d40dd15b33171dd1558c369cf45478b6ffdb31d8a805aec93702ec717";
const INPUT_SHA256: &str = "bda832d923c105a3a84a626f2324d9a7496fafa767b11f07fe4477ec8bbfb57e";
const OUTPUT_SHA256: &str = "d7521946a55b7e13d4e20472cbf7a0ae5448c6e8910e1592881a270b0ad24ee1";
const RAW_SHA256: &str = "06751c4b634171623cd9539b617892136f3d20b6193e5ce168c327e743ac1e99";
const CU_SHA256: &str = "844dca14a5932827729313a2500f343bdff950b8cdb6170aed1c5973e0ff290e";
const META_SHA256: &str = "2bb1bcdb12cbcce6143af26da5798c047ed628d4817c4b76358fc646db0e3cc8";
const PTX_SHA256: &str = "4179acd01fe8e7064fa97566cfe4158dd9ad4b89761dcab0da88001d9b521f57";
const GENERATOR_SHA256: &str = "7366b81dba2133f1e09eb8c2cbf27121af271edbb8295bc3f3c4b70c6793bd05";
const AUTHORING_SHA256: &str = "986ad2b5592ceaeb8ba880c5f7da68b286c4cc9f9c48153f9db2411192e3fb9f";
const EXECUTIONS_SHA256: &str = "42980d42be746a8917a13d377160798c55660455cef1739f7bfb6ea6039fbb06";
const PYTHON_SHA256: &str = "feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77";
const WARP_DLL_SHA256: &str = "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6";

const KIN_ENTRY: &str = "_kinematics_branch_95f8028e_cuda_kernel_forward";
const MATRIX_ENTRY: &str = "_compute_body_matrices_27760750_cuda_kernel_forward";
const INERTIAL_ENTRY: &str = "_compute_body_inertial_frames_d3bdd81a_cuda_kernel_forward";
const INIT_ENTRY: &str = "_subtree_com_init_002b02fd_cuda_kernel_forward";
const ACC_ENTRY: &str = "_subtree_com_acc_350f0560_cuda_kernel_forward";
const DIV_ENTRY: &str = "_subtree_div_0ef57c26_cuda_kernel_forward";
const CDOF_ENTRY: &str = "_cdof_3e73eb78_cuda_kernel_forward";
const ENTRIES: [&str; 7] = [
    KIN_ENTRY,
    MATRIX_ENTRY,
    INERTIAL_ENTRY,
    INIT_ENTRY,
    ACC_ENTRY,
    DIV_ENTRY,
    CDOF_ENTRY,
];

const ATTEMPT_FILES: [(&str, &str); 32] = [
    (
        "attempt-1/generate.py",
        "fcff2ce6ad14438cbdea29a1d511e764eda94bd13b514cabf29e98d44d77a589",
    ),
    (
        "attempt-1/generation.log",
        "088aeaf1d79eb86060f03c2bfb87d6e3e68f8c0859b0aafa4d36bd9d01d402bf",
    ),
    (
        "attempt-1/input.json",
        "0e65b54cb00dabc97314a6acb9f2dd6c14164639644d2f09af65113fb2977c74",
    ),
    ("attempt-1/smooth.cu", CU_SHA256),
    ("attempt-1/smooth.meta", META_SHA256),
    ("attempt-1/smooth.ptx", PTX_SHA256),
    (
        "attempt-1/status.json",
        "43b053ac3b3770b436e146d88671a4881f0c688bae8bc3bd0ca2137b2a9315b0",
    ),
    (
        "attempt-2/authoring.json",
        "b3cc1da8982a10a399c9764c40d14b273adaf8f2365aabc601e53b8d423e0040",
    ),
    (
        "attempt-2/generate.py",
        "48a3195bbf4169d46ccae9066c42952e4d78949e22f50ade0341562b8327a20d",
    ),
    (
        "attempt-2/generation.log",
        "de8dc1bea4b1f674692f219f14af520decb738093991ef576f64f30cc90474fe",
    ),
    ("attempt-2/input.json", INPUT_SHA256),
    ("attempt-2/output.json", OUTPUT_SHA256),
    (
        "attempt-2/raw-bits.json",
        "11a0d12891c86ae1322e596f871731b37093697663fe9675e9d4f9d50bd892c0",
    ),
    ("attempt-2/smooth.cu", CU_SHA256),
    ("attempt-2/smooth.meta", META_SHA256),
    ("attempt-2/smooth.ptx", PTX_SHA256),
    (
        "attempt-2/status.json",
        "c60def656768b83814f7b3ab4148db9ac3846897032d973c956f3c189ad4c081",
    ),
    ("attempt-3/authoring.json", AUTHORING_SHA256),
    ("attempt-3/generate.py", GENERATOR_SHA256),
    (
        "attempt-3/generation.log",
        "31ec79d697c3bd1e6ddd2e471fd5ebc83acd061d31b8cd3077c140c368d82956",
    ),
    ("attempt-3/input.json", INPUT_SHA256),
    ("attempt-3/output.json", OUTPUT_SHA256),
    ("attempt-3/raw-bits.json", RAW_SHA256),
    ("attempt-3/raw-readback.json", RAW_SHA256),
    ("attempt-3/smooth.cu", CU_SHA256),
    ("attempt-3/smooth.meta", META_SHA256),
    ("attempt-3/smooth.ptx", PTX_SHA256),
    (
        "attempt-3/state.json",
        "29fbc75e4ea3aceb3015b62497b8f5d340fc05f0c81e663c00a81f43b6684e75",
    ),
    (
        "attempt-3/status.json",
        "9912dce596df3903efe3ba2fa5d99d18b22aa60630aa18c08ed3567b8b066c57",
    ),
    ("smooth.cu", CU_SHA256),
    ("smooth.meta", META_SHA256),
    ("smooth.ptx", PTX_SHA256),
];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenWarpKinematicsReceipt {
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
    pub compiler_options_provenance: String,
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
    pub generation_executions_file: String,
    pub generation_executions_sha256: String,
    pub attempt_statuses: Vec<ArtifactHash>,
    pub prevalidation_readback_file: String,
    pub prevalidation_readback_sha256: String,
    pub prevalidation_state_file: String,
    pub prevalidation_state_sha256: String,
    pub python_version: String,
    pub python_executable_sha256: String,
    pub warp_dll_sha256: String,
    pub cuda_toolkit: String,
    pub cuda_driver_api_version: String,
    pub nvidia_driver_version: String,
    pub device: String,
    pub generation_attempts: usize,
    pub first_attempt_readback_preserved: bool,
    pub second_attempt_output_preserved: bool,
    pub third_attempt_prevalidation_readback_preserved: bool,
    pub qpos_generated_kinematics: bool,
    pub zero_qpos_only: bool,
    pub nonzero_rotation_coverage: bool,
    pub legacy_static_fields_only_as_expected_assertions: bool,
    pub candidate_kinematics_inputs: bool,
    pub authoring_only: bool,
    pub product_python_dependency: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactHash {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenWarpKinematicsInput {
    pub schema: u32,
    pub scope: String,
    pub upstream_revision: String,
    pub warp_version: String,
    pub worlds: usize,
    pub nbody: usize,
    pub njnt: usize,
    pub nq: usize,
    pub nv: usize,
    pub nbranch: usize,
    pub nmocap: usize,
    pub qpos0: Vec<Vec<f32>>,
    pub body_parentid: Vec<i32>,
    pub body_rootid: Vec<i32>,
    pub body_mocapid: Vec<i32>,
    pub body_jntnum: Vec<i32>,
    pub body_jntadr: Vec<i32>,
    pub body_pos: Vec<Vec<f32>>,
    pub body_quat_wxyz: Vec<Vec<f32>>,
    pub jnt_type: Vec<i32>,
    pub jnt_qposadr: Vec<i32>,
    pub jnt_dofadr: Vec<i32>,
    pub jnt_bodyid: Vec<i32>,
    pub jnt_pos: Vec<Vec<f32>>,
    pub jnt_axis: Vec<Vec<f32>>,
    pub body_branches: Vec<i32>,
    pub body_branch_start: Vec<i32>,
    pub qpos: Vec<Vec<f32>>,
    pub mocap_pos: Vec<Vec<f32>>,
    pub mocap_quat_wxyz: Vec<Vec<f32>>,
    pub body_ipos: Vec<Vec<f32>>,
    pub body_iquat_wxyz: Vec<Vec<f32>>,
    pub body_mass: Vec<Vec<f32>>,
    pub body_subtreemass: Vec<Vec<f32>>,
    pub body_tree_levels: Vec<Vec<i32>>,
    pub legacy_static_input: String,
    pub legacy_static_input_sha256: String,
    pub candidate_kinematics_inputs: bool,
}

#[derive(Clone, Debug)]
pub struct FrozenWarpKinematicsReadback {
    pub xpos: Vec<f32>,
    pub xquat_wxyz: Vec<f32>,
    pub xipos: Vec<f32>,
    pub xmat: Vec<f32>,
    pub xanchor: Vec<f32>,
    pub xaxis: Vec<f32>,
    pub subtree_com: Vec<f32>,
    pub cdof: Vec<f32>,
    bits: Vec<Vec<u32>>,
}

impl FrozenWarpKinematicsReadback {
    pub fn bits(&self) -> &[Vec<u32>] {
        &self.bits
    }

    pub fn into_finite(self) -> Result<Self, ProbeError> {
        if self
            .fields()
            .into_iter()
            .flat_map(|values| values.iter())
            .any(|value| !value.is_finite())
        {
            return Err(artifact("output", "non-finite value"));
        }
        Ok(self)
    }

    fn fields(&self) -> [&[f32]; 8] {
        [
            &self.xpos,
            &self.xquat_wxyz,
            &self.xipos,
            &self.xmat,
            &self.xanchor,
            &self.xaxis,
            &self.subtree_com,
            &self.cdof,
        ]
    }
}

pub struct FrozenWarpKinematicsArtifact {
    receipt: FrozenWarpKinematicsReceipt,
    input: FrozenWarpKinematicsInput,
    readback: FrozenWarpKinematicsReadback,
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
    nq: usize,
    nv: usize,
    xpos: Vec<f32>,
    xquat_wxyz: Vec<f32>,
    xipos: Vec<f32>,
    xmat: Vec<f32>,
    xanchor: Vec<f32>,
    xaxis: Vec<f32>,
    subtree_com: Vec<f32>,
    cdof: Vec<f32>,
    legacy_static_fields_bit_identical: bool,
    legacy_com_cdof_bit_identical: bool,
    candidate_kinematics_inputs: bool,
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

// 子模块：纯移动自原单文件，行为不变。
mod artifact_launch;
mod artifact_load;
mod support;
mod validate_cuda;
mod validate_input;
mod validate_output;
mod validate_ptx;
mod validate_receipt;
use self::support::artifact;
#[cfg(test)]
use self::validate_input::validate_input;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nonzero_qpos_and_nonfinite_readback() {
        let mut input: FrozenWarpKinematicsInput = serde_json::from_str(include_str!(
            "../../fixtures/gravity-upstream-abi/independent-kinematics/input.json"
        ))
        .unwrap();
        validate_input(&input).unwrap();
        input.qpos[0][0] = 0.25;
        assert!(validate_input(&input).is_err());

        let mut readback = FrozenWarpKinematicsReadback {
            xpos: vec![0.0],
            xquat_wxyz: vec![0.0],
            xipos: vec![0.0],
            xmat: vec![0.0],
            xanchor: vec![0.0],
            xaxis: vec![0.0],
            subtree_com: vec![0.0],
            cdof: vec![0.0],
            bits: vec![vec![0]; 8],
        };
        readback.xpos[0] = f32::NAN;
        assert_eq!(readback.bits()[0], [0]);
        assert!(readback.into_finite().is_err());
    }
}
