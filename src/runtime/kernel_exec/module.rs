//! 编译与内核（A0 §2 第 3 组之一 / F 域）。
//!
//! `Module` 以 trait object 暴露；`Kernel` 是 opaque 句柄（含后端私有编号），
//! 消费方只把它放进 [`super::queue::Launch`]，不解读内部编号。

use super::backend::NumericPolicy;
use super::error::DeviceError;

/// 内核源（F.1）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum KernelSource {
    /// 具名源（后端按名字/内容键解析；对应现有 recipe/冻结源）。
    Named(&'static str),
    /// 内联源（书写面拼接后的单份源）。
    Inline(&'static str),
}

/// 编译选项（F.5/F.6）。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CompileOptions {
    /// 数值策略（必填语义：默认即冻结配方）。
    pub numeric: NumericPolicy,
    /// 预处理宏（按 `(name, value)` 顺序传递）。
    pub defines: Vec<(&'static str, &'static str)>,
}

impl CompileOptions {
    /// 以数值策略构造。
    pub fn new(numeric: NumericPolicy) -> Self {
        Self {
            numeric,
            defines: Vec::new(),
        }
    }

    /// 追加一个预处理宏。
    pub fn with_define(mut self, name: &'static str, value: &'static str) -> Self {
        self.defines.push((name, value));
        self
    }
}

/// 已编译内核的 opaque 句柄。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Kernel {
    handle: u64,
}

impl Kernel {
    /// 由后端实现层构造。
    pub const fn from_handle(handle: u64) -> Self {
        Self { handle }
    }

    /// opaque 编号（仅用于同一后端内等价比较/日志）。
    pub const fn handle(&self) -> u64 {
        self.handle
    }
}

/// 模块/编译入口（F.1/F.4/F.5）。
pub trait Module {
    /// 编译内核源；错误按 `DeviceError::Compile` 分类。
    fn compile(&self, src: KernelSource, opts: CompileOptions) -> Result<Kernel, DeviceError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::kernel_exec::backend::NumericPolicy;

    #[test]
    fn compile_options_default_to_frozen_policy_and_carry_defines() {
        let opts = CompileOptions::default();
        assert_eq!(opts.numeric, NumericPolicy::FROZEN_F32);
        let opts = opts.with_define("MJWARP_FIELD_BATCHES", "1");
        assert_eq!(opts.defines.len(), 1);
        assert_eq!(opts.defines[0].0, "MJWARP_FIELD_BATCHES");
    }

    #[test]
    fn kernel_handle_is_opaque_but_roundtrips() {
        let kernel = Kernel::from_handle(7);
        assert_eq!(kernel.handle(), 7);
    }
}
