//! 设备身份、租约包装与异域上下文。

use super::super::cuda_error;
use crate::{
    diagnostics::ProbeError,
    runtime::lease::{DeviceIdentity, LeasedBuffer},
};
use cudarc::driver::{CudaContext, CudaSlice, sys};
use std::sync::Arc;

pub(super) fn identity(context: &CudaContext) -> DeviceIdentity {
    DeviceIdentity {
        device: context.ordinal(),
        context: context.cu_ctx() as usize,
    }
}

pub(super) fn wrap(
    owner: CudaSlice<f32>,
    writable: bool,
) -> Result<LeasedBuffer<CudaSlice<f32>>, ProbeError> {
    let target = identity(owner.context());
    let capacity = owner.num_bytes();
    LeasedBuffer::new(owner, target, capacity, size_of::<f32>(), writable)
}

// 仅用于验证同设备异上下文。
// 不将此上下文接入内核调用。
pub(in crate::runtime::cuda) struct ForeignContext {
    pub(in crate::runtime::cuda) raw: usize,
    primary: Arc<CudaContext>,
}
impl ForeignContext {
    pub(in crate::runtime::cuda) fn new(primary: &Arc<CudaContext>) -> Result<Self, ProbeError> {
        let mut raw = std::ptr::null_mut();
        // SAFETY: 驱动写入独立上下文句柄。
        // primary持有有效CUDA设备。
        unsafe { sys::cuCtxCreate_v2(&mut raw, 0, primary.cu_device()) }
            .result()
            .map_err(cuda_error("lease-foreign-context"))?;
        let context = Self {
            raw: raw as usize,
            primary: primary.clone(),
        };
        primary
            .bind_to_thread()
            .map_err(cuda_error("lease-restore-context"))?;
        Ok(context)
    }
}
impl Drop for ForeignContext {
    fn drop(&mut self) {
        // SAFETY: 此对象独占异域上下文。
        // 本探针从未向此上下文提交任务。
        self.primary
            .record_err(unsafe { sys::cuCtxDestroy_v2(self.raw as sys::CUcontext) }.result());
        self.primary.record_err(self.primary.bind_to_thread());
    }
}
