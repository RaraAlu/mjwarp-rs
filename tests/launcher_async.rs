//! M5：异步提交凭证与访问依赖账本的宿主验证。
//! 不发射内核；用模拟栅栏与保留资源核对 W06/W07/W08/W20。

use mjwarp_rs::runtime::launcher::{
    AccessMode, BufferAccess, Fence, LauncherError, RetainedResources, StreamKey, SubmissionId,
    SubmissionState, SubmitQueue,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Default)]
struct FenceState {
    ready: bool,
    waits: usize,
    error: Option<LauncherError>,
}

struct MockFence {
    state: Arc<Mutex<FenceState>>,
}

impl MockFence {
    fn new() -> (Self, Arc<Mutex<FenceState>>) {
        let state = Arc::new(Mutex::new(FenceState::default()));
        (
            Self {
                state: Arc::clone(&state),
            },
            state,
        )
    }
}

impl Fence for MockFence {
    fn query(&self) -> Result<bool, LauncherError> {
        let state = self.state.lock().unwrap();
        if let Some(error) = &state.error {
            return Err(error.clone());
        }
        Ok(state.ready)
    }

    fn wait(&self) -> Result<(), LauncherError> {
        let mut state = self.state.lock().unwrap();
        state.waits += 1;
        if let Some(error) = &state.error {
            return Err(error.clone());
        }
        state.ready = true;
        Ok(())
    }
}

fn mock_error(stage: &'static str) -> LauncherError {
    LauncherError::Driver(stage.to_owned())
}

#[derive(Debug)]
struct Tracked {
    alive: Arc<AtomicBool>,
    quarantined: Arc<AtomicBool>,
}

impl Tracked {
    fn new() -> (Self, Arc<AtomicBool>, Arc<AtomicBool>) {
        let alive = Arc::new(AtomicBool::new(true));
        let quarantined = Arc::new(AtomicBool::new(false));
        (
            Self {
                alive: Arc::clone(&alive),
                quarantined: Arc::clone(&quarantined),
            },
            alive,
            quarantined,
        )
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
    }
}

impl RetainedResources for Tracked {
    fn quarantine(&self) {
        self.quarantined.store(true, Ordering::SeqCst);
    }
}

type Queue = SubmitQueue<MockFence, Tracked>;

fn stream(id: u64) -> StreamKey {
    StreamKey::new(0, 1, id)
}

fn write(range: std::ops::Range<u64>) -> BufferAccess {
    BufferAccess::write(0, 1, range).unwrap()
}

fn read(range: std::ops::Range<u64>) -> BufferAccess {
    BufferAccess::read(0, 1, range).unwrap()
}

fn submit(
    queue: &mut Queue,
    stream: StreamKey,
    accesses: &[BufferAccess],
    dependencies: &[SubmissionId],
) -> (SubmissionId, Arc<AtomicBool>, Arc<AtomicBool>) {
    let (fence, _) = MockFence::new();
    let (tracked, alive, quarantined) = Tracked::new();
    let id = queue
        .submit(stream, accesses, dependencies, tracked, fence)
        .unwrap();
    (id, alive, quarantined)
}

#[test]
fn credential_retains_resources_until_wait() {
    let mut queue = Queue::new();
    let (fence, fence_state) = MockFence::new();
    let (tracked, alive, _) = Tracked::new();
    let id = queue
        .submit(stream(1), &[write(0..64)], &[], tracked, fence)
        .unwrap();
    assert_eq!(queue.outstanding(), 1);
    assert_eq!(queue.state(id).unwrap(), SubmissionState::Pending);
    assert!(!queue.query(id).unwrap());
    assert!(alive.load(Ordering::SeqCst));
    fence_state.lock().unwrap().ready = true;
    // 查询完成不释放资源。
    assert!(queue.query(id).unwrap());
    assert_eq!(queue.state(id).unwrap(), SubmissionState::Completed);
    assert!(alive.load(Ordering::SeqCst));
    drop(queue.wait(id).unwrap());
    assert!(!alive.load(Ordering::SeqCst));
    assert_eq!(queue.outstanding(), 0);
}

