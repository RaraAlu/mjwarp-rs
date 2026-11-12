//! 错误模型（A0 §2 第 6 组 / A.4）。
//!
//! 硬性三分类：编译失败 / 发射失败 / 设备丢失；三者不得合并、不得吞掉。
//! [`DeviceError::Unsupported`] 是「能力缺失且无声明退路」的显式失败（A0 §2 设计约束 1），
//! 不属于三分类，但同样不得静默降级。
//!
//! 缓冲参数错误（容量/越界/对齐）不属于设备错误，单独用 [`BufferError`] 表达。

use core::fmt;

use super::backend::CapabilityId;

/// 设备错误的硬性三分类（A.4）。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorClass {
    /// 内核源编译/装载失败（F.1/F.4）。
    Compile,
    /// 内核发射失败（G 域）。
    Launch,
    /// 设备丢失或驱动失败，需要上层重建上下文。
    DeviceLost,
}

/// 编译失败（F.1/F.5）。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileError {
    /// 编译器/装载器返回的消息（不含敏感路径的约定由实现层负责）。
    pub message: String,
    /// 可选的后端错误码。
    pub code: Option<i32>,
}

impl CompileError {
    /// 用消息构造。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    /// 附后端错误码。
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            Some(code) => write!(f, "compile error (code {code}): {}", self.message),
            None => write!(f, "compile error: {}", self.message),
        }
    }
}

impl std::error::Error for CompileError {}

/// 发射失败（G 域）。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaunchError {
    /// 参数/描述符不合法（类型、数量、别名约束等）。
    InvalidArgument(String),
    /// 发射配置不合法（维度/块大小/世界数等）。
    InvalidConfig(String),
    /// 同一缓冲同时作输入与输出时别名声明不成立（G.5）。
    AliasViolation(String),
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(m) => write!(f, "invalid launch argument: {m}"),
            Self::InvalidConfig(m) => write!(f, "invalid launch configuration: {m}"),
            Self::AliasViolation(m) => write!(f, "alias violation: {m}"),
        }
    }
}

impl std::error::Error for LaunchError {}

/// 设备丢失（A.4）。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceLost {
    /// 诊断消息。
    pub message: String,
}

impl DeviceLost {
    /// 用消息构造。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for DeviceLost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "device lost: {}", self.message)
    }
}

impl std::error::Error for DeviceLost {}

/// 设备层错误。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeviceError {
    /// 编译失败。
    Compile(CompileError),
    /// 发射失败。
    Launch(LaunchError),
    /// 设备丢失。
    DeviceLost(DeviceLost),
    /// 能力缺失且无退路（不属于三分类，见模块文档）。
    Unsupported(CapabilityId),
}

impl DeviceError {
    /// 返回硬性三分类；`Unsupported` 返回 `None`。
    pub fn class(&self) -> Option<ErrorClass> {
        match self {
            Self::Compile(_) => Some(ErrorClass::Compile),
            Self::Launch(_) => Some(ErrorClass::Launch),
            Self::DeviceLost(_) => Some(ErrorClass::DeviceLost),
            Self::Unsupported(_) => None,
        }
    }

    /// 是否为能力缺失。
    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported(_))
    }
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile(e) => write!(f, "{e}"),
            Self::Launch(e) => write!(f, "{e}"),
            Self::DeviceLost(e) => write!(f, "{e}"),
            Self::Unsupported(c) => write!(f, "unsupported capability: {}", c.0),
        }
    }
}

impl std::error::Error for DeviceError {}

impl From<CompileError> for DeviceError {
    fn from(e: CompileError) -> Self {
        Self::Compile(e)
    }
}

impl From<LaunchError> for DeviceError {
    fn from(e: LaunchError) -> Self {
        Self::Launch(e)
    }
}

impl From<DeviceLost> for DeviceError {
    fn from(e: DeviceLost) -> Self {
        Self::DeviceLost(e)
    }
}

