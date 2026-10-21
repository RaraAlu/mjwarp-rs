//! 物理PTX精度校验与证据记录。

use super::*;

// Warp's f32 sin/cos also call CUDA sinf/cosf. CUDA 12.8's slow integer angle
// reduction emits this exact s64 -> f64 -> f32 triplet. It does not promote a
// physical f32 value or accumulator. Reject every other non-f32 operation.
pub(crate) fn check_physics_ptx(ptx: &str, entry: &'static str) -> Result<(), ProbeError> {
    let lines: Vec<_> = ptx
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code).trim())
        .filter(|line| !line.is_empty())
        .collect();
    let operands = |line: &str| -> Vec<String> {
        line.split(|c: char| c.is_ascii_whitespace() || c == ',' || c == ';')
            .filter(|word| !word.is_empty())
            .map(str::to_owned)
            .collect()
    };
    let mut reduction_count = 0;
    let mut declarations = false;
    let mut i = 0;
    while i < lines.len() {
        let code = lines[i];
        if code.starts_with(".reg .f64 ") || code.starts_with(".reg .f64\t") {
            declarations = true;
            i += 1;
            continue;
        }
        let a = operands(code);
        if a.len() == 3 && a[0] == "cvt.rn.f64.s64" && i + 2 < lines.len() {
            let b = operands(lines[i + 1]);
            let c = operands(lines[i + 2]);
            if b.len() == 4
                // 未标舍入的mul默认采用rn。
                // FMA开关会改变库内指令拼写。
                && matches!(b[0].as_str(), "mul.rn.f64" | "mul.f64")
                && b[2] == a[1]
                && b[3] == "0d3BF921FB54442D19"
                && c.len() == 3
                && c[0] == "cvt.rn.f32.f64"
                && c[2] == b[1]
            {
                reduction_count += 1;
                i += 3;
                continue;
            }
        }
        for token in code
            .split_whitespace()
            .flat_map(|word| word.split('.').skip(1))
        {
            let token = token.trim_matches(|c: char| !c.is_ascii_alphanumeric());
            if matches!(token, "f64" | "f16" | "f16x2" | "bf16" | "bf16x2" | "tf32") {
                return Err(ProbeError::Compilation {
                    stage: "physics-precision-ptx",
                    detail: format!("{entry}: forbidden {token}: {code}"),
                });
            }
        }
        i += 1;
    }
    if declarations && reduction_count == 0 {
        return Err(ProbeError::Compilation {
            stage: "physics-precision-ptx",
            detail: format!("{entry}: unaccounted f64 registers"),
        });
    }
    Ok(())
}

pub(super) fn record_physics_ptx(
    source: &str,
    ptx: &str,
    entry: &'static str,
    passed: bool,
) -> Result<(), ProbeError> {
    let Some(directory) = std::env::var_os("MJWARP_PHYSICS_PTX_DIR") else {
        return Ok(());
    };
    let (mut major, mut minor) = (0, 0);
    // SAFETY: Compilation already loaded NVRTC; both output pointers are valid.
    unsafe { sys::nvrtcVersion(&mut major, &mut minor) }
        .result()
        .map_err(|error| ProbeError::Compilation {
            stage: "physics-precision-evidence",
            detail: format!("{entry}: NVRTC version: {error:?}"),
        })?;
    use sha2::{Digest, Sha256};
    let source_hash = format!("{:x}", Sha256::digest(source.as_bytes()));
    let ptx_hash = format!("{:x}", Sha256::digest(ptx.as_bytes()));
    let prefix = std::path::PathBuf::from(directory).join(format!("{entry}-{ptx_hash}"));
    let record = serde_json::json!({
        "entry": entry,
        "sourceSha256": source_hash,
        "ptxSha256": ptx_hash,
        "precisionGate": "f32-physics",
        "cudaMathReduction": "CUDA-12.8 sinf/cosf integer angle reduction only",
        "passed": passed,
        "nvrtcVersion": [major, minor],
        "fmad": PHYSICS_FMAD,
        "useFastMath": false,
        "options": ["--std=c++17", "--gpu-architecture=compute_70"],
    });
    let save = || -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(prefix.parent().unwrap())?;
        std::fs::write(prefix.with_extension("cu"), source)?;
        std::fs::write(prefix.with_extension("ptx"), ptx)?;
        std::fs::write(
            prefix.with_extension("json"),
            serde_json::to_vec_pretty(&record)?,
        )?;
        Ok(())
    };
    save().map_err(|error| ProbeError::Compilation {
        stage: "physics-precision-evidence",
        detail: format!("{entry}: {error}"),
    })
}