#[test]
fn same_stream_accesses_are_ordered_without_dependencies() {
    let mut queue = Queue::new();
    submit(&mut queue, stream(1), &[write(0..64)], &[]);
    submit(&mut queue, stream(1), &[read(0..64)], &[]);
    submit(&mut queue, stream(1), &[write(32..96)], &[]);
    assert_eq!(queue.outstanding(), 3);
}

#[test]
fn cross_stream_conflicts_require_covered_dependencies() {
    let mut queue = Queue::new();
    let (first, _, _) = submit(&mut queue, stream(1), &[write(0..64)], &[]);
    // 同区间不同流且无依赖：拒绝。
    let (fence, _) = MockFence::new();
    let (tracked, alive, _) = Tracked::new();
    let error = queue
        .submit(stream(2), &[read(0..64)], &[], tracked, fence)
        .unwrap_err();
    assert_eq!(
        error,
        LauncherError::AccessConflict {
            blocking: first.value()
        }
    );
    // 校验失败不保留资源。
    assert!(!alive.load(Ordering::SeqCst));
    // 区间不重叠：放行。
    submit(&mut queue, stream(2), &[read(64..128)], &[]);
    // 声明依赖：放行。
    submit(&mut queue, stream(2), &[read(0..64)], &[first]);
    assert_eq!(queue.outstanding(), 3);
}

#[test]
fn device_context_and_range_participate_in_conflicts() {
    let mut queue = Queue::new();
    let (first, _, _) = submit(&mut queue, stream(1), &[write(0..64)], &[]);
    // 另一设备与另一上下文不构成冲突。
    let other_device = BufferAccess::read(1, 1, 0..64).unwrap();
    let other_context = BufferAccess::read(0, 2, 0..64).unwrap();
    submit(&mut queue, stream(2), &[other_device], &[]);
    submit(&mut queue, stream(2), &[other_context], &[]);
    // 同设备同上下文只读重叠不构成冲突。
    submit(&mut queue, stream(1), &[read(96..160)], &[]);
    let read_read = BufferAccess::read(0, 1, 96..160).unwrap();
    submit(&mut queue, stream(2), &[read_read], &[]);
    assert_eq!(queue.outstanding(), 5);
    assert_eq!(first.value(), 0);
}

#[test]
fn dependency_closure_covers_transitive_conflicts() {
    let mut queue = Queue::new();
    let first_write = submit(&mut queue, stream(1), &[write(0..64)], &[]).0;
    let later = submit(&mut queue, stream(1), &[write(0..64)], &[]).0;
    // 只依赖前者时仍与后者冲突。
    let (fence, _) = MockFence::new();
    let (tracked, _, _) = Tracked::new();
    assert_eq!(
        queue
            .submit(stream(2), &[read(0..64)], &[first_write], tracked, fence)
            .unwrap_err(),
        LauncherError::AccessConflict {
            blocking: later.value()
        }
    );
    // 显式声明两者后放行。
    submit(&mut queue, stream(2), &[read(0..64)], &[first_write, later]);
    // 经中间提交传递依赖：中间提交声明两者，读取者依赖中间提交。
    let middle = submit(&mut queue, stream(3), &[], &[first_write, later]).0;
    let reader = submit(&mut queue, stream(4), &[read(0..64)], &[middle]).0;
    assert_eq!(reader.value(), 4);
    assert_eq!(queue.outstanding(), 5);
}

