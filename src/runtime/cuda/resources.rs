//! 移交真实CUDA所有者。
//! 探针不接收外部裸指针。

use super::{LoadedKernel, ProbeGraph, TrackingPause, cuda_error, prepare};
use crate::{
    diagnostics::{ProbeError, ResourceError},
    runtime::{
        BLOCK_THREADS, GUARD_ELEMENTS, GUARD_VALUE, ProbeBackend, ResourceProbeConfig,
        ResourceProbeReport,
        completion::{CompletionQueue, CompletionStatus, Fence, RetainedResources},
        input_values,
        lease::{Access, BufferLease, DeviceIdentity, LeasedBuffer, ViewRequest},
        samples::validate_float_values,
    },
};
use cudarc::driver::{
    CudaContext, CudaEvent, CudaSlice, CudaStream, DevicePtr, LaunchConfig, PushKernelArg, result,
    sys,
};
use std::sync::Arc;

struct CudaFence {
    stream: Arc<CudaStream>,
    event: Option<CudaEvent>,
}

impl CudaFence {
    fn arm(&mut self) -> Result<(), ProbeError> {
        self.event = Some(
            self.stream
                .record_event(None)
                .map_err(cuda_error("lease-record"))?,
        );
        Ok(())
    }
}

fn event_result(result: Result<(), cudarc::driver::DriverError>) -> Result<bool, ProbeError> {
    match result {
        Ok(()) => Ok(true),
        Err(error) if error.0 == sys::cudaError_enum::CUDA_ERROR_NOT_READY => Ok(false),
        Err(error) => Err(cuda_error("lease-query")(error)),
    }
}

impl Fence for CudaFence {
    fn query(&self) -> Result<bool, ProbeError> {
        let Some(event) = &self.event else {
            // CUDA将未记录事件视为完成。
            // 本队列将未提交任务视为待定。
            return Ok(false);
        };
        self.stream
            .context()
            .bind_to_thread()
            .map_err(cuda_error("lease-query-context"))?;
        // SAFETY: 栅栏独占此事件。
        // 队列保留事件所属上下文。
        event_result(unsafe { result::event::query(event.cu_event()) })
    }
    fn wait(&self) -> Result<(), ProbeError> {
        if let Some(event) = &self.event {
            event.synchronize().map_err(cuda_error("lease-wait"))?;
        } else {
            // 部分提交失败也等待已排入工作。
            self.stream
                .synchronize()
                .map_err(cuda_error("lease-unarmed-wait"))?;
        }
        self.stream
            .context()
            .check_err()
            .map_err(cuda_error("lease-context-status"))
    }
}

struct AffineResources {
    // 按字段顺序先销毁图。
    graph: Option<ProbeGraph>,
    input: BufferLease<CudaSlice<f32>>,
    output: BufferLease<CudaSlice<f32>>,
    warmup_output: Option<BufferLease<CudaSlice<f32>>>,
    metadata: Option<CudaSlice<u32>>,
    kernel: LoadedKernel,
    elements: usize,
}

impl RetainedResources for AffineResources {
    fn quarantine(&self) {
        self.input.quarantine();
        self.output.quarantine();
        if let Some(output) = &self.warmup_output {
            output.quarantine();
        }
    }
}

impl AffineResources {
    fn launch(&self, stream: &CudaStream) -> Result<(), ProbeError> {
        self.launch_to(stream, &self.output)
    }

