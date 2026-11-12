//! P1-a mock backend 单测（仅 `cfg(test)`；不计入产品路径）。

use std::rc::Rc;
use std::time::Duration;

use super::backend::{Backend, CapabilityId, Device, DeviceId};
use super::buffer::{Address64, Allocator, Buffer, DType, Init};
use super::device::{Atomics, Counter, DeviceMath, Diagnostics, Dim3, ExecControl, ThreadScope};
use super::error::{BufferError, DeviceError, LaunchError};
use super::mock_backend::*;
use super::module::{CompileOptions, Kernel, KernelSource, Module};
use super::queue::{IndexSpace, Launch, Queue, QueueCapture};

fn assert_dyn_backend(_: &dyn Backend) {}
fn assert_dyn_device(_: &dyn Device) {}
fn assert_dyn_allocator(_: &dyn Allocator) {}
fn assert_dyn_buffer(_: &dyn Buffer) {}
fn assert_dyn_queue(_: &dyn Queue) {}
fn assert_dyn_module(_: &dyn Module) {}
fn assert_dyn_diagnostics(_: &dyn Diagnostics) {}
fn assert_dyn_scope(_: &dyn ThreadScope) {}
fn assert_dyn_atomics(_: &dyn Atomics) {}

fn assert_math<T: DeviceMath>(value: f32) -> f32 {
    T::sqrt(value) + T::max(1.0, 2.0)
}

fn make_launch() -> Launch {
    Launch::new(Kernel::from_handle(42))
}

#[test]
fn backend_lists_and_opens_default_device() {
    let backend = MockBackend::new();
    assert_eq!(backend.devices().unwrap(), vec![DeviceId(0)]);
    let device = backend.open(&DeviceId(0)).unwrap();
    let caps = device.caps();
    assert_eq!(caps.max_block_threads, 1024);
    assert!(caps.fallbacks.graph_capture);
    assert_eq!(caps.numeric, super::backend::NumericPolicy::FROZEN_F32);
}

#[test]
fn unknown_device_is_device_lost() {
    let backend = MockBackend::new();
    let err = match backend.open(&DeviceId(9)) {
        Err(err) => err,
        Ok(_) => panic!("expected open error"),
    };
    assert_eq!(err.class(), Some(super::error::ErrorClass::DeviceLost));
}

#[test]
fn allocator_capacity_is_explicit() {
    let alloc = MockAllocator::new(16);
    match alloc.alloc(DType::F32, 8, Init::Zeroed) {
        Err(BufferError::CapacityExceeded {
            requested,
            capacity,
        }) => {
            assert_eq!((requested, capacity), (32, 16));
        }
        Ok(_) => panic!("expected capacity error"),
        Err(other) => panic!("unexpected buffer error: {other}"),
    }
    let buffer = alloc.alloc(DType::U8, 16, Init::Zeroed).unwrap();
    assert_eq!(buffer.byte_len(), 16);
    assert!(matches!(
        buffer.upload(&[0; 4], 14),
        Err(BufferError::OutOfRange { .. })
    ));
}

#[test]
fn buffer_upload_readback_roundtrip() {
    let alloc = MockAllocator::new(1024);
    let buffer = alloc.alloc(DType::U8, 32, Init::Zeroed).unwrap();
    buffer.upload(&[1, 2, 3, 4], 4).unwrap();
    let mut out = [0u8; 4];
    buffer.readback(&mut out, 4).unwrap();
    assert_eq!(out, [1, 2, 3, 4]);
    assert_eq!(buffer.pin().unwrap().address, buffer.addr());
}

#[test]
fn copy_range_is_explicit_and_works() {
    let alloc = MockAllocator::new(1024);
    let src = alloc.alloc(DType::U8, 16, Init::Zeroed).unwrap();
    let dst = alloc.alloc(DType::U8, 16, Init::Zeroed).unwrap();
    src.upload(&[9, 8, 7, 6], 0).unwrap();
    src.copy_range(dst.as_ref(), 4, 0, 4).unwrap();
    let mut out = [0u8; 4];
    dst.readback(&mut out, 4).unwrap();
    assert_eq!(out, [9, 8, 7, 6]);
    assert!(matches!(
        src.copy_range(dst.as_ref(), 14, 0, 4),
        Err(BufferError::OutOfRange { .. })
    ));
}

#[test]
fn submit_is_not_implicitly_synchronous() {
    let device = MockDevice::new();
    let queue = device.queue().unwrap();
    let state = Rc::clone(&device.state);
    assert!(state.borrow().idle);
    queue.submit(make_launch()).unwrap();
    assert!(!state.borrow().idle, "submit 不得隐式同步");
    queue.wait_idle().unwrap();
    assert!(state.borrow().idle, "wait_idle 是唯一显式排空点");
    assert_eq!(state.borrow().submissions.len(), 1);
}