#[test]
fn deferred_errors_are_attributed_and_isolated() {
    let mut queue = Queue::new();
    let (failing_fence, failing_state) = MockFence::new();
    let (failing_tracked, _, failing_quarantined) = Tracked::new();
    let failing = queue
        .submit(
            stream(1),
            &[write(0..64)],
            &[],
            failing_tracked,
            failing_fence,
        )
        .unwrap();
    let (healthy_id, _, _) = submit(&mut queue, stream(2), &[write(128..192)], &[]);
    failing_state.lock().unwrap().error = Some(mock_error("inject"));
    assert_eq!(queue.query(failing).unwrap_err(), mock_error("inject"));
    assert_eq!(queue.state(failing).unwrap(), SubmissionState::Failed);
    assert!(failing_quarantined.load(Ordering::SeqCst));
    // 无关提交不受影响。
    let (fence, state) = MockFence::new();
    state.lock().unwrap().ready = true;
    let (tracked, alive, _) = Tracked::new();
    let id = queue
        .submit(stream(3), &[write(256..320)], &[], tracked, fence)
        .unwrap();
    assert!(queue.query(id).unwrap());
    queue.wait(id).unwrap();
    assert!(!alive.load(Ordering::SeqCst));
    // 失败提交的错误保持可复现。
    assert_eq!(queue.wait(failing).unwrap_err(), mock_error("inject"));
    assert_eq!(queue.state(healthy_id).unwrap(), SubmissionState::Pending);
}

#[test]
fn dependents_fault_with_upstream_error() {
    let mut queue = Queue::new();
    let (fence, state) = MockFence::new();
    let (tracked, _, tracked_quarantined) = Tracked::new();
    let upstream = queue
        .submit(stream(1), &[write(0..64)], &[], tracked, fence)
        .unwrap();
    let (dependent, _, dependent_quarantined) =
        submit(&mut queue, stream(2), &[read(0..64)], &[upstream]);
    state.lock().unwrap().error = Some(mock_error("device-loss"));
    assert_eq!(
        queue.query(dependent).unwrap_err(),
        LauncherError::FaultedUpstream(upstream.value())
    );
    assert!(dependent_quarantined.load(Ordering::SeqCst));
    assert!(tracked_quarantined.load(Ordering::SeqCst));
    assert_eq!(queue.wait(upstream).unwrap_err(), mock_error("device-loss"));
}

#[test]
fn rejects_unknown_duplicate_and_failed_dependencies() {
    let mut queue = Queue::new();
    let (fence, _) = MockFence::new();
    let (tracked, _, _) = Tracked::new();
    let ghost = SubmissionId::from_value(7);
    let error = queue
        .submit(stream(1), &[write(0..64)], &[ghost], tracked, fence)
        .unwrap_err();
    assert_eq!(error, LauncherError::UnknownSubmission(7));
    let first = submit(&mut queue, stream(1), &[write(0..64)], &[]).0;
    let (fence, _) = MockFence::new();
    let (tracked, _, _) = Tracked::new();
    assert_eq!(
        queue
            .submit(
                stream(1),
                &[write(64..128)],
                &[first, first],
                tracked,
                fence
            )
            .unwrap_err(),
        LauncherError::DuplicateDependency(first.value())
    );
    queue.fail(first, mock_error("submit")).unwrap();
    let (fence, _) = MockFence::new();
    let (tracked, _, _) = Tracked::new();
    assert_eq!(
        queue
            .submit(stream(1), &[write(128..192)], &[first], tracked, fence)
            .unwrap_err(),
        LauncherError::FaultedUpstream(first.value())
    );
}

#[test]
fn submission_failure_quarantines_until_drain() {
    let mut queue = Queue::new();
    let (fence, state) = MockFence::new();
    let (tracked, alive, quarantined) = Tracked::new();
    let id = queue
        .submit(stream(1), &[write(0..64)], &[], tracked, fence)
        .unwrap();
    let error = mock_error("enqueue");
    queue.fail(id, error.clone()).unwrap();
    assert_eq!(queue.query(id).unwrap_err(), error);
    assert_eq!(queue.wait(id).unwrap_err(), error);
    assert!(quarantined.load(Ordering::SeqCst));
    assert!(alive.load(Ordering::SeqCst));
    assert_eq!(queue.outstanding(), 1);
    state.lock().unwrap().ready = true;
    drop(queue);
}

