//! 三态回退与「不 panic」的纯 CPU 单测（不创建真实 `CudaContext`；seam 仅 `#[cfg(test)]`）。
//!
//! 覆盖：无驱动 → `NoDevice`；有驱动无设备 → `NoDevice`；`CudaContext::new` 内部 panic 被捕获
//! → `NoDevice`（进程不 abort）；无 NVRTC → `Compile`（编译先于设备构造）。每条测试都在独立
//! 进程运行时也成立（证据里按路径分别单独运行）。

use std::panic::AssertUnwindSafe;

use crate::runtime::presence;

use super::error::CollisionGpuError;
use super::{CollisionGpuProgram, context::device_context};

/// 无驱动：预检直接返回 `NoDevice`，且外部 `catch_unwind` 能证明没有 panic 逃逸。
#[test]
fn device_context_without_driver_returns_nodevice_without_panic() {
    let _seam = presence::SeamGuard::lock();
    presence::set_driver_present_for_tests(false);
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| device_context(0)));
    match outcome {
        Ok(Err(CollisionGpuError::NoDevice(detail))) => {
            assert!(detail.contains("驱动库缺失"), "detail={detail}");
        }
        Ok(Ok(_)) => panic!("缺驱动时应返回 NoDevice，而不是构造出 context"),
        Ok(Err(other)) => panic!("缺驱动时应返回 NoDevice，实际 {other:?}"),
        Err(_) => panic!("panic 逃出了 device_context（守卫缺失）"),
    }
}

/// 有驱动无设备：`CudaContext::new` 返回错误 → `NoDevice`（可诊断、不 panic）。
#[test]
fn device_context_without_usable_device_returns_nodevice() {
    let _seam = presence::SeamGuard::lock();
    presence::set_driver_present_for_tests(true);
    presence::set_context_mode_for_tests(presence::CONTEXT_ERROR);
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| device_context(0)));
    match outcome {
        Ok(Err(CollisionGpuError::NoDevice(detail))) => {
            assert!(detail.contains("CudaContext::new"), "detail={detail}");
        }
        Ok(Ok(_)) => panic!("设备不可用时应返回 NoDevice"),
        Ok(Err(other)) => panic!("设备不可用时应返回 NoDevice，实际 {other:?}"),
        Err(_) => panic!("panic 逃出了 device_context（守卫缺失）"),
    }
}

/// `CudaContext::new` 内部 panic（模拟 cudarc `culib()` 缺库）必须被捕获：返回 `NoDevice`，
/// panic 不逃逸（调用方进程不 abort）。
#[test]
fn device_context_initialization_panic_is_caught() {
    let _seam = presence::SeamGuard::lock();
    presence::set_driver_present_for_tests(true);
    presence::set_context_mode_for_tests(presence::CONTEXT_PANIC);
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| device_context(0)));
    match outcome {
        Ok(Err(CollisionGpuError::NoDevice(detail))) => {
            assert!(detail.contains("panic"), "detail={detail}");
            assert!(detail.contains("已捕获"), "detail={detail}");
        }
        Ok(Ok(_)) => panic!("注入 panic 时应返回 NoDevice"),
        Ok(Err(other)) => panic!("注入 panic 时应返回 NoDevice，实际 {other:?}"),
        Err(_) => panic!("panic 逃出了 device_context（守卫缺失）"),
    }
}

/// 无 NVRTC：产品 API 返回 `Compile`，且在设备构造之前短路（不创建 context）。
#[test]
fn constructor_without_nvrtc_returns_compile_without_panic() {
    let _seam = presence::SeamGuard::lock();
    presence::set_driver_present_for_tests(true);
    presence::set_nvrtc_present_for_tests(false);
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| CollisionGpuProgram::new(0)));
    match outcome {
        Ok(Err(CollisionGpuError::Compile(detail))) => {
            assert!(detail.contains("NVRTC"), "detail={detail}");
        }
        Ok(Ok(_)) => panic!("缺 NVRTC 时应返回 Compile"),
        Ok(Err(other)) => panic!("缺 NVRTC 时应返回 Compile，实际 {other:?}"),
        Err(_) => panic!("panic 逃出了 CollisionGpuProgram::new（守卫缺失）"),
    }
}
