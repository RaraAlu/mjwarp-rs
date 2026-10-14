//! 冻结Warp质心中间量导入器。
//! 它拒绝候选链反馈的中间量。

use crate::diagnostics::ProbeError;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

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

impl FrozenWarpComReadback {
    pub fn subtree_com(&self) -> &[f32] {
        &self.subtree_com
    }

    pub fn cdof(&self) -> &[f32] {
        &self.cdof
    }

    pub fn subtree_com_bits(&self) -> &[u32] {
        &self.subtree_com_bits
    }

    pub fn cdof_bits(&self) -> &[u32] {
        &self.cdof_bits
    }

    pub fn into_finite(self) -> Result<FrozenWarpComOutput, ProbeError> {
        if self
            .subtree_com
            .iter()
            .chain(&self.cdof)
            .any(|value| !value.is_finite())
        {
            return Err(artifact("output", "non-finite value"));
        }
        Ok(FrozenWarpComOutput {
            subtree_com: self.subtree_com,
            cdof: self.cdof,
        })
    }
}

pub struct FrozenWarpComArtifact {
    receipt: FrozenWarpComReceipt,
    input: FrozenWarpComInput,
    readback: FrozenWarpComReadback,
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

impl FrozenWarpComArtifact {
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, ProbeError> {
        let directory = directory.as_ref();
        let receipt_bytes = read(directory.join("receipt.json"), "receipt")?;
        let receipt: FrozenWarpComReceipt = serde_json::from_slice(&receipt_bytes)
            .map_err(|error| artifact("receipt", error.to_string()))?;
        validate_receipt(&receipt)?;

        let input_bytes = checked_file(
            directory,
            &receipt.input_file,
            &receipt.input_sha256,
            "input",
        )?;
        let output_bytes = checked_file(
            directory,
            &receipt.output_file,
            &receipt.output_sha256,
            "output",
        )?;
        let raw_bytes = checked_file(
            directory,
            &receipt.raw_bits_file,
            &receipt.raw_bits_sha256,
            "raw bits",
        )?;
        let cu = checked_file(directory, &receipt.cu_file, &receipt.cu_sha256, "cu")?;
        let meta = checked_file(directory, &receipt.meta_file, &receipt.meta_sha256, "meta")?;
        let ptx = checked_file(directory, &receipt.ptx_file, &receipt.ptx_sha256, "ptx")?;
        for (name, file, expected) in [
            (
                "generator",
                receipt.generator_file.as_str(),
                receipt.generator_sha256.as_str(),
            ),
            (
                "authoring",
                receipt.authoring_file.as_str(),
                receipt.authoring_sha256.as_str(),
            ),
            (
                "generation log",
                receipt.generation_log.as_str(),
                receipt.generation_log_sha256.as_str(),
            ),
            (
                "first attempt log",
                receipt.first_attempt_log.as_str(),
                receipt.first_attempt_log_sha256.as_str(),
            ),
            (
                "generation executions",
                receipt.generation_executions_file.as_str(),
                receipt.generation_executions_sha256.as_str(),
            ),
        ] {
            checked_file(directory, file, expected, name)?;
        }

        validate_cuda(&cu)?;
        validate_ptx(&ptx, receipt.target_arch)?;
        validate_meta(&meta)?;
        let input: FrozenWarpComInput = serde_json::from_slice(&input_bytes)
            .map_err(|error| artifact("input", error.to_string()))?;
        validate_input(&input)?;
        let output: FrozenOutputFile = serde_json::from_slice(&output_bytes)
            .map_err(|error| artifact("output", error.to_string()))?;
        validate_output(&output, &input)?;
        let raw: RawBitsFile = serde_json::from_slice(&raw_bytes)
            .map_err(|error| artifact("raw bits", error.to_string()))?;
        let (subtree_com_bits, cdof_bits) = validate_raw(&raw, &output)?;
        Ok(Self {
            receipt,
            input,
            readback: FrozenWarpComReadback {
                subtree_com: output.subtree_com,
                cdof: output.cdof,
                subtree_com_bits,
                cdof_bits,
            },
        })
    }

    pub fn receipt(&self) -> &FrozenWarpComReceipt {
        &self.receipt
    }

    pub fn input(&self) -> &FrozenWarpComInput {
        &self.input
    }

