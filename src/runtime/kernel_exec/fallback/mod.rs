//! 退路实现（P1-c）：12 个 fallback flag 的退路语义、降级留痕与双路测试载体。
//!
//! 口径（P1-b 冻结，A0 §2）：[`FallbackSet`] 12 flag，`true` = 该能力有原生实现（走原生），
//! `false` = 走退路（与直觉相反）。本模块只提供**后端无关**的退路实现与选择逻辑：
//!
//! - 10 个 `now` 项实现退路：B.10/B.11/G.9/K.1/K.2/K.3/K.5/L.3/C.1/H.1；
//! - 2 个 `after_shim` 项（D.4 泛型设备函数、D.10 注入原生片段）依赖 P2 书写面，
//!   本模块只登记编号/语义/触发点与 `impl_scope`，**不实现、不出双路结论**。
//!
//! 硬规则：
//! - 12 个退项的任何组合都**不得**返回 `DeviceError::Unsupported`（退路存在即必须可走）；
//! - 降级必须写入 [`FallbackTrace`]（能力编号 + 退路名），不得静默改语义；
//! - 退路实现只做宿主侧登记/记账；`upload/transient` 语义要求"不隐式同步"，
//!   本模块全部 API 不接收队列/流、不调用 `wait_idle`。

mod buffers;
mod control;
mod graph;
mod numeric;
mod printf;
#[cfg(test)]
mod tests;

use crate::runtime::kernel_exec::backend::FallbackSet;

pub use buffers::{
    BorrowDecision, ExternalCopy, PooledSlot, TransientDecision, TransientPool, borrow_external,
    transient_alloc,
};
pub use control::{
    DEFAULT_FIXED_ITERATIONS, branch_on_device_fallback, loop_while_device_fallback,
};
pub use graph::{
    CapturePool, RecordedGraph, RecordingQueue, capture_sequential, preallocate_capture,
    replay_sequential,
};
pub use numeric::{
    F32AtomicCell, Int64Halves, Int64Mode, atomic_add_f32, int64_mode, join_i64, split_i64,
};
pub use printf::{PrintfStatusBuffer, format_record, printf_fallback};

/// 12 个 fallback 能力的实现范围（P1-c 冻结：10 个 now，2 个 after_shim）。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImplScope {
    /// 本片实现退路 + 双路测试。
    Now,
    /// 依赖 P2 书写面 shim：本片只登记语义与触发点，不实现。
    AfterShim,
}

/// 12 个 fallback 能力编号（与 [`FallbackSet`] 字段一一对应）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FallbackCapability {
    /// B.10 临时分配不触发同步（退：预分配池）。
    TransientAlloc,
    /// B.11 零拷贝借用外部框架（退：拷贝）。
    ZeroCopyExternal,
    /// D.4 泛型设备函数（`after_shim`）。
    GenericDeviceFn,
    /// G.9 建议块宽/最小驻留块数（退：固定值）。
    OccupancyHint,
    /// K.1 录制重放（退：逐次提交）。
    GraphCapture,
    /// K.2 设备端条件循环（退：固定次数）。
    DeviceLoop,
    /// K.3 设备端条件分支（退：无条件执行）。
    DeviceBranch,
    /// K.5 录制期分配（退：录制前预分配）。
    CaptureAlloc,
    /// L.3 内核内格式化打印（退：状态缓冲 + 宿主打印）。
    KernelPrintf,
    /// D.10 注入设备原生片段（`after_shim`）。
    NativeSourceInjection,
    /// C.1 64 位整数（必带退：双 32 位表示）。
    Int64,
    /// H.1 f32 原子加（必带退：比较交换循环）。
    F32Atomic,
}

impl FallbackCapability {
    /// 12 项全集（顺序与 A0 §2 名单一致：先 10 退项，后 2 必带退）。
    pub const ALL: [Self; 12] = [
        Self::TransientAlloc,
        Self::ZeroCopyExternal,
        Self::GenericDeviceFn,
        Self::NativeSourceInjection,
        Self::OccupancyHint,
        Self::GraphCapture,
        Self::DeviceLoop,
        Self::DeviceBranch,
        Self::CaptureAlloc,
        Self::KernelPrintf,
        Self::Int64,
        Self::F32Atomic,
    ];

