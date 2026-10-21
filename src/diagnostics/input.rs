//! 原生输入与基础输入错误。

use std::fmt;

/// 原生已编译输入探针的错误。
/// 不承诺恶意MJB的安全隔离。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeProbeError {
    Input(InputError),
    InvalidPath,
    InvalidMjbHeader,
    AbiMismatch,
    VersionMismatch { expected: i32, actual: i32 },
    Native { stage: &'static str, code: u32 },
    HostAllocation { bytes: usize },
}

impl From<InputError> for NativeProbeError {
    fn from(error: InputError) -> Self {
        Self::Input(error)
    }
}

impl fmt::Display for NativeProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(error) => write!(f, "原生输入错误：{error}"),
            Self::InvalidPath => write!(f, "请提供无NUL的绝对DLL路径"),
            Self::InvalidMjbHeader => write!(f, "MJB头部无效"),
            Self::AbiMismatch => write!(f, "原生探针ABI不匹配"),
            Self::VersionMismatch { expected, actual } => {
                write!(f, "原生版本错误：期望{expected}，实际{actual}")
            }
            Self::Native { stage, code } => write!(f, "原生失败：{stage}；错误码{code}"),
            Self::HostAllocation { bytes } => write!(f, "宿主分配失败：{bytes}字节"),
        }
    }
}

impl std::error::Error for NativeProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(error) => Some(error),
            _ => None,
        }
    }
}

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
    NegativeValue {
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
    InvalidRange {
        field: &'static str,
        offset: usize,
        elements: usize,
        capacity: usize,
    },
    InvalidHistoryTime {
        index: usize,
    },
    DisabledOutput,
    InvalidTopology {
        field: &'static str,
        index: usize,
        reason: &'static str,
    },
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
            Self::NegativeValue { field, index } => write!(f, "数值为负：{field}[{index}]"),
            Self::ScalarOverflow { field, index } => write!(f, "转换溢出：{field}[{index}]"),
            Self::InvalidIndex {
                field,
                index,
                limit,
            } => write!(f, "索引越界：{field}[{index}]；上限{limit}"),
            Self::InvalidRange {
                field,
                offset,
                elements,
                capacity,
            } => write!(
                f,
                "范围越界：{field}；偏移{offset}，元素{elements}，容量{capacity}"
            ),
            Self::InvalidHistoryTime { index } => write!(f, "历史时间间隔无效：{index}"),
            Self::DisabledOutput => write!(f, "图像输出未启用"),
            Self::InvalidTopology {
                field,
                index,
                reason,
            } => write!(f, "拓扑无效：{field}[{index}]；{reason}"),
        }
    }
}

impl std::error::Error for InputError {}
