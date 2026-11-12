//! CUDA 队列与发射（A.3/G 域；P1-b）。
//!
//! P1-b 只覆盖"缓冲区 + 4 种标量、1 维索引空间、缓冲 offset=0、无别名/无重复缓冲"的发射形态
//! （物理层 507 个调用点的主形态）；D2/D3、纹理/数组描述符、别名约束、捕获/重放属 P1-c/P2/P3，
//! 一律显式 `Unsupported`，不静默降级。
//!
//! `submit` 语义：只入队、不等待（A0 设计约束 2）；唯一显式排空点是 [`Queue::wait_idle`]。
//!
//! 实参打包：缓冲参数按其设备地址以 [`DevicePtrArg`]（transparent u64）传给内核；宿主侧不
//! 解引用设备指针，也不依赖 `cudarc` 的 `CudaSlice` 借用。缓冲存活由注册表 `Arc` 保证到
//! `launch` 返回（CUDA 只捕获指针值，真正执行异步发生）。同流有序由 `CudaStream` 保证。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use cudarc::driver::{DeviceRepr, LaunchConfig, PushKernelArg};

use crate::runtime::kernel_exec::backend::DeviceCaps;
use crate::runtime::kernel_exec::error::{DeviceError, LaunchError};
use crate::runtime::kernel_exec::queue::{Arg, Graph, IndexSpace, Launch, Queue, Submission};

use super::buffer::CudaBufferInner;
use super::errors;
use super::{CudaDeviceInner, require_frozen_policy};

/// 内核实参里的设备指针（CUDA ABI：8 字节无符号指针值）。
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
struct DevicePtrArg(u64);

// SAFETY: `#[repr(transparent)]` over u64，无填充、大小/对齐与 CUdeviceptr 一致；cudarc 用它
// 把值拷进内核参数区，宿主侧不解引用。该类型只在 `submit` 内作为实参值使用。
unsafe impl DeviceRepr for DevicePtrArg {}

/// 标量实参的保活载体（`LaunchArgs` 只存指针，值必须活到 `launch()`）。
#[derive(Clone, Copy, Debug)]
enum Scalar {
    I32(i32),
    U32(u32),
    U64(u64),
    F32(f32),
}

fn invalid_argument(message: impl Into<String>) -> DeviceError {
    DeviceError::Launch(LaunchError::InvalidArgument(message.into()))
}

/// 发射描述校验（纯函数，0 GPU 可测）。
pub(crate) fn validate_launch(launch: &Launch, caps: &DeviceCaps) -> Result<(), DeviceError> {
    if launch.dims != IndexSpace::D1 {
        return Err(errors::unsupported("G.1 multidim-index-space"));
    }
    if launch.blocks == 0 || launch.block_width == 0 {
        return Err(DeviceError::Launch(LaunchError::InvalidConfig(
            "launch blocks/block_width must be non-zero".to_owned(),
        )));
    }
    if launch.block_width > caps.max_block_threads {
        return Err(DeviceError::Launch(LaunchError::InvalidConfig(format!(
            "block_width {} exceeds device max {}",
            launch.block_width, caps.max_block_threads
        ))));
    }
    require_frozen_policy(launch.numeric)?;
    if !launch.aliases.is_empty() {
        return Err(errors::unsupported("G.5 alias-declaration"));
    }
    let mut addresses: Vec<u64> = Vec::new();
    for arg in &launch.args {
        if let Arg::Buffer { address, offset } = arg {
            if *offset != 0 {
                return Err(errors::unsupported("G.3 buffer-offset"));
            }
            if addresses.contains(&address.0) {
                return Err(errors::unsupported("G.5 duplicate-buffer"));
            }
            addresses.push(address.0);
        }
    }
    Ok(())
}

/// 队列（A.3 载体）。
pub(crate) struct CudaQueue {
    dev: Arc<CudaDeviceInner>,
    next: AtomicU64,
}

impl CudaQueue {
    pub(crate) fn new(dev: Arc<CudaDeviceInner>) -> Self {
        Self {
            dev,
            next: AtomicU64::new(1),
        }
    }

