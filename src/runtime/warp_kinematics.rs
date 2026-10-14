//! 冻结Warp刚体运动学链导入器。
//! 它从固定模型与qpos生成重力前置量。

use crate::diagnostics::ProbeError;
use cudarc::{
    driver::{
        CudaFunction, CudaStream, DevicePtr, DevicePtrMut, DeviceRepr, DriverError, LaunchConfig,
        PushKernelArg,
    },
    nvrtc::Ptx,
};
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

#[repr(C)]
#[derive(Clone, Copy)]
struct WarpArray {
    data: u64,
    grad: u64,
    shape: [i32; 4],
    strides: [i32; 4],
    ndim: u16,
    flags: u16,
    padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WarpBounds2 {
    shape: [i32; 2],
    size: u64,
    coord_mult: u64,
}

// SAFETY: 两个类型只含固定宽度值。
// repr(C)绑定冻结Warp 1.15头文件。
unsafe impl DeviceRepr for WarpArray {}
// SAFETY: 布局匹配launch_bounds_t<2>。
unsafe impl DeviceRepr for WarpBounds2 {}

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
    ptx: String,
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

impl FrozenWarpKinematicsArtifact {
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, ProbeError> {
        let directory = directory.as_ref();
        let receipt: FrozenWarpKinematicsReceipt =
            serde_json::from_slice(&read(directory.join("receipt.json"), "receipt")?)
                .map_err(|error| artifact("receipt", error.to_string()))?;
        validate_receipt(&receipt)?;
        for &(path, expected) in &ATTEMPT_FILES {
            checked_path(directory, path, expected, "attempt")?;
        }
        for item in &receipt.attempt_statuses {
            checked_path(directory, &item.path, &item.sha256, "attempt status")?;
        }
        checked_path(
            directory,
            &receipt.prevalidation_readback_file,
            &receipt.prevalidation_readback_sha256,
            "prevalidation readback",
        )?;
        checked_path(
            directory,
            &receipt.prevalidation_state_file,
            &receipt.prevalidation_state_sha256,
            "prevalidation state",
        )?;
        let input_bytes = checked_file(directory, &receipt.input_file, INPUT_SHA256, "input")?;
        let output_bytes = checked_file(directory, &receipt.output_file, OUTPUT_SHA256, "output")?;
        let raw_bytes = checked_file(directory, &receipt.raw_bits_file, RAW_SHA256, "raw bits")?;
        let cu = checked_file(directory, &receipt.cu_file, CU_SHA256, "cu")?;
        let meta = checked_file(directory, &receipt.meta_file, META_SHA256, "meta")?;
        let ptx = checked_file(directory, &receipt.ptx_file, PTX_SHA256, "ptx")?;
        checked_file(
            directory,
            &receipt.generator_file,
            GENERATOR_SHA256,
            "generator",
        )?;
        checked_file(
            directory,
            &receipt.authoring_file,
            AUTHORING_SHA256,
            "authoring",
        )?;
        checked_file(
            directory,
            &receipt.generation_executions_file,
            EXECUTIONS_SHA256,
            "generation executions",
        )?;

        validate_cuda(&cu)?;
        let ptx = validate_ptx(&ptx, receipt.target_arch)?;
        validate_meta(&meta)?;
        let input: FrozenWarpKinematicsInput = serde_json::from_slice(&input_bytes)
            .map_err(|error| artifact("input", error.to_string()))?;
        validate_input(&input)?;
        let output: FrozenOutputFile = serde_json::from_slice(&output_bytes)
            .map_err(|error| artifact("output", error.to_string()))?;
        validate_output(&output, &input)?;
        let raw: RawBitsFile = serde_json::from_slice(&raw_bytes)
            .map_err(|error| artifact("raw bits", error.to_string()))?;
        let bits = validate_raw(&raw, &output)?;
        let readback = FrozenWarpKinematicsReadback {
            xpos: output.xpos,
            xquat_wxyz: output.xquat_wxyz,
            xipos: output.xipos,
            xmat: output.xmat,
            xanchor: output.xanchor,
            xaxis: output.xaxis,
            subtree_com: output.subtree_com,
            cdof: output.cdof,
            bits,
        };
        Ok(Self {
            receipt,
            input,
            readback,
            ptx,
        })
    }

    pub fn receipt(&self) -> &FrozenWarpKinematicsReceipt {
        &self.receipt
    }

    pub fn input(&self) -> &FrozenWarpKinematicsInput {
        &self.input
    }

    pub fn readback(&self) -> FrozenWarpKinematicsReadback {
        self.readback.clone()
    }

