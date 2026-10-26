//! G10@P5：产品 NVRTC 设备源拼装。
//!
//! 设备源由 `src/physics/collision/cuda/*.cu` 的 28 个片段按**依赖拓扑序**拼接
//! （被依赖文件在前：common→geom→support→simplex→gjk_core→polytope→polytope_build→
//! epa→ccd→sdf_octree→sdf_math→sdf_narrowphase→hfield→multicontact_geom→multicontact_mesh→
//! multicontact_clip→multicontact→contact→primitive→primitive_wrapper→boxbox→
//! boxbox_wrapper→primitive_plane_common→primitive_plane_convex→primitive_plane_shapes→
//! primitive_plane_wrapper→primitive_sphere_capsule_shapes→primitive_sphere_capsule_wrapper），
//! 最后追加产品入口 `product.cu`。拼接前删除各片段里的本地
//! `#include "xxx.cu"` 行（只删引号 `.cu`），其它 `#include` 与 `#pragma once` 原样保留：
//! NVRTC 把拼好的整串当作**一个**翻译单元编译，本地 include 会重复展开。
//!
//! 拼装结果只由设备源内容决定（无时间戳/绝对路径），故 `source_sha256` 可作设备源指纹。

use sha2::{Digest, Sha256};

use crate::physics::{
    BOXBOX_CUDA, BOXBOX_WRAPPER_CUDA, CCD_CUDA, COMMON_CUDA, CONTACT_CUDA, EPA_CUDA, GEOM_CUDA,
    GJK_CORE_CUDA, HFIELD_CUDA, MULTICONTACT_CLIP_CUDA, MULTICONTACT_CUDA, MULTICONTACT_GEOM_CUDA,
    POLYTOPE_BUILD_CUDA, POLYTOPE_CUDA, PRIMITIVE_CUDA, PRIMITIVE_WRAPPER_CUDA, SDF_MATH_CUDA,
    SDF_NARROWPHASE_CUDA, SDF_OCTREE_CUDA, SIMPLEX_CUDA, SUPPORT_CUDA,
};

// B1 plane 族四个片段直接在拼装处取文本（与下方 `PRODUCT_SOURCE` 同口径）。不经
// `crate::physics` re-export 是刻意的：那需要改 `src/physics/mod.rs`，而该文件是 T4
// `physics_core` 组合身份的一部分，改动会使冻结捕获身份审计
// （`runtime::transfer::kernel::recapture::tests_host::preserves_current_sources_and_all_thirty_variants`）
// 漂移。设备源内容与拼装结果不受影响。
const PRIMITIVE_PLANE_COMMON_CUDA: &str =
    include_str!("../../physics/collision/cuda/primitive_plane_common.cu");
const PRIMITIVE_PLANE_CONVEX_CUDA: &str =
    include_str!("../../physics/collision/cuda/primitive_plane_convex.cu");
const PRIMITIVE_PLANE_SHAPES_CUDA: &str =
    include_str!("../../physics/collision/cuda/primitive_plane_shapes.cu");
const PRIMITIVE_PLANE_WRAPPER_CUDA: &str =
    include_str!("../../physics/collision/cuda/primitive_plane_wrapper.cu");

// B2 sphere/capsule 族两个片段：同样在拼装处本地取文本（不动 `src/physics/mod.rs`，
// 保持 T4 `physics_core` 组合身份不变）。
const PRIMITIVE_SPHERE_CAPSULE_SHAPES_CUDA: &str =
    include_str!("../../physics/collision/cuda/primitive_sphere_capsule_shapes.cu");
const PRIMITIVE_SPHERE_CAPSULE_WRAPPER_CUDA: &str =
    include_str!("../../physics/collision/cuda/primitive_sphere_capsule_wrapper.cu");

// M2 convex-vs-mesh multicontact 片段：同样在拼装处本地取文本（不动 `src/physics/mod.rs`，
// 保持 T4 `physics_core` 组合身份不变）。
const MULTICONTACT_MESH_CUDA: &str =
    include_str!("../../physics/collision/cuda/multicontact_mesh.cu");