    /// 调用方先归档位模式，再执行有限值门禁。
    pub fn readback(&self) -> FrozenWarpComReadback {
        self.readback.clone()
    }
}

fn validate_receipt(receipt: &FrozenWarpComReceipt) -> Result<(), ProbeError> {
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

fn validate_input(input: &FrozenWarpComInput) -> Result<(), ProbeError> {
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

fn validate_output(
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

fn validate_raw(
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

fn validate_raw_field(
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

fn validate_cuda(bytes: &[u8]) -> Result<(), ProbeError> {
    if bytes.contains(&0) {
        return Err(artifact("cu", "embedded nul"));
    }
    let source = std::str::from_utf8(bytes).map_err(|error| artifact("cu", error.to_string()))?;
    for (entry, arrays) in [
        (
            INIT_ENTRY,
            [
                "wp::array_t<wp::float32>var_body_mass",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xipos_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ]
            .as_slice(),
        ),
        (
            ACC_ENTRY,
            [
                "wp::array_t<wp::int32>var_body_parentid",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::int32>var_body_tree_",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ]
            .as_slice(),
        ),
        (
            DIV_ENTRY,
            [
                "wp::array_t<wp::float32>var_body_subtreemass",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_out",
            ]
            .as_slice(),
        ),
        (
            CDOF_ENTRY,
            [
                "wp::array_t<wp::int32>var_body_rootid",
                "wp::array_t<wp::int32>var_jnt_type",
                "wp::array_t<wp::int32>var_jnt_dofadr",
                "wp::array_t<wp::int32>var_jnt_bodyid",
                "wp::array_t<wp::mat_t<3,3,wp::float32>>var_xmat_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xanchor_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_xaxis_in",
                "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
                "wp::array_t<wp::vec_t<6,wp::float32>>var_cdof_out",
            ]
            .as_slice(),
        ),
    ] {
        let signature = signature_lines(source, &format!("extern \"C\" __global__ void {entry}("))?;
        let expected = format!(
            "extern\"C\"__global__void{entry}(wp::launch_bounds_t<2>dim,{})",
            arrays.join(",")
        );
        if compact(&signature) != expected {
            return Err(artifact("cu", "entry ABI mismatch"));
        }
    }
    Ok(())
}

fn validate_ptx(bytes: &[u8], arch: u32) -> Result<(), ProbeError> {
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
            return Err(artifact("ptx", "entry ABI mismatch"));
        }
    }
    Ok(())
}

fn validate_meta(bytes: &[u8]) -> Result<(), ProbeError> {
    let meta: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(bytes).map_err(|error| artifact("meta", error.to_string()))?;
    for entry in ENTRIES {
        let key = format!("{entry}_smem_bytes");
        if meta.get(&key).and_then(|value| value.as_u64()) != Some(0) {
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
    let bytes = read(directory.join(name), stage)?;
    if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 || digest(&bytes) != expected {
        return Err(artifact(stage, "file hash or size mismatch"));
    }
    Ok(bytes)
}

fn read(path: impl AsRef<Path>, stage: &'static str) -> Result<Vec<u8>, ProbeError> {
    fs::read(path).map_err(|error| artifact(stage, error.to_string()))
}

fn checked_product(left: usize, right: usize) -> Result<usize, ProbeError> {
    left.checked_mul(right)
        .ok_or_else(|| artifact("input", "length overflow"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_raw_bit_and_topology_drift() {
        let mut input: FrozenWarpComInput = serde_json::from_str(include_str!(
            "../../fixtures/gravity-upstream-abi/independent-inputs/input.json"
        ))
        .unwrap();
        validate_input(&input).unwrap();
        input.body_tree_levels[0][0] = 2;
        assert!(validate_input(&input).is_err());

        let values = [0.0_f32, -1.0];
        let mut field = RawField {
            field: "test".into(),
            count: 2,
            little_endian_sha256: digest(
                &values
                    .iter()
                    .flat_map(|value| value.to_bits().to_le_bytes())
                    .collect::<Vec<_>>(),
            ),
            bits: values
                .iter()
                .map(|value| format!("{:08x}", value.to_bits()))
                .collect(),
        };
        validate_raw_field(&field, "test", &values).unwrap();
        field.bits[1] = "00000000".into();
        assert!(validate_raw_field(&field, "test", &values).is_err());

        let non_finite = FrozenWarpComReadback {
            subtree_com: vec![f32::NAN],
            cdof: vec![0.0],
            subtree_com_bits: vec![f32::NAN.to_bits()],
            cdof_bits: vec![0],
        };
        assert_eq!(non_finite.subtree_com_bits(), &[f32::NAN.to_bits()]);
        assert!(non_finite.into_finite().is_err());
    }
}
