//! 集中封装CUDA适配。
mod graph;
mod scan;
#[cfg(test)]
mod tests;
mod typed;

#[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
use super::BLOCK_THREADS;
use super::{ProbeBackend, ProbeConfig, ProbeKernel, ProbeReport, buffer_bytes, samples};
use crate::diagnostics::ProbeError;
use cudarc::driver::{CudaContext, CudaFunction, CudaStream, DriverError, sys};
use cudarc::nvrtc::Ptx;
use std::sync::Arc;

use typed::run_typed;

pub(super) mod artifacts;
pub(super) mod external;
mod resources;
pub(super) use resources::run as run_resources;

fn cuda_error(stage: &'static str) -> impl FnOnce(DriverError) -> ProbeError {
    move |error| ProbeError::Cuda {
        stage,
        code: error.0 as u32,
    }
}

/// CLI/探针前置：NVIDIA 驱动库不可用时返回可诊断的 `DriverUnavailable`（不 panic、不探测设备）。
pub(crate) fn require_driver() -> Result<(), ProbeError> {
    if super::presence::driver_library_present() {
        Ok(())
    } else {
        Err(ProbeError::DriverUnavailable)
    }
}

/// CLI/探针前置：NVRTC 库不可用时返回可诊断的 `NvrtcUnavailable`（不 panic、不调用编译器）。
///
/// 产品构建里只有 cubecl 编译后端会调用它，默认 `cuda` 构建无调用者；测试构建保留该前置用于
/// 覆盖「有设备无 NVRTC」的 CLI 错误路径。
#[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe", test))]
pub(crate) fn require_nvrtc() -> Result<(), ProbeError> {
    if super::presence::nvrtc_library_present() {
        Ok(())
    } else {
        Err(ProbeError::NvrtcUnavailable)
    }
}

struct TrackingPause<'a>(&'a CudaContext);

impl<'a> TrackingPause<'a> {
    fn after_warmup(context: &'a CudaContext) -> Self {
        // SAFETY: 调用方已等待全部预热。
        // 调用方独占本探针的资源。
        // 外部导入器先等待生产事件。
        // 捕获仅记录内核，且不释放缓冲。
        unsafe { context.disable_event_tracking() };
        Self(context)
    }
}

impl Drop for TrackingPause<'_> {
    fn drop(&mut self) {
        // SAFETY: 捕获期间不创建缓冲。
        // 全部缓冲保留原有事件记录。
        unsafe { self.0.enable_event_tracking() };
    }
}

// 私有图句柄不导出设备资源。
struct ProbeGraph {
    raw: sys::CUgraph,
    exec: sys::CUgraphExec,
    stream: Arc<CudaStream>,
    kernels: Vec<CapturedKernel>,
    backend: ProbeBackend,
}

struct CapturedKernel {
    node: sys::CUgraphNode,
    params: sys::CUDA_KERNEL_NODE_PARAMS,
    // 保存原始参数值，不借用节点内存。
    arguments: [u64; 3],
}

struct LoadedKernel {
    function: CudaFunction,
    shared_memory: u32,
}

struct ScanKernels {
    totals: LoadedKernel,
    offsets: LoadedKernel,
}

struct GpuProgram {
    main: LoadedKernel,
    scan: Option<ScanKernels>,
}

#[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
fn load_cubecl(
    context: &Arc<CudaContext>,
    backend: ProbeBackend,
    stage: super::cubecl::KernelStage,
    sm: u32,
    driver: i32,
) -> Result<LoadedKernel, ProbeError> {
    let compiled = super::cubecl::compile(backend, stage, sm, driver)?;
    let module = context
        .load_module(compiled.ptx)
        .map_err(cuda_error("ptx-load"))?;
    Ok(LoadedKernel {
        function: module
            .load_function(&compiled.entrypoint)
            .map_err(cuda_error("kernel-load"))?,
        shared_memory: compiled.shared_memory,
    })
}

struct DeviceInfo {
    context: Arc<CudaContext>,
    gpu_name: String,
    capability: (i32, i32),
    driver_api: i32,
}

