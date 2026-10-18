//! 统一发射器的运行时基础（M1）。
//! 本模块只描述配方、参数布局与绑定校验。
//! 它不编译、不加载，也不发射内核。
//! 物理调用迁移属于M3；接口尚未冻结。
//! 设计见`docs/unified-launcher.md`。

mod abi;
mod alias;
mod recipe;

pub use abi::{
    AbiSignature, Access, AliasException, BlockRule, ElemType, KernelAbi, LaunchBounds1,
    LaunchBounds2, LaunchBounds3, ProofKind, SlotDesc, SlotKind, SlotRole, WarpArray, WarpElement,
};
pub use alias::{DeviceSpan, SlotBinding, SlotRequirement, rebind_values, validate_bindings};
pub use recipe::{ArchTarget, CompileRecipe, CppStd, PrecisionGate, RecipePurpose};

/// 发射器基础错误。
/// 每类错误都指明槽位或字段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LauncherError {
    /// 配方自身违反用途约束。
    InvalidRecipe(&'static str),
    /// 配方用途与内核要求不符。
    RecipePurpose {
        expected: RecipePurpose,
        actual: RecipePurpose,
    },
    /// 参数签名不成立。
    InvalidSignature {
        abi: &'static str,
        reason: &'static str,
    },
    /// Warp数组或边界参数无效。
    InvalidArray(&'static str),
    /// 整数换算溢出。
    Overflow(&'static str),
    /// 设备区间无效。
    InvalidSpan(&'static str),
    /// 缓冲槽位缺少绑定。
    MissingBinding { slot: usize },
    /// 绑定指向非缓冲槽位或越界槽位。
    UnexpectedBinding { slot: usize },
    /// 同一槽位绑定两次。
    DuplicateBinding { slot: usize },
    /// 元素类型与槽位声明不符。
    ElementMismatch { slot: usize },
    /// 缓冲来自不同上下文。
    ContextMismatch { slot: usize },
    /// 设备地址未按元素对齐。
    Misaligned { slot: usize },
    /// 缓冲容量不足。
    Capacity {
        slot: usize,
        required: u64,
        actual: u64,
    },
    /// 两个槽位违反别名约束。
    Alias { first: usize, second: usize },
    /// 首版禁止重绑地址类槽位。
    AddressSlotRebind { slot: usize },
    /// 重绑缓冲小于原缓冲。
    RebindShrink { slot: usize },
}

impl std::fmt::Display for LauncherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRecipe(reason) => write!(f, "编译配方无效：{reason}"),
            Self::RecipePurpose { expected, actual } => {
                write!(f, "配方用途不符：需要{expected:?}，实际{actual:?}")
            }
            Self::InvalidSignature { abi, reason } => write!(f, "参数签名{abi}无效：{reason}"),
            Self::InvalidArray(reason) => write!(f, "Warp参数无效：{reason}"),
            Self::Overflow(field) => write!(f, "整数溢出：{field}"),
            Self::InvalidSpan(reason) => write!(f, "设备区间无效：{reason}"),
            Self::MissingBinding { slot } => write!(f, "槽位{slot}缺少绑定"),
            Self::UnexpectedBinding { slot } => write!(f, "槽位{slot}不接受缓冲绑定"),
            Self::DuplicateBinding { slot } => write!(f, "槽位{slot}重复绑定"),
            Self::ElementMismatch { slot } => write!(f, "槽位{slot}元素类型不符"),
            Self::ContextMismatch { slot } => write!(f, "槽位{slot}来自其他上下文"),
            Self::Misaligned { slot } => write!(f, "槽位{slot}地址未对齐"),
            Self::Capacity {
                slot,
                required,
                actual,
            } => write!(
                f,
                "槽位{slot}容量不足：需要{required}字节，实际{actual}字节"
            ),
            Self::Alias { first, second } => write!(f, "槽位{first}与{second}违反别名约束"),
            Self::AddressSlotRebind { slot } => write!(f, "槽位{slot}属于地址类，禁止重绑"),
            Self::RebindShrink { slot } => write!(f, "槽位{slot}重绑缓冲小于原缓冲"),
        }
    }
}

impl std::error::Error for LauncherError {}
