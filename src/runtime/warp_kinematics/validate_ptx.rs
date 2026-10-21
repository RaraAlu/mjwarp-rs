//! PTX架构与元数据校验。

use super::support::artifact;
use super::support::signature_lines;
use super::*;

pub(super) fn validate_ptx(bytes: &[u8], arch: u32) -> Result<String, ProbeError> {
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

pub(super) fn validate_meta(bytes: &[u8]) -> Result<(), ProbeError> {
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
