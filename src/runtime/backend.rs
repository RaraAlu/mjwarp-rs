//! 窄相后端选择层（运行时探测 + 结果缓存 + Host 回退语义）。
//!
//! 默认 **Host**：本模块不改变既有 host 数值链，也不改变任何 device kernel；只为调用方提供
//! 「设备是否可用 / 是否被允许」的可诊断结论，以及一次性的设备入口缓存。
//!
//! 探测顺序（有 `cuda` 特性时）：驱动库 → 设备数 → NVRTC 库 → PTX 编译；全部通过才
//! [`BackendStatus::CudaReady`]，任一失败记录原因并回退 [`BackendStatus::HostOnly`]（不 panic）。
//! 无 `cuda` 特性构建恒为 `HostOnly(FeatureDisabled)`（编译期无 cudarc 依赖）。
//!
//! 精度语义：本模块不参与数值计算、不新增判据/容差；host 与 device 的数值口径分别沿用既有
//! 已 verified 证据（host CPU recipe / device pinned PTX）。

use std::sync::OnceLock;

/// 调用方显式选择的后端。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendChoice {
    /// 默认：宿主窄相（既有数值链，逐值不变）。
    Host,
    /// 设备窄相；仅当 [`BackendStatus::CudaReady`] 时才被允许，否则回退 Host。
    Device,
}

/// 设备后端不可用的可诊断原因（不 panic、不静默失败）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendUnavailable {
    /// 构建未启用 `cuda` 特性（`--no-default-features`）。
    FeatureDisabled,
    /// NVIDIA 驱动库不可加载。
    DriverUnavailable,
    /// 有驱动但无可用设备，或设备上下文构造失败（含 cudarc 初始化 panic 已捕获）。
    NoDevice(String),
    /// NVRTC 编译库不可加载。
    NvrtcUnavailable,
    /// PTX 编译失败。
    Compile(String),
}

/// 窄相后端可用性结论（进程内缓存）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendStatus {
    /// 只有 host 可用；设备后端不可用原因见内层。
    HostOnly(BackendUnavailable),
    /// 设备后端探测通过（编译/上下文可用）；是否实际走设备仍由调用方显式选择。
    CudaReady,
}

impl BackendStatus {
    /// 设备后端是否可用（仅表示探测通过，不代表默认切换）。
    pub fn cuda_ready(&self) -> bool {
        matches!(self, Self::CudaReady)
    }
}

/// 进程级缓存：首次探测后固定（不每 pair 重探）。
static STATUS: OnceLock<BackendStatus> = OnceLock::new();

/// 返回缓存的窄相后端状态（进程内只探测一次）。
pub fn narrowphase_backend_status() -> &'static BackendStatus {
    STATUS.get_or_init(probe_backend)
}

/// 执行一次不缓存的探测（供测试与诊断使用；产品调用方用 [`narrowphase_backend_status`]）。
pub fn probe_backend() -> BackendStatus {
    #[cfg(not(feature = "cuda-probe"))]
    {
        BackendStatus::HostOnly(BackendUnavailable::FeatureDisabled)
    }
    #[cfg(feature = "cuda-probe")]
    {
        probe_backend_with_cuda()
    }
}

/// 调用方显式选择 Device 且设备确实可用时才允许设备路径；否则回退 Host。
pub fn device_permitted(choice: BackendChoice) -> bool {
    choice == BackendChoice::Device && narrowphase_backend_status().cuda_ready()
}

/// S3：产品碰撞入口（显式设备选择 + 三态回退）的后端可用性判定。
///
/// `cuda` 特性未编译 → [`BackendUnavailable::FeatureDisabled`]（编译期回退，不触碰驱动/
/// NVRTC/设备）；否则沿用 [`narrowphase_backend_status`] 的进程级缓存：探测通过 → `Ok(())`，
/// 否则返回可诊断原因。本函数只读缓存结论，不改变默认 Host 数值路径，也不改变任何设备内核。
pub fn product_collision_availability() -> Result<(), BackendUnavailable> {
    #[cfg(not(feature = "cuda"))]
    {
        Err(BackendUnavailable::FeatureDisabled)
    }
    #[cfg(feature = "cuda")]
    {
        match narrowphase_backend_status() {
            BackendStatus::CudaReady => Ok(()),
            BackendStatus::HostOnly(reason) => Err(reason.clone()),
        }
    }
}

