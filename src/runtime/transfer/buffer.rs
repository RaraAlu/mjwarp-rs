//! 独占设备缓冲与范围复制。

#[cfg(feature = "cuda-probe")]
use super::session::HostStaging;
use super::{TransferElement, TransferSession, check_disjoint, checked_range};
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::InputError;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::DeviceSpan;
#[cfg(feature = "cuda-probe")]
use cudarc::driver::CudaSlice;
use std::sync::Arc;
#[cfg(feature = "cuda-probe")]
use std::sync::atomic::Ordering;

/// 缓冲独占设备分配。
/// 缓冲保留流与实际上下文。
/// 不提供clone或外部导入。
pub struct TransferBuffer<T: TransferElement> {
    pub(super) session: TransferSession,
    pub(super) len: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) allocation: Option<CudaSlice<T>>,
    #[cfg(feature = "cuda-probe")]
    pub(super) device_addr: u64,
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

    /// 返回当前会话内的真实设备跨度。
    ///
    /// 地址在同步分配阶段缓存。
    /// 查询不执行任何驱动操作。
    #[cfg(feature = "cuda-probe")]
    pub(crate) fn launcher_span(&self) -> Result<DeviceSpan, TransferError> {
        self.session.inner.status.check()?;
        if self.allocation.is_none() {
            return Err(InputError::InvalidDimension {
                field: "launcher_buffer",
            }
            .into());
        }
        let bytes = u64::try_from(self.byte_len()).map_err(|_| InputError::Overflow {
            field: "launcher_buffer_bytes",
        })?;
        let context = Arc::as_ptr(&self.session.inner) as usize as u64;
        DeviceSpan::new(context, self.device_addr, bytes).map_err(|_| {
            TransferError::Input(InputError::InvalidDimension {
                field: "launcher_buffer_span",
            })
        })
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
