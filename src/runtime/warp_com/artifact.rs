//! 冻结产物加载与回读。

use std::path::Path;

use crate::diagnostics::ProbeError;

use super::format::{validate_cuda, validate_meta, validate_ptx};
use super::util::{artifact, checked_file, read};
use super::validate::{validate_input, validate_output, validate_raw, validate_receipt};
use super::*;

impl FrozenWarpComReadback {
    pub fn subtree_com(&self) -> &[f32] {
        &self.subtree_com
    }

    pub fn cdof(&self) -> &[f32] {
        &self.cdof
    }

    pub fn subtree_com_bits(&self) -> &[u32] {
        &self.subtree_com_bits
    }

    pub fn cdof_bits(&self) -> &[u32] {
        &self.cdof_bits
    }

    pub fn into_finite(self) -> Result<FrozenWarpComOutput, ProbeError> {
        if self
            .subtree_com
            .iter()
            .chain(&self.cdof)
            .any(|value| !value.is_finite())
        {
            return Err(artifact("output", "non-finite value"));
        }
        Ok(FrozenWarpComOutput {
            subtree_com: self.subtree_com,
            cdof: self.cdof,
        })
    }
}

impl FrozenWarpComArtifact {
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, ProbeError> {
        let directory = directory.as_ref();
        let receipt_bytes = read(directory.join("receipt.json"), "receipt")?;
        let receipt: FrozenWarpComReceipt = serde_json::from_slice(&receipt_bytes)
            .map_err(|error| artifact("receipt", error.to_string()))?;
        validate_receipt(&receipt)?;

        let input_bytes = checked_file(
            directory,
            &receipt.input_file,
            &receipt.input_sha256,
            "input",
        )?;
        let output_bytes = checked_file(
            directory,
            &receipt.output_file,
            &receipt.output_sha256,
            "output",
        )?;
        let raw_bytes = checked_file(
            directory,
            &receipt.raw_bits_file,
            &receipt.raw_bits_sha256,
            "raw bits",
        )?;
        let cu = checked_file(directory, &receipt.cu_file, &receipt.cu_sha256, "cu")?;
        let meta = checked_file(directory, &receipt.meta_file, &receipt.meta_sha256, "meta")?;
        let ptx = checked_file(directory, &receipt.ptx_file, &receipt.ptx_sha256, "ptx")?;
        for (name, file, expected) in [
            (
                "generator",
                receipt.generator_file.as_str(),
                receipt.generator_sha256.as_str(),
            ),
            (
                "authoring",
                receipt.authoring_file.as_str(),
                receipt.authoring_sha256.as_str(),
            ),
            (
                "generation log",
                receipt.generation_log.as_str(),
                receipt.generation_log_sha256.as_str(),
            ),
            (
                "first attempt log",
                receipt.first_attempt_log.as_str(),
                receipt.first_attempt_log_sha256.as_str(),
            ),
            (
                "generation executions",
                receipt.generation_executions_file.as_str(),
                receipt.generation_executions_sha256.as_str(),
            ),
        ] {
            checked_file(directory, file, expected, name)?;
        }

        validate_cuda(&cu)?;
        validate_ptx(&ptx, receipt.target_arch)?;
        validate_meta(&meta)?;
        let input: FrozenWarpComInput = serde_json::from_slice(&input_bytes)
            .map_err(|error| artifact("input", error.to_string()))?;
        validate_input(&input)?;
        let output: FrozenOutputFile = serde_json::from_slice(&output_bytes)
            .map_err(|error| artifact("output", error.to_string()))?;
        validate_output(&output, &input)?;
        let raw: RawBitsFile = serde_json::from_slice(&raw_bytes)
            .map_err(|error| artifact("raw bits", error.to_string()))?;
        let (subtree_com_bits, cdof_bits) = validate_raw(&raw, &output)?;
        Ok(Self {
            receipt,
            input,
            readback: FrozenWarpComReadback {
                subtree_com: output.subtree_com,
                cdof: output.cdof,
                subtree_com_bits,
                cdof_bits,
            },
        })
    }

    pub fn receipt(&self) -> &FrozenWarpComReceipt {
        &self.receipt
    }

    pub fn input(&self) -> &FrozenWarpComInput {
        &self.input
    }

    /// 调用方先归档位模式，再执行有限值门禁。
    pub fn readback(&self) -> FrozenWarpComReadback {
        self.readback.clone()
    }
}
