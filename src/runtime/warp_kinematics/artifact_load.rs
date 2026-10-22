//! 冻结工件加载与访问器。

use super::support::artifact;
use super::support::checked_file;
use super::support::checked_path;
use super::support::read;
use super::validate_cuda::validate_cuda;
use super::validate_input::validate_input;
use super::validate_output::validate_output;
use super::validate_output::validate_raw;
use super::validate_ptx::validate_meta;
use super::validate_ptx::validate_ptx;
use super::validate_receipt::validate_receipt;
use super::*;

impl FrozenWarpKinematicsArtifact {
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, ProbeError> {
        let directory = directory.as_ref();
        let receipt: FrozenWarpKinematicsReceipt =
            serde_json::from_slice(&read(directory.join("receipt.json"), "receipt")?)
                .map_err(|error| artifact("receipt", error.to_string()))?;
        validate_receipt(&receipt)?;
        for &(path, expected) in &ATTEMPT_FILES {
            checked_path(directory, path, expected, "attempt")?;
        }
        for item in &receipt.attempt_statuses {
            checked_path(directory, &item.path, &item.sha256, "attempt status")?;
        }
        checked_path(
            directory,
            &receipt.prevalidation_readback_file,
            &receipt.prevalidation_readback_sha256,
            "prevalidation readback",
        )?;
        checked_path(
            directory,
            &receipt.prevalidation_state_file,
            &receipt.prevalidation_state_sha256,
            "prevalidation state",
        )?;
        let input_bytes = checked_file(directory, &receipt.input_file, INPUT_SHA256, "input")?;
        let output_bytes = checked_file(directory, &receipt.output_file, OUTPUT_SHA256, "output")?;
        let raw_bytes = checked_file(directory, &receipt.raw_bits_file, RAW_SHA256, "raw bits")?;
        let cu = checked_file(directory, &receipt.cu_file, CU_SHA256, "cu")?;
        let meta = checked_file(directory, &receipt.meta_file, META_SHA256, "meta")?;
        let ptx = checked_file(directory, &receipt.ptx_file, PTX_SHA256, "ptx")?;
        checked_file(
            directory,
            &receipt.generator_file,
            GENERATOR_SHA256,
            "generator",
        )?;
        checked_file(
            directory,
            &receipt.authoring_file,
            AUTHORING_SHA256,
            "authoring",
        )?;
        checked_file(
            directory,
            &receipt.generation_executions_file,
            EXECUTIONS_SHA256,
            "generation executions",
        )?;

        validate_cuda(&cu)?;
        validate_ptx(&ptx, receipt.target_arch)?;
        let frozen = super::support::frozen_ptx(&ptx)?;
        validate_meta(&meta)?;
        let input: FrozenWarpKinematicsInput = serde_json::from_slice(&input_bytes)
            .map_err(|error| artifact("input", error.to_string()))?;
        validate_input(&input)?;
        let output: FrozenOutputFile = serde_json::from_slice(&output_bytes)
            .map_err(|error| artifact("output", error.to_string()))?;
        validate_output(&output, &input)?;
        let raw: RawBitsFile = serde_json::from_slice(&raw_bytes)
            .map_err(|error| artifact("raw bits", error.to_string()))?;
        let bits = validate_raw(&raw, &output)?;
        let readback = FrozenWarpKinematicsReadback {
            xpos: output.xpos,
            xquat_wxyz: output.xquat_wxyz,
            xipos: output.xipos,
            xmat: output.xmat,
            xanchor: output.xanchor,
            xaxis: output.xaxis,
            subtree_com: output.subtree_com,
            cdof: output.cdof,
            bits,
        };
        Ok(Self {
            receipt,
            input,
            readback,
            frozen,
        })
    }

    pub fn receipt(&self) -> &FrozenWarpKinematicsReceipt {
        &self.receipt
    }

    pub fn input(&self) -> &FrozenWarpKinematicsInput {
        &self.input
    }

    /// 已通过摘要准入的冻结PTX来源。
    pub fn frozen_ptx(&self) -> &FrozenPtx {
        &self.frozen
    }

    pub fn readback(&self) -> FrozenWarpKinematicsReadback {
        self.readback.clone()
    }
}
