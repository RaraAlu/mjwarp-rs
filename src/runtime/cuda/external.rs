//! 私有P1外部资源适配。
//! 只接收本探针的传统CUDA分配。

use super::resources::{CudaFence, ForeignContext, launch_affine_raw};
use super::{LoadedKernel, ProbeGraph, TrackingPause, cuda_error, prepare};
use crate::{
    diagnostics::{ProbeError, ResourceError},
    runtime::{
        ExternalResourceProbeReport, GUARD_ELEMENTS, GUARD_VALUE, ProbeBackend,
        ResourceProbeConfig, ResourceProbeReport,
        completion::{CompletionQueue, CompletionStatus, RetainedResources},
        external::{EXTERNAL_PROBE_WRITABLE, ExternalBufferDescriptor, ImportRegistry},
        input_values,
        lease::{Access, BufferLease, DeviceIdentity, LeasedBuffer, ViewRequest},
        samples::validate_float_values,
    },
};
use cudarc::driver::{CudaContext, CudaEvent, CudaSlice, CudaStream, DevicePtr, result, sys};
use std::sync::{
    Arc, Mutex, OnceLock, Weak,
    atomic::{AtomicUsize, Ordering},
};

struct ImportedOwner {
    context: Arc<CudaContext>,
    pointer: u64,
    ready: Arc<CudaEvent>,
    owner: Arc<dyn Send + Sync>,
}

static IMPORTS: OnceLock<Mutex<ImportRegistry<ImportedOwner>>> = OnceLock::new();

fn identity(context: &CudaContext) -> DeviceIdentity {
    DeviceIdentity {
        device: context.ordinal(),
        context: context.cu_ctx() as usize,
    }
}

// SAFETY契约：owner保留分配与上下文。
// ready必须记录生产操作完成。
// 调用方不得重录事件或并发访问。
// 分配不得释放、重映射或跨进程。
// 描述符只声明布局，驱动不证明dtype。
unsafe fn import(
    context: &Arc<CudaContext>,
    descriptor: ExternalBufferDescriptor,
    owner: Arc<dyn Send + Sync>,
    ready: Arc<CudaEvent>,
) -> Result<LeasedBuffer<ImportedOwner>, ProbeError> {
    descriptor.validate()?;
    if descriptor.device_ordinal as usize != context.ordinal() {
        return Err(ResourceError::DeviceMismatch.into());
    }
    if descriptor.context != context.cu_ctx() as u64 || ready.context().cu_ctx() != context.cu_ctx()
    {
        return Err(ResourceError::ContextMismatch.into());
    }
    if descriptor.layout_version != 1 {
        return Err(ResourceError::StaleLayout {
            expected: 1,
            actual: descriptor.layout_version,
        }
        .into());
    }
    context
        .bind_to_thread()
        .map_err(cuda_error("external-import-context"))?;
    let mut memory_type = 0u32;
    let mut managed = 0u32;
    let mut pool: sys::CUmemoryPool = std::ptr::null_mut();
    let mut actual_context: sys::CUcontext = std::ptr::null_mut();
    let mut device = -1i32;
    let mut buffer_id = 0u64;
    use sys::CUpointer_attribute as Attribute;
    // 每项指针对应驱动要求的输出类型。
    // 单项查询拒绝无效地址。
    let attributes = [
        (
            Attribute::CU_POINTER_ATTRIBUTE_MEMORY_TYPE,
            (&mut memory_type as *mut u32).cast(),
        ),
        (
            Attribute::CU_POINTER_ATTRIBUTE_IS_MANAGED,
            (&mut managed as *mut u32).cast(),
        ),
        (
            Attribute::CU_POINTER_ATTRIBUTE_MEMPOOL_HANDLE,
            (&mut pool as *mut sys::CUmemoryPool).cast(),
        ),
        (
            Attribute::CU_POINTER_ATTRIBUTE_CONTEXT,
            (&mut actual_context as *mut sys::CUcontext).cast(),
        ),
        (
            Attribute::CU_POINTER_ATTRIBUTE_DEVICE_ORDINAL,
            (&mut device as *mut i32).cast(),
        ),
        (
            Attribute::CU_POINTER_ATTRIBUTE_BUFFER_ID,
            (&mut buffer_id as *mut u64).cast(),
        ),
    ];
    for (attribute, output) in attributes {
        // SAFETY: 每项输出具有正确类型与容量。
        // owner保留待查询的分配。
        unsafe { sys::cuPointerGetAttribute(output, attribute, descriptor.allocation) }
            .result()
            .map_err(cuda_error("external-pointer-attribute"))?;
    }
    if memory_type != sys::CUmemorytype::CU_MEMORYTYPE_DEVICE as u32
        || managed != 0
        || !pool.is_null()
    {
        return Err(ResourceError::UnsupportedExternalMemory.into());
    }
    if actual_context != context.cu_ctx() {
        return Err(ResourceError::ContextMismatch.into());
    }
    if device as usize != context.ordinal() {
        return Err(ResourceError::DeviceMismatch.into());
    }
    let mut base = 0u64;
    let mut capacity = 0usize;
    // SAFETY: 驱动写入两个正确类型的输出。
    // owner保持传统分配存活。
    unsafe { sys::cuMemGetAddressRange_v2(&mut base, &mut capacity, descriptor.allocation) }
        .result()
        .map_err(cuda_error("external-allocation-range"))?;
    if base != descriptor.allocation
        || descriptor.allocation_bytes > capacity as u64
        || buffer_id == 0
    {
        return Err(ResourceError::InvalidExternalDescriptor.into());
    }
    let buffer = LeasedBuffer::new(
        ImportedOwner {
            context: context.clone(),
            pointer: base,
            ready,
            owner,
        },
        identity(context),
        descriptor.allocation_bytes as usize,
        4,
        descriptor.flags & EXTERNAL_PROBE_WRITABLE != 0,
    )?;
    IMPORTS
        .get_or_init(|| Mutex::new(ImportRegistry::new()))
        .lock()
        .map_err(|_| ResourceError::Poisoned)?
        .register((descriptor.context, buffer_id), buffer.owner_weak())?;
    Ok(buffer)
}

