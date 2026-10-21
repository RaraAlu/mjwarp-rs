//! 同步内核通用四缓冲发射。

use super::*;

impl SynchronousKernel {
    /// # Safety
    /// Caller proves the exact ABI above and every kernel access for these
    /// dimensions. All four buffers must be nonempty. The kernel must only write
    /// output, must not outlive this synchronized launch, and must not spawn work
    /// elsewhere. Kernel code must not access pointers outside these buffers.
    pub(crate) unsafe fn launch<P: TransferElement, S: TransferElement, O: TransferElement>(
        &self,
        metadata: &TransferBuffer<i32>,
        parameters: &TransferBuffer<P>,
        state: &TransferBuffer<S>,
        output: &mut TransferBuffer<O>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &parameters.session.inner),
            Arc::ptr_eq(&self.session.inner, &state.session.inner),
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
        let (Some(metadata), Some(parameters), Some(state), Some(output)) = (
            metadata.allocation.as_ref(),
            parameters.allocation.as_ref(),
            state.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        self.session.run("synchronous-kernel-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(metadata).arg(parameters).arg(state).arg(output);
            for value in &dimensions {
                args.arg(value);
            }
            // SAFETY: The caller proves the ABI and all buffer accesses. The
            // operation lock serializes session work; run waits for completion.
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
