//! 内部固定ABI内核适配。
//! 不提供安全的任意内核入口。

use super::{TransferBuffer, TransferElement, TransferSession};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use cudarc::{
    driver::{CudaFunction, LaunchConfig, PushKernelArg},
    nvrtc::{CompileOptions, compile_ptx_with_opts, sys},
};
use std::sync::Arc;
mod kinetic_energy;

#[cfg(test)]
#[path = "../../../tests/support/t4_rigid_capture.rs"]
mod recapture;

// 跟随冻结上游的Warp 1.15策略。
// fuse_fp开启FMA，且不启用fast-math。
const PHYSICS_FMAD: bool = true;

/// ABI: (const int*, const P*, const S*, O*, u32, u32, u32, u32).
/// 柔体边另用固定三输入ABI。
/// 拉伸矩阵另用六指针ABI。
/// RNE前向另用六指针及u32开关。
/// 调用方证明三种元素的实际ABI。
/// 核函数与缓冲保留同一会话。
pub(crate) struct SynchronousKernel {
    session: TransferSession,
    function: Option<CudaFunction>,
}

impl SynchronousKernel {
    /// 混合求解固定128线程；因子跨度包含稠密前缀。
    ///
    /// # Safety
    /// Caller proves packed route/diagonal/level/Small metadata and its bounds.
    /// Each block owns one world/RHS; all threads reach every barrier.
    pub(crate) unsafe fn launch_mixed_solve(
        &self,
        metadata: &TransferBuffer<i32>,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &rhs.session.inner),
            Arc::ptr_eq(&self.session.inner, &factor.session.inner),
            Arc::ptr_eq(&self.session.inner, &output.session.inner),
        ] {
            if !same {
                return Err(TransferError::SessionMismatch);
            }
        }
        let [n, stride, nr, worlds] = dimensions.map(|v| v as usize);
        let invalid = || InputError::InvalidDimension {
            field: "mixed_solve_abi",
        };
        let groups = worlds
            .checked_mul(nr)
            .filter(|&v| v > 0 && v <= i32::MAX as usize)
            .ok_or_else(invalid)?;
        if dimensions.iter().any(|&v| v > i32::MAX as u32) || n == 0 || stride < 2 * n {
            return Err(invalid().into());
        }
        let rhs_len = groups
            .checked_mul(n)
            .and_then(|v| v.checked_add(8))
            .ok_or_else(invalid)?;
        let factor_len = worlds
            .checked_mul(stride)
            .and_then(|v| v.checked_add(8))
            .ok_or_else(invalid)?;
        let meta_min = n
            .checked_mul(2)
            .and_then(|v| v.checked_add(5))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or_else(invalid)?;
        if rhs.len() != rhs_len
            || output.len() != rhs_len
            || factor.len() != factor_len
            || metadata.len() < meta_min
        {
            return Err(invalid().into());
        }
        let (Some(m), Some(r), Some(f), Some(o)) = (
            metadata.allocation.as_ref(),
            rhs.allocation.as_ref(),
            factor.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(invalid().into());
        };
        self.session.run("mixed-solve-128-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m).arg(r).arg(f).arg(o);
            for value in &dimensions {
                args.arg(value);
            }
            // SAFETY: Checked capacities and private metadata proof bound all
            // accesses; run serializes and synchronizes completion.
            unsafe {
                args.launch(LaunchConfig {
                    grid_dim: (groups as u32, 1, 1),
                    block_dim: (128, 1, 1),
                    shared_mem_bytes: 0,
                })
            }?;
            Ok(())
        })
    }
    /// Small分解固定64线程；每线程独占一个世界块。
    ///
    /// # Safety
    /// Caller proves triples (DOF start, prefix address, static size) and ABI.
    /// Input buffers are immutable; each thread owns a disjoint factor block.
    pub(crate) unsafe fn launch_small_factor(
        &self,
        metadata: &TransferBuffer<i32>,
        matrix: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &matrix.session.inner),
            Arc::ptr_eq(&self.session.inner, &unused.session.inner),
            Arc::ptr_eq(&self.session.inner, &output.session.inner),
        ] {
            if !same {
                return Err(TransferError::SessionMismatch);
            }
        }
        let [prefix, first, count, work] = dimensions.map(|v| v as usize);
        let invalid = || InputError::InvalidDimension {
            field: "small_factor_abi",
        };
        if count == 0
            || work == 0
            || work > (u32::MAX - 63) as usize
            || work % count != 0
            || prefix > i32::MAX as usize
        {
            return Err(invalid().into());
        }
        let min_meta = first
            .checked_add(count)
            .and_then(|v| v.checked_mul(3))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or_else(invalid)?;
        let length = (work / count)
            .checked_mul(prefix)
            .and_then(|v| v.checked_add(8))
            .ok_or_else(invalid)?;
        if metadata.len() < min_meta
            || matrix.len() != length
            || unused.len() != length
            || output.len() != length
        {
            return Err(invalid().into());
        }
        let (Some(m), Some(a), Some(b), Some(o)) = (
            metadata.allocation.as_ref(),
            matrix.allocation.as_ref(),
            unused.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(invalid().into());
        };
        self.session.run("small-factor-64-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m).arg(a).arg(b).arg(o);
            for d in &dimensions {
                args.arg(d);
            }
            // SAFETY: Capacity checks and the caller's static block descriptors
            // bound all accesses; run serializes and waits for completion.
            unsafe {
                args.launch(LaunchConfig {
                    grid_dim: ((work as u32).div_ceil(64), 1, 1),
                    block_dim: (64, 1, 1),
                    shared_mem_bytes: 0,
                })
            }?;
            Ok(())
        })
    }

    /// 固定128线程；每世界每RHS独占一个块。
    ///
    /// # Safety
    /// Caller proves metadata contents and the eight-argument sparse solve ABI.
    /// Buffers are disjoint; only output changes. Every thread reaches barriers.
    pub(crate) unsafe fn launch_sparse_solve(
        &self,
        metadata: &TransferBuffer<i32>,
        rhs: &TransferBuffer<f32>,
        factor: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &rhs.session.inner),
            Arc::ptr_eq(&self.session.inner, &factor.session.inner),
            Arc::ptr_eq(&self.session.inner, &output.session.inner),
        ] {
            if !same {
                return Err(TransferError::SessionMismatch);
            }
        }
        let [n, nc, nr, worlds] = dimensions.map(|v| v as usize);
        let invalid = || InputError::InvalidDimension {
            field: "sparse_solve_abi",
        };
        let groups = worlds
            .checked_mul(nr)
            .filter(|&v| v > 0 && v <= i32::MAX as usize)
            .ok_or_else(invalid)?;
        if dimensions.iter().any(|&v| v > i32::MAX as u32) {
            return Err(invalid().into());
        }
        let rhs_len = groups
            .checked_mul(n)
            .and_then(|v| v.checked_add(8))
            .ok_or_else(invalid)?;
        let factor_len = n
            .checked_add(nc)
            .and_then(|v| v.checked_mul(worlds))
            .and_then(|v| v.checked_add(8))
            .ok_or_else(invalid)?;
        let meta_min = n
            .checked_mul(2)
            .and_then(|v| v.checked_add(2))
            .ok_or_else(invalid)?;
        if rhs.len() != rhs_len
            || output.len() != rhs_len
            || factor.len() != factor_len
            || metadata.len() < meta_min
        {
            return Err(invalid().into());
        }
        let (Some(m), Some(r), Some(f), Some(o)) = (
            metadata.allocation.as_ref(),
            rhs.allocation.as_ref(),
            factor.allocation.as_ref(),
            output.allocation.as_mut(),
        ) else {
            return Err(invalid().into());
        };
        self.session.run("sparse-solve-128-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m).arg(r).arg(f).arg(o);
            for value in &dimensions {
                args.arg(value);
            }
            // SAFETY: Checked capacities and caller-proven metadata bound every
            // access. Each block owns a distinct RHS and run waits for completion.
            unsafe {
                args.launch(LaunchConfig {
                    grid_dim: (groups as u32, 1, 1),
                    block_dim: (128, 1, 1),
                    shared_mem_bytes: 0,
                })
            }?;
            Ok(())
        })
    }

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

    pub(crate) fn compile(
        session: &TransferSession,
        source: &str,
        entry: &'static str,
    ) -> Result<Self, TransferError> {
        Self::compile_checked(session, source, entry, true)
    }

    fn compile_checked(
        session: &TransferSession,
        source: &str,
        entry: &'static str,
        require_f32: bool,
    ) -> Result<Self, TransferError> {
        session.inner.status.check()?;
        // Compiler calls do not enqueue device work. Compile failures need not
        // quarantine a healthy transfer session, and cannot trigger CPU fallback.
        let ptx = std::panic::catch_unwind(|| {
            // SAFETY: Only probes the compiler library, without user pointers.
            if !unsafe { sys::is_culib_present() } {
                return Err(ProbeError::NvrtcUnavailable);
            }
            compile_ptx_with_opts(
                source,
                CompileOptions {
                    fmad: Some(PHYSICS_FMAD),
                    use_fast_math: Some(false),
                    options: vec!["--std=c++17".into(), "--gpu-architecture=compute_70".into()],
                    name: Some(format!("{entry}.cu")),
                    ..Default::default()
                },
            )
            .map_err(|error| ProbeError::Compilation {
                stage: "synchronous-kernel-nvrtc",
                detail: format!("{error:?}"),
            })
        })
        .unwrap_or(Err(ProbeError::BackendPanic))?;
        if require_f32 {
            let generated = ptx.to_src();
            let precision = check_physics_ptx(&generated, entry);
            record_physics_ptx(source, &generated, entry, precision.is_ok())?;
            precision?;
        }
        let function = session.run("synchronous-kernel-load", || {
            session
                .inner
                .stream
                .context()
                .load_module(ptx)?
                .load_function(entry)
        })?;
        Ok(Self {
            session: session.clone(),
            function: Some(function),
        })
    }

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

