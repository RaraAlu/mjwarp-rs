//! 私有P1外部资源适配。
//! 只接收本探针的传统CUDA分配。
mod client;
mod import;
mod reject;
mod task;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};

use cudarc::driver::{CudaContext, CudaEvent, CudaSlice, CudaStream};

use super::resources::{CudaFence, ForeignContext};
use super::{LoadedKernel, ProbeGraph, cuda_error, prepare};
use crate::diagnostics::{ProbeError, ResourceError};
use crate::runtime::{
    ExternalResourceProbeReport, GUARD_ELEMENTS, GUARD_VALUE, ProbeBackend, ResourceProbeConfig,
    ResourceProbeReport,
    completion::{CompletionQueue, CompletionStatus},
    input_values,
    lease::{Access, BufferLease, ViewRequest},
    samples::validate_float_values,
};

use import::{identity, import};
use reject::{check_import_rejections, download, reject};

struct ImportedOwner {
    context: Arc<CudaContext>,
    pointer: u64,
    ready: Arc<CudaEvent>,
    owner: Arc<dyn Send + Sync>,
}

// 模拟外部客户端，而非引擎分配器。
struct ClientAllocation {
    pointer: u64,
    producer: Option<Arc<CudaStream>>,
    staging: Option<Vec<f32>>,
    releases: Arc<AtomicUsize>,
}

struct Task {
    // 先释放图，再释放图引用的资源。
    graph: Option<ProbeGraph>,
    input: BufferLease<ImportedOwner>,
    output: BufferLease<ImportedOwner>,
    warmup: Option<BufferLease<ImportedOwner>>,
    metadata: Option<CudaSlice<u32>>,
    kernel: LoadedKernel,
    elements: usize,
}

// 只用于未提交内核的拒绝样本。
struct RejectedAllocation {
    context: Arc<CudaContext>,
    pointer: u64,
    host: bool,
    foreign: Option<ForeignContext>,
}

#[derive(Clone, Copy)]
enum Mode {
    Wait,
    Query,
    DropToken,
    ForgetToken,
    DropQueue,
    HostFailure,
    Graph,
}

fn alive<T>(owner: &Weak<T>, expected: bool) -> Result<(), ProbeError> {
    if owner.upgrade().is_some() != expected {
        return Err(ProbeError::InvalidArgument("外部所有者寿命不符"));
    }
    Ok(())
}

