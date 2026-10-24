//! W01/W04 渲染扩展：元素类、56 字节 Warp 数组描述符与实参值（渲染/BVH S7a）。
//!
//! 只做宿主侧声明与校验，不访问设备、不编译内核。元素类对应冻结上游
//! `_render_megakernel` 的实参面（`wp.array[..]` 与标量），元素宽度与对齐以
//! W15 头树的 device C++ 布局为准（S7a 预检 static_assert 钉死）：
//! `texture2d_t`=24/align8、vec2=8、vec3=12、vec4=16、mat33=36、quat=16、
//! vec2i=8、vec3i=12、bool=1、u64=8、i32/u32/f32=4、`array_t<T>`=56。
//!
//! cudarc 把每个实参按 `*const T` 交给 `cuLaunchKernel`，由驱动按内核声明读取；
//! 因此这里的结构体布局必须与 Warp 头文件的 C++ 布局逐字节一致。

use cudarc::driver::DeviceRepr;

use super::super::{DeviceSpan, LaunchBounds2, LauncherError};

/// 渲染内核数组的元素类（13 类）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderElem {
    I32,
    U32,
    U64,
    Bool,
    F32,
    Vec2,
    Vec3,
    Vec4,
    Vec2I,
    Vec3I,
    Mat33,
    Quat,
    Texture2d,
}

/// 设备侧原始数据族；决定容量换算与打包类型（布局始终按元素宽度）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderFamily {
    F32,
    I32,
    U32,
    U64,
    U8,
}

impl RenderElem {
    /// 上游 `_render_megakernel` 用到的全部数组元素类。
    pub const ALL: [Self; 13] = [
        Self::I32,
        Self::U32,
        Self::U64,
        Self::Bool,
        Self::F32,
        Self::Vec2,
        Self::Vec3,
        Self::Vec4,
        Self::Vec2I,
        Self::Vec3I,
        Self::Mat33,
        Self::Quat,
        Self::Texture2d,
    ];

    /// 单元素字节数（Warp `sizeof(T)`，即数组默认步长单位）。
    pub const fn bytes(self) -> u64 {
        match self {
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::U64 => 8,
            Self::Bool => 1,
            Self::Vec2 | Self::Vec2I => 8,
            Self::Vec3 | Self::Vec3I => 12,
            Self::Vec4 | Self::Quat => 16,
            Self::Mat33 => 36,
            Self::Texture2d => 24,
        }
    }

    /// 单元素对齐（Warp 头文件 C++ `alignof`）；数据指针必须满足。
    pub const fn align(self) -> u64 {
        match self {
            Self::Bool => 1,
            Self::U64 | Self::Texture2d => 8,
            _ => 4,
        }
    }

    /// 设备侧原始数据族。
    pub const fn family(self) -> RenderFamily {
        match self {
            Self::F32 | Self::Vec2 | Self::Vec3 | Self::Vec4 | Self::Mat33 | Self::Quat => {
                RenderFamily::F32
            }
            Self::I32 | Self::Vec2I | Self::Vec3I => RenderFamily::I32,
            Self::U32 => RenderFamily::U32,
            Self::U64 => RenderFamily::U64,
            Self::Bool | Self::Texture2d => RenderFamily::U8,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::Bool => "bool",
            Self::F32 => "f32",
            Self::Vec2 => "vec2f",
            Self::Vec3 => "vec3f",
            Self::Vec4 => "vec4f",
            Self::Vec2I => "vec2i",
            Self::Vec3I => "vec3i",
            Self::Mat33 => "mat33f",
            Self::Quat => "quatf",
            Self::Texture2d => "texture2d_t",
        }
    }
}

impl RenderFamily {
    /// 打包用的 Rust 元素宽度（仅描述原始数据，不改变 Warp 数组步长）。
    pub const fn rust_bytes(self) -> u64 {
        match self {
            Self::F32 | Self::I32 | Self::U32 => 4,
            Self::U64 => 8,
            Self::U8 => 1,
        }
    }
}

/// Warp `array_t<T>` 的按值布局，共 56 字节（与 `WarpArray` 逐字段一致）。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArray {
    data: u64,
    grad: u64,
    shape: [i32; 4],
    strides: [i32; 4],
    ndim: u16,
    flags: u16,
    padding: u32,
}

// SAFETY: 全部字段为固定宽度整数与显式填充，布局与 Warp 1.15 `array_t<T>` 相同。
unsafe impl DeviceRepr for RenderArray {}

