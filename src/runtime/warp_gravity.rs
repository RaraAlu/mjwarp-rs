//! 冻结Warp重力产物导入器。
//! 它只接收固定来源的离线产物。

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
use std::{fs, path::Path, sync::Arc};

const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
const PASSIVE_SHA256: &str = "1334dfb73bde5980d54e1d112b10d1429554a64089a161debe78cf543f7aed8e";
const SUPPORT_SHA256: &str = "b06cdd49d49be73773674ff777fa4544fe5b75ca47b6560891a77fbb9f703ea3";
const CODEGEN_SHA256: &str = "d07a778d7f754dd74620da5c9b2c161b98f9605bc3ddb01c59907f77bd76a170";
const ENTRY: &str = "_gravity_force_7c63ac28_cuda_kernel_forward";
const CU_SHA256: &str = "dd9338e9f38feb767273c3e1a445aea54bee3dee7d899076363202a4d26e865d";
const PTX_SHA256: &str = "d06154da63f64118e8fdb19c0b646a59c5b485d580f4716e5edea97a57c1dd70";
const META_SHA256: &str = "4a80327bc21a875a63b9cf3dfc43386ed40119012357560339f071ac50ea461c";
const GENERATOR_SHA256: &str = "3f0348511e241a345cb3ae66b201cb02b1cef19d532b5980cd86a0e1700b0f2e";
const GENERATION_LOG_SHA256: &str =
    "92626aabae26d638ad6d1c0e83461eac15a0d4c564afb9e3873dab9884578efa";
const GENERATION_EXECUTION_SHA256: &str =
    "8247ab2031fdccfd47d12ca8c3856150aed1c3d5539006cc65a0fd0f57edfd12";
const PYTHON_SHA256: &str = "feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77";
const WARP_DLL_SHA256: &str = "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6";

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
struct WarpBounds3 {
    shape: [i32; 3],
    padding: u32,
    size: u64,
    coord_mult: u64,
}

// SAFETY: 两个类型只含固定宽度值。
// repr(C)绑定冻结Warp 1.15头文件。
unsafe impl DeviceRepr for WarpArray {}
// SAFETY: 布局匹配launch_bounds_t<3>。
unsafe impl DeviceRepr for WarpBounds3 {}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenWarpGravityReceipt {
    pub schema: u32,
    pub evidence_kind: String,
    pub upstream_revision: String,
    pub warp_version: String,
    pub module: String,
    pub passive_source_sha256: String,
    pub support_source_sha256: String,
    pub codegen_source_sha256: String,
    pub target_arch: u32,
    pub mode: String,
    pub fast_math: bool,
    pub fuse_fp: bool,
    pub entry: String,
    pub launch_rank: usize,
    pub array_arguments: usize,
    pub cu_file: String,
    pub ptx_file: String,
    pub meta_file: String,
    pub cu_sha256: String,
    pub ptx_sha256: String,
    pub meta_sha256: String,
    pub python_version: String,
    pub python_executable_sha256: String,
    pub generator_file: String,
    pub generator_sha256: String,
    pub warp_dll_sha256: String,
    pub cuda_toolkit: String,
    pub ptx_compiler: String,
    pub driver_version: String,
    pub device: String,
    pub generation_invocation_id: String,
    pub generation_log: String,
    pub generation_log_sha256: String,
    pub generation_execution_file: String,
    pub generation_execution_sha256: String,
    pub compiler_provenance: String,
    pub authoring_only: bool,
    pub product_python_dependency: bool,
}

#[derive(Clone, Debug)]
pub struct FrozenWarpGravityInput {
    pub worlds: usize,
    pub nbody: usize,
    pub nv: usize,
    pub opt_gravity: Vec<f32>,
    pub body_parentid: Vec<i32>,
    pub body_rootid: Vec<i32>,
    pub body_mass: Vec<f32>,
    pub body_gravcomp: Vec<f32>,
    pub dof_bodyid: Vec<i32>,
    pub body_isdofancestor: Vec<i32>,
    pub xipos: Vec<f32>,
    pub subtree_com: Vec<f32>,
    pub cdof: Vec<f32>,
}

pub struct FrozenWarpGravityArtifact {
    stream: Arc<CudaStream>,
    function: CudaFunction,
    receipt: FrozenWarpGravityReceipt,
}

/// 保留冻结内核的原始设备回读。
/// 调用方先归档位模式，再执行有限值门禁。
pub struct FrozenWarpGravityReadback {
    values: Vec<f32>,
    bits: Vec<u32>,
}

