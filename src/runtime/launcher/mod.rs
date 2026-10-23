//! 统一发射器的运行时基础（M1、M2）。
//! M1描述配方、参数布局与绑定校验。
//! M2增加产物缓存与上下文模块缓存。
//! 本模块仍不发射内核。
//! 物理调用迁移属于M3；接口尚未冻结。
//! 设计见`docs/unified-launcher.md`。

mod abi;
mod alias;
#[cfg(feature = "cuda-probe")]
mod artifact;
mod checked;
#[cfg(feature = "cuda-probe")]
mod frozen;
mod graph;
#[cfg(feature = "cuda-probe")]
mod jitlink;
mod launch;
#[cfg(feature = "cuda-probe")]
mod mathdx;
#[cfg(feature = "cuda-probe")]
mod module;
mod recipe;
#[cfg(feature = "cuda-probe")]
mod route;
mod submit;
#[cfg(feature = "cuda-probe")]
mod warp;

pub use abi::{
    AbiSignature, Access, AliasException, BlockRule, ElemType, KernelAbi, LaunchBounds1,
    LaunchBounds2, LaunchBounds3, ProofKind, SlotDesc, SlotKind, SlotRole, WarpArray, WarpElement,
};
pub use alias::{DeviceSpan, SlotBinding, SlotRequirement, rebind_values, validate_bindings};
#[cfg(feature = "cuda-probe")]
pub use artifact::{
    Artifact, ArtifactCache, ArtifactKey, ArtifactStats, HeaderFile, LTOIR_OPTION_COUNT, Ltoir,
    LtoirCubin, LtoirInput, MAX_ARCH, MAX_LTOIR_SOURCE_BYTES, MIN_ARCH, NvrtcSource, compile_ltoir,
    link_ltoir, ltoir_options,
};
pub use checked::{Checked, CheckedBinding, MetadataProof, MetadataRevision};
#[cfg(feature = "cuda-probe")]
pub use frozen::FrozenPtx;
pub use graph::{
    Allocation, CaptureGuard, CaptureHost, CapturedGraph, CapturedNode, GraphInstance, GraphState,
    NodeId, RecycleId, ReplayId, SlotUpdate,
};
#[cfg(feature = "cuda-probe")]
pub use jitlink::{
    LinkInput, LinkInputKind, LinkedCubin, MAX_LINK_INPUT_BYTES, MAX_LINK_INPUTS,
    NVJITLINK_VERSION, NvJitLink,
};
pub use launch::{
    DeviceSharedMemoryLimits, Kernel, KernelSpec, LaunchDims, LaunchPlan, SharedMemoryLimits,
};
#[cfg(feature = "cuda-probe")]
pub use mathdx::{MATHDX_VERSION, MAX_DIM, MAX_THREADS, MathDx, SolverLto, SolverSpec};
#[cfg(feature = "cuda-probe")]
pub use module::{KernelHandle, ModuleCache, ModuleStats};
pub use recipe::{ArchTarget, CompileRecipe, CppStd, PrecisionGate, RecipePurpose};
#[cfg(feature = "cuda-probe")]
pub use route::{
    KernelArtifact, KernelArtifactMetadata, KernelSourceRoute, MATHDX_NVRTC_VERSION, RouteKey,
    RouteOptIn, build_mathdx_lto_kernel, require_mathdx_nvrtc, require_route_enabled,
};
pub use submit::{
    AccessMode, BufferAccess, Fence, RetainedResources, StreamKey, SubmissionId, SubmissionState,
    SubmitQueue,
};
#[cfg(feature = "cuda-probe")]
pub use warp::{L2_BLOCK, L2Completion, WarpArg, WarpBounds, WarpLaunch, f32_arg, i32_arg};

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
    /// 内核来源无效。
    InvalidSource(&'static str),
    /// 当前阶段不支持该能力。
    Unsupported(&'static str),
    /// NVRTC不可用或编译失败。
    Compiler(String),
    /// nvJitLink加载、版本、链接或产物检查失败。
    Linker(String),
    /// 产物未通过配方的精度检查。
    Precision(String),
    /// 磁盘产物读写、损坏或键不符。
    Cache { stage: &'static str, detail: String },
    /// 驱动加载模块或函数失败。
    Driver(String),
    /// 模块缓存收到其他上下文。
    ModuleContext,
    /// 元数据内容不满足证明规则。
    InvalidMetadata(&'static str),
    /// 元数据证明已经失效。
    StaleMetadata,
    /// 元数据证明不属于本次发射。
    MetadataMismatch(&'static str),
    /// 地址类槽位缺少证明。
    MissingMetadataProof { slot: usize },
    /// 地址类槽位重复提供证明。
    DuplicateMetadataProof { slot: usize },
    /// 跨流重叠访问缺少依赖声明；`blocking`为阻断提交编号。
    AccessConflict { blocking: u64 },
    /// 提交编号不存在或已回收。
    UnknownSubmission(u64),
    /// 重复声明同一依赖。
    DuplicateDependency(u64),
    /// 上游提交失败，本提交按归属失败。
    FaultedUpstream(u64),
    /// 同一上下文已有捕获，或捕获已结束。
    CaptureBusy,
    /// 图节点不存在。
    UnknownNode(u64),
    /// 重放编号不存在或已完成。
    UnknownReplay(u64),
    /// 图实例已作废，须重新捕获。
    GraphInvalid,
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
            Self::InvalidSource(reason) => write!(f, "内核来源无效：{reason}"),
            Self::Unsupported(reason) => write!(f, "暂不支持：{reason}"),
            Self::Compiler(detail) => write!(f, "编译失败：{detail}"),
            Self::Linker(detail) => write!(f, "nvJitLink失败：{detail}"),
            Self::Precision(detail) => write!(f, "精度检查失败：{detail}"),
            Self::Cache { stage, detail } => write!(f, "产物缓存失败：{stage}：{detail}"),
            Self::Driver(detail) => write!(f, "驱动加载失败：{detail}"),
            Self::ModuleContext => write!(f, "模块缓存属于其他上下文"),
            Self::InvalidMetadata(reason) => write!(f, "元数据无效：{reason}"),
            Self::StaleMetadata => write!(f, "元数据证明已经失效"),
            Self::MetadataMismatch(reason) => write!(f, "元数据证明不匹配：{reason}"),
            Self::MissingMetadataProof { slot } => write!(f, "槽位{slot}缺少元数据证明"),
            Self::DuplicateMetadataProof { slot } => write!(f, "槽位{slot}重复元数据证明"),
            Self::AccessConflict { blocking } => {
                write!(f, "跨流访问冲突：须声明对提交{blocking}的依赖")
            }
            Self::UnknownSubmission(id) => write!(f, "提交{id}不存在或已回收"),
            Self::DuplicateDependency(id) => write!(f, "重复声明依赖提交{id}"),
            Self::FaultedUpstream(id) => write!(f, "上游提交{id}失败，本提交按归属失败"),
            Self::CaptureBusy => write!(f, "同一上下文已有捕获，拒绝嵌套"),
            Self::UnknownNode(node) => write!(f, "图节点{node}不存在"),
            Self::UnknownReplay(id) => write!(f, "重放{id}不存在或已完成"),
            Self::GraphInvalid => write!(f, "图实例已作废，须重新捕获"),
        }
    }
}

impl std::error::Error for LauncherError {}
