//! 外部任务保留、发射与图捕获。

use std::sync::Arc;

use cudarc::driver::{CudaStream, sys};

use crate::diagnostics::ProbeError;
use crate::runtime::ProbeBackend;
use crate::runtime::completion::RetainedResources;
use crate::runtime::lease::BufferLease;

use super::super::resources::launch_affine_raw;
use super::super::{ProbeGraph, TrackingPause, cuda_error};
use super::{ImportedOwner, Task};

impl RetainedResources for Task {
    fn quarantine(&self) {
        self.input.quarantine();
        self.output.quarantine();
        if let Some(warmup) = &self.warmup {
            warmup.quarantine();
        }
    }
}

impl Task {
    pub(super) fn wait_producers(&self, stream: &CudaStream) -> Result<usize, ProbeError> {
        let mut count = 0;
        for lease in [&self.input, &self.output]
            .into_iter()
            .chain(self.warmup.iter())
        {
            stream
                .wait(&lease.owner().ready)
                .map_err(cuda_error("external-producer-wait"))?;
            count += 1;
        }
        Ok(count)
    }
    pub(super) fn launch_to(
        &self,
        stream: &CudaStream,
        output: &BufferLease<ImportedOwner>,
    ) -> Result<(), ProbeError> {
        let input_pointer = self.input.owner().pointer + self.input.range().start as u64;
        let output_pointer = output.owner().pointer + output.range().start as u64;
        // SAFETY: 导入查询真实范围与上下文。
        // 租约保护偏移、权限与独占写入。
        // 队列保留所有者、事件与模块。
        // 执行流先等待记录的生产事件。
        unsafe {
            launch_affine_raw(
                stream,
                &self.kernel,
                input_pointer,
                output_pointer,
                self.elements,
                self.metadata.as_ref(),
            )
        }
    }
    pub(super) fn capture(
        &mut self,
        stream: &Arc<CudaStream>,
        backend: ProbeBackend,
    ) -> Result<(), ProbeError> {
        self.launch_to(stream, self.warmup.as_ref().unwrap())?;
        stream
            .synchronize()
            .map_err(cuda_error("external-warmup"))?;
        stream
            .begin_capture(sys::CUstreamCaptureMode::CU_STREAM_CAPTURE_MODE_THREAD_LOCAL)
            .map_err(cuda_error("external-capture-begin"))?;
        let tracking = TrackingPause::after_warmup(stream.context());
        let captured = self.launch_to(stream, &self.output);
        let ended = ProbeGraph::end_capture(stream, backend);
        drop(tracking);
        captured?;
        let graph = ended?;
        if graph.kernels.len() != 1 {
            return Err(ProbeError::InvalidGraph("外部图节点数量不符"));
        }
        self.graph = Some(graph);
        Ok(())
    }
}