// Warp's f32 sin/cos also call CUDA sinf/cosf. CUDA 12.8's slow integer angle
// reduction emits this exact s64 -> f64 -> f32 triplet. It does not promote a
// physical f32 value or accumulator. Reject every other non-f32 operation.
fn check_physics_ptx(ptx: &str, entry: &'static str) -> Result<(), ProbeError> {
    let lines: Vec<_> = ptx
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code).trim())
        .filter(|line| !line.is_empty())
        .collect();
    let operands = |line: &str| -> Vec<String> {
        line.split(|c: char| c.is_ascii_whitespace() || c == ',' || c == ';')
            .filter(|word| !word.is_empty())
            .map(str::to_owned)
            .collect()
    };
    let mut reduction_count = 0;
    let mut declarations = false;
    let mut i = 0;
    while i < lines.len() {
        let code = lines[i];
        if code.starts_with(".reg .f64 ") || code.starts_with(".reg .f64\t") {
            declarations = true;
            i += 1;
            continue;
        }
        let a = operands(code);
        if a.len() == 3 && a[0] == "cvt.rn.f64.s64" && i + 2 < lines.len() {
            let b = operands(lines[i + 1]);
            let c = operands(lines[i + 2]);
            if b.len() == 4
                // 未标舍入的mul默认采用rn。
                // FMA开关会改变库内指令拼写。
                && matches!(b[0].as_str(), "mul.rn.f64" | "mul.f64")
                && b[2] == a[1]
                && b[3] == "0d3BF921FB54442D19"
                && c.len() == 3
                && c[0] == "cvt.rn.f32.f64"
                && c[2] == b[1]
            {
                reduction_count += 1;
                i += 3;
                continue;
            }
        }
        for token in code
            .split_whitespace()
            .flat_map(|word| word.split('.').skip(1))
        {
            let token = token.trim_matches(|c: char| !c.is_ascii_alphanumeric());
            if matches!(token, "f64" | "f16" | "f16x2" | "bf16" | "bf16x2" | "tf32") {
                return Err(ProbeError::Compilation {
                    stage: "physics-precision-ptx",
                    detail: format!("{entry}: forbidden {token}: {code}"),
                });
            }
        }
        i += 1;
    }
    if declarations && reduction_count == 0 {
        return Err(ProbeError::Compilation {
            stage: "physics-precision-ptx",
            detail: format!("{entry}: unaccounted f64 registers"),
        });
    }
    Ok(())
}

