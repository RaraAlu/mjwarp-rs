//! 内部字节范围租约。
//! 类型不冻结外部设备ABI。

use crate::diagnostics::{ProbeError, ResourceError};
use std::{
    ops::Range,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeviceIdentity {
    pub device: usize,
    pub context: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Access {
    Read,
    Write,
}

#[derive(Clone, Copy)]
pub(super) struct ViewRequest {
    pub target: DeviceIdentity,
    pub offset: usize,
    pub bytes: usize,
    pub layout: u64,
    pub access: Access,
}

struct Reservation {
    id: u64,
    range: Range<usize>,
    access: Access,
}
struct State {
    layout: u64,
    next_id: u64,
    active: Vec<Reservation>,
}
struct Inner<R> {
    owner: Arc<R>,
    identity: DeviceIdentity,
    capacity: usize,
    element_bytes: usize,
    writable: bool,
    quarantined: AtomicBool,
    state: Mutex<State>,
}

pub(super) struct LeasedBuffer<R> {
    inner: Arc<Inner<R>>,
}

impl<R> Clone for LeasedBuffer<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<R> LeasedBuffer<R> {
    // 适配层从真实所有者提取这些属性。
    // 调用方将所有操作转交此包装。
    pub fn new(
        owner: R,
        identity: DeviceIdentity,
        capacity: usize,
        element_bytes: usize,
        writable: bool,
    ) -> Result<Self, ProbeError> {
        if element_bytes == 0
            || !element_bytes.is_power_of_two()
            || capacity == 0
            || !capacity.is_multiple_of(element_bytes)
        {
            return Err(ProbeError::InvalidArgument("缓冲布局不合法"));
        }
        Ok(Self {
            inner: Arc::new(Inner {
                owner: Arc::new(owner),
                identity,
                capacity,
                element_bytes,
                writable,
                quarantined: AtomicBool::new(false),
                state: Mutex::new(State {
                    layout: 1,
                    next_id: 0,
                    active: Vec::new(),
                }),
            }),
        })
    }

    pub fn owner_weak(&self) -> Weak<R> {
        Arc::downgrade(&self.inner.owner)
    }

    pub fn lease(&self, request: ViewRequest) -> Result<BufferLease<R>, ProbeError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        if self.inner.quarantined.load(Ordering::Acquire) {
            return Err(ResourceError::Quarantined.into());
        }
        if request.target.device != self.inner.identity.device {
            return Err(ResourceError::DeviceMismatch.into());
        }
        if request.target.context != self.inner.identity.context {
            return Err(ResourceError::ContextMismatch.into());
        }
        if request.layout != state.layout {
            return Err(ResourceError::StaleLayout {
                expected: state.layout,
                actual: request.layout,
            }
            .into());
        }
        if request.access == Access::Write && !self.inner.writable {
            return Err(ResourceError::ReadOnly.into());
        }
        let end = request
            .offset
            .checked_add(request.bytes)
            .filter(|&end| request.bytes != 0 && end <= self.inner.capacity)
            .ok_or(ResourceError::InvalidRange {
                offset: request.offset,
                bytes: request.bytes,
                capacity: self.inner.capacity,
            })?;
        if !request.offset.is_multiple_of(self.inner.element_bytes)
            || !request.bytes.is_multiple_of(self.inner.element_bytes)
        {
            return Err(ResourceError::Misaligned.into());
        }
        let range = request.offset..end;
        if state.active.iter().any(|lease| {
            range.start < lease.range.end
                && lease.range.start < range.end
                && (request.access == Access::Write || lease.access == Access::Write)
        }) {
            return Err(ResourceError::Conflict.into());
        }
        let id = state.next_id;
        state.next_id = id.checked_add(1).ok_or(ResourceError::VersionExhausted)?;
        state.active.push(Reservation {
            id,
            range: range.clone(),
            access: request.access,
        });
        Ok(BufferLease {
            inner: self.inner.clone(),
            id,
            range,
        })
    }

    pub fn invalidate_layout(&self) -> Result<u64, ProbeError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        if self.inner.quarantined.load(Ordering::Acquire) {
            return Err(ResourceError::Quarantined.into());
        }
        if !state.active.is_empty() {
            return Err(ResourceError::LayoutBusy.into());
        }
        state.layout = state
            .layout
            .checked_add(1)
            .ok_or(ResourceError::VersionExhausted)?;
        Ok(state.layout)
    }
}

pub(super) struct BufferLease<R> {
    inner: Arc<Inner<R>>,
    id: u64,
    range: Range<usize>,
}

