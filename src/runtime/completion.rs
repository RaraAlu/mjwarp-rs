//! 队列独立保留在途资源。
//! 令牌只观察完成状态。

use crate::diagnostics::{ProbeError, ResourceError};
use std::sync::{Arc, Mutex};

pub(super) trait Fence {
    fn query(&self) -> Result<bool, ProbeError>;
    fn wait(&self) -> Result<(), ProbeError>;
}

pub(super) trait RetainedResources {
    fn quarantine(&self);
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum CompletionStatus {
    Pending,
    Completed,
    Failed(ProbeError),
}

#[derive(Clone)]
pub(super) struct CompletionToken {
    state: Arc<Mutex<CompletionStatus>>,
}

impl CompletionToken {
    pub fn status(&self) -> CompletionStatus {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
    fn set(&self, state: CompletionStatus) {
        *self.state.lock().unwrap_or_else(|error| error.into_inner()) = state;
    }
}

struct Entry<F, R> {
    fence: F,
    resources: R,
    token: CompletionToken,
}

pub(super) struct CompletionQueue<F: Fence, R: RetainedResources> {
    entries: Vec<Entry<F, R>>,
}

impl<F: Fence, R: RetainedResources> CompletionQueue<F, R> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // 调用方必须先登记，再启动GPU工作。
    pub fn retain(&mut self, resources: R, fence: F) -> CompletionToken {
        let token = CompletionToken {
            state: Arc::new(Mutex::new(CompletionStatus::Pending)),
        };
        self.entries.push(Entry {
            fence,
            resources,
            token: token.clone(),
        });
        token
    }

    fn index(&self, token: &CompletionToken) -> Result<usize, ProbeError> {
        self.entries
            .iter()
            .position(|entry| Arc::ptr_eq(&entry.token.state, &token.state))
            .ok_or_else(|| ResourceError::InvalidCompletion.into())
    }

    pub fn retained(&mut self, token: &CompletionToken) -> Result<(&mut R, &mut F), ProbeError> {
        let index = self.index(token)?;
        let entry = &mut self.entries[index];
        if let CompletionStatus::Failed(error) = entry.token.status() {
            return Err(error);
        }
        if entry.token.status() == CompletionStatus::Completed {
            return Err(ResourceError::InvalidCompletion.into());
        }
        Ok((&mut entry.resources, &mut entry.fence))
    }

    pub fn query(&mut self, token: &CompletionToken) -> Result<bool, ProbeError> {
        let index = self.index(token)?;
        let entry = &self.entries[index];
        if let CompletionStatus::Failed(error) = entry.token.status() {
            return Err(error);
        }
        match entry.fence.query() {
            Ok(ready) => {
                if ready {
                    entry.token.set(CompletionStatus::Completed);
                }
                Ok(ready)
            }
            Err(error) => {
                entry.resources.quarantine();
                entry.token.set(CompletionStatus::Failed(error.clone()));
                Err(error)
            }
        }
    }

    // 提交失败仍保留资源至清理等待。
    pub fn fail(&mut self, token: &CompletionToken, error: ProbeError) -> Result<(), ProbeError> {
        let index = self.index(token)?;
        let entry = &self.entries[index];
        entry.resources.quarantine();
        if !matches!(entry.token.status(), CompletionStatus::Failed(_)) {
            entry.token.set(CompletionStatus::Failed(error));
        }
        Ok(())
    }

