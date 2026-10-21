//! 类型化探针运行、内核发射与下载。

use std::sync::Arc;

use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, DevicePtr, DevicePtrMut, DeviceRepr,
    LaunchConfig, PushKernelArg, sys,
};

use crate::diagnostics::ProbeError;
use crate::runtime::{
    BLOCK_THREADS, GUARD_ELEMENTS, ProbeBackend, ProbeConfig, ProbeKernel, samples::Samples,
};

use super::cuda_error;
use super::scan::{allocate_scan, check_scan, launch_program};
use super::{GpuProgram, ProbeGraph, TrackingPause};

type ValidateSamples<T> = fn(ProbeKernel, &Samples<T>, &[T]) -> Result<(), ProbeError>;
pub(super) type ValidateValues<T> = fn(&[T], &[T]) -> Result<(), ProbeError>;

pub(super) fn run_typed<T: DeviceRepr + Copy>(
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

pub(super) fn launch<T: DeviceRepr>(
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

pub(super) fn download<T: DeviceRepr>(
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
