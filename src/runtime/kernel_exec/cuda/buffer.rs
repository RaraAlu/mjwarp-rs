//! CUDA 缓冲与分配（B 域，P1-b）。
//!
//! 适配层自持 `CudaSlice<T>`（不复用 `TransferBuffer<T>`：其存储是 `pub(super)`、元素类型
//! 被 sealed 到 4 种，无法支撑 14 种 `DType`，也无法把 `&mut CudaSlice<T>` 交给
//! `launch_builder`）。旧 `runtime::transfer` 路径一行不动（双轨，迁移属 P2）。
//!
//! 语义：
//! - 字节偏移一律 64 位（B.3）；越界/未对齐显式 [`BufferError`]；
//! - `upload/readback` 按小端位模式解释宿主字节（本机与目标平台均为小端）；
//! - `Init::Uninit` 在 P1-b 用清零实现（内容是"未定义"的合法取值，且不引入 `unsafe` 的
//!   `alloc`；宿主路径不等待，A0 设计约束 2 不受影响）；
//! - 同缓冲拷贝只允许零长度/等范围 no-op；重叠自拷贝与跨设备拷贝显式失败。

use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use cudarc::driver::{CudaSlice, CudaStream, DevicePtr, DeviceRepr};

use crate::runtime::kernel_exec::buffer::{
    Address64, Allocator, Buffer, DType, Init, Pinned, check_range,
};
use crate::runtime::kernel_exec::error::BufferError;

use super::buffer_codec::{copy_typed, readback_typed, upload_typed};
use super::errors;
use super::fallback_plan::{ExternalBorrowMode, TransientMode};
use super::{CudaDeviceInner, lock};

/// `DType` → CUDA 存储类别（C.1 标量 7 种；复合 7 种 P1-b 显式 `Unsupported`）。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageKind {
    /// 1 字节（U8/Bool）。
    Bytes,
    /// f32。
    F32,
    /// i32。
    I32,
    /// u32。
    U32,
    /// i64。
    I64,
    /// u64。
    U64,
}

/// 类型 → 存储类别；不支持的类型返回显式能力缺失。
pub(crate) fn storage_kind(dtype: DType) -> Result<StorageKind, BufferError> {
    match dtype {
        DType::U8 | DType::Bool => Ok(StorageKind::Bytes),
        DType::F32 => Ok(StorageKind::F32),
        DType::I32 => Ok(StorageKind::I32),
        DType::U32 => Ok(StorageKind::U32),
        DType::I64 => Ok(StorageKind::I64),
        DType::U64 => Ok(StorageKind::U64),
        DType::Vec2
        | DType::Vec3
        | DType::Vec4
        | DType::Vec6
        | DType::Vec10
        | DType::Mat33
        | DType::Quat => Err(errors::buffer_unsupported("C.4 composite-dtype")),
    }
}

/// 字节范围 → 元素范围（越界/未对齐显式报错）。
pub(crate) fn elem_range(
    offset: u64,
    len: u64,
    elem_bytes: u64,
    size_bytes: u64,
) -> Result<Range<usize>, BufferError> {
    check_range(offset, len, size_bytes)?;
    if elem_bytes == 0 || !offset.is_multiple_of(elem_bytes) || !len.is_multiple_of(elem_bytes) {
        return Err(BufferError::Unaligned {
            offset,
            alignment: elem_bytes.max(1),
        });
    }
    let start = usize::try_from(offset / elem_bytes).map_err(|_| BufferError::OutOfRange {
        offset,
        len,
        byte_len: size_bytes,
    })?;
    let count = usize::try_from(len / elem_bytes).map_err(|_| BufferError::OutOfRange {
        offset,
        len,
        byte_len: size_bytes,
    })?;
    Ok(start..start + count)
}

/// 位模式填充辅助（`Init::Fill`，按元素宽度截断）。
trait FromPattern {
    fn from_pattern(pattern: u64) -> Self;
}

impl FromPattern for u8 {
    fn from_pattern(pattern: u64) -> Self {
        pattern as Self
    }
}
impl FromPattern for f32 {
    fn from_pattern(pattern: u64) -> Self {
        Self::from_bits(pattern as u32)
    }
}
impl FromPattern for i32 {
    fn from_pattern(pattern: u64) -> Self {
        pattern as u32 as Self
    }
}
impl FromPattern for u32 {
    fn from_pattern(pattern: u64) -> Self {
        pattern as Self
    }
}
impl FromPattern for i64 {
    fn from_pattern(pattern: u64) -> Self {
        pattern as Self
    }
}
impl FromPattern for u64 {
    fn from_pattern(pattern: u64) -> Self {
        pattern
    }
}

