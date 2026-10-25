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
