//! 设备无关的内核执行层接口（P1-a 骨架；**只声明，不接线**）。
//!
//! 依据：`docs/architecture/device-layer-contract.md`（A0 草案 §2）。
//! 本模块只定义 trait、值类型与（`cfg(test)`）mock backend；**不接现有调用点**：
//! 既有 `runtime` 模块的路径、recipe、PTX/ABI 与行为保持不变，CUDA 适配在 P1-b 才接线。
//!
//! 设计取舍（P1-a）：
//! - 消费方可见的句柄面（[`Backend`]/[`Device`]/[`Allocator`]/[`Buffer`]/[`Queue`]/[`Module`]/
//!   [`Diagnostics`]）保持对象安全，可用 `&dyn`/`Box<dyn>` 传递，换后端不改消费方；
//! - 设备内联原语（[`DeviceMath`]/[`Atomics`]/[`ThreadScope`]）与带闭包的 [`ExecControl`]
//!   不追求 `dyn` 兼容：它们按单态化落到具体后端内核，属于书写面而非宿主调度面；
//! - 本层不使用 `unsafe`：原子/线程原语只用 opaque 地址与描述符表达，具体指针语义由适配层承担；
//! - 缓冲容量/越界错误用 [`BufferError`] 显式表达，不混入 [`DeviceError`] 的硬性三分类。

pub mod backend;
pub mod buffer;
pub mod device;
pub mod error;
pub mod module;
pub mod queue;

pub use backend::{
    Backend, CapabilityId, Device, DeviceCaps, DeviceId, FallbackSet, FmaMode, NumericPolicy,
};
pub use buffer::{Address64, Allocator, Buffer, DType, ExternalHandle, Init, Pinned};
pub use device::{
    Atomics, Counter, DeviceMath, Diagnostics, Dim3, Duration, Event, ExecControl, ThreadScope,
};
pub use error::{BufferError, CompileError, DeviceError, DeviceLost, ErrorClass, LaunchError};
pub use module::{CompileOptions, Kernel, KernelSource, Module};
pub use queue::{Alias, Arg, Graph, IndexSpace, Launch, Queue, QueueCapture, Submission};

#[cfg(test)]
mod mock_backend;
#[cfg(test)]
mod mock_tests;