fn prepare_device(device: usize) -> Result<DeviceInfo, ProbeError> {
    // 驱动库存在性预检：缺库时返回 `DriverUnavailable`，不进入 CUDA 适配层（cudarc 缺库会 panic）。
    require_driver()?;
    let mut version = 0;
    // SAFETY: 驱动写入有效i32地址。
    // version在调用期间保持存活。
    unsafe { sys::cuDriverGetVersion(&mut version) }
        .result()
        .map_err(cuda_error("driver-version"))?;
    if version < 12000 {
        return Err(ProbeError::UnsupportedDriver { version });
    }
    // 测试 seam 可注入设备数（覆盖「有驱动无设备」的 CLI 错误路径）；产品构建直接查 cudarc。
    #[cfg(test)]
    let available: i32 = match super::presence::device_count_override() {
        Some(count) => count as i32,
        None => CudaContext::device_count().map_err(cuda_error("device-count"))?,
    };
    #[cfg(not(test))]
    let available = CudaContext::device_count().map_err(cuda_error("device-count"))?;
    if device >= available as usize {
        return Err(ProbeError::InvalidDevice {
            requested: device,
            available,
        });
    }
    let context = CudaContext::new(device).map_err(cuda_error("context"))?;
    let gpu_name = context.name().map_err(cuda_error("device-name"))?;
    let (major, minor) = context
        .compute_capability()
        .map_err(cuda_error("compute-capability"))?;
    if major < 7 {
        return Err(ProbeError::UnsupportedDevice { major, minor });
    }
    Ok(DeviceInfo {
        context,
        gpu_name,
        capability: (major, minor),
        driver_api: version,
    })
}

pub(super) fn transfer_stream(device: usize) -> Result<Arc<CudaStream>, ProbeError> {
    prepare_device(device)?
        .context
        .new_stream()
        .map_err(cuda_error("transfer-stream"))
}

fn prepare(config: ProbeConfig) -> Result<(DeviceInfo, GpuProgram), ProbeError> {
    let info = prepare_device(config.device)?;
    let context = &info.context;
    let (major, minor) = info.capability;
    let version = info.driver_api;
    let _ = (major, minor, version);
    let main = match config.backend {
        ProbeBackend::NativePtx => {
            let module = context
                .load_module(Ptx::from_src(include_str!("probe.ptx")))
                .map_err(cuda_error("ptx-load"))?;
            LoadedKernel {
                function: module
                    .load_function("affine_probe")
                    .map_err(cuda_error("kernel-load"))?,
                shared_memory: 0,
            }
        }
        #[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
        backend => load_cubecl(
            context,
            backend,
            super::cubecl::KernelStage::Probe(config.kernel),
            (major * 10 + minor) as u32,
            version,
        )?,
        #[cfg(not(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe")))]
        backend => return Err(ProbeError::FeatureDisabled(backend.required_feature())),
    };
    #[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
    let scan = if config.kernel == ProbeKernel::GlobalScan && config.elements > BLOCK_THREADS {
        Some(ScanKernels {
            totals: load_cubecl(
                context,
                config.backend,
                super::cubecl::KernelStage::ScanTotals,
                (major * 10 + minor) as u32,
                version,
            )?,
            offsets: load_cubecl(
                context,
                config.backend,
                super::cubecl::KernelStage::ScanOffsets,
                (major * 10 + minor) as u32,
                version,
            )?,
        })
    } else {
        None
    };
    #[cfg(not(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe")))]
    let scan = None;
    let program = GpuProgram { main, scan };
    Ok((info, program))
}

pub(super) fn run(config: ProbeConfig) -> Result<ProbeReport, ProbeError> {
    let (info, program) = prepare(config)?;
    run_prepared(config, info, program)
}

fn run_prepared(
    config: ProbeConfig,
    info: DeviceInfo,
    program: GpuProgram,
) -> Result<ProbeReport, ProbeError> {
    let context = &info.context;
    let (graph_kernel_nodes, graph_node_updates) = match config.kernel {
        ProbeKernel::Affine | ProbeKernel::SmallSolve | ProbeKernel::FloatAtomicSum => run_typed(
            context,
            &program,
            config,
            samples::floats,
            samples::validate_floats,
            samples::validate_float_values,
        )?,
        _ => run_typed(
            context,
            &program,
            config,
            samples::integers,
            samples::validate_integers,
            samples::validate_integer_values,
        )?,
    };
    drop(program);
    context.check_err().map_err(cuda_error("context-status"))?;
    Ok(ProbeReport {
        backend: config.backend,
        kernel: config.kernel,
        compiler_revision: match config.backend {
            ProbeBackend::NativePtx => None,
            _ => Some(super::cache::CUBECL_REVISION),
        },
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        device: config.device,
        gpu_name: info.gpu_name,
        compute_capability: info.capability,
        driver_api_version: info.driver_api,
        elements: config.elements,
        buffer_bytes: buffer_bytes(config.kernel.output_elements(config.elements)?)?,
        graph_replays: config.replays,
        graph_kernel_nodes,
        graph_node_updates,
    })
}
