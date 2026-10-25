//! P1驱动级探针。
//! 探针不冻结运行时接口。

use crate::diagnostics::ProbeError;
pub mod external;
pub mod launcher;
pub mod tooling;
mod transfer;
pub(crate) use transfer::host_staging;
#[cfg(feature = "cuda-probe")]
pub(crate) use transfer::kernel::SynchronousKernel;
#[cfg(all(test, feature = "cuda-probe"))]
pub(crate) use transfer::kernel::{capture_child, capture_output};
pub use transfer::{TransferBuffer, TransferElement, TransferSession};

#[cfg(feature = "cuda-probe")]
mod cache;

#[cfg(any(feature = "cuda-probe", test))]
mod completion;
#[cfg(any(feature = "cuda-probe", test))]
mod lease;
#[cfg(any(feature = "cuda-probe", test))]
mod samples;

#[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
mod cubecl;
#[cfg(feature = "cuda-probe")]
mod cuda;
#[cfg(feature = "cuda-probe")]
mod warp_gravity;
#[cfg(feature = "cuda-probe")]
pub use warp_gravity::{
    FrozenWarpGravityArtifact, FrozenWarpGravityInput, FrozenWarpGravityReadback,
    FrozenWarpGravityReceipt,
};
#[cfg(feature = "cuda-probe")]
mod warp_com;
#[cfg(feature = "cuda-probe")]
pub use warp_com::{
    FrozenWarpComArtifact, FrozenWarpComInput, FrozenWarpComOutput, FrozenWarpComReadback,
    FrozenWarpComReceipt,
};
#[cfg(feature = "cuda-probe")]
mod warp_kinematics;
#[cfg(feature = "cuda-probe")]
pub use warp_kinematics::{
    FrozenWarpKinematicsArtifact, FrozenWarpKinematicsInput, FrozenWarpKinematicsReadback,
    FrozenWarpKinematicsReceipt,
};

mod config;
mod report;
mod run;
#[cfg(test)]
mod tests;

/// G5：碰撞/体积模型字段的 world 维上传布局与接触容量视图（纯 CPU 侧，供 GPU 窗口 launcher 消费）。
pub(crate) mod collision_upload;

pub use config::{ProbeBackend, ProbeConfig, ProbeKernel, ResourceProbeConfig};
pub use report::{
    ArtifactReport, ArtifactStatus, CachedProbeReport, ExternalResourceProbeReport, ProbeReport,
    ResourceProbeReport,
};
pub use run::{
    build_probe_artifact, run_cached_probe, run_external_resource_probe, run_probe,
    run_resource_probe,
};

const GUARD_ELEMENTS: usize = 16;
#[cfg(any(feature = "cuda-probe", test))]
const GUARD_VALUE: f32 = -12345.0;
const MAX_ELEMENTS: usize = 1 << 20;
const MAX_REPLAYS: usize = 1000;
const BLOCK_THREADS: usize = 128;

fn buffer_bytes(elements: usize) -> Result<usize, ProbeError> {
    elements
        .checked_add(GUARD_ELEMENTS)
        .and_then(|length| length.checked_mul(size_of::<f32>()))
        .ok_or(ProbeError::InvalidArgument("缓冲字节数溢出"))
}

#[cfg(any(feature = "cuda-probe", test))]
fn input_values(elements: usize, epoch: usize) -> Vec<f32> {
    // 二进制分数确保精确比较。
    (0..elements)
        .map(|index| ((index % 31) as f32 - 15.0) * 0.25 + epoch as f32)
        .collect()
}

#[cfg(any(feature = "cuda-probe", test))]
fn validate_output(input: &[f32], output: &[f32]) -> Result<(), ProbeError> {
    let expected_length = input.len() + GUARD_ELEMENTS;
    if output.len() != expected_length {
        return Err(ProbeError::InvalidOutputLength {
            expected: expected_length,
            actual: output.len(),
        });
    }
    for (index, &actual) in output.iter().enumerate() {
        let expected = if index < input.len() {
            input[index] * 2.0 + 1.0
        } else {
            GUARD_VALUE
        };
        if actual != expected {
            return Err(ProbeError::Mismatch {
                index,
                expected,
                actual,
            });
        }
    }
    Ok(())
}
