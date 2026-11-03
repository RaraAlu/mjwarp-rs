//! G10@P5：产品 GPU 入口的宿主数据类型（元参数、输入视图、回读结果）。
//!
//! 结构与字段顺序同时是 ABI 的宿主侧定义：`CollisionGpuMeta` 的标量顺序即
//! `product.cu::mjwarp_collision_args` 的标量顺序，`CollisionGpuInputs` 的 32 个
//! slice（23 项既有 + 9 项可选 mesh_poly）与探针 `DeviceArrays` 同口径。拆分只为控制
//! 单文件行数，公开路径不变（`runtime::collision_gpu::*` 仍由父模块 re-export）。

/// 设备侧标量参数（与探针 `meta.txt` 的标量一一对应，字段顺序即 ABI 顺序）。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CollisionGpuMeta {
    pub nworld: i32,
    pub ngeom: i32,
    pub g1: i32,
    pub g2: i32,
    pub type1: i32,
    pub type2: i32,
    pub geomgeomid: i32,
    pub naccdmax: i32,
    pub hfield_id: i32,
    pub oct_root: i32,
    pub noct: i32,
    pub tolerance: f32,
    pub margin: f32,
    pub gap: f32,
    pub gjk_iterations: i32,
    pub epa_iterations: i32,
    pub condim: i32,
    pub pairid0: i32,
    pub pairid1: i32,
    pub friction: [f32; 5],
    pub solref: [f32; 2],
    pub solreffriction: [f32; 2],
    pub solimp: [f32; 5],
    pub adhesion: f32,
    pub sdf_initpoints: i32,
    pub sdf_iterations: i32,
    pub naconmax: i32,
    pub efc_width: i32,
}

/// 一次发射的 32 个宿主输入数组（与探针 `DeviceArrays` 相同的 slice 口径）。
///
/// 模型静态字段（mesh/hfield/octree）按上游口径与 world 无关：调用方给一份共享数据。
/// `mesh_poly*` 9 项是 M0a sidecar 的可选输入：9 项全空（默认 `&[]`）时 sidecar 不构造、
/// `mesh_poly_args = null`（无 mesh polygon 数据、mesh 多接触功能不启用、行为不变）；
/// 部分为空或长度不自洽在 [`super::abi::validate_inputs`] 返回 `InvalidArgument`。
/// 9 项的形状语义与 pinned `mujoco_warp/_src/types.py` L1329-1337/L1864-1872 一致：
/// 逐 mesh（`nmesh`）/ 逐 polygon（`nmeshpoly`）/ 逐 vertex（`nmeshvert`）三段，
/// 已由 M0a 修正片 `2026-10-26-mesh-multicontact-m0a-fix-mesh-poly-shapes` 更正。
#[derive(Clone, Copy, Debug)]
pub struct CollisionGpuInputs<'a> {
    pub geom_xpos: &'a [f32],
    pub geom_xmat: &'a [f32],
    pub geom_size: &'a [f32],
    pub geom_margin: &'a [f32],
    pub geom_gap: &'a [f32],
    pub geom_rbound: &'a [f32],
    pub geom_aabb: &'a [f32],
    pub geom_type: &'a [i32],
    pub geom_dataid: &'a [i32],
    pub mesh_vertadr: &'a [i32],
    pub mesh_vertnum: &'a [i32],
    pub mesh_graphadr: &'a [i32],
    pub mesh_octadr: &'a [i32],
    pub mesh_vert: &'a [f32],
    pub mesh_graph: &'a [i32],
    /// M0a sidecar：逐 mesh 的 polygon 数（`nmesh`）。
    pub mesh_polynum: &'a [i32],
    /// M0a sidecar：逐 mesh 的首个 polygon 地址（`nmesh`）。
    pub mesh_polyadr: &'a [i32],
    /// M0a sidecar：逐 polygon 的法线（`nmeshpoly × 3` 个 f32）。
    pub mesh_polynormal: &'a [f32],
    /// M0a sidecar：逐 polygon 的顶点首地址（`nmeshpoly`）。
    pub mesh_polyvertadr: &'a [i32],
    /// M0a sidecar：逐 polygon 的顶点数（`nmeshpoly`）。
    pub mesh_polyvertnum: &'a [i32],
    /// M0a sidecar：全部 polygon 顶点索引（`nmeshpolyvert`）。
    pub mesh_polyvert: &'a [i32],
    /// M0a sidecar：逐 vertex 的映射首地址（`nmeshvert`）。
    pub mesh_polymapadr: &'a [i32],
    /// M0a sidecar：逐 vertex 的映射项数（`nmeshvert`）。
    pub mesh_polymapnum: &'a [i32],
    /// M0a sidecar：全部 vertex→polygon 映射索引（`nmeshpolymap`）。
    pub mesh_polymap: &'a [i32],
    pub hfield_size: &'a [f32],
    pub hfield_nrow: &'a [i32],
    pub hfield_ncol: &'a [i32],
    pub hfield_adr: &'a [i32],
    pub hfield_data: &'a [f32],
    pub oct_aabb: &'a [f32],
    pub oct_child: &'a [i32],
    pub oct_coeff: &'a [f32],
    pub meta: CollisionGpuMeta,
}

