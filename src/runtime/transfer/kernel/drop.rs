//! 同步内核析构保留未确认模块。

use super::*;

impl Drop for SynchronousKernel {
    fn drop(&mut self) {
        if self
            .session
            .inner
            .status
            .uncertain
            .load(std::sync::atomic::Ordering::Acquire)
        {
            // An unconfirmed launch must retain the module and its context too.
            std::mem::forget(self.function.take());
        }
    }
}
