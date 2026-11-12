//! CUDA 适配层单测（P1-b）。
//!
//! 默认测试 **0 GPU**：只测纯函数（冻结表、能力表构造、DType→存储映射、范围换算、发射
//! 校验、错误映射）与契约错误路径；需要驱动/NVRTC 的等价测试一律 `#[ignore]`，只在既有
//! GPU 窗口按需运行，**不在本卡声明通过**。

use crate::runtime::kernel_exec::backend::NumericPolicy;
use crate::runtime::kernel_exec::buffer::DType;
use crate::runtime::kernel_exec::error::{DeviceError, ErrorClass};
use crate::runtime::kernel_exec::module::Kernel;
use crate::runtime::kernel_exec::queue::{Alias, Arg, IndexSpace, Launch};

use super::buffer::{StorageKind, elem_range, storage_kind};
use super::queue::validate_launch;
use super::{cuda_fallbacks, device_caps_from_attributes, require_frozen_policy};

fn caps() -> crate::runtime::kernel_exec::backend::DeviceCaps {
    device_caps_from_attributes(1024, 65_535, 49_152, 8 << 30)
}

fn launch() -> Launch {
    Launch::new(Kernel::from_handle(1))
}

#[test]
fn frozen_fallback_flags_match_p1b_table() {
    let flags = cuda_fallbacks();
    // true = 适配层已实现原生路径；false = 走退路（P1-c 复核翻转）。
    assert!(flags.transient_alloc);
    assert!(!flags.zero_copy_external);
    assert!(flags.generic_device_fn);
    assert!(flags.native_source_injection);
    assert!(flags.occupancy_hint);
    assert!(!flags.graph_capture);
    assert!(!flags.device_loop);
    assert!(!flags.device_branch);
    assert!(!flags.capture_alloc);
    assert!(!flags.kernel_printf);
    assert!(flags.int64);
    assert!(flags.f32_atomic);
}

#[test]
fn device_caps_are_built_from_attributes() {
    let caps = device_caps_from_attributes(1024, 65_535, 49_152, 8 << 30);
    assert_eq!(caps.max_index_dims, 3);
    assert_eq!(caps.max_block_threads, 1024);
    assert_eq!(caps.max_blocks, 65_535);
    assert_eq!(caps.shared_bytes_per_block, 49_152);
    assert_eq!(caps.max_buffer_bytes, 8 << 30);
    assert_eq!(caps.numeric, NumericPolicy::FROZEN_F32);
    // 负数属性保守归零，不 panic。
    let conservative = device_caps_from_attributes(-1, -1, -1, 0);
    assert_eq!(conservative.max_block_threads, 0);
    assert_eq!(conservative.shared_bytes_per_block, 0);
}

#[test]
fn scalar_dtypes_are_supported_and_composites_are_explicit() {
    for dtype in [
        DType::U8,
        DType::F32,
        DType::I32,
        DType::U32,
        DType::I64,
        DType::U64,
        DType::Bool,
    ] {
        assert!(storage_kind(dtype).is_ok(), "scalar dtype {dtype:?}");
    }
    assert_eq!(storage_kind(DType::F32).unwrap(), StorageKind::F32);
    assert_eq!(storage_kind(DType::Bool).unwrap(), StorageKind::Bytes);
    for dtype in [
        DType::Vec2,
        DType::Vec3,
        DType::Vec4,
        DType::Vec6,
        DType::Vec10,
        DType::Mat33,
        DType::Quat,
    ] {
        assert!(matches!(
            storage_kind(dtype),
            Err(crate::runtime::kernel_exec::error::BufferError::Unsupported(_))
        ));
    }
}

#[test]
fn byte_ranges_are_converted_with_explicit_failures() {
    assert_eq!(elem_range(0, 16, 4, 16).unwrap(), 0..4);
    assert_eq!(elem_range(8, 8, 4, 16).unwrap(), 2..4);
    assert!(matches!(
        elem_range(2, 4, 4, 16),
        Err(crate::runtime::kernel_exec::error::BufferError::Unaligned { .. })
    ));
    assert!(matches!(
        elem_range(16, 4, 4, 16),
        Err(crate::runtime::kernel_exec::error::BufferError::OutOfRange { .. })
    ));
    assert!(matches!(
        elem_range(u64::MAX, 1, 4, 16),
        Err(crate::runtime::kernel_exec::error::BufferError::OutOfRange { .. })
    ));
}

#[test]
fn frozen_policy_is_required() {
    assert!(require_frozen_policy(NumericPolicy::FROZEN_F32).is_ok());
    let mut policy = NumericPolicy::FROZEN_F32;
    policy.fast_math = true;
    let err = require_frozen_policy(policy).unwrap_err();
    assert!(err.is_unsupported());
}

