//! P1驱动级探针。
//! 探针不冻结运行时接口。

use crate::diagnostics::ProbeError;
use std::path::{Path, PathBuf};
pub mod external;

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

const GUARD_ELEMENTS: usize = 16;
#[cfg(any(feature = "cuda-probe", test))]
const GUARD_VALUE: f32 = -12345.0;
const MAX_ELEMENTS: usize = 1 << 20;
const MAX_REPLAYS: usize = 1000;
const BLOCK_THREADS: usize = 128;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProbeKernel {
    #[default]
    Affine,
    AtomicSum,
    FloatAtomicSum,
    BlockReduce,
    BlockScan,
    GlobalScan,
    ControlFlow,
    SmallSolve,
}

impl ProbeKernel {
    pub const ALL: [Self; 8] = [
        Self::Affine,
        Self::AtomicSum,
        Self::FloatAtomicSum,
        Self::BlockReduce,
        Self::BlockScan,
        Self::GlobalScan,
        Self::ControlFlow,
        Self::SmallSolve,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Affine => "affine",
            Self::AtomicSum => "atomic-sum",
            Self::FloatAtomicSum => "float-atomic-sum",
            Self::BlockReduce => "block-reduce",
            Self::BlockScan => "block-scan",
            Self::GlobalScan => "global-scan",
            Self::ControlFlow => "control-flow",
            Self::SmallSolve => "small-solve",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProbeError> {
        Self::ALL
            .into_iter()
            .find(|kernel| kernel.name() == value)
            .ok_or(ProbeError::InvalidArgument("未知探针内核"))
    }

    fn output_elements(self, elements: usize) -> Result<usize, ProbeError> {
        match self {
            Self::AtomicSum | Self::FloatAtomicSum => Ok(1),
            Self::BlockReduce => Ok(elements.div_ceil(BLOCK_THREADS)),
            Self::SmallSolve => elements
                .checked_mul(2)
                .ok_or(ProbeError::InvalidArgument("矩阵缓冲长度溢出")),
            _ => Ok(elements),
        }
    }

    #[cfg(feature = "cuda-probe")]
    fn entrypoint(self) -> &'static str {
        match self {
            Self::Affine => "cubecl_affine_probe",
            Self::AtomicSum => "cubecl_atomic_sum_probe",
            Self::FloatAtomicSum => "cubecl_float_atomic_sum_probe",
            Self::BlockReduce => "cubecl_block_reduce_probe",
            Self::BlockScan => "cubecl_block_scan_probe",
            Self::GlobalScan => "cubecl_global_scan_probe",
            Self::ControlFlow => "cubecl_control_flow_probe",
            Self::SmallSolve => "cubecl_small_solve_probe",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProbeBackend {
    #[default]
    NativePtx,
    CubeClCpp,
    CubeClLlvm,
}

impl ProbeBackend {
    pub fn name(self) -> &'static str {
        match self {
            Self::NativePtx => "native-ptx",
            Self::CubeClCpp => "cubecl-cpp",
            Self::CubeClLlvm => "cubecl-llvm",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProbeError> {
        match value {
            "native-ptx" => Ok(Self::NativePtx),
            "cubecl-cpp" => Ok(Self::CubeClCpp),
            "cubecl-llvm" => Ok(Self::CubeClLlvm),
            _ => Err(ProbeError::InvalidArgument("未知编译路线")),
        }
    }

    fn required_feature(self) -> &'static str {
        match self {
            Self::NativePtx => "cuda-probe",
            Self::CubeClCpp => "cubecl-cpp-probe",
            Self::CubeClLlvm => "cubecl-llvm-probe",
        }
    }

    fn enabled(self) -> bool {
        match self {
            Self::NativePtx => cfg!(feature = "cuda-probe"),
            Self::CubeClCpp => cfg!(feature = "cubecl-cpp-probe"),
            Self::CubeClLlvm => cfg!(feature = "cubecl-llvm-probe"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProbeConfig {
    pub backend: ProbeBackend,
    pub kernel: ProbeKernel,
    pub device: usize,
    pub elements: usize,
    pub replays: usize,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            backend: ProbeBackend::NativePtx,
            kernel: ProbeKernel::Affine,
            device: 0,
            elements: 257,
            replays: 3,
        }
    }
}

impl ProbeConfig {
    pub fn validate(&self) -> Result<(), ProbeError> {
        if !(1..=MAX_ELEMENTS).contains(&self.elements) {
            return Err(ProbeError::InvalidArgument("元素数须在1至1048576"));
        }
        if !(1..=MAX_REPLAYS).contains(&self.replays) {
            return Err(ProbeError::InvalidArgument("重放数须在1至1000"));
        }
        if self.backend == ProbeBackend::NativePtx && self.kernel != ProbeKernel::Affine {
            return Err(ProbeError::InvalidArgument("原生PTX只支持affine探针"));
        }
        buffer_bytes(self.kernel.output_elements(self.elements)?)?;
        Ok(())
    }
}

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

/// 构建固定ABI的PTX缓存。
/// 命中时不加载前端编译器。
/// 损坏时不自动重建。
pub fn build_probe_artifact(
    config: ProbeConfig,
    directory: &Path,
    refresh: bool,
) -> Result<ArtifactReport, ProbeError> {
    config.validate()?;
    #[cfg(feature = "cuda-probe")]
    {
        std::panic::catch_unwind(|| cuda::artifacts::build(config, directory, refresh))
            .map_err(|_| ProbeError::BackendPanic)?
    }
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (directory, refresh);
        Err(ProbeError::FeatureDisabled("cuda-probe"))
    }
}

/// 只读取已有缓存，不调用编译器。
/// 摘要只验证完整性，不验证来源。
///
/// # Safety
/// 调用方须信任目录及全部PTX。
/// PTX须来自本探针编译流程。
/// PTX须遵守固定三参数ABI。
/// PTX不得越界或引入数据竞争。
/// 调用方须防止文件篡改。
pub unsafe fn run_cached_probe(
    config: ProbeConfig,
    directory: &Path,
    require_no_nvrtc: bool,
) -> Result<CachedProbeReport, ProbeError> {
    config.validate()?;
    #[cfg(feature = "cuda-probe")]
    {
        // SAFETY: 调用方保证产物可信。
        // 私有适配层保留模块与缓冲。
        std::panic::catch_unwind(|| unsafe {
            cuda::artifacts::run(config, directory, require_no_nvrtc)
        })
        .map_err(|_| ProbeError::BackendPanic)?
    }
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (directory, require_no_nvrtc);
        Err(ProbeError::FeatureDisabled("cuda-probe"))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceProbeConfig {
    pub backend: ProbeBackend,
    pub device: usize,
    pub elements: usize,
}

impl Default for ResourceProbeConfig {
    fn default() -> Self {
        let config = ProbeConfig::default();
        Self {
            backend: config.backend,
            device: config.device,
            elements: config.elements,
        }
    }
}

impl ResourceProbeConfig {
    fn kernel_config(self) -> ProbeConfig {
        ProbeConfig {
            backend: self.backend,
            device: self.device,
            elements: self.elements,
            kernel: ProbeKernel::Affine,
            replays: 1,
        }
    }
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

/// 验证私有外部导入与所有者移交。
/// 本接口不导出生产张量适配器。
pub fn run_external_resource_probe(
    config: ResourceProbeConfig,
) -> Result<ExternalResourceProbeReport, ProbeError> {
    config.kernel_config().validate()?;
    if !config.backend.enabled() {
        return Err(ProbeError::FeatureDisabled(
            config.backend.required_feature(),
        ));
    }
    #[cfg(feature = "cuda-probe")]
    {
        std::panic::catch_unwind(|| cuda::external::run(config))
            .map_err(|_| ProbeError::BackendPanic)?
    }
    #[cfg(not(feature = "cuda-probe"))]
    Err(ProbeError::FeatureDisabled("cuda-probe"))
}

/// 同步验证内部租约原型。
/// 此接口不导出外部设备视图。
pub fn run_resource_probe(config: ResourceProbeConfig) -> Result<ResourceProbeReport, ProbeError> {
    config.kernel_config().validate()?;
    if !config.backend.enabled() {
        return Err(ProbeError::FeatureDisabled(
            config.backend.required_feature(),
        ));
    }
    #[cfg(feature = "cuda-probe")]
    {
        std::panic::catch_unwind(|| cuda::run_resources(config))
            .map_err(|_| ProbeError::BackendPanic)?
    }
    #[cfg(not(feature = "cuda-probe"))]
    Err(ProbeError::FeatureDisabled("cuda-probe"))
}

/// 同步返回探针结果。
/// 函数不导出指针或在途资源。
pub fn run_probe(config: ProbeConfig) -> Result<ProbeReport, ProbeError> {
    config.validate()?;
    if !config.backend.enabled() {
        return Err(ProbeError::FeatureDisabled(
            config.backend.required_feature(),
        ));
    }
    #[cfg(feature = "cuda-probe")]
    {
        // 驱动加载器会对缺少符号执行panic。
        // 适配层返回错误，且不选择CPU。
        std::panic::catch_unwind(|| cuda::run(config)).map_err(|_| ProbeError::BackendPanic)?
    }
    #[cfg(not(feature = "cuda-probe"))]
    Err(ProbeError::FeatureDisabled("cuda-probe"))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_default_and_capacity_boundaries() {
        ProbeConfig::default().validate().unwrap();
        ProbeConfig {
            elements: 1,
            replays: 1,
            ..Default::default()
        }
        .validate()
        .unwrap();
        ProbeConfig {
            elements: MAX_ELEMENTS,
            replays: MAX_REPLAYS,
            ..Default::default()
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn rejects_invalid_elements_and_replays() {
        for elements in [0, MAX_ELEMENTS + 1, usize::MAX] {
            assert!(
                ProbeConfig {
                    elements,
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
        for replays in [0, MAX_REPLAYS + 1, usize::MAX] {
            assert!(
                ProbeConfig {
                    replays,
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
    }

    #[test]
    fn rejects_byte_count_overflow() {
        assert!(buffer_bytes(usize::MAX).is_err());
        assert!(buffer_bytes(usize::MAX / size_of::<f32>()).is_err());
        assert_eq!(buffer_bytes(257).unwrap(), 1092);
    }

    #[test]
    fn validates_exact_results_and_guard() {
        let input = input_values(257, 3);
        let output: Vec<_> = input
            .iter()
            .map(|value| value * 2.0 + 1.0)
            .chain([GUARD_VALUE; GUARD_ELEMENTS])
            .collect();
        validate_output(&input, &output).unwrap();
    }

    #[test]
    fn rejects_short_and_long_output() {
        for output in [vec![], vec![3.0; GUARD_ELEMENTS + 2]] {
            assert!(matches!(
                validate_output(&[1.0], &output),
                Err(ProbeError::InvalidOutputLength { .. })
            ));
        }
    }

    #[test]
    fn detects_bad_values_nonfinite_values_and_tail_writes() {
        let mut output = vec![GUARD_VALUE; 1 + GUARD_ELEMENTS];
        for value in [0.0, f32::NAN, f32::INFINITY] {
            output[0] = value;
            assert!(matches!(
                validate_output(&[1.0], &output),
                Err(ProbeError::Mismatch { index: 0, .. })
            ));
        }
        output[0] = 3.0;
        output[1] = 0.0;
        assert!(matches!(
            validate_output(&[1.0], &output),
            Err(ProbeError::Mismatch { index: 1, .. })
        ));
    }

    #[test]
    fn changes_inputs_between_graph_replays() {
        let input = input_values(1, 0);
        let old_output: Vec<_> = [input[0] * 2.0 + 1.0]
            .into_iter()
            .chain([GUARD_VALUE; GUARD_ELEMENTS])
            .collect();
        assert!(validate_output(&input_values(1, 1), &old_output).is_err());
    }

    #[cfg(not(feature = "cuda-probe"))]
    #[test]
    fn rejects_missing_feature_without_cpu_fallback() {
        assert_eq!(
            run_probe(ProbeConfig::default()).unwrap_err(),
            ProbeError::FeatureDisabled("cuda-probe")
        );
    }

    #[test]
    fn parses_only_explicit_backend_names() {
        for backend in [
            ProbeBackend::NativePtx,
            ProbeBackend::CubeClCpp,
            ProbeBackend::CubeClLlvm,
        ] {
            assert_eq!(ProbeBackend::parse(backend.name()).unwrap(), backend);
        }
        assert!(ProbeBackend::parse("auto").is_err());
        assert!(ProbeBackend::parse("cpu").is_err());
    }

    #[test]
    fn rejects_disabled_routes_without_substitution() {
        for backend in [
            ProbeBackend::NativePtx,
            ProbeBackend::CubeClCpp,
            ProbeBackend::CubeClLlvm,
        ] {
            if !backend.enabled() {
                assert_eq!(
                    run_external_resource_probe(ResourceProbeConfig {
                        backend,
                        ..Default::default()
                    })
                    .unwrap_err(),
                    ProbeError::FeatureDisabled(backend.required_feature())
                );
                assert_eq!(
                    run_resource_probe(ResourceProbeConfig {
                        backend,
                        ..Default::default()
                    })
                    .unwrap_err(),
                    ProbeError::FeatureDisabled(backend.required_feature())
                );
                for kernel in ProbeKernel::ALL {
                    if backend == ProbeBackend::NativePtx && kernel != ProbeKernel::Affine {
                        continue;
                    }
                    assert_eq!(
                        run_probe(ProbeConfig {
                            backend,
                            kernel,
                            ..Default::default()
                        })
                        .unwrap_err(),
                        ProbeError::FeatureDisabled(backend.required_feature())
                    );
                }
            }
        }
    }

    #[test]
    fn rejects_invalid_resource_dimensions_before_loading_driver() {
        for elements in [0, MAX_ELEMENTS + 1, usize::MAX] {
            assert!(matches!(
                run_external_resource_probe(ResourceProbeConfig {
                    elements,
                    ..Default::default()
                }),
                Err(ProbeError::InvalidArgument(_))
            ));
            assert!(matches!(
                run_resource_probe(ResourceProbeConfig {
                    elements,
                    ..Default::default()
                }),
                Err(ProbeError::InvalidArgument(_))
            ));
        }
    }

    #[test]
    fn parses_kernels_and_rejects_native_route_substitution() {
        for kernel in ProbeKernel::ALL {
            assert_eq!(ProbeKernel::parse(kernel.name()).unwrap(), kernel);
            let config = ProbeConfig {
                kernel,
                ..Default::default()
            };
            assert_eq!(config.validate().is_ok(), kernel == ProbeKernel::Affine);
            ProbeConfig {
                backend: ProbeBackend::CubeClCpp,
                ..config
            }
            .validate()
            .unwrap();
        }
        assert!(ProbeKernel::parse("auto").is_err());
        assert!(ProbeKernel::parse("unknown").is_err());
    }

    #[test]
    fn calculates_kernel_output_capacity() {
        assert_eq!(ProbeKernel::AtomicSum.output_elements(257).unwrap(), 1);
        assert_eq!(ProbeKernel::FloatAtomicSum.output_elements(257).unwrap(), 1);
        assert_eq!(ProbeKernel::BlockReduce.output_elements(257).unwrap(), 3);
        assert_eq!(ProbeKernel::BlockScan.output_elements(257).unwrap(), 257);
        assert_eq!(ProbeKernel::GlobalScan.output_elements(257).unwrap(), 257);
        assert_eq!(ProbeKernel::SmallSolve.output_elements(257).unwrap(), 514);
        assert!(ProbeKernel::SmallSolve.output_elements(usize::MAX).is_err());
    }
}