    pub fn wait(&mut self, token: &CompletionToken) -> Result<R, ProbeError> {
        let index = self.index(token)?;
        if let CompletionStatus::Failed(error) = self.entries[index].token.status() {
            return Err(error);
        }
        if let Err(error) = self.entries[index].fence.wait() {
            self.entries[index].resources.quarantine();
            token.set(CompletionStatus::Failed(error.clone()));
            return Err(error);
        }
        let entry = self.entries.swap_remove(index);
        token.set(CompletionStatus::Completed);
        Ok(entry.resources)
    }
}

impl<F: Fence, R: RetainedResources> Drop for CompletionQueue<F, R> {
    fn drop(&mut self) {
        for entry in self.entries.drain(..) {
            let waited =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| entry.fence.wait()));
            match waited {
                Ok(Ok(())) => {
                    if entry.token.status() == CompletionStatus::Pending {
                        entry.token.set(CompletionStatus::Completed);
                    }
                }
                outcome => {
                    let error = match outcome {
                        Ok(Err(error)) => error,
                        _ => ProbeError::BackendPanic,
                    };
                    entry.resources.quarantine();
                    if !matches!(entry.token.status(), CompletionStatus::Failed(_)) {
                        entry.token.set(CompletionStatus::Failed(error));
                    }
                    // 无法证明完成时保守保留资源。
                    // 同时保留上下文及完成栅栏。
                    std::mem::forget(entry);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::lease::{Access, BufferLease, DeviceIdentity, LeasedBuffer, ViewRequest};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    struct MockFence {
        ready: Arc<AtomicBool>,
        waits: Arc<AtomicUsize>,
        error: bool,
        panic: bool,
    }
    impl Fence for MockFence {
        fn query(&self) -> Result<bool, ProbeError> {
            if self.error {
                Err(ProbeError::Cuda {
                    stage: "mock-query",
                    code: 700,
                })
            } else {
                Ok(self.ready.load(Ordering::Acquire))
            }
        }
        fn wait(&self) -> Result<(), ProbeError> {
            self.waits.fetch_add(1, Ordering::Relaxed);
            assert!(!self.panic, "test wait panic");
            if self.error {
                Err(ProbeError::Cuda {
                    stage: "mock-wait",
                    code: 700,
                })
            } else {
                self.ready.store(true, Ordering::Release);
                Ok(())
            }
        }
    }
    impl RetainedResources for BufferLease<Vec<u32>> {
        fn quarantine(&self) {
            self.quarantine();
        }
    }
    fn buffer() -> LeasedBuffer<Vec<u32>> {
        LeasedBuffer::new(
            vec![0; 16],
            DeviceIdentity {
                device: 0,
                context: 7,
            },
            64,
            4,
            true,
        )
        .unwrap()
    }
    fn lease(buffer: &LeasedBuffer<Vec<u32>>) -> BufferLease<Vec<u32>> {
        buffer
            .lease(ViewRequest {
                target: DeviceIdentity {
                    device: 0,
                    context: 7,
                },
                offset: 0,
                bytes: 64,
                layout: 1,
                access: Access::Write,
            })
            .unwrap()
    }
    fn fence(error: bool, panic: bool) -> MockFence {
        MockFence {
            ready: Arc::new(AtomicBool::new(false)),
            waits: Arc::new(AtomicUsize::new(0)),
            error,
            panic,
        }
    }

    #[test]
    fn keeps_owner_until_wait_and_rejects_second_retirement() {
        let buffer = buffer();
        let weak = buffer.owner_weak();
        let mut queue = CompletionQueue::new();
        let token = queue.retain(lease(&buffer), fence(false, false));
        assert_eq!(token.status(), CompletionStatus::Pending);
        assert!(!queue.query(&token).unwrap());
        queue.retained(&token).unwrap();
        drop(buffer);
        assert!(weak.upgrade().is_some());
        let resources = queue.wait(&token).unwrap();
        assert_eq!(token.status(), CompletionStatus::Completed);
        assert!(queue.wait(&token).is_err());
        assert!(weak.upgrade().is_some());
        drop(resources);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn token_drop_and_forget_do_not_control_owner_lifetime() {
        for forget in [false, true] {
            let buffer = buffer();
            let weak = buffer.owner_weak();
            let mut queue = CompletionQueue::new();
            let token = queue.retain(lease(&buffer), fence(false, false));
            drop(buffer);
            if forget {
                std::mem::forget(token);
            } else {
                drop(token);
            }
            assert!(weak.upgrade().is_some());
            drop(queue);
            assert!(weak.upgrade().is_none());
        }
    }

    #[test]
    fn complete_query_does_not_release_the_lease() {
        let buffer = buffer();
        let mock = fence(false, false);
        let ready = mock.ready.clone();
        let mut queue = CompletionQueue::new();
        let token = queue.retain(lease(&buffer), mock);
        ready.store(true, Ordering::Release);
        assert!(queue.query(&token).unwrap());
        assert_eq!(token.status(), CompletionStatus::Completed);
        assert!(queue.retained(&token).is_err());
        assert!(lease_result(&buffer).is_err());
        drop(queue.wait(&token).unwrap());
        lease(&buffer);
    }
    fn lease_result(buffer: &LeasedBuffer<Vec<u32>>) -> Result<BufferLease<Vec<u32>>, ProbeError> {
        buffer.lease(ViewRequest {
            target: DeviceIdentity {
                device: 0,
                context: 7,
            },
            offset: 0,
            bytes: 64,
            layout: 1,
            access: Access::Read,
        })
    }

    #[test]
    fn rejects_tokens_from_another_queue() {
        let buffer = buffer();
        let mut first = CompletionQueue::new();
        let token = first.retain(lease(&buffer), fence(false, false));
        let mut other: CompletionQueue<MockFence, BufferLease<Vec<u32>>> = CompletionQueue::new();
        assert_eq!(
            other.query(&token).unwrap_err(),
            ResourceError::InvalidCompletion.into()
        );
        assert!(other.retained(&token).is_err());
        assert!(other.wait(&token).is_err());
        assert!(other.fail(&token, ProbeError::BackendPanic).is_err());
        assert_eq!(token.status(), CompletionStatus::Pending);
    }

    #[test]
    fn preserves_submission_failure_after_successful_cleanup() {
        let buffer = buffer();
        let weak = buffer.owner_weak();
        let mock = fence(false, false);
        let waits = mock.waits.clone();
        let mut queue = CompletionQueue::new();
        let token = queue.retain(lease(&buffer), mock);
        queue
            .fail(
                &token,
                ProbeError::InvalidArgument("test submission failure"),
            )
            .unwrap();
        queue.fail(&token, ProbeError::BackendPanic).unwrap();
        assert!(queue.query(&token).is_err());
        assert!(queue.retained(&token).is_err());
        drop(buffer);
        drop(queue);
        assert_eq!(waits.load(Ordering::Acquire), 1);
        assert!(weak.upgrade().is_none());
        assert_eq!(
            token.status(),
            CompletionStatus::Failed(ProbeError::InvalidArgument("test submission failure"))
        );
    }

    #[test]
    fn retires_multiple_entries_out_of_order() {
        let first = buffer();
        let second = buffer();
        let first_weak = first.owner_weak();
        let second_weak = second.owner_weak();
        let mut queue = CompletionQueue::new();
        let a = queue.retain(lease(&first), fence(false, false));
        let b = queue.retain(lease(&second), fence(false, false));
        drop(first);
        drop(second);
        drop(queue.wait(&b).unwrap());
        assert!(second_weak.upgrade().is_none());
        assert!(first_weak.upgrade().is_some());
        assert_eq!(a.status(), CompletionStatus::Pending);
        drop(queue.wait(&a).unwrap());
        assert!(first_weak.upgrade().is_none());
    }

    #[test]
    fn keeps_failed_wait_resources_quarantined_and_alive() {
        let buffer = buffer();
        let weak = buffer.owner_weak();
        let mut queue = CompletionQueue::new();
        let token = queue.retain(lease(&buffer), fence(true, false));
        assert!(matches!(
            queue.wait(&token),
            Err(ProbeError::Cuda {
                stage: "mock-wait",
                code: 700
            })
        ));
        assert!(matches!(
            lease_result(&buffer),
            Err(ProbeError::Resource(ResourceError::Quarantined))
        ));
        assert!(queue.retained(&token).is_err());
        drop(buffer);
        drop(queue);
        assert!(weak.upgrade().is_some());
        assert!(matches!(
            token.status(),
            CompletionStatus::Failed(ProbeError::Cuda { code: 700, .. })
        ));
    }

    #[test]
    fn failed_query_never_becomes_success_after_cleanup() {
        let buffer = buffer();
        let weak = buffer.owner_weak();
        let mut queue = CompletionQueue::new();
        let token = queue.retain(lease(&buffer), fence(true, false));
        assert!(queue.query(&token).is_err());
        queue.entries[0].fence.error = false;
        assert!(queue.wait(&token).is_err());
        assert!(queue.query(&token).is_err());
        drop(buffer);
        drop(queue);
        assert!(weak.upgrade().is_none());
        assert!(matches!(token.status(), CompletionStatus::Failed(_)));
    }

    #[test]
    fn drains_resources_during_host_unwind() {
        let buffer = buffer();
        let weak = buffer.owner_weak();
        assert!(
            std::panic::catch_unwind(|| {
                let mut queue = CompletionQueue::new();
                let _token = queue.retain(lease(&buffer), fence(false, false));
                drop(buffer);
                panic!("test caller panic");
            })
            .is_err()
        );
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn drop_contains_fence_panics_without_freeing_uncertain_owners() {
        let buffer = buffer();
        let weak = buffer.owner_weak();
        let mut queue = CompletionQueue::new();
        let token = queue.retain(lease(&buffer), fence(false, true));
        drop(buffer);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(queue))).is_ok());
        assert!(weak.upgrade().is_some());
        assert_eq!(
            token.status(),
            CompletionStatus::Failed(ProbeError::BackendPanic)
        );
    }
}
