//! K.1 / K.5 退路：逐次提交（录制-重放）与录制前预分配。
//!
//! - K.1：`graph_capture=false` 时 `capture` 退化为**记录闭包内提交并逐次执行**，
//!   `replay` 退化为**逐次重发**；语义等价但无图优化（降级留痕）。
//! - K.5：`capture_alloc=false` 时录制期分配退化为**录制前预分配**（池耗尽显式报错）。

use std::cell::RefCell;

use crate::runtime::kernel_exec::backend::FallbackSet;
use crate::runtime::kernel_exec::buffer::{Allocator, Buffer, DType, Init};
use crate::runtime::kernel_exec::error::{BufferError, DeviceError, LaunchError};
use crate::runtime::kernel_exec::queue::{Graph, Launch, Queue, Submission};

use super::{FallbackCapability, FallbackTrace, select_native};

/// K.1 若走退路，一次 `capture` 记录到的发射序列。
#[derive(Clone, Debug, Default)]
pub struct RecordedGraph {
    launches: Vec<Launch>,
}

impl RecordedGraph {
    /// 录到的发射次数。
    pub fn len(&self) -> usize {
        self.launches.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.launches.is_empty()
    }

    /// 录到的发射序列（只读）。
    pub fn launches(&self) -> &[Launch] {
        &self.launches
    }
}

/// K.1 退路录制队列：只记录 `submit`，不执行。
///
/// 这是宿主侧记账对象，不是后端队列；`replay`/`capture_alloc` 明确报错（不会返回
/// `Unsupported`——退路本身必须可走，错误只表示"录队列不能承担该操作"）。
pub struct RecordingQueue {
    launches: RefCell<Vec<Launch>>,
}

impl RecordingQueue {
    /// 空录制队列。
    pub fn new() -> Self {
        Self {
            launches: RefCell::new(Vec::new()),
        }
    }

    fn invalid(action: &'static str) -> DeviceError {
        DeviceError::Launch(LaunchError::InvalidConfig(format!(
            "recording queue cannot {action}"
        )))
    }
}

impl Default for RecordingQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Queue for RecordingQueue {
    fn submit(&self, launch: Launch) -> Result<Submission, DeviceError> {
        let mut launches = self.launches.borrow_mut();
        launches.push(launch);
        Ok(Submission(launches.len() as u64))
    }

    fn wait_idle(&self) -> Result<(), DeviceError> {
        Ok(())
    }

    fn replay(&self, _graph: &Graph) -> Result<(), DeviceError> {
        Err(Self::invalid("replay"))
    }

    fn capture_alloc(&self) -> Result<Box<dyn Buffer>, DeviceError> {
        Err(Self::invalid("allocate during recording"))
    }
}

/// K.1 退路 `capture`：在录制队列上跑闭包，记录其中的提交。
pub fn capture_sequential<F>(body: F, trace: &mut FallbackTrace) -> RecordedGraph
where
    F: FnOnce(&dyn Queue),
{
    let recording = RecordingQueue::new();
    body(&recording);
    trace.record(FallbackCapability::GraphCapture, "sequential-replay");
    RecordedGraph {
        launches: recording.launches.borrow().clone(),
    }
}

/// K.1 退路 `replay`：把录到的发射逐次重发到真实队列；返回重发次数。
pub fn replay_sequential(
    queue: &dyn Queue,
    recorded: &RecordedGraph,
    trace: &mut FallbackTrace,
) -> Result<usize, DeviceError> {
    for launch in recorded.launches() {
        queue.submit(launch.clone())?;
    }
    trace.record(FallbackCapability::GraphCapture, "sequential-replay");
    Ok(recorded.len())
}

/// K.5 退路载体：录制前预分配池。
#[derive(Default)]
pub struct CapturePool {
    preallocated: Vec<Box<dyn Buffer>>,
}

impl CapturePool {
    /// 空池。
    pub fn new() -> Self {
        Self {
            preallocated: Vec::new(),
        }
    }

    /// 预分配一个缓冲并放入池（录制前调用）。
    pub fn preallocate(
        &mut self,
        allocator: &dyn Allocator,
        bytes: u64,
        init: Init,
    ) -> Result<(), BufferError> {
        let buffer = allocator.alloc(DType::U8, bytes, init)?;
        self.preallocated.push(buffer);
        Ok(())
    }

    /// 池内缓冲数。
    pub fn len(&self) -> usize {
        self.preallocated.len()
    }

    /// 池是否为空。
    pub fn is_empty(&self) -> bool {
        self.preallocated.is_empty()
    }
}

/// K.5 选择：`true` → 录制期直接分配；`false` → 取录制前预分配缓冲（池空显式报错）。
pub fn preallocate_capture(
    fallbacks: &FallbackSet,
    allocator: &dyn Allocator,
    pool: &mut CapturePool,
    bytes: u64,
    init: Init,
    trace: &mut FallbackTrace,
) -> Result<Box<dyn Buffer>, BufferError> {
    if select_native(
        FallbackCapability::CaptureAlloc,
        fallbacks,
        "preallocated-before-capture",
        trace,
    ) {
        return allocator.alloc(DType::U8, bytes, init);
    }
    match pool.preallocated.pop() {
        Some(buffer) => Ok(buffer),
        None => Err(BufferError::CapacityExceeded {
            requested: bytes,
            capacity: 0,
        }),
    }
}
