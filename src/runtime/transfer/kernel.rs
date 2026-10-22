//! 内部固定ABI内核适配。
//! 不提供安全的任意内核入口。

use super::{TransferBuffer, TransferElement, TransferSession};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use cudarc::{
    driver::{CudaFunction, LaunchConfig, PushKernelArg, result},
    nvrtc::{CompileOptions, compile_ptx_with_opts, sys},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
mod kinetic_energy;

#[cfg(test)]
#[path = "../../../tests/support/t4_rigid_capture.rs"]
mod recapture;

#[cfg(test)]
#[path = "../../../tests/support/launcher_compile_baseline.rs"]
mod compile_baseline;

// 跟随冻结上游的Warp 1.15策略。
// fuse_fp开启FMA，且不启用fast-math。
const PHYSICS_FMAD: bool = true;

// 编译开销基线的输出目录。
// 未设置时不计时、不查询显存、不写文件。
const COMPILE_BASELINE_DIR: &str = "MJWARP_LAUNCHER_BASELINE_DIR";

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

// 子模块：纯移动自原单文件，行为不变。
mod baseline;
mod drop;
mod launch;
mod launchers_passive;
mod launchers_physics;
mod launchers_solve;
mod ptx;
#[cfg(test)]
use baseline::{
    COMPILE_OCCURRENCES, append_compile_baseline, compile_baseline_record, next_occurrence,
};
use baseline::{CompileSample, LoadSample, record_compile_baseline};
pub(crate) use ptx::check_physics_ptx;
use ptx::record_physics_ptx;
#[cfg(test)]
#[path = "kernel/tests/abi.rs"]
mod tests_abi;
#[cfg(test)]
#[path = "kernel/tests/adapter.rs"]
mod tests_adapter;
#[cfg(test)]
#[path = "kernel/tests/guards.rs"]
mod tests_guards;
#[cfg(test)]
#[path = "kernel/tests/precision.rs"]
mod tests_precision;
#[cfg(test)]
#[path = "kernel/tests/support.rs"]
mod tests_support;

impl SynchronousKernel {
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
        let baseline = std::env::var_os(COMPILE_BASELINE_DIR).map(PathBuf::from);
        Self::compile_measured(session, source, entry, require_f32, baseline.as_deref())
    }

    /// `baseline`为空时与原编译路径等价。
    /// 非空时只追加计时、显存查询与记录文件。
    fn compile_measured(
        session: &TransferSession,
        source: &str,
        entry: &'static str,
        require_f32: bool,
        baseline: Option<&Path>,
    ) -> Result<Self, TransferError> {
        session.inner.status.check()?;
        let compile_started = Instant::now();
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
        let compile = compile_started.elapsed();
        let check_started = Instant::now();
        let generated = (require_f32 || baseline.is_some()).then(|| ptx.to_src());
        if require_f32 {
            let generated = generated.as_deref().unwrap_or_default();
            let precision = check_physics_ptx(generated, entry);
            record_physics_ptx(source, generated, entry, precision.is_ok())?;
            precision?;
        }
        let check = check_started.elapsed();
        let (function, load) = session.run("synchronous-kernel-load", || {
            // 显存查询只在基线模式执行。
            let before = baseline.map(|_| result::mem_get_info()).transpose()?;
            let load_started = Instant::now();
            let function = session
                .inner
                .stream
                .context()
                .load_module(ptx)?
                .load_function(entry)?;
            let elapsed = load_started.elapsed();
            let after = baseline.map(|_| result::mem_get_info()).transpose()?;
            Ok((
                function,
                LoadSample {
                    elapsed,
                    before,
                    after,
                },
            ))
        })?;
        if let (Some(directory), Some(generated)) = (baseline, generated.as_deref()) {
            let sample = CompileSample {
                entry,
                source,
                ptx: generated,
                require_f32,
                compile,
                check,
                load,
            };
            record_compile_baseline(directory, &sample)?;
        }
        Ok(Self {
            session: session.clone(),
            function: Some(function),
        })
    }
}
