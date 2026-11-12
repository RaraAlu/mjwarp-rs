//! C.1 / H.1 必带退：64 位整数双 32 位表示、f32 原子加 CAS 循环。
//!
//! 这两项是「必级能力 + 常备退法」：`int64`/`f32_atomic` 为 `true` 表示后端原生可用；
//! 为 `false` 时分别走**高/低 32 位拆分**与**比较交换循环**。本模块提供退法语义的宿主
//! 参考实现（设备侧 lowering 由 P2 书写面/emitter 使用），全部为纯计算，不接触队列/流，
//! 因此不产生隐式同步。

use std::sync::atomic::{AtomicU32, Ordering};

use crate::runtime::kernel_exec::backend::FallbackSet;

use super::{FallbackCapability, FallbackTrace, select_native};

/// C.1 退路表示：64 位整数拆成高/低 32 位。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Int64Halves {
    /// 高 32 位（含符号）。
    pub high: i32,
    /// 低 32 位（无符号位模式）。
    pub low: u32,
}

/// 拆成高/低 32 位。
pub const fn split_i64(value: i64) -> Int64Halves {
    Int64Halves {
        high: (value >> 32) as i32,
        low: value as u32,
    }
}

/// 由高/低 32 位合回 64 位（按位模式，无精度损失）。
pub const fn join_i64(halves: Int64Halves) -> i64 {
    ((halves.high as i64) << 32) | (halves.low as i64)
}

/// C.1 选择结果。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Int64Mode {
    /// 原生 64 位整数。
    Native,
    /// 退路：双 32 位表示。
    TwoI32,
}

/// C.1 选择：`true` → 原生；`false` → 双 32 位（降级留痕）。
pub fn int64_mode(fallbacks: &FallbackSet, trace: &mut FallbackTrace) -> Int64Mode {
    if select_native(FallbackCapability::Int64, fallbacks, "two-i32", trace) {
        Int64Mode::Native
    } else {
        Int64Mode::TwoI32
    }
}

/// H.1 退路载体：以位模式存放 f32 的原子单元。
#[derive(Debug)]
pub struct F32AtomicCell {
    bits: AtomicU32,
}

impl F32AtomicCell {
    /// 以初值构造。
    pub fn new(value: f32) -> Self {
        Self {
            bits: AtomicU32::new(value.to_bits()),
        }
    }

    /// 当前值。
    pub fn load(&self) -> f32 {
        f32::from_bits(self.bits.load(Ordering::SeqCst))
    }

    /// 原生 f32 原子加的**参考语义**：返回旧值，写入 old+delta。
    ///
    /// 参考实现用单次比较交换完成（设备侧是原生 `atomicAdd(float*)`；语义一致：返回旧值）。
    pub fn native_add(&self, delta: f32) -> f32 {
        self.compare_exchange_add(delta, false)
    }

    /// 退路：比较交换循环（CAS loop），返回旧值。
    pub fn cas_loop_add(&self, delta: f32) -> f32 {
        self.compare_exchange_add(delta, true)
    }

    fn compare_exchange_add(&self, delta: f32, retry_loop: bool) -> f32 {
        let mut current = self.bits.load(Ordering::SeqCst);
        loop {
            let old = f32::from_bits(current);
            let new = (old + delta).to_bits();
            match self
                .bits
                .compare_exchange_weak(current, new, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return old,
                Err(observed) => {
                    current = observed;
                    if !retry_loop {
                        // 原生路径参考实现同样需要重试；`retry_loop` 仅用于文档区分两条入口。
                        continue;
                    }
                }
            }
        }
    }
}

/// H.1 选择：`true` → 原生 f32 原子加；`false` → CAS 循环（降级留痕）。
pub fn atomic_add_f32(
    fallbacks: &FallbackSet,
    cell: &F32AtomicCell,
    delta: f32,
    trace: &mut FallbackTrace,
) -> f32 {
    if select_native(FallbackCapability::F32Atomic, fallbacks, "cas-loop", trace) {
        cell.native_add(delta)
    } else {
        cell.cas_loop_add(delta)
    }
}
