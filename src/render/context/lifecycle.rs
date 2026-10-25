//! render context 生命周期与输出缓冲所有权（渲染/BVH S6b，cuda-probe）。
//!
//! 上游 `create_render_context` 的返回对象在 Python GC 时释放全部设备资源；本片给产品
//! 侧一个显式的 [`RenderContext`]：
//!
//! - `create`/`destroy` 成对，`Drop` 兜底，字段顺序即释放顺序；
//! - `refit_bvh` 是**唯一**会重拟合场景 BVH 的入口（上游语义：物理写位姿后需显式 refit）；
//! - `render_rays`/`mesh_rays` 是渲染消费 BVH 的查询路径：只用当前拥有的 BVH，不重拟合、
//!   不触发物理；像素内核（RGB/depth/seg 着色）属 S7b，本片不实现；
//! - [`OutputResources`] 按 S6a `OutputLayout` 分配 rgb/depth/seg 缓冲，禁用输出一律
//!   拒绝读取（`InputError::DisabledOutput`），与 `ImageBatchLayout` 口径一致。

use std::sync::atomic::{AtomicU64, Ordering};

use crate::diagnostics::{InputError, TransferError};
use crate::render::ImageOutput;
use crate::render::bvh::{MeshRayHit, Ray, RayHit};
use crate::render::texture::TextureRegistry;
use crate::runtime::launcher::{LauncherError, RenderArrayValue, RenderElem};
use crate::runtime::{TransferBuffer, TransferElement, TransferSession};

use super::OutputLayout;
use super::owner::{
    LeafPose, MeshResources, MeshState, OwnerError, SceneResources, SceneState, TextureResources,
};
use super::{ContextError, ResolvedContext};

static CREATED: AtomicU64 = AtomicU64::new(0);
static DESTROYED: AtomicU64 = AtomicU64::new(0);

/// context 创建/销毁计数 (created, destroyed)；用于生命周期证据，不含业务语义。
pub fn release_stats() -> (u64, u64) {
    (
        CREATED.load(Ordering::Relaxed),
        DESTROYED.load(Ordering::Relaxed),
    )
}

/// 输出缓冲当前内容；类型与通道对应。
#[derive(Clone, Debug, PartialEq)]
pub enum OutputValues {
    /// 每像素 packed RGBA（`pack_rgba_to_uint32` 口径）。
    Rgb(Vec<u32>),
    /// 每像素深度。
    Depth(Vec<f32>),
    /// 每像素 (id, type)。
    Seg(Vec<[i32; 2]>),
}

/// 按 `OutputLayout` 拥有的 rgb/depth/seg 设备缓冲。
///
/// 未启用的输出不分配、不可读：`read`/`byte_len` 返回 `DisabledOutput`。
pub struct OutputResources {
    layout: OutputLayout,
    nworld: usize,
    rgb: Option<TransferBuffer<u32>>,
    depth: Option<TransferBuffer<f32>>,
    seg: Option<TransferBuffer<i32>>,
}

impl OutputResources {
    /// 按布局分配启用通道；`nworld` 须非零。
    pub fn allocate(
        session: &TransferSession,
        layout: &OutputLayout,
        nworld: usize,
    ) -> Result<Self, OwnerError> {
        if nworld == 0 {
            return Err(OwnerError::LengthMismatch {
                field: "nworld",
                expected: 1,
                actual: 0,
            });
        }
        let rgb = allocate(session, layout.rgb_pixels, nworld, 0u32)?;
        let depth = allocate(session, layout.depth_pixels, nworld, 0.0f32)?;
        let seg = match layout.seg_slots.checked_mul(2) {
            Some(slots) if layout.seg_pixels > 0 => allocate(session, slots, nworld, 0i32)?,
            Some(_) => None,
            None => return Err(overflow("seg_slots")),
        };
        Ok(Self {
            layout: layout.clone(),
            nworld,
            rgb,
            depth,
            seg,
        })
    }

    pub fn layout(&self) -> &OutputLayout {
        &self.layout
    }

    pub fn nworld(&self) -> usize {
        self.nworld
    }

    /// 输出是否在本次 context 中启用（启用即有非零分配）。
    pub fn is_enabled(&self, output: ImageOutput) -> bool {
        match output {
            ImageOutput::Rgb => self.rgb.is_some(),
            ImageOutput::Depth => self.depth.is_some(),
            ImageOutput::Segmentation => self.seg.is_some(),
        }
    }