impl RenderArray {
    /// 行主序连续数组描述符；`shape` 为 1..=4 维且各维非零。
    pub fn contiguous(data: u64, elem: RenderElem, shape: &[usize]) -> Result<Self, LauncherError> {
        if data == 0 {
            return Err(LauncherError::InvalidArray("渲染数组数据指针为空"));
        }
        if !(1..=4).contains(&shape.len()) || shape.contains(&0) {
            return Err(LauncherError::InvalidArray("渲染数组形状须为1至4维且非零"));
        }
        let mut array = Self {
            data,
            grad: 0,
            shape: [0; 4],
            strides: [0; 4],
            ndim: shape.len() as u16,
            flags: 0,
            padding: 0,
        };
        let mut stride = elem.bytes();
        for axis in (0..shape.len()).rev() {
            array.shape[axis] = i32::try_from(shape[axis])
                .map_err(|_| LauncherError::Overflow("render_array_shape"))?;
            array.strides[axis] = i32::try_from(stride)
                .map_err(|_| LauncherError::Overflow("render_array_stride"))?;
            stride = stride
                .checked_mul(shape[axis] as u64)
                .ok_or(LauncherError::Overflow("render_array_bytes"))?;
        }
        Ok(array)
    }

    /// 数据指针。
    pub fn data(&self) -> u64 {
        self.data
    }

    /// 维度数。
    pub fn ndim(&self) -> usize {
        self.ndim as usize
    }

    /// 形状切片。
    pub fn shape(&self) -> &[i32] {
        &self.shape[..self.ndim as usize]
    }

    /// 字节步长切片。
    pub fn strides(&self) -> &[i32] {
        &self.strides[..self.ndim as usize]
    }

    /// 描述符覆盖的字节数（按元素类宽度）。
    pub fn byte_len(&self, elem: RenderElem) -> Result<u64, LauncherError> {
        let mut total = elem.bytes();
        for &axis in self.shape() {
            total = total
                .checked_mul(
                    u64::try_from(axis).map_err(|_| LauncherError::Overflow("render_shape"))?,
                )
                .ok_or(LauncherError::Overflow("render_array_bytes"))?;
        }
        Ok(total)
    }

    /// 与元素类和连续行主序布局逐项核对。
    pub fn check(
        &self,
        slot: usize,
        elem: RenderElem,
        span: DeviceSpan,
    ) -> Result<(), LauncherError> {
        if self.data == 0 || self.data != span.addr() {
            return Err(LauncherError::InvalidArray(
                "渲染数组数据指针为空或与绑定不符",
            ));
        }
        if !(1..=4).contains(&self.ndim()) || self.shape().iter().any(|&axis| axis <= 0) {
            return Err(LauncherError::RenderArrayLayout { slot });
        }
        if !self.data.is_multiple_of(elem.align()) {
            return Err(LauncherError::RenderArrayAlign { slot });
        }
        let mut stride = elem.bytes();
        for axis in (0..self.ndim()).rev() {
            if self.strides()[axis] as u64 != stride {
                return Err(LauncherError::RenderArrayLayout { slot });
            }
            stride = stride
                .checked_mul(self.shape()[axis] as u64)
                .ok_or(LauncherError::Overflow("render_array_bytes"))?;
        }
        let required = self.byte_len(elem)?;
        if required > span.bytes() {
            return Err(LauncherError::Capacity {
                slot,
                required,
                actual: span.bytes(),
            });
        }
        Ok(())
    }
}

/// 一次数组绑定：描述符 + 元素类 + 设备区间（全部在发射前校验）。
#[derive(Clone, Copy, Debug)]
pub struct RenderArrayValue {
    pub(super) array: RenderArray,
    elem: RenderElem,
    span: DeviceSpan,
}

impl RenderArrayValue {
    /// 由设备区间、元素类与形状构造；形状与区间容量在此校验。
    pub fn contiguous(
        span: DeviceSpan,
        elem: RenderElem,
        shape: &[usize],
    ) -> Result<Self, LauncherError> {
        let array = RenderArray::contiguous(span.addr(), elem, shape)?;
        let value = Self { array, elem, span };
        value.array.check(0, elem, span)?;
        Ok(value)
    }

    pub fn elem(&self) -> RenderElem {
        self.elem
    }

    pub fn span(&self) -> DeviceSpan {
        self.span
    }

    pub fn array(&self) -> &RenderArray {
        &self.array
    }
}

/// 渲染内核的标量类（上游 megakernel 的按值参数只有这三种）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderScalarKind {
    I32,
    U64,
    Bool,
}

impl RenderScalarKind {
    pub const fn bytes(self) -> u64 {
        match self {
            Self::I32 => 4,
            Self::U64 => 8,
            Self::Bool => 1,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::I32 => "int32",
            Self::U64 => "uint64",
            Self::Bool => "bool",
        }
    }
}

/// 按值标量实参。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderScalar {
    I32(i32),
    U64(u64),
    Bool(bool),
}

impl RenderScalar {
    pub const fn kind(self) -> RenderScalarKind {
        match self {
            Self::I32(_) => RenderScalarKind::I32,
            Self::U64(_) => RenderScalarKind::U64,
            Self::Bool(_) => RenderScalarKind::Bool,
        }
    }
}

