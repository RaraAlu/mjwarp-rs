//! 传输会话状态与同步执行。

#[cfg(any(feature = "cuda-probe", test))]
use super::host_staging;
use super::{TransferBuffer, TransferElement, checked_bytes};
use crate::diagnostics::{ProbeError, TransferError};
#[cfg(feature = "cuda-probe")]
use cudarc::driver::{CudaStream, DevicePtr, DriverError};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
pub(super) struct SessionStatus {
    pub(super) failed: AtomicBool,
    #[cfg(any(feature = "cuda-probe", test))]
    pub(super) uncertain: AtomicBool,
}

impl SessionStatus {
    pub(super) fn check(&self) -> Result<(), TransferError> {
        if self.failed.load(Ordering::Acquire) {
            return Err(TransferError::Quarantined);
        }
        Ok(())
    }

    #[cfg(any(feature = "cuda-probe", test))]
    pub(super) fn execute<T>(
        &self,
        operation: impl FnOnce() -> Result<T, ProbeError>,
        complete: impl FnOnce() -> Result<(), ProbeError>,
    ) -> Result<T, TransferError> {
        let copied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
            .unwrap_or(Err(ProbeError::BackendPanic));
        // panic后也执行完成检查。
        let completion = std::panic::catch_unwind(std::panic::AssertUnwindSafe(complete))
            .unwrap_or(Err(ProbeError::BackendPanic));
        if completion.is_err() {
            self.uncertain.store(true, Ordering::Release);
        }
        if copied.is_err() || completion.is_err() {
            self.failed.store(true, Ordering::Release);
        }
        match (copied, completion) {
            (Err(error), _) => Err(TransferError::Backend(error)),
            (Ok(value), Err(error)) => {
                // 无法证明完成时保留结果资源。
                std::mem::forget(value);
                Err(TransferError::Backend(error))
            }
            (Ok(value), Ok(())) => Ok(value),
        }
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct HostStaging<'a, T: TransferElement> {
    values: Option<Vec<T>>,
    status: &'a SessionStatus,
}

#[cfg(any(feature = "cuda-probe", test))]
impl<'a, T: TransferElement> HostStaging<'a, T> {
    pub(super) fn new(status: &'a SessionStatus, elements: usize) -> Result<Self, TransferError> {
        Ok(Self {
            values: Some(host_staging(elements)?),
            status,
        })
    }
    pub(super) fn values(&self) -> &[T] {
        self.values.as_ref().unwrap()
    }
    pub(super) fn values_mut(&mut self) -> &mut [T] {
        self.values.as_mut().unwrap()
    }
}

#[cfg(any(feature = "cuda-probe", test))]
impl<T: TransferElement> Drop for HostStaging<'_, T> {
    fn drop(&mut self) {
        if self.status.uncertain.load(Ordering::Acquire) {
            // 不让未知DMA访问已释放的宿主内存。
            std::mem::forget(self.values.take());
        }
    }
}

pub(super) struct SessionInner {
    device: usize,
    pub(super) status: SessionStatus,
    #[cfg(feature = "cuda-probe")]
    pub(in crate::runtime::transfer) stream: Arc<CudaStream>,
    #[cfg(feature = "cuda-probe")]
    operations: std::sync::Mutex<()>,
}

/// 一个独立传输会话。
/// clone共享流与隔离状态。
/// 方法等待复制完成再返回。
#[derive(Clone)]
pub struct TransferSession {
    pub(super) inner: Arc<SessionInner>,
}

impl TransferSession {
    /// 要求cuda-probe与NVIDIA驱动。
    /// 不加载内核编译器。
    pub fn new(device: usize) -> Result<Self, TransferError> {
        #[cfg(feature = "cuda-probe")]
        {
            let stream = std::panic::catch_unwind(|| super::super::cuda::transfer_stream(device))
                .map_err(|_| ProbeError::BackendPanic)??;
            Ok(Self {
                inner: Arc::new(SessionInner {
                    device,
                    status: SessionStatus::default(),
                    stream,
                    operations: std::sync::Mutex::new(()),
                }),
            })
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = device;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
    }

    pub fn device(&self) -> usize {
        self.inner.device
    }

    #[cfg(feature = "cuda-probe")]
    pub(crate) fn same_session(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// 原样上传全部位模式。
    /// 空字段不申请设备缓冲。
    pub fn upload<T: TransferElement>(
        &self,
        source: &[T],
    ) -> Result<TransferBuffer<T>, TransferError> {
        checked_bytes::<T>(source.len())?;
        self.inner.status.check()?;
        #[cfg(feature = "cuda-probe")]
        {
            let (allocation, device_addr) = if source.is_empty() {
                (None, 0)
            } else {
                let (allocation, address) = self.run("transfer-allocate", || {
                    let allocation = self.inner.stream.alloc_zeros::<T>(source.len())?;
                    let (address, read) = allocation.device_ptr(&self.inner.stream);
                    drop(read);
                    Ok((allocation, address))
                })?;
                (Some(allocation), address)
            };
            let mut buffer = TransferBuffer {
                session: self.clone(),
                allocation,
                device_addr,
                len: source.len(),
            };
            buffer.write_range(0, source)?;
            Ok(buffer)
        }
        #[cfg(not(feature = "cuda-probe"))]
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn run<R>(
        &self,
        stage: &'static str,
        operation: impl FnOnce() -> Result<R, DriverError>,
    ) -> Result<R, TransferError> {
        let _guard = self.inner.operations.lock().map_err(|_| {
            self.inner.status.failed.store(true, Ordering::Release);
            self.inner.status.uncertain.store(true, Ordering::Release);
            TransferError::Quarantined
        })?;
        self.inner.status.check()?;
        let error = |stage, error: DriverError| ProbeError::Cuda {
            stage,
            code: error.0 as u32,
        };
        self.inner.status.execute(
            || {
                self.inner
                    .stream
                    .context()
                    .bind_to_thread()
                    .map_err(|err| error("transfer-context", err))?;
                operation().map_err(|err| error(stage, err))
            },
            || {
                // 失败也等待已排入的复制。
                // 保留首个操作错误。
                self.inner
                    .stream
                    .synchronize()
                    .map_err(|err| error("transfer-wait", err))?;
                self.inner
                    .stream
                    .context()
                    .check_err()
                    .map_err(|err| error("transfer-status", err))
            },
        )
    }
}
