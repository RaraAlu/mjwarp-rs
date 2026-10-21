//! W01、W04与W22：参数布局、槽位描述与槽位用途。
//! Warp结构布局以L2已核验的定义为准。
//! 本模块不打包实参；实参打包属于M3。

use super::LauncherError;
use std::marker::PhantomData;

/// 设备缓冲的元素类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ElemType {
    F32,
    F64,
    I32,
    U32,
    U8,
}

impl ElemType {
    pub const fn bytes(self) -> u64 {
        match self {
            Self::F32 | Self::I32 | Self::U32 => 4,
            Self::F64 => 8,
            Self::U8 => 1,
        }
    }
}

/// 内核对槽位的访问方式。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Access {
    Read,
    Write,
    ReadWrite,
}

impl Access {
    pub const fn writes(self) -> bool {
        matches!(self, Self::Write | Self::ReadWrite)
    }
}

/// 地址类槽位所需证明的名称。
/// 证明类型本身属于W18与W23。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProofKind(pub &'static str);

/// 槽位按内容用途分类。
/// 无法确定时必须声明为地址类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlotRole {
    /// 内容只参与算术。
    Value,
    /// 内容参与寻址、分支或循环界。
    AddressBearing { proof: ProofKind },
}

/// 槽位的参数形态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlotKind {
    /// 私有指针ABI的裸设备指针。
    DevicePtr { elem: ElemType },
    /// Warp ABI的`array_t`数据指针。
    WarpArrayData { elem: ElemType, ndim: u8 },
    /// 按值传递的标量。
    Scalar { bytes: u8 },
    /// Warp的`launch_bounds_t<dims>`，`dims`为发射网格维度。
    /// 冻结上游只用1至3维；4维不受支持。
    Bounds { dims: u8 },
}

impl SlotKind {
    /// 是否为可绑定的设备缓冲槽位。
    pub const fn is_buffer(self) -> bool {
        matches!(self, Self::DevicePtr { .. } | Self::WarpArrayData { .. })
    }

    pub const fn elem(self) -> Option<ElemType> {
        match self {
            Self::DevicePtr { elem } | Self::WarpArrayData { elem, .. } => Some(elem),
            Self::Scalar { .. } | Self::Bounds { .. } => None,
        }
    }
}

/// 单个参数槽位。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotDesc {
    pub index: usize,
    pub kind: SlotKind,
    pub access: Access,
    pub role: SlotRole,
}

/// 线程块规则。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockRule {
    /// 固定线程数，属于ABI的一部分。
    Fixed(u32),
    /// Warp默认256线程网格。
    WarpGrid,
}

impl BlockRule {
    pub const fn threads(self) -> u32 {
        match self {
            Self::Fixed(threads) => threads,
            Self::WarpGrid => 256,
        }
    }
}

/// 显式允许重叠的两个缓冲槽位。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AliasException {
    pub first: usize,
    pub second: usize,
}

/// 内核参数ABI的静态声明。
///
/// # Safety
/// 实现方保证`SLOTS`逐项匹配内核实际签名，
/// 并保证`BLOCK`与内核内的屏障和归约要求一致。
/// Rust无法检查CUDA源码，签名校验只覆盖声明自身。
pub unsafe trait KernelAbi {
    const NAME: &'static str;
    const BLOCK: BlockRule;
    const SLOTS: &'static [SlotDesc];
    const ALIAS_EXCEPTIONS: &'static [AliasException] = &[];
}

/// 已校验的参数签名。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbiSignature {
    name: &'static str,
    block: BlockRule,
    slots: &'static [SlotDesc],
    exceptions: &'static [AliasException],
}

impl AbiSignature {
    /// 校验一个`KernelAbi`实现的声明。
    pub fn of<A: KernelAbi>() -> Result<Self, LauncherError> {
        Self::check(A::NAME, A::BLOCK, A::SLOTS, A::ALIAS_EXCEPTIONS)
    }

