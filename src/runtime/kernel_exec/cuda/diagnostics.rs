//! CUDA 诊断（L.2/L.3，P1-b）：设备事件计时；内核打印（L.3）未接线。
//!
//! L2 语义：`record_event` 创建**启用计时**的事件并在当前流上记录；`elapsed` 用驱动事件差算
//! 耗时（`CudaEvent::elapsed_ms` 内部会同步两端事件——测量语义本身要求如此，与
//! `upload/transient` 的"不得隐式同步"约束不冲突）。
//!
//! 未知/异域事件句柄：`elapsed` 返回 `Duration::ZERO` 并在文档写明（P1-a 签名无错误通道；
//! 仅接受本对象产生的事件）。`record_event` 失败必须显式返回 `DeviceLost`（P1-b 冻结）。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cudarc::driver::CudaEvent;
use cudarc::driver::sys;

use crate::runtime::kernel_exec::device::{Diagnostics, Event};
use crate::runtime::kernel_exec::error::DeviceError;
use crate::runtime::kernel_exec::queue::Arg;

use super::errors;
use super::{CudaDeviceInner, lock};

/// 诊断句柄：持有本设备的事件表。
pub struct CudaDiagnostics {
    dev: Arc<CudaDeviceInner>,
    next: AtomicU64,
    events: Mutex<HashMap<u64, Arc<CudaEvent>>>,
}

impl CudaDiagnostics {
    pub(crate) fn new(dev: Arc<CudaDeviceInner>) -> Self {
        Self {
            dev,
            next: AtomicU64::new(1),
            events: Mutex::new(HashMap::new()),
        }
    }

    fn event(&self, handle: u64) -> Option<Arc<CudaEvent>> {
        lock(&self.events).get(&handle).cloned()
    }
}

impl Diagnostics for CudaDiagnostics {
    fn record_event(&self) -> Result<Event, DeviceError> {
        self.dev
            .context
            .bind_to_thread()
            .map_err(|error| errors::device_lost("cuda-event-bind", error))?;
        // 默认 flags 会禁用计时；L.2 需要计时，显式请求 CU_EVENT_DEFAULT。
        let event = self
            .dev
            .context
            .new_event(Some(sys::CUevent_flags::CU_EVENT_DEFAULT))
            .map_err(|error| errors::device_lost("cuda-event-create", error))?;
        event
            .record(&self.dev.stream)
            .map_err(|error| errors::device_lost("cuda-event-record", error))?;
        let handle = self.next.fetch_add(1, Ordering::Relaxed);
        lock(&self.events).insert(handle, Arc::new(event));
        Ok(Event { handle })
    }

    fn elapsed(&self, a: &Event, b: &Event) -> Duration {
        let (Some(start), Some(end)) = (self.event(a.handle), self.event(b.handle)) else {
            return Duration::ZERO;
        };
        match start.elapsed_ms(&end) {
            Ok(millis) if millis.is_finite() && millis > 0.0 => {
                Duration::from_secs_f32(millis / 1000.0)
            }
            _ => Duration::ZERO,
        }
    }

    fn printf(&self, fmt: &str, args: &[Arg]) {
        // L3：P1-b 未接线（状态缓冲回写属 P1-c）。此处显式 no-op：不分配、不打印、不等待。
        let _ = (fmt, args);
    }
}
