//! 冻结内核签名提取与期望 ABI。

use super::artifact;
use crate::diagnostics::ProbeError;

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
    String::from_utf8(output).unwrap()
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

pub(super) fn expected_cuda_signature(entry: &str) -> String {
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

pub(super) fn expected_ptx_signature(entry: &str) -> Vec<String> {
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