    fn launch_to(
        &self,
        stream: &CudaStream,
        output: &BufferLease<CudaSlice<f32>>,
    ) -> Result<(), ProbeError> {
        let (input, _input_record) = self.input.owner().device_ptr(stream);
        let (pointer, _output_record) = output.owner().device_ptr(stream);
        let input = input
            .checked_add(self.input.range().start as u64)
            .ok_or(ProbeError::InvalidArgument("输入地址溢出"))?;
        let output = pointer
            .checked_add(output.range().start as u64)
            .ok_or(ProbeError::InvalidArgument("输出地址溢出"))?;
        let elements = self.elements as u32;
        let config = LaunchConfig {
            grid_dim: (elements.div_ceil(BLOCK_THREADS as u32), 1, 1),
            block_dim: (BLOCK_THREADS as u32, 1, 1),
            shared_mem_bytes: self.kernel.shared_memory,
        };
        // SAFETY: 私有ABI包含三项参数。
        // 租约校验真实容量与字节偏移。
        // 输入只读；输出租约独占范围。
        // 队列先保留所有者再调用本函数。
        // 完成栅栏保护实际输出写入。
        // 元数据记录视图长度而非容量。
        // 内核守卫限制索引至视图范围。
        unsafe {
            let mut builder = stream.launch_builder(&self.kernel.function);
            builder.arg(&input).arg(&output);
            if let Some(metadata) = &self.metadata {
                builder.arg(metadata);
            } else {
                builder.arg(&elements);
            }
            builder.launch(config)
        }
        .map_err(cuda_error("lease-launch"))?;
        Ok(())
    }

    fn capture(
        &mut self,
        stream: &Arc<CudaStream>,
        backend: ProbeBackend,
    ) -> Result<(), ProbeError> {
        // 预热写独立缓冲，避免掩盖重放失败。
        self.launch_to(stream, self.warmup_output.as_ref().unwrap())?;
        stream.synchronize().map_err(cuda_error("lease-warmup"))?;
        // 所有私有上传流也已完成。
        stream
            .context()
            .synchronize()
            .map_err(cuda_error("lease-capture-ready"))?;
        stream
            .begin_capture(sys::CUstreamCaptureMode::CU_STREAM_CAPTURE_MODE_THREAD_LOCAL)
            .map_err(cuda_error("lease-capture-begin"))?;
        let tracking = TrackingPause::after_warmup(stream.context());
        let captured = self.launch(stream);
        let ended = ProbeGraph::end_capture(stream, backend);
        drop(tracking);
        captured?;
        let graph = ended?;
        if graph.kernels.len() != 1 {
            return Err(ProbeError::InvalidGraph("租约图节点数量不符"));
        }
        self.graph = Some(graph);
        Ok(())
    }
}

fn identity(context: &CudaContext) -> DeviceIdentity {
    DeviceIdentity {
        device: context.ordinal(),
        context: context.cu_ctx() as usize,
    }
}

fn wrap(owner: CudaSlice<f32>, writable: bool) -> Result<LeasedBuffer<CudaSlice<f32>>, ProbeError> {
    let target = identity(owner.context());
    let capacity = owner.num_bytes();
    LeasedBuffer::new(owner, target, capacity, size_of::<f32>(), writable)
}

// 仅用于验证同设备异上下文。
// 不将此上下文接入内核调用。
struct ForeignContext {
    raw: sys::CUcontext,
    primary: Arc<CudaContext>,
}
impl ForeignContext {
    fn new(primary: &Arc<CudaContext>) -> Result<Self, ProbeError> {
        let mut raw = std::ptr::null_mut();
        // SAFETY: 驱动写入独立上下文句柄。
        // primary持有有效CUDA设备。
        unsafe { sys::cuCtxCreate_v2(&mut raw, 0, primary.cu_device()) }
            .result()
            .map_err(cuda_error("lease-foreign-context"))?;
        let context = Self {
            raw,
            primary: primary.clone(),
        };
        primary
            .bind_to_thread()
            .map_err(cuda_error("lease-restore-context"))?;
        Ok(context)
    }
}
impl Drop for ForeignContext {
    fn drop(&mut self) {
        // SAFETY: 此对象独占异域上下文。
        // 本探针从未向此上下文提交任务。
        self.primary
            .record_err(unsafe { sys::cuCtxDestroy_v2(self.raw) }.result());
        self.primary.record_err(self.primary.bind_to_thread());
    }
}

fn reject<T>(result: Result<T, ProbeError>, expected: ResourceError) -> Result<(), ProbeError> {
    match result {
        Err(ProbeError::Resource(actual)) if actual == expected => Ok(()),
        _ => Err(ProbeError::InvalidArgument("资源拒绝结果不符")),
    }
}

