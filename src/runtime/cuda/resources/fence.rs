//! 完成栅栏：查询与等待 CUDA 事件。

use super::super::cuda_error;
use crate::{diagnostics::ProbeError, runtime::completion::Fence};
use cudarc::driver::{CudaEvent, CudaStream, result, sys};
use std::sync::Arc;

pub(in crate::runtime::cuda) struct CudaFence {
    pub(in crate::runtime::cuda) stream: Arc<CudaStream>,
    pub(in crate::runtime::cuda) event: Option<CudaEvent>,
}

impl CudaFence {
    pub(in crate::runtime::cuda) fn arm(&mut self) -> Result<(), ProbeError> {
        self.event = Some(
            self.stream
                .record_event(None)
                .map_err(cuda_error("lease-record"))?,
        );
        Ok(())
    }
}

pub(super) fn event_result(
    result: Result<(), cudarc::driver::DriverError>,
) -> Result<bool, ProbeError> {
    match result {
        Ok(()) => Ok(true),
        Err(error) if error.0 == sys::cudaError_enum::CUDA_ERROR_NOT_READY => Ok(false),
        Err(error) => Err(cuda_error("lease-query")(error)),
    }
}

impl Fence for CudaFence {
    fn query(&self) -> Result<bool, ProbeError> {
        let Some(event) = &self.event else {
            // CUDA将未记录事件视为完成。
            // 本队列将未提交任务视为待定。
            return Ok(false);
        };
        self.stream
            .context()
            .bind_to_thread()
            .map_err(cuda_error("lease-query-context"))?;
        // SAFETY: 栅栏独占此事件。
        // 队列保留事件所属上下文。
        event_result(unsafe { result::event::query(event.cu_event()) })
    }
    fn wait(&self) -> Result<(), ProbeError> {
        if let Some(event) = &self.event {
            event.synchronize().map_err(cuda_error("lease-wait"))?;
        } else {
            // 部分提交失败也等待已排入工作。
            self.stream
                .synchronize()
                .map_err(cuda_error("lease-unarmed-wait"))?;
        }
        self.stream
            .context()
            .check_err()
            .map_err(cuda_error("lease-context-status"))
    }
}
