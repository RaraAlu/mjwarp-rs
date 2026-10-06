//! 探针错误独立于物理诊断。

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ProbeError {
    InvalidArgument(&'static str),
    FeatureDisabled(&'static str),
    NvrtcUnavailable,
    NvrtcPresent,
    Artifact {
        stage: &'static str,
        detail: String,
    },
    Compilation {
        stage: &'static str,
        detail: String,
    },
    DriverUnavailable,
    InvalidDevice {
        requested: usize,
        available: i32,
    },
    UnsupportedDevice {
        major: i32,
        minor: i32,
    },
    UnsupportedDriver {
        version: i32,
    },
    Cuda {
        stage: &'static str,
        code: u32,
    },
    BackendPanic,
    EmptyGraph,
    InvalidGraph(&'static str),
    Resource(ResourceError),
    InvalidOutputLength {
        expected: usize,
        actual: usize,
    },
    Mismatch {
        index: usize,
        expected: f32,
        actual: f32,
    },
    IntegerMismatch {
        index: usize,
        expected: u32,
        actual: u32,
    },
    SolveMismatch {
        index: usize,
        expected: f64,
        actual: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceError {
    DeviceMismatch,
    ContextMismatch,
    InvalidRange {
        offset: usize,
        bytes: usize,
        capacity: usize,
    },
    Misaligned,
    StaleLayout {
        expected: u64,
        actual: u64,
    },
    ReadOnly,
    Conflict,
    LayoutBusy,
    VersionExhausted,
    Quarantined,
    Poisoned,
    InvalidCompletion,
}

impl From<ResourceError> for ProbeError {
    fn from(error: ResourceError) -> Self {
        Self::Resource(error)
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceMismatch => write!(f, "设备不匹配"),
            Self::ContextMismatch => write!(f, "上下文不匹配"),
            Self::InvalidRange {
                offset,
                bytes,
                capacity,
            } => write!(f, "范围越界：偏移{offset}，字节{bytes}，容量{capacity}"),
            Self::Misaligned => write!(f, "范围未按元素对齐"),
            Self::StaleLayout { expected, actual } => {
                write!(f, "布局过期：当前{expected}，请求{actual}")
            }
            Self::ReadOnly => write!(f, "只读资源拒绝写入"),
            Self::Conflict => write!(f, "在途租约冲突"),
            Self::LayoutBusy => write!(f, "在途资源拒绝布局变更"),
            Self::VersionExhausted => write!(f, "资源版本或租约编号耗尽"),
            Self::Quarantined => write!(f, "资源已隔离"),
            Self::Poisoned => write!(f, "资源锁异常"),
            Self::InvalidCompletion => write!(f, "完成令牌不属于在途队列"),
        }
    }
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "参数错误：{message}"),
            Self::FeatureDisabled(feature) => write!(f, "请启用{feature}功能"),
            Self::NvrtcUnavailable => write!(f, "找不到NVRTC库；请检查PATH"),
            Self::NvrtcPresent => write!(f, "仅驱动探针拒绝可见NVRTC"),
            Self::Artifact { stage, detail } => write!(f, "产物错误：{stage}；{detail}"),
            Self::Compilation { stage, detail } => write!(f, "编译失败：{stage}\n{detail}"),
            Self::DriverUnavailable => write!(f, "找不到NVIDIA驱动库"),
            Self::InvalidDevice {
                requested,
                available,
            } => write!(f, "设备索引{requested}越界；设备数{available}"),
            Self::UnsupportedDevice { major, minor } => {
                write!(f, "探针需要SM 7.0；当前SM {major}.{minor}")
            }
            Self::UnsupportedDriver { version } => {
                write!(f, "探针需要CUDA 12.0；当前API版本{version}")
            }
            Self::Cuda { stage, code } => write!(f, "CUDA失败：{stage}；错误码{code}"),
            Self::BackendPanic => write!(f, "GPU适配层异常；未回退CPU"),
            Self::EmptyGraph => write!(f, "图捕获没有记录内核"),
            Self::InvalidGraph(reason) => write!(f, "图布局错误：{reason}"),
            Self::Resource(error) => write!(f, "资源错误：{error}"),
            Self::InvalidOutputLength { expected, actual } => {
                write!(f, "输出长度错误：期望{expected}，实际{actual}")
            }
            Self::Mismatch {
                index,
                expected,
                actual,
            } => write!(f, "输出[{index}]错误：期望{expected}，实际{actual}"),
            Self::IntegerMismatch {
                index,
                expected,
                actual,
            } => write!(f, "整数输出[{index}]错误：期望{expected}，实际{actual}"),
            Self::SolveMismatch {
                index,
                expected,
                actual,
            } => write!(
                f,
                "求解输出或残差[{index}]错误：期望{expected}，实际{actual}"
            ),
        }
    }
}

impl std::error::Error for ProbeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_cuda_stage_and_code() {
        let error = ProbeError::Cuda {
            stage: "load",
            code: 200,
        };
        assert_eq!(error.to_string(), "CUDA失败：load；错误码200");
    }

    #[test]
    fn preserves_large_integer_mismatches_without_rounding() {
        let error = ProbeError::IntegerMismatch {
            index: 0,
            expected: 16_777_217,
            actual: 16_777_216,
        };
        assert_eq!(
            error.to_string(),
            "整数输出[0]错误：期望16777217，实际16777216"
        );
    }
}
