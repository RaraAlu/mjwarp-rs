//! 原始捕获回执核验。

use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    entry: String,
    source_sha256: String,
    ptx_sha256: String,
    precision_gate: String,
    cuda_math_reduction: String,
    passed: bool,
    nvrtc_version: [u32; 2],
    fmad: bool,
    use_fast_math: bool,
    options: Vec<String>,
}

/// 验证原回执与原始字节，不信任调用方改写的passed字段。
pub fn verify_capture(value: &Inventory, directory: &Path) -> Result<(), String> {
    validate_inventory(value)?;
    let mut observed = BTreeSet::new();
    for file in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        if file.path().extension().is_some_and(|e| e == "json") {
            let receipt: Receipt =
                serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if !value.excluded_test_entries.contains(&receipt.entry) {
                let expected = format!("{}-{}.json", receipt.entry, receipt.ptx_sha256);
                require(
                    file.file_name().to_str() == Some(&expected),
                    "noncanonical receipt name",
                )?;
                observed.insert((receipt.entry, receipt.ptx_sha256));
            }
        }
    }
    let expected: BTreeSet<_> = value
        .artifacts
        .iter()
        .map(|v| (v.entry.clone(), v.ptx_sha256.clone()))
        .collect();
    require(observed == expected, "capture coverage mismatch")?;
    for item in &value.artifacts {
        let stem = format!("{}-{}", item.entry, item.ptx_sha256);
        let raw = read_under(directory, &format!("{stem}.json"))?;
        require(digest(&raw) == item.receipt_sha256, "receipt hash mismatch")?;
        let receipt: Receipt = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        require(
            receipt.entry == item.entry
                && receipt.source_sha256 == item.source_sha256
                && receipt.ptx_sha256 == item.ptx_sha256,
            "receipt identity mismatch",
        )?;
        require(
            receipt.passed && receipt.precision_gate == "f32-physics",
            "precision gate rejected",
        )?;
        require(
            receipt.cuda_math_reduction == "CUDA-12.8 sinf/cosf integer angle reduction only",
            "unexpected math exception",
        )?;
        require(
            receipt.fmad && !receipt.use_fast_math && receipt.nvrtc_version == [12, 8],
            "compiler precision mismatch",
        )?;
        require(
            receipt.options == ["--std=c++17", "--gpu-architecture=compute_70"],
            "compiler options mismatch",
        )?;
        for (extension, hash) in [("cu", &item.source_sha256), ("ptx", &item.ptx_sha256)] {
            require(
                digest(&read_under(directory, &format!("{stem}.{extension}"))?) == *hash,
                "generated artifact hash mismatch",
            )?;
        }
    }
    Ok(())
}
