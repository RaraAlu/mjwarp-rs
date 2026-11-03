//! R06-S3b③：flex 宿主通道的生产渲染调用点（`FlexHostAssembly::render_with`）。

use super::FlexHostAssembly;
use crate::model::render::RenderFields;
use crate::render::bvh::FlexMeshChannel;
use crate::render::kernel::MegakernelRenderer;
use crate::render::kernel::cast::CameraPose;
use crate::render::{CameraModel, RayBuffer, RenderContext};
use crate::runtime::launcher::LauncherError;

impl FlexHostAssembly<'_> {
    /// 生产调用点：把组装好的宿主 flex 通道接进渲染器（`mesh=None` 走 S3a 1D 胶囊路径）。
    ///
    /// 无 flex 时调用方按 [`Self::build`] 的 `Ok(None)` 走既有 `MegakernelRenderer::render`；
    /// 本方法只服务 `Some` 分支：`mesh=None` 时构出的 `RenderPass` 与父版本 `render` 的
    /// `{world: None, flex: None, flex_mesh: None}` 同构（同一 `render_impl`，无额外变换）。
    // 8 参数：与 `MegakernelRenderer::render` 的渲染表面一一对应（targets + flex 通道）。
    #[allow(clippy::too_many_arguments)]
    pub fn render_with(
        &self,
        renderer: &MegakernelRenderer,
        context: &RenderContext,
        fields: &RenderFields,
        model: &CameraModel,
        poses: &[CameraPose],
        rays: &RayBuffer,
        mesh: Option<&FlexMeshChannel<'_>>,
    ) -> Result<(), LauncherError> {
        let inputs = self.inputs();
        renderer.render_with_flex(context, fields, model, poses, rays, Some(&inputs), mesh)
    }
}
