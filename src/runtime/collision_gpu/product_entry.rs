//! S3：产品碰撞输入接线入口（S1 快照 + S2 adapter → 既有设备路径）。
//!
//! 语义（写死）：
//! - **默认数值路径仍是 Host**；设备路径必须由调用方显式传 [`BackendChoice::Device`]
//!   （沿用 UNC-2 限定语：入口不自动切换后端；设备不可用按既有约定回退 Host）。
//! - **三态回退，不 panic**：
//!   ① 未编译 `cuda` 特性 → [`ProductHostReason::FeatureDisabled`]；
//!   ② 设备或设备程序不可用 → [`ProductHostReason::DeviceUnavailable`]；
//!   ③ 探测通过但运行期失败 → [`ProductHostReason::DeviceFailure`]。
//!   三者都返回可诊断的 Host 回退，不做隐式重试；纯 CPU 输入非法（含只支持
//!   `nworld=1` 的拒绝）返回 [`CollisionGpuError::InvalidArgument`]。
//! - **nworld 口径**：本片只接受 `nworld=1`；`nworld>1` 明确拒绝（多 world 状态读取另立）。
//! - 入口只做 CPU 组装（复用 S2 的 [`inputs_from_snapshot`] 与既有校验），不复制、不改造
//!   [`super::CollisionGpuProgram::run`]（其行为与 `CollisionArgs` 504 B ABI 均不变）。

use super::{
    CollisionGpuError, CollisionGpuInputs, CollisionGpuMeta, CollisionGpuOutput,
    CollisionGpuProgram, CollisionInputsSnapshot, CollisionKernel, inputs_from_snapshot,
};
use crate::runtime::backend::{
    BackendChoice, BackendUnavailable, device_program, product_collision_availability,
};

/// 一次产品碰撞请求：S1 快照视图 + 运行时配对参数 + 设备内核种类。
///
/// `snapshot` 的 32 项 slice 顺序/宽度与 [`super::CollisionGpuInputs`] 一致；
/// `meta` 是调用方给定的配对参数包（本片口径：`nworld` 必须为 1）。
#[derive(Clone, Copy, Debug)]
pub struct ProductCollisionRequest<'a> {
    pub snapshot: CollisionInputsSnapshot<'a>,
    pub meta: CollisionGpuMeta,
    pub kernel: CollisionKernel,
}

/// 入口按 Host 执行的原因（默认数值路径不变；入口本身不跑 host 窄相）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductHostReason {
    /// 调用方显式选择 Host（默认；不尝试设备）。
    Selected,
    /// 构建未启用 `cuda` 特性（编译期回退；不触碰设备与 NVRTC）。
    FeatureDisabled,
    /// 设备或设备程序不可用（驱动/设备/NVRTC/编译的可诊断原因）。
    DeviceUnavailable(BackendUnavailable),
    /// 探测通过但设备执行失败（运行期错误文本；不 panic、不设不可诊断的兜底）。
    DeviceFailure(String),
}

/// 产品入口结果：设备回读接触数组，或带诊断原因的 Host 回退判定。
///
/// `Device` 变体持有一次完整回读结果；单次调用只产生一个结果，`Box` 避免枚举按最大变体
/// 放大栈占用，同时保留输出所有权口径。
#[derive(Clone, Debug)]
pub enum ProductCollisionOutcome {
    /// 设备路径成功（输出与 [`super::CollisionGpuProgram::run`] 同口径）。
    Device(Box<CollisionGpuOutput>),
    /// Host 回退；调用方按既有 host 数值路径执行并读取原因。
    Host(ProductHostReason),
}

impl ProductCollisionOutcome {
    /// 设备输出（Host 回退时为 `None`）。
    pub fn device(&self) -> Option<&CollisionGpuOutput> {
        match self {
            Self::Device(output) => Some(output.as_ref()),
            Self::Host(_) => None,
        }
    }

    /// Host 回退原因（设备路径成功时为 `None`）。
    pub fn host_reason(&self) -> Option<&ProductHostReason> {
        match self {
            Self::Device(_) => None,
            Self::Host(reason) => Some(reason),
        }
    }
}

