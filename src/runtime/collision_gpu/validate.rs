//! `CollisionKernel` 的 CPU 侧类型组合校验（不建 `CudaContext`、不跑 GPU）。
//!
//! 数值须与设备源 `support.cu` 的 `G5_*` 常量一致；B1 起 primitive 入口放行 plane 族 6 组合，
//! B2 起再放行 sphere/capsule 族 4 组合（sphere-sphere / sphere-capsule / sphere-cylinder /
//! capsule-capsule，无序对）。

use super::CollisionGpuError;

/// 上游 `mjtGeom` 常量（仅 CPU 侧组合校验用，数值须与设备源一致）。
const GEOM_PLANE: i32 = 0;
const GEOM_SPHERE: i32 = 2;
const GEOM_CAPSULE: i32 = 3;
const GEOM_ELLIPSOID: i32 = 4;
const GEOM_CYLINDER: i32 = 5;
const GEOM_BOX: i32 = 6;
const GEOM_MESH: i32 = 7;

/// primitive 入口接受（无序对）：既有 {SPHERE,BOX}/{CAPSULE,BOX}、B1 plane 族 6 组合
/// {PLANE, SPHERE/CAPSULE/ELLIPSOID/CYLINDER/BOX/MESH}、B2 sphere/capsule 族 4 组合
/// {SPHERE,SPHERE}/{SPHERE,CAPSULE}/{SPHERE,CYLINDER}/{CAPSULE,CAPSULE}；其它组合返回
/// `InvalidArgument`。
pub(crate) fn validate_primitive_types(type1: i32, type2: i32) -> Result<(), CollisionGpuError> {
    let unordered = |a: i32, b: i32| (type1 == a && type2 == b) || (type1 == b && type2 == a);
    let plane_other = |other: i32| {
        matches!(
            other,
            GEOM_SPHERE | GEOM_CAPSULE | GEOM_ELLIPSOID | GEOM_CYLINDER | GEOM_BOX | GEOM_MESH
        )
    };
    let plane_pair =
        (type1 == GEOM_PLANE && plane_other(type2)) || (type2 == GEOM_PLANE && plane_other(type1));
    if unordered(GEOM_SPHERE, GEOM_BOX)
        || unordered(GEOM_CAPSULE, GEOM_BOX)
        || unordered(GEOM_SPHERE, GEOM_SPHERE)
        || unordered(GEOM_SPHERE, GEOM_CAPSULE)
        || unordered(GEOM_SPHERE, GEOM_CYLINDER)
        || unordered(GEOM_CAPSULE, GEOM_CAPSULE)
        || plane_pair
    {
        return Ok(());
    }
    Err(CollisionGpuError::InvalidArgument(
        "primitive 只支持 SPHERE-BOX / CAPSULE-BOX、SPHERE-SPHERE / SPHERE-CAPSULE / SPHERE-CYLINDER / CAPSULE-CAPSULE 与 PLANE-{SPHERE,CAPSULE,ELLIPSOID,CYLINDER,BOX,MESH} 组合",
    ))
}

/// box-box 入口只接受 `(BOX, BOX)`（上游 NATIVECCD 关闭时把 `(BOX,BOX)` 从 CONVEX 改走
/// primitive `box_box_wrapper`）。其它组合返回 [`CollisionGpuError::InvalidArgument`]。
pub(crate) fn validate_boxbox_types(type1: i32, type2: i32) -> Result<(), CollisionGpuError> {
    if type1 == GEOM_BOX && type2 == GEOM_BOX {
        return Ok(());
    }
    Err(CollisionGpuError::InvalidArgument(
        "boxbox 只支持 BOX-BOX 组合",
    ))
}
