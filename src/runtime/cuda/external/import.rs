//! 外部传统分配导入与指针校验。

use std::sync::{Arc, Mutex, OnceLock};

use cudarc::driver::{CudaContext, CudaEvent, sys};

use crate::diagnostics::{ProbeError, ResourceError};
use crate::runtime::external::{EXTERNAL_PROBE_WRITABLE, ExternalBufferDescriptor, ImportRegistry};
use crate::runtime::lease::{DeviceIdentity, LeasedBuffer};

use super::super::cuda_error;
use super::ImportedOwner;

static IMPORTS: OnceLock<Mutex<ImportRegistry<ImportedOwner>>> = OnceLock::new();

pub(super) fn identity(context: &CudaContext) -> DeviceIdentity {
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
pub(super) unsafe fn import(
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
