use super::{LaunchBounds1, LaunchBounds2, LaunchBounds3, WarpArray, WarpElement};
use cudarc::driver::DeviceRepr;

// SAFETY: 全部字段为固定宽度整数，填充均为显式字段。
// 布局由本模块测试与L2定义对齐；无隐式填充字节。
unsafe impl<T: WarpElement> DeviceRepr for WarpArray<T> {}
// SAFETY: 同上；奇数维度含显式u32填充。
unsafe impl DeviceRepr for LaunchBounds1 {}
// SAFETY: 同上。
unsafe impl DeviceRepr for LaunchBounds2 {}
// SAFETY: 同上。
unsafe impl DeviceRepr for LaunchBounds3 {}
