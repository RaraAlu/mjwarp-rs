//! 冻结工件候选校验与准入。

use super::abi::*;
use super::package_gate::validate_package;
use super::support::*;
use super::*;

pub(super) fn validate_artifact_candidate(
    package: &Value,
    directory: &Path,
) -> Result<ArtifactIdentity, GateError> {
    validate_package(package)?;
    let receipt_bytes = read(&directory.join("receipt.json"), "artifact receipt")?;
    let receipt: ArtifactReceipt =
        serde_json::from_slice(&receipt_bytes).map_err(|_| GateError::Parse("artifact receipt"))?;
    let requirements = &package["artifact_requirements"];
    let options = &requirements["compile_options"];
    if receipt.schema != 1
        || receipt.upstream_revision != REVISION
        || receipt.warp_version != WARP_VERSION
        || receipt.module != requirements["module"]
        || receipt.passive_source_sha256 != package["source"]["sha256"]
        || receipt.support_source_sha256 != package["source_dependencies"][0]["sha256"]
        || receipt.codegen_source_sha256 != package["warp_install"][4]["sha256"]
        || receipt.target_arch != options["target_arch"]
        || receipt.mode != options["mode"]
        || receipt.fast_math != options["fast_math"]
        || receipt.fuse_fp != options["fuse_fp"]
        || receipt.launch_rank != 3
        || receipt.array_arguments != 11
        || receipt.meta_file != "module.meta"
        || receipt.meta_sha256 != "4a80327bc21a875a63b9cf3dfc43386ed40119012357560339f071ac50ea461c"
        || receipt.python_version != "3.12.13"
        || receipt.python_executable_sha256
            != "feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77"
        || receipt.generator_file != "generate.py"
        || receipt.generator_sha256
            != "3f0348511e241a345cb3ae66b201cb02b1cef19d532b5980cd86a0e1700b0f2e"
        || receipt.warp_dll_sha256
            != "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6"
        || receipt.cuda_toolkit != "12.9"
        || receipt.ptx_compiler != "12.9.86"
        || receipt.driver_version != "596.36"
        || receipt.device != "NVIDIA GeForce RTX 4070 Ti SUPER"
        || receipt.generation_invocation_id != "exec-9a825267"
        || receipt.generation_log != "generation.log"
        || receipt.generation_log_sha256
            != "92626aabae26d638ad6d1c0e83461eac15a0d4c564afb9e3873dab9884578efa"
        || receipt.generation_execution_file != "generation-execution.json"
        || receipt.generation_execution_sha256
            != "8247ab2031fdccfd47d12ca8c3856150aed1c3d5539006cc65a0fd0f57edfd12"
        || receipt.compiler_provenance != "reconstructed_from_stdout_ptx_and_pinned_source_defaults"
        || !receipt.authoring_only
        || receipt.product_python_dependency
    {
        return Err(GateError::Mismatch("artifact receipt identity"));
    }
    let prefix = requirements["kernel_prefix"]
        .as_str()
        .ok_or(GateError::Parse("kernel prefix"))?;
    let suffix = requirements["kernel_suffix"]
        .as_str()
        .ok_or(GateError::Parse("kernel suffix"))?;
    if !receipt.entry.starts_with(prefix) || !receipt.entry.ends_with(suffix) {
        return Err(GateError::Mismatch("artifact entry"));
    }
    let cu = safe_file(directory, &receipt.cu_file, "artifact cu")?;
    let ptx = safe_file(directory, &receipt.ptx_file, "artifact ptx")?;
    if digest(&cu) != receipt.cu_sha256 || digest(&ptx) != receipt.ptx_sha256 {
        return Err(GateError::Mismatch("artifact hash"));
    }
    if receipt.evidence_kind == "frozen-generated" {
        for (file, expected, what) in [
            (&receipt.meta_file, &receipt.meta_sha256, "artifact meta"),
            (
                &receipt.generator_file,
                &receipt.generator_sha256,
                "artifact generator",
            ),
            (
                &receipt.generation_log,
                &receipt.generation_log_sha256,
                "artifact generation log",
            ),
            (
                &receipt.generation_execution_file,
                &receipt.generation_execution_sha256,
                "artifact execution",
            ),
        ] {
            if digest(&safe_file(directory, file, what)?) != *expected {
                return Err(GateError::Mismatch("artifact support hash"));
            }
        }
    }
    let cu = std::str::from_utf8(&cu).map_err(|_| GateError::Parse("artifact cu"))?;
    let cu_signature = signature_lines(
        cu,
        &format!("extern \"C\" __global__ void {}(", receipt.entry),
    )?;
    if compact(&cu_signature) != expected_cuda_signature(&receipt.entry) {
        return Err(GateError::Mismatch("cuda abi"));
    }
    let ptx = if ptx.last() == Some(&0) {
        &ptx[..ptx.len() - 1]
    } else {
        &ptx
    };
    let ptx = std::str::from_utf8(ptx).map_err(|_| GateError::Parse("artifact ptx"))?;
    let target = format!(".target sm_{}", receipt.target_arch);
    if !ptx.lines().any(|line| line.trim() == target) {
        return Err(GateError::Mismatch("ptx target"));
    }
    if !ptx.lines().any(|line| line.trim() == ".address_size 64") {
        return Err(GateError::Mismatch("ptx address size"));
    }
    let ptx_signature = signature_lines(ptx, &format!(".visible .entry {}(", receipt.entry))?;
    let expected = expected_ptx_signature(&receipt.entry);
    if ptx_signature
        .iter()
        .map(|line| line.trim())
        .ne(expected.iter().map(String::as_str))
    {
        return Err(GateError::Mismatch("ptx abi"));
    }
    Ok(ArtifactIdentity {
        evidence_kind: receipt.evidence_kind,
        entry: receipt.entry,
        cu_sha256: receipt.cu_sha256,
        ptx_sha256: receipt.ptx_sha256,
    })
}
