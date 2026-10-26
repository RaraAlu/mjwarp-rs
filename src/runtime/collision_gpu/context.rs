//! 产品 GPU 入口的 `CudaContext` 构造：三态回退、可诊断错误、不 panic。
//!
//! 语义与阶段 1 DESIGN §4 一致：驱动库缺失 → `NoDevice`（预检，避免 cudarc 缺库时 `culib()`
//! panic）；`CudaContext::new` 返回错误（有驱动无可用设备）→ `NoDevice`；`CudaContext::new`
//! 内部 panic（cudarc 适配层）→ `catch_unwind` 捕获后 `NoDevice`。

use std::sync::Arc;

use cudarc::driver::CudaContext;

use super::CollisionGpuError;
use crate::runtime::presence;

/// 构造 `CudaContext` 并按三态回退返回可诊断错误（**不 panic**）。
pub(crate) fn device_context(device: usize) -> Result<Arc<CudaContext>, CollisionGpuError> {
    if !presence::driver_library_present() {
        return Err(CollisionGpuError::NoDevice(
            "CUDA驱动库缺失（libcuda/nvcuda 不可加载）".to_owned(),
        ));
    }
    let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // 测试 seam：注入 panic / 设备不可用，覆盖「culib 初始化 panic」与「有驱动无设备」路径。
        // 产品构建不包含该注入（`#[cfg(test)]`）。
        #[cfg(test)]
        match presence::test_context_mode() {
            presence::CONTEXT_PANIC => panic!("注入的 cudarc culib panic"),
            presence::CONTEXT_ERROR => {
                return Err(format!(
                    "CudaContext::new({device}): 注入的设备不可用（no device）"
                ));
            }
            _ => {}
        }
        CudaContext::new(device).map_err(|error| format!("CudaContext::new({device}): {error:?}"))
    }));
    match attempt {
        Ok(Ok(context)) => Ok(context),
        Ok(Err(detail)) => Err(CollisionGpuError::NoDevice(detail)),
        Err(_) => Err(CollisionGpuError::NoDevice(format!(
            "CudaContext::new({device}) 初始化 panic（cudarc culib）已捕获"
        ))),
    }
}