/// 回读结果：数组字段只保留前 `nacon` 行（与探针 `out1/` 落盘口径一致）。
#[derive(Clone, Debug)]
pub struct CollisionGpuOutput {
    pub nacon: i32,
    pub nccd: i32,
    /// 长度 = `nworld`（每 world 的 `OverflowType` 位汇总）。
    pub overflow: Vec<u32>,
    pub dist: Vec<f32>,
    pub pos: Vec<f32>,
    pub frame: Vec<f32>,
    pub includemargin: Vec<f32>,
    pub friction: Vec<f32>,
    pub solref: Vec<f32>,
    pub solreffriction: Vec<f32>,
    pub solimp: Vec<f32>,
    pub adhesion: Vec<f32>,
    pub dim: Vec<i32>,
    pub geom: Vec<i32>,
    pub efc_address: Vec<i32>,
    pub worldid: Vec<i32>,
    pub contact_type: Vec<i32>,
    pub geomcollisionid: Vec<i32>,
}

/// 方案 C：mesh 多接触容量维度（宿主诊断载荷）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshCapacityDimension {
    /// 单 polygon 顶点数（设备 `G5_MC_MESH_FACE_MAX`，上游 `npolygonmax` 的模型级最大值）。
    Face,
    /// 单顶点相邻 polygon 数（设备 `G5_MC_MESH_DEG_MAX`，上游 `nmeshdegmax` 的模型级最大值）。
    Degree,
}

impl MeshCapacityDimension {
    /// 诊断/日志用的维度标签。
    pub fn label(self) -> &'static str {
        match self {
            Self::Face => "face",
            Self::Degree => "degree",
        }
    }
}

/// 设备侧 mesh 多接触固定容量上限（`multicontact_geom.cu` 宏的宿主镜像；
/// 模块内 `include_str!` 断言守护两者一致，A 方案改宏后必须同步）。
pub const MESH_MULTICONTACT_FACE_MAX: i32 = 20;
/// 单顶点相邻 polygon 数上限（同上，镜像 `G5_MC_MESH_DEG_MAX`）。
pub const MESH_MULTICONTACT_DEGREE_MAX: i32 = 10;
/// 方案 C 的拒绝路径标识（写进诊断载荷，便于产品日志定位）。
pub const MESH_MULTICONTACT_REJECT_PATH: &str =
    "runtime::collision_gpu::inputs_from_snapshot (product collision plan)";

/// 方案 C：单个维度的超界记录（维度、实际值、上限值）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshCapacityViolation {
    /// 超界维度。
    pub dimension: MeshCapacityDimension,
    /// 模型级实际最大值（`max(polyvertnum)` 或 `max(polymapnum)`）。
    pub actual: i32,
    /// 设备固定上限。
    pub limit: i32,
}

/// 方案 C：用已有 mesh 数组算模型级 `npolygonmax`/`nmeshdegmax`，与设备固定容量比较。
///
/// `polyvertnum` = 逐 polygon 顶点数（`nmeshpoly`）；`polymapnum` = 逐 vertex 相邻 polygon 数
/// （`nmeshvert`）。返回空 vec 表示未超界（无 mesh sidecar 也返回空 → 行为与改动前一致）。
pub fn mesh_multicontact_capacity_violations(
    polyvertnum: &[i32],
    polymapnum: &[i32],
) -> Vec<MeshCapacityViolation> {
    let mut violations = Vec::new();
    if let Some(actual) = polyvertnum.iter().copied().max()
        && actual > MESH_MULTICONTACT_FACE_MAX
    {
        violations.push(MeshCapacityViolation {
            dimension: MeshCapacityDimension::Face,
            actual,
            limit: MESH_MULTICONTACT_FACE_MAX,
        });
    }
    if let Some(actual) = polymapnum.iter().copied().max()
        && actual > MESH_MULTICONTACT_DEGREE_MAX
    {
        violations.push(MeshCapacityViolation {
            dimension: MeshCapacityDimension::Degree,
            actual,
            limit: MESH_MULTICONTACT_DEGREE_MAX,
        });
    }
    violations
}
