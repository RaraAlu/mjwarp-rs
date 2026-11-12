//! P1-c 退路双路测试（mock backend；0 GPU）。
//!
//! 覆盖验收口径：10 个 now 项各至少 1 条 true 路 + 1 条 false 路 + 1 条"不隐式同步"断言；
//! 12 个退项任意组合都不返回 `Unsupported`；降级必留 [`FallbackTrace`]；D.4/D.10 只登记。

use crate::runtime::kernel_exec::backend::{Device, FallbackSet};
use crate::runtime::kernel_exec::buffer::{Address64, ExternalHandle, Init};
use crate::runtime::kernel_exec::error::BufferError;
use crate::runtime::kernel_exec::mock_backend::{MockAllocator, MockDevice};
use crate::runtime::kernel_exec::module::{BlockSuggestion, Kernel};
use crate::runtime::kernel_exec::queue::{Arg, Launch};

use super::buffers::{
    BorrowDecision, TransientDecision, TransientPool, borrow_external, transient_alloc,
};
use super::control::{branch_on_device_fallback, loop_while_device_fallback};
use super::graph::{CapturePool, capture_sequential, preallocate_capture, replay_sequential};
use super::numeric::{F32AtomicCell, Int64Mode, atomic_add_f32, int64_mode, join_i64, split_i64};
use super::printf::{PrintfStatusBuffer, format_record, printf_fallback};
use super::{FallbackCapability, FallbackTrace, ImplScope, select_native};

fn native() -> FallbackSet {
    FallbackSet::all()
}

fn fallback() -> FallbackSet {
    FallbackSet::none()
}

fn launch() -> Launch {
    Launch::new(Kernel::from_handle(1))
}

fn assert_no_sync(device: &MockDevice) {
    assert_eq!(device.state.borrow().waits, 0, "退路路径不得隐式 wait_idle");
}

#[test]
fn registry_has_12_flags_with_10_now_and_2_after_shim() {
    assert_eq!(FallbackCapability::ALL.len(), 12);
    let mut ids: Vec<&'static str> = FallbackCapability::ALL.iter().map(|c| c.id()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 12);
    let mut flags: Vec<&'static str> = FallbackCapability::ALL
        .iter()
        .map(|c| c.flag_name())
        .collect();
    flags.sort_unstable();
    flags.dedup();
    assert_eq!(flags.len(), 12);
    let now = FallbackCapability::ALL
        .iter()
        .filter(|c| c.impl_scope() == ImplScope::Now)
        .count();
    let after_shim = FallbackCapability::ALL
        .iter()
        .filter(|c| c.impl_scope() == ImplScope::AfterShim)
        .count();
    assert_eq!((now, after_shim), (10, 2));
    assert_eq!(
        FallbackCapability::GenericDeviceFn.impl_scope(),
        ImplScope::AfterShim
    );
    assert_eq!(
        FallbackCapability::NativeSourceInjection.impl_scope(),
        ImplScope::AfterShim
    );
    // 极性：all() 全原生，none() 全退路。
    for capability in FallbackCapability::ALL {
        assert!(
            capability.is_native(&native()),
            "{} native",
            capability.id()
        );
        assert!(
            capability.requires_fallback(&fallback()),
            "{} fallback",
            capability.id()
        );
    }
}

#[test]
fn any_combination_of_12_flags_has_no_unsupported_path() {
    for mask in 0u16..(1 << 12) {
        let mut set = FallbackSet::none();
        set.transient_alloc = mask & (1 << 0) != 0;
        set.zero_copy_external = mask & (1 << 1) != 0;
        set.generic_device_fn = mask & (1 << 2) != 0;
        set.native_source_injection = mask & (1 << 3) != 0;
        set.occupancy_hint = mask & (1 << 4) != 0;
        set.graph_capture = mask & (1 << 5) != 0;
        set.device_loop = mask & (1 << 6) != 0;
        set.device_branch = mask & (1 << 7) != 0;
        set.capture_alloc = mask & (1 << 8) != 0;
        set.kernel_printf = mask & (1 << 9) != 0;
        set.int64 = mask & (1 << 10) != 0;
        set.f32_atomic = mask & (1 << 11) != 0;
        for capability in FallbackCapability::ALL {
            let mut trace = FallbackTrace::new();
            let is_native = select_native(capability, &set, "test", &mut trace);
            assert_eq!(is_native, capability.is_native(&set));
            assert_eq!(trace.is_empty(), is_native);
            assert_eq!(trace.len(), usize::from(!is_native));
        }
    }
}

