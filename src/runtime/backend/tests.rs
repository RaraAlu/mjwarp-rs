//! 窄相后端选择层 seam 测试（纯 CPU：不创建真实 CUDA context、不编译真实 PTX）。
//!
//! 覆盖探测顺序的每一档失败（驱动/设备/NVRTC/编译）与 CudaReady，并与 CLI 既有
//! `DriverUnavailable`/`NvrtcUnavailable` 错误码做一致性断言（CLI 与 API 同源探测）。

#[cfg(feature = "cuda-probe")]
mod cuda_probe {
    use crate::runtime::backend::{
        BackendChoice, BackendStatus, BackendUnavailable, COMPILE_FAIL, COMPILE_READY,
        device_permitted, probe_backend, set_compile_override_for_tests,
    };
    use crate::runtime::presence;
    use crate::runtime::{backend, cuda};

    /// 测试结束（含断言失败）时清空编译注入，避免污染其它测试。
    struct CompileReset;

    impl Drop for CompileReset {
        fn drop(&mut self) {
            set_compile_override_for_tests(0);
        }
    }

    fn reset() {
        set_compile_override_for_tests(0);
        presence::set_device_count_for_tests(None);
    }

    /// 无驱动：选择层报 `DriverUnavailable`，CLI 同源探测也报 `DriverUnavailable`。
    #[test]
    fn driver_missing_reports_driver_unavailable_and_matches_cli() {
        let _seam = presence::SeamGuard::lock();
        let _reset = CompileReset;
        reset();
        presence::set_driver_present_for_tests(false);
        assert_eq!(
            probe_backend(),
            BackendStatus::HostOnly(BackendUnavailable::DriverUnavailable)
        );
        assert_eq!(
            cuda::require_driver().err(),
            Some(crate::diagnostics::ProbeError::DriverUnavailable)
        );
    }

    /// 有驱动无设备：选择层报 `NoDevice`（可诊断原因），不 panic。
    #[test]
    fn no_device_reports_nodevice() {
        let _seam = presence::SeamGuard::lock();
        let _reset = CompileReset;
        reset();
        presence::set_driver_present_for_tests(true);
        presence::set_device_count_for_tests(Some(0));
        match probe_backend() {
            BackendStatus::HostOnly(BackendUnavailable::NoDevice(detail)) => {
                assert!(detail.contains("无可用"), "detail={detail}");
            }
            other => panic!("期望 NoDevice，实际 {other:?}"),
        }
    }

    /// 有设备无 NVRTC：选择层报 `NvrtcUnavailable`，CLI 同源探测同码。
    #[test]
    fn nvrtc_missing_reports_nvrtc_unavailable_and_matches_cli() {
        let _seam = presence::SeamGuard::lock();
        let _reset = CompileReset;
        reset();
        presence::set_driver_present_for_tests(true);
        presence::set_device_count_for_tests(Some(1));
        presence::set_nvrtc_present_for_tests(false);
        assert_eq!(
            probe_backend(),
            BackendStatus::HostOnly(BackendUnavailable::NvrtcUnavailable)
        );
        assert_eq!(
            cuda::require_nvrtc().err(),
            Some(crate::diagnostics::ProbeError::NvrtcUnavailable)
        );
    }

    /// 探测全通过（编译步骤用 seam 注入）：`CudaReady`；默认 Host 选择不被允许。
    #[test]
    fn all_steps_ready_reports_cuda_ready_but_default_is_host() {
        let _seam = presence::SeamGuard::lock();
        let _reset = CompileReset;
        reset();
        presence::set_driver_present_for_tests(true);
        presence::set_device_count_for_tests(Some(1));
        presence::set_nvrtc_present_for_tests(true);
        set_compile_override_for_tests(COMPILE_READY);
        assert_eq!(probe_backend(), BackendStatus::CudaReady);
        assert!(!device_permitted(BackendChoice::Host));
    }

    /// 编译失败：`HostOnly(Compile(..))`，原因可诊断、不 panic。
    #[test]
    fn compile_failure_reports_compile_error() {
        let _seam = presence::SeamGuard::lock();
        let _reset = CompileReset;
        reset();
        presence::set_driver_present_for_tests(true);
        presence::set_device_count_for_tests(Some(1));
        presence::set_nvrtc_present_for_tests(true);
        set_compile_override_for_tests(COMPILE_FAIL);
        match probe_backend() {
            BackendStatus::HostOnly(BackendUnavailable::Compile(detail)) => {
                assert!(detail.contains("注入"), "detail={detail}");
            }
            other => panic!("期望 Compile，实际 {other:?}"),
        }
    }

    /// 缓存：同一进程内 `narrowphase_backend_status` 返回同一静态引用（只探测一次）。
    #[test]
    fn status_is_cached_once_per_process() {
        let first = backend::narrowphase_backend_status();
        let second = backend::narrowphase_backend_status();
        assert!(std::ptr::eq(first, second));
        assert_eq!(
            device_permitted(BackendChoice::Device),
            first.cuda_ready(),
            "Device 是否被允许必须与缓存状态一致"
        );
    }
}

#[cfg(not(feature = "cuda-probe"))]
mod no_cuda {
    use crate::runtime::backend::{
        BackendChoice, BackendStatus, BackendUnavailable, device_permitted, probe_backend,
    };

    /// 无 `cuda` 特性：选择层恒 Host + `FeatureDisabled`，不 panic、不需要驱动/NVRTC。
    #[test]
    fn feature_disabled_reports_host_only() {
        assert_eq!(
            probe_backend(),
            BackendStatus::HostOnly(BackendUnavailable::FeatureDisabled)
        );
        assert!(!device_permitted(BackendChoice::Device));
    }
}