    /// 输出缓冲字节数；禁用输出返回 `DisabledOutput`。
    pub fn byte_len(&self, output: ImageOutput) -> Result<usize, OwnerError> {
        Ok(match output {
            ImageOutput::Rgb => self.rgb.as_ref().ok_or_else(disabled)?.byte_len(),
            ImageOutput::Depth => self.depth.as_ref().ok_or_else(disabled)?.byte_len(),
            ImageOutput::Segmentation => self.seg.as_ref().ok_or_else(disabled)?.byte_len(),
        })
    }

    /// 读回输出缓冲；禁用输出返回 `DisabledOutput`。
    pub fn read(&self, output: ImageOutput) -> Result<OutputValues, OwnerError> {
        match output {
            ImageOutput::Rgb => {
                let buffer = self.rgb.as_ref().ok_or_else(disabled)?;
                let mut values = vec![0u32; buffer.len()];
                buffer.read_range_into(0, &mut values)?;
                Ok(OutputValues::Rgb(values))
            }
            ImageOutput::Depth => {
                let buffer = self.depth.as_ref().ok_or_else(disabled)?;
                let mut values = vec![0.0f32; buffer.len()];
                buffer.read_range_into(0, &mut values)?;
                Ok(OutputValues::Depth(values))
            }
            ImageOutput::Segmentation => {
                let buffer = self.seg.as_ref().ok_or_else(disabled)?;
                let mut values = vec![0i32; buffer.len()];
                buffer.read_range_into(0, &mut values)?;
                Ok(OutputValues::Seg(values.as_chunks::<2>().0.to_vec()))
            }
        }
    }

    /// S7b1 渲染内核使用的 depth 设备缓冲实参（crate 可见；不扩大公共 API）。
    ///
    /// 输出未启用时返回 `LauncherError::Unsupported`；缓冲仍是 S6b 分配的那一份，
    /// 不转移所有权、不改生命周期。
    pub(crate) fn depth_render_arg(&self) -> Result<RenderArrayValue, LauncherError> {
        let buffer = self
            .depth
            .as_ref()
            .ok_or(LauncherError::Unsupported("depth 输出未启用"))?;
        render_output_arg(buffer, RenderElem::F32, buffer.len())
    }

    /// S7b1 渲染内核使用的 rgb 设备缓冲实参（crate 可见；本片只读不写 RGB）。
    #[allow(dead_code)] // S7b2 着色/RGB 内核接线后移除
    pub(crate) fn rgb_render_arg(&self) -> Result<RenderArrayValue, LauncherError> {
        let buffer = self
            .rgb
            .as_ref()
            .ok_or(LauncherError::Unsupported("rgb 输出未启用"))?;
        render_output_arg(buffer, RenderElem::U32, buffer.len())
    }

    /// S7b1 渲染内核使用的 seg 设备缓冲实参（两个 i32 一个 `(object_id, object_type)` 对）。
    pub(crate) fn seg_render_arg(&self) -> Result<RenderArrayValue, LauncherError> {
        let buffer = self
            .seg
            .as_ref()
            .ok_or(LauncherError::Unsupported("seg 输出未启用"))?;
        if !buffer.len().is_multiple_of(2) {
            return Err(LauncherError::InvalidSpan("seg 缓冲不是偶数个 i32"));
        }
        render_output_arg(buffer, RenderElem::Vec2I, buffer.len() / 2)
    }
    /// S7b2：与 `*_render_arg` 同缓冲，但描述符为上游 `array2d[world, pitch]` 形状。
    pub(crate) fn rgb_render_arg_2d(
        &self,
        nworld: usize,
    ) -> Result<RenderArrayValue, LauncherError> {
        let buffer = self
            .rgb
            .as_ref()
            .ok_or(LauncherError::Unsupported("rgb 输出未启用"))?;
        Self::render_output_arg_shaped(buffer, RenderElem::U32, &[nworld, self.layout().rgb_pixels])
    }

    /// S7b2：depth 的 `array2d[world, depth_pixels]` 描述符。
    pub(crate) fn depth_render_arg_2d(
        &self,
        nworld: usize,
    ) -> Result<RenderArrayValue, LauncherError> {
        let buffer = self
            .depth
            .as_ref()
            .ok_or(LauncherError::Unsupported("depth 输出未启用"))?;
        Self::render_output_arg_shaped(
            buffer,
            RenderElem::F32,
            &[nworld, self.layout().depth_pixels],
        )
    }