#[test]
fn drain_waits_pending_fences_and_quarantines_failures() {
    let mut queue = Queue::new();
    let (fence, state) = MockFence::new();
    let (tracked, alive, quarantined) = Tracked::new();
    state.lock().unwrap().error = Some(mock_error("drain"));
    queue
        .submit(stream(1), &[write(0..64)], &[], tracked, fence)
        .unwrap();
    drop(queue);
    assert!(quarantined.load(Ordering::SeqCst));
    // 无法证明完成时保守保留资源。
    assert!(alive.load(Ordering::SeqCst));
}

#[test]
fn empty_and_inverted_access_ranges_are_rejected() {
    assert!(BufferAccess::read(0, 1, 8..8).is_err());
    let (inverted_start, inverted_end) = (9u64, 8u64);
    assert!(BufferAccess::write(0, 1, inverted_start..inverted_end).is_err());
    let mut queue = Queue::new();
    let (fence, _) = MockFence::new();
    let (tracked, _, _) = Tracked::new();
    let id = queue.submit(stream(1), &[], &[], tracked, fence).unwrap();
    assert_eq!(queue.state(id).unwrap(), SubmissionState::Pending);
    assert_eq!(write(0..1).mode(), AccessMode::Write);
    assert_eq!(read(0..1).mode(), AccessMode::Read);
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires an NVIDIA GPU and NVRTC 12.8"]
fn gpu_async_submit_completes_and_retains_the_module() {
    use cudarc::driver::{CudaContext, DevicePtr};
    use mjwarp_rs::runtime::launcher::{
        ArtifactCache, CompileRecipe, KernelHandle, L2Completion, LaunchBounds2, ModuleCache,
        NvrtcSource, RecipePurpose, WarpBounds, WarpLaunch, f32_arg,
    };

    const FILL: &str = r#"
struct Bounds2 { int shape[2]; unsigned long long size; unsigned long long coord_mult; };
extern "C" __global__ void fill_bounds(Bounds2 bounds, float* out) {
    int index = threadIdx.x;
    if (index < bounds.shape[0]) {
        out[index] = 1.0f;
    }
}
"#;
    let artifacts = ArtifactCache::in_memory();
    let source = NvrtcSource::new("async_fill", FILL).unwrap();
    let artifact = artifacts
        .compile(
            &source,
            &CompileRecipe::WARP_1_15_DIAGNOSTIC,
            RecipePurpose::Diagnostic,
        )
        .unwrap();
    let context = CudaContext::new(0).unwrap();
    let modules = ModuleCache::new(context.clone());
    let handle = modules
        .function(&context, &artifact, "fill_bounds")
        .unwrap();
    let stream = context.default_stream();
    let out = stream.alloc_zeros::<f32>(64).unwrap();
    let (pointer, guard) = out.device_ptr(&stream);
    drop(guard);
    let arg = f32_arg(pointer, &[32], 4, 64 * 4).unwrap();
    let bounds = WarpBounds::Two(LaunchBounds2::new([32, 1]).unwrap());
    let launch = WarpLaunch {
        bounds,
        args: &[arg],
    };
    let completion = handle.submit_l2(&stream, &launch).unwrap();
    let mut queue: SubmitQueue<L2Completion, KernelHandle> = SubmitQueue::new();
    let key = StreamKey::new(0, 0, 1);
    let id = queue
        .submit(key, &[write(0..64 * 4)], &[], handle.clone(), completion)
        .unwrap();
    assert_eq!(queue.state(id).unwrap(), SubmissionState::Pending);
    // 凭证在等待前持有句柄与流；等待后模块随句柄归还。
    let module = queue.wait(id).unwrap();
    assert_eq!(module.entry(), "fill_bounds");
    let values = stream.clone_dtoh(&out).unwrap();
    assert!(values[..32].iter().all(|value| *value == 1.0));
    assert!(values[32..].iter().all(|value| *value == 0.0));
}
