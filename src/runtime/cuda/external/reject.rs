//! 外部资源拒绝样本与下载辅助。

use std::sync::Arc;

use cudarc::driver::{CudaEvent, CudaStream, DevicePtr, result, sys};

use crate::diagnostics::{ProbeError, ResourceError};
use crate::runtime::external::ExternalBufferDescriptor;

use super::super::cuda_error;
use super::super::resources::ForeignContext;
use super::import::import;
use super::{ClientAllocation, ImportedOwner, RejectedAllocation};

pub(super) fn download(
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

pub(super) fn reject<T>(
    result: Result<T, ProbeError>,
    expected: ResourceError,
) -> Result<(), ProbeError> {
    match result {
        Err(ProbeError::Resource(actual)) if actual == expected => Ok(()),
        _ => Err(ProbeError::InvalidArgument("外部资源拒绝结果不符")),
    }
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

pub(super) fn check_import_rejections(
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