    /// S7b2：seg 的 `array2d[world, seg_slots]` 描述符（两个 i32 一个配对）。
    pub(crate) fn seg_render_arg_2d(
        &self,
        nworld: usize,
    ) -> Result<RenderArrayValue, LauncherError> {
        let buffer = self
            .seg
            .as_ref()
            .ok_or(LauncherError::Unsupported("seg 输出未启用"))?;
        Self::render_output_arg_shaped(
            buffer,
            RenderElem::Vec2I,
            &[nworld, self.layout().seg_slots],
        )
    }

    /// 输出缓冲 → 2D 渲染实参（共享 span，不改所有权/生命周期）。
    fn render_output_arg_shaped<T: TransferElement>(
        buffer: &TransferBuffer<T>,
        elem: RenderElem,
        shape: &[usize],
    ) -> Result<RenderArrayValue, LauncherError> {
        let span = buffer
            .launcher_span()
            .map_err(|error| LauncherError::Driver(format!("render-output-span: {error}")))?;
        RenderArrayValue::contiguous(span, elem, shape)
    }
}

/// 输出缓冲 → 渲染实参：形状 `[elements]`，元素类由调用方指定。
#[allow(dead_code)] // 由 S7b1 渲染内核接线后移除
fn render_output_arg<T: TransferElement>(
    buffer: &TransferBuffer<T>,
    elem: RenderElem,
    elements: usize,
) -> Result<RenderArrayValue, LauncherError> {
    let span = buffer
        .launcher_span()
        .map_err(|error| LauncherError::Driver(format!("render-output-span: {error}")))?;
    RenderArrayValue::contiguous(span, elem, &[elements])
}

fn disabled() -> OwnerError {
    OwnerError::Transfer(TransferError::Input(InputError::DisabledOutput))
}

fn overflow(field: &'static str) -> OwnerError {
    OwnerError::Transfer(TransferError::Input(InputError::Overflow { field }))
}

/// 按 `elements * nworld` 分配零初始化缓冲；`elements == 0` 时不分配。
fn allocate<T: TransferElement + Clone>(
    session: &TransferSession,
    elements: usize,
    nworld: usize,
    zero: T,
) -> Result<Option<TransferBuffer<T>>, OwnerError> {
    let count = elements
        .checked_mul(nworld)
        .ok_or_else(|| overflow("output_elements"))?;
    if count == 0 {
        return Ok(None);
    }
    let buffer = session.upload(&vec![zero; count])?;
    Ok(Some(buffer))
}

/// 一次 render context 的生命周期：拥有场景/网格/纹理资源与输出缓冲。
pub struct RenderContext {
    session: TransferSession,
    resolved: ResolvedContext,
    state: SceneState,
    scene: SceneResources,
    meshes: Vec<MeshResources>,
    textures: Option<TextureResources>,
    outputs: OutputResources,
}

impl RenderContext {
    /// 创建 context：构建设备资源并分配输出缓冲。
    ///
    /// 不发射任何渲染内核、不创建未被请求的纹理；`use_textures=false` 时纹理注册表
    /// 应为空（上游同语义）。
    pub fn create(
        session: &TransferSession,
        resolved: ResolvedContext,
        state: SceneState,
        meshes: Vec<MeshState>,
        textures: &TextureRegistry,
    ) -> Result<Self, LifecycleError> {
        if state.worlds() != resolved.nworld {
            return Err(LifecycleError::Owner(OwnerError::WorldsMismatch {
                expected: resolved.nworld,
                actual: state.worlds(),
            }));
        }
        let outputs = OutputResources::allocate(session, &resolved.output, resolved.nworld)?;
        let scene = SceneResources::build(session, &state)?;
        let mut mesh_resources = Vec::with_capacity(meshes.len());
        for mesh in &meshes {
            mesh_resources.push(MeshResources::build(session, mesh)?);
        }
        let texture_resources = if resolved.use_textures && !textures.is_empty() {
            Some(TextureResources::new(session, textures)?)
        } else {
            None
        };
        CREATED.fetch_add(1, Ordering::Relaxed);
        Ok(Self {
            session: session.clone(),
            resolved,
            state,
            scene,
            meshes: mesh_resources,
            textures: texture_resources,
            outputs,
        })
    }

    pub fn session(&self) -> &TransferSession {
        &self.session
    }

    pub fn resolved(&self) -> &ResolvedContext {
        &self.resolved
    }

    /// 当前场景状态（几何 + 位姿）；BVH 可能比它旧（显式 refit 前）。
    pub fn scene(&self) -> &SceneState {
        &self.state
    }

    pub fn outputs(&self) -> &OutputResources {
        &self.outputs
    }