    /// 校验缓冲地址属于本设备，并返回保活句柄（按实参顺序）。
    fn resolve_buffers(&self, launch: &Launch) -> Result<Vec<Arc<CudaBufferInner>>, DeviceError> {
        let mut buffers = Vec::new();
        for arg in &launch.args {
            let Arg::Buffer { address, .. } = arg else {
                continue;
            };
            let buffer =
                self.dev.buffers.lookup(*address).ok_or_else(|| {
                    invalid_argument(format!("unknown buffer address {}", address.0))
                })?;
            if !Arc::ptr_eq(&buffer.dev, &self.dev) {
                return Err(invalid_argument("buffer belongs to another device"));
            }
            buffers.push(buffer);
        }
        Ok(buffers)
    }
}

impl Queue for CudaQueue {
    fn submit(&self, launch: Launch) -> Result<Submission, DeviceError> {
        validate_launch(&launch, &self.dev.caps)?;
        let function = self
            .dev
            .kernels
            .get(launch.kernel.handle())
            .ok_or_else(|| invalid_argument("unknown kernel handle"))?;
        // 缓冲句柄保活到 `launch` 返回；CUDA 只拷贝指针值。
        let _buffers = self.resolve_buffers(&launch)?;
        let mut pointers: Vec<DevicePtrArg> = Vec::new();
        let mut scalars: Vec<Option<Scalar>> = Vec::with_capacity(launch.args.len());
        for arg in &launch.args {
            match arg {
                Arg::Buffer { address, .. } => {
                    pointers.push(DevicePtrArg(address.0));
                    scalars.push(None);
                }
                Arg::I32(value) => {
                    scalars.push(Some(Scalar::I32(*value)));
                }
                Arg::U32(value) => {
                    scalars.push(Some(Scalar::U32(*value)));
                }
                Arg::U64(value) => {
                    scalars.push(Some(Scalar::U64(*value)));
                }
                Arg::F32(value) => {
                    scalars.push(Some(Scalar::F32(*value)));
                }
            }
        }
        let mut builder = self.dev.stream.launch_builder(function.as_ref());
        let mut pointer_index = 0;
        for scalar in &scalars {
            match scalar {
                Some(Scalar::I32(value)) => {
                    builder.arg(value);
                }
                Some(Scalar::U32(value)) => {
                    builder.arg(value);
                }
                Some(Scalar::U64(value)) => {
                    builder.arg(value);
                }
                Some(Scalar::F32(value)) => {
                    builder.arg(value);
                }
                None => {
                    builder.arg(&pointers[pointer_index]);
                    pointer_index += 1;
                }
            }
        }
        let config = LaunchConfig {
            grid_dim: (launch.blocks, 1, 1),
            block_dim: (launch.block_width, 1, 1),
            shared_mem_bytes: 0,
        };
        // SAFETY: 内核句柄来自本设备 `Module::compile`；实参个数/类型与 ABI 的匹配由调用方按
        // `CompileOptions`/入口约定保证（A0 §2 G.3 的 opaque 描述符取舍）；缓冲地址来自本设备
        // 注册表，句柄在 `launch` 返回前保持存活，`launch` 只入队不等待。调用方须保证内核访问
        // 范围落在各缓冲容量内，且不在本流之外并发使用同一缓冲。
        unsafe { builder.launch(config) }
            .map_err(|error| errors::launch_driver("cuda-launch", error))?;
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        Ok(Submission(id))
    }

    fn wait_idle(&self) -> Result<(), DeviceError> {
        self.dev
            .context
            .bind_to_thread()
            .map_err(|error| errors::device_lost("cuda-context-bind", error))?;
        self.dev
            .stream
            .synchronize()
            .map_err(|error| errors::device_lost("cuda-wait-idle", error))?;
        self.dev
            .context
            .check_err()
            .map_err(|error| errors::device_lost("cuda-status", error))
    }

    fn replay(&self, _graph: &Graph) -> Result<(), DeviceError> {
        Err(errors::unsupported("K.1 replay"))
    }

    fn capture_alloc(
        &self,
    ) -> Result<Box<dyn crate::runtime::kernel_exec::buffer::Buffer>, DeviceError> {
        Err(errors::unsupported("K.5 capture-alloc"))
    }
}
