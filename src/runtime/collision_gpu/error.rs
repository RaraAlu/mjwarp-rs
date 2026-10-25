//! G10@P5：产品 NVRTC 入口错误类型。
//!
//! 所有失败路径都返回本枚举（不 panic）：驱动缺失、NVRTC 编译失败、模块/函数加载失败、
//! 运行/回读失败、以及纯 CPU 可判的参数错误。`thiserror` 不在依赖里，故手写
//! `Display`/`Error`。

use std::error::Error;
use std::fmt;

/// 产品 GPU 入口错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollisionGpuError {
    /// `CudaContext::new(device)` 失败（无设备/驱动不可用）。
    NoDevice(String),
    /// NVRTC 编译失败或编译期 panic。
    Compile(String),
    /// PTX 模块加载或入口函数查找失败。
    Load(String),
    /// 上传/分配/发射/同步/回读失败。
    Launch(String),
    /// 纯 CPU 可判的输入参数错误（尺寸、容量、slice 长度）。
    InvalidArgument(&'static str),
}

impl fmt::Display for CollisionGpuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDevice(detail) => write!(formatter, "CUDA设备不可用：{detail}"),
            Self::Compile(detail) => write!(formatter, "NVRTC编译失败：{detail}"),
            Self::Load(detail) => write!(formatter, "设备模块加载失败：{detail}"),
            Self::Launch(detail) => write!(formatter, "内核执行失败：{detail}"),
            Self::InvalidArgument(detail) => write!(formatter, "输入参数无效：{detail}"),
        }
    }
}

impl Error for CollisionGpuError {}