fn record_physics_ptx(
    source: &str,
    ptx: &str,
    entry: &'static str,
    passed: bool,
) -> Result<(), ProbeError> {
    let Some(directory) = std::env::var_os("MJWARP_PHYSICS_PTX_DIR") else {
        return Ok(());
    };
    let (mut major, mut minor) = (0, 0);
    // SAFETY: Compilation already loaded NVRTC; both output pointers are valid.
    unsafe { sys::nvrtcVersion(&mut major, &mut minor) }
        .result()
        .map_err(|error| ProbeError::Compilation {
            stage: "physics-precision-evidence",
            detail: format!("{entry}: NVRTC version: {error:?}"),
        })?;
    use sha2::{Digest, Sha256};
    let source_hash = format!("{:x}", Sha256::digest(source.as_bytes()));
    let ptx_hash = format!("{:x}", Sha256::digest(ptx.as_bytes()));
    let prefix = std::path::PathBuf::from(directory).join(format!("{entry}-{ptx_hash}"));
    let record = serde_json::json!({
        "entry": entry,
        "sourceSha256": source_hash,
        "ptxSha256": ptx_hash,
        "precisionGate": "f32-physics",
        "cudaMathReduction": "CUDA-12.8 sinf/cosf integer angle reduction only",
        "passed": passed,
        "nvrtcVersion": [major, minor],
        "fmad": PHYSICS_FMAD,
        "useFastMath": false,
        "options": ["--std=c++17", "--gpu-architecture=compute_70"],
    });
    let save = || -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(prefix.parent().unwrap())?;
        std::fs::write(prefix.with_extension("cu"), source)?;
        std::fs::write(prefix.with_extension("ptx"), ptx)?;
        std::fs::write(
            prefix.with_extension("json"),
            serde_json::to_vec_pretty(&record)?,
        )?;
        Ok(())
    };
    save().map_err(|error| ProbeError::Compilation {
        stage: "physics-precision-evidence",
        detail: format!("{entry}: {error}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "new mixed128 prefix-stride ABI; no old ABI or physics replay"]
    fn mixed_solve_128_abi_checks_prefix_stride_sessions_and_grouped_capacity() {
        let session = TransferSession::new(0).unwrap();
        let other = TransferSession::new(0).unwrap();
        let source = r#"extern "C" __global__ void mixed_solve_abi(const int* meta,const float* rhs,
 const float* factor,float* output,unsigned n,unsigned stride,unsigned nr,unsigned worlds) {
 __shared__ float slots[128];unsigned tid=threadIdx.x;slots[tid]=float(tid)+0.25f;__syncthreads();
 for(unsigned i=tid;i<n;i+=128)output[4+blockIdx.x*n+i]=float(blockIdx.x)*1000.0f+float(i)+slots[(tid+1)%128];
}"#;
        let kernel = SynchronousKernel::compile(&session, source, "mixed_solve_abi").unwrap();
        let metadata = session.upload(&vec![0i32; 263]).unwrap();
        let rhs = session.upload(&vec![0.0f32; 782]).unwrap();
        let factor = session.upload(&vec![0.0f32; 908]).unwrap();
        let mut output = session.upload(&vec![-131072.0f32; 782]).unwrap();
        let foreign_meta = other.upload(&vec![0i32; 263]).unwrap();
        let foreign_rhs = other.upload(&vec![0.0f32; 782]).unwrap();
        let foreign_factor = other.upload(&vec![0.0f32; 908]).unwrap();
        let mut foreign_output = other.upload(&vec![-131072.0f32; 782]).unwrap();
        let dims = [129, 300, 2, 3];
        // SAFETY: Rejection paths never launch. The probe ignores metadata and
        // inputs; its checked groups and n bound every disjoint output write.
        unsafe {
            assert!(matches!(
                kernel.launch_mixed_solve(&foreign_meta, &rhs, &factor, &mut output, dims),
                Err(TransferError::SessionMismatch)
            ));
            assert!(matches!(
                kernel.launch_mixed_solve(&metadata, &foreign_rhs, &factor, &mut output, dims),
                Err(TransferError::SessionMismatch)
            ));
            assert!(matches!(
                kernel.launch_mixed_solve(&metadata, &rhs, &foreign_factor, &mut output, dims),
                Err(TransferError::SessionMismatch)
            ));
            assert!(matches!(
                kernel.launch_mixed_solve(&metadata, &rhs, &factor, &mut foreign_output, dims),
                Err(TransferError::SessionMismatch)
            ));
            for bad in [
                [0, 300, 2, 3],
                [129, 257, 2, 3],
                [129, 300, 0, 3],
                [129, 300, 2, 0],
                [u32::MAX, 300, 2, 3],
                [1, 2, 65536, 65536],
            ] {
                assert!(
                    kernel
                        .launch_mixed_solve(&metadata, &rhs, &factor, &mut output, bad)
                        .is_err()
                );
            }
            let short_meta = session.upload(&vec![0i32; 262]).unwrap();
            let short_rhs = session.upload(&vec![0.0f32; 781]).unwrap();
            let short_factor = session.upload(&vec![0.0f32; 907]).unwrap();
            let mut short_output = session.upload(&vec![0.0f32; 781]).unwrap();
            assert!(
                kernel
                    .launch_mixed_solve(&short_meta, &rhs, &factor, &mut output, dims)
                    .is_err()
            );
            assert!(
                kernel
                    .launch_mixed_solve(&metadata, &short_rhs, &factor, &mut output, dims)
                    .is_err()
            );
            assert!(
                kernel
                    .launch_mixed_solve(&metadata, &rhs, &short_factor, &mut output, dims)
                    .is_err()
            );
            assert!(
                kernel
                    .launch_mixed_solve(&metadata, &rhs, &factor, &mut short_output, dims)
                    .is_err()
            );
            kernel
                .launch_mixed_solve(&metadata, &rhs, &factor, &mut output, dims)
                .unwrap();
        }
        let mut actual = vec![0.0; 782];
        output.read_range_into(0, &mut actual).unwrap();
        assert!(
            actual[..4]
                .iter()
                .chain(&actual[778..])
                .all(|&v| v == -131072.0)
        );
        for group in 0..6 {
            for i in 0..129 {
                assert_eq!(
                    actual[4 + group * 129 + i],
                    (group * 1000 + i) as f32 + ((i % 128 + 1) % 128) as f32 + 0.25
                );
            }
        }
    }

    #[test]
    #[ignore = "requires NVIDIA/NVRTC; new Small64 ABI purpose"]
    fn small_factor_64_abi_checks_sessions_capacities_and_work_partition() {
        let session = TransferSession::new(0).unwrap();
        let other = TransferSession::new(0).unwrap();
        let source = r#"
extern "C" __global__ void small_abi_probe(const int* m,const float* a,const float* b,float* o,
 unsigned prefix,unsigned first,unsigned count,unsigned work) {
 __shared__ unsigned slots[64];slots[threadIdx.x]=threadIdx.x;__syncthreads();
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;
 if(tid<work)o[4+tid]=blockDim.x*1000+blockIdx.x*64+slots[63-threadIdx.x];
}
"#;
        let kernel = SynchronousKernel::compile(&session, source, "small_abi_probe").unwrap();
        let m = session.upload(&vec![0i32; 192]).unwrap();
        let a = session.upload(&vec![0f32; 136]).unwrap();
        let mut out = session.upload(&vec![-131072f32; 136]).unwrap();
        let fm = other.upload(&vec![0i32; 192]).unwrap();
        let fa = other.upload(&vec![0f32; 136]).unwrap();
        let mut fo = other.upload(&vec![0f32; 136]).unwrap();
        let tiny_m = session.upload(&[0i32]).unwrap();
        let tiny = session.upload(&[0f32]).unwrap();
        let mut tiny_out = session.upload(&[0f32]).unwrap();
        // SAFETY: Rejection cases return before launch. The accepted probe has
        // two full64 blocks, 128 output slots and exactly64 shared slots.
        unsafe {
            for k in 0..4 {
                assert_eq!(
                    kernel.launch_small_factor(
                        if k == 0 { &fm } else { &m },
                        if k == 1 { &fa } else { &a },
                        if k == 2 { &fa } else { &a },
                        if k == 3 { &mut fo } else { &mut out },
                        [64, 0, 64, 128]
                    ),
                    Err(TransferError::SessionMismatch)
                );
            }
            for dims in [
                [64, 0, 0, 128],
                [64, 0, 64, 0],
                [64, 0, 64, 129],
                [u32::MAX, 0, 64, 128],
                [64, u32::MAX, 64, 128],
                [64, 0, 64, u32::MAX],
            ] {
                assert!(matches!(
                    kernel.launch_small_factor(&m, &a, &a, &mut out, dims),
                    Err(TransferError::Input(_))
                ));
            }
            for k in 0..4 {
                assert!(matches!(
                    kernel.launch_small_factor(
                        if k == 0 { &tiny_m } else { &m },
                        if k == 1 { &tiny } else { &a },
                        if k == 2 { &tiny } else { &a },
                        if k == 3 { &mut tiny_out } else { &mut out },
                        [64, 0, 64, 128]
                    ),
                    Err(TransferError::Input(_))
                ));
            }
            kernel
                .launch_small_factor(&m, &a, &a, &mut out, [64, 0, 64, 128])
                .unwrap();
        }
        let mut values = vec![0f32; 136];
        out.read_range_into(0, &mut values).unwrap();
        assert!(
            values[..4]
                .iter()
                .chain(&values[132..])
                .all(|&v| v == -131072.0)
        );
        for group in 0..2 {
            for tid in 0..64 {
                assert_eq!(
                    values[4 + group * 64 + tid],
                    (64000 + group * 64 + 63 - tid) as f32
                );
            }
        }
    }

    #[test]
    #[ignore = "requires NVIDIA driver/NVRTC; new 128-thread sparse solve ABI purpose"]
    fn sparse_solve_128_abi_checks_sessions_capacities_and_block_barriers() {
        let session = TransferSession::new(0).unwrap();
        let other = TransferSession::new(0).unwrap();
        let source = r#"
extern "C" __global__ void sparse_abi_probe(const int* m,const float* r,const float* f,
 float* o,unsigned n,unsigned nc,unsigned nr,unsigned worlds) {
 __shared__ unsigned count[128];
 count[threadIdx.x]=threadIdx.x;
 __syncthreads();
 o[4+(unsigned long long)blockIdx.x*n+threadIdx.x]=blockDim.x*1000+blockIdx.x*n+count[127-threadIdx.x];
}
"#;
        let kernel = SynchronousKernel::compile(&session, source, "sparse_abi_probe").unwrap();
        let m = session.upload(&vec![0i32; 258]).unwrap();
        let r = session.upload(&vec![0f32; 776]).unwrap();
        let f = session.upload(&vec![0f32; 392]).unwrap();
        let mut out = session.upload(&vec![-131072f32; 776]).unwrap();
        let fm = other.upload(&vec![0i32; 258]).unwrap();
        let fr = other.upload(&vec![0f32; 776]).unwrap();
        let ff = other.upload(&vec![0f32; 392]).unwrap();
        let mut fo = other.upload(&vec![0f32; 776]).unwrap();
        let small_m = session.upload(&[0i32]).unwrap();
        let small = session.upload(&[0f32]).unwrap();
        let mut small_out = session.upload(&[0f32]).unwrap();
        // SAFETY: Rejection cases return before launch. The accepted probe has
        // n=128, six disjoint guarded slices, and exactly 128 shared slots.
        unsafe {
            for k in 0..4 {
                assert_eq!(
                    kernel.launch_sparse_solve(
                        if k == 0 { &fm } else { &m },
                        if k == 1 { &fr } else { &r },
                        if k == 2 { &ff } else { &f },
                        if k == 3 { &mut fo } else { &mut out },
                        [128, 0, 2, 3]
                    ),
                    Err(TransferError::SessionMismatch)
                );
            }
            for dim in [
                [128, 0, 0, 3],
                [128, 0, 2, 0],
                [128, 0, u32::MAX, 3],
                [u32::MAX, 0, 2, 3],
                [128, u32::MAX, 2, 3],
                [128, 0, 2, u32::MAX],
            ] {
                assert!(matches!(
                    kernel.launch_sparse_solve(&m, &r, &f, &mut out, dim),
                    Err(TransferError::Input(_))
                ));
            }
            for k in 0..4 {
                assert!(matches!(
                    kernel.launch_sparse_solve(
                        if k == 0 { &small_m } else { &m },
                        if k == 1 { &small } else { &r },
                        if k == 2 { &small } else { &f },
                        if k == 3 { &mut small_out } else { &mut out },
                        [128, 0, 2, 3]
                    ),
                    Err(TransferError::Input(_))
                ));
            }
            kernel
                .launch_sparse_solve(&m, &r, &f, &mut out, [128, 0, 2, 3])
                .unwrap();
        }
        let mut values = vec![0f32; 776];
        out.read_range_into(0, &mut values).unwrap();
        assert!(
            values[..4]
                .iter()
                .chain(&values[772..])
                .all(|&v| v == -131072.0)
        );
        for group in 0..6 {
            for tid in 0..128 {
                assert_eq!(
                    values[4 + group * 128 + tid],
                    (128000 + group * 128 + 127 - tid) as f32
                );
            }
        }
    }

    #[test]
    fn rejects_generated_physics_precision_promotions_and_demotions() {
        for code in [
            "add.rn.f64 %fd1, %fd2, %fd3;",
            "cvt.f32.f64 %f1, %fd2;",
            ".reg .f64 %fd<2>;",
            "mul.rn.f16 %h1, %h2, %h3;",
            "add.f16x2 %r1, %r2, %r3;",
            "cvt.rn.bf16.f32 %h1, %f1;",
            "add.bf16x2 %r1, %r2, %r3;",
            "cvt.rn.tf32.f32 %r1, %f1;",
        ] {
            assert!(matches!(
                check_physics_ptx(code, "probe"),
                Err(ProbeError::Compilation {
                    stage: "physics-precision-ptx",
                    ..
                })
            ));
        }
        check_physics_ptx(
            "// f64 is forbidden\n.reg .b64 %rd<2>;\nadd.rn.f32 %f16, %f64, %f3;",
            "probe",
        )
        .unwrap();
        let reduction = ".reg .f64 %fd<3>;\ncvt.rn.f64.s64 %fd1, %rd1;\nmul.rn.f64 %fd2, %fd1, 0d3BF921FB54442D19;\ncvt.rn.f32.f64 %f1, %fd2;";
        check_physics_ptx(reduction, "probe").unwrap();
        check_physics_ptx(&reduction.replace("mul.rn.f64", "mul.f64"), "probe").unwrap();
        for changed in [
            reduction.replace("f64.s64", "f64.f32"),
            reduction.replace("0d3BF921FB54442D19", "0d3FF0000000000000"),
            reduction.replace("%fd1, 0d", "%fd3, 0d"),
            reduction.replace("mul.rn.f64", "mul.rz.f64"),
            format!("{reduction}\nadd.rn.f64 %fd1, %fd1, %fd2;"),
            format!("{reduction}\nmul.f64 %fd1, %fd1, %fd2;"),
        ] {
            assert!(check_physics_ptx(&changed, "probe").is_err());
        }
    }

    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn physical_kernel_fuses_f32_multiply_add_like_warp() {
        let session = TransferSession::new(0).unwrap();
        let source = r#"
extern "C" __global__ void fma_probe(const int* m,const float* p,const float* s,
    float* o,unsigned a,unsigned b,unsigned c,unsigned n) {
  if(threadIdx.x==0 && blockIdx.x==0) o[0]=p[0]*s[0]+p[1];
}
"#;
        let kernel = SynchronousKernel::compile(&session, source, "fma_probe").unwrap();
        let a = 1.0 + f32::EPSILON;
        let b = 1.0 - f32::EPSILON;
        let meta = session.upload(&[0_i32]).unwrap();
        let parameters = session.upload(&[a, -1.0]).unwrap();
        let state = session.upload(&[b]).unwrap();
        let mut output = session.upload(&[0.0_f32]).unwrap();
        // SAFETY: 固定内核只读两参数和一状态。
        // 同会话输出独占一个f32元素。
        unsafe {
            kernel
                .launch(&meta, &parameters, &state, &mut output, [0, 0, 0, 1])
                .unwrap();
        }
        let mut values = [0.0_f32];
        output.read_range_into(0, &mut values).unwrap();
        assert_eq!(values[0], a.mul_add(b, -1.0));
        assert_ne!(values[0], a * b - 1.0);
    }

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn preserves_f64_buffer_width_in_fixed_kernel_abi() {
        let session = TransferSession::new(0).unwrap();
        let source = SOURCE.replace("float", "double");
        assert!(matches!(
            SynchronousKernel::compile(&session, &source, "adapter_probe"),
            Err(TransferError::Backend(ProbeError::Compilation {
                stage: "physics-precision-ptx",
                ..
            }))
        ));
        // Only this ABI test opts out; production compile always checks PTX.
        let kernel =
            SynchronousKernel::compile_checked(&session, &source, "adapter_probe", false).unwrap();
        let meta = session.upload(&[0i32]).unwrap();
        let parameters = session.upload(&[2.0f64.powi(-40)]).unwrap();
        let state = session.upload(&[1.0f64, -1.0]).unwrap();
        let mut output = session.upload(&[0.0f64; 2]).unwrap();
        // SAFETY: This trusted shader uses double pointers and two elements in
        // same-session f64 state/output buffers. Metadata is one i32 element.
        unsafe {
            kernel
                .launch(&meta, &parameters, &state, &mut output, [0, 0, 0, 2])
                .unwrap();
        }
        let mut values = [0.0f64; 2];
        output.read_range_into(0, &mut values).unwrap();
        assert_eq!(values, [1.0 + 2.0f64.powi(-40), -1.0 + 2.0f64.powi(-40)]);
        assert_ne!(values[0], 1.0);
    }
    const SOURCE: &str = r#"
extern "C" __global__ void adapter_probe(const int* m,const float* p,const float* s,
    float* o,unsigned a,unsigned b,unsigned c,unsigned n) {
  unsigned i=blockIdx.x*blockDim.x+threadIdx.x;
  if(i<n) o[i]=float(m[0])+p[0]+s[i]+float(a+b+c);
}
"#;

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn rejects_foreign_sessions_empty_buffers_and_invalid_work_without_quarantine() {
        let session = TransferSession::new(0).unwrap();
        let other = TransferSession::new(0).unwrap();
        let kernel = SynchronousKernel::compile(&session, SOURCE, "adapter_probe").unwrap();
        let meta = session.upload(&[3i32]).unwrap();
        let foreign = other.upload(&[3i32]).unwrap();
        let parameters = session.upload(&[2.0f32]).unwrap();
        let state = session.upload(&[1.0f32, 2.0, 3.0]).unwrap();
        let empty = session.upload::<f32>(&[]).unwrap();
        let mut output = session.upload(&[0.0f32; 3]).unwrap();
        // SAFETY: This trusted kernel uses three state/output elements; rejected
        // paths return before launch and do not dereference any buffer pointer.
        unsafe {
            assert_eq!(
                kernel.launch(&foreign, &parameters, &state, &mut output, [1, 2, 3, 3]),
                Err(TransferError::SessionMismatch)
            );
            assert!(matches!(
                kernel.launch(&meta, &parameters, &empty, &mut output, [1, 2, 3, 3]),
                Err(TransferError::Input(_))
            ));
            for count in [0, u32::MAX] {
                assert!(matches!(
                    kernel.launch(&meta, &parameters, &state, &mut output, [1, 2, 3, count]),
                    Err(TransferError::Input(_))
                ));
            }
            kernel
                .launch(&meta, &parameters, &state, &mut output, [1, 2, 3, 3])
                .unwrap();
        }
        drop(kernel);
        let mut values = [0.0; 3];
        output.read_range_into(0, &mut values).unwrap();
        assert_eq!(values, [12.0, 13.0, 14.0]);
    }

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn preserves_three_independent_buffer_widths_in_mixed_kernel_abi() {
        let session = TransferSession::new(0).unwrap();
        let source = SOURCE
            .replace("const float* p", "const double* p")
            .replace("float* o", "int* o");
        let kernel =
            SynchronousKernel::compile_checked(&session, &source, "adapter_probe", false).unwrap();
        let meta = session.upload(&[3i32]).unwrap();
        let parameters = session.upload(&[2.0f64]).unwrap();
        let state = session.upload(&[1.0f32, 2.0, 3.0]).unwrap();
        let mut output = session.upload(&[-1i32; 3]).unwrap();
        // SAFETY: The trusted shader reads one i32, one f64 and three f32
        // elements, then writes three i32 elements in the same session.
        unsafe {
            kernel
                .launch(&meta, &parameters, &state, &mut output, [1, 2, 3, 3])
                .unwrap();
        }
        let mut values = [0i32; 3];
        output.read_range_into(0, &mut values).unwrap();
        assert_eq!(values, [12, 13, 14]);
    }

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn checks_hessian_six_buffers_and_fixed_abi_without_quarantine() {
        let session = TransferSession::new(0).unwrap();
        let other = TransferSession::new(0).unwrap();
        let source = r#"
extern "C" __global__ void hessian_probe(const int* m,const float* p,const float* x,
    const float* e,const int* v,float* o,unsigned n) {
  unsigned i=blockIdx.x*blockDim.x+threadIdx.x;
  if(i<n) o[i]=float(m[0])+p[0]+x[i]+e[i]+float(v[i]);
}
"#;
        let kernel = SynchronousKernel::compile(&session, source, "hessian_probe").unwrap();
        let m = session.upload(&[3i32]).unwrap();
        let p = session.upload(&[2.0f32]).unwrap();
        let x = session.upload(&[1.0f32, 2.0, 3.0]).unwrap();
        let e = session.upload(&[4.0f32; 3]).unwrap();
        let v = session.upload(&[1i32; 3]).unwrap();
        let mut output = session.upload(&[0.0f32; 3]).unwrap();
        let fm = other.upload(&[0i32]).unwrap();
        let fp = other.upload(&[0.0f32]).unwrap();
        let fx = other.upload(&[0.0f32; 3]).unwrap();
        let fe = other.upload(&[0.0f32; 3]).unwrap();
        let fv = other.upload(&[0i32; 3]).unwrap();
        let mut fo = other.upload(&[0.0f32; 3]).unwrap();
        let empty = session.upload::<f32>(&[]).unwrap();
        // SAFETY: The trusted seven-argument shader reads three entries.
        // Six rejection cases return before launch; one accepted case uses
        // matching sessions and bounded, distinct input/output allocations.
        unsafe {
            for k in 0..6 {
                assert_eq!(
                    kernel.launch_flex_hessian(
                        (if k == 0 { &fm } else { &m }, if k == 1 { &fp } else { &p }),
                        if k == 2 { &fx } else { &x },
                        if k == 3 { &fe } else { &e },
                        if k == 4 { &fv } else { &v },
                        if k == 5 { &mut fo } else { &mut output },
                        3
                    ),
                    Err(TransferError::SessionMismatch)
                );
            }
            for worlds in [0, u32::MAX] {
                assert!(matches!(
                    kernel.launch_flex_hessian((&m, &p), &x, &e, &v, &mut output, worlds),
                    Err(TransferError::Input(_))
                ));
            }
            assert!(matches!(
                kernel.launch_flex_hessian((&m, &empty), &x, &e, &v, &mut output, 3),
                Err(TransferError::Input(_))
            ));
            kernel
                .launch_flex_hessian((&m, &p), &x, &e, &v, &mut output, 3)
                .unwrap();
        }
        let mut values = [0.0; 3];
        output.read_range_into(0, &mut values).unwrap();
        assert_eq!(values, [11.0, 12.0, 13.0]);
    }

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn compiler_errors_do_not_quarantine_transfers() {
        let session = TransferSession::new(0).unwrap();
        assert!(matches!(
            SynchronousKernel::compile(&session, "invalid cuda source", "bad"),
            Err(TransferError::Backend(ProbeError::Compilation { .. }))
        ));
        assert!(session.upload(&[1.0]).is_ok());
        let kernel = SynchronousKernel::compile(&session, SOURCE, "adapter_probe").unwrap();
        // Simulate completion uncertainty without an actual GPU failure. This
        // exercises module retention; the old transfer tests cover buffer owners.
        session
            .inner
            .status
            .uncertain
            .store(true, std::sync::atomic::Ordering::Release);
        drop(kernel);
        session
            .inner
            .status
            .uncertain
            .store(false, std::sync::atomic::Ordering::Release);
    }
}