impl<R> BufferLease<R> {
    pub fn owner(&self) -> &R {
        &self.inner.owner
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn quarantine(&self) {
        let _state = self.inner.state.lock();
        self.inner.quarantined.store(true, Ordering::Release);
    }
}

impl<R> Drop for BufferLease<R> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.inner.state.lock() {
            state.active.retain(|lease| lease.id != self.id);
        } else {
            // 中毒状态拒绝新租约。
            // 此处不再次获取同一锁。
            self.inner.quarantined.store(true, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer(writable: bool) -> LeasedBuffer<Vec<u32>> {
        LeasedBuffer::new(
            vec![0; 16],
            DeviceIdentity {
                device: 0,
                context: 7,
            },
            64,
            4,
            writable,
        )
        .unwrap()
    }
    fn request(access: Access) -> ViewRequest {
        ViewRequest {
            target: DeviceIdentity {
                device: 0,
                context: 7,
            },
            offset: 0,
            bytes: 32,
            layout: 1,
            access,
        }
    }

    #[test]
    fn shares_reads_and_rejects_overlapping_writes() {
        let buffer = buffer(true);
        let first = buffer.lease(request(Access::Read)).unwrap();
        let second = buffer.clone().lease(request(Access::Read)).unwrap();
        assert_eq!(first.owner().len(), 16);
        assert_eq!(first.range(), 0..32);
        assert!(matches!(
            buffer.lease(request(Access::Write)),
            Err(ProbeError::Resource(ResourceError::Conflict))
        ));
        drop(first);
        drop(second);
        let write = buffer.lease(request(Access::Write)).unwrap();
        assert!(buffer.lease(request(Access::Read)).is_err());
        assert!(buffer.lease(request(Access::Write)).is_err());
        drop(write);
        buffer.lease(request(Access::Read)).unwrap();
    }

    #[test]
    fn permits_disjoint_writes_and_preserves_owner() {
        let buffer = buffer(true);
        let weak = buffer.owner_weak();
        let first = buffer.lease(request(Access::Write)).unwrap();
        let second = buffer
            .lease(ViewRequest {
                offset: 32,
                ..request(Access::Write)
            })
            .unwrap();
        drop(buffer);
        assert!(weak.upgrade().is_some());
        drop(first);
        drop(second);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn checks_device_context_and_write_permission() {
        let buffer = buffer(false);
        assert!(matches!(
            buffer.lease(request(Access::Write)),
            Err(ProbeError::Resource(ResourceError::ReadOnly))
        ));
        let mut view = request(Access::Read);
        view.target.device = 1;
        assert!(matches!(
            buffer.lease(view),
            Err(ProbeError::Resource(ResourceError::DeviceMismatch))
        ));
        view.target = DeviceIdentity {
            device: 0,
            context: 8,
        };
        assert!(matches!(
            buffer.lease(view),
            Err(ProbeError::Resource(ResourceError::ContextMismatch))
        ));
    }

    #[test]
    fn rejects_empty_overflowing_out_of_bounds_and_unaligned_ranges() {
        let buffer = buffer(true);
        for (offset, bytes) in [(0, 0), (0, 68), (64, 4), (usize::MAX, 8), (1, 4), (0, 3)] {
            assert!(
                buffer
                    .lease(ViewRequest {
                        offset,
                        bytes,
                        ..request(Access::Read)
                    })
                    .is_err()
            );
        }
        for (capacity, element_bytes) in [(0, 4), (64, 0), (63, 4), (64, 3)] {
            assert!(
                LeasedBuffer::new(
                    vec![0u32; 16],
                    request(Access::Read).target,
                    capacity,
                    element_bytes,
                    true
                )
                .is_err()
            );
        }
    }

    #[test]
    fn rejects_in_flight_layout_changes_and_old_views() {
        let buffer = buffer(true);
        let lease = buffer.lease(request(Access::Read)).unwrap();
        assert_eq!(
            buffer.invalidate_layout().unwrap_err(),
            ResourceError::LayoutBusy.into()
        );
        drop(lease);
        assert_eq!(buffer.invalidate_layout().unwrap(), 2);
        assert!(matches!(
            buffer.lease(request(Access::Read)),
            Err(ProbeError::Resource(ResourceError::StaleLayout {
                expected: 2,
                actual: 1
            }))
        ));
        buffer
            .lease(ViewRequest {
                layout: 2,
                ..request(Access::Read)
            })
            .unwrap();
    }

    #[test]
    fn quarantines_failed_resources_and_checks_counter_exhaustion() {
        let buffer = buffer(true);
        let lease = buffer.lease(request(Access::Read)).unwrap();
        lease.quarantine();
        assert!(matches!(
            buffer.lease(request(Access::Read)),
            Err(ProbeError::Resource(ResourceError::Quarantined))
        ));
        assert_eq!(
            buffer.invalidate_layout().unwrap_err(),
            ResourceError::Quarantined.into()
        );
        drop(lease);
        let other = super::tests::buffer(true);
        other.inner.state.lock().unwrap().next_id = u64::MAX;
        assert!(matches!(
            other.lease(request(Access::Read)),
            Err(ProbeError::Resource(ResourceError::VersionExhausted))
        ));
        other.inner.state.lock().unwrap().layout = u64::MAX;
        assert_eq!(
            other.invalidate_layout().unwrap_err(),
            ResourceError::VersionExhausted.into()
        );
    }

    #[test]
    fn reports_poisoned_registry_without_reusing_it() {
        let buffer = buffer(true);
        let lease = buffer.lease(request(Access::Read)).unwrap();
        let clone = buffer.clone();
        assert!(
            std::panic::catch_unwind(|| {
                let _lock = clone.inner.state.lock().unwrap();
                panic!("test poison");
            })
            .is_err()
        );
        assert!(matches!(
            buffer.lease(request(Access::Read)),
            Err(ProbeError::Resource(ResourceError::Poisoned))
        ));
        assert_eq!(
            buffer.invalidate_layout().unwrap_err(),
            ResourceError::Poisoned.into()
        );
        drop(lease);
    }

    #[test]
    fn serializes_competing_thread_writes() {
        let buffer = buffer(true);
        let start = Arc::new(std::sync::Barrier::new(3));
        let finish = Arc::new(std::sync::Barrier::new(3));
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let (buffer, start, finish) = (buffer.clone(), start.clone(), finish.clone());
                std::thread::spawn(move || {
                    start.wait();
                    let lease = buffer.lease(request(Access::Write));
                    let success = lease.is_ok();
                    finish.wait();
                    drop(lease);
                    success
                })
            })
            .collect();
        start.wait();
        finish.wait();
        assert_eq!(
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .filter(|&success| success)
                .count(),
            1
        );
    }
}