/// 设备源片段与固定拼装顺序（`(文件名, 内容)`，依赖在前）。
pub(crate) const DEVICE_SOURCES: [(&str, &str); 28] = [
    ("common.cu", COMMON_CUDA),
    ("geom.cu", GEOM_CUDA),
    ("support.cu", SUPPORT_CUDA),
    ("simplex.cu", SIMPLEX_CUDA),
    ("gjk_core.cu", GJK_CORE_CUDA),
    ("polytope.cu", POLYTOPE_CUDA),
    ("polytope_build.cu", POLYTOPE_BUILD_CUDA),
    ("epa.cu", EPA_CUDA),
    ("ccd.cu", CCD_CUDA),
    ("sdf_octree.cu", SDF_OCTREE_CUDA),
    ("sdf_math.cu", SDF_MATH_CUDA),
    ("sdf_narrowphase.cu", SDF_NARROWPHASE_CUDA),
    ("hfield.cu", HFIELD_CUDA),
    ("multicontact_geom.cu", MULTICONTACT_GEOM_CUDA),
    // M2：mesh 助手（依赖 multicontact_geom 的 mesh 视图）在 clip/multicontact 之前。
    ("multicontact_mesh.cu", MULTICONTACT_MESH_CUDA),
    ("multicontact_clip.cu", MULTICONTACT_CLIP_CUDA),
    ("multicontact.cu", MULTICONTACT_CUDA),
    ("contact.cu", CONTACT_CUDA),
    ("primitive.cu", PRIMITIVE_CUDA),
    ("primitive_wrapper.cu", PRIMITIVE_WRAPPER_CUDA),
    ("boxbox.cu", BOXBOX_CUDA),
    ("boxbox_wrapper.cu", BOXBOX_WRAPPER_CUDA),
    // B1 plane 族（拼装顺序：primitive_wrapper→boxbox→boxbox_wrapper→plane common/convex/shapes/
    // wrapper→product；B1 时 25 段、M2 起 28 段；0 新 ABI 入口，入口仍是 mjwarp_collision_primitive）。
    ("primitive_plane_common.cu", PRIMITIVE_PLANE_COMMON_CUDA),
    ("primitive_plane_convex.cu", PRIMITIVE_PLANE_CONVEX_CUDA),
    ("primitive_plane_shapes.cu", PRIMITIVE_PLANE_SHAPES_CUDA),
    ("primitive_plane_wrapper.cu", PRIMITIVE_PLANE_WRAPPER_CUDA),
    // B2 sphere/capsule 族（拼装顺序：plane wrapper→shapes→wrapper→product；B2 时 27 段、
    // M2 起 28 段（多 multicontact_mesh），0 新 ABI 入口，入口仍是 mjwarp_collision_primitive）。
    (
        "primitive_sphere_capsule_shapes.cu",
        PRIMITIVE_SPHERE_CAPSULE_SHAPES_CUDA,
    ),
    (
        "primitive_sphere_capsule_wrapper.cu",
        PRIMITIVE_SPHERE_CAPSULE_WRAPPER_CUDA,
    ),
];

/// 产品侧 CUDA 入口（追加在拼接结果最后；`d_contact_out` 等由前面的片段提供）。
pub(crate) const PRODUCT_SOURCE: &str = include_str!("../../physics/collision/cuda/product.cu");

/// 是否为 `#include "xxx.cu"` 形式的本地设备源引用（行尾换行符保留在行内）。
pub(crate) fn is_local_cu_include(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix("#include") else {
        return false;
    };
    let Some(rest) = rest.trim_start().strip_prefix('"') else {
        return false;
    };
    let Some(end) = rest.find('"') else {
        return false;
    };
    rest[..end].ends_with(".cu")
}

/// 删除所有本地 `#include "*.cu"` 行；其它内容（含 `#pragma once`、系统头）不变。
pub(crate) fn strip_local_includes(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if is_local_cu_include(line) {
            continue;
        }
        output.push_str(line);
    }
    output
}

/// 按固定顺序拼装完整设备源（确定性：同一输入必得同一输出）。
pub(crate) fn assemble_device_source() -> String {
    let mut source = String::new();
    for (_, text) in DEVICE_SOURCES {
        source.push_str(&strip_local_includes(text));
        // 片段末行缺换行时补齐，避免与下一片段粘连。
        if !source.ends_with('\n') {
            source.push('\n');
        }
    }
    source.push_str(PRODUCT_SOURCE);
    source
}

/// 拼装结果的 SHA-256（小写十六进制）；用于锁死设备源指纹。
pub(crate) fn source_sha256() -> String {
    format!("{:x}", Sha256::digest(assemble_device_source().as_bytes()))
}