    pub fn launch_diagnostic(
        &self,
        device: usize,
    ) -> Result<FrozenWarpKinematicsReadback, ProbeError> {
        let input = &self.input;
        validate_input(input)?;
        let stream = super::cuda::transfer_stream(device)?;
        let module = stream
            .context()
            .load_module(Ptx::from_src(self.ptx.clone()))
            .map_err(cuda_error("frozen-warp-kinematics-load"))?;
        let kinematics = module
            .load_function(KIN_ENTRY)
            .map_err(cuda_error("frozen-warp-kinematics-entry"))?;
        let matrices = module
            .load_function(MATRIX_ENTRY)
            .map_err(cuda_error("frozen-warp-matrices-entry"))?;
        let inertial = module
            .load_function(INERTIAL_ENTRY)
            .map_err(cuda_error("frozen-warp-inertial-entry"))?;
        let init = module
            .load_function(INIT_ENTRY)
            .map_err(cuda_error("frozen-warp-com-init-entry"))?;
        let acc = module
            .load_function(ACC_ENTRY)
            .map_err(cuda_error("frozen-warp-com-acc-entry"))?;
        let div = module
            .load_function(DIV_ENTRY)
            .map_err(cuda_error("frozen-warp-com-div-entry"))?;
        let cdof_kernel = module
            .load_function(CDOF_ENTRY)
            .map_err(cuda_error("frozen-warp-cdof-entry"))?;

        let qpos0_values = flatten(&input.qpos0);
        let body_pos_values = flatten(&input.body_pos);
        let body_quat_values = flatten(&input.body_quat_wxyz);
        let jnt_pos_values = flatten(&input.jnt_pos);
        let jnt_axis_values = flatten(&input.jnt_axis);
        let qpos_values = flatten(&input.qpos);
        let mocap_pos_values = repeat_rows(&input.mocap_pos, input.worlds);
        let mocap_quat_values = repeat_rows(&input.mocap_quat_wxyz, input.worlds);
        let body_ipos_values = flatten(&input.body_ipos);
        let body_iquat_values = flatten(&input.body_iquat_wxyz);
        let body_mass_values = flatten(&input.body_mass);
        let body_subtreemass_values = flatten(&input.body_subtreemass);
        let tree_values = input
            .body_tree_levels
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();

        let qpos0 = stream
            .clone_htod(&qpos0_values)
            .map_err(cuda_error("upload"))?;
        let parent = stream
            .clone_htod(&input.body_parentid)
            .map_err(cuda_error("upload"))?;
        let roots = stream
            .clone_htod(&input.body_rootid)
            .map_err(cuda_error("upload"))?;
        let mocapid = stream
            .clone_htod(&input.body_mocapid)
            .map_err(cuda_error("upload"))?;
        let jntnum = stream
            .clone_htod(&input.body_jntnum)
            .map_err(cuda_error("upload"))?;
        let jntadr = stream
            .clone_htod(&input.body_jntadr)
            .map_err(cuda_error("upload"))?;
        let body_pos = stream
            .clone_htod(&body_pos_values)
            .map_err(cuda_error("upload"))?;
        let body_quat = stream
            .clone_htod(&body_quat_values)
            .map_err(cuda_error("upload"))?;
        let jnt_type = stream
            .clone_htod(&input.jnt_type)
            .map_err(cuda_error("upload"))?;
        let jnt_qposadr = stream
            .clone_htod(&input.jnt_qposadr)
            .map_err(cuda_error("upload"))?;
        let jnt_dofadr = stream
            .clone_htod(&input.jnt_dofadr)
            .map_err(cuda_error("upload"))?;
        let jnt_bodyid = stream
            .clone_htod(&input.jnt_bodyid)
            .map_err(cuda_error("upload"))?;
        let jnt_pos = stream
            .clone_htod(&jnt_pos_values)
            .map_err(cuda_error("upload"))?;
        let jnt_axis = stream
            .clone_htod(&jnt_axis_values)
            .map_err(cuda_error("upload"))?;
        let branches = stream
            .clone_htod(&input.body_branches)
            .map_err(cuda_error("upload"))?;
        let branch_start = stream
            .clone_htod(&input.body_branch_start)
            .map_err(cuda_error("upload"))?;
        let qpos = stream
            .clone_htod(&qpos_values)
            .map_err(cuda_error("upload"))?;
        let mocap_pos = stream
            .clone_htod(&mocap_pos_values)
            .map_err(cuda_error("upload"))?;
        let mocap_quat = stream
            .clone_htod(&mocap_quat_values)
            .map_err(cuda_error("upload"))?;
        let body_ipos = stream
            .clone_htod(&body_ipos_values)
            .map_err(cuda_error("upload"))?;
        let body_iquat = stream
            .clone_htod(&body_iquat_values)
            .map_err(cuda_error("upload"))?;
        let body_mass = stream
            .clone_htod(&body_mass_values)
            .map_err(cuda_error("upload"))?;
        let body_subtreemass = stream
            .clone_htod(&body_subtreemass_values)
            .map_err(cuda_error("upload"))?;
        let tree = stream
            .clone_htod(&tree_values)
            .map_err(cuda_error("upload"))?;

        let mut xpos_seed = vec![f32::NAN; input.worlds * input.nbody * 3];
        let mut xquat_seed = vec![f32::NAN; input.worlds * input.nbody * 4];
        for world in 0..input.worlds {
            xpos_seed[world * input.nbody * 3..world * input.nbody * 3 + 3].fill(0.0);
            xquat_seed[world * input.nbody * 4] = 1.0;
            xquat_seed[world * input.nbody * 4 + 1..world * input.nbody * 4 + 4].fill(0.0);
        }
        let mut xpos = stream
            .clone_htod(&xpos_seed)
            .map_err(cuda_error("output"))?;
        let mut xquat = stream
            .clone_htod(&xquat_seed)
            .map_err(cuda_error("output"))?;
        let mut xanchor = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.njnt * 3])
            .map_err(cuda_error("output"))?;
        let mut xaxis = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.njnt * 3])
            .map_err(cuda_error("output"))?;
        let mut xmat = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.nbody * 9])
            .map_err(cuda_error("output"))?;
        let mut xipos = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.nbody * 3])
            .map_err(cuda_error("output"))?;
        let mut ximat = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.nbody * 9])
            .map_err(cuda_error("output"))?;
        let mut subtree = stream
            .alloc_zeros::<f32>(input.worlds * input.nbody * 3)
            .map_err(cuda_error("output"))?;
        let mut cdof = stream
            .alloc_zeros::<f32>(input.worlds * input.nv * 6)
            .map_err(cuda_error("output"))?;

        let (qpos0_ptr, qpos0_record) = qpos0.device_ptr(&stream);
        let (parent_ptr, parent_record) = parent.device_ptr(&stream);
        let (roots_ptr, roots_record) = roots.device_ptr(&stream);
        let (mocapid_ptr, mocapid_record) = mocapid.device_ptr(&stream);
        let (jntnum_ptr, jntnum_record) = jntnum.device_ptr(&stream);
        let (jntadr_ptr, jntadr_record) = jntadr.device_ptr(&stream);
        let (body_pos_ptr, body_pos_record) = body_pos.device_ptr(&stream);
        let (body_quat_ptr, body_quat_record) = body_quat.device_ptr(&stream);
        let (jnt_type_ptr, jnt_type_record) = jnt_type.device_ptr(&stream);
        let (jnt_qposadr_ptr, jnt_qposadr_record) = jnt_qposadr.device_ptr(&stream);
        let (jnt_dofadr_ptr, jnt_dofadr_record) = jnt_dofadr.device_ptr(&stream);
        let (jnt_bodyid_ptr, jnt_bodyid_record) = jnt_bodyid.device_ptr(&stream);
        let (jnt_pos_ptr, jnt_pos_record) = jnt_pos.device_ptr(&stream);
        let (jnt_axis_ptr, jnt_axis_record) = jnt_axis.device_ptr(&stream);
        let (branches_ptr, branches_record) = branches.device_ptr(&stream);
        let (branch_start_ptr, branch_start_record) = branch_start.device_ptr(&stream);
        let (qpos_ptr, qpos_record) = qpos.device_ptr(&stream);
        let (mocap_pos_ptr, mocap_pos_record) = mocap_pos.device_ptr(&stream);
        let (mocap_quat_ptr, mocap_quat_record) = mocap_quat.device_ptr(&stream);
        let (body_ipos_ptr, body_ipos_record) = body_ipos.device_ptr(&stream);
        let (body_iquat_ptr, body_iquat_record) = body_iquat.device_ptr(&stream);
        let (body_mass_ptr, body_mass_record) = body_mass.device_ptr(&stream);
        let (body_subtreemass_ptr, body_subtreemass_record) = body_subtreemass.device_ptr(&stream);
        let (tree_ptr, tree_record) = tree.device_ptr(&stream);
        let (xpos_ptr, xpos_record) = xpos.device_ptr_mut(&stream);
        let (xquat_ptr, xquat_record) = xquat.device_ptr_mut(&stream);
        let (xanchor_ptr, xanchor_record) = xanchor.device_ptr_mut(&stream);
        let (xaxis_ptr, xaxis_record) = xaxis.device_ptr_mut(&stream);
        let (xmat_ptr, xmat_record) = xmat.device_ptr_mut(&stream);
        let (xipos_ptr, xipos_record) = xipos.device_ptr_mut(&stream);
        let (ximat_ptr, ximat_record) = ximat.device_ptr_mut(&stream);
        let (subtree_ptr, subtree_record) = subtree.device_ptr_mut(&stream);
        let (cdof_ptr, cdof_record) = cdof.device_ptr_mut(&stream);

        launch2(
            &stream,
            &kinematics,
            [input.worlds, input.nbranch],
            &[
                contiguous(qpos0_ptr, &[input.qpos0.len(), input.nq], 4)?,
                contiguous(parent_ptr, &[input.nbody], 4)?,
                contiguous(mocapid_ptr, &[input.nbody], 4)?,
                contiguous(jntnum_ptr, &[input.nbody], 4)?,
                contiguous(jntadr_ptr, &[input.nbody], 4)?,
                contiguous(body_pos_ptr, &[input.body_pos.len(), input.nbody], 12)?,
                contiguous(
                    body_quat_ptr,
                    &[input.body_quat_wxyz.len(), input.nbody],
                    16,
                )?,
                contiguous(jnt_type_ptr, &[input.njnt], 4)?,
                contiguous(jnt_qposadr_ptr, &[input.njnt], 4)?,
                contiguous(jnt_pos_ptr, &[input.jnt_pos.len(), input.njnt], 12)?,
                contiguous(jnt_axis_ptr, &[input.jnt_axis.len(), input.njnt], 12)?,
                contiguous(branches_ptr, &[input.body_branches.len()], 4)?,
                contiguous(branch_start_ptr, &[input.body_branch_start.len()], 4)?,
                contiguous(qpos_ptr, &[input.worlds, input.nq], 4)?,
                contiguous(mocap_pos_ptr, &[input.worlds, input.nmocap], 12)?,
                contiguous(mocap_quat_ptr, &[input.worlds, input.nmocap], 16)?,
                contiguous(xpos_ptr, &[input.worlds, input.nbody], 12)?,
                contiguous(xquat_ptr, &[input.worlds, input.nbody], 16)?,
                contiguous(xanchor_ptr, &[input.worlds, input.njnt], 12)?,
                contiguous(xaxis_ptr, &[input.worlds, input.njnt], 12)?,
            ],
            "frozen-warp-kinematics-launch",
        )?;
        launch2(
            &stream,
            &matrices,
            [input.worlds, input.nbody],
            &[
                contiguous(xquat_ptr, &[input.worlds, input.nbody], 16)?,
                contiguous(xmat_ptr, &[input.worlds, input.nbody], 36)?,
            ],
            "frozen-warp-matrices-launch",
        )?;
        launch2(
            &stream,
            &inertial,
            [input.worlds, input.nbody],
            &[
                contiguous(body_ipos_ptr, &[input.body_ipos.len(), input.nbody], 12)?,
                contiguous(
                    body_iquat_ptr,
                    &[input.body_iquat_wxyz.len(), input.nbody],
                    16,
                )?,
                contiguous(xpos_ptr, &[input.worlds, input.nbody], 12)?,
                contiguous(xquat_ptr, &[input.worlds, input.nbody], 16)?,
                contiguous(xipos_ptr, &[input.worlds, input.nbody], 12)?,
                contiguous(ximat_ptr, &[input.worlds, input.nbody], 36)?,
            ],
            "frozen-warp-inertial-launch",
        )?;
        launch2(
            &stream,
            &init,
            [input.worlds, input.nbody],
            &[
                contiguous(body_mass_ptr, &[input.body_mass.len(), input.nbody], 4)?,
                contiguous(xipos_ptr, &[input.worlds, input.nbody], 12)?,
                contiguous(subtree_ptr, &[input.worlds, input.nbody], 12)?,
            ],
            "frozen-warp-com-init-launch",
        )?;
        let mut tree_offset = tree_values.len();
        for level in input.body_tree_levels.iter().rev() {
            tree_offset -= level.len();
            let byte_offset = u64::try_from(tree_offset)
                .ok()
                .and_then(|value| value.checked_mul(4))
                .ok_or_else(|| artifact("input", "tree offset overflow"))?;
            let level_ptr = tree_ptr
                .checked_add(byte_offset)
                .ok_or_else(|| artifact("input", "tree pointer overflow"))?;
            launch2(
                &stream,
                &acc,
                [input.worlds, level.len()],
                &[
                    contiguous(parent_ptr, &[input.nbody], 4)?,
                    contiguous(subtree_ptr, &[input.worlds, input.nbody], 12)?,
                    contiguous(level_ptr, &[level.len()], 4)?,
                    contiguous(subtree_ptr, &[input.worlds, input.nbody], 12)?,
                ],
                "frozen-warp-com-acc-launch",
            )?;
        }
        launch2(
            &stream,
            &div,
            [input.worlds, input.nbody],
            &[
                contiguous(
                    body_subtreemass_ptr,
                    &[input.body_subtreemass.len(), input.nbody],
                    4,
                )?,
                contiguous(subtree_ptr, &[input.worlds, input.nbody], 12)?,
                contiguous(subtree_ptr, &[input.worlds, input.nbody], 12)?,
            ],
            "frozen-warp-com-div-launch",
        )?;
        launch2(
            &stream,
            &cdof_kernel,
            [input.worlds, input.njnt],
            &[
                contiguous(roots_ptr, &[input.nbody], 4)?,
                contiguous(jnt_type_ptr, &[input.njnt], 4)?,
                contiguous(jnt_dofadr_ptr, &[input.njnt], 4)?,
                contiguous(jnt_bodyid_ptr, &[input.njnt], 4)?,
                contiguous(xmat_ptr, &[input.worlds, input.nbody], 36)?,
                contiguous(xanchor_ptr, &[input.worlds, input.njnt], 12)?,
                contiguous(xaxis_ptr, &[input.worlds, input.njnt], 12)?,
                contiguous(subtree_ptr, &[input.worlds, input.nbody], 12)?,
                contiguous(cdof_ptr, &[input.worlds, input.nv], 24)?,
            ],
            "frozen-warp-cdof-launch",
        )?;

        drop(cdof_record);
        drop(subtree_record);
        drop(ximat_record);
        drop(xipos_record);
        drop(xmat_record);
        drop(xaxis_record);
        drop(xanchor_record);
        drop(xquat_record);
        drop(xpos_record);
        drop(tree_record);
        drop(body_subtreemass_record);
        drop(body_mass_record);
        drop(body_iquat_record);
        drop(body_ipos_record);
        drop(mocap_quat_record);
        drop(mocap_pos_record);
        drop(qpos_record);
        drop(branch_start_record);
        drop(branches_record);
        drop(jnt_axis_record);
        drop(jnt_pos_record);
        drop(jnt_bodyid_record);
        drop(jnt_dofadr_record);
        drop(jnt_qposadr_record);
        drop(jnt_type_record);
        drop(body_quat_record);
        drop(body_pos_record);
        drop(jntadr_record);
        drop(jntnum_record);
        drop(mocapid_record);
        drop(roots_record);
        drop(parent_record);
        drop(qpos0_record);

        let fields = [
            stream.clone_dtoh(&xpos).map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&xquat).map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&xipos).map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&xmat).map_err(cuda_error("readback"))?,
            stream
                .clone_dtoh(&xanchor)
                .map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&xaxis).map_err(cuda_error("readback"))?,
            stream
                .clone_dtoh(&subtree)
                .map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&cdof).map_err(cuda_error("readback"))?,
        ];
        let bits = fields
            .iter()
            .map(|values| values.iter().map(|value| value.to_bits()).collect())
            .collect();
        let [
            xpos,
            xquat_wxyz,
            xipos,
            xmat,
            xanchor,
            xaxis,
            subtree_com,
            cdof,
        ] = fields;
        Ok(FrozenWarpKinematicsReadback {
            xpos,
            xquat_wxyz,
            xipos,
            xmat,
            xanchor,
            xaxis,
            subtree_com,
            cdof,
            bits,
        })
    }
}

