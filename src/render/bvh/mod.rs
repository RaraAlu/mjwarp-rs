//! 场景 BVH 的宿主构建、重拟合与语义查询（渲染/BVH S2a）。
//! 场景层覆盖 primitive geoms 与 mesh 包围盒；网格三角形 BVH 在 `mesh` 子模块
//! （S3：构建/重拟合/顶点更新），设备命中在 `cuda/scene.cu`（S2b）与
//! `cuda/mesh.cu`（S3）。
//!
//! AABB 的 f32 运算顺序按冻结 MJWarp 3.15 参考钉死（NVRTC 对 `wp::mul` /
//! `wp::dot` 的乘加收缩顺序，见 `aabb` 子模块与
//! `docs/development-evidence/render-bvh/s2a-scene-bvh-host/`）。
//! 节点布局只对自身实现有效，验收不对节点布局做断言，只看语义结果。

mod aabb;
mod builder;
#[cfg(feature = "cuda-probe")]
pub mod cuda;
mod mesh;
#[cfg(feature = "cuda-probe")]
mod mesh_query;
mod query;
mod refit;

use std::fmt;

pub use aabb::{GeomKind, LeafGeometry, compute_leaf_aabb, mesh_half_extents};
pub use builder::{BvhNode, NodeKind, SceneBvh, SceneBvhInput, build_scene_bvh};
#[cfg(feature = "cuda-probe")]
pub use cuda::{LeafRayGeometry, Ray, RayHit, SceneBvhQuery};
pub use mesh::{MeshBvh, MeshBvhError, MeshBvhInput, MeshNode, MeshNodeKind};
#[cfg(feature = "cuda-probe")]
pub use mesh_query::{MeshBvhQuery, MeshRayGeometry, MeshRayHit};
pub use refit::refit_bvh;

/// 轴对齐包围盒，分量保持 f32 位型。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub lower: [f32; 3],
    pub upper: [f32; 3],
}

impl Aabb {
    /// 逐元素检查有限且 `lower <= upper` 后构造。
    pub fn new(lower: [f32; 3], upper: [f32; 3]) -> Result<Self, BvhError> {
        if !lower
            .iter()
            .chain(upper.iter())
            .all(|value| value.is_finite())
        {
            return Err(BvhError::InvalidAabb { leaf: usize::MAX });
        }
        for axis in 0..3 {
            if lower[axis] > upper[axis] {
                return Err(BvhError::InvalidAabb { leaf: usize::MAX });
            }
        }
        Ok(Self { lower, upper })
    }

    /// 是否为有限且有序的包围盒。
    pub fn is_valid(&self) -> bool {
        self.lower
            .iter()
            .chain(self.upper.iter())
            .all(|value| value.is_finite())
            && (0..3).all(|axis| self.lower[axis] <= self.upper[axis])
    }

    /// 两包围盒的并集；分量取 min/max 保持位型。
    pub fn union(self, other: Self) -> Self {
        let mut lower = self.lower;
        let mut upper = self.upper;
        for axis in 0..3 {
            lower[axis] = self.lower[axis].min(other.lower[axis]);
            upper[axis] = self.upper[axis].max(other.upper[axis]);
        }
        Self { lower, upper }
    }

    /// 点是否在闭区间内。
    pub fn contains_point(self, point: [f32; 3]) -> bool {
        (0..3).all(|axis| point[axis] >= self.lower[axis] && point[axis] <= self.upper[axis])
    }

    /// 中心点；只用于划分启发式，不参与验收。
    pub fn center(self) -> [f32; 3] {
        let mut center = [0.0f32; 3];
        for (axis, value) in center.iter_mut().enumerate() {
            *value = 0.5 * (self.lower[axis] + self.upper[axis]);
        }
        center
    }

    /// 射线（含原点在盒内）与盒的 slab 相交测试。
    pub fn intersects_ray(self, origin: [f32; 3], direction: [f32; 3]) -> bool {
        let mut t_min = 0.0f32;
        let mut t_max = f32::INFINITY;
        for axis in 0..3 {
            let offset = origin[axis];
            let delta = direction[axis];
            if delta == 0.0 {
                if offset < self.lower[axis] || offset > self.upper[axis] {
                    return false;
                }
                continue;
            }
            let mut near = (self.lower[axis] - offset) / delta;
            let mut far = (self.upper[axis] - offset) / delta;
            if near > far {
                std::mem::swap(&mut near, &mut far);
            }
            t_min = t_min.max(near);
            t_max = t_max.min(far);
            if t_min > t_max {
                return false;
            }
        }
        true
    }
}

