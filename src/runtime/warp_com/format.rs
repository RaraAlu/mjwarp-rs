//! CUDA、PTX与元数据格式校验。

use crate::diagnostics::ProbeError;

use super::util::{artifact, compact, signature_lines};
use super::*;

pub(super) fn validate_cuda(bytes: &[u8]) -> Result<(), ProbeError> {
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

pub(super) fn validate_ptx(bytes: &[u8], arch: u32) -> Result<(), ProbeError> {
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

pub(super) fn validate_meta(bytes: &[u8]) -> Result<(), ProbeError> {
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