    /// 能力编号（A0 §3）。
    pub const fn id(self) -> &'static str {
        match self {
            Self::TransientAlloc => "B.10",
            Self::ZeroCopyExternal => "B.11",
            Self::GenericDeviceFn => "D.4",
            Self::NativeSourceInjection => "D.10",
            Self::OccupancyHint => "G.9",
            Self::GraphCapture => "K.1",
            Self::DeviceLoop => "K.2",
            Self::DeviceBranch => "K.3",
            Self::CaptureAlloc => "K.5",
            Self::KernelPrintf => "L.3",
            Self::Int64 => "C.1",
            Self::F32Atomic => "H.1",
        }
    }

    /// `FallbackSet` 字段名（P1-b 定稿名）。
    pub const fn flag_name(self) -> &'static str {
        match self {
            Self::TransientAlloc => "transient_alloc",
            Self::ZeroCopyExternal => "zero_copy_external",
            Self::GenericDeviceFn => "generic_device_fn",
            Self::NativeSourceInjection => "native_source_injection",
            Self::OccupancyHint => "occupancy_hint",
            Self::GraphCapture => "graph_capture",
            Self::DeviceLoop => "device_loop",
            Self::DeviceBranch => "device_branch",
            Self::CaptureAlloc => "capture_alloc",
            Self::KernelPrintf => "kernel_printf",
            Self::Int64 => "int64",
            Self::F32Atomic => "f32_atomic",
        }
    }

    /// P1-c 实现范围（冻结）。
    pub const fn impl_scope(self) -> ImplScope {
        match self {
            Self::GenericDeviceFn | Self::NativeSourceInjection => ImplScope::AfterShim,
            _ => ImplScope::Now,
        }
    }

    /// 该能力是否有原生实现（`true`）/ 走退路（`false`）。
    pub const fn is_native(self, fallbacks: &FallbackSet) -> bool {
        match self {
            Self::TransientAlloc => fallbacks.transient_alloc,
            Self::ZeroCopyExternal => fallbacks.zero_copy_external,
            Self::GenericDeviceFn => fallbacks.generic_device_fn,
            Self::NativeSourceInjection => fallbacks.native_source_injection,
            Self::OccupancyHint => fallbacks.occupancy_hint,
            Self::GraphCapture => fallbacks.graph_capture,
            Self::DeviceLoop => fallbacks.device_loop,
            Self::DeviceBranch => fallbacks.device_branch,
            Self::CaptureAlloc => fallbacks.capture_alloc,
            Self::KernelPrintf => fallbacks.kernel_printf,
            Self::Int64 => fallbacks.int64,
            Self::F32Atomic => fallbacks.f32_atomic,
        }
    }

    /// 是否需要走退路（`false` 极性取反）。
    pub const fn requires_fallback(self, fallbacks: &FallbackSet) -> bool {
        !self.is_native(fallbacks)
    }
}

/// 一次降级记录：能力编号 + 走的退路名。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Degraded {
    /// 能力编号。
    pub capability: FallbackCapability,
    /// 退路名（短 ASCII，便于日志/断言）。
    pub path: &'static str,
}

/// 降级轨迹（不静默：每次走退路都记一条）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FallbackTrace {
    entries: Vec<Degraded>,
}

impl FallbackTrace {
    /// 空轨迹。
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 记录一次降级。
    pub fn record(&mut self, capability: FallbackCapability, path: &'static str) {
        self.entries.push(Degraded { capability, path });
    }

    /// 全部记录。
    pub fn entries(&self) -> &[Degraded] {
        &self.entries
    }

    /// 记录数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 某能力的降级次数。
    pub fn count(&self, capability: FallbackCapability) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.capability == capability)
            .count()
    }
}

/// 选择原生/退路：走退路时留痕；返回 `true` 表示走原生路径。
///
/// 12 个退项都有退路，因此本函数**不会**返回错误（也不产生 `Unsupported`）。
pub fn select_native(
    capability: FallbackCapability,
    fallbacks: &FallbackSet,
    path: &'static str,
    trace: &mut FallbackTrace,
) -> bool {
    let native = capability.is_native(fallbacks);
    if !native {
        trace.record(capability, path);
    }
    native
}