/// 设备执行器抽象：生产实现是 [`CollisionGpuProgram`]，测试可注入成功/失败。
///
/// 只把「发射并回读」这一步抽象出来，入口的 CPU 校验与回退判定走真实实现。
pub(crate) trait CollisionDeviceRunner {
    fn run_device(
        &self,
        kernel: CollisionKernel,
        inputs: &CollisionGpuInputs<'_>,
    ) -> Result<CollisionGpuOutput, CollisionGpuError>;
}

impl CollisionDeviceRunner for CollisionGpuProgram {
    fn run_device(
        &self,
        kernel: CollisionKernel,
        inputs: &CollisionGpuInputs<'_>,
    ) -> Result<CollisionGpuOutput, CollisionGpuError> {
        self.run(kernel, inputs)
    }
}

/// 产品碰撞入口：显式后端选择 + 三态回退（不 panic）。
///
/// 步骤固定：`nworld=1` 口径检查 + S2 adapter 组装/校验（纯 CPU，先于任何设备探测）→
/// 选择 Host 直接返回 [`ProductHostReason::Selected`]、不触碰驱动/NVRTC/设备 → 选择 Device
/// 时先查 [`product_collision_availability`]，通过后取设备程序并发射；任一步失败都返回
/// 可诊断的 Host 回退。
pub fn run_product_collision(
    choice: BackendChoice,
    request: ProductCollisionRequest<'_>,
) -> Result<ProductCollisionOutcome, CollisionGpuError> {
    let inputs = prepare_inputs(&request)?;
    if choice == BackendChoice::Host {
        return Ok(ProductCollisionOutcome::Host(ProductHostReason::Selected));
    }
    let availability = product_collision_availability();
    let program = if availability.is_ok() {
        device_program()
    } else {
        None
    };
    let device = program
        .as_deref()
        .map(|program| program as &dyn CollisionDeviceRunner);
    run_device_or_fallback(request.kernel, &inputs, availability, device)
}

/// 设备路径编排的可注入实现（后端可用性与设备执行器由调用方给出）。
///
/// 生产路径传入 [`product_collision_availability`] 与缓存的设备程序；单测注入三态可用性
/// 与模拟执行器，覆盖 ①未编译 `cuda` ②设备/程序不可用 ③运行期失败，不需要真实设备。
pub(crate) fn run_device_or_fallback(
    kernel: CollisionKernel,
    inputs: &CollisionGpuInputs<'_>,
    availability: Result<(), BackendUnavailable>,
    device: Option<&dyn CollisionDeviceRunner>,
) -> Result<ProductCollisionOutcome, CollisionGpuError> {
    if let Err(reason) = availability {
        return Ok(ProductCollisionOutcome::Host(host_fallback(reason)));
    }
    let Some(device) = device else {
        return Ok(ProductCollisionOutcome::Host(
            ProductHostReason::DeviceFailure("设备程序不可用（探测通过但未取得程序）".to_owned()),
        ));
    };
    match device.run_device(kernel, inputs) {
        Ok(output) => Ok(ProductCollisionOutcome::Device(Box::new(output))),
        Err(error) => Ok(ProductCollisionOutcome::Host(
            ProductHostReason::DeviceFailure(error.to_string()),
        )),
    }
}

/// CPU 组装：先做本片 `nworld=1` 口径拒绝，再复用 S2 adapter（含既有校验）。
pub(crate) fn prepare_inputs<'a>(
    request: &ProductCollisionRequest<'a>,
) -> Result<CollisionGpuInputs<'a>, CollisionGpuError> {
    if request.meta.nworld != 1 {
        return Err(CollisionGpuError::InvalidArgument(
            "产品碰撞入口只接受 nworld=1；多 world 状态读取另立（本片不支持 nworld>1）",
        ));
    }
    inputs_from_snapshot(&request.snapshot, request.meta)
}

/// 后端不可用原因 → Host 回退原因（①未编译 `cuda` 与 ②设备不可用分开保留）。
fn host_fallback(reason: BackendUnavailable) -> ProductHostReason {
    if reason == BackendUnavailable::FeatureDisabled {
        ProductHostReason::FeatureDisabled
    } else {
        ProductHostReason::DeviceUnavailable(reason)
    }
}

#[cfg(test)]
#[path = "product_entry_tests.rs"]
mod product_entry_tests;
