//! G10@P5 前置片：上游 `collision_gjk.py`（GJK/EPA 凸碰撞）的宿主侧复刻。
//!
//! 对齐冻结上游 `mujoco_warp._src.collision_gjk`（提交 `71da24d956…`，Apache-2.0）
//! 与 `collision_convex.py` 的 `eval_ccd_write_contact` 接触装配子集：
//!
//! - `ccd()`：球/胶囊 shrink → GJK（`is_discrete` 分支、Frank-Wolfe 间隙、分离判据、
//!   `_S3D/_S2D/_S1D` 子算法、`x_norm<1e-4` 方向调优）→ 浅穿透 `_inflate` /
//!   深穿透 EPA（polytope 构造、HORIZON 溢出、`FLOAT_MAX` 退化）。
//! - `support()`：SPHERE/BOX/CAPSULE/ELLIPSOID/CYLINDER/MESH/HFIELD/TRIANGLE/FLEX
//!   与 margin 尾项。
//! - `multicontact()`：按 feature 法线生成 ≤4 接触（`_polygon_clip`）。
//! - 接触装配：原点平移、margin 膨胀修正、frame 由 witness 差分生成。
//!
//! 未覆盖：GPU 内核与图条件、`naccdmax`/接触原子计数（调用方负责）、
//! hfield 四接触后处理（G4）、SDF 分支（G3）、flex 专用 GEOM 路径。
//! 数值以 f32 全链复刻；与 pinned 上游的位级对照见证据目录。
// 忠实对齐上游逐句结构：内部函数参数多、显式下标循环、Default 后逐字段赋值等属刻意写法，
// 在本模块内统一豁免对应 clippy lint（产物语义与上游逐行对应，拆包装不影响可读性）。
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::manual_memcpy)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::identity_op)]
#![allow(clippy::erasing_op)]

#[cfg(test)]
mod case_compare;
#[cfg(test)]
mod case_data;
#[cfg(test)]
mod case_evidence;
mod ccd;
mod contact;
mod core;
mod epa;
pub(crate) mod math;
mod multicontact;
mod phase;
mod polytope;
mod polytope_build;
mod simplex;
mod support;
#[cfg(test)]
mod tests;

pub use ccd::ccd;
pub use contact::{CcdContact, CcdOutput, ContactOptions, ccd_pair};
pub use phase::{CcdOptions, CcdResult};

/// 上游 `collision_gjk.SupportPoint`。
#[derive(Clone, Copy, Debug, Default)]
pub struct SupportPoint {
    pub point: [f32; 3],
    /// mesh 支撑的缓存顶点索引（`-1` 表示无缓存）。
    pub cached_index: i32,
    /// 上游 `vertex_index`（box 顶点编码、cylinder 顶/底、hfield 边标记等）。
    pub vertex_index: i32,
}

/// 上游 `mjtGeom` 类型编号（冻结 3.15 `types.GeomType`）。
pub mod geom_type {
    pub const PLANE: i32 = 0;
    pub const HFIELD: i32 = 1;
    pub const SPHERE: i32 = 2;
    pub const CAPSULE: i32 = 3;
    pub const ELLIPSOID: i32 = 4;
    pub const CYLINDER: i32 = 5;
    pub const BOX: i32 = 6;
    pub const MESH: i32 = 7;
    pub const SDF: i32 = 8;
    pub const FLEX: i32 = 105;
    pub const TRIANGLE: i32 = 999;
}

/// 上游 `collision_gjk.py` 的浮点常量。
pub(crate) const FLOAT_MIN: f32 = -1.0e30;
pub(crate) const FLOAT_MAX: f32 = 1.0e30;
pub(crate) const MINVAL: f32 = 1.0e-15;
pub(crate) const MINVAL2: f32 = 1.0e-30;
pub(crate) const MAXVAL: f32 = 1.0e15;
pub(crate) const MAXVAL2: f32 = 1.0e30;
/// polytope 到原点的最小内部距离。
pub(crate) const MIN_DIST2: f32 = 1.0e-10;
pub(crate) const MIN_DIST3: f32 = 1.0e-10;
pub(crate) const MIN_DIST4: f32 = 1.0e-17;
/// EPA 的最小容差。
pub(crate) const MIN_EPATOL: f32 = 1.0e-7;
/// `cos(0.0889)`（`wp.static` f64 计算后按 f32 落地）。
pub(crate) const FACE_TOL: f32 = f32::from_bits(0x3F7E_FD33);
/// `sin(0.0889)`。
pub(crate) const EDGE_TOL: f32 = f32::from_bits(0x3DB5_D3D5);
/// multicontact 平面-线段相交容差。
pub(crate) const INTERSECT_TOL: f32 = 0.000_000_3;