fn device_ptr_of<T: DeviceRepr>(slice: &CudaSlice<T>, stream: &Arc<CudaStream>) -> Address64 {
    let (address, read) = slice.device_ptr(stream);
    drop(read);
    Address64(address)
}
/// 设备缓冲存储（P1-b 只用 C.1 标量 7 种）。
pub(crate) enum CudaStorage {
    /// 1 字节。
    Bytes(CudaSlice<u8>),
    /// f32。
    F32(CudaSlice<f32>),
    /// i32。
    I32(CudaSlice<i32>),
    /// u32。
    U32(CudaSlice<u32>),
    /// i64。
    I64(CudaSlice<i64>),
    /// u64。
    U64(CudaSlice<u64>),
}

impl CudaStorage {
    fn addr(&self, stream: &Arc<CudaStream>) -> Address64 {
        match self {
            Self::Bytes(slice) => device_ptr_of(slice, stream),
            Self::F32(slice) => device_ptr_of(slice, stream),
            Self::I32(slice) => device_ptr_of(slice, stream),
            Self::U32(slice) => device_ptr_of(slice, stream),
            Self::I64(slice) => device_ptr_of(slice, stream),
            Self::U64(slice) => device_ptr_of(slice, stream),
        }
    }
}

/// 缓冲注册表（地址 → 弱引用；`Weak` 防循环，Drop 时注销）。
#[derive(Default)]
pub(crate) struct BufferRegistry {
    entries: Mutex<HashMap<u64, Weak<CudaBufferInner>>>,
}

impl BufferRegistry {
    pub(crate) fn insert(&self, inner: &Arc<CudaBufferInner>) {
        lock(&self.entries).insert(inner.addr.0, Arc::downgrade(inner));
    }

    pub(crate) fn lookup(&self, addr: Address64) -> Option<Arc<CudaBufferInner>> {
        lock(&self.entries).get(&addr.0)?.upgrade()
    }

    pub(crate) fn remove(&self, addr: Address64) {
        lock(&self.entries).remove(&addr.0);
    }
}

/// 缓冲内部状态（句柄与注册表共享）。
pub(crate) struct CudaBufferInner {
    pub(crate) dev: Arc<CudaDeviceInner>,
    pub(crate) storage: Mutex<CudaStorage>,
    pub(crate) dtype: DType,
    pub(crate) bytes: u64,
    pub(crate) addr: Address64,
}

impl CudaBufferInner {
    fn lock_storage(&self) -> MutexGuard<'_, CudaStorage> {
        lock(&self.storage)
    }
}

/// 设备缓冲句柄（`Drop` 时注销注册表项）。
pub struct CudaBuffer {
    inner: Arc<CudaBufferInner>,
}

impl CudaBuffer {
    fn new(inner: Arc<CudaBufferInner>) -> Self {
        inner.dev.buffers.insert(&inner);
        Self { inner }
    }
}

impl Drop for CudaBuffer {
    fn drop(&mut self) {
        self.inner.dev.buffers.remove(self.inner.addr);
    }
}

/// 分配器（B.1/B.10/B.11）。退路池/外部登记挂在 `CudaDeviceInner`（设备级共享）。
pub(crate) struct CudaAllocator {
    dev: Arc<CudaDeviceInner>,
}

impl CudaAllocator {
    pub(crate) fn new(dev: Arc<CudaDeviceInner>) -> Self {
        Self { dev }
    }

    /// 原始存储分配（K.5 预分配池用）：按字节分配零初始化存储，不包装成缓冲。
    pub(crate) fn alloc_raw(&self, bytes: u64) -> Result<CudaStorage, BufferError> {
        let count = usize::try_from(bytes).map_err(|_| BufferError::CapacityExceeded {
            requested: bytes,
            capacity: self.dev.caps.max_buffer_bytes,
        })?;
        self.alloc_storage(StorageKind::Bytes, count, Init::Zeroed)
    }
    pub(crate) fn wrap(&self, storage: CudaStorage, dtype: DType, bytes: u64) -> Box<dyn Buffer> {
        let addr = storage.addr(&self.dev.stream);
        Box::new(CudaBuffer::new(Arc::new(CudaBufferInner {
            dev: Arc::clone(&self.dev),
            storage: Mutex::new(storage),
            dtype,
            bytes,
            addr,
        })))
    }

