//! 冻结产物文件读取与格式核验。

use super::artifact;
use super::signature::{compact, expected_cuda_signature, expected_ptx_signature, signature_lines};
use crate::diagnostics::ProbeError;
use std::{fs, path::Path};

pub(super) fn read(path: impl AsRef<Path>, stage: &'static str) -> Result<Vec<u8>, ProbeError> {
    fs::read(path).map_err(|error| artifact(stage, error.to_string()))
}

pub(super) fn safe_file(
    directory: &Path,
    name: &str,
    stage: &'static str,
) -> Result<Vec<u8>, ProbeError> {
    if Path::new(name).file_name().and_then(|value| value.to_str()) != Some(name) {
        return Err(artifact(stage, "unsafe filename"));
    }
    let bytes = read(directory.join(name), stage)?;
    if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
        return Err(artifact(stage, "invalid file size"));
    }
    Ok(bytes)
}

pub(super) fn validate_cuda(bytes: &[u8], entry: &str) -> Result<(), ProbeError> {
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

pub(super) fn validate_ptx(bytes: &[u8], entry: &str, arch: u32) -> Result<String, ProbeError> {
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
