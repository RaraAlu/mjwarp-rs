//! mock backend（仅 `cfg(test)`）：验证 trait 可组合性、显式错误与同步语义。
//!
//! 这不是任何真实后端的实现：缓冲驻留宿主 `RefCell<Vec<u8>>`，原子/执行原语只做
//! 确定的桩行为；用于在 0 GPU 条件下证明契约面可被实现与使用。

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use super::backend::{Backend, CapabilityId, Device, DeviceCaps, DeviceId, FallbackSet};
use super::buffer::{
    Address64, Allocator, Buffer, DType, ExternalHandle, Init, Pinned, check_range,
};
use super::device::{
    Atomics, Counter, DeviceMath, Diagnostics, Dim3, Event, ExecControl, SharedBuf, ThreadScope,
};
use super::error::{BufferError, CompileError, DeviceError, DeviceLost, LaunchError};
use super::module::{BlockSuggestion, CompileOptions, Kernel, KernelSource, Module};
use super::queue::{Graph, Launch, Queue, QueueCapture, Submission};

const MOCK_MAX_BUFFER_BYTES: u64 = 1024;

pub(crate) struct MockBackend;

impl MockBackend {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Backend for MockBackend {
    fn devices(&self) -> Result<Vec<DeviceId>, DeviceError> {
        Ok(vec![DeviceId(0)])
    }

    fn open(&self, id: &DeviceId) -> Result<Box<dyn Device>, DeviceError> {
        if *id == DeviceId(0) {
            Ok(Box::new(MockDevice::new()))
        } else {
            Err(DeviceError::DeviceLost(DeviceLost::new(format!(
                "unknown device {}",
                id.0
            ))))
        }
    }
}

#[derive(Default)]
pub(crate) struct QueueState {
    pub(crate) submissions: Vec<Launch>,
    pub(crate) idle: bool,
    pub(crate) replays: u32,
    pub(crate) captures: u32,
}

pub(crate) struct MockDevice {
    caps: DeviceCaps,
    pub(crate) state: Rc<RefCell<QueueState>>,
}

impl MockDevice {
    pub(crate) fn new() -> Self {
        Self {
            caps: DeviceCaps {
                fallbacks: FallbackSet::all(),
                max_index_dims: 3,
                max_block_threads: 1024,
                max_blocks: 4096,
                shared_bytes_per_block: 48 * 1024,
                max_buffer_bytes: MOCK_MAX_BUFFER_BYTES,
                numeric: super::backend::NumericPolicy::FROZEN_F32,
            },
            state: Rc::new(RefCell::new(QueueState {
                idle: true,
                ..QueueState::default()
            })),
        }
    }
}

impl Device for MockDevice {
    fn caps(&self) -> DeviceCaps {
        self.caps
    }

    fn queue(&self) -> Result<Box<dyn Queue>, DeviceError> {
        Ok(Box::new(MockQueue {
            state: Rc::clone(&self.state),
        }))
    }

    fn allocator(&self) -> Box<dyn Allocator> {
        Box::new(MockAllocator::new(self.caps.max_buffer_bytes))
    }

    fn module(&self) -> Box<dyn Module> {
        Box::new(MockModule)
    }
}

pub(crate) struct MockAllocator {
    max_bytes: u64,
    next_addr: Cell<u64>,
}

impl MockAllocator {
    pub(crate) fn new(max_bytes: u64) -> Self {
        Self {
            max_bytes,
            next_addr: Cell::new(1),
        }
    }

    fn make(&self, bytes: u64, init: Init) -> Result<Box<dyn Buffer>, BufferError> {
        if bytes > self.max_bytes {
            return Err(BufferError::CapacityExceeded {
                requested: bytes,
                capacity: self.max_bytes,
            });
        }
        let addr = Address64(self.next_addr.get());
        self.next_addr.set(self.next_addr.get() + 1);
        Ok(Box::new(MockBuffer::new(addr, bytes, init)))
    }
}

impl Allocator for MockAllocator {
    fn alloc(&self, dtype: DType, len: u64, init: Init) -> Result<Box<dyn Buffer>, BufferError> {
        let bytes = dtype
            .byte_size()
            .checked_mul(len)
            .ok_or(BufferError::CapacityExceeded {
                requested: u64::MAX,
                capacity: self.max_bytes,
            })?;
        self.make(bytes, init)
    }

    fn transient(&self, bytes: u64) -> Result<Box<dyn Buffer>, BufferError> {
        self.make(bytes, Init::Uninit)
    }