#[test]
fn b10_transient_dual_path_and_pool_exhaustion() {
    let device = MockDevice::with_fallbacks(fallback());
    let mut trace = FallbackTrace::new();
    let mut pool = TransientPool::new();
    // 原生：一次性分配，不占池。
    let decision = transient_alloc(&native(), &mut pool, 64, &mut trace).unwrap();
    assert_eq!(decision, TransientDecision::Native);
    assert!(trace.is_empty());
    assert_eq!(pool.slots(), 0);
    // 退路：从预分配池取；同一槽位可释放复用。
    let reserved = pool.reserve(64);
    let decision = transient_alloc(&fallback(), &mut pool, 64, &mut trace).unwrap();
    assert_eq!(decision, TransientDecision::Pooled(reserved));
    assert_eq!(trace.count(FallbackCapability::TransientAlloc), 1);
    match decision {
        TransientDecision::Pooled(slot) => pool.release(slot),
        TransientDecision::Native => panic!("expected pooled"),
    }
    assert_eq!(pool.free_slots(), 1);
    let reused = transient_alloc(&fallback(), &mut pool, 32, &mut trace).unwrap();
    assert_eq!(reused, TransientDecision::Pooled(reserved));
    // 池耗尽：显式错误（不 panic、不静默）。
    let mut empty = TransientPool::new();
    assert!(matches!(
        transient_alloc(&fallback(), &mut empty, 16, &mut trace),
        Err(BufferError::CapacityExceeded { .. })
    ));
    assert_no_sync(&device);
    assert!(device.state.borrow().submissions.is_empty());
}

#[test]
fn b11_external_borrow_dual_path() {
    let device = MockDevice::with_fallbacks(fallback());
    let allocator = MockAllocator::new(1024);
    let snapshot = [1u8, 2, 3, 4];
    let handle = ExternalHandle(0x1000);
    let mut trace = FallbackTrace::new();
    // 原生：零拷贝（地址等于句柄）。
    let decision =
        borrow_external(&native(), handle, 4, &snapshot, &allocator, &mut trace).unwrap();
    assert_eq!(
        decision,
        BorrowDecision::NativeZeroCopy {
            address: Address64(0x1000)
        }
    );
    assert!(trace.is_empty());
    // 退路：拷贝到独立所有权缓冲（地址不同）。
    let decision =
        borrow_external(&fallback(), handle, 4, &snapshot, &allocator, &mut trace).unwrap();
    match decision {
        BorrowDecision::Copied(copy) => {
            assert_eq!(copy.source, Address64(0x1000));
            assert_eq!(copy.bytes, 4);
            assert_ne!(copy.owned_address, Address64(0x1000));
        }
        BorrowDecision::NativeZeroCopy { .. } => panic!("expected copied"),
    }
    assert_eq!(trace.count(FallbackCapability::ZeroCopyExternal), 1);
    assert_no_sync(&device);
}

#[test]
fn k1_graph_capture_dual_path_records_and_replays() {
    let device = MockDevice::new();
    let queue = device.queue().unwrap();
    let mut trace = FallbackTrace::new();
    // 原生：交由后端图能力，不产生降级记录。
    assert!(select_native(
        FallbackCapability::GraphCapture,
        &native(),
        "sequential-replay",
        &mut trace
    ));
    assert!(trace.is_empty());
    // 退路：录制闭包提交，再逐次重发。
    let recorded = capture_sequential(
        |recording| {
            recording.submit(launch()).unwrap();
            recording.submit(launch()).unwrap();
        },
        &mut trace,
    );
    assert_eq!(recorded.len(), 2);
    let replayed = replay_sequential(queue.as_ref(), &recorded, &mut trace).unwrap();
    assert_eq!(replayed, 2);
    assert_eq!(device.state.borrow().submissions.len(), 2);
    assert_eq!(trace.count(FallbackCapability::GraphCapture), 2);
    assert_no_sync(&device);
}

