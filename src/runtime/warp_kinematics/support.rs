//! 文件、摘要与数值共用工具。

use super::*;

pub(super) fn checked_file(
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

pub(super) fn checked_path(
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

pub(super) fn rows(values: &[Vec<f32>], width: usize) -> bool {
    !values.is_empty() && values.iter().all(|row| row.len() == width)
}

pub(super) fn flatten(values: &[Vec<f32>]) -> Vec<f32> {
    values.iter().flatten().copied().collect()
}

pub(super) fn repeat_rows(values: &[Vec<f32>], count: usize) -> Vec<f32> {
    (0..count)
        .flat_map(|index| values[index % values.len()].iter().copied())
        .collect()
}

pub(super) fn checked_product(left: usize, right: usize) -> Result<usize, ProbeError> {
    left.checked_mul(right)
        .ok_or_else(|| artifact("input", "length overflow"))
}

pub(super) fn read(path: impl AsRef<Path>, stage: &'static str) -> Result<Vec<u8>, ProbeError> {
    fs::read(path).map_err(|error| artifact(stage, error.to_string()))
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn strip_comments(source: &str) -> String {
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

pub(super) fn signature_lines(source: &str, opening: &str) -> Result<Vec<String>, ProbeError> {
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

pub(super) fn compact(lines: &[String]) -> String {
    lines
        .iter()
        .flat_map(|line| line.chars())
        .filter(|value| !value.is_ascii_whitespace())
        .collect()
}

pub(super) fn artifact(stage: &'static str, detail: impl Into<String>) -> ProbeError {
    ProbeError::Artifact {
        stage,
        detail: detail.into(),
    }
}

pub(super) fn cuda_error(stage: &'static str) -> impl FnOnce(DriverError) -> ProbeError {
    move |error| ProbeError::Cuda {
        stage,
        code: error.0 as u32,
    }
}

pub(super) fn launcher_error(stage: &'static str) -> impl FnOnce(LauncherError) -> ProbeError {
    move |error| artifact(stage, format!("{error:?}"))
}

/// 冻结PTX来源：摘要取L2既有常量。
pub(super) fn frozen_ptx(bytes: &[u8]) -> Result<FrozenPtx, ProbeError> {
    FrozenPtx::from_file_bytes("frozen-warp-kinematics", bytes, PTX_SHA256)
        .map_err(launcher_error("frozen-warp-ptx"))
}

/// 冻结L2单精度数组实参：元素宽度显式给定。
pub(super) fn f32_arg(
    data: u64,
    shape: &[usize],
    element_bytes: u64,
    capacity: u64,
) -> Result<WarpArg, ProbeError> {
    crate::runtime::launcher::f32_arg(data, shape, element_bytes, capacity)
        .map_err(launcher_error("frozen-warp-array"))
}

/// 冻结L2整数数组实参。
pub(super) fn i32_arg(data: u64, shape: &[usize], capacity: u64) -> Result<WarpArg, ProbeError> {
    crate::runtime::launcher::i32_arg(data, shape, capacity)
        .map_err(launcher_error("frozen-warp-array"))
}

/// 二维边界加数组实参的统一发射。
pub(super) fn launch2(
    stream: &cudarc::driver::CudaStream,
    handle: &KernelHandle,
    shape: [usize; 2],
    args: &[WarpArg],
    stage: &'static str,
) -> Result<(), ProbeError> {
    let bounds =
        WarpBounds::Two(LaunchBounds2::new(shape).map_err(launcher_error("frozen-warp-bounds"))?);
    handle
        .launch_l2(stream, &WarpLaunch { bounds, args })
        .map_err(launcher_error(stage))
}