    fn borrow_external(&self, handle: ExternalHandle) -> Result<Box<dyn Buffer>, BufferError> {
        Ok(Box::new(MockBuffer::new(
            Address64(handle.0),
            16,
            Init::Zeroed,
        )))
    }
}

pub(crate) struct MockBuffer {
    addr: Address64,
    data: RefCell<Vec<u8>>,
}

impl MockBuffer {
    pub(crate) fn new(addr: Address64, bytes: u64, init: Init) -> Self {
        let len = usize::try_from(bytes).unwrap_or(0);
        let mut data = vec![0u8; len];
        match init {
            Init::Uninit | Init::Zeroed => {}
            Init::Fill(pattern) => {
                for (i, byte) in data.iter_mut().enumerate() {
                    *byte = (pattern >> ((i % 8) * 8)) as u8;
                }
            }
        }
        Self {
            addr,
            data: RefCell::new(data),
        }
    }
}

impl Buffer for MockBuffer {
    fn addr(&self) -> Address64 {
        self.addr
    }

    fn byte_len(&self) -> u64 {
        self.data.borrow().len() as u64
    }

    fn copy_range(
        &self,
        dst: &dyn Buffer,
        dst_off: u64,
        src_off: u64,
        len: u64,
    ) -> Result<(), BufferError> {
        check_range(src_off, len, self.byte_len())?;
        check_range(dst_off, len, dst.byte_len())?;
        let start = src_off as usize;
        let end = start + len as usize;
        let chunk = self.data.borrow()[start..end].to_vec();
        dst.upload(&chunk, dst_off)
    }

    fn upload(&self, host: &[u8], off: u64) -> Result<(), BufferError> {
        check_range(off, host.len() as u64, self.byte_len())?;
        let start = off as usize;
        self.data.borrow_mut()[start..start + host.len()].copy_from_slice(host);
        Ok(())
    }

    fn readback(&self, host: &mut [u8], off: u64) -> Result<(), BufferError> {
        check_range(off, host.len() as u64, self.byte_len())?;
        let start = off as usize;
        host.copy_from_slice(&self.data.borrow()[start..start + host.len()]);
        Ok(())
    }

    fn pin(&self) -> Result<Pinned, BufferError> {
        Ok(Pinned { address: self.addr })
    }
}

pub(crate) struct MockQueue {
    pub(crate) state: Rc<RefCell<QueueState>>,
}

impl Queue for MockQueue {
    fn submit(&self, launch: Launch) -> Result<Submission, DeviceError> {
        let mut state = self.state.borrow_mut();
        state.submissions.push(launch);
        state.idle = false;
        Ok(Submission(state.submissions.len() as u64))
    }

    fn wait_idle(&self) -> Result<(), DeviceError> {
        self.state.borrow_mut().idle = true;
        Ok(())
    }

    fn replay(&self, _graph: &Graph) -> Result<(), DeviceError> {
        self.state.borrow_mut().replays += 1;
        Ok(())
    }

    fn capture_alloc(&self) -> Result<Box<dyn Buffer>, DeviceError> {
        Ok(Box::new(MockBuffer::new(
            Address64(0xffff),
            16,
            Init::Zeroed,
        )))
    }
}

impl QueueCapture for MockQueue {
    fn capture<F>(&self, body: F) -> Result<Graph, DeviceError>
    where
        F: FnOnce(&dyn Queue),
    {
        let mut state = self.state.borrow_mut();
        state.captures += 1;
        drop(state);
        body(self);
        Ok(Graph(self.state.borrow().captures as u64))
    }
}

pub(crate) struct MockModule;

impl Module for MockModule {
    fn compile(&self, src: KernelSource, opts: CompileOptions) -> Result<Kernel, DeviceError> {
        match src {
            KernelSource::Named("") => Err(DeviceError::Compile(CompileError::new(
                "empty kernel source name",
            ))),
            KernelSource::Inline("") => Err(DeviceError::Compile(CompileError::new(
                "empty inline kernel source",
            ))),
            KernelSource::Inline(_) => {
                if opts.entry.is_none() {
                    return Err(DeviceError::Compile(CompileError::new(
                        "inline kernel source requires an entry name",
                    )));
                }
                Ok(Kernel::from_handle(42))
            }
            KernelSource::Named(name) => {
                let _ = name;
                Ok(Kernel::from_handle(42))
            }
        }
    }

    fn suggest_block_width(
        &self,
        _kernel: &Kernel,
        _work_items: u64,
    ) -> Result<BlockSuggestion, DeviceError> {
        Ok(BlockSuggestion::FALLBACK)
    }
}

pub(crate) struct MockDiagnostics {
    events: Cell<u64>,
    pub(crate) logs: RefCell<Vec<String>>,
}

impl MockDiagnostics {
    pub(crate) fn new() -> Self {
        Self {
            events: Cell::new(0),
            logs: RefCell::new(Vec::new()),
        }
    }
}

impl Diagnostics for MockDiagnostics {
    fn record_event(&self) -> Result<Event, DeviceError> {
        self.events.set(self.events.get() + 1);
        Ok(Event {
            handle: self.events.get(),
        })
    }

