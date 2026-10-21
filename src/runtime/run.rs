//! 探针执行入口与产物缓存构建。

use super::{
    ArtifactReport, CachedProbeReport, ExternalResourceProbeReport, ProbeConfig, ProbeReport,
    ResourceProbeConfig, ResourceProbeReport,
};
use crate::diagnostics::ProbeError;
use std::path::Path;

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
        std::panic::catch_unwind(|| super::cuda::artifacts::build(config, directory, refresh))
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
            super::cuda::artifacts::run(config, directory, require_no_nvrtc)
        })
        .map_err(|_| ProbeError::BackendPanic)?
    }
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (directory, require_no_nvrtc);
        Err(ProbeError::FeatureDisabled("cuda-probe"))
    }
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
        std::panic::catch_unwind(|| super::cuda::external::run(config))
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
        std::panic::catch_unwind(|| super::cuda::run_resources(config))
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
        std::panic::catch_unwind(|| super::cuda::run(config))
            .map_err(|_| ProbeError::BackendPanic)?
    }
    #[cfg(not(feature = "cuda-probe"))]
    Err(ProbeError::FeatureDisabled("cuda-probe"))
}
