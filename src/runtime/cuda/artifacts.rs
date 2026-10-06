//! 驱动加载器与产物编译适配。

use super::{
    DeviceInfo, GpuProgram, LoadedKernel, ScanKernels, cuda_error, prepare_device, run_prepared,
};
use crate::diagnostics::ProbeError;
use crate::runtime::{
    ArtifactReport, ArtifactStatus, BLOCK_THREADS, CachedProbeReport, ProbeBackend, ProbeConfig,
    ProbeKernel, cache,
};
use cudarc::nvrtc::Ptx;
use std::path::Path;

pub(in crate::runtime) fn build(
    config: ProbeConfig,
    directory: &Path,
    refresh: bool,
) -> Result<ArtifactReport, ProbeError> {
    let info = prepare_device(config.device)?;
    let key = cache::Key::new(config, info.capability, info.driver_api);
    cache::build(directory, key, config, refresh, || compile(config, &info))
}

fn compile(config: ProbeConfig, info: &DeviceInfo) -> Result<Vec<cache::Stage>, ProbeError> {
    if !config.backend.enabled() {
        return Err(ProbeError::FeatureDisabled(
            config.backend.required_feature(),
        ));
    }
    if config.backend == ProbeBackend::NativePtx {
        return Ok(vec![cache::Stage {
            entrypoint: "affine_probe".into(),
            abi: cache::abi(config.backend).into(),
            block_threads: BLOCK_THREADS as u32,
            shared_memory: 0,
            ptx: include_str!("../probe.ptx").into(),
        }]);
    }
    #[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
    {
        use crate::runtime::cubecl::{self, KernelStage};
        if config.backend == ProbeBackend::CubeClCpp {
            use cudarc::nvrtc::sys;
            // SAFETY: 只检测编译器库。
            if !unsafe { sys::is_culib_present() } {
                return Err(ProbeError::NvrtcUnavailable);
            }
            let (mut major, mut minor) = (0, 0);
            // SAFETY: 两个输出地址保持有效。
            unsafe { sys::nvrtcVersion(&mut major, &mut minor) }
                .result()
                .map_err(|e| cache::error("nvrtc-version", format!("{e:?}")))?;
            if (major, minor) != (12, 8) {
                return Err(cache::error(
                    "nvrtc-version",
                    format!("需要12.8；当前{major}.{minor}"),
                ));
            }
        }
        let stages = if config.kernel == ProbeKernel::GlobalScan {
            vec![
                KernelStage::Probe(config.kernel),
                KernelStage::ScanTotals,
                KernelStage::ScanOffsets,
            ]
        } else {
            vec![KernelStage::Probe(config.kernel)]
        };
        stages
            .into_iter()
            .map(|stage| {
                let compiled = cubecl::compile(
                    config.backend,
                    stage,
                    (info.capability.0 * 10 + info.capability.1) as u32,
                    info.driver_api,
                )?;
                Ok(cache::Stage {
                    entrypoint: compiled.entrypoint,
                    abi: cache::abi(config.backend).into(),
                    block_threads: BLOCK_THREADS as u32,
                    shared_memory: compiled.shared_memory,
                    ptx: compiled.ptx.to_src(),
                })
            })
            .collect()
    }
    #[cfg(not(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe")))]
    {
        let _ = info;
        Err(ProbeError::FeatureDisabled(
            config.backend.required_feature(),
        ))
    }
}

fn load(info: &DeviceInfo, stage: &cache::Stage) -> Result<LoadedKernel, ProbeError> {
    let module = info
        .context
        .load_module(Ptx::from_src(stage.ptx.clone()))
        .map_err(cuda_error("cached-ptx-load"))?;
    Ok(LoadedKernel {
        function: module
            .load_function(&stage.entrypoint)
            .map_err(cuda_error("cached-kernel-load"))?,
        shared_memory: stage.shared_memory,
    })
}

// 调用方保证全部PTX遵守探针ABI。
pub(in crate::runtime) unsafe fn run(
    config: ProbeConfig,
    directory: &Path,
    require_no_nvrtc: bool,
) -> Result<CachedProbeReport, ProbeError> {
    if require_no_nvrtc {
        // SAFETY: 只探测库，不启动编译。
        if unsafe { cudarc::nvrtc::sys::is_culib_present() } {
            return Err(ProbeError::NvrtcPresent);
        }
    }
    let info = prepare_device(config.device)?;
    let key = cache::Key::new(config, info.capability, info.driver_api);
    let payload = cache::read(directory, &key, config)?.ok_or_else(|| {
        cache::error(
            "missing",
            format!(
                "{}: {}",
                directory.display(),
                key.hash().unwrap_or_default()
            ),
        )
    })?;
    let artifact = cache::report(directory, &payload, ArtifactStatus::Hit)?;
    let main = load(&info, &payload.stages[0])?;
    let scan = if config.kernel == ProbeKernel::GlobalScan && config.elements > BLOCK_THREADS {
        Some(ScanKernels {
            totals: load(&info, &payload.stages[1])?,
            offsets: load(&info, &payload.stages[2])?,
        })
    } else {
        None
    };
    // 调用方担保固定ABI与内存访问。
    // 复用缓冲、事件与图的存活范围。
    let probe = run_prepared(config, info, GpuProgram { main, scan })?;
    Ok(CachedProbeReport { artifact, probe })
}
