//! B.10 / B.11 退路：预分配池与拷贝借用。
//!
//! - B.10：`transient_alloc=false` 时临时分配走**预分配池**；分配路径不触发同步
//!   （本模块不接收队列/流，纯宿主记账；池耗尽显式报 `BufferError::CapacityExceeded`）。
//! - B.11：`zero_copy_external=false` 时借用退化为**拷贝**（所有权独立）；`true` 时零拷贝。
//!
//! 本模块只定义退路语义与选择；真实设备缓冲绑定由各后端在迁移时对接（P2）。

use crate::runtime::kernel_exec::backend::FallbackSet;
use crate::runtime::kernel_exec::buffer::{Address64, Allocator, DType, ExternalHandle, Init};
use crate::runtime::kernel_exec::error::BufferError;

use super::{FallbackCapability, FallbackTrace, select_native};

/// 预分配池中的一个槽位。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PooledSlot {
    /// 槽位编号（稳定，便于断言复用）。
    pub id: u64,
    /// 槽位容量（字节）。
    pub bytes: u64,
}

#[derive(Clone, Copy, Debug)]
struct SlotState {
    slot: PooledSlot,
    in_use: bool,
}

/// B.10 退路载体：预分配池（宿主侧登记）。
#[derive(Debug, Default)]
pub struct TransientPool {
    next_id: u64,
    slots: Vec<SlotState>,
}

impl TransientPool {
    /// 空池。
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            slots: Vec::new(),
        }
    }

    /// 预分配一个槽位（仍为空闲）；返回槽位描述。
    pub fn reserve(&mut self, bytes: u64) -> PooledSlot {
        let slot = PooledSlot {
            id: self.next_id,
            bytes,
        };
        self.next_id += 1;
        self.slots.push(SlotState {
            slot,
            in_use: false,
        });
        slot
    }

    /// 取一个容量足够的空闲槽位；无可用槽位时返回 `None`。
    pub fn take(&mut self, bytes: u64) -> Option<PooledSlot> {
        let state = self
            .slots
            .iter_mut()
            .find(|state| !state.in_use && state.slot.bytes >= bytes)?;
        state.in_use = true;
        Some(state.slot)
    }

    /// 归还槽位。
    pub fn release(&mut self, slot: PooledSlot) {
        if let Some(state) = self.slots.iter_mut().find(|state| state.slot.id == slot.id) {
            state.in_use = false;
        }
    }

    /// 池总容量（字节）。
    pub fn capacity(&self) -> u64 {
        self.slots.iter().map(|state| state.slot.bytes).sum()
    }

    /// 槽位数。
    pub fn slots(&self) -> usize {
        self.slots.len()
    }

    /// 空闲槽位数。
    pub fn free_slots(&self) -> usize {
        self.slots.iter().filter(|state| !state.in_use).count()
    }
}

/// B.10 选择结果。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransientDecision {
    /// 原生路径：一次性分配（无池）。
    Native,
    /// 退路：复用预分配槽位。
    Pooled(PooledSlot),
}

/// B.10 选择：`true` → 原生一次性分配；`false` → 预分配池；池耗尽显式报错。
pub fn transient_alloc(
    fallbacks: &FallbackSet,
    pool: &mut TransientPool,
    bytes: u64,
    trace: &mut FallbackTrace,
) -> Result<TransientDecision, BufferError> {
    if select_native(
        FallbackCapability::TransientAlloc,
        fallbacks,
        "preallocated-pool",
        trace,
    ) {
        return Ok(TransientDecision::Native);
    }
    match pool.take(bytes) {
        Some(slot) => Ok(TransientDecision::Pooled(slot)),
        None => Err(BufferError::CapacityExceeded {
            requested: bytes,
            capacity: pool.capacity(),
        }),
    }
}

/// 一次 B.11 退路拷贝的记录（源地址、字节数、独立所有权缓冲地址）。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExternalCopy {
    /// 外部源地址（opaque）。
    pub source: Address64,
    /// 字节数。
    pub bytes: u64,
    /// 拷贝后的独立缓冲地址。
    pub owned_address: Address64,
}

/// B.11 选择结果。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorrowDecision {
    /// 原生路径：零拷贝借用（地址与原句柄一致）。
    NativeZeroCopy {
        /// 借用的设备地址。
        address: Address64,
    },
    /// 退路：拷贝到独立缓冲。
    Copied(ExternalCopy),
}

/// B.11 选择：`true` → 零拷贝借用；`false` → 拷贝（内容来自 `snapshot`，所有权独立）。
pub fn borrow_external(
    fallbacks: &FallbackSet,
    handle: ExternalHandle,
    bytes: u64,
    snapshot: &[u8],
    allocator: &dyn Allocator,
    trace: &mut FallbackTrace,
) -> Result<BorrowDecision, BufferError> {
    let source = Address64(handle.0);
    if select_native(
        FallbackCapability::ZeroCopyExternal,
        fallbacks,
        "copy",
        trace,
    ) {
        return Ok(BorrowDecision::NativeZeroCopy { address: source });
    }
    let buffer = allocator.alloc(DType::U8, bytes, Init::Zeroed)?;
    buffer.upload(snapshot, 0)?;
    Ok(BorrowDecision::Copied(ExternalCopy {
        source,
        bytes,
        owned_address: buffer.addr(),
    }))
}
