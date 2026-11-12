//! mock module（仅 `cfg(test)`）：P1-c 的 G.9 双路注入点。
//!
//! `occupancy_hint=true` 走 mock 原生桩 `{512,2}`；`false` 走冻结退路 `{256,1}`。
//! 编译入口语义与 P1-b 一致（Inline 必填 `entry`；Named 由后端按名解析）。

use super::backend::DeviceCaps;
use super::error::{CompileError, DeviceError};
use super::module::{BlockSuggestion, CompileOptions, Kernel, KernelSource, Module};

pub(crate) struct MockModule {
    caps: DeviceCaps,
}

impl MockModule {
    pub(crate) fn new(caps: DeviceCaps) -> Self {
        Self { caps }
    }
}

impl Module for MockModule {
    fn compile(&self, src: KernelSource, opts: CompileOptions) -> Result<Kernel, DeviceError> {
        match src {
            KernelSource::Named("") => Err(DeviceError::Compile(CompileError::new(
                "empty kernel source name",
            ))),
            KernelSource::Inline("") => Err(DeviceError::Compile(CompileError::new(
                "empty inline kernel source",
            ))),
            KernelSource::Inline(_) => {
                if opts.entry.is_none() {
                    return Err(DeviceError::Compile(CompileError::new(
                        "inline kernel source requires an entry name",
                    )));
                }
                Ok(Kernel::from_handle(42))
            }
            KernelSource::Named(name) => {
                let _ = name;
                Ok(Kernel::from_handle(42))
            }
        }
    }

    fn suggest_block_width(
        &self,
        _kernel: &Kernel,
        _work_items: u64,
    ) -> Result<BlockSuggestion, DeviceError> {
        if self.caps.fallbacks.occupancy_hint {
            Ok(BlockSuggestion {
                block_width: 512,
                min_blocks: 2,
            })
        } else {
            Ok(BlockSuggestion::FALLBACK)
        }
    }
}