pub(in crate::runtime) fn run(
    config: ResourceProbeConfig,
) -> Result<ExternalResourceProbeReport, ProbeError> {
    let (info, program) = prepare(config.kernel_config())?;
    let mut program = Some(program);
    let producer = info
        .context
        .new_stream()
        .map_err(cuda_error("external-producer"))?;
    let execute = info
        .context
        .new_stream()
        .map_err(cuda_error("external-execute"))?;
    let consumer = info
        .context
        .new_stream()
        .map_err(cuda_error("external-consumer"))?;
    let releases = Arc::new(AtomicUsize::new(0));
    let view_bytes = config.elements * 4;
    let capacity = (config.elements + 2 * GUARD_ELEMENTS) * 4;
    let view = ViewRequest {
        target: identity(&info.context),
        offset: GUARD_ELEMENTS * 4,
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
    let mut imported_allocations = 0;
    let mut producer_dependencies = 0;
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
        let (input_client, input_ready) =
            ClientAllocation::upload(&producer, expected_input.clone(), &releases)?;
        let (output_client, output_ready) = ClientAllocation::upload(
            &producer,
            vec![GUARD_VALUE; expected_output.len()],
            &releases,
        )?;
        let descriptor = input_client.descriptor(&info.context, config.elements, false);
        let input_client_weak = Arc::downgrade(&input_client);
        let output_client_weak = Arc::downgrade(&output_client);
        let input_event_weak = Arc::downgrade(&input_ready);
        let output_event_weak = Arc::downgrade(&output_ready);
        if matches!(mode, Mode::Wait) {
            rejected_requests +=
                check_import_rejections(&producer, descriptor, &input_client, &input_ready)?;
        }
        // SAFETY: 客户端所有者保留传统分配。
        // ready记录上传完成；调用方停止外部访问。
        let input = unsafe {
            import(
                &info.context,
                descriptor,
                input_client.clone(),
                input_ready.clone(),
            )
        }?;
        // SAFETY: 输出采用相同的独占移交契约。
        let output = unsafe {
            import(
                &info.context,
                output_client.descriptor(&info.context, config.elements, true),
                output_client.clone(),
                output_ready.clone(),
            )
        }?;
        imported_allocations += 2;
        let input_weak = input.owner_weak();
        let output_weak = output.owner_weak();
        if matches!(mode, Mode::Wait) {
            // SAFETY: 同一所有者仍保持分配存活。
            reject(
                unsafe {
                    import(
                        &info.context,
                        descriptor,
                        input_client.clone(),
                        input_ready.clone(),
                    )
                },
                ResourceError::DuplicateExternalAllocation,
            )?;
            rejected_requests += 1;
        }
        drop(input_client);
        drop(output_client);
        drop(input_ready);
        drop(output_ready);
        let metadata = if config.backend == ProbeBackend::NativePtx {
            None
        } else {
            let data = producer
                .clone_htod(&[config.elements as u32; 2])
                .map_err(cuda_error("external-metadata"))?;
            let ready = producer
                .record_event(None)
                .map_err(cuda_error("external-metadata-ready"))?;
            execute
                .wait(&ready)
                .map_err(cuda_error("external-metadata-wait"))?;
            Some(data)
        };
        let warmup = if matches!(mode, Mode::Graph) {
            let (client, ready) = ClientAllocation::upload(
                &producer,
                vec![GUARD_VALUE; expected_output.len()],
                &releases,
            )?;
            // SAFETY: 所有者与已记录事件移交给导入器。
            let buffer = unsafe {
                import(
                    &info.context,
                    client.descriptor(&info.context, config.elements, true),
                    client,
                    ready,
                )
            }?;
            imported_allocations += 1;
            Some(buffer.lease(view)?)
        } else {
            None
        };
        let task = Task {
            graph: None,
            input: input.lease(ViewRequest {
                access: Access::Read,
                ..view
            })?,
            output: output.lease(view)?,
            warmup,
            metadata,
            kernel: LoadedKernel {
                function: program.as_ref().unwrap().main.function.clone(),
                shared_memory: program.as_ref().unwrap().main.shared_memory,
            },
            elements: config.elements,
        };
        if matches!(mode, Mode::Wait) {
            reject(input.lease(view), ResourceError::ReadOnly)?;
            reject(output.lease(view), ResourceError::Conflict)?;
            reject(output.invalidate_layout(), ResourceError::LayoutBusy)?;
            rejected_requests += 3;
        }
        let mut queue = CompletionQueue::new();
        let token = queue.retain(
            task,
            CudaFence {
                stream: execute.clone(),
                event: None,
            },
        );
        drop(input);
        let mut output = Some(output);
        if matches!(mode, Mode::Wait | Mode::Query | Mode::Graph) {
            drop(output.take());
        }
        if matches!(mode, Mode::Graph) {
            drop(program.take());
        }
        alive(&input_weak, true)?;
        alive(&output_weak, true)?;
        alive(&input_client_weak, true)?;
        alive(&output_client_weak, true)?;
        alive(&input_event_weak, true)?;
        alive(&output_event_weak, true)?;
        if matches!(mode, Mode::Wait) {
            let owner = input_weak.upgrade().unwrap();
            // SAFETY: 租约仍保留同一所有者。
            reject(
                unsafe {
                    import(
                        &info.context,
                        descriptor,
                        owner.owner.clone(),
                        owner.ready.clone(),
                    )
                },
                ResourceError::DuplicateExternalAllocation,
            )?;
            rejected_requests += 1;
        }
        if queue.query(&token)? {
            return Err(ProbeError::InvalidArgument("外部任务提前完成"));
        }
        let submitted = (|| {
            let (task, fence) = queue.retained(&token)?;
            producer_dependencies += task.wait_producers(&execute)?;
            if matches!(mode, Mode::Graph) {
                task.capture(&execute, config.backend)?;
                task.graph.as_ref().unwrap().launch()?;
            } else {
                task.launch_to(&execute, &task.output)?;
            }
            if matches!(mode, Mode::HostFailure) {
                return Err(ProbeError::InvalidArgument("模拟外部提交错误"));
            }
            fence.arm()?;
            consumer
                .wait(fence.event.as_ref().unwrap())
                .map_err(cuda_error("external-downstream-wait"))
        })();
        if let Err(error) = submitted {
            queue.fail(&token, error.clone())?;
            if !matches!(mode, Mode::HostFailure)
                || error != ProbeError::InvalidArgument("模拟外部提交错误")
            {
                return Err(error);
            }
            drop(queue);
            if token.status() != CompletionStatus::Failed(error) {
                return Err(ProbeError::InvalidArgument("外部失败状态丢失"));
            }
            let output = output.take().unwrap();
            reject(output.lease(view), ResourceError::Quarantined)?;
            rejected_requests += 1;
            drop(output);
        } else if matches!(mode, Mode::Wait | Mode::Query | Mode::Graph) {
            if matches!(mode, Mode::Query) {
                execute
                    .synchronize()
                    .map_err(cuda_error("external-query-ready"))?;
                if !queue.query(&token)? || queue.retained(&token).is_ok() {
                    return Err(ProbeError::InvalidArgument("外部完成查询不符"));
                }
                alive(&input_weak, true)?;
                alive(&output_weak, true)?;
            }
            let task = queue.wait(&token)?;
            validate_float_values(
                &expected_input,
                &download(&consumer, task.input.owner(), expected_input.len())?,
            )?;
            validate_float_values(
                &expected_output,
                &download(&consumer, task.output.owner(), expected_output.len())?,
            )?;
            if token.status() != CompletionStatus::Completed {
                return Err(ProbeError::InvalidArgument("外部完成状态不符"));
            }
            drop(task);
        } else {
            let observer = token.clone();
            match mode {
                Mode::DropToken => drop(token),
                Mode::ForgetToken => std::mem::forget(token),
                _ => (),
            }
            drop(queue);
            if observer.status() != CompletionStatus::Completed {
                return Err(ProbeError::InvalidArgument("外部队列未等待完成"));
            }
            alive(&input_weak, false)?;
            let output = output.take().unwrap();
            let read = output.lease(ViewRequest {
                access: Access::Read,
                ..view
            })?;
            validate_float_values(
                &expected_output,
                &download(&consumer, read.owner(), expected_output.len())?,
            )?;
            drop(read);
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
        alive(&input_weak, false)?;
        alive(&output_weak, false)?;
        alive(&input_client_weak, false)?;
        alive(&output_client_weak, false)?;
        alive(&input_event_weak, false)?;
        alive(&output_event_weak, false)?;
    }
    info.context
        .check_err()
        .map_err(cuda_error("external-final-status"))?;
    let released_owners = releases.load(Ordering::SeqCst);
    if released_owners != imported_allocations {
        return Err(ProbeError::InvalidArgument("外部释放计数不符"));
    }
    Ok(ExternalResourceProbeReport {
        resources: ResourceProbeReport {
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
            completed_submissions: 6,
            failed_submissions: 1,
            rejected_requests,
            graph_replays: 1,
        },
        imported_allocations,
        released_owners,
        producer_dependencies,
    })
}
