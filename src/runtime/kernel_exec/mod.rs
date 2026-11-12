//! 设备无关的内核执行层接口（P1-a 骨架 + P1-b CUDA 适配 + P1-c 退路；**不接调用点**）。
//!
//! 依据：`docs/architecture/device-layer-contract.md`（A0 定稿 §2）。
//! 本模块定义 trait、值类型与（`cfg(test)`）mock backend；**不接现有调用点**：
//! 既有 `runtime` 模块的路径、recipe、PTX/ABI 与行为保持不变。
//! `cuda` 子模块（P1-b）只在 `cuda-probe` 特性下编译，是宿主面 trait 的 CUDA 适配层；
//! `fallback` 子模块（P1-c）是 12 个 fallback flag 的后端无关退路实现与双路测试载体；
//! 旧 cudarc/NVRTC/launcher 路径保持原样（双轨），迁移属 P2。
//!
//! 设计取舍（P1-a）：
//! - 消费方可见的句柄面（[`Backend`]/[`Device`]/[`Allocator`]/[`Buffer`]/[`Queue`]/[`Module`]/
//!   [`Diagnostics`]）保持对象安全，可用 `&dyn`/`Box<dyn>` 传递，换后端不改消费方；
//! - 设备内联原语（[`DeviceMath`]/[`Atomics`]/[`ThreadScope`]）与带闭包的 [`ExecControl`]
//!   不追求 `dyn` 兼容：它们按单态化落到具体后端内核，属于书写面而非宿主调度面；
//! - 缓冲容量/越界错误用 [`BufferError`] 显式表达，不混入 [`DeviceError`] 的硬性三分类。

pub mod backend;
pub mod buffer;
#[cfg(feature = "cuda-probe")]
pub mod cuda;
pub mod device;
pub mod error;
pub mod fallback;
pub mod module;
pub mod queue;

pub use backend::{
    Backend, CapabilityId, Device, DeviceCaps, DeviceId, FallbackSet, FmaMode, NumericPolicy,
};
pub use buffer::{Address64, Allocator, Buffer, DType, ExternalHandle, Init, Pinned};
pub use device::{
    Atomics, Counter, DeviceMath, Diagnostics, Dim3, Duration, Event, ExecControl, SharedBuf,
    ThreadScope,
};
pub use error::{BufferError, CompileError, DeviceError, DeviceLost, ErrorClass, LaunchError};
pub use module::{BlockSuggestion, CompileOptions, Kernel, KernelSource, Module};
pub use queue::{Alias, Arg, Graph, IndexSpace, Launch, Queue, QueueCapture, Submission};

#[cfg(test)]
mod mock_backend;
#[cfg(test)]
mod mock_module;
#[cfg(test)]
mod mock_tests;