/// EPA face 位域：每顶点 10 bit；最高两位为 invalid/deleted 标记。
pub(crate) const FACE_DELETED_BIT: u32 = 0x8000_0000;
pub(crate) const FACE_INVALID_BIT: u32 = 0x4000_0000;
pub(crate) const FACE_INVALID_OR_DELETED_MASK: u32 = 0xC000_0000;

/// 上游 `types.MJ_MAX_EPAHORIZON`。
pub(crate) const MJ_MAX_EPAHORIZON: usize = 24;
/// 上游 `types.MJ_MAX_EPAFACES`。
pub(crate) const MJ_MAX_EPAFACES: usize = 5;
/// 上游 `types.MJ_MAXCONPAIR`（`mujoco.mjMAXCONPAIR`）。
pub const MJ_MAXCONPAIR: usize = 50;

/// 16 边形圆柱端面近似的 cos 表（`_CYLINDER_COS_16`）。
pub(crate) const CYLINDER_COS_16: [f32; 16] = [
    f32::from_bits(0x3F80_0000),
    f32::from_bits(0x3F6C_835E),
    f32::from_bits(0x3F35_04F3),
    f32::from_bits(0x3EC3_EF15),
    f32::from_bits(0x248D_3132),
    f32::from_bits(0xBEC3_EF15),
    f32::from_bits(0xBF35_04F3),
    f32::from_bits(0xBF6C_835E),
    f32::from_bits(0xBF80_0000),
    f32::from_bits(0xBF6C_835E),
    f32::from_bits(0xBF35_04F3),
    f32::from_bits(0xBEC3_EF15),
    f32::from_bits(0xA553_C9CA),
    f32::from_bits(0x3EC3_EF15),
    f32::from_bits(0x3F35_04F3),
    f32::from_bits(0x3F6C_835E),
];

/// 16 边形圆柱端面近似的 sin 表（`_CYLINDER_SIN_16`）。
pub(crate) const CYLINDER_SIN_16: [f32; 16] = [
    f32::from_bits(0x0000_0000),
    f32::from_bits(0x3EC3_EF15),
    f32::from_bits(0x3F35_04F3),
    f32::from_bits(0x3F6C_835E),
    f32::from_bits(0x3F80_0000),
    f32::from_bits(0x3F6C_835E),
    f32::from_bits(0x3F35_04F3),
    f32::from_bits(0x3EC3_EF15),
    f32::from_bits(0x250D_3132),
    f32::from_bits(0xBEC3_EF15),
    f32::from_bits(0xBF35_04F3),
    f32::from_bits(0xBF6C_835E),
    f32::from_bits(0xBF80_0000),
    f32::from_bits(0xBF6C_835E),
    f32::from_bits(0xBF35_04F3),
    f32::from_bits(0xBEC3_EF15),
];

/// 上游 `OverflowType` 位（本片只产生 `CCD` 与 `EPA_HORIZON`）。
pub mod overflow_type {
    pub const CCD: u32 = 1 << 4;
    pub const HFIELD: u32 = 1 << 5;
    pub const EPA_HORIZON: u32 = 1 << 8;
    pub const ALL: u32 = 4095;
}

/// 上游 `collision_core.Geom` 的宿主侧视图（mesh 拓扑为借用切片）。
#[derive(Clone, Copy, Debug)]
pub struct Geom<'a> {
    pub pos: [f32; 3],
    /// 行优先 3×3 旋转矩阵。
    pub rot: [f32; 9],
    pub size: [f32; 3],
    pub margin: f32,
    /// hfield/flex 六顶点（hfield 由调用方在世界系预置，flex 为 4 点）。
    pub polyvert: [[f32; 3]; 6],
    pub vertadr: i32,
    pub vertnum: i32,
    pub graphadr: i32,
    pub mesh: Option<&'a MeshView<'a>>,
    /// 上游 `geom.index`（mesh 支撑热启动缓存）。
    pub index: i32,
}

