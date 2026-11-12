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

/// 退项集合（A.2；P1-b 冻结 12 flag 名单与极性）。
///
/// **极性（与直觉相反，必须按此读）**：`true` = 该能力有原生实现（走原生路径）；
/// `false` = 走退路（退路语义见每个字段的文档）。
///
/// 12 flag = 10 个「退」项（`transient_alloc` / `zero_copy_external` /
/// `generic_device_fn` / `native_source_injection` / `occupancy_hint` /
/// `graph_capture` / `device_loop` / `device_branch` / `capture_alloc` /
/// `kernel_printf`）+ 2 个「必带退」项（`int64` / `f32_atomic`）。
///
/// 「必带退」的含义：能力本身是必级，但退法常备、不能在缺原生实现时静默失败——
/// `int64=false` 时退法=双 32 位表示；`f32_atomic=false` 时退法=比较交换循环。
/// 字段名一律用**能力名**，不退项机制名，避免与极性冲突。缺失且无退路 → `Unsupported`。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FallbackSet {
    /// B.10：临时分配不触发同步。`false` → 退路=预分配池。
    pub transient_alloc: bool,
    /// B.11：零拷贝借给外部框架。`false` → 退路=拷贝。
    pub zero_copy_external: bool,
    /// D.4：泛型设备函数。`false` → 退路=按类型单态化展开。
    pub generic_device_fn: bool,
    /// D.10：注入设备原生片段。`false` → 退路=改用 I.3（barrier）。
    pub native_source_injection: bool,
    /// G.9：建议块宽/最小驻留块数查询。`false` → 退路=固定值。
    pub occupancy_hint: bool,
    /// K.1：录制/重放。`false` → 退路=逐次提交。
    pub graph_capture: bool,
    /// K.2：设备端条件循环。`false` → 退路=固定次数。
    pub device_loop: bool,
    /// K.3：设备端条件分支。`false` → 退路=无条件执行。
    pub device_branch: bool,
    /// K.5：录制期分配。`false` → 退路=录制前预分配。
    pub capture_alloc: bool,
    /// L.3：内核内格式化打印。`false` → 退路=状态缓冲 + 宿主打印。
    pub kernel_printf: bool,
    /// C.1：64 位整数原生。`false` → 退路=双 32 位表示（必带退）。
    pub int64: bool,
    /// H.1：f32 原子加原生。`false` → 退路=比较交换循环（必带退）。
    pub f32_atomic: bool,
}

impl FallbackSet {
    /// 全部退项均有原生实现（全功能后端）。
    pub const fn all() -> Self {
        Self {
            transient_alloc: true,
            zero_copy_external: true,
            generic_device_fn: true,
            native_source_injection: true,
            occupancy_hint: true,
            graph_capture: true,
            device_loop: true,
            device_branch: true,
            capture_alloc: true,
            kernel_printf: true,
            int64: true,
            f32_atomic: true,
        }
    }

    /// 全部退项走退路（最小后端；`false` 语义见类型文档）。
    pub const fn none() -> Self {
        Self::minimal_set()
    }

    const fn minimal_set() -> Self {
        Self {
            transient_alloc: false,
            zero_copy_external: false,
            generic_device_fn: false,
            native_source_injection: false,
            occupancy_hint: false,
            graph_capture: false,
            device_loop: false,
            device_branch: false,
            capture_alloc: false,
            kernel_printf: false,
            int64: false,
            f32_atomic: false,
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
            fallbacks: FallbackSet::minimal_set(),
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
        // 12 flag 逐项：all() 必须全 true（true=原生可用）。
        assert!(all.transient_alloc);
        assert!(all.zero_copy_external);
        assert!(all.generic_device_fn);
        assert!(all.native_source_injection);
        assert!(all.occupancy_hint);
        assert!(all.graph_capture);
        assert!(all.device_loop);
        assert!(all.device_branch);
        assert!(all.capture_alloc);
        assert!(all.kernel_printf);
        assert!(all.int64);
        assert!(all.f32_atomic);
        let none = FallbackSet::none();
        assert_eq!(none, FallbackSet::default());
        assert!(!none.transient_alloc && !none.int64 && !none.f32_atomic);
        let caps = DeviceCaps::minimal();
        assert_eq!(caps.max_index_dims, 1);
        assert!(!caps.fallbacks.capture_alloc);
        assert_eq!(caps.numeric, NumericPolicy::FROZEN_F32);
    }
}
