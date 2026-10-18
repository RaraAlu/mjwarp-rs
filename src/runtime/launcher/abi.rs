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
#[repr(C)]
pub struct WarpArray<T> {
    data: u64,
    grad: u64,
    shape: [i32; 4],
    strides: [i32; 4],
    ndim: u16,
    flags: u16,
    padding: u32,
    marker: PhantomData<T>,
}

impl<T> Clone for WarpArray<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WarpArray<T> {}

impl<T> WarpArray<T> {
    pub fn data(&self) -> u64 {
        self.data
    }

    pub fn ndim(&self) -> usize {
        self.ndim as usize
    }

    pub fn shape(&self) -> &[i32] {
        &self.shape[..self.ndim as usize]
    }

    pub fn strides(&self) -> &[i32] {
        &self.strides[..self.ndim as usize]
    }
}

impl<T> std::fmt::Debug for WarpArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WarpArray")
            .field("data", &self.data)
            .field("shape", &self.shape())
            .field("strides", &self.strides())
            .finish()
    }
}

impl<T: WarpElement> WarpArray<T> {
    /// 行主序连续数组；无梯度缓冲。
    /// 形状与步长须落入i32，与Warp头文件一致。
    pub fn contiguous(data: u64, shape: &[usize]) -> Result<Self, LauncherError> {
        if data == 0 {
            return Err(LauncherError::InvalidArray("数据指针为空"));
        }
        if !(1..=4).contains(&shape.len()) || shape.contains(&0) {
            return Err(LauncherError::InvalidArray("形状须为1至4维且各维非零"));
        }
        let mut array = Self {
            data,
            grad: 0,
            shape: [0; 4],
            strides: [0; 4],
            ndim: shape.len() as u16,
            flags: 0,
            padding: 0,
            marker: PhantomData,
        };
        let mut stride = T::ELEM.bytes();
        for axis in (0..shape.len()).rev() {
            array.shape[axis] = i32::try_from(shape[axis])
                .map_err(|_| LauncherError::Overflow("warp_array_shape"))?;
            array.strides[axis] =
                i32::try_from(stride).map_err(|_| LauncherError::Overflow("warp_array_stride"))?;
            stride = stride
                .checked_mul(shape[axis] as u64)
                .ok_or(LauncherError::Overflow("warp_array_bytes"))?;
        }
        Ok(array)
    }

    /// 连续数组覆盖的字节数。
    pub fn byte_len(&self) -> u64 {
        self.shape()
            .iter()
            .fold(T::ELEM.bytes(), |total, &axis| total * axis as u64)
    }
}