impl FrozenWarpGravityReadback {
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn bits(&self) -> &[u32] {
        &self.bits
    }

    pub fn into_finite(self) -> Result<Vec<f32>, ProbeError> {
        if self.values.iter().any(|value| !value.is_finite()) {
            return Err(artifact("output", "non-finite value"));
        }
        Ok(self.values)
    }
}

impl FrozenWarpGravityArtifact {
    pub fn load(directory: impl AsRef<Path>, device: usize) -> Result<Self, ProbeError> {
        let directory = directory.as_ref();
        let receipt_bytes = read(directory.join("receipt.json"), "receipt")?;
        let receipt: FrozenWarpGravityReceipt = serde_json::from_slice(&receipt_bytes)
            .map_err(|error| artifact("receipt", error.to_string()))?;
        validate_receipt(&receipt)?;
        let cu = safe_file(directory, &receipt.cu_file, "cu")?;
        let ptx = safe_file(directory, &receipt.ptx_file, "ptx")?;
        let meta = safe_file(directory, &receipt.meta_file, "meta")?;
        let generator = safe_file(directory, &receipt.generator_file, "generator")?;
        let generation_log = safe_file(directory, &receipt.generation_log, "generation log")?;
        let generation_execution = safe_file(
            directory,
            &receipt.generation_execution_file,
            "generation execution",
        )?;
        for (name, bytes, expected) in [
            ("cu", cu.as_slice(), receipt.cu_sha256.as_str()),
            ("ptx", ptx.as_slice(), receipt.ptx_sha256.as_str()),
            ("meta", meta.as_slice(), receipt.meta_sha256.as_str()),
            (
                "generator",
                generator.as_slice(),
                receipt.generator_sha256.as_str(),
            ),
            (
                "generation log",
                generation_log.as_slice(),
                receipt.generation_log_sha256.as_str(),
            ),
            (
                "generation execution",
                generation_execution.as_slice(),
                receipt.generation_execution_sha256.as_str(),
            ),
        ] {
            if digest(bytes) != expected {
                return Err(artifact("hash", format!("{name} mismatch")));
            }
        }
        validate_cuda(&cu, &receipt.entry)?;
        let ptx = validate_ptx(&ptx, &receipt.entry, receipt.target_arch)?;
        let stream = super::cuda::transfer_stream(device)?;
        let module = stream
            .context()
            .load_module(Ptx::from_src(ptx))
            .map_err(cuda_error("frozen-warp-gravity-load"))?;
        let function = module
            .load_function(&receipt.entry)
            .map_err(cuda_error("frozen-warp-gravity-entry"))?;
        Ok(Self {
            stream,
            function,
            receipt,
        })
    }

    pub fn receipt(&self) -> &FrozenWarpGravityReceipt {
        &self.receipt
    }

    pub fn launch(&self, input: &FrozenWarpGravityInput) -> Result<Vec<f32>, ProbeError> {
        self.launch_diagnostic(input)?.into_finite()
    }

