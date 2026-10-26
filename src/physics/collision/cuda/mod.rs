#![allow(dead_code)]
//! G10@P5 G5：碰撞 GPU 内核（NVRTC 设备源码，对齐 pinned `mujoco_warp`）。
//! 本模块只持有设备源码与宿主侧编译/接线入口；运行需要 `cuda-probe` 与 GPU 窗口。
//! 设备源码分块与 host 语义一一对应：`common`（math）→ `geom`（视图）→ `support`（支撑点）
//! → `simplex`/`polytope`/`ccd`（G2）→ `sdf`（G3）→ `hfield`（G4）→ 接触写入/截断。
//!
//! 编译模式（两套数字必须分开记录，不得混用）：
//! - **调试模式、非判据**：NVRTC `--fmad=false` 时设备表达式与 host 源码顺序逐位可对照，
//!   只用于定位搬运偏差，**不能**作为 GPU 验收结论；
//! - **正式判据**：对同配方上游 f32 GPU，按 pinned `ccd` PTX 的**全 fma 收缩**（显式 `mul_add`）
//!   编译，要求 ≤4 ULP、位级优先。
//!
//! 设备源码在 G5 各子步接入内核前暂由宿主侧持有。

/// 公共设备数学（与 host `gjk/math.rs` 逐式对应）。
pub(crate) const COMMON_CUDA: &str = include_str!("common.cu");
/// 设备侧几何视图（与 host `gjk::Geom`/`MeshView` 字段对应）。
pub(crate) const GEOM_CUDA: &str = include_str!("geom.cu");
/// 支撑点函数（与 host `gjk/support.rs` 逐式对应，含 mesh hillclimb）。
pub(crate) const SUPPORT_CUDA: &str = include_str!("support.cu");

/// GJK 距离子算法（与 host `gjk/simplex.rs` 逐式对应）。
pub(crate) const SIMPLEX_CUDA: &str = include_str!("simplex.cu");
/// GJK 主循环（与 host `gjk/core.rs` 逐式对应）。
pub(crate) const GJK_CORE_CUDA: &str = include_str!("gjk_core.cu");

/// EPA polytope 构造与 witness（与 host `gjk/polytope.rs` 对应）。
pub(crate) const POLYTOPE_CUDA: &str = include_str!("polytope.cu");
/// EPA polytope 初始构造（与 host `gjk/polytope_build.rs` 对应）。
pub(crate) const POLYTOPE_BUILD_CUDA: &str = include_str!("polytope_build.cu");
/// EPA 迭代（与 host `gjk/epa.rs` 对应）。
pub(crate) const EPA_CUDA: &str = include_str!("epa.cu");
/// ccd 两阶段驱动（与 host `gjk/phase.rs` + `gjk/ccd.rs` 对应）。
pub(crate) const CCD_CUDA: &str = include_str!("ccd.cu");

/// SDF octree 采样（与 host `sdf/octree.rs` 对应）。
pub(crate) const SDF_OCTREE_CUDA: &str = include_str!("sdf_octree.cu");
/// SDF 原语/梯度/clearance/Armijo/下降（与 host `sdf/primitives.rs` + `sdf/descent.rs` 对应）。
pub(crate) const SDF_MATH_CUDA: &str = include_str!("sdf_math.cu");
/// SDF narrowphase（与 host `sdf/narrowphase.rs` 对应：AABB 交集 + Halton 初始点）。
pub(crate) const SDF_NARROWPHASE_CUDA: &str = include_str!("sdf_narrowphase.cu");

/// hfield prism CCD（与 host `hfield/filter.rs` + `hfield/driver.rs` 对应：
/// `_hfield_filter` 粗筛、prism 六顶点、`ccd()` 组合、0..3 四接触 + `MJ_MAXCONPAIR` 溢出）。
pub(crate) const HFIELD_CUDA: &str = include_str!("hfield.cu");
/// 接触写入与容量门（与上游 `collision_core.py::write_contact` +
/// `collision_convex.py::eval_ccd_write_contact` 对应：`worldid`/`efc_address`/`OverflowType`、
/// `naconmax`/`naccdmax` 截断不越界）。
pub(crate) const CONTACT_CUDA: &str = include_str!("contact.cu");
/// primitive narrowphase 数值内核（与上游 `collision_primitive_core.py::sphere_box` /
/// `capsule_box` 逐式对应）。
pub(crate) const PRIMITIVE_CUDA: &str = include_str!("primitive.cu");
/// primitive wrapper 接触写入（与上游 `collision_primitive.py::sphere_box_wrapper` /
/// `capsule_box_wrapper` 对应：`write_contact` 的 `id_` 序号与计数门）。
pub(crate) const PRIMITIVE_WRAPPER_CUDA: &str = include_str!("primitive_wrapper.cu");
// B1 plane 族设备源（common/convex/shapes/wrapper）的 `include_str!` 直接放在拼装处
// `runtime/collision_gpu/source.rs`（与 `PRODUCT_SOURCE` 同口径）：若在此定义并经
// `crate::physics` re-export，则必须改 `src/physics/mod.rs`，会使 T4 `physics_core`
// 组合身份（`src/physics/mod.rs` + 4 个子模块）漂移，触发冻结捕获身份审计。
/// multicontact 几何助手（上游 `collision_gjk.py::_feature_dim` / `_box_normals` /
/// `_cylinder_normals` / `_aligned_faces` / `_aligned_face_edge` / `*_face` / `*_edge_normals`）。
pub(crate) const MULTICONTACT_GEOM_CUDA: &str = include_str!("multicontact_geom.cu");
/// multicontact 多边形裁剪（上游 `collision_gjk.py::_polygon_clip`）。
pub(crate) const MULTICONTACT_CLIP_CUDA: &str = include_str!("multicontact_clip.cu");
/// multicontact 主函数（上游 `collision_gjk.py::multicontact`：<=4 个接触；本片只实现
/// BOX/CYLINDER 分支，mesh 分支回退单接触并记未覆盖）。
pub(crate) const MULTICONTACT_CUDA: &str = include_str!("multicontact.cu");
/// primitive box-box 数值核心（上游 `collision_primitive_core.py::box_box` 的 8 槽候选）。
pub(crate) const BOXBOX_CUDA: &str = include_str!("boxbox.cu");
/// primitive box-box wrapper（上游 `collision_primitive.py::box_box_wrapper`：
/// NATIVECCD 关闭时 `(BOX, BOX)` 的 primitive dispatch）。
pub(crate) const BOXBOX_WRAPPER_CUDA: &str = include_str!("boxbox_wrapper.cu");