/// 上游 mesh 拓扑字段（`mesh_*`）。
#[derive(Clone, Copy, Debug)]
pub struct MeshView<'a> {
    pub vert: &'a [[f32; 3]],
    pub graph: &'a [i32],
    /// 本 geom 的 `mesh_polyadr`（多边形数组基址）。
    pub polyadr: i32,
    pub polynormal: &'a [[f32; 3]],
    pub polyvertadr: &'a [i32],
    pub polyvertnum: &'a [i32],
    pub polyvert: &'a [i32],
    pub polymapadr: &'a [i32],
    pub polymapnum: &'a [i32],
    pub polymap: &'a [i32],
}

impl<'a> Geom<'a> {
    /// 构造无 mesh 拓扑的解析几何。
    pub fn simple(pos: [f32; 3], rot: [f32; 9], size: [f32; 3], margin: f32) -> Self {
        Self {
            pos,
            rot,
            size,
            margin,
            polyvert: [[0.0; 3]; 6],
            vertadr: -1,
            vertnum: -1,
            graphadr: -1,
            mesh: None,
            index: -1,
        }
    }
}

/// 上游 `collision_gjk.GJKResult`。
#[derive(Clone, Copy, Debug)]
pub(crate) struct GjkResult {
    pub separated: bool,
    pub dist: f32,
    pub x1: [f32; 3],
    pub x2: [f32; 3],
    pub dim: usize,
    pub simplex: [[f32; 3]; 4],
    pub simplex1: [[f32; 3]; 4],
    pub simplex2: [[f32; 3]; 4],
    pub simplex_index1: [i32; 4],
    pub simplex_index2: [i32; 4],
    pub index1: i32,
    pub index2: i32,
}

impl Default for GjkResult {
    fn default() -> Self {
        Self {
            separated: false,
            dist: 0.0,
            x1: [0.0; 3],
            x2: [0.0; 3],
            dim: 0,
            simplex: [[0.0; 3]; 4],
            simplex1: [[0.0; 3]; 4],
            simplex2: [[0.0; 3]; 4],
            simplex_index1: [0; 4],
            simplex_index2: [0; 4],
            index1: -1,
            index2: -1,
        }
    }
}

/// 上游 `collision_gjk.Polytope`：EPA 用的重心 polytope。
pub(crate) struct Polytope {
    pub status: i32,
    /// 依次交错存放 geom1/geom2 顶点：`vert[2*i]` 属 geom1，`vert[2*i+1]` 属 geom2。
    pub vert: Vec<[f32; 3]>,
    pub vert_index: Vec<i32>,
    pub nvert: usize,
    pub center: [f32; 3],
    pub face: Vec<u32>,
    pub face_pr: Vec<[f32; 3]>,
    pub face_norm2: Vec<f32>,
    pub nface: usize,
    pub horizon: Vec<i32>,
    /// 上游以 `-1` 表示 horizon 溢出。
    pub nhorizon: i32,
}

impl Polytope {
    /// 按上游缓冲容量分配：顶点 `10+2I`、面 `6+5I`、horizon `24`。
    pub(crate) fn with_capacity(epa_iterations: usize) -> Self {
        Self {
            status: 0,
            vert: vec![[0.0; 3]; 10 + 2 * epa_iterations],
            vert_index: vec![0; 10 + 2 * epa_iterations],
            nvert: 0,
            center: [0.0; 3],
            face: vec![0; 6 + MJ_MAX_EPAFACES * epa_iterations],
            face_pr: vec![[0.0; 3]; 6 + MJ_MAX_EPAFACES * epa_iterations],
            face_norm2: vec![0.0; 6 + MJ_MAX_EPAFACES * epa_iterations],
            nface: 0,
            horizon: vec![0; MJ_MAX_EPAHORIZON],
            nhorizon: 0,
        }
    }
}
