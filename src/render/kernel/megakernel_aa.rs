//! S7b3 AA 子样本发射与解析（从 `megakernel.rs` 拆出，保持单文件 ≤500 行棘轮）。
//!
//! 上游 `render.py`：
//! ```text
//! nsamples = samples_per_pixel²
//! for sample in range(nsamples):
//!     launch _render_megakernel(..., ray_base = sample * total_rays)
//! if nsamples > 1:
//!     launch _aa_resolve(accum, 1/nsamples) -> rgb_data
//! ```
//!
//! 本模块只负责「按 sample 顺序发射」与「resolve 发射」；`aa_accum` 的累积在
//! `cuda/render_megakernel.cu` 的 `store_pixel` AA 分支，解析公式在 `cuda/aa_resolve.cu`。

use crate::render::context::RenderContext;
use crate::runtime::TransferSession;
use crate::runtime::launcher::{
    BlockRule, LaunchBounds2, LaunchDims, LauncherError, RenderArg, RenderElem, RenderScalar,
    RenderSignature,
};

use super::launch::RenderKernel;
use super::megakernel_arrays::{array, upload};
use super::{AA_RESOLVE_ENTRY, AA_RESOLVE_SLOTS, AA_RESOLVE_SOURCE};

/// 81 槽里 `ray_base` 的槽位下标（S7a 冻结槽序；AA 的 sample 循环只改这一槽）。
pub(crate) const RAY_BASE_SLOT: usize = 47;

const BLOCK: u32 = 256;

/// `aa_resolve` 内核（每帧在全部 sample 之后发射一次）。
pub(crate) struct AAResolve {
    kernel: RenderKernel,
}

impl AAResolve {
    pub(crate) fn compile(session: &TransferSession) -> Result<Self, LauncherError> {
        let signature =
            RenderSignature::new(AA_RESOLVE_ENTRY, BlockRule::WarpGrid, &AA_RESOLVE_SLOTS)?;
        let kernel = RenderKernel::compile(
            session,
            AA_RESOLVE_ENTRY,
            AA_RESOLVE_SOURCE,
            signature,
            AA_RESOLVE_ENTRY,
        )?;
        Ok(Self { kernel })
    }

    /// 上游 `_aa_resolve`：`c = accum * (1/nsamples)` → clamp[0,255] → pack。
    pub(crate) fn resolve(
        &self,
        session: &TransferSession,
        context: &RenderContext,
        nworld: usize,
        rgb_pixels: usize,
        nsamples: usize,
    ) -> Result<(), LauncherError> {
        let inv_n = 1.0f32
            / i32::try_from(nsamples).map_err(|_| LauncherError::Overflow("nsamples"))? as f32;
        let inv_n_buffer = upload(session, &[inv_n])?;
        let resolve_args = vec![
            RenderArg::Bounds2D(LaunchBounds2::new([nworld, rgb_pixels])?),
            RenderArg::Array(context.outputs().aa_accum_render_arg_2d(nworld)?),
            RenderArg::Array(context.outputs().rgb_render_arg_2d(nworld)?),
            array(&inv_n_buffer, RenderElem::F32, &[1])?,
        ];
        let grid = u32::try_from((nworld * rgb_pixels).div_ceil(BLOCK as usize))
            .map_err(|_| LauncherError::Overflow("aa_resolve_grid"))?;
        let dims = LaunchDims::new((grid, 1, 1), (BLOCK, 1, 1), 0)?;
        self.kernel.launch(session, &resolve_args, dims)
    }
}

/// 按 sample 顺序发射 megakernel：只改 `ray_base` 槽，其余实参整帧不变。
pub(crate) fn launch_samples(
    kernel: &RenderKernel,
    session: &TransferSession,
    args: &mut [RenderArg],
    dims: LaunchDims,
    nsamples: usize,
    total_rays: usize,
) -> Result<(), LauncherError> {
    for sample in 0..nsamples {
        let offset = sample
            .checked_mul(total_rays)
            .ok_or(LauncherError::Overflow("ray_base"))?;
        let ray_base = i32::try_from(offset).map_err(|_| LauncherError::Overflow("ray_base"))?;
        args[RAY_BASE_SLOT] = RenderArg::Scalar(RenderScalar::I32(ray_base));
        kernel.launch(session, args, dims)?;
    }
    Ok(())
}
