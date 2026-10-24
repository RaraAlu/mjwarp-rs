//! 渲染发射 ABI（渲染/BVH S7a）。
//!
//! 只做声明式签名、发射前校验与多参数发射；不做像素内核（S7b）、不引入新的安全
//! 边界（校验不通过一律在 launch 前拒绝）。实参面与冻结上游 `_render_megakernel`
//! 的约 81 个参数一一对应（1 个 `launch_bounds_t<2>`、73 个 `array_t<T>`、7 个标量）。
//!
//! 与四缓冲物理 ABI 的区别：参数个数/种类不固定、支持 u64/bool 标量与 13 类数组元素、
//! 多输出；执行仍走同一 `TransferSession` 的流与上下文。

use std::sync::Arc;

use cudarc::driver::{LaunchConfig as CudaLaunchConfig, PushKernelArg};

use crate::runtime::TransferSession;

use super::{BlockRule, KernelHandle, LaunchDims, LauncherError};

mod args;

pub use args::{
    MEGAKERNEL_SLOTS, RenderArg, RenderArgKind, RenderArray, RenderArrayValue, RenderElem,
    RenderFamily, RenderScalar, RenderScalarKind, Texture2d,
};

/// 渲染内核的静态签名。
#[derive(Clone, Copy, Debug)]
pub struct RenderSignature {
    name: &'static str,
    block: BlockRule,
    slots: &'static [RenderArgKind],
}

impl RenderSignature {
    /// 首槽位必须是 `launch_bounds_t<2>`，其余槽位不得再出现边界。
    pub fn new(
        name: &'static str,
        block: BlockRule,
        slots: &'static [RenderArgKind],
    ) -> Result<Self, LauncherError> {
        let fail = |reason| Err(LauncherError::InvalidSignature { abi: name, reason });
        if slots.is_empty() {
            return fail("渲染签名没有槽位");
        }
        if slots[0] != RenderArgKind::Bounds2D {
            return fail("渲染签名首槽位须为launch_bounds_t<2>");
        }
        if slots[1..].contains(&RenderArgKind::Bounds2D) {
            return fail("渲染边界只能位于首槽位");
        }
        Ok(Self { name, block, slots })
    }

    /// 冻结上游 `_render_megakernel` 的签名（81 槽位、Warp 默认 256 线程网格）。
    pub const MEGAKERNEL: Self = Self {
        name: "_render_megakernel",
        block: BlockRule::WarpGrid,
        slots: &MEGAKERNEL_SLOTS,
    };

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn block(&self) -> BlockRule {
        self.block
    }

    pub fn slots(&self) -> &'static [RenderArgKind] {
        self.slots
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

/// 已通过宿主校验的渲染发射计划；不能执行GPU工作。
#[derive(Clone, Debug)]
pub struct RenderPlan {
    signature: RenderSignature,
    dims: LaunchDims,
    context: u64,
}

impl RenderPlan {
    /// 逐槽校验实参数量、种类、数组对齐/容量/上下文与线程块规则。
    pub fn plan(
        signature: RenderSignature,
        args: &[RenderArg],
        dims: LaunchDims,
    ) -> Result<Self, LauncherError> {
        let slots = signature.slots();
        if args.len() != slots.len() {
            return Err(LauncherError::RenderArgCount {
                expected: slots.len(),
                actual: args.len(),
            });
        }
        let expected_block = (signature.block().threads(), 1, 1);
        if dims.block() != expected_block {
            return Err(LauncherError::RenderBounds { slot: 0 });
        }
        let mut context = None;
        for (slot, (arg, expected)) in args.iter().zip(slots).enumerate() {
            let actual = arg.kind();
            if actual != *expected {
                return Err(LauncherError::RenderArgKind {
                    slot,
                    expected: expected.name(),
                    actual: actual.name(),
                });
            }
            let RenderArg::Array(value) = arg else {
                continue;
            };
            value.array().check(slot, value.elem(), value.span())?;
            match context {
                None => context = Some(value.span().context()),
                Some(id) if id != value.span().context() => {
                    return Err(LauncherError::ContextMismatch { slot });
                }
                Some(_) => {}
            }
        }
        let context = context.ok_or(LauncherError::InvalidSignature {
            abi: signature.name(),
            reason: "渲染签名没有任何数组槽位",
        })?;
        Ok(Self {
            signature,
            dims,
            context,
        })
    }

    /// 冻结上游 `_render_megakernel` 计划的便捷入口。
    pub fn megakernel(args: &[RenderArg], dims: LaunchDims) -> Result<Self, LauncherError> {
        Self::plan(RenderSignature::MEGAKERNEL, args, dims)
    }

    pub fn signature(&self) -> RenderSignature {
        self.signature
    }

    pub fn dims(&self) -> LaunchDims {
        self.dims
    }

    /// 在计划与实参上同步发射；返回前等待完成并检查驱动错误。
    ///
    /// 发射前复核：实参数量、执行会话与实参缓冲一致、句柄与流同上下文。
    pub fn launch(
        &self,
        handle: &KernelHandle,
        session: &TransferSession,
        args: &[RenderArg],
    ) -> Result<(), LauncherError> {
        if args.len() != self.signature.len() {
            return Err(LauncherError::RenderArgCount {
                expected: self.signature.len(),
                actual: args.len(),
            });
        }
        if self.context != session.span_context() {
            return Err(LauncherError::ContextMismatch { slot: 0 });
        }
        let stream = session.stream_ref();
        if !Arc::ptr_eq(handle.context(), stream.context()) {
            return Err(LauncherError::ModuleContext);
        }
        let dims = self.dims;
        session
            .run_crate("render-launch", || {
                let mut builder = stream.launch_builder(handle.function());
                for arg in args {
                    match arg {
                        RenderArg::Bounds2D(bounds) => {
                            builder.arg(bounds);
                        }
                        RenderArg::Array(value) => {
                            builder.arg(value.array());
                        }
                        RenderArg::Scalar(RenderScalar::I32(value)) => {
                            builder.arg(value);
                        }
                        RenderArg::Scalar(RenderScalar::U64(value)) => {
                            builder.arg(value);
                        }
                        RenderArg::Scalar(RenderScalar::Bool(value)) => {
                            builder.arg(value);
                        }
                    }
                }
                // SAFETY: 数量/种类/对齐/容量/上下文与网格已在 `plan` 与本函数复核；
                // 内核只访问这些缓冲的声明范围，发射同步完成。
                unsafe {
                    builder.launch(CudaLaunchConfig {
                        grid_dim: dims.grid(),
                        block_dim: dims.block(),
                        shared_mem_bytes: dims.shared_bytes(),
                    })?;
                }
                Ok(())
            })
            .map_err(|error| LauncherError::Driver(format!("render-launch: {error}")))
    }
}