    fn elapsed(&self, a: &Event, b: &Event) -> Duration {
        Duration::from_micros(b.handle.saturating_sub(a.handle))
    }

    fn printf(&self, fmt: &str, _args: &[super::queue::Arg]) {
        self.logs.borrow_mut().push(fmt.to_string());
    }
}

pub(crate) struct MockDeviceMath;

impl DeviceMath for MockDeviceMath {
    fn abs(x: f32) -> f32 {
        x.abs()
    }
    fn max(a: f32, b: f32) -> f32 {
        a.max(b)
    }
    fn min(a: f32, b: f32) -> f32 {
        a.min(b)
    }
    fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
        x.clamp(lo, hi)
    }
    fn sign(x: f32) -> f32 {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            0.0
        }
    }
    fn pow(x: f32, y: f32) -> f32 {
        x.powf(y)
    }
    fn exp(x: f32) -> f32 {
        x.exp()
    }
    fn log(x: f32) -> f32 {
        x.ln()
    }
    fn floor(x: f32) -> f32 {
        x.floor()
    }
    fn ceil(x: f32) -> f32 {
        x.ceil()
    }
    fn round(x: f32) -> f32 {
        x.round()
    }
    fn isnan(x: f32) -> bool {
        x.is_nan()
    }
    fn sqrt(x: f32) -> f32 {
        x.sqrt()
    }
    fn sin(x: f32) -> f32 {
        x.sin()
    }
    fn cos(x: f32) -> f32 {
        x.cos()
    }
    fn tan(x: f32) -> f32 {
        x.tan()
    }
    fn asin(x: f32) -> f32 {
        x.asin()
    }
    fn acos(x: f32) -> f32 {
        x.acos()
    }
    fn atan2(y: f32, x: f32) -> f32 {
        y.atan2(x)
    }
}

pub(crate) struct MockAtomics;

impl Atomics for MockAtomics {
    fn add_i32(&self, _target: Address64, value: i32) -> i32 {
        value
    }
    fn add_f32(&self, _target: Address64, value: f32) -> f32 {
        value
    }
    fn sub_i32(&self, _target: Address64, value: i32) -> i32 {
        value
    }
    fn or_u32(&self, _target: Address64, value: u32) -> u32 {
        value
    }
    fn min_i32(&self, _target: Address64, value: i32) -> i32 {
        value
    }
    fn max_i32(&self, _target: Address64, value: i32) -> i32 {
        value
    }
    fn cas_i32(&self, _target: Address64, compare: i32, _value: i32) -> i32 {
        compare
    }
}

pub(crate) struct MockThreadScope {
    pub(crate) barriers: Cell<u32>,
    /// 块共享存储容量（0 = 不提供块共享存储）。
    pub(crate) shared_bytes: u64,
}

impl MockThreadScope {
    pub(crate) fn new() -> Self {
        Self::with_shared_bytes(48 * 1024)
    }

    pub(crate) fn with_shared_bytes(shared_bytes: u64) -> Self {
        Self {
            barriers: Cell::new(0),
            shared_bytes,
        }
    }
}

impl ThreadScope for MockThreadScope {
    fn tid(&self) -> Dim3 {
        Dim3::new(0, 0, 0)
    }
    fn block_dim(&self) -> Dim3 {
        Dim3::new(1, 1, 1)
    }
    fn shared(&self, bytes: u64) -> Result<SharedBuf, DeviceError> {
        if self.shared_bytes == 0 {
            return Err(DeviceError::Unsupported(CapabilityId("I.2")));
        }
        if bytes > self.shared_bytes {
            return Err(DeviceError::Launch(LaunchError::InvalidConfig(format!(
                "shared bytes {bytes} exceed capacity {}",
                self.shared_bytes
            ))));
        }
        Ok(SharedBuf {
            address: Address64(0x5A00_0000 + bytes),
        })
    }
    fn barrier(&self) {
        self.barriers.set(self.barriers.get() + 1);
    }
}

pub(crate) struct MockExecControl;

impl ExecControl for MockExecControl {
    fn loop_while_device(
        &self,
        _counter: &Counter,
        mut body: impl FnMut(),
    ) -> Result<(), DeviceError> {
        body();
        Ok(())
    }

    fn branch_on_device(
        &self,
        counter: &Counter,
        then: impl FnOnce(),
        els: impl FnOnce(),
    ) -> Result<(), DeviceError> {
        if counter.handle.is_multiple_of(2) {
            then();
        } else {
            els();
        }
        Ok(())
    }
}
