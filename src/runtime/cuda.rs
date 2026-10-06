//! 集中封装CUDA适配。

use super::{
    BLOCK_THREADS, GUARD_ELEMENTS, ProbeBackend, ProbeConfig, ProbeKernel, ProbeReport,
    buffer_bytes,
    samples::{self, Samples, ScanSamples},
};
use crate::diagnostics::ProbeError;
use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, DevicePtr, DevicePtrMut, DeviceRepr,
    DriverError, LaunchConfig, PushKernelArg, result, sys,
};
use cudarc::nvrtc::Ptx;
use std::sync::Arc;

pub(super) mod artifacts;
mod resources;
pub(super) use resources::run as run_resources;

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
    kernels: Vec<CapturedKernel>,
    backend: ProbeBackend,
}

struct CapturedKernel {
    node: sys::CUgraphNode,
    params: sys::CUDA_KERNEL_NODE_PARAMS,
    // 保存原始参数值，不借用节点内存。
    arguments: [u64; 3],
}

impl ProbeGraph {
    fn end_capture(stream: &Arc<CudaStream>, backend: ProbeBackend) -> Result<Self, ProbeError> {
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
            kernels: Vec::new(),
            backend,
        };
        graph.read_kernel_nodes()?;
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

    fn read_kernel_nodes(&mut self) -> Result<(), ProbeError> {
        let mut count = 0;
        // SAFETY: 本对象独占完整图。
        // 驱动只写节点数量。
        unsafe { sys::cuGraphGetNodes(self.raw, std::ptr::null_mut(), &mut count) }
            .result()
            .map_err(cuda_error("graph-node-count"))?;
        let mut nodes = vec![std::ptr::null_mut(); count];
        // SAFETY: nodes容量等于查询数量。
        // 本线程不修改图结构。
        unsafe { sys::cuGraphGetNodes(self.raw, nodes.as_mut_ptr(), &mut count) }
            .result()
            .map_err(cuda_error("graph-nodes"))?;
        for node in nodes.into_iter().take(count) {
            let mut kind = sys::CUgraphNodeType::CU_GRAPH_NODE_TYPE_KERNEL;
            // SAFETY: node来自本对象的图。
            unsafe { sys::cuGraphNodeGetType(node, &mut kind) }
                .result()
                .map_err(cuda_error("graph-node-type"))?;
            if kind != sys::CUgraphNodeType::CU_GRAPH_NODE_TYPE_KERNEL {
                return Err(ProbeError::InvalidGraph("探针只捕获内核节点"));
            }
            let mut params = std::mem::MaybeUninit::zeroed();
            // SAFETY: 驱动完整初始化参数结构。
            unsafe { sys::cuGraphKernelNodeGetParams_v2(node, params.as_mut_ptr()) }
                .result()
                .map_err(cuda_error("graph-kernel-params"))?;
            // SAFETY: 上次调用成功初始化结构。
            let params = unsafe { params.assume_init() };
            if params.kernelParams.is_null() || !params.extra.is_null() {
                return Err(ProbeError::InvalidGraph("内核参数布局不符"));
            }
            // SAFETY: 固定ABI包含三项参数。
            // 驱动拥有该参数地址表。
            let pointers = unsafe {
                [
                    *params.kernelParams,
                    *params.kernelParams.add(1),
                    *params.kernelParams.add(2),
                ]
            };
            if pointers.iter().any(|pointer| pointer.is_null()) {
                return Err(ProbeError::InvalidGraph("内核参数地址为空"));
            }
            // SAFETY: 本私有图只捕获固定ABI。
            // 前两项均为64位设备指针。
            // 原生第三项为u32长度。
            // CubeCL第三项为设备指针。
            // 驱动拥有参数副本，图仍存活。
            let arguments = unsafe {
                let input = pointers[0].cast::<u64>().read_unaligned();
                let output = pointers[1].cast::<u64>().read_unaligned();
                let third = pointers[2];
                let third = if self.backend == ProbeBackend::NativePtx {
                    u64::from(third.cast::<u32>().read_unaligned())
                } else {
                    third.cast::<u64>().read_unaligned()
                };
                [input, output, third]
            };
            self.kernels.push(CapturedKernel {
                node,
                params,
                arguments,
            });
        }
        if self.kernels.is_empty() {
            return Err(ProbeError::EmptyGraph);
        }
        Ok(())
    }

