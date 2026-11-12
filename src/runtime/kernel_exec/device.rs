//! 设备内原语、执行控制与诊断（A0 §2 第 4/5 组 / D/E/H/I/K/L 域）。
//!
//! 分组：
//! - [`DeviceMath`]：E.1/E.2 的数学内建（设备端单态化，不追求 `dyn`）；
//! - [`Atomics`]：H.1/H.3/H.4/H.5/H.6（opaque 地址，不做宿主 `unsafe` 解引用）；
//! - [`ThreadScope`]：I.1/I.2/I.3（块内线程号/共享存储/栅栏）；
//! - [`ExecControl`]：K.2/K.3（设备端条件循环/分支；带闭包，不追求 `dyn`）；
//! - [`Diagnostics`]：L.2/L.3（事件计时、内核打印退路）。
//!
//! 本文件全部为声明；实际指针语义、屏障实现与打印通道由各后端适配层承担。

pub use core::time::Duration;

use super::buffer::Address64;
use super::error::DeviceError;
use super::queue::Arg;

/// 三维线程/块尺寸（I.1）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Dim3 {
    /// x。
    pub x: u32,
    /// y。
    pub y: u32,
    /// z。
    pub z: u32,
}

impl Dim3 {
    /// 构造。
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    /// 标量填充。
    pub const fn splat(value: u32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }

    /// 零尺寸（占位）。
    pub const ZERO: Self = Self::new(0, 0, 0);
}

/// 设备端计数器句柄（K.2/K.3 的条件来源，A0 §2）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Counter {
    /// opaque 编号。
    pub handle: u64,
}

/// 设备事件句柄（L.2）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Event {
    /// opaque 编号。
    pub handle: u64,
}

/// 数学内建（E.1/E.2）：设备端按具体后端单态化，静态方法不进入 `dyn` 面。
pub trait DeviceMath {
    /// 绝对值。
    fn abs(x: f32) -> f32;
    /// 最大值。
    fn max(a: f32, b: f32) -> f32;
    /// 最小值。
    fn min(a: f32, b: f32) -> f32;
    /// 区间夹取。
    fn clamp(x: f32, lo: f32, hi: f32) -> f32;
    /// 符号。
    fn sign(x: f32) -> f32;
    /// 幂。
    fn pow(x: f32, y: f32) -> f32;
    /// 指数。
    fn exp(x: f32) -> f32;
    /// 对数。
    fn log(x: f32) -> f32;
    /// 向下取整。
    fn floor(x: f32) -> f32;
    /// 向上取整。
    fn ceil(x: f32) -> f32;
    /// 四舍五入。
    fn round(x: f32) -> f32;
    /// NaN 判定。
    fn isnan(x: f32) -> bool;
    /// 平方根。
    fn sqrt(x: f32) -> f32;
    /// 正弦。
    fn sin(x: f32) -> f32;
    /// 余弦。
    fn cos(x: f32) -> f32;
    /// 正切。
    fn tan(x: f32) -> f32;
    /// 反正弦。
    fn asin(x: f32) -> f32;
    /// 反余弦。
    fn acos(x: f32) -> f32;
    /// 反正切。
    fn atan2(y: f32, x: f32) -> f32;
}

/// 原子原语（H.1/H.3/H.4/H.5/H.6）：地址为 opaque 设备地址。
///
/// 约定：`add_f32` 在设备无原生 f32 原子时由后端走 CAS 循环（A0 §2 退路）；
/// 本层不暴露裸指针，宿主侧不产生 `unsafe` 解引用。
pub trait Atomics {
    /// i32 原子加，返回旧值。
    fn add_i32(&self, target: Address64, value: i32) -> i32;
    /// f32 原子加，返回旧值（退：CAS 循环）。
    fn add_f32(&self, target: Address64, value: f32) -> f32;
    /// i32 原子减，返回旧值。
    fn sub_i32(&self, target: Address64, value: i32) -> i32;
    /// u32 原子或，返回旧值。
    fn or_u32(&self, target: Address64, value: u32) -> u32;
    /// i32 原子极小，返回旧值。
    fn min_i32(&self, target: Address64, value: i32) -> i32;
    /// i32 原子极大，返回旧值。
    fn max_i32(&self, target: Address64, value: i32) -> i32;
    /// i32 比较交换，返回旧值。
    fn cas_i32(&self, target: Address64, compare: i32, value: i32) -> i32;
}

/// 块内作用域（I.1/I.2/I.3）。
pub trait ThreadScope {
    /// 块内线程号（I.1）。
    fn tid(&self) -> Dim3;
    /// 块尺寸（I.1）。
    fn block_dim(&self) -> Dim3;
    /// 块内屏障（I.3；I.2 的共享存储由实现层按块声明）。
    fn barrier(&self);
}

/// 设备端执行控制（K.2/K.3）。带闭包，不追求 `dyn` 兼容。
pub trait ExecControl {
    /// 设备端条件循环：条件来自设备上的 i32 计数器（K.2）。
    fn loop_while_device(&self, counter: &Counter, body: impl FnMut()) -> Result<(), DeviceError>;
    /// 设备端条件分支（K.3）。
    fn branch_on_device(
        &self,
        counter: &Counter,
        then: impl FnOnce(),
        els: impl FnOnce(),
    ) -> Result<(), DeviceError>;
}

/// 诊断（L.2/L.3）：事件计时；打印在无 printf 后端退化为状态缓冲 + 宿主打印。
pub trait Diagnostics {
    /// 记录事件（L.2）。
    fn record_event(&self) -> Event;
    /// 两事件间耗时（L.2）。
    fn elapsed(&self, a: &Event, b: &Event) -> Duration;
    /// 内核内格式化打印（L.3 退：状态缓冲）。
    fn printf(&self, fmt: &str, args: &[Arg]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim3_constructors_are_explicit() {
        assert_eq!(Dim3::new(1, 2, 3).y, 2);
        assert_eq!(Dim3::splat(32), Dim3::new(32, 32, 32));
        assert_eq!(Dim3::ZERO, Dim3::new(0, 0, 0));
    }
}
