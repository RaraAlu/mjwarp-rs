//! 模拟外部客户端分配。

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use cudarc::driver::{CudaContext, CudaEvent, CudaStream, result};

use crate::diagnostics::ProbeError;
use crate::runtime::GUARD_ELEMENTS;
use crate::runtime::external::ExternalBufferDescriptor;

use super::super::cuda_error;
use super::ClientAllocation;

impl ClientAllocation {
    pub(super) fn upload(
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

    pub(super) fn descriptor(
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