#[test]
fn k5_capture_alloc_dual_path_and_exhaustion() {
    let allocator = MockAllocator::new(1024);
    let mut pool = CapturePool::new();
    let mut trace = FallbackTrace::new();
    // 原生：录制期直接分配，不消耗池。
    let direct = preallocate_capture(
        &native(),
        &allocator,
        &mut pool,
        16,
        Init::Zeroed,
        &mut trace,
    )
    .unwrap();
    assert!(pool.is_empty());
    assert!(trace.is_empty());
    drop(direct);
    // 退路：必须来自录制前预分配池。
    pool.preallocate(&allocator, 16, Init::Zeroed).unwrap();
    let pooled = preallocate_capture(
        &fallback(),
        &allocator,
        &mut pool,
        16,
        Init::Zeroed,
        &mut trace,
    )
    .unwrap();
    assert_eq!(pool.len(), 0);
    drop(pooled);
    assert_eq!(trace.count(FallbackCapability::CaptureAlloc), 1);
    // 池空：显式错误。
    assert!(matches!(
        preallocate_capture(
            &fallback(),
            &allocator,
            &mut pool,
            16,
            Init::Zeroed,
            &mut trace
        ),
        Err(BufferError::CapacityExceeded { .. })
    ));
}

#[test]
fn k2_device_loop_dual_path() {
    let device = MockDevice::new();
    let queue = device.queue().unwrap();
    let mut trace = FallbackTrace::new();
    // 原生：由后端条件图承担，本参考实现不接管。
    let handled =
        loop_while_device_fallback(&native(), queue.as_ref(), launch, 3, &mut trace).unwrap();
    assert_eq!(handled, 0);
    assert!(device.state.borrow().submissions.is_empty());
    assert!(trace.is_empty());
    // 退路：固定次数发射。
    let handled =
        loop_while_device_fallback(&fallback(), queue.as_ref(), launch, 3, &mut trace).unwrap();
    assert_eq!(handled, 3);
    assert_eq!(device.state.borrow().submissions.len(), 3);
    assert_eq!(trace.count(FallbackCapability::DeviceLoop), 1);
    assert_no_sync(&device);
}

#[test]
fn k3_device_branch_dual_path() {
    let device = MockDevice::new();
    let queue = device.queue().unwrap();
    let mut trace = FallbackTrace::new();
    let handled =
        branch_on_device_fallback(&native(), queue.as_ref(), launch(), launch(), &mut trace)
            .unwrap();
    assert_eq!(handled, 0);
    assert!(device.state.borrow().submissions.is_empty());
    // 退路：两分支都执行（then → else）。
    let handled =
        branch_on_device_fallback(&fallback(), queue.as_ref(), launch(), launch(), &mut trace)
            .unwrap();
    assert_eq!(handled, 2);
    assert_eq!(device.state.borrow().submissions.len(), 2);
    assert_eq!(trace.count(FallbackCapability::DeviceBranch), 1);
    assert_no_sync(&device);
}

#[test]
fn l3_kernel_printf_dual_path() {
    let mut status = PrintfStatusBuffer::new();
    let mut native_log: Vec<String> = Vec::new();
    let mut trace = FallbackTrace::new();
    printf_fallback(
        &native(),
        &mut status,
        &mut native_log,
        "value={}",
        &[Arg::I32(7)],
        &mut trace,
    );
    assert_eq!(native_log, vec!["value=7".to_owned()]);
    assert!(status.is_empty());
    assert!(trace.is_empty());
    printf_fallback(
        &fallback(),
        &mut status,
        &mut native_log,
        "value={}",
        &[Arg::I32(7)],
        &mut trace,
    );
    assert_eq!(status.records(), ["value=7".to_owned()]);
    assert_eq!(status.len(), 1);
    assert_eq!(status.drain().len(), 1);
    assert_eq!(trace.count(FallbackCapability::KernelPrintf), 1);
    assert_eq!(
        format_record("a={} b={} c", &[Arg::U32(1), Arg::F32(2.5)]),
        "a=1 b=2.5 c"
    );
    // 占位符不足/多余都不 panic。
    assert_eq!(format_record("x={} y={}", &[Arg::I32(1)]), "x=1 y={}");
    assert_eq!(format_record("x", &[Arg::U64(9)]), "x 9");
}

