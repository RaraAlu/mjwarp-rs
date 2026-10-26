//! 图参数替换单元测试。

use std::panic::AssertUnwindSafe;

use crate::diagnostics::ProbeError;
use crate::runtime::presence;

use super::graph::replace_output;
use super::{prepare_device, require_nvrtc};

#[test]
fn replaces_only_matching_buffer_arguments() {
    let mut args = [10, 20, 20];
    assert!(replace_output(&mut args, 20, 30));
    assert_eq!(args, [10, 30, 20]);
    assert!(!replace_output(&mut args, 99, 30));
    let mut downstream = [20, 40, 50];
    assert!(replace_output(&mut downstream, 20, 30));
    assert_eq!(downstream, [30, 40, 50]);
    let mut original = [10, 20, 50];
    assert!(replace_output(&mut original, 20, 20));
    assert_eq!(original, [10, 20, 50]);
}

/// CLI 路径：无驱动 → `DriverUnavailable`，且没有 panic 逃逸（进程不 abort）。
#[test]
fn prepare_device_without_driver_returns_driver_unavailable_without_panic() {
    let _seam = presence::SeamGuard::lock();
    presence::set_driver_present_for_tests(false);
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| prepare_device(0)));
    match outcome {
        Ok(Err(ProbeError::DriverUnavailable)) => {}
        Ok(Ok(_)) => panic!("缺驱动时应返回 DriverUnavailable"),
        Ok(Err(other)) => panic!("缺驱动时应返回 DriverUnavailable，实际 {other:?}"),
        Err(_) => panic!("panic 逃出了 prepare_device（守卫缺失）"),
    }
}

/// CLI 路径：有驱动但无设备 → `InvalidDevice{requested:0, available:0}`（可诊断、不 panic）。
#[test]
fn prepare_device_without_devices_returns_invalid_device() {
    let _seam = presence::SeamGuard::lock();
    presence::set_driver_present_for_tests(true);
    presence::set_device_count_for_tests(Some(0));
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| prepare_device(0)));
    match outcome {
        Ok(Err(ProbeError::InvalidDevice {
            requested: 0,
            available: 0,
        })) => {}
        Ok(Ok(_)) => panic!("无设备时应返回 InvalidDevice"),
        Ok(Err(other)) => panic!("无设备时应返回 InvalidDevice，实际 {other:?}"),
        Err(_) => panic!("panic 逃出了 prepare_device（守卫缺失）"),
    }
}

/// CLI/探针路径：无 NVRTC → `NvrtcUnavailable`（可诊断、不 panic、不调用编译器）。
#[test]
fn require_nvrtc_without_library_returns_nvrtc_unavailable() {
    let _seam = presence::SeamGuard::lock();
    presence::set_nvrtc_present_for_tests(false);
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(require_nvrtc));
    match outcome {
        Ok(Err(ProbeError::NvrtcUnavailable)) => {}
        Ok(Ok(())) => panic!("缺 NVRTC 时应返回 NvrtcUnavailable"),
        Ok(Err(other)) => panic!("缺 NVRTC 时应返回 NvrtcUnavailable，实际 {other:?}"),
        Err(_) => panic!("panic 逃出了 require_nvrtc（守卫缺失）"),
    }
}