    /// 仅供诊断先归档原始位模式。
    /// 调用方随后必须调用`into_finite`。
    pub fn launch_diagnostic(
        &self,
        input: &FrozenWarpGravityInput,
    ) -> Result<FrozenWarpGravityReadback, ProbeError> {
        validate_input(input)?;
        let stream = &self.stream;
        let gravity = stream
            .clone_htod(&input.opt_gravity)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let parent = stream
            .clone_htod(&input.body_parentid)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let roots = stream
            .clone_htod(&input.body_rootid)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let mass = stream
            .clone_htod(&input.body_mass)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let gravcomp = stream
            .clone_htod(&input.body_gravcomp)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let dof = stream
            .clone_htod(&input.dof_bodyid)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let ancestors = stream
            .clone_htod(&input.body_isdofancestor)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let xipos = stream
            .clone_htod(&input.xipos)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let subtree = stream
            .clone_htod(&input.subtree_com)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let cdof = stream
            .clone_htod(&input.cdof)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let mut output = stream
            .alloc_zeros::<f32>(input.worlds * input.nv)
            .map_err(cuda_error("frozen-warp-gravity-output"))?;

        let (gravity_ptr, gravity_record) = gravity.device_ptr(stream);
        let (parent_ptr, parent_record) = parent.device_ptr(stream);
        let (roots_ptr, roots_record) = roots.device_ptr(stream);
        let (mass_ptr, mass_record) = mass.device_ptr(stream);
        let (gravcomp_ptr, gravcomp_record) = gravcomp.device_ptr(stream);
        let (dof_ptr, dof_record) = dof.device_ptr(stream);
        let (ancestor_ptr, ancestor_record) = ancestors.device_ptr(stream);
        let (xipos_ptr, xipos_record) = xipos.device_ptr(stream);
        let (subtree_ptr, subtree_record) = subtree.device_ptr(stream);
        let (cdof_ptr, cdof_record) = cdof.device_ptr(stream);
        let (output_ptr, output_record) = output.device_ptr_mut(stream);
        let mass_batches = input.body_mass.len() / input.nbody;
        let gravcomp_batches = input.body_gravcomp.len() / input.nbody;
        let descriptors = [
            contiguous(gravity_ptr, &[input.opt_gravity.len() / 3], 12)?,
            contiguous(parent_ptr, &[input.nbody], 4)?,
            contiguous(roots_ptr, &[input.nbody], 4)?,
            contiguous(mass_ptr, &[mass_batches, input.nbody], 4)?,
            contiguous(gravcomp_ptr, &[gravcomp_batches, input.nbody], 4)?,
            contiguous(dof_ptr, &[input.nv], 4)?,
            contiguous(ancestor_ptr, &[input.nbody, input.nv], 4)?,
            contiguous(xipos_ptr, &[input.worlds, input.nbody], 12)?,
            contiguous(subtree_ptr, &[input.worlds, input.nbody], 12)?,
            contiguous(cdof_ptr, &[input.worlds, input.nv], 24)?,
            contiguous(output_ptr, &[input.worlds, input.nv], 4)?,
        ];
        let dynamic_bodies = input.nbody - 1;
        let size = input
            .worlds
            .checked_mul(dynamic_bodies)
            .and_then(|value| value.checked_mul(input.nv))
            .ok_or_else(|| artifact("input", "launch size overflow"))?;
        let work = u32::try_from(size).map_err(|_| artifact("input", "launch overflow"))?;
        let bounds = WarpBounds3 {
            shape: [
                i32::try_from(input.worlds).map_err(|_| artifact("input", "world overflow"))?,
                i32::try_from(dynamic_bodies).map_err(|_| artifact("input", "body overflow"))?,
                i32::try_from(input.nv).map_err(|_| artifact("input", "dof overflow"))?,
            ],
            padding: 0,
            size: u64::from(work),
            coord_mult: 1,
        };
        let mut launch = stream.launch_builder(&self.function);
        launch.arg(&bounds);
        for descriptor in &descriptors {
            launch.arg(descriptor);
        }
        // SAFETY: 验证后的描述符覆盖全部输入。
        // 每线程只原子累加一个输出槽。
        unsafe {
            launch
                .launch(LaunchConfig {
                    grid_dim: (work.div_ceil(256), 1, 1),
                    block_dim: (256, 1, 1),
                    shared_mem_bytes: 0,
                })
                .map_err(cuda_error("frozen-warp-gravity-launch"))?;
        }
        drop(output_record);
        drop(cdof_record);
        drop(subtree_record);
        drop(xipos_record);
        drop(ancestor_record);
        drop(dof_record);
        drop(gravcomp_record);
        drop(mass_record);
        drop(roots_record);
        drop(parent_record);
        drop(gravity_record);
        let values = stream
            .clone_dtoh(&output)
            .map_err(cuda_error("frozen-warp-gravity-readback"))?;
        let bits = values.iter().map(|value| value.to_bits()).collect();
        Ok(FrozenWarpGravityReadback { values, bits })
    }
}

