//! K.2 / K.3 退路：固定次数循环与无条件分支。
//!
//! - K.2：`device_loop=false` 时设备端条件循环退化为**固定次数**（宿主按固定上限次数发射；
//!   设备侧不看条件），降级留痕；
//! - K.3：`device_branch=false` 时设备端条件分支退化为**无条件执行**
//!   （冻结：两分支都执行，顺序 then → else），降级留痕。

use crate::runtime::kernel_exec::backend::FallbackSet;
use crate::runtime::kernel_exec::error::DeviceError;
use crate::runtime::kernel_exec::queue::{Launch, Queue};

use super::{FallbackCapability, FallbackTrace, select_native};

/// K.2 退路的默认固定次数（P1-c 冻结；调用方可显式覆盖）。
pub const DEFAULT_FIXED_ITERATIONS: u32 = 1;

/// K.2 选择：`true` → 设备端条件执行（由调用方/图能力承担，不在本函数）；
/// `false` → 固定次数发射。
pub fn loop_while_device_fallback<F>(
    fallbacks: &FallbackSet,
    queue: &dyn Queue,
    make_launch: F,
    fixed_iterations: u32,
    trace: &mut FallbackTrace,
) -> Result<u32, DeviceError>
where
    F: Fn() -> Launch,
{
    if select_native(
        FallbackCapability::DeviceLoop,
        fallbacks,
        "fixed-iterations",
        trace,
    ) {
        // 原生路径由后端条件图承担；本参考实现返回 0 表示不接管。
        return Ok(0);
    }
    let iterations = fixed_iterations.max(1);
    for _ in 0..iterations {
        queue.submit(make_launch())?;
    }
    Ok(iterations)
}

/// K.3 选择：`true` → 设备端条件分支（后端承担）；`false` → 两分支都执行（then → else）。
pub fn branch_on_device_fallback(
    fallbacks: &FallbackSet,
    queue: &dyn Queue,
    then_launch: Launch,
    else_launch: Launch,
    trace: &mut FallbackTrace,
) -> Result<u32, DeviceError> {
    if select_native(
        FallbackCapability::DeviceBranch,
        fallbacks,
        "unconditional-both-branches",
        trace,
    ) {
        return Ok(0);
    }
    queue.submit(then_launch)?;
    queue.submit(else_launch)?;
    Ok(2)
}
