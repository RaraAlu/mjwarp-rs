//! CUDA 运行期库与设备可用性探测（driver / NVRTC / 设备数）。
//!
//! 产品路径只调用真实的 cudarc 探测（`is_culib_present` / `device_count`），不读环境变量、
//! 没有任何运行时开关。测试构建（`#[cfg(test)]`）可经 seam 注入「库缺失 / 设备不可用」状态，
//! 用于覆盖无驱动、有驱动无设备、有设备无 NVRTC 三条回退路径；产品二进制不包含任何 seam 代码。
//!
//! 语义与阶段 1 DESIGN §4 一致：驱动缺失 → ctx `NoDevice` / CLI `DriverUnavailable`；NVRTC 缺失 →
//! ctx `Compile` / CLI `NvrtcUnavailable`；有驱动无设备 → ctx `NoDevice` / CLI `InvalidDevice`。

#[cfg(test)]
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};

#[cfg(test)]
const UNSET: u8 = 0;
#[cfg(test)]
const PRESENT: u8 = 1;
#[cfg(test)]
const ABSENT: u8 = 2;

#[cfg(test)]
use std::sync::{Mutex, MutexGuard};

#[cfg(test)]
static DRIVER_OVERRIDE: AtomicU8 = AtomicU8::new(UNSET);
#[cfg(test)]
static NVRTC_OVERRIDE: AtomicU8 = AtomicU8::new(UNSET);
#[cfg(test)]
static DEVICE_COUNT_OVERRIDE: AtomicI32 = AtomicI32::new(-1);
#[cfg(test)]
static CONTEXT_MODE: AtomicU8 = AtomicU8::new(UNSET);
#[cfg(test)]
static TEST_OVERRIDE_LOCK: Mutex<()> = Mutex::new(());

/// 测试 seam：`CudaContext::new` 走真实路径。
#[cfg(test)]
pub(crate) const CONTEXT_REAL: u8 = 0;
/// 测试 seam：`CudaContext::new` 注入 panic（模拟 cudarc `culib()` 缺库 panic）。
#[cfg(test)]
pub(crate) const CONTEXT_PANIC: u8 = 1;
/// 测试 seam：`CudaContext::new` 注入 Err（模拟有驱动但无可用设备）。
#[cfg(test)]
pub(crate) const CONTEXT_ERROR: u8 = 2;

/// 当前可用 CUDA 设备数（测试可注入；产品直接查 cudarc，不创建 context）。
pub(crate) fn device_count_now() -> Result<usize, cudarc::driver::DriverError> {
    #[cfg(test)]
    if let Some(count) = device_count_override() {
        return Ok(count);
    }
    cudarc::driver::CudaContext::device_count().map(|count| count as usize)
}

/// NVIDIA 驱动库（libcuda/nvcuda）是否可加载。
pub(crate) fn driver_library_present() -> bool {
    #[cfg(test)]
    match DRIVER_OVERRIDE.load(Ordering::SeqCst) {
        PRESENT => return true,
        ABSENT => return false,
        _ => {}
    }
    // SAFETY: 只探测驱动库是否可加载（cudarc 内部 dynamic-loading），不解引用用户指针、
    // 不创建 CUDA context。
    unsafe { cudarc::driver::sys::is_culib_present() }
}

/// NVRTC 编译库是否可加载。
pub(crate) fn nvrtc_library_present() -> bool {
    #[cfg(test)]
    match NVRTC_OVERRIDE.load(Ordering::SeqCst) {
        PRESENT => return true,
        ABSENT => return false,
        _ => {}
    }
    // SAFETY: 只探测 NVRTC 库是否可加载，不调用编译器。
    unsafe { cudarc::nvrtc::sys::is_culib_present() }
}

/// 测试 seam：覆盖 `device_count`（`None` = 走真实 cudarc 查询）。产品构建无此函数。
#[cfg(test)]
pub(crate) fn device_count_override() -> Option<usize> {
    let value = DEVICE_COUNT_OVERRIDE.load(Ordering::SeqCst);
    if value < 0 {
        None
    } else {
        Some(value as usize)
    }
}

/// 测试 seam：当前 `CudaContext::new` 注入模式。产品构建无此函数。
#[cfg(test)]
pub(crate) fn test_context_mode() -> u8 {
    CONTEXT_MODE.load(Ordering::SeqCst)
}

/// 测试 seam：全部测试共享一把锁，避免并行测试互相污染注入状态。
#[cfg(test)]
pub(crate) fn test_override_lock() -> MutexGuard<'static, ()> {
    TEST_OVERRIDE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 测试 seam：注入驱动库存在性。
#[cfg(test)]
pub(crate) fn set_driver_present_for_tests(present: bool) {
    DRIVER_OVERRIDE.store(if present { PRESENT } else { ABSENT }, Ordering::SeqCst);
}

/// 测试 seam：注入 NVRTC 库存在性。
#[cfg(test)]
pub(crate) fn set_nvrtc_present_for_tests(present: bool) {
    NVRTC_OVERRIDE.store(if present { PRESENT } else { ABSENT }, Ordering::SeqCst);
}

/// 测试 seam：注入设备数（`None` = 真实查询）。
#[cfg(test)]
pub(crate) fn set_device_count_for_tests(count: Option<usize>) {
    DEVICE_COUNT_OVERRIDE.store(
        match count {
            Some(value) => value as i32,
            None => -1,
        },
        Ordering::SeqCst,
    );
}

/// 测试 seam：注入 `CudaContext::new` 行为（real / panic / error）。
#[cfg(test)]
pub(crate) fn set_context_mode_for_tests(mode: u8) {
    CONTEXT_MODE.store(mode, Ordering::SeqCst);
}

/// 测试 seam：清空全部注入状态。
#[cfg(test)]
pub(crate) fn reset_test_overrides() {
    DRIVER_OVERRIDE.store(UNSET, Ordering::SeqCst);
    NVRTC_OVERRIDE.store(UNSET, Ordering::SeqCst);
    DEVICE_COUNT_OVERRIDE.store(-1, Ordering::SeqCst);
    CONTEXT_MODE.store(CONTEXT_REAL, Ordering::SeqCst);
}

/// 测试 RAII：持锁并在 drop 时清空注入状态（断言失败也不污染后续测试）。
#[cfg(test)]
pub(crate) struct SeamGuard {
    _lock: MutexGuard<'static, ()>,
}

#[cfg(test)]
impl SeamGuard {
    pub(crate) fn lock() -> Self {
        Self {
            _lock: test_override_lock(),
        }
    }
}

#[cfg(test)]
impl Drop for SeamGuard {
    fn drop(&mut self) {
        reset_test_overrides();
    }
}
