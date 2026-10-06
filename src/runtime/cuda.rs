//! 集中封装CUDA适配。

use super::{
    BLOCK_THREADS, ProbeBackend, ProbeConfig, ProbeKernel, ProbeReport, buffer_bytes,
    samples::{self, Samples},
};
use crate::diagnostics::ProbeError;
use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, DeviceRepr, DriverError, LaunchConfig,
    PushKernelArg, result, sys,
};
use cudarc::nvrtc::Ptx;
use std::sync::Arc;

fn cuda_error(stage: &'static str) -> impl FnOnce(DriverError) -> ProbeError {
    move |error| ProbeError::Cuda {
        stage,
        code: error.0 as u32,
    }
}

struct TrackingPause<'a>(&'a CudaContext);

impl<'a> TrackingPause<'a> {
    fn after_warmup(context: &'a CudaContext) -> Self {
        // SAFETY: 调用方已等待全部预热。
        // 私有上下文不接入外部资源。
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
}

impl ProbeGraph {
    fn end_capture(stream: &Arc<CudaStream>) -> Result<Self, ProbeError> {
        stream
            .context()
            .bind_to_thread()
            .map_err(cuda_error("capture-context"))?;
        // SAFETY: 当前线程持有捕获流。
        // 调用结束本次捕获。
        let raw = unsafe { result::stream::end_capture(stream.cu_stream()) }
            .map_err(cuda_error("capture-end"))?;
        if raw.is_null() {
            return Err(ProbeError::EmptyGraph);
        }
        let mut graph = Self {
            raw,
            exec: std::ptr::null_mut(),
            stream: stream.clone(),
        };
        // SAFETY: raw指向完整捕获图。
        // 图不分配显存；标志不改写缓冲。
        // graph在失败时也释放原始图。
        graph.exec = unsafe {
            result::graph::instantiate(
                raw,
                sys::CUgraphInstantiate_flags::CUDA_GRAPH_INSTANTIATE_FLAG_AUTO_FREE_ON_LAUNCH,
            )
        }
        .map_err(cuda_error("graph-instantiate"))?;
        Ok(graph)
    }

