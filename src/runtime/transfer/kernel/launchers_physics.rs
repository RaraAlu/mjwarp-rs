//! 同步内核的物理类发射入口。

use super::*;

impl SynchronousKernel {
    /// ABI: (i32*, f32*, f32*, f32*, f32*, f32*, u32, u32).
    /// flag使用私有u32，不混用上游bool结构ABI。
    ///
    /// # Safety
    /// Caller proves metadata and all four f32 input strides.
    /// Inputs are immutable and disjoint from the guarded output.
    /// All buffers retain this session; the kernel spawns no child work.
    pub(crate) unsafe fn launch_rne_cacc(
        &self,
        metadata: &TransferBuffer<i32>,
        inputs: (
            &TransferBuffer<f32>,
            &TransferBuffer<f32>,
            &TransferBuffer<f32>,
            &TransferBuffer<f32>,
        ),
        output: &mut TransferBuffer<f32>,
        flg_acc: bool,
        worlds: u32,
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &inputs.0.session.inner),
            Arc::ptr_eq(&self.session.inner, &inputs.1.session.inner),
            Arc::ptr_eq(&self.session.inner, &inputs.2.session.inner),
            Arc::ptr_eq(&self.session.inner, &inputs.3.session.inner),
            Arc::ptr_eq(&self.session.inner, &output.session.inner),
        ] {
            if !same {
                return Err(TransferError::SessionMismatch);
            }
        }
        if worlds == 0 || worlds > u32::MAX - 255 {
            return Err(InputError::InvalidDimension {
                field: "kernel_work_items",
            }
            .into());
        }
        let (Some(m), Some(v), Some(a), Some(c), Some(d), Some(o)) = (
            metadata.allocation.as_ref(),
            inputs.0.allocation.as_ref(),
            inputs.1.allocation.as_ref(),
            inputs.2.allocation.as_ref(),
            inputs.3.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        let flag = u32::from(flg_acc);
        self.session.run("rne-cacc-kernel-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m)
                .arg(v)
                .arg(a)
                .arg(c)
                .arg(d)
                .arg(o)
                .arg(&flag)
                .arg(&worlds);
            // SAFETY: The caller proves the exact private ABI and all ranges.
            // run holds the session operation lock and waits for completion.
            unsafe {
                args.launch(LaunchConfig {
                    grid_dim: (worlds.div_ceil(256), 1, 1),
                    block_dim: (256, 1, 1),
                    shared_mem_bytes: 0,
                })
            }?;
            Ok(())
        })
    }

    /// ABI：四个只读缓冲、一个可写缓冲、四项维度。
    ///
    /// # Safety
    /// 调用方证明私有ABI和全部跨度。
    /// 输入只读且输出独占。
    pub(crate) unsafe fn launch_tendon_passive(
        &self,
        metadata: &TransferBuffer<i32>,
        parameters: &TransferBuffer<f32>,
        source: &TransferBuffer<f32>,
        velocity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &parameters.session.inner),
            Arc::ptr_eq(&self.session.inner, &source.session.inner),
            Arc::ptr_eq(&self.session.inner, &velocity.session.inner),
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
        let (Some(m), Some(p), Some(s), Some(v), Some(o)) = (
            metadata.allocation.as_ref(),
            parameters.allocation.as_ref(),
            source.allocation.as_ref(),
            velocity.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        self.session.run("tendon-passive-kernel-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m).arg(p).arg(s).arg(v).arg(o);
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

    /// ABI：三个只读f32缓冲、一个可写缓冲、四项维度。
    ///
    /// # Safety
    /// 调用方证明私有ABI与全部跨度。
    /// 输入只读且输出独占。
    pub(crate) unsafe fn launch_gravity_compensation(
        &self,
        metadata: &TransferBuffer<i32>,
        parameters: &TransferBuffer<f32>,
        rigid: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &parameters.session.inner),
            Arc::ptr_eq(&self.session.inner, &rigid.session.inner),
            Arc::ptr_eq(&self.session.inner, &com.session.inner),
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
        let (Some(m), Some(p), Some(r), Some(c), Some(o)) = (
            metadata.allocation.as_ref(),
            parameters.allocation.as_ref(),
            rigid.allocation.as_ref(),
            com.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        self.session.run("gravity-compensation-kernel-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m).arg(p).arg(r).arg(c).arg(o);
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

    /// ABI：五个只读f32缓冲、一个可写缓冲、四项维度。
    ///
    /// # Safety
    /// 调用方证明流体内核的私有ABI与全部跨度。
    /// 输入只读且输出独占。
    pub(crate) unsafe fn launch_fluid_force(
        &self,
        metadata: &TransferBuffer<i32>,
        inputs: [&TransferBuffer<f32>; 5],
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in std::iter::once(Arc::ptr_eq(&self.session.inner, &metadata.session.inner))
            .chain(
                inputs
                    .iter()
                    .map(|buffer| Arc::ptr_eq(&self.session.inner, &buffer.session.inner)),
            )
            .chain(std::iter::once(Arc::ptr_eq(
                &self.session.inner,
                &output.session.inner,
            )))
        {
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
        let [first, second, third, fourth, fifth] = inputs;
        let (
            Some(metadata),
            Some(first),
            Some(second),
            Some(third),
            Some(fourth),
            Some(fifth),
            Some(output),
        ) = (
            metadata.allocation.as_ref(),
            first.allocation.as_ref(),
            second.allocation.as_ref(),
            third.allocation.as_ref(),
            fourth.allocation.as_ref(),
            fifth.allocation.as_ref(),
            output.allocation.as_mut(),
        )
        else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        self.session.run("fluid-force-kernel-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(metadata)
                .arg(first)
                .arg(second)
                .arg(third)
                .arg(fourth)
                .arg(fifth)
                .arg(output);
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

    /// ABI: (i32*, f32*, f32*, f32*, i32*, f32*, u32).
    /// 六个缓冲保留同一会话。
    ///
    /// # Safety
    /// Caller proves checked metadata, strides and this exact private ABI.
    /// Only output is mutable; all buffers are disjoint and nonempty.
    /// The kernel writes guarded Hessians and launches no child work.
    pub(crate) unsafe fn launch_flex_hessian(
        &self,
        model: (&TransferBuffer<i32>, &TransferBuffer<f32>),
        positions: &TransferBuffer<f32>,
        edges: &TransferBuffer<f32>,
        validity: &TransferBuffer<i32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &model.0.session.inner),
            Arc::ptr_eq(&self.session.inner, &model.1.session.inner),
            Arc::ptr_eq(&self.session.inner, &positions.session.inner),
            Arc::ptr_eq(&self.session.inner, &edges.session.inner),
            Arc::ptr_eq(&self.session.inner, &validity.session.inner),
            Arc::ptr_eq(&self.session.inner, &output.session.inner),
        ] {
            if !same {
                return Err(TransferError::SessionMismatch);
            }
        }
        if worlds == 0 || worlds > u32::MAX - 255 {
            return Err(InputError::InvalidDimension {
                field: "kernel_work_items",
            }
            .into());
        }
        let (Some(metadata), Some(params), Some(pos), Some(edges), Some(valid), Some(out)) = (
            model.0.allocation.as_ref(),
            model.1.allocation.as_ref(),
            positions.allocation.as_ref(),
            edges.allocation.as_ref(),
            validity.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        self.session.run("flex-hessian-kernel-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(metadata)
                .arg(params)
                .arg(pos)
                .arg(edges)
                .arg(valid)
                .arg(out)
                .arg(&worlds);
            // SAFETY: The caller proves all guarded ranges and pointer types.
            // run serializes this session and waits before releasing borrows.
            unsafe {
                args.launch(LaunchConfig {
                    grid_dim: (worlds.div_ceil(256), 1, 1),
                    block_dim: (256, 1, 1),
                    shared_mem_bytes: 0,
                })
            }?;
            Ok(())
        })
    }

    /// ABI: (const int*, const float*, const float*, const float*, float*, u32).
    ///
    /// # Safety
    /// Caller proves this f32 ABI, strides and accesses.
    /// Inputs are read-only; output is disjoint. All buffers are nonempty.
    /// The kernel only writes output and launches no asynchronous child work.
    pub(crate) unsafe fn launch_flex_edges(
        &self,
        metadata: &TransferBuffer<i32>,
        qvel: &TransferBuffer<f32>,
        com: &TransferBuffer<f32>,
        flex: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: u32,
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &qvel.session.inner),
            Arc::ptr_eq(&self.session.inner, &com.session.inner),
            Arc::ptr_eq(&self.session.inner, &flex.session.inner),
            Arc::ptr_eq(&self.session.inner, &output.session.inner),
        ] {
            if !same {
                return Err(TransferError::SessionMismatch);
            }
        }
        if worlds == 0 || worlds > u32::MAX - 255 {
            return Err(InputError::InvalidDimension {
                field: "kernel_work_items",
            }
            .into());
        }
        let (Some(metadata), Some(qvel), Some(com), Some(flex), Some(output)) = (
            metadata.allocation.as_ref(),
            qvel.allocation.as_ref(),
            com.allocation.as_ref(),
            flex.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(InputError::InvalidDimension {
                field: "kernel_buffers",
            }
            .into());
        };
        self.session.run("flex-edge-kernel-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(metadata)
                .arg(qvel)
                .arg(com)
                .arg(flex)
                .arg(output)
                .arg(&worlds);
            // SAFETY: The caller proves the private ABI and checked ranges.
            // run serializes this session and waits before releasing borrows.
            unsafe {
                args.launch(LaunchConfig {
                    grid_dim: (worlds.div_ceil(256), 1, 1),
                    block_dim: (256, 1, 1),
                    shared_mem_bytes: 0,
                })
            }?;
            Ok(())
        })
    }
}
