//! CUDA 适配层的错误映射（A.4 三分类不合并；能力缺失显式 `Unsupported`）。
//!
//! 映射约定（P1-b 冻结）：
//! - 驱动库缺失 / 上下文创建失败 / 拷贝或同步的驱动失败 → [`DeviceError::DeviceLost`]；
//! - NVRTC 缺失 / 编译失败 / F.6 PTX 精度闸失败 → [`DeviceError::Compile`]；
//! - 发射配置/参数/描述符不符 → [`DeviceError::Launch`]；
//! - 未接线能力或该后端不支持的形状 → [`DeviceError::Unsupported`]；
//! - 缓冲面的参数错误走 [`BufferError`]，缓冲面的驱动失败走
//!   [`BufferError::DeviceLost`]（D1 扩展，避免三分类在缓冲面被吞）。

use cudarc::driver::DriverError;

use crate::runtime::kernel_exec::backend::CapabilityId;
use crate::runtime::kernel_exec::error::{
    BufferError, CompileError, DeviceError, DeviceLost, LaunchError,
};

/// 驱动/上下文/资源错误 → `DeviceLost`。
pub(crate) fn device_lost(stage: &'static str, detail: impl std::fmt::Debug) -> DeviceError {
    DeviceError::DeviceLost(DeviceLost::new(format!("{stage}: {detail:?}")))
}

/// 发射阶段驱动错误 → `Launch`（配置/参数不符，不是设备丢失）。
pub(crate) fn launch_driver(stage: &'static str, error: DriverError) -> DeviceError {
    DeviceError::Launch(LaunchError::InvalidConfig(format!("{stage}: {error:?}")))
}

/// 编译失败（消息已包含阶段）。
pub(crate) fn compile(message: impl Into<String>) -> DeviceError {
    DeviceError::Compile(CompileError::new(message))
}

/// 能力缺失且无退路。
pub(crate) fn unsupported(capability: &'static str) -> DeviceError {
    DeviceError::Unsupported(CapabilityId(capability))
}

/// 缓冲面能力缺失。
pub(crate) fn buffer_unsupported(capability: &'static str) -> BufferError {
    BufferError::Unsupported(CapabilityId(capability))
}

/// 异域/异上下文缓冲。
pub(crate) fn foreign_buffer() -> BufferError {
    BufferError::ForeignBuffer { backend: "cuda" }
}

/// 缓冲面驱动失败。
pub(crate) fn buffer_device_lost(stage: &'static str, detail: impl std::fmt::Debug) -> BufferError {
    BufferError::DeviceLost(DeviceLost::new(format!("{stage}: {detail:?}")))
}

/// F.6 PTX 精度闸失败（`ProbeError`）→ `Compile`。
pub(crate) fn from_probe(error: crate::diagnostics::ProbeError) -> DeviceError {
    compile(format!("{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::kernel_exec::error::ErrorClass;

    #[test]
    fn error_classes_are_mapped_without_merging() {
        let lost = device_lost("cuda-copy", "xid 79");
        assert_eq!(lost.class(), Some(ErrorClass::DeviceLost));
        let compiled = compile("nvrtc: bad source");
        assert_eq!(compiled.class(), Some(ErrorClass::Compile));
        let configured = DeviceError::Launch(LaunchError::InvalidConfig("blocks=0".into()));
        assert_eq!(configured.class(), Some(ErrorClass::Launch));
        let missing = unsupported("B.11");
        assert!(missing.is_unsupported());
        assert_eq!(missing.class(), None);
    }

    #[test]
    fn buffer_side_errors_are_explicit() {
        assert_eq!(
            buffer_unsupported("B.11"),
            BufferError::Unsupported(CapabilityId("B.11"))
        );
        assert_eq!(
            foreign_buffer(),
            BufferError::ForeignBuffer { backend: "cuda" }
        );
        assert!(matches!(
            buffer_device_lost("cuda-buffer", "copy failed"),
            BufferError::DeviceLost(_)
        ));
    }
}
