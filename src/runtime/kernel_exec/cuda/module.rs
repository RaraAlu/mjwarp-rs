//! CUDA 模块/编译（F 域，P1-b）：NVRTC 现场编译 + 载入 + G.9 occupancy 查询。
//!
//! 与现有 L1 同步内核路径同源：`--std=c++17`、`--gpu-architecture=compute_70`、fmad 开、
//! fast-math 关、`check_physics_ptx` F.6 精度闸；但不复用其 `unsafe` 固定 ABI 发射路径，
//! 旧 `runtime::transfer::kernel` 一行不动（双轨）。
//!
//! P1-b 只接受 [`KernelSource::Inline`] + 必填入口名；具名源（recipe/冻结 PTX 路由）返回
//! `Unsupported`，属 P1-c/P3。

use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use cudarc::nvrtc::{CompileOptions as NvrtcOptions, Ptx, compile_ptx_with_opts};

use crate::runtime::kernel_exec::error::{DeviceError, LaunchError};
use crate::runtime::kernel_exec::module::{
    BlockSuggestion, CompileOptions, Kernel, KernelSource, Module,
};
use crate::runtime::transfer::kernel::check_physics_ptx;

use super::errors;
use super::{CudaDeviceInner, require_frozen_policy};

/// NVRTC 编译选项与现有 L1 路径一致（F.6 冻结配方）。
fn nvrtc_options(entry: &'static str) -> NvrtcOptions {
    NvrtcOptions {
        fmad: Some(true),
        use_fast_math: Some(false),
        options: vec![
            "--std=c++17".to_owned(),
            "--gpu-architecture=compute_70".to_owned(),
        ],
        name: Some(format!("{entry}.cu")),
        ..Default::default()
    }
}

fn compile_source(source: &str, entry: &'static str) -> Result<Ptx, DeviceError> {
    compile_ptx_with_opts(source, nvrtc_options(entry))
        .map_err(|error| errors::compile(format!("nvrtc {entry}: {error:?}")))
}

/// G.9 的 occupancy 回调：无动态共享内存。
extern "C" fn zero_shared_memory(_block_size: std::ffi::c_int) -> usize {
    0
}

/// 模块/编译入口（F.1/F.5/F.6）。
pub(crate) struct CudaModule {
    dev: Arc<CudaDeviceInner>,
}

impl CudaModule {
    pub(crate) fn new(dev: Arc<CudaDeviceInner>) -> Self {
        Self { dev }
    }
}

impl Module for CudaModule {
    fn compile(&self, src: KernelSource, opts: CompileOptions) -> Result<Kernel, DeviceError> {
        let KernelSource::Inline(source) = src else {
            return Err(errors::unsupported("F.1 named-source"));
        };
        require_frozen_policy(opts.numeric)?;
        if !opts.defines.is_empty() {
            return Err(errors::unsupported("F.5 defines"));
        }
        let entry = opts
            .entry
            .ok_or_else(|| errors::compile("inline kernel source requires an entry name"))?;
        if !crate::runtime::presence::nvrtc_library_present() {
            return Err(errors::compile("NVRTC library unavailable"));
        }
        // 缺 NVRTC 库时 cudarc 动态加载可能 panic：先做库存在性预检，再捕获残余 panic。
        let ptx = std::panic::catch_unwind(AssertUnwindSafe(|| compile_source(source, entry)))
            .map_err(|_| errors::compile("cudarc panicked during NVRTC compile"))??;
        let generated = ptx.to_src();
        check_physics_ptx(&generated, entry).map_err(errors::from_probe)?;
        let module = self
            .dev
            .context
            .load_module(ptx)
            .map_err(|error| errors::compile(format!("cuda-module-load {entry}: {error:?}")))?;
        let function = module
            .load_function(entry)
            .map_err(|error| errors::compile(format!("cuda-function-load {entry}: {error:?}")))?;
        let handle = self.dev.kernels.insert(function);
        Ok(Kernel::from_handle(handle))
    }

    fn suggest_block_width(
        &self,
        kernel: &Kernel,
        work_items: u64,
    ) -> Result<BlockSuggestion, DeviceError> {
        // P1-b：occupancy 原生查询不使用工作项数；工作项数只影响退路常量的选择（暂固定）。
        let _ = work_items;
        if !self.dev.caps.fallbacks.occupancy_hint {
            return Ok(BlockSuggestion::FALLBACK);
        }
        let function = self.dev.kernels.get(kernel.handle()).ok_or_else(|| {
            DeviceError::Launch(LaunchError::InvalidArgument(format!(
                "unknown kernel handle {}",
                kernel.handle()
            )))
        })?;
        match function.occupancy_max_potential_block_size(
            zero_shared_memory,
            0,
            self.dev.caps.max_block_threads,
            None,
        ) {
            Ok((min_blocks, block_width)) if block_width > 0 => Ok(BlockSuggestion {
                block_width,
                min_blocks,
            }),
            Ok(_) => Ok(BlockSuggestion::FALLBACK),
            Err(error) => Err(errors::device_lost("cuda-occupancy", error)),
        }
    }
}