    fn launch(&self) -> Result<(), ProbeError> {
        self.stream
            .context()
            .bind_to_thread()
            .map_err(cuda_error("replay-context"))?;
        // SAFETY: 图、流和缓冲持续存活。
        // 调用方通过事件串行读写缓冲。
        unsafe { result::graph::launch(self.exec, self.stream.cu_stream()) }
            .map_err(cuda_error("graph-replay"))
    }
}

impl Drop for ProbeGraph {
    fn drop(&mut self) {
        let context = self.stream.context();
        if let Err(error) = context.bind_to_thread() {
            context.record_err::<()>(Err(error));
            return;
        }
        // panic路径也等待图内核完成。
        context.record_err(self.stream.synchronize());
        if !self.exec.is_null() {
            // SAFETY: 本对象独占图实例。
            // 调用方已等待所有图重放。
            context.record_err(unsafe { result::graph::exec_destroy(self.exec) });
        }
        // SAFETY: 本对象独占原始图。
        // 实例已释放，或实例化失败。
        context.record_err(unsafe { result::graph::destroy(self.raw) });
    }
}

pub(super) fn run(config: ProbeConfig) -> Result<ProbeReport, ProbeError> {
    // SAFETY: 只探测NVIDIA驱动库。
    // 加载器不解引用用户指针。
    if !unsafe { sys::is_culib_present() } {
        return Err(ProbeError::DriverUnavailable);
    }
    let mut version = 0;
    // SAFETY: 驱动写入有效i32地址。
    // version在调用期间保持存活。
    unsafe { sys::cuDriverGetVersion(&mut version) }
        .result()
        .map_err(cuda_error("driver-version"))?;
    if version < 12000 {
        return Err(ProbeError::UnsupportedDriver { version });
    }
    let available = CudaContext::device_count().map_err(cuda_error("device-count"))?;
    if config.device >= available as usize {
        return Err(ProbeError::InvalidDevice {
            requested: config.device,
            available,
        });
    }
    let context = CudaContext::new(config.device).map_err(cuda_error("context"))?;
    let gpu_name = context.name().map_err(cuda_error("device-name"))?;
    let (major, minor) = context
        .compute_capability()
        .map_err(cuda_error("compute-capability"))?;
    if major < 7 {
        return Err(ProbeError::UnsupportedDevice { major, minor });
    }
    let (ptx, entrypoint, shared_memory) = match config.backend {
        ProbeBackend::NativePtx => (
            Ptx::from_src(include_str!("probe.ptx")),
            "affine_probe".to_owned(),
            0,
        ),
        #[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
        backend => {
            let compiled = super::cubecl::compile(
                backend,
                config.kernel,
                (major * 10 + minor) as u32,
                version,
            )?;
            (compiled.ptx, compiled.entrypoint, compiled.shared_memory)
        }
        #[cfg(not(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe")))]
        backend => return Err(ProbeError::FeatureDisabled(backend.required_feature())),
    };
    let module = context.load_module(ptx).map_err(cuda_error("ptx-load"))?;
    let function = module
        .load_function(&entrypoint)
        .map_err(cuda_error("kernel-load"))?;
    match config.kernel {
        ProbeKernel::Affine | ProbeKernel::SmallSolve => run_typed(
            &context,
            &function,
            config,
            shared_memory,
            samples::floats,
            samples::validate_floats,
        )?,
        _ => run_typed(
            &context,
            &function,
            config,
            shared_memory,
            samples::integers,
            samples::validate_integers,
        )?,
    }
    context.check_err().map_err(cuda_error("context-status"))?;
    Ok(ProbeReport {
        backend: config.backend,
        kernel: config.kernel,
        compiler_revision: match config.backend {
            ProbeBackend::NativePtx => None,
            #[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
            _ => Some(super::cubecl::REVISION),
            #[cfg(not(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe")))]
            _ => None,
        },
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        device: config.device,
        gpu_name,
        compute_capability: (major, minor),
        driver_api_version: version,
        elements: config.elements,
        buffer_bytes: buffer_bytes(config.kernel.output_elements(config.elements)?)?,
        graph_replays: config.replays,
    })
}

type ValidateSamples<T> = fn(ProbeKernel, &Samples<T>, &[T]) -> Result<(), ProbeError>;

fn run_typed<T: DeviceRepr + Copy>(
    context: &Arc<CudaContext>,
    function: &CudaFunction,
    config: ProbeConfig,
    shared_memory: u32,
    sample: fn(ProbeKernel, usize, usize) -> Samples<T>,
    validate: ValidateSamples<T>,
) -> Result<(), ProbeError> {
    let transfer = context
        .new_stream()
        .map_err(cuda_error("transfer-stream"))?;
    let execute = context.new_stream().map_err(cuda_error("execute-stream"))?;
    let initial = sample(config.kernel, config.elements, 0);
    let mut device_input = transfer
        .clone_htod(&initial.input)
        .map_err(cuda_error("upload"))?;
    let mut device_output = transfer
        .clone_htod(&initial.initial)
        .map_err(cuda_error("allocate-output"))?;
    // CubeCL固定u32长度；不传动态元数据。
    // 两路均禁用grid_constants。
    let metadata = if config.backend != ProbeBackend::NativePtx {
        Some(
            transfer
                .clone_htod(&[initial.input.len() as u32, initial.initial.len() as u32])
                .map_err(cuda_error("metadata-upload"))?,
        )
    } else {
        None
    };
    execute
        .join(&transfer)
        .map_err(cuda_error("upload-event"))?;
    launch(
        &execute,
        function,
        &device_input,
        &mut device_output,
        config.elements,
        metadata.as_ref(),
        shared_memory,
    )?;
    let output = download(&transfer, &execute, &device_output)?;
    validate(config.kernel, &initial, &output)?;
    execute.synchronize().map_err(cuda_error("warmup-wait"))?;

    // 捕获期间不编译或分配缓冲。
    execute
        .begin_capture(sys::CUstreamCaptureMode::CU_STREAM_CAPTURE_MODE_THREAD_LOCAL)
        .map_err(cuda_error("capture-begin"))?;
    // 避免捕获等待图外的跟踪事件。
    let tracking = TrackingPause::after_warmup(context);
    let captured = launch(
        &execute,
        function,
        &device_input,
        &mut device_output,
        config.elements,
        metadata.as_ref(),
        shared_memory,
    );
    // 失败时也结束捕获。
    let ended = ProbeGraph::end_capture(&execute);
    drop(tracking);
    captured?;
    let graph = ended?;

    // 此作用域保留全部图资源。
    // 错误路径也先等待，再释放。
    let replayed = (|| {
        for epoch in 1..=config.replays {
            let current = sample(config.kernel, config.elements, epoch);
            transfer
                .memcpy_htod(&current.input, &mut device_input)
                .map_err(cuda_error("replay-upload"))?;
            transfer
                .memcpy_htod(&current.initial, &mut device_output)
                .map_err(cuda_error("replay-reset"))?;
            execute
                .join(&transfer)
                .map_err(cuda_error("replay-upload-event"))?;
            graph.launch()?;
            let output = download(&transfer, &execute, &device_output)?;
            validate(config.kernel, &current, &output)?;
        }
        Ok(())
    })();
    let completed = context.synchronize().map_err(cuda_error("completion"));
    // graph先释放，缓冲随后释放。
    drop(graph);
    replayed?;
    completed?;
    Ok(())
}

fn launch<T: DeviceRepr>(
    stream: &CudaStream,
    function: &CudaFunction,
    input: &CudaSlice<T>,
    output: &mut CudaSlice<T>,
    elements: usize,
    metadata: Option<&CudaSlice<u32>>,
    shared_memory: u32,
) -> Result<(), ProbeError> {
    let elements = elements as u32;
    let config = LaunchConfig {
        grid_dim: (elements.div_ceil(BLOCK_THREADS as u32), 1, 1),
        block_dim: (BLOCK_THREADS as u32, 1, 1),
        shared_mem_bytes: shared_memory,
    };
    // SAFETY: 私有调用仅使用两种固定ABI。
    // 原生PTX使用双指针与u32长度。
    // CubeCL使用双指针与元数据指针。
    // 元数据持有两个u32缓冲长度。
    // 输入只读，输出独占且容量充足。
    // 内核守卫限制索引，资源持续存活。
    // 私有调用只传f32或u32样本。
    // 原子u32与普通u32共用存储布局。
    unsafe {
        let mut builder = stream.launch_builder(function);
        builder.arg(input).arg(output);
        if let Some(metadata) = metadata {
            builder.arg(metadata);
        } else {
            builder.arg(&elements);
        }
        builder.launch(config)
    }
    .map_err(cuda_error("kernel-launch"))?;
    Ok(())
}

fn download<T: DeviceRepr>(
    transfer: &Arc<CudaStream>,
    execute: &CudaStream,
    output: &CudaSlice<T>,
) -> Result<Vec<T>, ProbeError> {
    transfer
        .join(execute)
        .map_err(cuda_error("completion-event"))?;
    let downloaded = transfer
        .clone_dtoh(output)
        .map_err(cuda_error("download"))?;
    transfer
        .synchronize()
        .map_err(cuda_error("download-wait"))?;
    Ok(downloaded)
}
