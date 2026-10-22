//! 同步内核的被动力类发射入口。

use super::*;

impl SynchronousKernel {
    /// 四缓冲被动力共用的私有ABI执行体。
    ///
    /// # Safety
    /// 调用方证明私有ABI与全部跨度。
    /// 输入只读且输出独占。
    unsafe fn launch_four_buffers(
        &self,
        metadata: &TransferBuffer<i32>,
        first: &TransferBuffer<f32>,
        second: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
        stage: &'static str,
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &first.session.inner),
            Arc::ptr_eq(&self.session.inner, &second.session.inner),
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
        let (Some(m), Some(a), Some(b), Some(o)) = (
            metadata.allocation.as_ref(),
            first.allocation.as_ref(),
            second.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        self.session.run(stage, || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m).arg(a).arg(b).arg(o);
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

    /// ABI: (const int*, const float*, const float*, float*, u32, u32, u32, u32).
    ///
    /// # Safety
    /// 调用方证明私有ABI与全部跨度。
    pub(crate) unsafe fn launch_flex_edge_passive(
        &self,
        metadata: &TransferBuffer<i32>,
        parameters: &TransferBuffer<f32>,
        source: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        // SAFETY: 由调用方转交的同一组契约。
        unsafe {
            self.launch_four_buffers(
                metadata,
                parameters,
                source,
                output,
                dimensions,
                "flex-edge-passive-kernel-launch",
            )
        }
    }

    /// ABI: (const int*, const float*, const float*, float*, u32, u32, u32, u32).
    ///
    /// # Safety
    /// 调用方证明私有ABI与全部跨度。
    pub(crate) unsafe fn launch_joint_passive(
        &self,
        metadata: &TransferBuffer<i32>,
        parameters: &TransferBuffer<f32>,
        state: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        // SAFETY: 由调用方转交的同一组契约。
        unsafe {
            self.launch_four_buffers(
                metadata,
                parameters,
                state,
                output,
                dimensions,
                "joint-passive-kernel-launch",
            )
        }
    }
}
