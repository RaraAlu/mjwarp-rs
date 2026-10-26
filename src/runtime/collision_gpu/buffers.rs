//! G10@P5：产品 NVRTC 入口的 CUDA 缓冲 helper（分配/上传/设备指针/回读/截断）。
//!
//! 从 `collision_gpu.rs` 抽出以控制单文件行数（行为不变；空 slice 上传语义与 ABI 注释同源）。

use std::sync::Arc;

use cudarc::driver::{
    CudaSlice, CudaStream, DevicePtr, DevicePtrMut, DeviceRepr, SyncOnDrop, ValidAsZeroBits,
};

use super::CollisionGpuError;

pub(crate) fn zeroed<T: DeviceRepr + ValidAsZeroBits>(
    stream: &Arc<CudaStream>,
    len: usize,
) -> Result<CudaSlice<T>, CollisionGpuError> {
    stream
        .alloc_zeros::<T>(len)
        .map_err(|error| CollisionGpuError::Launch(format!("alloc: {error:?}")))
}

/// 上传输入数组之一；空数组给 1 元素零缓冲（0 字节分配在 CUDA 上无意义，
/// 且对应内核在该数组上的读取次数为 0）。
pub(crate) fn upload<T: DeviceRepr + ValidAsZeroBits>(
    stream: &Arc<CudaStream>,
    values: &[T],
) -> Result<CudaSlice<T>, CollisionGpuError> {
    if values.is_empty() {
        return zeroed(stream, 1);
    }
    stream
        .clone_htod(values)
        .map_err(|error| CollisionGpuError::Launch(format!("upload: {error:?}")))
}

pub(crate) fn input_pointer<'a, T>(
    slice: &'a CudaSlice<T>,
    stream: &'a CudaStream,
    guards: &mut Vec<SyncOnDrop<'a>>,
) -> *const T {
    let (pointer, guard) = slice.device_ptr(stream);
    guards.push(guard);
    pointer as usize as *const T
}

pub(crate) fn output_pointer<'a, T>(
    slice: &'a mut CudaSlice<T>,
    stream: &'a CudaStream,
    guards: &mut Vec<SyncOnDrop<'a>>,
) -> *mut T {
    let (pointer, guard) = slice.device_ptr_mut(stream);
    guards.push(guard);
    pointer as usize as *mut T
}

pub(crate) fn readback<T: DeviceRepr, S: DevicePtr<T>>(
    stream: &Arc<CudaStream>,
    slice: &S,
) -> Result<Vec<T>, CollisionGpuError> {
    stream
        .clone_dtoh(slice)
        .map_err(|error| CollisionGpuError::Launch(format!("readback: {error:?}")))
}

pub(crate) fn truncated<T>(mut values: Vec<T>, count: usize) -> Vec<T> {
    if count < values.len() {
        values.truncate(count);
    }
    values
}
