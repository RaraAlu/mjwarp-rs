//! 同步内核的求解类发射入口。

use super::*;

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
}
