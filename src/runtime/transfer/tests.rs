use super::session::{HostStaging, SessionStatus};
use super::*;
use crate::diagnostics::{InputError, ProbeError, TransferError};
use std::sync::Arc;
use std::sync::atomic::Ordering;

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
    assert_eq!(checked_bytes::<f64>(5).unwrap(), 40);
    assert!(checked_bytes::<u32>(usize::MAX).is_err());
    assert!(checked_bytes::<f32>(isize::MAX as usize / 4 + 1).is_err());
    assert!(checked_bytes::<f64>(isize::MAX as usize / 8 + 1).is_err());
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

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver"]
fn launcher_span_uses_cached_address_and_rejects_quarantined_session() {
    let session = TransferSession::new(0).unwrap();
    let buffer = session.upload(&[1i32]).unwrap();
    let first = buffer.launcher_span().unwrap();
    let second = buffer.launcher_span().unwrap();
    assert_eq!(first, second);
    session.inner.status.failed.store(true, Ordering::Release);
    assert_eq!(buffer.launcher_span(), Err(TransferError::Quarantined));
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