fn check_rejections(
    input: &LeasedBuffer<CudaSlice<f32>>,
    output: &LeasedBuffer<CudaSlice<f32>>,
    view: ViewRequest,
    capacity: usize,
    foreign: &ForeignContext,
) -> Result<usize, ProbeError> {
    let requests = [
        (
            ViewRequest {
                target: DeviceIdentity {
                    device: view.target.device ^ 1,
                    ..view.target
                },
                ..view
            },
            ResourceError::DeviceMismatch,
        ),
        (
            ViewRequest {
                target: DeviceIdentity {
                    context: foreign.raw as usize,
                    ..view.target
                },
                ..view
            },
            ResourceError::ContextMismatch,
        ),
        (
            ViewRequest { bytes: 0, ..view },
            ResourceError::InvalidRange {
                offset: view.offset,
                bytes: 0,
                capacity,
            },
        ),
        (
            ViewRequest {
                offset: usize::MAX,
                bytes: 4,
                ..view
            },
            ResourceError::InvalidRange {
                offset: usize::MAX,
                bytes: 4,
                capacity,
            },
        ),
        (
            ViewRequest {
                offset: capacity,
                bytes: 4,
                ..view
            },
            ResourceError::InvalidRange {
                offset: capacity,
                bytes: 4,
                capacity,
            },
        ),
        (
            ViewRequest {
                offset: view.offset + 1,
                ..view
            },
            ResourceError::Misaligned,
        ),
        (ViewRequest { bytes: 3, ..view }, ResourceError::Misaligned),
        (
            ViewRequest { layout: 0, ..view },
            ResourceError::StaleLayout {
                expected: 1,
                actual: 0,
            },
        ),
        (view, ResourceError::Conflict),
        (
            ViewRequest {
                access: Access::Read,
                ..view
            },
            ResourceError::Conflict,
        ),
    ];
    let count = requests.len() + 2;
    for (request, expected) in requests {
        reject(output.lease(request), expected)?;
    }
    reject(input.lease(view), ResourceError::ReadOnly)?;
    reject(output.invalidate_layout(), ResourceError::LayoutBusy)?;
    Ok(count)
}

#[derive(Clone, Copy)]
enum Mode {
    Wait,
    Query,
    DropToken,
    ForgetToken,
    DropQueue,
    Graph,
    HostFailure,
}

fn check_alive(weak: &std::sync::Weak<CudaSlice<f32>>, alive: bool) -> Result<(), ProbeError> {
    if weak.upgrade().is_some() == alive {
        Ok(())
    } else {
        Err(ProbeError::InvalidArgument("资源所有者寿命不符"))
    }
}

fn validate(
    consumer: &Arc<CudaStream>,
    input: &CudaSlice<f32>,
    output: &CudaSlice<f32>,
    expected_input: &[f32],
    expected_output: &[f32],
) -> Result<(), ProbeError> {
    let input = consumer
        .clone_dtoh(input)
        .map_err(cuda_error("lease-download-input"))?;
    let output = consumer
        .clone_dtoh(output)
        .map_err(cuda_error("lease-download-output"))?;
    consumer
        .synchronize()
        .map_err(cuda_error("lease-download-wait"))?;
    validate_float_values(expected_input, &input)?;
    validate_float_values(expected_output, &output)
}

