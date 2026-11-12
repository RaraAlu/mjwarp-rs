//! CUDA 侧退路计划（P1-c）：纯函数选择"原生/退路"，供适配层分支与 0 GPU 双路测试。
//!
//! 口径：`FallbackSet` 极性 `true`=原生可用 / `false`=走退路。本模块冻结**选择哪条路**；
//! 设备端执行（K.2/K.3/C.1/H.1 的 lowering 与原子/控制流指令）属 P2 emitter，P1-c 只交付
//! 选择逻辑与退化留痕。10 个 now 项各有对应函数；D.4/D.10 为 `after_shim`，不在本模块。
//!
//! 所有函数都不排队列、不等待：选择本身不构成隐式同步。

use crate::runtime::kernel_exec::backend::FallbackSet;
#[cfg(test)]
use crate::runtime::kernel_exec::fallback::Int64Mode;
use crate::runtime::kernel_exec::fallback::{FallbackCapability, FallbackTrace, select_native};

/// B.10 临时分配模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TransientMode {
    /// 原生：一次性异步分配。
    NativeAsync,
    /// 退路：复用预分配池。
    PreallocatedPool,
}

/// B.11 外部借用模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExternalBorrowMode {
    /// 原生：零拷贝借用。
    NativeZeroCopy,
    /// 退路：拷贝到独立所有权缓冲。
    Copy,
}

/// G.9 建议块宽模式。
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OccupancyMode {
    /// 原生：driver occupancy 查询。
    NativeQuery,
    /// 退路：固定值。
    FixedValue,
}

/// K.1 图模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphMode {
    /// 原生：图捕获/重放。
    NativeCapture,
    /// 退路：逐次提交（录制-重放）。
    SequentialReplay,
}

/// K.2 设备端循环模式。
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoopMode {
    /// 原生：设备端条件循环。
    NativeConditional,
    /// 退路：固定次数。
    FixedIterations,
}

/// K.3 设备端分支模式。
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BranchMode {
    /// 原生：设备端条件分支。
    NativeConditional,
    /// 退路：两分支都执行（then → else，P1-c 冻结）。
    UnconditionalBoth,
}

/// K.5 录制期分配模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaptureAllocMode {
    /// 原生：录制期分配。
    NativeDuringCapture,
    /// 退路：录制前预分配。
    Preallocated,
}

/// L.3 打印模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrintfMode {
    /// 原生：设备 printf。
    NativeDevicePrintf,
    /// 退路：状态缓冲 + 宿主打印。
    StatusBuffer,
}

/// H.1 f32 原子加模式。
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum F32AtomicMode {
    /// 原生：设备 `atomicAdd(float*)`。
    Native,
    /// 退路：比较交换循环。
    CasLoop,
}

macro_rules! choose {
    ($capability:expr, $fallbacks:expr, $path:expr, $trace:expr, $native:expr, $fallback:expr) => {
        if select_native($capability, $fallbacks, $path, $trace) {
            $native
        } else {
            $fallback
        }
    };
}

/// B.10 选择。
pub(crate) fn transient_mode(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> TransientMode {
    choose!(
        FallbackCapability::TransientAlloc,
        fallbacks,
        "preallocated-pool",
        trace,
        TransientMode::NativeAsync,
        TransientMode::PreallocatedPool
    )
}

/// B.11 选择。
pub(crate) fn external_borrow_mode(
    fallbacks: &FallbackSet,
    trace: &mut FallbackTrace,
) -> ExternalBorrowMode {
    choose!(
        FallbackCapability::ZeroCopyExternal,
        fallbacks,
        "copy",
        trace,
        ExternalBorrowMode::NativeZeroCopy,
        ExternalBorrowMode::Copy
    )
}

/// G.9 选择。
#[cfg(test)]
pub(crate) fn occupancy_mode(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> OccupancyMode {
    choose!(
        FallbackCapability::OccupancyHint,
        fallbacks,
        "fixed-value",
        trace,
        OccupancyMode::NativeQuery,
        OccupancyMode::FixedValue
    )
}

/// K.1 选择。
pub(crate) fn graph_mode(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> GraphMode {
    choose!(
        FallbackCapability::GraphCapture,
        fallbacks,
        "sequential-replay",
        trace,
        GraphMode::NativeCapture,
        GraphMode::SequentialReplay
    )
}

/// K.2 选择。
#[cfg(test)]
pub(crate) fn loop_mode(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> LoopMode {
    choose!(
        FallbackCapability::DeviceLoop,
        fallbacks,
        "fixed-iterations",
        trace,
        LoopMode::NativeConditional,
        LoopMode::FixedIterations
    )
}

/// K.3 选择。
#[cfg(test)]
pub(crate) fn branch_mode(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> BranchMode {
    choose!(
        FallbackCapability::DeviceBranch,
        fallbacks,
        "unconditional-both-branches",
        trace,
        BranchMode::NativeConditional,
        BranchMode::UnconditionalBoth
    )
}

/// K.5 选择。
pub(crate) fn capture_alloc_mode(
    fallbacks: &FallbackSet,
    trace: &mut FallbackTrace,
) -> CaptureAllocMode {
    choose!(
        FallbackCapability::CaptureAlloc,
        fallbacks,
        "preallocated-before-capture",
        trace,
        CaptureAllocMode::NativeDuringCapture,
        CaptureAllocMode::Preallocated
    )
}

/// L.3 选择。
pub(crate) fn printf_mode(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> PrintfMode {
    choose!(
        FallbackCapability::KernelPrintf,
        fallbacks,
        "status-buffer",
        trace,
        PrintfMode::NativeDevicePrintf,
        PrintfMode::StatusBuffer
    )
}

/// C.1 选择（设备端 lowering 在 P2；此处冻结选择）。
#[cfg(test)]
pub(crate) fn int64_lowering(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> Int64Mode {
    crate::runtime::kernel_exec::fallback::int64_mode(fallbacks, trace)
}

/// H.1 选择（设备端 lowering 在 P2；此处冻结选择）。
#[cfg(test)]
pub(crate) fn f32_atomic_mode(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> F32AtomicMode {
    choose!(
        FallbackCapability::F32Atomic,
        fallbacks,
        "cas-loop",
        trace,
        F32AtomicMode::Native,
        F32AtomicMode::CasLoop
    )
}