    /// 校验声明自身是否成立。
    pub fn check(
        name: &'static str,
        block: BlockRule,
        slots: &'static [SlotDesc],
        exceptions: &'static [AliasException],
    ) -> Result<Self, LauncherError> {
        let fail = |reason| Err(LauncherError::InvalidSignature { abi: name, reason });
        if let BlockRule::Fixed(threads) = block
            && (threads == 0 || threads > 1024 || !threads.is_multiple_of(32))
        {
            return fail("固定线程数须为32的倍数且不超过1024");
        }
        if slots.is_empty() {
            return fail("签名没有槽位");
        }
        let mut warp = false;
        let mut pointer = false;
        for (position, slot) in slots.iter().enumerate() {
            if slot.index != position {
                return fail("槽位编号须从零连续递增");
            }
            match slot.kind {
                SlotKind::Bounds { dims } => {
                    if position != 0 {
                        return fail("Warp边界只能位于首槽位");
                    }
                    if !(1..=3).contains(&dims) {
                        return fail("Warp发射边界维度须为1至3");
                    }
                }
                SlotKind::WarpArrayData { ndim, .. } => {
                    warp = true;
                    if !(1..=4).contains(&ndim) {
                        return fail("Warp数组维度须为1至4");
                    }
                }
                SlotKind::DevicePtr { .. } => pointer = true,
                SlotKind::Scalar { bytes } => {
                    if !matches!(bytes, 1 | 2 | 4 | 8) {
                        return fail("标量宽度须为1、2、4或8字节");
                    }
                }
            }
            if !slot.kind.is_buffer() && slot.access != Access::Read {
                return fail("标量与边界只能读取");
            }
            if matches!(slot.role, SlotRole::AddressBearing { .. }) && slot.access != Access::Read {
                return fail("地址类槽位须只读");
            }
        }
        let bounds = matches!(slots[0].kind, SlotKind::Bounds { .. });
        if warp && pointer {
            return fail("Warp数组不得与裸指针混用");
        }
        if warp != bounds {
            return fail("Warp ABI须以边界开头，私有ABI不得含边界");
        }
        for (i, exception) in exceptions.iter().enumerate() {
            let (a, b) = (exception.first, exception.second);
            if a >= b || b >= slots.len() {
                return fail("别名例外须按升序引用不同槽位");
            }
            for index in [a, b] {
                if !slots[index].kind.is_buffer() {
                    return fail("别名例外只能引用缓冲槽位");
                }
                if matches!(slots[index].role, SlotRole::AddressBearing { .. }) {
                    return fail("别名例外不得引用地址类槽位");
                }
            }
            if exceptions[..i].contains(exception) {
                return fail("别名例外重复");
            }
        }
        Ok(Self {
            name,
            block,
            slots,
            exceptions,
        })
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn block(&self) -> BlockRule {
        self.block
    }

    pub fn slots(&self) -> &'static [SlotDesc] {
        self.slots
    }

    /// 两槽位是否被显式允许重叠。
    pub fn allows_overlap(&self, first: usize, second: usize) -> bool {
        let (a, b) = if first < second {
            (first, second)
        } else {
            (second, first)
        };
        self.exceptions
            .iter()
            .any(|exception| exception.first == a && exception.second == b)
    }
}

/// 可作为Warp数组元素的类型。
pub trait WarpElement: Copy {
    const ELEM: ElemType;
}

impl WarpElement for f32 {
    const ELEM: ElemType = ElemType::F32;
}
impl WarpElement for f64 {
    const ELEM: ElemType = ElemType::F64;
}
impl WarpElement for i32 {
    const ELEM: ElemType = ElemType::I32;
}
impl WarpElement for u32 {
    const ELEM: ElemType = ElemType::U32;
}
impl WarpElement for u8 {
    const ELEM: ElemType = ElemType::U8;
}

/// Warp 1.15的`array_t<T>`按值布局，共56字节。
/// 与L2两份`WarpArray`定义逐字段一致。
mod array;
mod bounds;
#[cfg(feature = "cuda-probe")]
#[path = "abi/device.rs"]
mod device;
#[cfg(test)]
#[path = "abi/tests.rs"]
mod tests;

pub use array::WarpArray;
pub use bounds::{LaunchBounds1, LaunchBounds2, LaunchBounds3};
