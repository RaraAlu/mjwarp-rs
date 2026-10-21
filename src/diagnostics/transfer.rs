//! 同步传输辅助错误。

use super::{InputError, ProbeError};
use std::fmt;

/// 同步传输辅助的错误。
/// 不替代正式引擎诊断。
#[derive(Clone, Debug, PartialEq)]
pub enum TransferError {
    Input(InputError),
    Backend(ProbeError),
    HostAllocation {
        bytes: usize,
    },
    SessionMismatch,
    /// 状态不属于当前常驻计划。
    ModelMismatch,
    /// 严格辅助尚未生成对应结果。
    StageNotReady {
        stage: &'static str,
    },
    OverlappingCopy,
    Quarantined,
    /// GPU分解遇到非正或非有限主元。
    InvalidPivot {
        world: usize,
        dof: usize,
        value: f64,
    },
}

impl From<InputError> for TransferError {
    fn from(error: InputError) -> Self {
        Self::Input(error)
    }
}

impl From<ProbeError> for TransferError {
    fn from(error: ProbeError) -> Self {
        Self::Backend(error)
    }
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(error) => write!(f, "传输输入错误：{error}"),
            Self::Backend(error) => write!(f, "传输后端错误：{error}"),
            Self::HostAllocation { bytes } => write!(f, "宿主分配失败：{bytes}字节"),
            Self::SessionMismatch => write!(f, "传输会话不匹配"),
            Self::ModelMismatch => write!(f, "状态不属于当前设备模型"),
            Self::StageNotReady { stage } => write!(f, "辅助阶段尚未更新：{stage}"),
            Self::OverlappingCopy => write!(f, "设备自复制拒绝重叠范围"),
            Self::Quarantined => write!(f, "传输会话已隔离"),
            Self::InvalidPivot { world, dof, value } => {
                write!(f, "分解主元无效：世界{world}；自由度{dof}；值{value}")
            }
        }
    }
}

impl std::error::Error for TransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(error) => Some(error),
            Self::Backend(error) => Some(error),
            _ => None,
        }
    }
}
