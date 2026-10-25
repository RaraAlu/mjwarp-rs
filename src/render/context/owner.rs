//! render context 的设备资源所有权（渲染/BVH S6b，cuda-probe）。
//!
//! 上游 `render_util.create_render_context` 把 BVH/mesh/texture/输出缓冲挂在返回的
//! `RenderContext` 上，由 Python GC 释放；产品侧把同一组资源收拢到显式 owner：
//!
//! - [`SceneState`]：当前场景几何（leaf 静态参数 + 当前位姿），是重拟合与渲染查询的共同输入；
//! - [`SceneResources`]：场景 BVH（S2a 构建/重拟合）+ GPU 查询器（S2b）；
//! - [`MeshResources`]：网格三角 BVH（S3）+ GPU 查询器；
//! - [`TextureResources`]：CUDA 纹理对象（S4）。
//!
//! owner 不隐藏刷新：`SceneResources::refit` 只由 `RenderContext::refit_bvh` 显式调用，
//! 渲染查询路径（`SceneResources::query`）不修改任何 BVH 状态。资源随 owner 析构释放。

use std::panic::catch_unwind;

use crate::diagnostics::TransferError;
use crate::render::bvh::{
    Aabb, BvhError, LeafGeometry, LeafRayGeometry, MeshBvh, MeshBvhError, MeshBvhInput,
    MeshBvhQuery, MeshRayGeometry, MeshRayHit, Ray, RayHit, SceneBvh, SceneBvhInput, SceneBvhQuery,
    build_scene_bvh, compute_leaf_aabb, refit_bvh,
};
use crate::render::texture::{TextureError, TextureObjects, TextureRegistry};
use crate::runtime::TransferSession;

/// leaf 的静态几何（MuJoCo `geom_type`/`geom_size` 与分组）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneLeaf {
    /// MuJoCo `geom_type`；S2a 支持 0、2..=8（hfield 等由 `LeafGeometry` 拒绝）。
    pub geom_type: i32,
    /// `geom_size` 的 f32 值；mesh 为 mesh bounds 半尺寸。
    pub size: [f32; 3],
    /// world 分组下标（`< worlds`）。
    pub group: u32,
    /// MuJoCo `geom_id`。
    pub geom_id: i32,
}

/// leaf 的当前位姿（上游 `geom_xpos`/`geom_xmat`）；渲染读取当前值，BVH 保留旧边界。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafPose {
    pub xpos: [f32; 3],
    /// 行主序 3x3（`geom_xmat` 语义）。
    pub xmat: [f32; 9],
}

impl LeafPose {
    fn rotation(self) -> [[f32; 3]; 3] {
        [
            [self.xmat[0], self.xmat[1], self.xmat[2]],
            [self.xmat[3], self.xmat[4], self.xmat[5]],
            [self.xmat[6], self.xmat[7], self.xmat[8]],
        ]
    }
}

/// 场景当前状态：leaf 静态参数与位姿一一对应。
#[derive(Clone, Debug, PartialEq)]
pub struct SceneState {
    leaves: Vec<SceneLeaf>,
    poses: Vec<LeafPose>,
    worlds: usize,
}

impl SceneState {
    /// 叶数与位姿数必须相等且非空，`worlds` 非零。
    pub fn new(
        leaves: Vec<SceneLeaf>,
        poses: Vec<LeafPose>,
        worlds: usize,
    ) -> Result<Self, OwnerError> {
        if worlds == 0 {
            return Err(OwnerError::EmptyWorlds);
        }
        if leaves.is_empty() {
            return Err(OwnerError::Bvh(BvhError::EmptyScene));
        }
        if leaves.len() != poses.len() {
            return Err(OwnerError::LengthMismatch {
                field: "scene_pose",
                expected: leaves.len(),
                actual: poses.len(),
            });
        }
        Ok(Self {
            leaves,
            poses,
            worlds,
        })
    }

    pub fn worlds(&self) -> usize {
        self.worlds
    }

    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    pub fn leaves(&self) -> &[SceneLeaf] {
        &self.leaves
    }

    pub fn poses(&self) -> &[LeafPose] {
        &self.poses
    }

    pub fn pose(&self, leaf: usize) -> Option<LeafPose> {
        self.poses.get(leaf).copied()
    }

    /// 更新当前位姿；不触碰已构建 BVH（上游「物理写 geom_xpos，BVH 需显式 refit」）。
    pub fn set_poses(&mut self, poses: &[LeafPose]) -> Result<(), OwnerError> {
        if poses.len() != self.poses.len() {
            return Err(OwnerError::LengthMismatch {
                field: "scene_pose",
                expected: self.poses.len(),
                actual: poses.len(),
            });
        }
        self.poses.copy_from_slice(poses);
        Ok(())
    }