    // 调用方只替换同型同容量输出。
    // 调用方保留所有者至执行完成。
    fn update_output(&mut self, original: u64, replacement: u64) -> Result<usize, ProbeError> {
        self.stream
            .context()
            .bind_to_thread()
            .map_err(cuda_error("graph-update-context"))?;
        // 节点更新不与任何重放并发。
        self.stream
            .synchronize()
            .map_err(cuda_error("graph-update-wait"))?;
        let mut updated = 0;
        for kernel in &self.kernels {
            let mut values = kernel.arguments;
            if !replace_output(&mut values, original, replacement) {
                continue;
            }
            let mut native_length = values[2] as u32;
            let third = if self.backend == ProbeBackend::NativePtx {
                (&mut native_length as *mut u32).cast()
            } else {
                (&mut values[2] as *mut u64).cast()
            };
            let mut arguments = [
                (&mut values[0] as *mut u64).cast(),
                (&mut values[1] as *mut u64).cast(),
                third,
            ];
            let mut params = kernel.params;
            params.kernelParams = arguments.as_mut_ptr();
            params.extra = std::ptr::null_mut();
            // SAFETY: 节点属于此图实例。
            // 函数、维度和共享区均不改变。
            // 参数按原始ABI复制有效值。
            // CUDA在调用内复制参数值。
            // 不改写原始节点的参数内存。
            unsafe { sys::cuGraphExecKernelNodeSetParams_v2(self.exec, kernel.node, &params) }
                .result()
                .map_err(cuda_error("graph-node-update"))?;
            updated += 1;
        }
        if updated == 0 {
            return Err(ProbeError::InvalidGraph("没有找到输出参数"));
        }
        Ok(updated)
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

fn replace_output(arguments: &mut [u64; 3], original: u64, replacement: u64) -> bool {
    let mut changed = false;
    // 第三项始终保留元数据或长度。
    for argument in &mut arguments[..2] {
        if *argument == original {
            *argument = replacement;
            changed = true;
        }
    }
    changed
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

struct DeviceInfo {
    context: Arc<CudaContext>,
    gpu_name: String,
    capability: (i32, i32),
    driver_api: i32,
}

fn prepare_device(device: usize) -> Result<DeviceInfo, ProbeError> {
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

type ValidateSamples<T> = fn(ProbeKernel, &Samples<T>, &[T]) -> Result<(), ProbeError>;
type ValidateValues<T> = fn(&[T], &[T]) -> Result<(), ProbeError>;

struct ScanLevel<T> {
    sums: CudaSlice<T>,
    prefix: CudaSlice<T>,
    totals_meta: CudaSlice<u32>,
    scan_meta: CudaSlice<u32>,
    offsets_meta: CudaSlice<u32>,
    elements: usize,
    preceding_elements: usize,
}

fn allocate_scan<T: DeviceRepr + Copy>(
    transfer: &Arc<CudaStream>,
    samples: &Samples<T>,
) -> Result<Vec<ScanLevel<T>>, ProbeError> {
    let mut levels = Vec::new();
    let mut preceding = samples.input.len();
    for sample in &samples.scan_levels {
        let elements = sample.sums.len() - GUARD_ELEMENTS;
        let initial = vec![*sample.sums.last().unwrap(); sample.sums.len()];
        levels.push(ScanLevel {
            sums: transfer
                .clone_htod(&initial)
                .map_err(cuda_error("scan-allocate-sums"))?,
            prefix: transfer
                .clone_htod(&initial)
                .map_err(cuda_error("scan-allocate-prefix"))?,
            totals_meta: transfer
                .clone_htod(&[preceding as u32, elements as u32])
                .map_err(cuda_error("scan-totals-meta"))?,
            scan_meta: transfer
                .clone_htod(&[elements as u32, elements as u32])
                .map_err(cuda_error("scan-prefix-meta"))?,
            offsets_meta: transfer
                .clone_htod(&[elements as u32, preceding as u32])
                .map_err(cuda_error("scan-offsets-meta"))?,
            elements,
            preceding_elements: preceding,
        });
        preceding = elements;
    }
    Ok(levels)
}

fn launch_program<T: DeviceRepr>(
    stream: &CudaStream,
    program: &GpuProgram,
    input: &CudaSlice<T>,
    output: &mut CudaSlice<T>,
    elements: usize,
    metadata: Option<&CudaSlice<u32>>,
    levels: &mut [ScanLevel<T>],
) -> Result<(), ProbeError> {
    launch(
        stream,
        &program.main.function,
        input,
        output,
        elements,
        metadata,
        program.main.shared_memory,
    )?;
    let Some(scan) = &program.scan else {
        return Ok(());
    };
    for depth in 0..levels.len() {
        let (before, remaining) = levels.split_at_mut(depth);
        let level = &mut remaining[0];
        let preceding = before.last().map_or(&*output, |previous| &previous.prefix);
        launch(
            stream,
            &scan.totals.function,
            preceding,
            &mut level.sums,
            level.elements,
            Some(&level.totals_meta),
            scan.totals.shared_memory,
        )?;
        launch(
            stream,
            &program.main.function,
            &level.sums,
            &mut level.prefix,
            level.elements,
            Some(&level.scan_meta),
            program.main.shared_memory,
        )?;
    }
    // 从最小层向下传播块间偏移。
    for depth in (0..levels.len()).rev() {
        let (before, remaining) = levels.split_at_mut(depth);
        let level = &remaining[0];
        let preceding = before
            .last_mut()
            .map_or(&mut *output, |previous| &mut previous.prefix);
        launch(
            stream,
            &scan.offsets.function,
            &level.prefix,
            preceding,
            level.preceding_elements,
            Some(&level.offsets_meta),
            scan.offsets.shared_memory,
        )?;
    }
    Ok(())
}

fn check_scan<T: DeviceRepr>(
    transfer: &Arc<CudaStream>,
    execute: &CudaStream,
    levels: &[ScanLevel<T>],
    samples: &[ScanSamples<T>],
    validate: ValidateValues<T>,
) -> Result<(), ProbeError> {
    for (level, sample) in levels.iter().zip(samples) {
        validate(&sample.sums, &download(transfer, execute, &level.sums)?)?;
        validate(&sample.prefix, &download(transfer, execute, &level.prefix)?)?;
    }
    Ok(())
}

fn run_typed<T: DeviceRepr + Copy>(
    context: &Arc<CudaContext>,
    program: &GpuProgram,
    config: ProbeConfig,
    sample: fn(ProbeKernel, usize, usize) -> Samples<T>,
    validate: ValidateSamples<T>,
    validate_values: ValidateValues<T>,
) -> Result<(usize, usize), ProbeError> {
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
    let mut alternate_output = transfer
        .clone_htod(&initial.initial)
        .map_err(cuda_error("allocate-alternate-output"))?;
    let mut levels = allocate_scan(&transfer, &initial)?;
    // CubeCL固定u32长度；不传动态元数据。
    // 两路均禁用grid_constants。
    let metadata = if config.backend != ProbeBackend::NativePtx {
        Some(
            transfer
                .clone_htod(&[
                    initial.input.len() as u32,
                    (initial.initial.len() - GUARD_ELEMENTS) as u32,
                ])
                .map_err(cuda_error("metadata-upload"))?,
        )
    } else {
        None
    };
    execute
        .join(&transfer)
        .map_err(cuda_error("upload-event"))?;
    launch_program(
        &execute,
        program,
        &device_input,
        &mut device_output,
        config.elements,
        metadata.as_ref(),
        &mut levels,
    )?;
    let output = download(&transfer, &execute, &device_output)?;
    validate(config.kernel, &initial, &output)?;
    check_scan(
        &transfer,
        &execute,
        &levels,
        &initial.scan_levels,
        validate_values,
    )?;
    // 提前保存捕获输出地址。
    let original_output = {
        let (pointer, _record) = device_output.device_ptr(&execute);
        pointer
    };
    execute.synchronize().map_err(cuda_error("warmup-wait"))?;

    // 捕获期间不编译或分配缓冲。
    execute
        .begin_capture(sys::CUstreamCaptureMode::CU_STREAM_CAPTURE_MODE_THREAD_LOCAL)
        .map_err(cuda_error("capture-begin"))?;
    // 避免捕获等待图外的跟踪事件。
    let tracking = TrackingPause::after_warmup(context);
    let captured = launch_program(
        &execute,
        program,
        &device_input,
        &mut device_output,
        config.elements,
        metadata.as_ref(),
        &mut levels,
    );
    // 失败时也结束捕获。
    let ended = ProbeGraph::end_capture(&execute, config.backend);
    drop(tracking);
    captured?;
    let mut graph = ended?;
    let graph_kernel_nodes = graph.kernels.len();
    if graph_kernel_nodes != 1 + levels.len() * 3 {
        return Err(ProbeError::InvalidGraph("捕获节点数量不符"));
    }
    let mut graph_node_updates = 0;

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
            transfer
                .memcpy_htod(&current.initial, &mut alternate_output)
                .map_err(cuda_error("replay-alternate-reset"))?;
            for (level, sample) in levels.iter_mut().zip(&current.scan_levels) {
                let initial = vec![*sample.sums.last().unwrap(); sample.sums.len()];
                transfer
                    .memcpy_htod(&initial, &mut level.sums)
                    .map_err(cuda_error("scan-reset-sums"))?;
                transfer
                    .memcpy_htod(&initial, &mut level.prefix)
                    .map_err(cuda_error("scan-reset-prefix"))?;
            }
            execute
                .join(&transfer)
                .map_err(cuda_error("replay-upload-event"))?;
            let (active, inactive) = if epoch % 2 == 1 {
                (&mut alternate_output, &device_output)
            } else {
                (&mut device_output, &alternate_output)
            };
            // 写事件覆盖真实图提交。
            {
                let (replacement, _record) = active.device_ptr_mut(&execute);
                let updates = graph.update_output(original_output, replacement)?;
                let expected = if levels.is_empty() { 1 } else { 3 };
                if updates != expected {
                    return Err(ProbeError::InvalidGraph("输出节点更新数量不符"));
                }
                graph_node_updates += updates;
                graph.launch()?;
            }
            let output = download(&transfer, &execute, active)?;
            validate(config.kernel, &current, &output)?;
            validate_values(&current.initial, &download(&transfer, &execute, inactive)?)?;
            check_scan(
                &transfer,
                &execute,
                &levels,
                &current.scan_levels,
                validate_values,
            )?;
        }
        Ok(())
    })();
    let completed = context.synchronize().map_err(cuda_error("completion"));
    // graph先释放，缓冲随后释放。
    drop(graph);
    replayed?;
    completed?;
    Ok((graph_kernel_nodes, graph_node_updates))
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
    // 原子值与同型普通值共用存储布局。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_only_matching_buffer_arguments() {
        let mut args = [10, 20, 20];
        assert!(replace_output(&mut args, 20, 30));
        assert_eq!(args, [10, 30, 20]);
        assert!(!replace_output(&mut args, 99, 30));
        let mut downstream = [20, 40, 50];
        assert!(replace_output(&mut downstream, 20, 30));
        assert_eq!(downstream, [30, 40, 50]);
        let mut original = [10, 20, 50];
        assert!(replace_output(&mut original, 20, 20));
        assert_eq!(original, [10, 20, 50]);
    }
}
