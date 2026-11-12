//! CUDA 后端适配层（P1-b）。
//!
//! 只实现 P1-a 的宿主面 trait（[`Backend`]/[`Device`]/[`Allocator`]/[`Buffer`]/
//! [`Queue`]/[`Module`]/[`Diagnostics`]）；设备内原语（`DeviceMath`/`Atomics`/
//! `ThreadScope`/`ExecControl`/`QueueCapture`）属书写面（P2），不在本层实现。
//!
//! 边界：**先包不改**——不接任何现有调用点，旧 `runtime::transfer`/`runtime::launcher`/
//! `runtime::cuda` 路径保持原样（双轨并存，迁移属 P2）；不碰 `.cu`/`*_CUDA`/recipe/PTX/ABI。
//! 本层默认测试不创建 CUDA context；GPU 等价测试一律 `#[ignore]`。
//!
//! 共享状态：`Backend::open` 建 [`CudaDeviceInner`]（context/stream/caps/两张注册表），
//! `Device::{queue,allocator,module}` 返回共享同一 `Arc` 的句柄，使缓冲与内核句柄在同一
//! 设备域内可解析；注册表用 `Mutex`，不做跨线程无锁优化。

mod buffer;
mod diagnostics;
mod errors;
mod module;
mod queue;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cudarc::driver::sys;
use cudarc::driver::{CudaContext, CudaFunction, CudaStream};

use crate::runtime::presence;

use super::backend::{Backend, Device, DeviceCaps, DeviceId, FallbackSet, NumericPolicy};
use super::buffer::Allocator;
use super::error::DeviceError;
use super::module::Module;
use super::queue::Queue;

use self::buffer::{BufferRegistry, CudaAllocator};
use self::diagnostics::CudaDiagnostics;
use self::module::CudaModule;
use self::queue::CudaQueue;

/// 加锁辅助：锁内只改映射，不调用驱动；中毒时映射仍完整。
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 已编译内核句柄表（opaque `Kernel::handle()` → 驱动函数）。
#[derive(Default)]
pub(crate) struct KernelRegistry {
    next: AtomicU64,
    functions: Mutex<HashMap<u64, Arc<CudaFunction>>>,
}

impl KernelRegistry {
    pub(crate) fn insert(&self, function: CudaFunction) -> u64 {
        let handle = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        lock(&self.functions).insert(handle, Arc::new(function));
        handle
    }

    pub(crate) fn get(&self, handle: u64) -> Option<Arc<CudaFunction>> {
        lock(&self.functions).get(&handle).cloned()
    }
}

/// 设备共享状态（同一设备的 queue/allocator/module 共用）。
pub(crate) struct CudaDeviceInner {
    pub(crate) context: Arc<CudaContext>,
    pub(crate) stream: Arc<CudaStream>,
    pub(crate) caps: DeviceCaps,
    pub(crate) buffers: BufferRegistry,
    pub(crate) kernels: KernelRegistry,
}

/// CUDA 后端（A.1 载体）。
#[derive(Clone, Copy, Debug, Default)]
pub struct CudaBackend;

impl CudaBackend {
    /// 构造。
    pub const fn new() -> Self {
        Self
    }

    /// 打开指定序号的设备（`Backend::open` 的便捷入口，语义相同）。
    pub fn open_device(&self, ordinal: u32) -> Result<Box<dyn Device>, DeviceError> {
        self.open(&DeviceId(ordinal))
    }
}

impl Backend for CudaBackend {
    fn devices(&self) -> Result<Vec<DeviceId>, DeviceError> {
        if !presence::driver_library_present() {
            return Err(errors::device_lost(
                "cuda-driver",
                "driver library unavailable",
            ));
        }
        let count = presence::device_count_now()
            .map_err(|error| errors::device_lost("cuda-device-count", error))?;
        Ok((0..count)
            .map(|ordinal| DeviceId(u32::try_from(ordinal).unwrap_or(u32::MAX)))
            .collect())
    }

    fn open(&self, id: &DeviceId) -> Result<Box<dyn Device>, DeviceError> {
        let inner = open_device_inner(id.0)?;
        Ok(Box::new(CudaDevice { inner }))
    }
}

