use super::fence::event_result;
use crate::diagnostics::ProbeError;
use cudarc::driver::{DriverError, sys};

#[test]
fn only_not_ready_means_pending() {
    assert!(event_result(Ok(())).unwrap());
    assert!(!event_result(Err(DriverError(sys::cudaError_enum::CUDA_ERROR_NOT_READY))).unwrap());
    assert_eq!(
        event_result(Err(DriverError(
            sys::cudaError_enum::CUDA_ERROR_ILLEGAL_ADDRESS
        )))
        .unwrap_err(),
        ProbeError::Cuda {
            stage: "lease-query",
            code: 700
        }
    );
}