    fn alloc_storage(
        &self,
        kind: StorageKind,
        count: usize,
        init: Init,
    ) -> Result<CudaStorage, BufferError> {
        let stream = &self.dev.stream;
        macro_rules! typed {
            ($ty:ty, $variant:ident) => {{
                let mut slice = stream
                    .alloc_zeros::<$ty>(count)
                    .map_err(|error| errors::buffer_device_lost("cuda-alloc", error))?;
                if let Init::Fill(pattern) = init {
                    let values = vec![<$ty as FromPattern>::from_pattern(pattern); count];
                    if !values.is_empty() {
                        stream
                            .memcpy_htod(&values, &mut slice)
                            .map_err(|error| errors::buffer_device_lost("cuda-fill", error))?;
                    }
                }
                Ok(CudaStorage::$variant(slice))
            }};
        }
        match kind {
            StorageKind::Bytes => typed!(u8, Bytes),
            StorageKind::F32 => typed!(f32, F32),
            StorageKind::I32 => typed!(i32, I32),
            StorageKind::U32 => typed!(u32, U32),
            StorageKind::I64 => typed!(i64, I64),
            StorageKind::U64 => typed!(u64, U64),
        }
    }
}

impl Allocator for CudaAllocator {
    fn alloc(&self, dtype: DType, len: u64, init: Init) -> Result<Box<dyn Buffer>, BufferError> {
        let kind = storage_kind(dtype)?;
        let elem = dtype.byte_size();
        let bytes = elem.checked_mul(len).ok_or(BufferError::CapacityExceeded {
            requested: u64::MAX,
            capacity: self.dev.caps.max_buffer_bytes,
        })?;
        if bytes > self.dev.caps.max_buffer_bytes {
            return Err(BufferError::CapacityExceeded {
                requested: bytes,
                capacity: self.dev.caps.max_buffer_bytes,
            });
        }
        let count = usize::try_from(len).map_err(|_| BufferError::CapacityExceeded {
            requested: bytes,
            capacity: self.dev.caps.max_buffer_bytes,
        })?;
        let storage = self.alloc_storage(kind, count, init)?;
        Ok(self.wrap(storage, dtype, bytes))
    }

    fn transient(&self, bytes: u64) -> Result<Box<dyn Buffer>, BufferError> {
        // B.10 双路：true=一次性异步分配；false=复用预分配池（池空显式报错；两条路径都不隐式同步）。
        let mode = {
            let mut trace = super::lock(&self.dev.fallback_trace);
            super::fallback_plan::transient_mode(&self.dev.caps.fallbacks, &mut trace)
        };
        match mode {
            TransientMode::NativeAsync => {
                let count = usize::try_from(bytes).map_err(|_| BufferError::CapacityExceeded {
                    requested: bytes,
                    capacity: self.dev.caps.max_buffer_bytes,
                })?;
                let storage = self.alloc_storage(StorageKind::Bytes, count, Init::Uninit)?;
                Ok(self.wrap(storage, DType::U8, bytes))
            }
            TransientMode::PreallocatedPool => {
                let mut pool = super::lock(&self.dev.transient_pool);
                let Some(position) = pool
                    .iter()
                    .position(|(_, pooled_bytes)| *pooled_bytes >= bytes)
                else {
                    return Err(BufferError::CapacityExceeded {
                        requested: bytes,
                        capacity: 0,
                    });
                };
                let (slice, pooled_bytes) = pool.swap_remove(position);
                Ok(self.wrap(CudaStorage::Bytes(slice), DType::U8, pooled_bytes))
            }
        }
    }

