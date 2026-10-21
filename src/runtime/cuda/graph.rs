//! 内核图捕获、参数替换与重放。

use std::sync::Arc;

use cudarc::driver::{CudaStream, result, sys};

use crate::diagnostics::ProbeError;
use crate::runtime::ProbeBackend;

use super::cuda_error;
use super::{CapturedKernel, ProbeGraph};

impl ProbeGraph {
    pub(super) fn end_capture(
        stream: &Arc<CudaStream>,
        backend: ProbeBackend,
    ) -> Result<Self, ProbeError> {
        stream
            .context()
            .bind_to_thread()
            .map_err(cuda_error("capture-context"))?;
        // SAFETY: 当前线程持有捕获流。
        // 调用结束本次捕获。
        let raw = unsafe { result::stream::end_capture(stream.cu_stream()) }
            .map_err(cuda_error("capture-end"))?;
        if raw.is_null() {
            return Err(ProbeError::EmptyGraph);
        }
        let mut graph = Self {
            raw,
            exec: std::ptr::null_mut(),
            stream: stream.clone(),
            kernels: Vec::new(),
            backend,
        };
        graph.read_kernel_nodes()?;
        // SAFETY: raw指向完整捕获图。
        // 图不分配显存；标志不改写缓冲。
        // graph在失败时也释放原始图。
        graph.exec = unsafe {
            result::graph::instantiate(
                raw,
                sys::CUgraphInstantiate_flags::CUDA_GRAPH_INSTANTIATE_FLAG_AUTO_FREE_ON_LAUNCH,
            )
        }
        .map_err(cuda_error("graph-instantiate"))?;
        Ok(graph)
    }

    fn read_kernel_nodes(&mut self) -> Result<(), ProbeError> {
        let mut count = 0;
        // SAFETY: 本对象独占完整图。
        // 驱动只写节点数量。
        unsafe { sys::cuGraphGetNodes(self.raw, std::ptr::null_mut(), &mut count) }
            .result()
            .map_err(cuda_error("graph-node-count"))?;
        let mut nodes = vec![std::ptr::null_mut(); count];
        // SAFETY: nodes容量等于查询数量。
        // 本线程不修改图结构。
        unsafe { sys::cuGraphGetNodes(self.raw, nodes.as_mut_ptr(), &mut count) }
            .result()
            .map_err(cuda_error("graph-nodes"))?;
        for node in nodes.into_iter().take(count) {
            let mut kind = sys::CUgraphNodeType::CU_GRAPH_NODE_TYPE_KERNEL;
            // SAFETY: node来自本对象的图。
            unsafe { sys::cuGraphNodeGetType(node, &mut kind) }
                .result()
                .map_err(cuda_error("graph-node-type"))?;
            if kind != sys::CUgraphNodeType::CU_GRAPH_NODE_TYPE_KERNEL {
                return Err(ProbeError::InvalidGraph("探针只捕获内核节点"));
            }
            let mut params = std::mem::MaybeUninit::zeroed();
            // SAFETY: 驱动完整初始化参数结构。
            unsafe { sys::cuGraphKernelNodeGetParams_v2(node, params.as_mut_ptr()) }
                .result()
                .map_err(cuda_error("graph-kernel-params"))?;
            // SAFETY: 上次调用成功初始化结构。
            let params = unsafe { params.assume_init() };
            if params.kernelParams.is_null() || !params.extra.is_null() {
                return Err(ProbeError::InvalidGraph("内核参数布局不符"));
            }
            // SAFETY: 固定ABI包含三项参数。
            // 驱动拥有该参数地址表。
            let pointers = unsafe {
                [
                    *params.kernelParams,
                    *params.kernelParams.add(1),
                    *params.kernelParams.add(2),
                ]
            };
            if pointers.iter().any(|pointer| pointer.is_null()) {
                return Err(ProbeError::InvalidGraph("内核参数地址为空"));
            }
            // SAFETY: 本私有图只捕获固定ABI。
            // 前两项均为64位设备指针。
            // 原生第三项为u32长度。
            // CubeCL第三项为设备指针。
            // 驱动拥有参数副本，图仍存活。
            let arguments = unsafe {
                let input = pointers[0].cast::<u64>().read_unaligned();
                let output = pointers[1].cast::<u64>().read_unaligned();
                let third = pointers[2];
                let third = if self.backend == ProbeBackend::NativePtx {
                    u64::from(third.cast::<u32>().read_unaligned())
                } else {
                    third.cast::<u64>().read_unaligned()
                };
                [input, output, third]
            };
            self.kernels.push(CapturedKernel {
                node,
                params,
                arguments,
            });
        }
        if self.kernels.is_empty() {
            return Err(ProbeError::EmptyGraph);
        }
        Ok(())
    }