pub(in crate::runtime) fn run(
    config: ResourceProbeConfig,
) -> Result<ResourceProbeReport, ProbeError> {
    let (info, program) = prepare(config.kernel_config())?;
    let mut program = Some(program);
    let producer = info
        .context
        .new_stream()
        .map_err(cuda_error("lease-producer"))?;
    let execute = info
        .context
        .new_stream()
        .map_err(cuda_error("lease-execute"))?;
    let consumer = info
        .context
        .new_stream()
        .map_err(cuda_error("lease-consumer"))?;
    let foreign = ForeignContext::new(&info.context)?;
    let view_bytes = config.elements * size_of::<f32>();
    let capacity = (config.elements + 2 * GUARD_ELEMENTS) * size_of::<f32>();
    let view = ViewRequest {
        target: identity(&info.context),
        offset: GUARD_ELEMENTS * size_of::<f32>(),
        bytes: view_bytes,
        layout: 1,
        access: Access::Write,
    };
    let modes = [
        Mode::Wait,
        Mode::Query,
        Mode::DropToken,
        Mode::ForgetToken,
        Mode::DropQueue,
        Mode::HostFailure,
        Mode::Graph,
    ];
    let mut rejected_requests = 0;
    for (epoch, mode) in modes.into_iter().enumerate() {
        let values = input_values(config.elements, epoch);
        let expected_input: Vec<_> = [GUARD_VALUE; GUARD_ELEMENTS]
            .into_iter()
            .chain(values.iter().copied())
            .chain([GUARD_VALUE; GUARD_ELEMENTS])
            .collect();
        let expected_output: Vec<_> = [GUARD_VALUE; GUARD_ELEMENTS]
            .into_iter()
            .chain(values.iter().map(|x| x * 2.0 + 1.0))
            .chain([GUARD_VALUE; GUARD_ELEMENTS])
            .collect();
        let input = wrap(
            producer
                .clone_htod(&expected_input)
                .map_err(cuda_error("lease-upload"))?,
            false,
        )?;
        let output = wrap(
            producer
                .clone_htod(&vec![GUARD_VALUE; expected_output.len()])
                .map_err(cuda_error("lease-output"))?,
            true,
        )?;
        let input_weak = input.owner_weak();
        let output_weak = output.owner_weak();
        let metadata = if config.backend == ProbeBackend::NativePtx {
            None
        } else {
            Some(
                producer
                    .clone_htod(&[config.elements as u32; 2])
                    .map_err(cuda_error("lease-metadata"))?,
            )
        };
        let warmup_output = if matches!(mode, Mode::Graph) {
            let owner = producer
                .clone_htod(&vec![GUARD_VALUE; expected_output.len()])
                .map_err(cuda_error("lease-warmup-output"))?;
            Some(wrap(owner, true)?.lease(view)?)
        } else {
            None
        };
        let ready = producer
            .record_event(None)
            .map_err(cuda_error("lease-producer-ready"))?;
        execute
            .wait(&ready)
            .map_err(cuda_error("lease-consumer-ready"))?;
        let resources = AffineResources {
            graph: None,
            input: input.lease(ViewRequest {
                access: Access::Read,
                ..view
            })?,
            output: output.lease(view)?,
            warmup_output,
            metadata,
            kernel: LoadedKernel {
                function: program.as_ref().unwrap().main.function.clone(),
                shared_memory: program.as_ref().unwrap().main.shared_memory,
            },
            elements: config.elements,
        };
        if matches!(mode, Mode::Wait) {
            rejected_requests += check_rejections(&input, &output, view, capacity, &foreign)?;
        }
        let mut queue = CompletionQueue::new();
        // 队列先接管租约、内核与元数据。
        let token = queue.retain(
            resources,
            CudaFence {
                stream: execute.clone(),
                event: None,
            },
        );
        // 调用方在内核提交前释放句柄。
        drop(input);
        let mut output = Some(output);
        if matches!(mode, Mode::Wait | Mode::Query | Mode::Graph) {
            drop(output.take());
        }
        if matches!(mode, Mode::Graph) {
            // 最后一个任务独立保留内核模块。
            drop(program.take());
        }
        check_alive(&input_weak, true)?;
        check_alive(&output_weak, true)?;
        if queue.query(&token)? {
            return Err(ProbeError::InvalidArgument("未提交任务提前完成"));
        }
        let submitted = (|| {
            let (resources, fence) = queue.retained(&token)?;
            if matches!(mode, Mode::Graph) {
                resources.capture(&execute, config.backend)?;
                resources.graph.as_ref().unwrap().launch()?;
            } else {
                resources.launch(&execute)?;
            }
            if matches!(mode, Mode::HostFailure) {
                // 模拟提交后、记录事件前的宿主错误。
                return Err(ProbeError::InvalidArgument("模拟宿主提交错误"));
            }
            fence.arm()?;
            // 下游流只等待显式完成事件。
            consumer
                .wait(fence.event.as_ref().unwrap())
                .map_err(cuda_error("lease-downstream-wait"))
        })();
        if let Err(error) = submitted {
            queue.fail(&token, error.clone())?;
            if !matches!(mode, Mode::HostFailure)
                || error != ProbeError::InvalidArgument("模拟宿主提交错误")
            {
                return Err(error);
            }
            drop(queue);
            if token.status() != CompletionStatus::Failed(error) {
                return Err(ProbeError::InvalidArgument("提交失败状态丢失"));
            }
            let output = output.take().unwrap();
            reject(output.lease(view), ResourceError::Quarantined)?;
            rejected_requests += 1;
            drop(output);
            check_alive(&input_weak, false)?;
            check_alive(&output_weak, false)?;
            continue;
        }
        if matches!(mode, Mode::Wait | Mode::Query | Mode::Graph) {
            if matches!(mode, Mode::Query) {
                execute
                    .synchronize()
                    .map_err(cuda_error("lease-query-ready"))?;
                if !queue.query(&token)? {
                    return Err(ProbeError::InvalidArgument("已完成事件仍待定"));
                }
                if queue.retained(&token).is_ok() {
                    return Err(ProbeError::InvalidArgument("完成任务再次提交"));
                }
                check_alive(&output_weak, true)?;
            }
            let resources = queue.wait(&token)?;
            validate(
                &consumer,
                resources.input.owner(),
                resources.output.owner(),
                &expected_input,
                &expected_output,
            )?;
            if token.status() != CompletionStatus::Completed {
                return Err(ProbeError::InvalidArgument("完成状态不符"));
            }
            drop(resources);
        } else {
            let output = output.take().unwrap();
            let observer = token.clone();
            match mode {
                Mode::DropToken => drop(token),
                Mode::ForgetToken => std::mem::forget(token),
                _ => (),
            }
            drop(queue);
            if observer.status() != CompletionStatus::Completed {
                return Err(ProbeError::InvalidArgument("释放队列未等待完成"));
            }
            check_alive(&input_weak, false)?;
            let read = output.lease(ViewRequest {
                access: Access::Read,
                ..view
            })?;
            let downloaded = consumer
                .clone_dtoh(read.owner())
                .map_err(cuda_error("lease-drop-download"))?;
            consumer
                .synchronize()
                .map_err(cuda_error("lease-drop-download-wait"))?;
            validate_float_values(&expected_output, &downloaded)?;
            drop(read);
            // 完成后才允许切换布局版本。
            let layout = output.invalidate_layout()?;
            reject(
                output.lease(view),
                ResourceError::StaleLayout {
                    expected: layout,
                    actual: view.layout,
                },
            )?;
            rejected_requests += 1;
            drop(output);
        }
        check_alive(&input_weak, false)?;
        check_alive(&output_weak, false)?;
    }
    drop(foreign);
    drop(program);
    info.context
        .check_err()
        .map_err(cuda_error("lease-final-status"))?;
    Ok(ResourceProbeReport {
        backend: config.backend,
        compiler_revision: match config.backend {
            ProbeBackend::NativePtx => None,
            #[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
            _ => Some(crate::runtime::cubecl::REVISION),
            #[cfg(not(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe")))]
            _ => None,
        },
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        device: config.device,
        gpu_name: info.gpu_name,
        compute_capability: info.capability,
        driver_api_version: info.driver_api,
        elements: config.elements,
        view_bytes,
        output_owner_bytes: capacity,
        completed_submissions: modes.len() - 1,
        failed_submissions: 1,
        rejected_requests,
        graph_replays: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cudarc::driver::DriverError;
    #[test]
    fn only_not_ready_means_pending() {
        assert!(event_result(Ok(())).unwrap());
        assert!(
            !event_result(Err(DriverError(sys::cudaError_enum::CUDA_ERROR_NOT_READY))).unwrap()
        );
        assert_eq!(
            event_result(Err(DriverError(
                sys::cudaError_enum::CUDA_ERROR_ILLEGAL_ADDRESS
            )))
            .unwrap_err(),
            ProbeError::Cuda {
                stage: "lease-query",
                code: 700
            }
        );
    }
}
