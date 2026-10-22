//! 同步内核的被动力类发射入口。

use super::*;

impl SynchronousKernel {
    /// ABI: (const int*, const float*, const float*, float*, u32, u32, u32, u32).
    ///
    /// # Safety
    /// 调用方证明私有ABI与全部跨度。
    /// 输入只读且输出独占。
    pub(crate) unsafe fn launch_flex_edge_passive(
        &self,
        metadata: &TransferBuffer<i32>,
        parameters: &TransferBuffer<f32>,
        source: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &parameters.session.inner),
            Arc::ptr_eq(&self.session.inner, &source.session.inner),
            Arc::ptr_eq(&self.session.inner, &output.session.inner),
        ] {
            if !same {
                return Err(TransferError::SessionMismatch);
            }
        }
        let count = dimensions[3];
        if count == 0 || count > u32::MAX - 255 {
            return Err(InputError::InvalidDimension {
                field: "kernel_work_items",
            }
            .into());
        }
        let (Some(m), Some(p), Some(s), Some(o)) = (
            metadata.allocation.as_ref(),
            parameters.allocation.as_ref(),
            source.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        self.session.run("flex-edge-passive-kernel-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m).arg(p).arg(s).arg(o);
            for value in &dimensions {
                args.arg(value);
            }
            // SAFETY: 调用方证明私有ABI。
            // run会串行并等待完成。
            unsafe {
                args.launch(LaunchConfig {
                    grid_dim: (count.div_ceil(256), 1, 1),
                    block_dim: (256, 1, 1),
                    shared_mem_bytes: 0,
                })
            }?;
            Ok(())
        })
    }
}
