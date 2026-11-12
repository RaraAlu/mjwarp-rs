//! 发现与能力（A0 §2 第 1 组 / A.1/A.2/A.3 载体）。
//!
//! 后端与设备都以 trait object 暴露：消费方只持有 `Box<dyn Device>`，换后端不改调用方。

use super::buffer::Allocator;
use super::error::DeviceError;
use super::module::Module;
use super::queue::Queue;

/// 设备标识（后端口径，非全局索引）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct DeviceId(pub u32);

/// 能力编号（用于能力缺失时的显式失败）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CapabilityId(pub &'static str);

/// 乘加（FMA）策略：冻结精度契约要求可显式控制（F.6）。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FmaMode {
    /// 允许 fused multiply-add。
    Enabled,
    /// 禁止 FMA（需要时由实现层拆成独立 mul/add）。
    Disabled,
}

/// 数值策略（F.6）：随编译/发射传递，后端不得自行变更。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumericPolicy {
    /// 全程 f32（禁止隐式升精度）。
    pub f32_only: bool,
    /// 乘加策略。
    pub fma: FmaMode,
    /// 是否启用 fast-math（冻结配方要求关闭）。
    pub fast_math: bool,
}

impl NumericPolicy {
    /// 冻结配方对齐的默认策略：f32、FMA 开、fast-math 关。
    pub const FROZEN_F32: Self = Self {
        f32_only: true,
        fma: FmaMode::Enabled,
        fast_math: false,
    };

    /// 显式构造。
    pub const fn new(f32_only: bool, fma: FmaMode, fast_math: bool) -> Self {
        Self {
            f32_only,
            fma,
            fast_math,
        }
    }
}

impl Default for NumericPolicy {
    fn default() -> Self {
        Self::FROZEN_F32
    }
}

/// 退项集合（A.2）：每项声明后端是否有原生实现；缺失且无退路 → `Unsupported`。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FallbackSet {
    /// B.10：临时分配不触发同步（退：预分配）。
    pub uninit_alloc: bool,
    /// B.11：零拷贝借给外部框架（退：拷贝）。
    pub zero_copy_external: bool,
    /// D.4：泛型设备函数（退：按类型展开）。
    pub generic_device_fn: bool,
    /// G.9：块宽/驻留查询（退：固定值）。
    pub occupancy_hint: bool,
    /// K.1：录制/重放（退：逐次提交）。
    pub graph_capture: bool,
    /// K.2：设备端条件循环（退：固定次数）。
    pub device_loop: bool,
    /// K.3：设备端条件分支（退：无条件执行）。
    pub device_branch: bool,
    /// K.5：录制期分配（退：录制前预分配）。
    pub capture_alloc: bool,
    /// L.3：内核内格式化打印（退：状态缓冲）。
    pub kernel_printf: bool,
    /// D.10：注入设备原生片段（退：改用 I.3）。
    pub native_source_injection: bool,
    /// C.1 的 64 位整数退路：双 32 位表示。
    pub i64_as_two_i32: bool,
    /// H.1 的 f32 原子退路：比较交换循环。
    pub f32_atomic_via_cas: bool,
}

impl FallbackSet {
    /// 全部退项均有实现（全功能后端）。
    pub const fn all() -> Self {
        Self {
            uninit_alloc: true,
            zero_copy_external: true,
            generic_device_fn: true,
            occupancy_hint: true,
            graph_capture: true,
            device_loop: true,
            device_branch: true,
            capture_alloc: true,
            kernel_printf: true,
            native_source_injection: true,
            i64_as_two_i32: true,
            f32_atomic_via_cas: true,
        }
    }
}

/// 设备能力（A.2 + B/G/I 域容量）。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceCaps {
    /// 退项：是否有原生实现。
    pub fallbacks: FallbackSet,
    /// 最大索引维度（1..=3）。
    pub max_index_dims: u8,
    /// 单块最大线程数。
    pub max_block_threads: u32,
    /// 最大块数。
    pub max_blocks: u32,
    /// 每块共享存储上限（字节）。
    pub shared_bytes_per_block: u32,
    /// 单缓冲上限（字节）。
    pub max_buffer_bytes: u64,
    /// 数值策略（F.6）。
    pub numeric: NumericPolicy,
}

impl DeviceCaps {
    /// 保守的能力表：1 维、单线程块上限很小——用于 mock/未知后端。
    pub const fn minimal() -> Self {
        Self {
            fallbacks: FallbackSet {
                uninit_alloc: false,
                zero_copy_external: false,
                generic_device_fn: false,
                occupancy_hint: false,
                graph_capture: false,
                device_loop: false,
                device_branch: false,
                capture_alloc: false,
                kernel_printf: false,
                native_source_injection: false,
                i64_as_two_i32: false,
                f32_atomic_via_cas: false,
            },
            max_index_dims: 1,
            max_block_threads: 1,
            max_blocks: 1,
            shared_bytes_per_block: 0,
            max_buffer_bytes: 0,
            numeric: NumericPolicy::FROZEN_F32,
        }
    }
}

/// 后端（A.1）：枚举设备并打开。
pub trait Backend {
    /// 枚举可用设备。
    fn devices(&self) -> Result<Vec<DeviceId>, DeviceError>;
    /// 打开设备。
    fn open(&self, id: &DeviceId) -> Result<Box<dyn Device>, DeviceError>;
}

/// 设备（A.2/A.3 载体）：能力表，以及队列/分配器/模块入口。
pub trait Device {
    /// 能力表。
    fn caps(&self) -> DeviceCaps;
    /// 新队列（提交顺序由队列保证，A.3/G.10）。
    fn queue(&self) -> Result<Box<dyn Queue>, DeviceError>;
    /// 分配器（B 域）。
    fn allocator(&self) -> Box<dyn Allocator>;
    /// 模块/编译入口（F 域）。
    fn module(&self) -> Box<dyn Module>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_numeric_policy_is_f32_fma_without_fast_math() {
        let policy = NumericPolicy::FROZEN_F32;
        assert!(policy.f32_only);
        assert_eq!(policy.fma, FmaMode::Enabled);
        assert!(!policy.fast_math);
        assert_eq!(NumericPolicy::default(), policy);
    }

    #[test]
    fn fallback_set_all_and_minimal_caps() {
        let all = FallbackSet::all();
        assert!(all.graph_capture && all.kernel_printf && all.f32_atomic_via_cas);
        let caps = DeviceCaps::minimal();
        assert_eq!(caps.max_index_dims, 1);
        assert!(!caps.fallbacks.capture_alloc);
        assert_eq!(caps.numeric, NumericPolicy::FROZEN_F32);
    }
}