/// 场景 BVH 宿主侧错误；拒绝面在分配与构建前给出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BvhError {
    /// 场景没有任何 leaf。
    EmptyScene,
    /// `worlds` 为 0。
    InvalidWorldCount { worlds: usize },
    /// 某个 world 分组没有 leaf。
    EmptyWorld { world: usize },
    /// 并列输入长度不一致。
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// leaf 的 group 超出 world 数。
    GroupOutOfRange {
        leaf: usize,
        group: usize,
        worlds: usize,
    },
    /// 查询的 world 超出 world 数。
    WorldOutOfRange { world: usize, worlds: usize },
    /// leaf 数超过节点数组可表示范围。
    LeafCountOverflow { leaves: usize },
    /// AABB 非有限或上下界颠倒。
    InvalidAabb { leaf: usize },
    /// leaf 几何输入含非有限量。
    NonFinite { field: &'static str, index: usize },
    /// 不支持在此切片处理的 geom 类型（hfield/SDF 等）。
    UnsupportedGeomType { geom_type: i32 },
    /// mesh 类型 leaf 缺少 mesh bounds。
    MissingMeshBounds,
}

impl fmt::Display for BvhError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyScene => write!(formatter, "场景没有可构建的 leaf"),
            Self::InvalidWorldCount { worlds } => {
                write!(formatter, "world 数不能为 0（实际 {worlds}）")
            }
            Self::EmptyWorld { world } => write!(formatter, "world {world} 分组没有 leaf"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} 长度不匹配：期望 {expected}，实际 {actual}"
            ),
            Self::GroupOutOfRange {
                leaf,
                group,
                worlds,
            } => write!(
                formatter,
                "leaf {leaf} 的 group {group} 超出 world 数 {worlds}"
            ),
            Self::WorldOutOfRange { world, worlds } => {
                write!(formatter, "查询 world {world} 超出 world 数 {worlds}")
            }
            Self::LeafCountOverflow { leaves } => {
                write!(formatter, "leaf 数 {leaves} 溢出节点数组容量")
            }
            Self::InvalidAabb { leaf } => {
                write!(formatter, "leaf {leaf} 的 AABB 非有限或上下界颠倒")
            }
            Self::NonFinite { field, index } => write!(formatter, "{field}[{index}] 不是有限数"),
            Self::UnsupportedGeomType { geom_type } => {
                write!(formatter, "S2a 不支持的 geom 类型 {geom_type}")
            }
            Self::MissingMeshBounds => write!(formatter, "mesh leaf 缺少 mesh bounds"),
        }
    }
}

impl std::error::Error for BvhError {}

impl std::fmt::Display for MeshBvhError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMesh => write!(formatter, "网格没有三角形"),
            Self::InvalidIndexCount { len } => write!(formatter, "索引长度 {len} 不是 3 的倍数"),
            Self::TriangleCountOverflow { triangles } => {
                write!(formatter, "三角形数 {triangles} 溢出")
            }
            Self::IndexOutOfRange {
                index, vertices, ..
            } => {
                write!(formatter, "索引 {index} 超出顶点数 {vertices}")
            }
            Self::NonFiniteVertex { index } => write!(formatter, "顶点 {index} 不是有限数"),
            Self::VertexCountMismatch { expected, actual } => {
                write!(formatter, "顶点数 {expected}/{actual} 不符")
            }
            Self::InvalidBounds { triangle } => write!(formatter, "三角形 {triangle} 的包围盒非法"),
        }
    }
}

impl std::error::Error for MeshBvhError {}

/// f64 模型字段转 f32 顶点；每分量一次正确舍入（与 MJWarp `dtype=wp.vec3` 一致）。
pub fn vertices_from_f64(values: &[f64]) -> Result<Vec<[f32; 3]>, BvhError> {
    if !values.len().is_multiple_of(3) {
        return Err(BvhError::LengthMismatch {
            field: "mesh_vert",
            expected: values.len() / 3 * 3,
            actual: values.len(),
        });
    }
    values
        .chunks(3)
        .enumerate()
        .map(|(index, chunk)| {
            if chunk.iter().any(|value| !value.is_finite()) {
                return Err(BvhError::NonFinite {
                    field: "mesh_vert",
                    index,
                });
            }
            Ok([chunk[0] as f32, chunk[1] as f32, chunk[2] as f32])
        })
        .collect()
}
