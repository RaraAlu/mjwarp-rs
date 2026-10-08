//! 内部固定ABI内核适配。
//! 不提供安全的任意内核入口。

use super::{TransferBuffer, TransferElement, TransferSession};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use cudarc::{
    driver::{CudaFunction, LaunchConfig, PushKernelArg},
    nvrtc::{CompileOptions, compile_ptx_with_opts, sys},
};
use std::sync::Arc;

/// ABI: (const int*, const P*, const S*, O*, u32, u32, u32, u32).
/// 柔体边另用固定三输入ABI。
/// 拉伸矩阵另用六指针ABI。
/// 调用方证明三种元素的实际ABI。
/// 核函数与缓冲保留同一会话。
pub(crate) struct SynchronousKernel {
    session: TransferSession,
    function: Option<CudaFunction>,
}

impl SynchronousKernel {
    pub(crate) fn compile(
        session: &TransferSession,
        source: &str,
        entry: &'static str,
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
                    fmad: Some(false),
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
    /// Caller proves this exact ABI, checked strides and every buffer access.
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn preserves_f64_buffer_width_in_fixed_kernel_abi() {
        let session = TransferSession::new(0).unwrap();
        let source = SOURCE.replace("float", "double");
        let kernel = SynchronousKernel::compile(&session, &source, "adapter_probe").unwrap();
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
        let kernel = SynchronousKernel::compile(&session, &source, "adapter_probe").unwrap();
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