    /// 由当前位姿逐 leaf 计算 AABB（S2a 冻结公式）；顺序与 `leaves` 一致。
    pub fn bounds(&self) -> Result<Vec<Aabb>, OwnerError> {
        let mut bounds = Vec::with_capacity(self.leaves.len());
        for (index, (leaf, pose)) in self.leaves.iter().zip(&self.poses).enumerate() {
            let geometry = LeafGeometry::from_geom_type(leaf.geom_type, leaf.size)
                .map_err(|error| OwnerError::LeafGeometry { leaf: index, error })?;
            let aabb = compute_leaf_aabb(&geometry, pose.xpos, pose.rotation())
                .map_err(|error| OwnerError::LeafBounds { leaf: index, error })?;
            bounds.push(aabb);
        }
        Ok(bounds)
    }

    /// 构建（或重建）场景 BVH；分组与 geom id 取 leaf 顺序。
    pub fn build_bvh(&self) -> Result<SceneBvh, OwnerError> {
        let bounds = self.bounds()?;
        let groups: Vec<u32> = self.leaves.iter().map(|leaf| leaf.group).collect();
        let geom_ids: Vec<i32> = self.leaves.iter().map(|leaf| leaf.geom_id).collect();
        build_scene_bvh(SceneBvhInput {
            bounds: &bounds,
            groups: &groups,
            geom_ids: &geom_ids,
            worlds: self.worlds,
        })
        .map_err(OwnerError::Bvh)
    }

    /// GPU 查询所需的逐 leaf 几何（geom_type/size/xpos/xmat），取当前位姿。
    pub fn ray_geometry(&self) -> Vec<LeafRayGeometry> {
        self.leaves
            .iter()
            .zip(&self.poses)
            .map(|(leaf, pose)| LeafRayGeometry {
                geom_type: leaf.geom_type,
                size: leaf.size,
                xpos: pose.xpos,
                xmat: pose.xmat,
            })
            .collect()
    }
}

/// 场景资源：S2a BVH + S2b 查询器，附显式重拟合计数。
pub struct SceneResources {
    bvh: SceneBvh,
    query: SceneBvhQuery,
    refits: u64,
}

impl SceneResources {
    pub fn build(session: &TransferSession, state: &SceneState) -> Result<Self, OwnerError> {
        let bvh = state.build_bvh()?;
        let query = SceneBvhQuery::new(session).map_err(OwnerError::Transfer)?;
        Ok(Self {
            bvh,
            query,
            refits: 0,
        })
    }

    pub fn bvh(&self) -> &SceneBvh {
        &self.bvh
    }

    /// 显式重拟合次数；只有 [`Self::refit`] 会增加。
    pub fn refits(&self) -> u64 {
        self.refits
    }

    /// 按当前位姿重拟合叶边界与内部节点；不重建拓扑。
    pub fn refit(&mut self, state: &SceneState) -> Result<(), OwnerError> {
        let bounds = state.bounds()?;
        refit_bvh(&mut self.bvh, &bounds).map_err(OwnerError::Bvh)?;
        self.refits += 1;
        Ok(())
    }

    /// 用**当前** BVH 与 `geometry` 查询一个 world；不触发重拟合。
    pub fn query(
        &self,
        world: usize,
        geometry: &[LeafRayGeometry],
        rays: &[Ray],
    ) -> Result<Vec<RayHit>, OwnerError> {
        self.query
            .query(world, &self.bvh, geometry, rays)
            .map_err(OwnerError::Transfer)
    }
}

/// 网格状态：局部系顶点与三角形索引。
#[derive(Clone, Debug, PartialEq)]
pub struct MeshState {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// 网格资源：S3 三角 BVH + 查询器；查询在网格局部系进行（世界变换属渲染内核）。
pub struct MeshResources {
    bvh: MeshBvh,
    query: MeshBvhQuery,
    geometry: MeshRayGeometry,
    refits: u64,
}

impl MeshResources {
    pub fn build(session: &TransferSession, state: &MeshState) -> Result<Self, OwnerError> {
        let bvh = MeshBvh::build(MeshBvhInput {
            vertices: &state.vertices,
            indices: &state.indices,
        })
        .map_err(OwnerError::Mesh)?;
        let query = MeshBvhQuery::new(session).map_err(OwnerError::Transfer)?;
        Ok(Self {
            bvh,
            query,
            geometry: MeshRayGeometry {
                geom_id: 0,
                xpos: [0.0; 3],
                xmat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            },
            refits: 0,
        })
    }

    pub fn triangle_count(&self) -> usize {
        self.bvh.triangle_count()
    }