#[test]
fn launch_validation_rejects_unsupported_shapes() {
    let caps = caps();
    assert!(validate_launch(&launch(), &caps).is_ok());

    let mut multidim = launch();
    multidim.dims = IndexSpace::D2;
    assert!(
        validate_launch(&multidim, &caps)
            .unwrap_err()
            .is_unsupported()
    );

    let mut zero_blocks = launch();
    zero_blocks.blocks = 0;
    assert_eq!(
        validate_launch(&zero_blocks, &caps).unwrap_err().class(),
        Some(ErrorClass::Launch)
    );

    let mut wide_block = launch();
    wide_block.block_width = 2048;
    assert_eq!(
        validate_launch(&wide_block, &caps).unwrap_err().class(),
        Some(ErrorClass::Launch)
    );

    let mut aliased = launch();
    aliased.aliases = vec![Alias {
        input: crate::runtime::kernel_exec::buffer::Address64(1),
        output: crate::runtime::kernel_exec::buffer::Address64(1),
        offset: 0,
    }];
    assert!(
        validate_launch(&aliased, &caps)
            .unwrap_err()
            .is_unsupported()
    );

    let mut offset = launch();
    offset.args = vec![Arg::Buffer {
        address: crate::runtime::kernel_exec::buffer::Address64(1),
        offset: 4,
    }];
    assert!(
        validate_launch(&offset, &caps)
            .unwrap_err()
            .is_unsupported()
    );

    let mut duplicate = launch();
    duplicate.args = vec![
        Arg::Buffer {
            address: crate::runtime::kernel_exec::buffer::Address64(1),
            offset: 0,
        },
        Arg::Buffer {
            address: crate::runtime::kernel_exec::buffer::Address64(1),
            offset: 0,
        },
    ];
    assert!(
        validate_launch(&duplicate, &caps)
            .unwrap_err()
            .is_unsupported()
    );
}

#[test]
fn unknown_kernel_handle_is_launch_error() {
    let err = DeviceError::Launch(
        crate::runtime::kernel_exec::error::LaunchError::InvalidArgument("unknown".to_owned()),
    );
    assert_eq!(err.class(), Some(ErrorClass::Launch));
}

// ---------------------------------------------------------------------------
// 以下测试需要 NVIDIA GPU + NVRTC，默认 `#[ignore]`；只在既有 GPU 窗口按需运行。
// ---------------------------------------------------------------------------

const FILL_KERNEL: &str = r#"
extern "C" __global__ void fill(float* out) {
    out[threadIdx.x] = 7.0f;
}
"#;

#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC"]
fn gpu_device_caps_are_filled_from_real_device() {
    let backend = super::CudaBackend::new();
    let device = backend
        .open_device(0)
        .expect("CUDA device 0 must be openable");
    let caps = device.caps();
    assert!(caps.max_block_threads > 0);
    assert!(caps.shared_bytes_per_block > 0);
    assert!(caps.max_buffer_bytes > 0);
    assert!(caps.fallbacks.occupancy_hint);
}

#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC"]
fn gpu_buffer_roundtrip_copy_and_launch_equivalence() {
    use crate::runtime::kernel_exec::buffer::Init;
    use crate::runtime::kernel_exec::module::{CompileOptions, KernelSource};

    let backend = super::CudaBackend::new();
    let device = backend.open_device(0).expect("CUDA device 0");
    let allocator = device.allocator();
    let queue = device.queue().expect("queue");
    let module = device.module();

    // 1) 上传/回读往返逐字节等值。
    let buffer = allocator.alloc(DType::F32, 4, Init::Zeroed).expect("alloc");
    let values = [1.0f32, 2.0, 3.0, 4.0];
    let bytes: Vec<u8> = values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    buffer.upload(&bytes, 0).expect("upload");
    let mut out = [0u8; 16];
    buffer.readback(&mut out, 0).expect("readback");
    assert_eq!(out, bytes.as_slice());

    // 2) 设备内范围拷贝与旧路径同值（同设备/同 dtype）。
    let dst = allocator
        .alloc(DType::F32, 4, Init::Zeroed)
        .expect("alloc dst");
    buffer
        .copy_range(dst.as_ref(), 0, 0, 16)
        .expect("copy_range");
    let mut copied = [0u8; 16];
    dst.readback(&mut copied, 0).expect("readback copy");
    assert_eq!(copied, bytes.as_slice());

    // 3) NVRTC 编译 → 发射 → wait_idle → 回读。
    let kernel = module
        .compile(
            KernelSource::Inline(FILL_KERNEL),
            CompileOptions::default().with_entry("fill"),
        )
        .expect("compile");
    let mut launch = Launch::new(kernel);
    launch.blocks = 1;
    launch.block_width = 32;
    launch.args = vec![Arg::Buffer {
        address: dst.addr(),
        offset: 0,
    }];
    queue.submit(launch).expect("submit");
    queue.wait_idle().expect("wait_idle");
    let mut launched = [0u8; 16];
    dst.readback(&mut launched, 0).expect("readback launch");
    let (chunks, _) = launched.as_chunks::<4>();
    let floats: Vec<f32> = chunks
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect();
    assert_eq!(floats, vec![7.0f32; 4]);
}

#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC"]
fn gpu_event_elapsed_is_observed() {
    use crate::runtime::kernel_exec::device::Diagnostics;

    let inner = super::open_device_inner(0).expect("CUDA device 0");
    let diagnostics = super::CudaDiagnostics::new(inner);
    let start = diagnostics.record_event().expect("start event");
    let end = diagnostics.record_event().expect("end event");
    let elapsed = diagnostics.elapsed(&start, &end);
    assert!(elapsed.as_secs_f64() < 1.0);
}