/// 打开设备的全部驱动资源；失败按 A.4 分类返回。
pub(crate) fn open_device_inner(ordinal: u32) -> Result<Arc<CudaDeviceInner>, DeviceError> {
    if !presence::driver_library_present() {
        return Err(errors::device_lost(
            "cuda-driver",
            "driver library unavailable",
        ));
    }
    let count = presence::device_count_now()
        .map_err(|error| errors::device_lost("cuda-device-count", error))?;
    let ordinal = usize::try_from(ordinal)
        .map_err(|_| errors::device_lost("cuda-device", "ordinal does not fit usize"))?;
    if ordinal >= count {
        return Err(errors::device_lost(
            "cuda-device",
            format!("requested {ordinal}, available {count}"),
        ));
    }
    // 缺驱动库时 cudarc 的动态加载会 panic：先做库存在性预检，再捕获残余 panic。
    let context = std::panic::catch_unwind(AssertUnwindSafe(|| CudaContext::new(ordinal)))
        .map_err(|_| errors::device_lost("cuda-context", "cudarc panicked"))?
        .map_err(|error| errors::device_lost("cuda-context", error))?;
    context
        .bind_to_thread()
        .map_err(|error| errors::device_lost("cuda-context-bind", error))?;
    let stream = context
        .new_stream()
        .map_err(|error| errors::device_lost("cuda-stream", error))?;
    let caps = query_caps(&context)?;
    Ok(Arc::new(CudaDeviceInner {
        context,
        stream,
        caps,
        buffers: BufferRegistry::default(),
        kernels: KernelRegistry::default(),
    }))
}

/// 真实设备能力查询（A.2）。
fn query_caps(context: &Arc<CudaContext>) -> Result<DeviceCaps, DeviceError> {
    let attribute = |attrib: sys::CUdevice_attribute| {
        context
            .attribute(attrib)
            .map_err(|error| errors::device_lost("cuda-caps", error))
    };
    let max_threads =
        attribute(sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_MAX_THREADS_PER_BLOCK)?;
    let max_blocks = attribute(sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_MAX_GRID_DIM_X)?;
    let shared =
        attribute(sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_MAX_SHARED_MEMORY_PER_BLOCK)?;
    let total_bytes = cudarc::driver::result::mem_get_info()
        .map_err(|error| errors::device_lost("cuda-mem-info", error))?
        .1 as u64;
    Ok(device_caps_from_attributes(
        max_threads,
        max_blocks,
        shared,
        total_bytes,
    ))
}

/// 由驱动属性构造能力表（纯函数，便于 0 GPU 单测冻结）。
pub(crate) fn device_caps_from_attributes(
    max_threads: i32,
    max_blocks: i32,
    shared_per_block: i32,
    total_bytes: u64,
) -> DeviceCaps {
    let to_u32 = |value: i32| u32::try_from(value).unwrap_or(0);
    DeviceCaps {
        fallbacks: cuda_fallbacks(),
        max_index_dims: 3,
        max_block_threads: to_u32(max_threads),
        max_blocks: to_u32(max_blocks),
        shared_bytes_per_block: to_u32(shared_per_block),
        max_buffer_bytes: total_bytes,
        numeric: NumericPolicy::FROZEN_F32,
    }
}

/// P1-b 冻结的 CUDA 退项取值（`true`=适配层已实现原生路径；其余 false 待 P1-c 复核翻转）。
pub(crate) const fn cuda_fallbacks() -> FallbackSet {
    FallbackSet {
        transient_alloc: true,
        zero_copy_external: false,
        generic_device_fn: true,
        native_source_injection: true,
        occupancy_hint: true,
        graph_capture: false,
        device_loop: false,
        device_branch: false,
        capture_alloc: false,
        kernel_printf: false,
        int64: true,
        f32_atomic: true,
    }
}

/// 数值策略检查（F.6：P1-b 只接受冻结配方，后端不得自行改 rounding/FMA/fast-math）。
pub(crate) fn require_frozen_policy(policy: NumericPolicy) -> Result<(), DeviceError> {
    if policy == NumericPolicy::FROZEN_F32 {
        Ok(())
    } else {
        Err(errors::unsupported("F.6 numeric-policy"))
    }
}

/// 一个已打开的 CUDA 设备（A.2/A.3 载体）。
pub struct CudaDevice {
    pub(crate) inner: Arc<CudaDeviceInner>,
}

impl CudaDevice {
    /// 诊断句柄（L.2/L.3；`Device` trait 面不含 Diagnostics，故由后端显式提供）。
    pub fn diagnostics(&self) -> CudaDiagnostics {
        CudaDiagnostics::new(Arc::clone(&self.inner))
    }
}

impl Device for CudaDevice {
    fn caps(&self) -> DeviceCaps {
        self.inner.caps
    }

    fn queue(&self) -> Result<Box<dyn Queue>, DeviceError> {
        Ok(Box::new(CudaQueue::new(Arc::clone(&self.inner))))
    }

    fn allocator(&self) -> Box<dyn Allocator> {
        Box::new(CudaAllocator::new(Arc::clone(&self.inner)))
    }

    fn module(&self) -> Box<dyn Module> {
        Box::new(CudaModule::new(Arc::clone(&self.inner)))
    }
}
