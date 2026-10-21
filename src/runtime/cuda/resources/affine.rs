//! 固定仿射内核的保留资源与图捕获。

use super::super::{LoadedKernel, ProbeGraph, TrackingPause, cuda_error};
use crate::{
    diagnostics::ProbeError,
    runtime::{BLOCK_THREADS, ProbeBackend, completion::RetainedResources, lease::BufferLease},
};
use cudarc::driver::{CudaSlice, CudaStream, DevicePtr, LaunchConfig, PushKernelArg, sys};
use std::sync::Arc;

pub(super) struct AffineResources {
    // 按字段顺序先销毁图。
    pub(super) graph: Option<ProbeGraph>,
    pub(super) input: BufferLease<CudaSlice<f32>>,
    pub(super) output: BufferLease<CudaSlice<f32>>,
    pub(super) warmup_output: Option<BufferLease<CudaSlice<f32>>>,
    pub(super) metadata: Option<CudaSlice<u32>>,
    pub(super) kernel: LoadedKernel,
    pub(super) elements: usize,
}

impl RetainedResources for AffineResources {
    fn quarantine(&self) {
        self.input.quarantine();
        self.output.quarantine();
        if let Some(output) = &self.warmup_output {
            output.quarantine();
        }
    }
}

impl AffineResources {
    pub(super) fn launch(&self, stream: &CudaStream) -> Result<(), ProbeError> {
        self.launch_to(stream, &self.output)
    }

    fn launch_to(
        &self,
        stream: &CudaStream,
        output: &BufferLease<CudaSlice<f32>>,
    ) -> Result<(), ProbeError> {
        let (input, _input_record) = self.input.owner().device_ptr(stream);
        let (pointer, _output_record) = output.owner().device_ptr(stream);
        let input = input
            .checked_add(self.input.range().start as u64)
            .ok_or(ProbeError::InvalidArgument("输入地址溢出"))?;
        let output = pointer
            .checked_add(output.range().start as u64)
            .ok_or(ProbeError::InvalidArgument("输出地址溢出"))?;
        // SAFETY: 内部所有者提供真实地址。
        // 租约校验真实容量与字节偏移。
        // 输入只读；输出租约独占范围。
        // 队列先保留所有者再调用本函数。
        // 完成栅栏保护实际输出写入。
        // 元数据记录视图长度而非容量。
        // 内核守卫限制索引至视图范围。
        unsafe {
            launch_affine_raw(
                stream,
                &self.kernel,
                input,
                output,
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
        // 预热写独立缓冲，避免掩盖重放失败。
        self.launch_to(stream, self.warmup_output.as_ref().unwrap())?;
        stream.synchronize().map_err(cuda_error("lease-warmup"))?;
        // 所有私有上传流也已完成。
        stream
            .context()
            .synchronize()
            .map_err(cuda_error("lease-capture-ready"))?;
        stream
            .begin_capture(sys::CUstreamCaptureMode::CU_STREAM_CAPTURE_MODE_THREAD_LOCAL)
            .map_err(cuda_error("lease-capture-begin"))?;
        let tracking = TrackingPause::after_warmup(stream.context());
        let captured = self.launch(stream);
        let ended = ProbeGraph::end_capture(stream, backend);
        drop(tracking);
        captured?;
        let graph = ended?;
        if graph.kernels.len() != 1 {
            return Err(ProbeError::InvalidGraph("租约图节点数量不符"));
        }
        self.graph = Some(graph);
        Ok(())
    }
}

// 调用方保证指针范围、权限与寿命。
// 调用方只传本探针固定仿射内核。
pub(in crate::runtime::cuda) unsafe fn launch_affine_raw(
    stream: &CudaStream,
    kernel: &LoadedKernel,
    input: u64,
    output: u64,
    elements: usize,
    metadata: Option<&CudaSlice<u32>>,
) -> Result<(), ProbeError> {
    let elements = elements as u32;
    let config = LaunchConfig {
        grid_dim: (elements.div_ceil(BLOCK_THREADS as u32), 1, 1),
        block_dim: (BLOCK_THREADS as u32, 1, 1),
        shared_mem_bytes: kernel.shared_memory,
    };
    // SAFETY: 调用方保证两项有效地址。
    // 第三项采用原生或CubeCL固定ABI。
    // 调用方保留所有者至内核完成。
    unsafe {
        let mut builder = stream.launch_builder(&kernel.function);
        builder.arg(&input).arg(&output);
        if let Some(metadata) = metadata {
            builder.arg(metadata);
        } else {
            builder.arg(&elements);
        }
        builder.launch(config)
    }
    .map_err(cuda_error("lease-launch"))?;
    Ok(())
}