#[cfg(feature = "cuda-probe")]
fn probe_backend_with_cuda() -> BackendStatus {
    use crate::runtime::collision_gpu::CollisionGpuError;
    use crate::runtime::presence;

    if !presence::driver_library_present() {
        return BackendStatus::HostOnly(BackendUnavailable::DriverUnavailable);
    }
    match presence::device_count_now() {
        Ok(0) => {
            return BackendStatus::HostOnly(BackendUnavailable::NoDevice(
                "无可用 CUDA 设备".to_owned(),
            ));
        }
        Ok(_) => {}
        Err(_) => {
            return BackendStatus::HostOnly(BackendUnavailable::NoDevice(
                "设备数查询失败".to_owned(),
            ));
        }
    }
    if !presence::nvrtc_library_present() {
        return BackendStatus::HostOnly(BackendUnavailable::NvrtcUnavailable);
    }
    #[cfg(test)]
    if let Some(injected) = test_compile_override() {
        return injected;
    }
    match crate::runtime::collision_gpu::compile_program() {
        Ok(_) => BackendStatus::CudaReady,
        Err(CollisionGpuError::Compile(detail)) => {
            BackendStatus::HostOnly(BackendUnavailable::Compile(detail))
        }
        Err(other) => BackendStatus::HostOnly(BackendUnavailable::Compile(other.to_string())),
    }
}

/// 设备入口（lazy，进程内只建一次并缓存；仅在 [`BackendStatus::CudaReady`] 时尝试）。
///
/// 调用方必须先通过 [`device_permitted`] 显式允许；本函数不改变 host 数值链。
#[cfg(feature = "cuda-probe")]
pub fn device_program() -> Option<std::sync::Arc<crate::runtime::collision_gpu::CollisionGpuProgram>>
{
    static PROGRAM: OnceLock<
        Option<std::sync::Arc<crate::runtime::collision_gpu::CollisionGpuProgram>>,
    > = OnceLock::new();
    PROGRAM
        .get_or_init(|| {
            if narrowphase_backend_status().cuda_ready() {
                crate::runtime::collision_gpu::CollisionGpuProgram::new(0)
                    .ok()
                    .map(std::sync::Arc::new)
            } else {
                None
            }
        })
        .clone()
}

/// 测试 seam：注入 PTX 编译步骤结果（`None` = 走真实编译）。
#[cfg(all(test, feature = "cuda-probe"))]
fn test_compile_override() -> Option<BackendStatus> {
    match COMPILE_OVERRIDE.load(std::sync::atomic::Ordering::SeqCst) {
        COMPILE_READY => Some(BackendStatus::CudaReady),
        COMPILE_FAIL => Some(BackendStatus::HostOnly(BackendUnavailable::Compile(
            "注入的 PTX 编译失败".to_owned(),
        ))),
        _ => None,
    }
}

#[cfg(all(test, feature = "cuda-probe"))]
pub(crate) const COMPILE_READY: u8 = 1;
#[cfg(all(test, feature = "cuda-probe"))]
pub(crate) const COMPILE_FAIL: u8 = 2;
#[cfg(all(test, feature = "cuda-probe"))]
static COMPILE_OVERRIDE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// 测试 seam：注入编译步骤结果（0 = 真实编译）。
#[cfg(all(test, feature = "cuda-probe"))]
pub(crate) fn set_compile_override_for_tests(mode: u8) {
    COMPILE_OVERRIDE.store(mode, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
mod tests;