    pub fn textures(&self) -> Option<&TextureResources> {
        self.textures.as_ref()
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// 场景 BVH 的显式重拟合次数。
    pub fn refits(&self) -> u64 {
        self.scene.refits()
    }

    /// S7b1 渲染内核打包用的 context 场景 BVH（crate 可见；不触发重拟合）。
    pub(crate) fn scene_bvh(&self) -> &crate::render::bvh::SceneBvh {
        self.scene.bvh()
    }

    /// 第 `mesh` 个 context 网格三角 BVH（S7b1 打包用；不转移所有权、不触发 refit）。
    pub(crate) fn mesh_bvh(&self, mesh: usize) -> Option<&crate::render::bvh::MeshBvh> {
        self.meshes.get(mesh).map(MeshResources::bvh)
    }

    /// 更新当前位姿（模拟物理写 `geom_xpos`/`geom_xmat`）；**不**重拟合 BVH。
    pub fn update_poses(&mut self, poses: &[LeafPose]) -> Result<(), LifecycleError> {
        self.state.set_poses(poses)?;
        Ok(())
    }

    /// 唯一的重拟合入口：按当前位姿更新叶边界与内部节点。
    pub fn refit_bvh(&mut self) -> Result<(), LifecycleError> {
        self.scene.refit(&self.state)?;
        Ok(())
    }

    /// 渲染消费路径：用当前拥有的场景 BVH + 当前位姿查询一个 world。
    ///
    /// 本片没有像素内核（S7b）；这里验证「陈旧 BVH 可渲染、refit 后更新、渲染不隐式刷新」。
    pub fn render_rays(&self, world: usize, rays: &[Ray]) -> Result<Vec<RayHit>, LifecycleError> {
        let geometry = self.state.ray_geometry();
        Ok(self.scene.query(world, &geometry, rays)?)
    }

    /// 渲染消费路径的网格查询（局部系，S3 语义）。
    pub fn mesh_rays(
        &self,
        mesh: usize,
        rays: &[Ray],
        cull_backfaces: bool,
    ) -> Result<Vec<MeshRayHit>, LifecycleError> {
        let limit = self.meshes.len();
        let resources = self
            .meshes
            .get(mesh)
            .ok_or(OwnerError::MeshIndex { index: mesh, limit })
            .map_err(LifecycleError::Owner)?;
        Ok(resources.query(rays, cull_backfaces)?)
    }

    /// 更新网格顶点（不自动重拟合；`refit_mesh` 显式触发）。
    pub fn set_mesh_points(
        &mut self,
        mesh: usize,
        vertices: &[[f32; 3]],
    ) -> Result<(), LifecycleError> {
        let limit = self.meshes.len();
        let resources = self
            .meshes
            .get_mut(mesh)
            .ok_or(OwnerError::MeshIndex { index: mesh, limit })
            .map_err(LifecycleError::Owner)?;
        resources.set_points(vertices)?;
        Ok(())
    }

    /// 网格 BVH 显式重拟合。
    pub fn refit_mesh(&mut self, mesh: usize) -> Result<(), LifecycleError> {
        let limit = self.meshes.len();
        let resources = self
            .meshes
            .get_mut(mesh)
            .ok_or(OwnerError::MeshIndex { index: mesh, limit })
            .map_err(LifecycleError::Owner)?;
        resources.refit();
        Ok(())
    }

    pub fn mesh_refits(&self, mesh: usize) -> Option<u64> {
        self.meshes.get(mesh).map(MeshResources::refits)
    }

    /// 显式销毁：消费 context 并按字段顺序释放输出缓冲、纹理对象、网格与场景资源。
    pub fn destroy(self) {
        drop(self);
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        DESTROYED.fetch_add(1, Ordering::Relaxed);
    }
}

/// context 生命周期错误。
#[derive(Clone, Debug, PartialEq)]
pub enum LifecycleError {
    /// context 参数解析错误（S6a）。
    Context(ContextError),
    /// 资源 owner 错误。
    Owner(OwnerError),
}

impl From<ContextError> for LifecycleError {
    fn from(error: ContextError) -> Self {
        Self::Context(error)
    }
}

impl From<OwnerError> for LifecycleError {
    fn from(error: OwnerError) -> Self {
        Self::Owner(error)
    }
}

impl std::error::Error for LifecycleError {}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Context(error) => write!(formatter, "context 参数：{error}"),
            Self::Owner(error) => write!(formatter, "context 资源：{error}"),
        }
    }
}