    fn borrow_external(
        &self,
        handle: crate::runtime::kernel_exec::buffer::ExternalHandle,
    ) -> Result<Box<dyn Buffer>, BufferError> {
        // B.11 双路：true=零拷贝借用；false=拷贝到独立所有权缓冲（退路）。
        let mode = {
            let mut trace = super::lock(&self.dev.fallback_trace);
            super::fallback_plan::external_borrow_mode(&self.dev.caps.fallbacks, &mut trace)
        };
        match mode {
            ExternalBorrowMode::NativeZeroCopy => {
                // 零拷贝借用需要外部所有者注册/驱动导入，属 P3；P1-c 只保证退路可用。
                Err(errors::buffer_unsupported("B.11 native-zero-copy"))
            }
            ExternalBorrowMode::Copy => {
                let Some(bytes) = super::lock(&self.dev.external).get(&handle.0).copied() else {
                    return Err(errors::buffer_unsupported("B.11 unregistered-extent"));
                };
                let count = usize::try_from(bytes).map_err(|_| BufferError::CapacityExceeded {
                    requested: bytes,
                    capacity: self.dev.caps.max_buffer_bytes,
                })?;
                let storage = self.alloc_storage(StorageKind::Bytes, count, Init::Zeroed)?;
                let owned = storage.addr(&self.dev.stream);
                // SAFETY: 外部所有者通过 `register_external` 承诺 [handle, handle+bytes) 是有效的
                // 设备内存；目标缓冲由本分配器持有且容量足够；同步 dtod 拷贝只在退路路径使用。
                unsafe {
                    cudarc::driver::result::memcpy_dtod_sync(
                        owned.0,
                        handle.0,
                        usize::try_from(bytes).unwrap_or(usize::MAX),
                    )
                }
                .map_err(|error| errors::buffer_device_lost("cuda-external-copy", error))?;
                Ok(self.wrap(storage, DType::U8, bytes))
            }
        }
    }
}

impl Buffer for CudaBuffer {
    fn addr(&self) -> Address64 {
        self.inner.addr
    }

    fn byte_len(&self) -> u64 {
        self.inner.bytes
    }

    fn copy_range(
        &self,
        dst: &dyn Buffer,
        dst_off: u64,
        src_off: u64,
        len: u64,
    ) -> Result<(), BufferError> {
        let dst_inner = self
            .inner
            .dev
            .buffers
            .lookup(dst.addr())
            .ok_or_else(errors::foreign_buffer)?;
        if !Arc::ptr_eq(&self.inner.dev, &dst_inner.dev) {
            return Err(errors::foreign_buffer());
        }
        if dst_inner.dtype != self.inner.dtype {
            return Err(errors::buffer_unsupported("B.5 dtype"));
        }
        let elem = self.inner.dtype.byte_size();
        let src_range = elem_range(src_off, len, elem, self.inner.bytes)?;
        let dst_range = elem_range(dst_off, len, elem, dst_inner.bytes)?;
        if Arc::ptr_eq(&self.inner, &dst_inner) {
            // 同缓冲：A0 B.5 不要求自拷贝；等范围按 no-op，重叠显式失败。
            return if src_range == dst_range {
                Ok(())
            } else {
                Err(errors::buffer_unsupported("B.5 self-copy"))
            };
        }
        let stream = Arc::clone(&self.inner.dev.stream);
        // 固定加锁顺序（地址升序）避免两个句柄并发互等。
        let src_first = self.inner.addr.0 < dst_inner.addr.0;
        let mut low = if src_first {
            self.inner.lock_storage()
        } else {
            dst_inner.lock_storage()
        };
        let mut high = if src_first {
            dst_inner.lock_storage()
        } else {
            self.inner.lock_storage()
        };
        let (src, dst) = if src_first {
            (&*low, &mut *high)
        } else {
            (&*high, &mut *low)
        };
        copy_typed(&stream, src, src_range, dst, dst_range)
    }

    fn upload(&self, host: &[u8], off: u64) -> Result<(), BufferError> {
        let elem = self.inner.dtype.byte_size();
        let range = elem_range(off, host.len() as u64, elem, self.inner.bytes)?;
        let stream = Arc::clone(&self.inner.dev.stream);
        let mut guard = self.inner.lock_storage();
        upload_typed(&stream, &mut guard, range, host)
    }

    fn readback(&self, host: &mut [u8], off: u64) -> Result<(), BufferError> {
        let elem = self.inner.dtype.byte_size();
        let range = elem_range(off, host.len() as u64, elem, self.inner.bytes)?;
        let stream = Arc::clone(&self.inner.dev.stream);
        let guard = self.inner.lock_storage();
        readback_typed(&stream, &guard, range, host)
    }

    fn pin(&self) -> Result<Pinned, BufferError> {
        Ok(Pinned {
            address: self.inner.addr,
        })
    }
}
