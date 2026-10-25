//! G10@P5：产品 NVRTC 设备源拼装。
//!
//! 设备源由 `src/physics/collision/cuda/*.cu` 的 16 个片段按**依赖拓扑序**拼接
//! （被依赖文件在前：common→geom→support→simplex→gjk_core→polytope→polytope_build→
//! epa→ccd→sdf_octree→sdf_math→sdf_narrowphase→hfield→contact→primitive→
//! primitive_wrapper），最后追加产品入口 `product.cu`。拼接前删除各片段里的本地
//! `#include "xxx.cu"` 行（只删引号 `.cu`），其它 `#include` 与 `#pragma once` 原样保留：
//! NVRTC 把拼好的整串当作**一个**翻译单元编译，本地 include 会重复展开。
//!
//! 拼装结果只由设备源内容决定（无时间戳/绝对路径），故 `source_sha256` 可作设备源指纹。

use sha2::{Digest, Sha256};

use crate::physics::{
    CCD_CUDA, COMMON_CUDA, CONTACT_CUDA, EPA_CUDA, GEOM_CUDA, GJK_CORE_CUDA, HFIELD_CUDA,
    POLYTOPE_BUILD_CUDA, POLYTOPE_CUDA, PRIMITIVE_CUDA, PRIMITIVE_WRAPPER_CUDA, SDF_MATH_CUDA,
    SDF_NARROWPHASE_CUDA, SDF_OCTREE_CUDA, SIMPLEX_CUDA, SUPPORT_CUDA,
};

/// 设备源片段与固定拼装顺序（`(文件名, 内容)`，依赖在前）。
pub(crate) const DEVICE_SOURCES: [(&str, &str); 16] = [
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
    ("contact.cu", CONTACT_CUDA),
    ("primitive.cu", PRIMITIVE_CUDA),
    ("primitive_wrapper.cu", PRIMITIVE_WRAPPER_CUDA),
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