    /// S7b1 渲染内核打包用的网格三角 BVH（crate 可见；不转移所有权、不触发 refit）。
    pub(crate) fn bvh(&self) -> &MeshBvh {
        &self.bvh
    }

    pub fn refits(&self) -> u64 {
        self.refits
    }

    /// 更新顶点（长度须与构建时一致）；不自动重拟合（S3 语义：`set_points` 后须 `refit`）。
    pub fn set_points(&mut self, vertices: &[[f32; 3]]) -> Result<(), OwnerError> {
        self.bvh.set_points(vertices).map_err(OwnerError::Mesh)
    }

    /// 显式重拟合网格 BVH。
    pub fn refit(&mut self) {
        self.bvh.refit();
        self.refits += 1;
    }

    /// 局部系网格射线查询；`max_t` 取 `f32::MAX`（渲染内核在 S7b 里按相机距离裁剪）。
    pub fn query(&self, rays: &[Ray], cull_backfaces: bool) -> Result<Vec<MeshRayHit>, OwnerError> {
        let max_t = vec![f32::MAX; rays.len()];
        self.query
            .query(&self.bvh, &self.geometry, rays, &max_t, cull_backfaces)
            .map_err(OwnerError::Transfer)
    }
}

/// CUDA 纹理对象集合（S4）；析构顺序为先纹理对象、后底层数组。
pub struct TextureResources {
    objects: TextureObjects,
}

impl TextureResources {
    pub fn new(session: &TransferSession, registry: &TextureRegistry) -> Result<Self, OwnerError> {
        let objects = TextureObjects::new(session, registry).map_err(OwnerError::Transfer)?;
        Ok(Self { objects })
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// 按模型纹理 id 采样固定 uv 列表（S4 硬件 LINEAR 路径）。
    pub fn sample(&self, texture: usize, uvs: &[[f32; 2]]) -> Result<Vec<[f32; 4]>, OwnerError> {
        self.objects
            .sample(texture, uvs)
            .map_err(OwnerError::Transfer)
    }
}

/// 设备显存 (free, total)；驱动不可用时返回 `None`（只用于测试与证据统计）。
pub fn device_memory() -> Option<(usize, usize)> {
    catch_unwind(|| cudarc::driver::result::mem_get_info().ok())
        .ok()
        .flatten()
}

/// 资源 owner 错误；全部在分配或发射前给出。
#[derive(Clone, Debug, PartialEq)]
pub enum OwnerError {
    Bvh(BvhError),
    Mesh(MeshBvhError),
    Texture(TextureError),
    Transfer(TransferError),
    /// world 数为零。
    EmptyWorlds,
    /// 并列输入长度不一致。
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 场景 world 数与解析后的 context 不一致。
    WorldsMismatch {
        expected: usize,
        actual: usize,
    },
    /// 某个 leaf 的几何参数不被 S2a 支持。
    LeafGeometry {
        leaf: usize,
        error: BvhError,
    },
    /// 某个 leaf 的位姿/尺寸非法。
    LeafBounds {
        leaf: usize,
        error: BvhError,
    },
    /// 网格下标越界。
    MeshIndex {
        index: usize,
        limit: usize,
    },
}

impl std::error::Error for OwnerError {}

impl std::fmt::Display for OwnerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bvh(error) => write!(formatter, "场景 BVH：{error}"),
            Self::Mesh(error) => write!(formatter, "网格 BVH：{error}"),
            Self::Texture(error) => write!(formatter, "纹理：{error}"),
            Self::Transfer(error) => write!(formatter, "设备传输：{error}"),
            Self::EmptyWorlds => write!(formatter, "场景 world 数不能为 0"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(formatter, "{field} 长度 {actual} != {expected}"),
            Self::WorldsMismatch { expected, actual } => {
                write!(formatter, "场景 world 数 {actual} != context {expected}")
            }
            Self::LeafGeometry { leaf, error } => {
                write!(formatter, "leaf {leaf} 几何：{error}")
            }
            Self::LeafBounds { leaf, error } => write!(formatter, "leaf {leaf} 包围盒：{error}"),
            Self::MeshIndex { index, limit } => {
                write!(formatter, "网格下标 {index} 超出 {limit}")
            }
        }
    }
}

impl From<BvhError> for OwnerError {
    fn from(error: BvhError) -> Self {
        Self::Bvh(error)
    }
}

impl From<MeshBvhError> for OwnerError {
    fn from(error: MeshBvhError) -> Self {
        Self::Mesh(error)
    }
}

impl From<TextureError> for OwnerError {
    fn from(error: TextureError) -> Self {
        Self::Texture(error)
    }
}

impl From<TransferError> for OwnerError {
    fn from(error: TransferError) -> Self {
        Self::Transfer(error)
    }
}