/// `wp.texture2d_t` 的 24 字节按值布局（与 W15 `texture.h` 一致）。
///
/// 设备数组的元素步长是 24（align 8）；设备端数组由渲染侧上传，本结构只用于
/// 编码/回读，不持有 `CUtexObject` 所有权。
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Texture2d {
    pub tex: u64,
    pub width: i32,
    pub height: i32,
    pub num_channels: i32,
}

impl Texture2d {
    pub const BYTES: usize = 24;

    /// 编码为 24 字节（本机序，与设备端 C++ 布局一致）。
    pub fn encode(self, target: &mut [u8]) {
        assert_eq!(target.len(), Self::BYTES, "texture2d 编码长度");
        target[0..8].copy_from_slice(&self.tex.to_ne_bytes());
        target[8..12].copy_from_slice(&self.width.to_ne_bytes());
        target[12..16].copy_from_slice(&self.height.to_ne_bytes());
        target[16..20].copy_from_slice(&self.num_channels.to_ne_bytes());
        target[20..24].fill(0);
    }

    /// 解码 24 字节。
    pub fn decode(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), Self::BYTES, "texture2d 解码长度");
        Self {
            tex: u64::from_ne_bytes(bytes[0..8].try_into().expect("tex")),
            width: i32::from_ne_bytes(bytes[8..12].try_into().expect("width")),
            height: i32::from_ne_bytes(bytes[12..16].try_into().expect("height")),
            num_channels: i32::from_ne_bytes(bytes[16..20].try_into().expect("channels")),
        }
    }
}

/// 一个渲染实参。
#[derive(Clone, Copy, Debug)]
pub enum RenderArg {
    /// Warp `launch_bounds_t<2>`（必须位于首槽位）。
    Bounds2D(LaunchBounds2),
    Array(RenderArrayValue),
    Scalar(RenderScalar),
}

impl RenderArg {
    /// 该实参的声明种类（用于与签名逐槽比对）。
    pub fn kind(&self) -> RenderArgKind {
        match self {
            Self::Bounds2D(_) => RenderArgKind::Bounds2D,
            Self::Array(value) => RenderArgKind::Array(value.elem()),
            Self::Scalar(scalar) => RenderArgKind::Scalar(scalar.kind()),
        }
    }
}

/// 签名的槽位种类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderArgKind {
    Bounds2D,
    Array(RenderElem),
    Scalar(RenderScalarKind),
}

impl RenderArgKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bounds2D => "launch_bounds_t<2>",
            Self::Array(elem) => elem.name(),
            Self::Scalar(kind) => kind.name(),
        }
    }
}

/// 冻结上游 `_render_megakernel` 的 81 槽位签名（1 bounds + 73 数组 + 7 标量）。
///
/// 顺序取自 S7a 预检从 Warp 1.15 生成的 CUDA 源（`_build_megakernel__locals___render_megakernel_*`）
/// 的真实 `extern "C"` 签名；S7a 只用它验证 ABI 表达能力与拒绝面，S7b 才实现同名内核。
pub static MEGAKERNEL_SLOTS: [RenderArgKind; 81] = {
    use RenderArgKind::{Array as A, Bounds2D as B, Scalar as S};
    use RenderElem::{
        Bool, F32, I32, Mat33, Quat, Texture2d, U32, U64, Vec2, Vec2I, Vec3, Vec3I, Vec4,
    };
    use RenderScalarKind::{Bool as SB, I32 as SI, U64 as SU};
    [
        B,
        A(I32),
        A(I32),
        A(I32),
        A(Vec3),
        A(Vec4),
        A(I32),
        A(F32),
        A(Vec2),
        A(Vec4),
        A(I32),
        A(Bool),
        A(Bool),
        A(Vec3),
        A(F32),
        A(F32),
        A(Vec3),
        A(Vec3),
        A(Vec3),
        A(I32),
        A(Vec2I),
        A(F32),
        A(I32),
        A(I32),
        A(Vec3),
        A(I32),
        A(Bool),
        A(Vec2),
        A(F32),
        A(F32),
        A(F32),
        A(Vec4),
        A(Vec3),
        A(Mat33),
        A(Vec3),
        A(Mat33),
        A(Vec3),
        A(Vec3),
        A(Vec3),
        S(SI),
        S(SB),
        S(SI),
        S(SI),
        A(Vec2I),
        A(I32),
        A(Vec3),
        A(Vec3),
        S(SI),
        A(I32),
        A(I32),
        A(I32),
        A(Bool),
        A(Bool),
        A(Bool),
        S(SU),
        A(I32),
        A(U64),
        A(I32),
        A(I32),
        A(U64),
        A(Vec3I),
        A(Vec3I),
        A(Vec2),
        A(I32),
        A(U64),
        A(Vec4),
        A(I32),
        A(I32),
        A(I32),
        A(I32),
        A(Texture2d),
        A(Vec3),
        A(Quat),
        A(Vec3),
        A(Vec4),
        S(SU),
        A(I32),
        A(U32),
        A(Vec3),
        A(F32),
        A(Vec2I),
    ]
};