    // 调用方只替换同型同容量输出。
    // 调用方保留所有者至执行完成。
    pub(super) fn update_output(
        &mut self,
        original: u64,
        replacement: u64,
    ) -> Result<usize, ProbeError> {
        self.stream
            .context()
            .bind_to_thread()
            .map_err(cuda_error("graph-update-context"))?;
        // 节点更新不与任何重放并发。
        self.stream
            .synchronize()
            .map_err(cuda_error("graph-update-wait"))?;
        let mut updated = 0;
        for kernel in &self.kernels {
            let mut values = kernel.arguments;
            if !replace_output(&mut values, original, replacement) {
                continue;
            }
            let mut native_length = values[2] as u32;
            let third = if self.backend == ProbeBackend::NativePtx {
                (&mut native_length as *mut u32).cast()
            } else {
                (&mut values[2] as *mut u64).cast()
            };
            let mut arguments = [
                (&mut values[0] as *mut u64).cast(),
                (&mut values[1] as *mut u64).cast(),
                third,
            ];
            let mut params = kernel.params;
            params.kernelParams = arguments.as_mut_ptr();
            params.extra = std::ptr::null_mut();
            // SAFETY: 节点属于此图实例。
            // 函数、维度和共享区均不改变。
            // 参数按原始ABI复制有效值。
            // CUDA在调用内复制参数值。
            // 不改写原始节点的参数内存。
            unsafe { sys::cuGraphExecKernelNodeSetParams_v2(self.exec, kernel.node, &params) }
                .result()
                .map_err(cuda_error("graph-node-update"))?;
            updated += 1;
        }
        if updated == 0 {
            return Err(ProbeError::InvalidGraph("没有找到输出参数"));
        }
        Ok(updated)
    }

    pub(super) fn launch(&self) -> Result<(), ProbeError> {
        self.stream
            .context()
            .bind_to_thread()
            .map_err(cuda_error("replay-context"))?;
        // SAFETY: 图、流和缓冲持续存活。
        // 调用方通过事件串行读写缓冲。
        unsafe { result::graph::launch(self.exec, self.stream.cu_stream()) }
            .map_err(cuda_error("graph-replay"))
    }
}

pub(super) fn replace_output(arguments: &mut [u64; 3], original: u64, replacement: u64) -> bool {
    let mut changed = false;
    // 第三项始终保留元数据或长度。
    for argument in &mut arguments[..2] {
        if *argument == original {
            *argument = replacement;
            changed = true;
        }
    }
    changed
}

impl Drop for ProbeGraph {
    fn drop(&mut self) {
        let context = self.stream.context();
        if let Err(error) = context.bind_to_thread() {
            context.record_err::<()>(Err(error));
            return;
        }
        // panic路径也等待图内核完成。
        context.record_err(self.stream.synchronize());
        if !self.exec.is_null() {
            // SAFETY: 本对象独占图实例。
            // 调用方已等待所有图重放。
            context.record_err(unsafe { result::graph::exec_destroy(self.exec) });
        }
        // SAFETY: 本对象独占原始图。
        // 实例已释放，或实例化失败。
        context.record_err(unsafe { result::graph::destroy(self.raw) });
    }
}