#[test]
fn launch_and_capture_are_counted() {
    let device = MockDevice::new();
    let queue = device.queue().unwrap();
    let mut launch = make_launch();
    launch.dims = IndexSpace::D2;
    queue.submit(launch).unwrap();
    let graph = {
        let mock_queue = MockQueue {
            state: Rc::clone(&device.state),
        };
        mock_queue.capture(|_| {}).unwrap()
    };
    queue.replay(&graph).unwrap();
    assert_eq!(device.state.borrow().captures, 1);
    assert_eq!(device.state.borrow().replays, 1);
}

#[test]
fn module_compile_returns_kernel_and_rejects_empty_source() {
    let module = MockModule;
    let kernel = module
        .compile(
            KernelSource::Named("render_depth_seg"),
            CompileOptions::default(),
        )
        .unwrap();
    assert_eq!(kernel.handle(), 42);
    let err = module
        .compile(KernelSource::Inline(""), CompileOptions::default())
        .unwrap_err();
    assert_eq!(err.class(), Some(super::error::ErrorClass::Compile));
    // Inline 必须带入口名（P1-b 冻结）；Named 仍由后端按名解析。
    let missing_entry = module
        .compile(
            KernelSource::Inline("extern \"C\" void f() {}"),
            CompileOptions::default(),
        )
        .unwrap_err();
    assert_eq!(
        missing_entry.class(),
        Some(super::error::ErrorClass::Compile)
    );
    let with_entry = module
        .compile(
            KernelSource::Inline("extern \"C\" void f() {}"),
            CompileOptions::default().with_entry("f"),
        )
        .unwrap();
    assert_eq!(with_entry.handle(), 42);
}

#[test]
fn module_suggest_block_width_uses_frozen_fallback() {
    let module = MockModule;
    let suggestion = module
        .suggest_block_width(&Kernel::from_handle(42), 4096)
        .unwrap();
    assert_eq!(suggestion.block_width, 256);
    assert_eq!(suggestion.min_blocks, 1);
}

#[test]
fn object_safe_handles_compile() {
    let backend = MockBackend::new();
    assert_dyn_backend(&backend);
    let device: Box<dyn Device> = Box::new(MockDevice::new());
    assert_dyn_device(device.as_ref());
    let alloc: Box<dyn Allocator> = device.allocator();
    assert_dyn_allocator(alloc.as_ref());
    let buffer = alloc.alloc(DType::U8, 8, Init::Zeroed).unwrap();
    assert_dyn_buffer(buffer.as_ref());
    let queue: Box<dyn Queue> = device.queue().unwrap();
    assert_dyn_queue(queue.as_ref());
    let module: Box<dyn Module> = device.module();
    assert_dyn_module(module.as_ref());
    let diag = MockDiagnostics::new();
    assert_dyn_diagnostics(&diag);
    let scope = MockThreadScope::new();
    assert_dyn_scope(&scope);
    assert_dyn_atomics(&MockAtomics);
}

#[test]
fn device_side_traits_are_usable_via_generic_bounds() {
    assert!((assert_math::<MockDeviceMath>(4.0) - 4.0).abs() < f32::EPSILON);
    let scope = MockThreadScope::new();
    assert_eq!(scope.tid(), Dim3::ZERO);
    assert_eq!(scope.block_dim(), Dim3::new(1, 1, 1));
    let shared = scope.shared(1024).unwrap();
    assert_eq!(shared.address, Address64(0x5A00_0000 + 1024));
    assert!(matches!(
        scope.shared(48 * 1024 + 1),
        Err(DeviceError::Launch(LaunchError::InvalidConfig(_)))
    ));
    let no_shared = MockThreadScope::with_shared_bytes(0);
    assert!(matches!(
        no_shared.shared(1),
        Err(DeviceError::Unsupported(CapabilityId("I.2")))
    ));
    scope.barrier();
    assert_eq!(scope.barriers.get(), 1);
    let control = MockExecControl;
    let counter = Counter { handle: 2 };
    let mut hits = 0;
    control
        .loop_while_device(&counter, || {
            hits += 1;
        })
        .unwrap();
    assert_eq!(hits, 1);
    let atomics = MockAtomics;
    assert_eq!(atomics.add_i32(Address64(1), 3), 3);
    assert_eq!(atomics.cas_i32(Address64(1), 7, 8), 7);
}

#[test]
fn diagnostics_events_elapsed_and_printf() {
    let diag = MockDiagnostics::new();
    let a = diag.record_event().unwrap();
    let b = diag.record_event().unwrap();
    assert_eq!(diag.elapsed(&a, &b), Duration::from_micros(1));
    diag.printf("value=%d", &[super::queue::Arg::I32(7)]);
    assert_eq!(diag.logs.borrow().len(), 1);
}

#[test]
fn unsupported_capability_is_explicit_error() {
    let err = DeviceError::Unsupported(CapabilityId("K.2"));
    assert!(err.is_unsupported());
    assert_eq!(err.class(), None);
}

#[test]
fn launch_error_variants_are_distinct() {
    let err = DeviceError::Launch(LaunchError::AliasViolation("in/out alias".into()));
    assert_eq!(err.class(), Some(super::error::ErrorClass::Launch));
    assert!(err.to_string().contains("alias"));
}
