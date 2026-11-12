//! CUDA 缓冲编解码辅助（P1-c 从 `buffer.rs` 拆出，控制单文件 ≤500 行棘轮）。
//!
//! 只做"字节 ↔ typed slice"的安全转换与 `memcpy_*` 调用；所有权、注册表与错误分类在
//! `buffer.rs`。这些函数不排队列、不等待（memcpy 走同一 stream，异步入队）。

use std::ops::Range;
use std::sync::Arc;

use cudarc::driver::{CudaSlice, CudaStream, DeviceRepr};

use crate::runtime::kernel_exec::error::BufferError;

use super::buffer::CudaStorage;
use super::errors;

pub(super) fn copy_slice_typed<T: DeviceRepr>(
    stream: &Arc<CudaStream>,
    src: &CudaSlice<T>,
    src_range: Range<usize>,
    dst: &mut CudaSlice<T>,
    dst_range: Range<usize>,
) -> Result<(), BufferError> {
    let src_view = src.slice(src_range);
    let mut dst_view = dst.slice_mut(dst_range);
    stream
        .memcpy_dtod(&src_view, &mut dst_view)
        .map_err(|error| errors::buffer_device_lost("cuda-copy", error))
}

pub(super) fn copy_typed(
    stream: &Arc<CudaStream>,
    src: &CudaStorage,
    src_range: Range<usize>,
    dst: &mut CudaStorage,
    dst_range: Range<usize>,
) -> Result<(), BufferError> {
    match (src, dst) {
        (CudaStorage::Bytes(src), CudaStorage::Bytes(dst)) => {
            copy_slice_typed(stream, src, src_range, dst, dst_range)
        }
        (CudaStorage::F32(src), CudaStorage::F32(dst)) => {
            copy_slice_typed(stream, src, src_range, dst, dst_range)
        }
        (CudaStorage::I32(src), CudaStorage::I32(dst)) => {
            copy_slice_typed(stream, src, src_range, dst, dst_range)
        }
        (CudaStorage::U32(src), CudaStorage::U32(dst)) => {
            copy_slice_typed(stream, src, src_range, dst, dst_range)
        }
        (CudaStorage::I64(src), CudaStorage::I64(dst)) => {
            copy_slice_typed(stream, src, src_range, dst, dst_range)
        }
        (CudaStorage::U64(src), CudaStorage::U64(dst)) => {
            copy_slice_typed(stream, src, src_range, dst, dst_range)
        }
        _ => Err(errors::buffer_unsupported("B.5 dtype")),
    }
}

pub(super) fn upload_typed(
    stream: &Arc<CudaStream>,
    storage: &mut CudaStorage,
    range: Range<usize>,
    host: &[u8],
) -> Result<(), BufferError> {
    macro_rules! htod {
        ($slice:expr, $ty:ty) => {{
            let width = std::mem::size_of::<$ty>();
            let values: Vec<$ty> = host
                .chunks_exact(width)
                .map(|chunk| <$ty>::from_le_bytes(chunk.try_into().expect("chunk width")))
                .collect();
            let mut view = $slice.slice_mut(range);
            stream
                .memcpy_htod(&values, &mut view)
                .map_err(|error| errors::buffer_device_lost("cuda-upload", error))
        }};
    }
    match storage {
        CudaStorage::Bytes(slice) => {
            let mut view = slice.slice_mut(range);
            stream
                .memcpy_htod(host, &mut view)
                .map_err(|error| errors::buffer_device_lost("cuda-upload", error))
        }
        CudaStorage::F32(slice) => htod!(slice, f32),
        CudaStorage::I32(slice) => htod!(slice, i32),
        CudaStorage::U32(slice) => htod!(slice, u32),
        CudaStorage::I64(slice) => htod!(slice, i64),
        CudaStorage::U64(slice) => htod!(slice, u64),
    }
}

pub(super) fn readback_typed(
    stream: &Arc<CudaStream>,
    storage: &CudaStorage,
    range: Range<usize>,
    host: &mut [u8],
) -> Result<(), BufferError> {
    macro_rules! dtoh {
        ($slice:expr, $ty:ty) => {{
            let mut values: Vec<$ty> = vec![<$ty>::default(); range.len()];
            stream
                .memcpy_dtoh(&$slice.slice(range), &mut values)
                .map_err(|error| errors::buffer_device_lost("cuda-readback", error))?;
            let width = std::mem::size_of::<$ty>();
            for (chunk, value) in host.chunks_exact_mut(width).zip(values.iter()) {
                chunk.copy_from_slice(&value.to_le_bytes());
            }
            Ok(())
        }};
    }
    match storage {
        CudaStorage::Bytes(slice) => stream
            .memcpy_dtoh(&slice.slice(range), host)
            .map_err(|error| errors::buffer_device_lost("cuda-readback", error)),
        CudaStorage::F32(slice) => dtoh!(slice, f32),
        CudaStorage::I32(slice) => dtoh!(slice, i32),
        CudaStorage::U32(slice) => dtoh!(slice, u32),
        CudaStorage::I64(slice) => dtoh!(slice, i64),
        CudaStorage::U64(slice) => dtoh!(slice, u64),
    }
}