fn validate_receipt(receipt: &FrozenWarpGravityReceipt) -> Result<(), ProbeError> {
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

fn validate_input(input: &FrozenWarpGravityInput) -> Result<(), ProbeError> {
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

fn read(path: impl AsRef<Path>, stage: &'static str) -> Result<Vec<u8>, ProbeError> {
    fs::read(path).map_err(|error| artifact(stage, error.to_string()))
}

fn safe_file(directory: &Path, name: &str, stage: &'static str) -> Result<Vec<u8>, ProbeError> {
    if Path::new(name).file_name().and_then(|value| value.to_str()) != Some(name) {
        return Err(artifact(stage, "unsafe filename"));
    }
    let bytes = read(directory.join(name), stage)?;
    if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
        return Err(artifact(stage, "invalid file size"));
    }
    Ok(bytes)
}

fn validate_cuda(bytes: &[u8], entry: &str) -> Result<(), ProbeError> {
    if bytes.contains(&0) {
        return Err(artifact("cu", "embedded nul"));
    }
    let source = std::str::from_utf8(bytes).map_err(|error| artifact("cu", error.to_string()))?;
    let signature = signature_lines(source, &format!("extern \"C\" __global__ void {entry}("))?;
    if compact(&signature) != expected_cuda_signature(entry) {
        return Err(artifact("cu", "entry abi mismatch"));
    }
    Ok(())
}

fn validate_ptx(bytes: &[u8], entry: &str, arch: u32) -> Result<String, ProbeError> {
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
        return Err(artifact("ptx", "target or entry mismatch"));
    }
    let signature = signature_lines(source, &format!(".visible .entry {entry}("))?;
    if signature
        .iter()
        .map(|line| line.trim())
        .ne(expected_ptx_signature(entry).iter().map(String::as_str))
    {
        return Err(artifact("ptx", "entry abi mismatch"));
    }
    Ok(source.to_owned())
}

fn checked_product(left: usize, right: usize) -> Result<usize, ProbeError> {
    left.checked_mul(right)
        .ok_or_else(|| artifact("input", "length overflow"))
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
    String::from_utf8(output).unwrap()
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

fn expected_cuda_signature(entry: &str) -> String {
    let arrays = [
        "wp::array_t<wp::vec_t<3,wp::float32>>var_opt_gravity",
        "wp::array_t<wp::int32>var_body_parentid",
        "wp::array_t<wp::int32>var_body_rootid",
        "wp::array_t<wp::float32>var_body_mass",
        "wp::array_t<wp::float32>var_body_gravcomp",
        "wp::array_t<wp::int32>var_dof_bodyid",
        "wp::array_t<wp::int32>var_body_isdofancestor",
        "wp::array_t<wp::vec_t<3,wp::float32>>var_xipos_in",
        "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
        "wp::array_t<wp::vec_t<6,wp::float32>>var_cdof_in",
        "wp::array_t<wp::float32>var_qfrc_gravcomp_out",
    ];
    format!(
        "extern\"C\"__global__void{entry}(wp::launch_bounds_t<3>dim,{})",
        arrays.join(",")
    )
}

fn expected_ptx_signature(entry: &str) -> Vec<String> {
    let mut lines = vec![format!(".visible .entry {entry}(")];
    lines.push(format!(".param .align 8 .b8 {entry}_param_0[32],"));
    for index in 1..=11 {
        let comma = if index == 11 { "" } else { "," };
        lines.push(format!(
            ".param .align 8 .b8 {entry}_param_{index}[56]{comma}"
        ));
    }
    lines.push(")".into());
    lines
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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

    fn input() -> FrozenWarpGravityInput {
        FrozenWarpGravityInput {
            worlds: 1,
            nbody: 2,
            nv: 1,
            opt_gravity: vec![0.0, -1.0, 0.0],
            body_parentid: vec![0, 0],
            body_rootid: vec![0, 1],
            body_mass: vec![0.0, 1.0],
            body_gravcomp: vec![0.0, 1.0],
            dof_bodyid: vec![1],
            body_isdofancestor: vec![0, 1],
            xipos: vec![0.0; 6],
            subtree_com: vec![0.0; 6],
            cdof: vec![0.0; 6],
        }
    }

    #[test]
    fn validates_exact_frozen_input_shapes() {
        validate_input(&input()).unwrap();

        let mut zero_body = input();
        zero_body.nbody = 1;
        assert!(validate_input(&zero_body).is_err());

        let mut non_finite = input();
        non_finite.cdof[0] = f32::NAN;
        assert!(validate_input(&non_finite).is_err());

        let mut bad_topology = input();
        bad_topology.dof_bodyid[0] = 2;
        assert!(validate_input(&bad_topology).is_err());
    }

    #[test]
    fn rejects_overflow_and_embedded_ptx_nul() {
        assert!(checked_product(usize::MAX, 2).is_err());
        let mut ptx = format!(
            ".version 8.8\n.target sm_89\n.address_size 64\n{}\n",
            expected_ptx_signature(ENTRY).join("\n")
        )
        .into_bytes();
        ptx.extend_from_slice(b"\0x");
        assert!(validate_ptx(&ptx, ENTRY, 89).is_err());
    }
}
