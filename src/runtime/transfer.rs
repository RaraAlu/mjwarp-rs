//! 类型化同步传输辅助。
//! 不导出设备地址或上下文。
//! 不冻结生产运行时接口。

#[cfg(feature = "cuda-probe")]
pub(crate) mod kernel;

mod buffer;
mod session;
#[cfg(test)]
mod tests;

use crate::diagnostics::{InputError, TransferError};
use std::ops::Range;

pub use buffer::TransferBuffer;
pub use session::TransferSession;

mod element {
    #[cfg(feature = "cuda-probe")]
    pub trait Sealed:
        cudarc::driver::DeviceRepr + cudarc::driver::ValidAsZeroBits + Copy + Default
    {
    }
    #[cfg(not(feature = "cuda-probe"))]
    pub trait Sealed: Copy + Default {}

    impl Sealed for f32 {}
    impl Sealed for f64 {}
    impl Sealed for i32 {}
    impl Sealed for u32 {}
}

/// 支持f32、f64、i32与u32。
/// 调用方不能增加设备类型。
///
/// ```compile_fail
/// use mjwarp_rs::runtime::TransferSession;
/// let session = TransferSession::new(0).unwrap();
/// session.upload(&[true]);
/// ```
pub trait TransferElement: element::Sealed {}
impl TransferElement for f32 {}
impl TransferElement for f64 {}
impl TransferElement for i32 {}
impl TransferElement for u32 {}

/// M5：设备缓冲可作为异步提交凭证的保留资源。
/// 隔离时使所属会话失败，阻止后续传输。
impl<T: TransferElement> crate::runtime::launcher::RetainedResources for TransferBuffer<T> {
    fn quarantine(&self) {
        self.session
            .inner
            .status
            .failed
            .store(true, std::sync::atomic::Ordering::Release);
        #[cfg(any(feature = "cuda-probe", test))]
        self.session
            .inner
            .status
            .uncertain
            .store(true, std::sync::atomic::Ordering::Release);
    }
}

impl TransferSession {
    /// M6：捕获前预分配显存；不写入内容。
    /// 捕获期间的分配由捕获守卫在调用路径上拒绝。
    pub fn preallocate<T: TransferElement>(
        &self,
        elements: usize,
    ) -> Result<TransferBuffer<T>, TransferError> {
        checked_bytes::<T>(elements)?;
        self.inner.status.check()?;
        #[cfg(feature = "cuda-probe")]
        {
            use cudarc::driver::DevicePtr;
            let (allocation, device_addr) = if elements == 0 {
                (None, 0)
            } else {
                let (allocation, address) = self.run("preallocate", || {
                    let allocation = self.inner.stream.alloc_zeros::<T>(elements)?;
                    let (address, read) = allocation.device_ptr(&self.inner.stream);
                    drop(read);
                    Ok((allocation, address))
                })?;
                (Some(allocation), address)
            };
            Ok(TransferBuffer {
                session: self.clone(),
                allocation,
                device_addr,
                len: elements,
            })
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = elements;
            Err(crate::diagnostics::ProbeError::FeatureDisabled("cuda-probe").into())
        }
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
