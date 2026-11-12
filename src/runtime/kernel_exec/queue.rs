//! 队列与发射（A0 §2 第 3 组 / A.3、G 域、L.1）。
//!
//! 同步语义：`submit` 不隐式同步；需要排空必须显式 [`Queue::wait_idle`]（A0 §2 设计约束 2）。
//! `capture` 带闭包，单列到 [`QueueCapture`]，因此 [`Queue`] 本身保持对象安全。

use super::backend::NumericPolicy;
use super::buffer::{Address64, Buffer};
use super::error::DeviceError;
use super::module::Kernel;

/// 索引空间维度（G.1）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IndexSpace {
    /// 1 维。
    D1,
    /// 2 维。
    D2,
    /// 3 维。
    D3,
}

impl IndexSpace {
    /// 维度数（1..=3）。
    pub const fn rank(self) -> u8 {
        match self {
            Self::D1 => 1,
            Self::D2 => 2,
            Self::D3 => 3,
        }
    }
}

/// 内核参数（G.3）：缓冲用 opaque 地址 + 偏移，标量用具体类型。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Arg {
    /// 缓冲参数。
    Buffer {
        /// 缓冲地址。
        address: Address64,
        /// 元素/字节偏移（由 ABI 层解释）。
        offset: u64,
    },
    /// i32 标量。
    I32(i32),
    /// u32 标量。
    U32(u32),
    /// u64 标量。
    U64(u64),
    /// f32 标量。
    F32(f32),
}

/// 输入/输出别名声明（G.5）：同一缓冲可同时作输入与输出。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Alias {
    /// 输入地址。
    pub input: Address64,
    /// 输出地址。
    pub output: Address64,
    /// 相对偏移。
    pub offset: u64,
}

/// 一次发射的描述（G.1/G.2/G.3/G.5/G.6/G.7/F.6）。
#[derive(Clone, Debug, PartialEq)]
pub struct Launch {
    /// 已编译内核。
    pub kernel: Kernel,
    /// 索引空间。
    pub dims: IndexSpace,
    /// 世界数（G.2：总索引 = worlds × elements）。
    pub worlds: u32,
    /// 每世界元素数。
    pub elements: u32,
    /// 块数（G.6）。
    pub blocks: u32,
    /// 块宽（G.7）。
    pub block_width: u32,
    /// 参数表。
    pub args: Vec<Arg>,
    /// 别名声明（G.5）。
    pub aliases: Vec<Alias>,
    /// 数值策略（F.6；随发射传递）。
    pub numeric: NumericPolicy,
}

impl Launch {
    /// 最小合法发射（供测试/骨架使用；不做能力校验）。
    pub fn new(kernel: Kernel) -> Self {
        Self {
            kernel,
            dims: IndexSpace::D1,
            worlds: 1,
            elements: 1,
            blocks: 1,
            block_width: 1,
            args: Vec::new(),
            aliases: Vec::new(),
            numeric: NumericPolicy::FROZEN_F32,
        }
    }
}

/// 提交句柄（命令队列内顺序由队列保证）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Submission(pub u64);

/// 录制图句柄（K.1）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Graph(pub u64);

/// 队列（A.3/G.10/L.1）：对象安全，消费方以 `Box<dyn Queue>` 持有。
pub trait Queue {
    /// 提交发射，不隐式同步（A0 §2 设计约束 2）。
    fn submit(&self, launch: Launch) -> Result<Submission, DeviceError>;
    /// 显式排空（L.1）：返回后此前提交的结果可用。
    fn wait_idle(&self) -> Result<(), DeviceError>;
    /// 重放录制图（K.1）。
    fn replay(&self, graph: &Graph) -> Result<(), DeviceError>;
    /// 录制期分配（K.5 退：录制前预分配）。
    fn capture_alloc(&self) -> Result<Box<dyn Buffer>, DeviceError>;
}

/// 图录制扩展（K.1）。带闭包，不追求 `dyn` 兼容；按具体后端单态化使用。
pub trait QueueCapture {
    /// 在捕获区间内执行闭包，返回可重放的图。
    fn capture<F>(&self, body: F) -> Result<Graph, DeviceError>
    where
        F: FnOnce(&dyn Queue);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::kernel_exec::module::Kernel;

    #[test]
    fn index_space_ranks_are_explicit() {
        assert_eq!(IndexSpace::D1.rank(), 1);
        assert_eq!(IndexSpace::D2.rank(), 2);
        assert_eq!(IndexSpace::D3.rank(), 3);
    }

    #[test]
    fn launch_defaults_use_frozen_numeric_policy() {
        let launch = Launch::new(Kernel::from_handle(1));
        assert_eq!(launch.dims, IndexSpace::D1);
        assert_eq!(launch.numeric, NumericPolicy::FROZEN_F32);
        assert!(launch.args.is_empty());
    }
}