macro_rules! launch_bounds {
    ($(#[$doc:meta])* $name:ident, $dims:literal, $($padding:ident)?) => {
        $(#[$doc])*
        #[repr(C)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name {
            shape: [i32; $dims],
            $($padding: u32,)?
            size: u64,
            coord_mult: u64,
        }

        impl $name {
            /// 各维须非零且落入i32；总数须落入u32发射计数。
            pub fn new(shape: [usize; $dims]) -> Result<Self, LauncherError> {
                let mut dims = [0i32; $dims];
                let mut size: u64 = 1;
                for (axis, &extent) in shape.iter().enumerate() {
                    if extent == 0 {
                        return Err(LauncherError::InvalidArray("发射维度为零"));
                    }
                    dims[axis] = i32::try_from(extent)
                        .map_err(|_| LauncherError::Overflow("launch_bounds_shape"))?;
                    size = size
                        .checked_mul(extent as u64)
                        .ok_or(LauncherError::Overflow("launch_bounds_size"))?;
                }
                if size > u64::from(u32::MAX) {
                    return Err(LauncherError::Overflow("launch_bounds_size"));
                }
                Ok(Self {
                    shape: dims,
                    $($padding: 0,)?
                    size,
                    // 沿用L2已核验取值。
                    coord_mult: 1,
                })
            }

            pub fn size(&self) -> u64 {
                self.size
            }

            pub fn shape(&self) -> [i32; $dims] {
                self.shape
            }
        }
    };
}

launch_bounds!(
    /// `launch_bounds_t<1>`，显式填充至8字节对齐。
    LaunchBounds1, 1, padding
);
launch_bounds!(
    /// `launch_bounds_t<2>`，与L2`WarpBounds2`一致。
    LaunchBounds2, 2,
);
launch_bounds!(
    /// `launch_bounds_t<3>`，与L2`WarpBounds3`一致。
    LaunchBounds3, 3, padding
);

#[cfg(feature = "cuda-probe")]
mod device {
    use super::{LaunchBounds1, LaunchBounds2, LaunchBounds3, WarpArray, WarpElement};
    use cudarc::driver::DeviceRepr;

    // SAFETY: 全部字段为固定宽度整数，填充均为显式字段。
    // 布局由本模块测试与L2定义对齐；无隐式填充字节。
    unsafe impl<T: WarpElement> DeviceRepr for WarpArray<T> {}
    // SAFETY: 同上；奇数维度含显式u32填充。
    unsafe impl DeviceRepr for LaunchBounds1 {}
    // SAFETY: 同上。
    unsafe impl DeviceRepr for LaunchBounds2 {}
    // SAFETY: 同上。
    unsafe impl DeviceRepr for LaunchBounds3 {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    const L2_KINEMATICS: &str = include_str!("../warp_kinematics.rs");
    const L2_GRAVITY: &str = include_str!("../warp_gravity.rs");

    #[test]
    fn warp_array_matches_frozen_l2_layout() {
        assert_eq!(size_of::<WarpArray<f32>>(), 56);
        assert_eq!(align_of::<WarpArray<f32>>(), 8);
        assert_eq!(offset_of!(WarpArray<f32>, data), 0);
        assert_eq!(offset_of!(WarpArray<f32>, grad), 8);
        assert_eq!(offset_of!(WarpArray<f32>, shape), 16);
        assert_eq!(offset_of!(WarpArray<f32>, strides), 32);
        assert_eq!(offset_of!(WarpArray<f32>, ndim), 48);
        assert_eq!(offset_of!(WarpArray<f32>, flags), 50);
        assert_eq!(offset_of!(WarpArray<f32>, padding), 52);
        assert_eq!(size_of::<WarpArray<f64>>(), 56);
        let fields = "    data: u64,\n    grad: u64,\n    shape: [i32; 4],\n    strides: [i32; 4],\n    ndim: u16,\n    flags: u16,\n    padding: u32,\n}";
        // `warp_com.rs`不定义数组结构；重复定义只有两份。
        for source in [L2_KINEMATICS, L2_GRAVITY] {
            assert!(source.contains(&format!("struct WarpArray {{\n{fields}")));
        }
    }

    #[test]
    fn launch_bounds_match_frozen_l2_layouts() {
        assert_eq!(size_of::<LaunchBounds1>(), 24);
        assert_eq!(size_of::<LaunchBounds2>(), 24);
        assert_eq!(size_of::<LaunchBounds3>(), 32);
        assert_eq!(align_of::<LaunchBounds1>(), 8);
        assert_eq!(offset_of!(LaunchBounds1, padding), 4);
        assert_eq!(offset_of!(LaunchBounds1, size), 8);
        assert_eq!(offset_of!(LaunchBounds1, coord_mult), 16);
        assert_eq!(offset_of!(LaunchBounds2, size), 8);
        assert_eq!(offset_of!(LaunchBounds2, coord_mult), 16);
        assert_eq!(offset_of!(LaunchBounds3, padding), 12);
        assert_eq!(offset_of!(LaunchBounds3, size), 16);
        assert_eq!(offset_of!(LaunchBounds3, coord_mult), 24);
        assert!(L2_KINEMATICS.contains(
            "struct WarpBounds2 {\n    shape: [i32; 2],\n    size: u64,\n    coord_mult: u64,\n}"
        ));
        assert!(L2_GRAVITY.contains(
            "struct WarpBounds3 {\n    shape: [i32; 3],\n    padding: u32,\n    size: u64,\n    coord_mult: u64,\n}"
        ));
        assert!(L2_KINEMATICS.contains("coord_mult: 1,"));
    }

    #[test]
    fn contiguous_array_uses_row_major_byte_strides() {
        let array = WarpArray::<f32>::contiguous(0x1000, &[3, 5, 7]).unwrap();
        assert_eq!(array.ndim(), 3);
        assert_eq!(array.shape(), [3, 5, 7]);
        assert_eq!(array.strides(), [140, 28, 4]);
        assert_eq!(array.byte_len(), 420);
        let doubles = WarpArray::<f64>::contiguous(0x1000, &[2, 6]).unwrap();
        assert_eq!(doubles.strides(), [48, 8]);
    }
}