fn validate_receipt(receipt: &FrozenWarpKinematicsReceipt) -> Result<(), ProbeError> {
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

fn validate_input(input: &FrozenWarpKinematicsInput) -> Result<(), ProbeError> {
    if input.schema != 1
        || input.scope != "gravity-upstream-independent-qpos-kinematics-com-cdof"
        || input.upstream_revision != REVISION
        || input.warp_version != WARP_VERSION
        || (input.worlds, input.nbody, input.njnt, input.nq, input.nv) != (7, 5, 4, 4, 4)
        || input.nbranch != 2
        || input.nmocap != 1
        || input.legacy_static_input != "../independent-inputs/input.json"
        || input.legacy_static_input_sha256
            != "5136b09e1d727b7799d527ebed42b1f7ad05b0a0040928a78a69137b2400ec0c"
        || input.candidate_kinematics_inputs
    {
        return Err(artifact("input", "frozen identity mismatch"));
    }
    let exact = [
        (input.body_parentid.len(), input.nbody),
        (input.body_rootid.len(), input.nbody),
        (input.body_mocapid.len(), input.nbody),
        (input.body_jntnum.len(), input.nbody),
        (input.body_jntadr.len(), input.nbody),
        (input.jnt_type.len(), input.njnt),
        (input.jnt_qposadr.len(), input.njnt),
        (input.jnt_dofadr.len(), input.njnt),
        (input.jnt_bodyid.len(), input.njnt),
        (input.body_branches.len(), input.nbody - 1),
        (input.body_branch_start.len(), input.nbranch + 1),
        (input.qpos.len(), input.worlds),
        (input.mocap_pos.len(), input.nmocap),
        (input.mocap_quat_wxyz.len(), input.nmocap),
    ];
    if exact.iter().any(|&(actual, expected)| actual != expected)
        || !rows(&input.qpos0, input.nq)
        || !rows(&input.body_pos, input.nbody * 3)
        || !rows(&input.body_quat_wxyz, input.nbody * 4)
        || !rows(&input.jnt_pos, input.njnt * 3)
        || !rows(&input.jnt_axis, input.njnt * 3)
        || !rows(&input.qpos, input.nq)
        || !rows(&input.mocap_pos, 3)
        || !rows(&input.mocap_quat_wxyz, 4)
        || !rows(&input.body_ipos, input.nbody * 3)
        || !rows(&input.body_iquat_wxyz, input.nbody * 4)
        || !rows(&input.body_mass, input.nbody)
        || !rows(&input.body_subtreemass, input.nbody)
    {
        return Err(artifact("input", "field length mismatch"));
    }
    if [
        &input.qpos0,
        &input.body_pos,
        &input.body_quat_wxyz,
        &input.jnt_pos,
        &input.jnt_axis,
        &input.qpos,
        &input.mocap_pos,
        &input.mocap_quat_wxyz,
        &input.body_ipos,
        &input.body_iquat_wxyz,
        &input.body_mass,
        &input.body_subtreemass,
    ]
    .into_iter()
    .flat_map(|rows| rows.iter().flatten())
    .any(|value| !value.is_finite())
    {
        return Err(artifact("input", "non-finite value"));
    }
    if input
        .qpos
        .iter()
        .flatten()
        .any(|value| value.to_bits() != 0)
        || input.body_parentid[0] != 0
        || input.body_rootid[0] != 0
        || input.body_branch_start.first() != Some(&0)
        || input.body_branch_start.last() != Some(&(input.body_branches.len() as i32))
    {
        return Err(artifact("input", "topology or state mismatch"));
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
    Ok(())
}

fn validate_output(
    output: &FrozenOutputFile,
    input: &FrozenWarpKinematicsInput,
) -> Result<(), ProbeError> {
    let world_body = checked_product(input.worlds, input.nbody)?;
    let world_joint = checked_product(input.worlds, input.njnt)?;
    if output.schema != 1
        || output.evidence_kind != "frozen-upstream-generated-qpos-kinematics-com-cdof"
        || output.upstream_revision != REVISION
        || output.warp_version != WARP_VERSION
        || output.input_sha256 != INPUT_SHA256
        || (
            output.worlds,
            output.nbody,
            output.njnt,
            output.nq,
            output.nv,
        ) != (input.worlds, input.nbody, input.njnt, input.nq, input.nv)
        || output.xpos.len() != checked_product(world_body, 3)?
        || output.xquat_wxyz.len() != checked_product(world_body, 4)?
        || output.xipos.len() != checked_product(world_body, 3)?
        || output.xmat.len() != checked_product(world_body, 9)?
        || output.xanchor.len() != checked_product(world_joint, 3)?
        || output.xaxis.len() != checked_product(world_joint, 3)?
        || output.subtree_com.len() != checked_product(world_body, 3)?
        || output.cdof.len() != checked_product(input.worlds * input.nv, 6)?
        || !output.legacy_static_fields_bit_identical
        || !output.legacy_com_cdof_bit_identical
        || output.candidate_kinematics_inputs
    {
        return Err(artifact("output", "shape or identity mismatch"));
    }
    Ok(())
}

fn validate_raw(raw: &RawBitsFile, output: &FrozenOutputFile) -> Result<Vec<Vec<u32>>, ProbeError> {
    if raw.schema != 1
        || raw.provenance != "direct_upstream_device_readback_before_finite_or_exact_validation"
        || raw.upstream_revision != REVISION
        || raw.fields.len() != 8
    {
        return Err(artifact("raw bits", "identity mismatch"));
    }
    let expected = [
        ("xpos", output.xpos.as_slice()),
        ("xquat_wxyz", output.xquat_wxyz.as_slice()),
        ("xipos", output.xipos.as_slice()),
        ("xmat", output.xmat.as_slice()),
        ("xanchor", output.xanchor.as_slice()),
        ("xaxis", output.xaxis.as_slice()),
        ("subtree_com", output.subtree_com.as_slice()),
        ("cdof", output.cdof.as_slice()),
    ];
    raw.fields
        .iter()
        .zip(expected)
        .map(|(field, (name, values))| validate_raw_field(field, name, values))
        .collect()
}

fn validate_raw_field(
    field: &RawField,
    name: &str,
    values: &[f32],
) -> Result<Vec<u32>, ProbeError> {
    if field.field != name || field.count != values.len() || field.bits.len() != values.len() {
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

fn launch2(
    stream: &CudaStream,
    function: &CudaFunction,
    shape: [usize; 2],
    arrays: &[WarpArray],
    stage: &'static str,
) -> Result<(), ProbeError> {
    let size = checked_product(shape[0], shape[1])?;
    let work = u32::try_from(size).map_err(|_| artifact("input", "launch overflow"))?;
    let bounds = WarpBounds2 {
        shape: [
            i32::try_from(shape[0]).map_err(|_| artifact("input", "shape overflow"))?,
            i32::try_from(shape[1]).map_err(|_| artifact("input", "shape overflow"))?,
        ],
        size: u64::from(work),
        coord_mult: 1,
    };
    let mut launch = stream.launch_builder(function);
    launch.arg(&bounds);
    for array in arrays {
        launch.arg(array);
    }
    // SAFETY: 固定签名已逐项核验。
    // 已验证描述符覆盖全部缓冲。
    unsafe {
        launch
            .launch(LaunchConfig {
                grid_dim: (work.div_ceil(256), 1, 1),
                block_dim: (256, 1, 1),
                shared_mem_bytes: 0,
            })
            .map_err(cuda_error(stage))?;
    }
    Ok(())
}

fn contiguous(data: u64, shape: &[usize], element_bytes: usize) -> Result<WarpArray, ProbeError> {
    if data == 0 || !(1..=4).contains(&shape.len()) || shape.contains(&0) {
        return Err(artifact("input", "invalid array shape"));
    }
    let mut array = WarpArray {
        data,
        grad: 0,
        shape: [0; 4],
        strides: [0; 4],
        ndim: shape.len() as u16,
        flags: 0,
        padding: 0,
    };
    let mut stride = element_bytes;
    for axis in (0..shape.len()).rev() {
        array.shape[axis] =
            i32::try_from(shape[axis]).map_err(|_| artifact("input", "shape overflow"))?;
        array.strides[axis] =
            i32::try_from(stride).map_err(|_| artifact("input", "stride overflow"))?;
        stride = stride
            .checked_mul(shape[axis])
            .ok_or_else(|| artifact("input", "stride overflow"))?;
    }
    Ok(array)
}

fn validate_cuda(bytes: &[u8]) -> Result<(), ProbeError> {
    if bytes.contains(&0) {
        return Err(artifact("cu", "embedded nul"));
    }
    let source = std::str::from_utf8(bytes).map_err(|error| artifact("cu", error.to_string()))?;
    for (entry, arrays) in cuda_signatures() {
        let signature = signature_lines(source, &format!("extern \"C\" __global__ void {entry}("))?;
        let expected = format!(
            "extern\"C\"__global__void{entry}(wp::launch_bounds_t<2>dim,{})",
            arrays.join(",")
        );
        if compact(&signature) != expected {
            return Err(artifact("cu", format!("{entry} ABI mismatch")));
        }
    }
    Ok(())
}

fn cuda_signatures() -> [(&'static str, &'static [&'static str]); 7] {
    [
        (
            KIN_ENTRY,
            &[
                "wp::array_t<wp::float32>var_qpos0",
                "wp::array_t<wp::int32>var_body_parentid",
                "wp::array_t<wp::int32>var_body_mocapid",
                "wp::array_t<wp::int32>var_body_jntnum",
                "wp::array_t<wp::int32>var_body_jntadr",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_body_pos",
                "wp::array_t<wp::quat_t<wp::float32>>var_body_quat",
                "wp::array_t<wp::int32>var_jnt_type",
                "wp::array_t<wp::int32>var_jnt_qposadr",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_jnt_pos",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_jnt_axis",
                "wp::array_t<wp::int32>var_body_branches",
                "wp::array_t<wp::int32>var_body_branch_start",
                "wp::array_t<wp::float32>var_qpos_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_mocap_pos_in",
                "wp::array_t<wp::quat_t<wp::float32>>var_mocap_quat_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xpos_out",
                "wp::array_t<wp::quat_t<wp::float32>>var_xquat_out",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xanchor_out",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xaxis_out",
            ],
        ),
        (
            MATRIX_ENTRY,
            &[
                "wp::array_t<wp::quat_t<wp::float32>>var_xquat_in",
                "wp::array_t<wp::mat_t<3,3,wp::float32>>var_xmat_out",
            ],
        ),
        (
            INERTIAL_ENTRY,
            &[
                "wp::array_t<wp::vec_t<3,wp::float32>>var_body_ipos",
                "wp::array_t<wp::quat_t<wp::float32>>var_body_iquat",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xpos_in",
                "wp::array_t<wp::quat_t<wp::float32>>var_xquat_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xipos_out",
                "wp::array_t<wp::mat_t<3,3,wp::float32>>var_ximat_out",
            ],
        ),
        (
            INIT_ENTRY,
            &[
                "wp::array_t<wp::float32>var_body_mass",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xipos_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ],
        ),
        (
            ACC_ENTRY,
            &[
                "wp::array_t<wp::int32>var_body_parentid",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::int32>var_body_tree_",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ],
        ),
        (
            DIV_ENTRY,
            &[
                "wp::array_t<wp::float32>var_body_subtreemass",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ],
        ),
        (
            CDOF_ENTRY,
            &[
                "wp::array_t<wp::int32>var_body_rootid",
                "wp::array_t<wp::int32>var_jnt_type",
                "wp::array_t<wp::int32>var_jnt_dofadr",
                "wp::array_t<wp::int32>var_jnt_bodyid",
                "wp::array_t<wp::mat_t<3,3,wp::float32>>var_xmat_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xanchor_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xaxis_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::vec_t<6,wp::float32>>var_cdof_out",
            ],
        ),
    ]
}

fn validate_ptx(bytes: &[u8], arch: u32) -> Result<String, ProbeError> {
    let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    if bytes.contains(&0) {
        return Err(artifact("ptx", "embedded nul"));
    }
    let source = std::str::from_utf8(bytes).map_err(|error| artifact("ptx", error.to_string()))?;
    if !source
        .lines()
        .any(|line| line.trim() == format!(".target sm_{arch}"))
        || !source.lines().any(|line| line.trim() == ".address_size 64")
    {
        return Err(artifact("ptx", "target mismatch"));
    }
    for (entry, arrays) in [
        (KIN_ENTRY, 20),
        (MATRIX_ENTRY, 2),
        (INERTIAL_ENTRY, 6),
        (INIT_ENTRY, 3),
        (ACC_ENTRY, 4),
        (DIV_ENTRY, 3),
        (CDOF_ENTRY, 9),
    ] {
        let signature = signature_lines(source, &format!(".visible .entry {entry}("))?;
        let mut expected = vec![format!(".visible .entry {entry}(")];
        expected.push(format!(".param .align 8 .b8 {entry}_param_0[24],"));
        for index in 1..=arrays {
            let comma = if index == arrays { "" } else { "," };
            expected.push(format!(
                ".param .align 8 .b8 {entry}_param_{index}[56]{comma}"
            ));
        }
        expected.push(")".into());
        if signature
            .iter()
            .map(|line| line.trim())
            .ne(expected.iter().map(String::as_str))
        {
            return Err(artifact("ptx", format!("{entry} ABI mismatch")));
        }
    }
    Ok(source.to_owned())
}

fn validate_meta(bytes: &[u8]) -> Result<(), ProbeError> {
    let meta: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(bytes).map_err(|error| artifact("meta", error.to_string()))?;
    for entry in ENTRIES {
        if meta
            .get(&format!("{entry}_smem_bytes"))
            .and_then(|value| value.as_u64())
            != Some(0)
        {
            return Err(artifact("meta", "entry metadata mismatch"));
        }
    }
    Ok(())
}

fn checked_file(
    directory: &Path,
    name: &str,
    expected: &str,
    stage: &'static str,
) -> Result<Vec<u8>, ProbeError> {
    if Path::new(name).file_name().and_then(|value| value.to_str()) != Some(name) {
        return Err(artifact(stage, "unsafe filename"));
    }
    checked_path(directory, name, expected, stage)
}

fn checked_path(
    directory: &Path,
    name: &str,
    expected: &str,
    stage: &'static str,
) -> Result<Vec<u8>, ProbeError> {
    let path = Path::new(name);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(artifact(stage, "unsafe path"));
    }
    let bytes = read(directory.join(path), stage)?;
    if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 || digest(&bytes) != expected {
        return Err(artifact(stage, "file hash or size mismatch"));
    }
    Ok(bytes)
}

fn rows(values: &[Vec<f32>], width: usize) -> bool {
    !values.is_empty() && values.iter().all(|row| row.len() == width)
}

fn flatten(values: &[Vec<f32>]) -> Vec<f32> {
    values.iter().flatten().copied().collect()
}

fn repeat_rows(values: &[Vec<f32>], count: usize) -> Vec<f32> {
    (0..count)
        .flat_map(|index| values[index % values.len()].iter().copied())
        .collect()
}

fn checked_product(left: usize, right: usize) -> Result<usize, ProbeError> {
    left.checked_mul(right)
        .ok_or_else(|| artifact("input", "length overflow"))
}

fn read(path: impl AsRef<Path>, stage: &'static str) -> Result<Vec<u8>, ProbeError> {
    fs::read(path).map_err(|error| artifact(stage, error.to_string()))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn strip_comments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut block = false;
    while index < bytes.len() {
        if block {
            if bytes[index..].starts_with(b"*/") {
                block = false;
                output.extend_from_slice(b"  ");
                index += 2;
            } else {
                output.push(if bytes[index] == b'\n' { b'\n' } else { b' ' });
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            block = true;
            output.extend_from_slice(b"  ");
            index += 2;
        } else if bytes[index..].starts_with(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                output.push(b' ');
                index += 1;
            }
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).expect("comment stripping preserves UTF-8")
}

fn signature_lines(source: &str, opening: &str) -> Result<Vec<String>, ProbeError> {
    let clean = strip_comments(source);
    let lines = clean.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| line.trim() == opening)
        .ok_or_else(|| artifact("abi", "entry missing"))?;
    let relative = lines[start..]
        .iter()
        .skip(1)
        .position(|line| line.trim_end().ends_with(')'))
        .map(|value| value + 1)
        .ok_or_else(|| artifact("abi", "entry signature incomplete"))?;
    Ok(lines[start..=start + relative]
        .iter()
        .map(|line| (*line).to_owned())
        .collect())
}

fn compact(lines: &[String]) -> String {
    lines
        .iter()
        .flat_map(|line| line.chars())
        .filter(|value| !value.is_ascii_whitespace())
        .collect()
}

fn artifact(stage: &'static str, detail: impl Into<String>) -> ProbeError {
    ProbeError::Artifact {
        stage,
        detail: detail.into(),
    }
}

fn cuda_error(stage: &'static str) -> impl FnOnce(DriverError) -> ProbeError {
    move |error| ProbeError::Cuda {
        stage,
        code: error.0 as u32,
    }
}

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
