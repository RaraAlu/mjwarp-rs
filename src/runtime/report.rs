//! 探针报告结构。

use super::{ProbeBackend, ProbeKernel};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ProbeReport {
    pub backend: ProbeBackend,
    pub kernel: ProbeKernel,
    /// 仅CubeCL路线记录冻结提交。
    pub compiler_revision: Option<&'static str>,
    pub os: &'static str,
    pub arch: &'static str,
    pub device: usize,
    pub gpu_name: String,
    pub compute_capability: (i32, i32),
    /// CUDA驱动API版本，不是驱动发行号。
    pub driver_api_version: i32,
    pub elements: usize,
    pub buffer_bytes: usize,
    pub graph_replays: usize,
    /// 实际捕获的内核节点数。
    pub graph_kernel_nodes: usize,
    /// 成功更新的节点参数次数。
    pub graph_node_updates: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactStatus {
    Hit,
    Compiled,
}

#[derive(Debug)]
pub struct ArtifactReport {
    pub path: PathBuf,
    pub key_sha256: String,
    pub stages: usize,
    pub status: ArtifactStatus,
}

#[derive(Debug)]
pub struct CachedProbeReport {
    pub artifact: ArtifactReport,
    pub probe: ProbeReport,
}

#[derive(Debug)]
pub struct ResourceProbeReport {
    pub backend: ProbeBackend,
    pub compiler_revision: Option<&'static str>,
    pub os: &'static str,
    pub arch: &'static str,
    pub device: usize,
    pub gpu_name: String,
    pub compute_capability: (i32, i32),
    pub driver_api_version: i32,
    pub elements: usize,
    pub view_bytes: usize,
    pub output_owner_bytes: usize,
    pub completed_submissions: usize,
    /// GPU提交后模拟宿主错误。
    /// 此项不注入设备故障。
    pub failed_submissions: usize,
    pub rejected_requests: usize,
    pub graph_replays: usize,
}

#[derive(Debug)]
pub struct ExternalResourceProbeReport {
    pub resources: ResourceProbeReport,
    pub imported_allocations: usize,
    pub released_owners: usize,
    pub producer_dependencies: usize,
}
