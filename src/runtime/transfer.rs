//! 类型化同步传输辅助。
//! 不导出设备地址或上下文。
//! 不冻结生产运行时接口。

use crate::diagnostics::{InputError, ProbeError, TransferError};
#[cfg(feature = "cuda-probe")]
use cudarc::driver::{CudaSlice, CudaStream, DriverError};
use std::{
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

mod element {
    #[cfg(feature = "cuda-probe")]
    pub trait Sealed:
        cudarc::driver::DeviceRepr + cudarc::driver::ValidAsZeroBits + Copy + Default
    {
    }
    #[cfg(not(feature = "cuda-probe"))]
    pub trait Sealed: Copy + Default {}

    impl Sealed for f32 {}
    impl Sealed for i32 {}
    impl Sealed for u32 {}
}

/// 仅支持f32、i32与u32。
/// 调用方不能增加设备类型。
///
/// ```compile_fail
/// use mjwarp_rs::runtime::TransferSession;
/// let session = TransferSession::new(0).unwrap();
/// session.upload(&[true]);
/// ```
pub trait TransferElement: element::Sealed {}
impl TransferElement for f32 {}
impl TransferElement for i32 {}
impl TransferElement for u32 {}

#[derive(Default)]
struct SessionStatus {
    failed: AtomicBool,
    #[cfg(any(feature = "cuda-probe", test))]
    uncertain: AtomicBool,
}

impl SessionStatus {
    fn check(&self) -> Result<(), TransferError> {
        if self.failed.load(Ordering::Acquire) {
            return Err(TransferError::Quarantined);
        }
        Ok(())
    }

    #[cfg(any(feature = "cuda-probe", test))]
    fn execute<T>(
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
struct HostStaging<'a, T: TransferElement> {
    values: Option<Vec<T>>,
    status: &'a SessionStatus,
}

#[cfg(any(feature = "cuda-probe", test))]
impl<'a, T: TransferElement> HostStaging<'a, T> {
    fn new(status: &'a SessionStatus, elements: usize) -> Result<Self, TransferError> {
        Ok(Self {
            values: Some(host_staging(elements)?),
            status,
        })
    }
    fn values(&self) -> &[T] {
        self.values.as_ref().unwrap()
    }
    fn values_mut(&mut self) -> &mut [T] {
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

struct SessionInner {
    device: usize,
    status: SessionStatus,
    #[cfg(feature = "cuda-probe")]
    stream: Arc<CudaStream>,
    #[cfg(feature = "cuda-probe")]
    operations: std::sync::Mutex<()>,
}

/// 一个独立传输会话。
/// clone共享流与隔离状态。
/// 方法等待复制完成再返回。
#[derive(Clone)]
pub struct TransferSession {
    inner: Arc<SessionInner>,
}

impl TransferSession {
    /// 要求cuda-probe与NVIDIA驱动。
    /// 不加载内核编译器。
    pub fn new(device: usize) -> Result<Self, TransferError> {
        #[cfg(feature = "cuda-probe")]
        {
            let stream = std::panic::catch_unwind(|| super::cuda::transfer_stream(device))
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
            let allocation = if source.is_empty() {
                None
            } else {
                Some(self.run("transfer-allocate", || {
                    self.inner.stream.alloc_zeros::<T>(source.len())
                })?)
            };
            let mut buffer = TransferBuffer {
                session: self.clone(),
                allocation,
                len: source.len(),
            };
            buffer.write_range(0, source)?;
            Ok(buffer)
        }
        #[cfg(not(feature = "cuda-probe"))]
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }

    #[cfg(feature = "cuda-probe")]
    fn run<R>(
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

/// 缓冲独占设备分配。
/// 缓冲保留流与实际上下文。
/// 不提供clone或外部导入。
pub struct TransferBuffer<T: TransferElement> {
    session: TransferSession,
    len: usize,
    #[cfg(feature = "cuda-probe")]
    allocation: Option<CudaSlice<T>>,
    #[cfg(not(feature = "cuda-probe"))]
    marker: std::marker::PhantomData<T>,
}

impl<T: TransferElement> Drop for TransferBuffer<T> {
    fn drop(&mut self) {
        #[cfg(feature = "cuda-probe")]
        if self.session.inner.status.uncertain.load(Ordering::Acquire) {
            // 分配同时保留流、事件与上下文。
            // 未知完成状态可能导致显存泄漏。
            std::mem::forget(self.allocation.take());
        }
    }
}

impl<T: TransferElement> TransferBuffer<T> {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn byte_len(&self) -> usize {
        self.len * size_of::<T>()
    }

    /// 先暂存，再改写调用方切片。
    /// 失败不会留下部分宿主写入。
    pub fn read_range_into(&self, offset: usize, target: &mut [T]) -> Result<(), TransferError> {
        let range = checked_range(self.len, offset, target.len())?;
        self.session.inner.status.check()?;
        #[cfg(feature = "cuda-probe")]
        {
            if range.is_empty() {
                return Ok(());
            }
            let mut staging = HostStaging::<T>::new(&self.session.inner.status, target.len())?;
            let view = self
                .allocation
                .as_ref()
                .expect("nonempty allocation")
                .slice(range);
            self.session.run("transfer-read", || {
                self.session
                    .inner
                    .stream
                    .memcpy_dtoh(&view, staging.values_mut())
            })?;
            target.copy_from_slice(staging.values());
            Ok(())
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = range;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
    }

    /// 输入错误不会写入设备。
    /// 后端失败隔离整个会话。
    pub fn write_range(&mut self, offset: usize, source: &[T]) -> Result<(), TransferError> {
        let range = checked_range(self.len, offset, source.len())?;
        self.session.inner.status.check()?;
        #[cfg(feature = "cuda-probe")]
        {
            if range.is_empty() {
                return Ok(());
            }
            let mut staging = HostStaging::<T>::new(&self.session.inner.status, source.len())?;
            staging.values_mut().copy_from_slice(source);
            let mut view = self
                .allocation
                .as_mut()
                .expect("nonempty allocation")
                .slice_mut(range);
            self.session.run("transfer-write", || {
                self.session
                    .inner
                    .stream
                    .memcpy_htod(staging.values(), &mut view)
            })
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = range;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
    }

    /// 仅接受同会话的独立缓冲。
    /// 会话clone保持兼容。
    pub fn copy_range_from(
        &mut self,
        offset: usize,
        source: &Self,
        source_offset: usize,
        elements: usize,
    ) -> Result<(), TransferError> {
        let target_range = checked_range(self.len, offset, elements)?;
        let source_range = checked_range(source.len, source_offset, elements)?;
        if !Arc::ptr_eq(&self.session.inner, &source.session.inner) {
            return Err(TransferError::SessionMismatch);
        }
        self.session.inner.status.check()?;
        #[cfg(feature = "cuda-probe")]
        {
            if elements == 0 {
                return Ok(());
            }
            let input = source
                .allocation
                .as_ref()
                .expect("nonempty allocation")
                .slice(source_range);
            let mut output = self
                .allocation
                .as_mut()
                .expect("nonempty allocation")
                .slice_mut(target_range);
            self.session.run("transfer-copy", || {
                self.session.inner.stream.memcpy_dtod(&input, &mut output)
            })
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (target_range, source_range);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
    }

    /// 同缓冲只允许不相交复制。
    /// 相同范围保持原值。
    pub fn copy_range_within(
        &mut self,
        offset: usize,
        source_offset: usize,
        elements: usize,
    ) -> Result<(), TransferError> {
        let target = checked_range(self.len, offset, elements)?;
        let source = checked_range(self.len, source_offset, elements)?;
        check_disjoint(&target, &source)?;
        self.session.inner.status.check()?;
        #[cfg(feature = "cuda-probe")]
        {
            if elements == 0 || target == source {
                return Ok(());
            }
            let allocation = self.allocation.as_mut().expect("nonempty allocation");
            if offset < source_offset {
                let (mut head, tail) = allocation.split_at_mut(source_offset);
                let input = tail.slice(0..elements);
                let mut output = head.slice_mut(target);
                self.session.run("transfer-copy-within", || {
                    self.session.inner.stream.memcpy_dtod(&input, &mut output)
                })
            } else {
                let (head, mut tail) = allocation.split_at_mut(offset);
                let input = head.slice(source);
                let mut output = tail.slice_mut(0..elements);
                self.session.run("transfer-copy-within", || {
                    self.session.inner.stream.memcpy_dtod(&input, &mut output)
                })
            }
        }
        #[cfg(not(feature = "cuda-probe"))]
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
}

fn checked_bytes<T: TransferElement>(elements: usize) -> Result<usize, InputError> {
    elements
        .checked_mul(size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "transfer_buffer",
        })
}

fn checked_range(
    capacity: usize,
    offset: usize,
    elements: usize,
) -> Result<Range<usize>, InputError> {
    match offset.checked_add(elements) {
        Some(end) if offset <= capacity && end <= capacity => Ok(offset..end),
        _ => Err(InputError::InvalidRange {
            field: "buffer",
            offset,
            elements,
            capacity,
        }),
    }
}

fn check_disjoint(target: &Range<usize>, source: &Range<usize>) -> Result<(), TransferError> {
    if target != source
        && !target.is_empty()
        && target.start < source.end
        && source.start < target.end
    {
        return Err(TransferError::OverlappingCopy);
    }
    Ok(())
}

pub(crate) fn host_staging<T: TransferElement>(elements: usize) -> Result<Vec<T>, TransferError> {
    let bytes = checked_bytes::<T>(elements)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(elements)
        .map_err(|_| TransferError::HostAllocation { bytes })?;
    values.resize(elements, T::default());
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_empty_and_endpoint_ranges() {
        assert_eq!(checked_range(5, 0, 5).unwrap(), 0..5);
        assert_eq!(checked_range(5, 5, 0).unwrap(), 5..5);
        assert_eq!(checked_range(0, 0, 0).unwrap(), 0..0);
    }

    #[test]
    fn rejects_invalid_and_overflowing_ranges() {
        for (offset, elements) in [(6, 0), (4, 2), (usize::MAX, 1)] {
            assert_eq!(
                checked_range(5, offset, elements),
                Err(InputError::InvalidRange {
                    field: "buffer",
                    offset,
                    elements,
                    capacity: 5
                })
            );
        }
    }

    #[test]
    fn checks_element_and_slice_byte_capacity() {
        assert_eq!(checked_bytes::<f32>(0).unwrap(), 0);
        assert_eq!(checked_bytes::<i32>(5).unwrap(), 20);
        assert!(checked_bytes::<u32>(usize::MAX).is_err());
        assert!(checked_bytes::<f32>(isize::MAX as usize / 4 + 1).is_err());
        assert_eq!(host_staging::<u32>(3).unwrap(), vec![0; 3]);
    }

    #[test]
    fn rejects_overlap_but_accepts_same_and_disjoint_ranges() {
        assert_eq!(
            check_disjoint(&(1..4), &(2..5)),
            Err(TransferError::OverlappingCopy)
        );
        assert!(check_disjoint(&(0..3), &(3..6)).is_ok());
        assert!(check_disjoint(&(1..4), &(1..4)).is_ok());
        assert!(check_disjoint(&(2..2), &(2..2)).is_ok());
    }

    #[test]
    fn quarantines_backend_failure_and_preserves_first_error() {
        let status = SessionStatus::default();
        let error = ProbeError::Cuda {
            stage: "copy",
            code: 700,
        };
        assert_eq!(
            status.execute::<()>(|| Err(error.clone()), || Ok(())),
            Err(TransferError::Backend(error))
        );
        assert_eq!(status.check(), Err(TransferError::Quarantined));
    }

    #[test]
    fn preserves_healthy_status_after_input_rejection() {
        let status = SessionStatus::default();
        assert!(checked_range(5, 4, 2).is_err());
        assert!(status.check().is_ok());
        status.execute(|| Ok(()), || Ok(())).unwrap();
        assert!(status.check().is_ok());
    }

    #[test]
    fn preserves_operation_failure_when_cleanup_also_fails() {
        let status = SessionStatus::default();
        let error = ProbeError::Cuda {
            stage: "copy",
            code: 700,
        };
        assert_eq!(
            status.execute::<()>(|| Err(error.clone()), || Err(ProbeError::BackendPanic)),
            Err(TransferError::Backend(error))
        );
        assert!(status.uncertain.load(Ordering::Acquire));
    }

    #[test]
    fn waits_after_operation_panic_without_marking_completion_uncertain() {
        let status = SessionStatus::default();
        let waits = std::sync::atomic::AtomicUsize::new(0);
        assert_eq!(
            status.execute::<()>(
                || panic!("mock operation"),
                || {
                    waits.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            ),
            Err(TransferError::Backend(ProbeError::BackendPanic))
        );
        assert_eq!(waits.load(Ordering::SeqCst), 1);
        assert!(!status.uncertain.load(Ordering::Acquire));
        assert_eq!(status.check(), Err(TransferError::Quarantined));
    }

    #[test]
    fn retains_result_owners_when_completion_panics() {
        struct Owner(Arc<std::sync::atomic::AtomicUsize>);
        impl Drop for Owner {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let releases = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let status = SessionStatus::default();
        let result = status.execute(|| Ok(Owner(releases.clone())), || panic!("mock completion"));
        assert!(matches!(
            result,
            Err(TransferError::Backend(ProbeError::BackendPanic))
        ));
        assert_eq!(releases.load(Ordering::SeqCst), 0);
        assert!(status.uncertain.load(Ordering::Acquire));
    }

    #[test]
    fn initializes_owned_host_staging() {
        let status = SessionStatus::default();
        let mut staging = HostStaging::<u32>::new(&status, 3).unwrap();
        assert_eq!(staging.values(), &[0; 3]);
        staging.values_mut().copy_from_slice(&[1, 2, 3]);
        assert_eq!(staging.values(), &[1, 2, 3]);
    }

    #[cfg(not(feature = "cuda-probe"))]
    #[test]
    fn rejects_missing_feature_without_cpu_fallback() {
        assert!(matches!(
            TransferSession::new(0),
            Err(TransferError::Backend(ProbeError::FeatureDisabled(
                "cuda-probe"
            )))
        ));
    }
}