// 模拟外部客户端，而非引擎分配器。
struct ClientAllocation {
    pointer: u64,
    producer: Option<Arc<CudaStream>>,
    staging: Option<Vec<f32>>,
    releases: Arc<AtomicUsize>,
}

impl ClientAllocation {
    fn upload(
        producer: &Arc<CudaStream>,
        values: Vec<f32>,
        releases: &Arc<AtomicUsize>,
    ) -> Result<(Arc<Self>, Arc<CudaEvent>), ProbeError> {
        producer
            .context()
            .bind_to_thread()
            .map_err(cuda_error("external-allocate-context"))?;
        // SAFETY: 当前上下文有效，字节数非零。
        let pointer = unsafe { result::malloc_sync(values.len() * 4) }
            .map_err(cuda_error("external-allocate"))?;
        let owner = Arc::new(Self {
            pointer,
            producer: Some(producer.clone()),
            staging: Some(values),
            releases: releases.clone(),
        });
        // SAFETY: owner保留目标分配与源切片。
        // Drop等待生产流后才释放它们。
        unsafe {
            result::memcpy_htod_async(
                pointer,
                owner.staging.as_ref().unwrap(),
                producer.cu_stream(),
            )
        }
        .map_err(cuda_error("external-upload"))?;
        let ready = Arc::new(
            producer
                .record_event(None)
                .map_err(cuda_error("external-ready"))?,
        );
        Ok((owner, ready))
    }

    fn descriptor(
        &self,
        context: &CudaContext,
        elements: usize,
        writable: bool,
    ) -> ExternalBufferDescriptor {
        ExternalBufferDescriptor {
            abi_version: 1,
            struct_size: 96,
            device_ordinal: context.ordinal() as u32,
            flags: u32::from(writable),
            context: context.cu_ctx() as u64,
            allocation: self.pointer,
            allocation_bytes: self.staging.as_ref().unwrap().len() as u64 * 4,
            offset_bytes: (GUARD_ELEMENTS * 4) as u64,
            elements: elements as u64,
            stride_bytes: 4,
            layout_version: 1,
            scalar_type: 1,
            rank: 1,
            reserved: [0; 2],
        }
    }
}

