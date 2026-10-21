//! 冻结重力核的Warp原生ABI与输入门禁。
//! 本目标不生成上游产物，也不启动Python。
#![cfg(feature = "cuda-probe")]

use cudarc::driver::DeviceRepr;
#[cfg(windows)]
use cudarc::{
    driver::{
        CudaContext, DevicePtr, DevicePtrMut, LaunchConfig, PushKernelArg, sys as driver_sys,
    },
    nvrtc::{CompileOptions, compile_ptx_with_opts, sys as nvrtc_sys},
};
use mjwarp_rs::runtime::{FrozenWarpGravityArtifact, FrozenWarpGravityInput};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, mem::size_of, path::Path, path::PathBuf};

const PACKAGE: &str = include_str!("../fixtures/gravity-upstream-abi/package.json");
const PACKAGE_SHA256: &str = "e8a103433c10c14a5aff41ed65ebbf737157280d6943c4c772101f0c72d830b7";
const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
#[cfg(windows)]
const DEVICE_PROBE_SOURCE: &str = include_str!("native/warp_gravity_abi.cu");
#[cfg(windows)]
const DEVICE_PROBE_ENTRY: &str = "_gravity_force_abi_probe_cuda_kernel_forward";
#[cfg(windows)]
const DEVICE_GRAVITY_ENTRY: &str = "_gravity_force_bridge_cuda_kernel_forward";

// 子模块：纯移动自原单文件，行为不变。
#[path = "t4_gravity_upstream_abi/abi.rs"]
mod abi;
#[path = "t4_gravity_upstream_abi/artifact.rs"]
mod artifact;
#[path = "t4_gravity_upstream_abi/package_gate.rs"]
mod package_gate;
#[path = "t4_gravity_upstream_abi/support.rs"]
mod support;
#[cfg(test)]
#[path = "t4_gravity_upstream_abi/test_descriptors.rs"]
mod test_descriptors;
#[cfg(test)]
#[path = "t4_gravity_upstream_abi/test_device.rs"]
mod test_device;
#[cfg(test)]
#[path = "t4_gravity_upstream_abi/test_frozen_install.rs"]
mod test_frozen_install;
#[cfg(test)]
#[path = "t4_gravity_upstream_abi/test_gate.rs"]
mod test_gate;
#[cfg(test)]
#[path = "t4_gravity_upstream_abi/test_import.rs"]
mod test_import;