/// 缓冲参数错误（不属于设备错误三分类；由适配层在边界决定是否升级）。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BufferError {
    /// 申请字节数超过后端/设备容量。
    CapacityExceeded {
        /// 申请字节数。
        requested: u64,
        /// 可用上限。
        capacity: u64,
    },
    /// 范围越界。
    OutOfRange {
        /// 偏移。
        offset: u64,
        /// 长度。
        len: u64,
        /// 缓冲总字节数。
        byte_len: u64,
    },
    /// 对齐不满足实现要求。
    Unaligned {
        /// 偏移。
        offset: u64,
        /// 要求对齐字节数。
        alignment: u64,
    },
    /// 能力缺失且无退路（如 B.11 零拷贝借用、不支持的 `DType`）。
    ///
    /// 与 [`DeviceError::Unsupported`] 同语义，只是出口在缓冲面（P1-b 冻结）。
    Unsupported(CapabilityId),
    /// 缓冲属于其他后端/上下文（跨设备/跨上下文范围拷贝）。
    ForeignBuffer {
        /// 目标后端标识（诊断用，固定小写短名）。
        backend: &'static str,
    },
    /// 缓冲操作期间设备/驱动失败（D1 扩展：P1-b 实现时发现缓冲面缺设备错误通道）。
    ///
    /// 三分类不得吞掉：缓冲面出口同样必须保留设备丢失，由上层决定重建上下文。
    DeviceLost(DeviceLost),
}

impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded {
                requested,
                capacity,
            } => write!(
                f,
                "buffer capacity exceeded: requested {requested} > {capacity}"
            ),
            Self::OutOfRange {
                offset,
                len,
                byte_len,
            } => write!(
                f,
                "buffer range out of bounds: offset {offset}, len {len}, byte_len {byte_len}"
            ),
            Self::Unaligned { offset, alignment } => {
                write!(f, "buffer offset {offset} not aligned to {alignment}")
            }
            Self::Unsupported(capability) => {
                write!(f, "unsupported buffer capability: {}", capability.0)
            }
            Self::ForeignBuffer { backend } => {
                write!(f, "foreign buffer (not owned by backend {backend})")
            }
            Self::DeviceLost(error) => write!(f, "buffer operation failed: {error}"),
        }
    }
}

impl std::error::Error for BufferError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_classes_are_not_merged() {
        let compile = DeviceError::Compile(CompileError::new("bad ptx"));
        let launch = DeviceError::Launch(LaunchError::InvalidConfig("blocks=0".into()));
        let lost = DeviceError::DeviceLost(DeviceLost::new("xid 79"));
        assert_eq!(compile.class(), Some(ErrorClass::Compile));
        assert_eq!(launch.class(), Some(ErrorClass::Launch));
        assert_eq!(lost.class(), Some(ErrorClass::DeviceLost));
        assert!(!compile.is_unsupported());
    }

    #[test]
    fn unsupported_is_explicit_and_not_one_of_three() {
        let unsupported = DeviceError::Unsupported(CapabilityId("K.2"));
        assert_eq!(unsupported.class(), None);
        assert!(unsupported.is_unsupported());
        assert_eq!(unsupported.to_string(), "unsupported capability: K.2");
    }

    #[test]
    fn buffer_errors_are_explicit() {
        let cap = BufferError::CapacityExceeded {
            requested: 4096,
            capacity: 1024,
        };
        let range = BufferError::OutOfRange {
            offset: 8,
            len: 16,
            byte_len: 16,
        };
        assert_eq!(
            cap.to_string(),
            "buffer capacity exceeded: requested 4096 > 1024"
        );
        assert_eq!(
            range.to_string(),
            "buffer range out of bounds: offset 8, len 16, byte_len 16"
        );
    }

    #[test]
    fn buffer_capability_and_foreign_variants_are_explicit() {
        let unsupported = BufferError::Unsupported(CapabilityId("B.11"));
        assert_eq!(
            unsupported.to_string(),
            "unsupported buffer capability: B.11"
        );
        let foreign = BufferError::ForeignBuffer { backend: "cuda" };
        assert_eq!(
            foreign.to_string(),
            "foreign buffer (not owned by backend cuda)"
        );
        assert_ne!(
            BufferError::Unsupported(CapabilityId("B.11")),
            BufferError::ForeignBuffer { backend: "cuda" }
        );
        let lost = BufferError::DeviceLost(DeviceLost::new("xid 79"));
        assert_eq!(
            lost.to_string(),
            "buffer operation failed: device lost: xid 79"
        );
        assert_ne!(lost, BufferError::ForeignBuffer { backend: "cuda" });
    }
}