#[test]
fn g9_occupancy_hint_dual_path() {
    let mut trace = FallbackTrace::new();
    assert!(select_native(
        FallbackCapability::OccupancyHint,
        &native(),
        "fixed-value",
        &mut trace
    ));
    assert!(trace.is_empty());
    assert!(!select_native(
        FallbackCapability::OccupancyHint,
        &fallback(),
        "fixed-value",
        &mut trace
    ));
    assert_eq!(trace.count(FallbackCapability::OccupancyHint), 1);
    // mock 原生桩 {512,2} vs 冻结退路 {256,1} 的双路由 mock_tests 的 module 测覆盖。
    assert_eq!(BlockSuggestion::FALLBACK.block_width, 256);
}

#[test]
fn c1_int64_dual_path_and_roundtrip() {
    let mut trace = FallbackTrace::new();
    assert_eq!(int64_mode(&native(), &mut trace), Int64Mode::Native);
    assert!(trace.is_empty());
    assert_eq!(int64_mode(&fallback(), &mut trace), Int64Mode::TwoI32);
    assert_eq!(trace.count(FallbackCapability::Int64), 1);
    let values = [
        0i64,
        1,
        -1,
        i32::MAX as i64,
        i32::MIN as i64,
        u32::MAX as i64,
        i64::MAX,
        i64::MIN,
        0x0123_4567_89AB_CDEF,
        -0x0123_4567_89AB_CDEF,
        1i64 << 32,
        -(1i64 << 32),
    ];
    for value in values {
        assert_eq!(join_i64(split_i64(value)), value, "roundtrip {value}");
    }
}

#[test]
fn h1_f32_atomic_dual_path_and_cas_concurrency() {
    let mut trace = FallbackTrace::new();
    let native_cell = F32AtomicCell::new(1.0);
    assert_eq!(
        atomic_add_f32(&native(), &native_cell, 2.0, &mut trace),
        1.0
    );
    assert_eq!(native_cell.load(), 3.0);
    assert!(trace.is_empty());
    let fallback_cell = F32AtomicCell::new(1.0);
    assert_eq!(
        atomic_add_f32(&fallback(), &fallback_cell, 2.0, &mut trace),
        1.0
    );
    assert_eq!(fallback_cell.load(), 3.0);
    assert_eq!(trace.count(FallbackCapability::F32Atomic), 1);
    // CAS 循环并发：32 线程 × 250 次 +1.0，f32 精确（< 2^24）。
    let shared = std::sync::Arc::new(F32AtomicCell::new(0.0));
    std::thread::scope(|scope| {
        for _ in 0..32 {
            let cell = std::sync::Arc::clone(&shared);
            scope.spawn(move || {
                for _ in 0..250 {
                    cell.cas_loop_add(1.0);
                }
            });
        }
    });
    assert_eq!(shared.load(), 8000.0);
}

#[test]
fn host_fallback_paths_do_not_touch_queue_or_sync() {
    // B.10/B.11/K.5/L.3/C.1/H.1 全部走退路，且不接触队列、不隐式同步。
    let device = MockDevice::with_fallbacks(fallback());
    let allocator = MockAllocator::new(1024);
    let mut trace = FallbackTrace::new();
    let mut pool = TransientPool::new();
    pool.reserve(32);
    let _ = transient_alloc(&fallback(), &mut pool, 32, &mut trace).unwrap();
    let _ = borrow_external(
        &fallback(),
        ExternalHandle(0x2000),
        4,
        &[0u8; 4],
        &allocator,
        &mut trace,
    )
    .unwrap();
    let mut capture_pool = CapturePool::new();
    capture_pool
        .preallocate(&allocator, 8, Init::Zeroed)
        .unwrap();
    let _ = preallocate_capture(
        &fallback(),
        &allocator,
        &mut capture_pool,
        8,
        Init::Zeroed,
        &mut trace,
    )
    .unwrap();
    let mut status = PrintfStatusBuffer::new();
    let mut native_log = Vec::new();
    printf_fallback(
        &fallback(),
        &mut status,
        &mut native_log,
        "x={}",
        &[Arg::U64(5)],
        &mut trace,
    );
    assert_eq!(int64_mode(&fallback(), &mut trace), Int64Mode::TwoI32);
    let cell = F32AtomicCell::new(0.0);
    atomic_add_f32(&fallback(), &cell, 1.5, &mut trace);
    assert_no_sync(&device);
    assert!(device.state.borrow().submissions.is_empty());
    // 6 项各留 1 条降级记录。
    assert_eq!(trace.len(), 6);
}
