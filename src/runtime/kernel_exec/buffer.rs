//! 缓冲与分配（A0 §2 第 2 组 / B 域）。
//!
//! 句柄面保持对象安全：`Buffer` 以 `&dyn Buffer` 参与拷贝，分配器返回 `Box<dyn Buffer>`。
//! 缓冲参数错误（容量/越界/对齐）用 [`BufferError`] 显式表达；本层不做隐式同步（A0 §2）。

use super::error::BufferError;

/// 设备缓冲地址（B.3/B.9：64 位字节地址；生命期内稳定）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Address64(pub u64);

/// 元素类型（B.1；宿主 ABI 的 DType 子集，按需要扩展）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DType {
    /// 8 位无符号。
    U8,
    /// 32 位有符号整数。
    I32,
    /// 32 位无符号整数。
    U32,
    /// 64 位有符号整数。
    I64,
    /// 64 位无符号整数。
    U64,
    /// 32 位浮点（产品主类型）。
    F32,
    /// 布尔（按 1 字节存储）。
    Bool,
    /// vec2<f32>。
    Vec2,
    /// vec3<f32>。
    Vec3,
    /// vec4<f32>。
    Vec4,
    /// vec6<f32>。
    Vec6,
    /// vec10<f32>。
    Vec10,
    /// mat33<f32>。
    Mat33,
    /// quat<f32>。
    Quat,
}

impl DType {
    /// 单元素字节数。
    pub const fn byte_size(self) -> u64 {
        match self {
            Self::U8 | Self::Bool => 1,
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::I64 | Self::U64 | Self::Vec2 => 8,
            Self::Vec3 => 12,
            Self::Vec4 | Self::Quat => 16,
            Self::Vec6 => 24,
            Self::Mat33 => 36,
            Self::Vec10 => 40,
        }
    }
}

/// 分配初值（B.1）：未初始化 / 清零 / 填值。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Init {
    /// 未初始化（B.1 的「未初始化分配」退项）。
    Uninit,
    /// 清零。
    Zeroed,
    /// 用 u64 位模式填充（按元素宽度截断；f32 用其位模式）。
    Fill(u64),
}

/// 固定（pin）后的设备缓冲视图（B.9）。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pinned {
    /// 稳定地址。
    pub address: Address64,
}

/// 外部框架句柄（B.11 零拷贝借用的 opaque 句柄）。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ExternalHandle(pub u64);

/// 设备缓冲。
pub trait Buffer {
    /// 稳定地址（B.3/B.9）。
    fn addr(&self) -> Address64;
    /// 总字节数。
    fn byte_len(&self) -> u64;
    /// 设备内范围拷贝（B.5）：`src_off..src_off+len` → `dst_off..`。
    fn copy_range(
        &self,
        dst: &dyn Buffer,
        dst_off: u64,
        src_off: u64,
        len: u64,
    ) -> Result<(), BufferError>;
    /// 宿主上传（B.6）：从 `host[0..]` 写到 `off`。
    fn upload(&self, host: &[u8], off: u64) -> Result<(), BufferError>;
    /// 范围回读（B.6）：从 `off` 读满 `host` 长度。
    fn readback(&self, host: &mut [u8], off: u64) -> Result<(), BufferError>;
    /// 固定地址（B.9）。
    fn pin(&self) -> Result<Pinned, BufferError>;
}

/// 分配器（B.1/B.10/B.11）。
pub trait Allocator {
    /// 按类型与元素个数分配。
    fn alloc(&self, dtype: DType, len: u64, init: Init) -> Result<Box<dyn Buffer>, BufferError>;
    /// 低成本临时缓冲（B.10 退：预分配）。
    fn transient(&self, bytes: u64) -> Result<Box<dyn Buffer>, BufferError>;
    /// 借用外部句柄（B.11 退：拷贝）。
    fn borrow_external(&self, handle: ExternalHandle) -> Result<Box<dyn Buffer>, BufferError>;
}

/// 范围检查辅助：`[offset, offset+len)` 必须落在 `byte_len` 内。
pub fn check_range(offset: u64, len: u64, byte_len: u64) -> Result<(), BufferError> {
    match offset.checked_add(len) {
        Some(end) if end <= byte_len => Ok(()),
        _ => Err(BufferError::OutOfRange {
            offset,
            len,
            byte_len,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dtype_sizes_are_explicit() {
        assert_eq!(DType::F32.byte_size(), 4);
        assert_eq!(DType::Vec3.byte_size(), 12);
        assert_eq!(DType::Vec10.byte_size(), 40);
        assert_eq!(DType::Mat33.byte_size(), 36);
    }

    #[test]
    fn range_check_is_explicit_on_overflow() {
        assert!(check_range(0, 16, 16).is_ok());
        assert!(check_range(8, 8, 16).is_ok());
        match check_range(8, 9, 16) {
            Err(BufferError::OutOfRange {
                offset,
                len,
                byte_len,
            }) => {
                assert_eq!((offset, len, byte_len), (8, 9, 16));
            }
            other => panic!("expected OutOfRange, got {other:?}"),
        }
        // u64 溢出也必须是显式越界，而不是 panic 或回绕。
        assert!(matches!(
            check_range(u64::MAX, 1, u64::MAX),
            Err(BufferError::OutOfRange { .. })
        ));
    }
}
