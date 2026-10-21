//! 冻结Warp数组、边界与工件身份类型。

use super::*;

// Warp 1.15 array_t按值传参。
// 显式尾填充固定全部参数字节。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WarpArray {
    pub(super) data: u64,
    pub(super) grad: u64,
    pub(super) shape: [i32; 4],
    pub(super) strides: [i32; 4],
    pub(super) ndim: u16,
    pub(super) flags: u16,
    pub(super) padding: u32,
}

// launch_bounds_t<3>含四字节对齐槽。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WarpBounds3 {
    pub(super) shape: [i32; 3],
    pub(super) padding: u32,
    pub(super) size: u64,
    pub(super) coord_mult: u64,
}

// SAFETY: 两个类型只含固定宽度值。
// repr(C)和原生探针绑定冻结头文件。
unsafe impl DeviceRepr for WarpArray {}
// SAFETY: 该布局逐字段匹配launch_bounds_t<3>。
unsafe impl DeviceRepr for WarpBounds3 {}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum GateError {
    Missing(&'static str),
    Io(&'static str),
    Parse(&'static str),
    Mismatch(&'static str),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactReceipt {
    pub(super) schema: u32,
    pub(super) evidence_kind: String,
    pub(super) upstream_revision: String,
    pub(super) warp_version: String,
    pub(super) module: String,
    pub(super) passive_source_sha256: String,
    pub(super) support_source_sha256: String,
    pub(super) codegen_source_sha256: String,
    pub(super) target_arch: u32,
    pub(super) mode: String,
    pub(super) fast_math: bool,
    pub(super) fuse_fp: bool,
    pub(super) entry: String,
    pub(super) launch_rank: usize,
    pub(super) array_arguments: usize,
    pub(super) cu_file: String,
    pub(super) ptx_file: String,
    pub(super) cu_sha256: String,
    pub(super) ptx_sha256: String,
    pub(super) meta_file: String,
    pub(super) meta_sha256: String,
    pub(super) python_version: String,
    pub(super) python_executable_sha256: String,
    pub(super) generator_file: String,
    pub(super) generator_sha256: String,
    pub(super) warp_dll_sha256: String,
    pub(super) cuda_toolkit: String,
    pub(super) ptx_compiler: String,
    pub(super) driver_version: String,
    pub(super) device: String,
    pub(super) generation_invocation_id: String,
    pub(super) generation_log: String,
    pub(super) generation_log_sha256: String,
    pub(super) generation_execution_file: String,
    pub(super) generation_execution_sha256: String,
    pub(super) compiler_provenance: String,
    pub(super) authoring_only: bool,
    pub(super) product_python_dependency: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ArtifactIdentity {
    pub(super) evidence_kind: String,
    pub(super) entry: String,
    pub(super) cu_sha256: String,
    pub(super) ptx_sha256: String,
}
