//! 基础输入错误与独立探针错误。

use std::fmt;

/// 基础辅助接口的输入错误。
/// 不替代正式引擎诊断。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputError {
    InvalidDimension {
        field: &'static str,
    },
    Overflow {
        field: &'static str,
    },
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    NonFinite {
        field: &'static str,
        index: usize,
    },
    ScalarOverflow {
        field: &'static str,
        index: usize,
    },
    InvalidIndex {
        field: &'static str,
        index: usize,
        limit: usize,
    },
    InvalidHistoryTime {
        index: usize,
    },
    DisabledOutput,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimension { field } => write!(f, "维度无效：{field}"),
            Self::Overflow { field } => write!(f, "容量溢出：{field}"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(f, "长度错误：{field}；期望{expected}，实际{actual}"),
            Self::NonFinite { field, index } => write!(f, "数值非有限：{field}[{index}]"),
            Self::ScalarOverflow { field, index } => write!(f, "转换溢出：{field}[{index}]"),
            Self::InvalidIndex {
                field,
                index,
                limit,
            } => write!(f, "索引越界：{field}[{index}]；上限{limit}"),
            Self::InvalidHistoryTime { index } => write!(f, "历史时间间隔无效：{index}"),
            Self::DisabledOutput => write!(f, "图像输出未启用"),
        }
    }
}

impl std::error::Error for InputError {}

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
    InvalidExternalDescriptor,
    UnsupportedExternalLayout,
    UnsupportedExternalMemory,
    DuplicateExternalAllocation,
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
            Self::InvalidExternalDescriptor => write!(f, "外部描述符版本或范围无效"),
            Self::UnsupportedExternalLayout => write!(f, "外部原型仅支持连续f32"),
            Self::UnsupportedExternalMemory => write!(f, "外部原型拒绝此内存类型"),
            Self::DuplicateExternalAllocation => write!(f, "外部分配仍持有注册或在途租约"),
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
    fn preserves_input_field_and_index() {
        let error = InputError::InvalidIndex {
            field: "world",
            index: 3,
            limit: 2,
        };
        assert_eq!(error.to_string(), "索引越界：world[3]；上限2");
        let error = InputError::NonFinite {
            field: "state",
            index: 7,
        };
        assert_eq!(error.to_string(), "数值非有限：state[7]");
    }

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
