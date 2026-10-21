//! 冻结产物装载与身份核验。

use super::bytes::{read, safe_file, validate_cuda, validate_ptx};
use super::receipt::validate_receipt;
use super::{FrozenWarpGravityArtifact, FrozenWarpGravityReceipt, artifact, cuda_error, digest};
use crate::diagnostics::ProbeError;
use cudarc::nvrtc::Ptx;
use std::path::Path;

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
        let stream = super::super::cuda::transfer_stream(device)?;
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
}