impl Drop for ClientAllocation {
    fn drop(&mut self) {
        let producer = self.producer.take().unwrap();
        if let Err(error) = producer.synchronize() {
            producer.context().record_err::<()>(Err(error));
            // 无法证明完成时保留源与上下文。
            std::mem::forget(self.staging.take());
            std::mem::forget(producer);
            return;
        }
        // SAFETY: 队列先等待内核完成。
        // 外部所有者同时等待生产流。
        // 此对象独占传统分配的释放权。
        let freed = unsafe { result::free_sync(self.pointer) };
        if freed.is_ok() {
            self.releases.fetch_add(1, Ordering::SeqCst);
        }
        producer.context().record_err(freed);
    }
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

impl RetainedResources for Task {
    fn quarantine(&self) {
        self.input.quarantine();
        self.output.quarantine();
        if let Some(warmup) = &self.warmup {
            warmup.quarantine();
        }
    }
}

impl Task {
    fn wait_producers(&self, stream: &CudaStream) -> Result<usize, ProbeError> {
        let mut count = 0;
        for lease in [&self.input, &self.output]
            .into_iter()
            .chain(self.warmup.iter())
        {
            stream
                .wait(&lease.owner().ready)
                .map_err(cuda_error("external-producer-wait"))?;
            count += 1;
        }
        Ok(count)
    }
    fn launch_to(
        &self,
        stream: &CudaStream,
        output: &BufferLease<ImportedOwner>,
    ) -> Result<(), ProbeError> {
        let input_pointer = self.input.owner().pointer + self.input.range().start as u64;
        let output_pointer = output.owner().pointer + output.range().start as u64;
        // SAFETY: 导入查询真实范围与上下文。
        // 租约保护偏移、权限与独占写入。
        // 队列保留所有者、事件与模块。
        // 执行流先等待记录的生产事件。
        unsafe {
            launch_affine_raw(
                stream,
                &self.kernel,
                input_pointer,
                output_pointer,
                self.elements,
                self.metadata.as_ref(),
            )
        }
    }
    fn capture(
        &mut self,
        stream: &Arc<CudaStream>,
        backend: ProbeBackend,
    ) -> Result<(), ProbeError> {
        self.launch_to(stream, self.warmup.as_ref().unwrap())?;
        stream
            .synchronize()
            .map_err(cuda_error("external-warmup"))?;
        stream
            .begin_capture(sys::CUstreamCaptureMode::CU_STREAM_CAPTURE_MODE_THREAD_LOCAL)
            .map_err(cuda_error("external-capture-begin"))?;
        let tracking = TrackingPause::after_warmup(stream.context());
        let captured = self.launch_to(stream, &self.output);
        let ended = ProbeGraph::end_capture(stream, backend);
        drop(tracking);
        captured?;
        let graph = ended?;
        if graph.kernels.len() != 1 {
            return Err(ProbeError::InvalidGraph("外部图节点数量不符"));
        }
        self.graph = Some(graph);
        Ok(())
    }
}

fn download(
    stream: &Arc<CudaStream>,
    owner: &ImportedOwner,
    elements: usize,
) -> Result<Vec<f32>, ProbeError> {
    owner
        .context
        .bind_to_thread()
        .map_err(cuda_error("external-download-context"))?;
    let mut values = vec![0.0f32; elements];
    // SAFETY: 调用方已等待内核完成。
    // owner保留整个分配，values提供足够容量。
    // 本函数在所有返回路径等待拷贝。
    let copied =
        unsafe { result::memcpy_dtoh_async(&mut values, owner.pointer, stream.cu_stream()) };
    if let Err(error) = stream.synchronize() {
        // 无法证明拷贝完成时保留全部资源。
        std::mem::forget(values);
        std::mem::forget(owner.owner.clone());
        std::mem::forget(owner.context.clone());
        std::mem::forget(stream.clone());
        return Err(cuda_error("external-download-wait")(error));
    }
    copied.map_err(cuda_error("external-download"))?;
    Ok(values)
}

fn reject<T>(result: Result<T, ProbeError>, expected: ResourceError) -> Result<(), ProbeError> {
    match result {
        Err(ProbeError::Resource(actual)) if actual == expected => Ok(()),
        _ => Err(ProbeError::InvalidArgument("外部资源拒绝结果不符")),
    }
}

// 只用于未提交内核的拒绝样本。
struct RejectedAllocation {
    context: Arc<CudaContext>,
    pointer: u64,
    host: bool,
    foreign: Option<ForeignContext>,
}
impl Drop for RejectedAllocation {
    fn drop(&mut self) {
        // SAFETY: 此对象独占未提交的测试分配。
        // foreign字段保留其所属上下文。
        let freed = unsafe {
            if self.host {
                result::free_host(self.pointer as *mut std::ffi::c_void)
            } else if let Some(foreign) = &self.foreign {
                sys::cuCtxSetCurrent(foreign.raw as sys::CUcontext)
                    .result()
                    .and_then(|()| result::free_sync(self.pointer))
            } else {
                result::free_sync(self.pointer)
            }
        };
        self.context.record_err(freed);
        self.context.record_err(self.context.bind_to_thread());
    }
}

fn check_import_rejections(
    producer: &Arc<CudaStream>,
    descriptor: ExternalBufferDescriptor,
    client: &Arc<ClientAllocation>,
    ready: &Arc<CudaEvent>,
) -> Result<usize, ProbeError> {
    let context = producer.context();
    let mut rejected = 0;
    for (changed, expected) in [
        (
            ExternalBufferDescriptor {
                device_ordinal: descriptor.device_ordinal + 1,
                ..descriptor
            },
            ResourceError::DeviceMismatch,
        ),
        (
            ExternalBufferDescriptor {
                context: descriptor.context + 1,
                ..descriptor
            },
            ResourceError::ContextMismatch,
        ),
        (
            ExternalBufferDescriptor {
                allocation_bytes: descriptor.allocation_bytes + (1 << 30),
                ..descriptor
            },
            ResourceError::InvalidExternalDescriptor,
        ),
        (
            ExternalBufferDescriptor {
                allocation: descriptor.allocation + 4,
                allocation_bytes: descriptor.allocation_bytes - 4,
                elements: 1,
                offset_bytes: 0,
                ..descriptor
            },
            ResourceError::InvalidExternalDescriptor,
        ),
    ] {
        // SAFETY: client与ready保留真实分配及事件。
        // 拒绝样本从不向内核移交指针。
        reject(
            unsafe { import(context, changed, client.clone(), ready.clone()) },
            expected,
        )?;
        rejected += 1;
    }
    for kind in 0..3 {
        context
            .bind_to_thread()
            .map_err(cuda_error("external-reject-context"))?;
        let foreign = if kind == 2 {
            Some(ForeignContext::new(context)?)
        } else {
            None
        };
        // SAFETY: 创建独占的拒绝样本。
        // 外部所有者负责按上下文释放它。
        let pointer = unsafe {
            match kind {
                0 => result::malloc_host(256, 0).map(|pointer| pointer as u64),
                1 => result::malloc_managed(256, sys::CUmemAttach_flags::CU_MEM_ATTACH_GLOBAL),
                _ => sys::cuCtxSetCurrent(foreign.as_ref().unwrap().raw as sys::CUcontext)
                    .result()
                    .and_then(|()| result::malloc_sync(256)),
            }
        }
        .map_err(cuda_error("external-reject-allocate"))?;
        let owner = Arc::new(RejectedAllocation {
            context: context.clone(),
            pointer,
            host: kind == 0,
            foreign,
        });
        context
            .bind_to_thread()
            .map_err(cuda_error("external-reject-restore"))?;
        let changed = ExternalBufferDescriptor {
            allocation: pointer,
            allocation_bytes: 256,
            elements: 1,
            offset_bytes: 0,
            ..descriptor
        };
        let expected = if kind == 2 {
            ResourceError::ContextMismatch
        } else {
            ResourceError::UnsupportedExternalMemory
        };
        // SAFETY: owner保留样本；不提交设备工作。
        reject(
            unsafe { import(context, changed, owner.clone(), ready.clone()) },
            expected,
        )?;
        drop(owner);
        rejected += 1;
    }
    let pooled = Arc::new(
        producer
            .alloc_zeros::<f32>(64)
            .map_err(cuda_error("external-pool-sample"))?,
    );
    producer
        .synchronize()
        .map_err(cuda_error("external-pool-ready"))?;
    let (pointer, record) = pooled.device_ptr(producer);
    let changed = ExternalBufferDescriptor {
        allocation: pointer,
        allocation_bytes: 256,
        elements: 1,
        offset_bytes: 0,
        ..descriptor
    };
    // SAFETY: pooled保留分配；生产已完成。
    reject(
        unsafe { import(context, changed, pooled.clone(), ready.clone()) },
        ResourceError::UnsupportedExternalMemory,
    )?;
    drop(record);
    rejected += 1;
    Ok(rejected)
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
