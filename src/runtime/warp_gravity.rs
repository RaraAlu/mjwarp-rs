//! 冻结Warp重力产物导入器。
//! 它只接收固定来源的离线产物。

mod bytes;
mod launch;
mod load;
mod receipt;
mod signature;
#[cfg(test)]
mod tests;

use crate::diagnostics::ProbeError;
use crate::runtime::launcher::{FrozenPtx, KernelHandle, LauncherError, ModuleCache};
use cudarc::driver::{CudaStream, DriverError};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
const PASSIVE_SHA256: &str = "1334dfb73bde5980d54e1d112b10d1429554a64089a161debe78cf543f7aed8e";
const SUPPORT_SHA256: &str = "b06cdd49d49be73773674ff777fa4544fe5b75ca47b6560891a77fbb9f703ea3";
const CODEGEN_SHA256: &str = "d07a778d7f754dd74620da5c9b2c161b98f9605bc3ddb01c59907f77bd76a170";
const ENTRY: &str = "_gravity_force_7c63ac28_cuda_kernel_forward";
const CU_SHA256: &str = "dd9338e9f38feb767273c3e1a445aea54bee3dee7d899076363202a4d26e865d";
const PTX_SHA256: &str = "d06154da63f64118e8fdb19c0b646a59c5b485d580f4716e5edea97a57c1dd70";
const META_SHA256: &str = "4a80327bc21a875a63b9cf3dfc43386ed40119012357560339f071ac50ea461c";
const GENERATOR_SHA256: &str = "3f0348511e241a345cb3ae66b201cb02b1cef19d532b5980cd86a0e1700b0f2e";
const GENERATION_LOG_SHA256: &str =
    "92626aabae26d638ad6d1c0e83461eac15a0d4c564afb9e3873dab9884578efa";
const GENERATION_EXECUTION_SHA256: &str =
    "8247ab2031fdccfd47d12ca8c3856150aed1c3d5539006cc65a0fd0f57edfd12";
const PYTHON_SHA256: &str = "feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77";
const WARP_DLL_SHA256: &str = "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenWarpGravityReceipt {
    pub schema: u32,
    pub evidence_kind: String,
    pub upstream_revision: String,
    pub warp_version: String,
    pub module: String,
    pub passive_source_sha256: String,
    pub support_source_sha256: String,
    pub codegen_source_sha256: String,
    pub target_arch: u32,
    pub mode: String,
    pub fast_math: bool,
    pub fuse_fp: bool,
    pub entry: String,
    pub launch_rank: usize,
    pub array_arguments: usize,
    pub cu_file: String,
    pub ptx_file: String,
    pub meta_file: String,
    pub cu_sha256: String,
    pub ptx_sha256: String,
    pub meta_sha256: String,
    pub python_version: String,
    pub python_executable_sha256: String,
    pub generator_file: String,
    pub generator_sha256: String,
    pub warp_dll_sha256: String,
    pub cuda_toolkit: String,
    pub ptx_compiler: String,
    pub driver_version: String,
    pub device: String,
    pub generation_invocation_id: String,
    pub generation_log: String,
    pub generation_log_sha256: String,
    pub generation_execution_file: String,
    pub generation_execution_sha256: String,
    pub compiler_provenance: String,
    pub authoring_only: bool,
    pub product_python_dependency: bool,
}

#[derive(Clone, Debug)]
pub struct FrozenWarpGravityInput {
    pub worlds: usize,
    pub nbody: usize,
    pub nv: usize,
    pub opt_gravity: Vec<f32>,
    pub body_parentid: Vec<i32>,
    pub body_rootid: Vec<i32>,
    pub body_mass: Vec<f32>,
    pub body_gravcomp: Vec<f32>,
    pub dof_bodyid: Vec<i32>,
    pub body_isdofancestor: Vec<i32>,
    pub xipos: Vec<f32>,
    pub subtree_com: Vec<f32>,
    pub cdof: Vec<f32>,
}

pub struct FrozenWarpGravityArtifact {
    stream: Arc<CudaStream>,
    handle: KernelHandle,
    receipt: FrozenWarpGravityReceipt,
}

/// 保留冻结内核的原始设备回读。
/// 调用方先归档位模式，再执行有限值门禁。
pub struct FrozenWarpGravityReadback {
    values: Vec<f32>,
    bits: Vec<u32>,
}

impl FrozenWarpGravityReadback {
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn bits(&self) -> &[u32] {
        &self.bits
    }

    pub fn into_finite(self) -> Result<Vec<f32>, ProbeError> {
        if self.values.iter().any(|value| !value.is_finite()) {
            return Err(artifact("output", "non-finite value"));
        }
        Ok(self.values)
    }
}

fn checked_product(left: usize, right: usize) -> Result<usize, ProbeError> {
    left.checked_mul(right)
        .ok_or_else(|| artifact("input", "length overflow"))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn artifact(stage: &'static str, detail: impl Into<String>) -> ProbeError {
    ProbeError::Artifact {
        stage,
        detail: detail.into(),
    }
}

fn launcher_error(stage: &'static str) -> impl FnOnce(LauncherError) -> ProbeError {
    move |error| artifact(stage, format!("{error:?}"))
}

fn cuda_error(stage: &'static str) -> impl FnOnce(DriverError) -> ProbeError {
    move |error| ProbeError::Cuda {
        stage,
        code: error.0 as u32,
    }
}
