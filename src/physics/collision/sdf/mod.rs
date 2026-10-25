//! G10@P5 G3：体积 SDF 宿主数学（对齐 pinned `mujoco_warp/_src/collision_sdf.py`）。
//! 只读宿主实现：不实装 GPU 内核、不分配设备缓冲。
//! 判据见证据目录；本模块只提供 `sdf()`/`sdf_grad()`/`clearance`/梯度下降与每初始点接触候选。

mod descent;
mod narrowphase;
mod octree;
mod primitives;
#[cfg(test)]
mod tests;

// G3 只交付宿主数学：公开导出供后续 G5 内核片与测试使用。
#[allow(unused_imports)]
pub use descent::{
    OptimizationParams, clearance, compute_grad, gradient_descent, gradient_step, sdf, sdf_grad,
};
#[allow(unused_imports)]
pub use narrowphase::{SdfContact, SdfPair, sdf_pair_contacts};
#[allow(unused_imports)]
pub use octree::{
    VolumeData, box_project, find_oct, halton, sample_volume_grad, sample_volume_sdf,
};
#[allow(unused_imports)]
pub use primitives::{
    Attr, box_sdf, capsule, cylinder, ellipsoid, grad_box, grad_capsule, grad_cylinder,
    grad_ellipsoid, grad_sphere, radial_field, sphere,
};

/// 上游 `MJ_MINVAL`（f32）。
pub(crate) const MINVAL: f32 = 1.0e-15;

/// 上游 `mjtGeom` 中本片需要的编号（与 `collision.rs` 常量一致）。
pub const GEOM_PLANE: i32 = 0;
pub const GEOM_SPHERE: i32 = 2;
pub const GEOM_CAPSULE: i32 = 3;
pub const GEOM_ELLIPSOID: i32 = 4;
pub const GEOM_CYLINDER: i32 = 5;
pub const GEOM_BOX: i32 = 6;
pub const GEOM_MESH: i32 = 7;
pub const GEOM_SDF: i32 = 8;

/// 上游 `mjtGeom` 投影到本片实现的 SDF 分发类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SdfType {
    Plane,
    Sphere,
    Capsule,
    Cylinder,
    Box,
    Ellipsoid,
    /// 普通 mesh；仅当带 octree 体积数据时走 `sample_volume_*`。
    Mesh,
    /// 体积 SDF geom（`mjGEOM_SDF`）。
    Sdf,
}

impl SdfType {
    /// 由原生 geom 类型编号构造；未实现的类型返回 `None`。
    pub fn from_geom(type_id: i32) -> Option<Self> {
        match type_id {
            GEOM_PLANE => Some(Self::Plane),
            GEOM_SPHERE => Some(Self::Sphere),
            GEOM_CAPSULE => Some(Self::Capsule),
            GEOM_CYLINDER => Some(Self::Cylinder),
            GEOM_BOX => Some(Self::Box),
            GEOM_ELLIPSOID => Some(Self::Ellipsoid),
            GEOM_MESH => Some(Self::Mesh),
            GEOM_SDF => Some(Self::Sdf),
            _ => None,
        }
    }
}

/// 一个参与 SDF 判据的几何体：类型、尺寸与可选的 octree 体积数据。
#[derive(Clone, Copy, Debug)]
pub struct SdfShape<'a> {
    pub geom: SdfType,
    pub size: [f32; 3],
    pub volume: Option<&'a VolumeData>,
}

impl<'a> SdfShape<'a> {
    /// 以 geom 尺寸构造（无可选体积数据）。
    pub fn new(geom: SdfType, size: [f32; 3]) -> Self {
        Self {
            geom,
            size,
            volume: None,
        }
    }

    /// 附加 octree 体积数据；`MESH`/`SDF` 类型走体积分支。
    pub fn with_volume(mut self, volume: &'a VolumeData) -> Self {
        self.volume = Some(volume);
        self
    }
}
