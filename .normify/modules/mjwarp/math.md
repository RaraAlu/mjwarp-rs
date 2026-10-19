+++
uid = "dfdb6f0d"
id = "mjwarp.math"
state = "planned"
tags = ["planned", "engine", "math"]
fingerprint = "e511cd123d59fb65fdcc0cf13b128ca683406b37c62724c570ca90d98cd4c03b"

[name]
zh = "数学与空间代数"
en = "Math and spatial algebra"

[description]
zh = """
提供纯数学与空间代数。不持有设备资源。保留数值约定。
"""
en = """
Plan pure spatial algebra and numerical conventions without owning device or rendering resources.
"""

[[source]]
path = "src/math/mod.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::mul_quat"

[apis.description]
zh = """
宿主f32公式已有解析测试。 GPU等价验收仍待补齐。 此项不计99项公开符号。
"""
en = """
Implemented host f32 formula with analytic unit tests at the frozen upstream path. GPU numerical acceptance remains pending; outside the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::quat_mul_axis"

[apis.description]
zh = """
宿主f32公式已有解析测试。 GPU等价验收仍待补齐。 此项不计99项公开符号。
"""
en = """
Implemented host f32 formula with analytic unit tests at the frozen upstream path. GPU numerical acceptance remains pending; outside the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::rot_vec_quat"

[apis.description]
zh = """
宿主f32公式已有解析测试。 GPU等价验收仍待补齐。 此项不计99项公开符号。
"""
en = """
Implemented host f32 formula with analytic unit tests at the frozen upstream path. GPU numerical acceptance remains pending; outside the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::axis_angle_to_quat"

[apis.description]
zh = """
宿主f32公式已有解析测试。 GPU等价验收仍待补齐。 此项不计99项公开符号。
"""
en = """
Implemented host f32 formula with analytic unit tests at the frozen upstream path. GPU numerical acceptance remains pending; outside the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::quat_to_mat"

[apis.description]
zh = """
宿主f32公式已有解析测试。 GPU等价验收仍待补齐。 此项不计99项公开符号。
"""
en = """
Implemented host f32 formula with analytic unit tests at the frozen upstream path. GPU numerical acceptance remains pending; outside the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::quat_z2vec"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::quat_inv"

[apis.description]
zh = """
宿主f32公式已有解析测试。 GPU等价验收仍待补齐。 此项不计99项公开符号。
"""
en = """
Implemented host f32 formula with analytic unit tests at the frozen upstream path. GPU numerical acceptance remains pending; outside the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::inert_vec"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::motion_cross"

[apis.description]
zh = """
宿主f32公式已有解析测试。 GPU等价验收仍待补齐。 此项不计99项公开符号。
"""
en = """
Implemented host f32 formula with analytic unit tests at the frozen upstream path. GPU numerical acceptance remains pending; outside the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::motion_cross_force"

[apis.description]
zh = """
宿主f32公式已有解析测试。 GPU等价验收仍待补齐。 此项不计99项公开符号。
"""
en = """
Implemented host f32 formula with analytic unit tests at the frozen upstream path. GPU numerical acceptance remains pending; outside the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::quat_to_vel"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::quat_sub"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::quat_integrate"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::orthogonals"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::orthonormal"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::orthonormal_to_z"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::gjk_normalize"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::make_frame"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::normalize_with_norm"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::closest_segment_point"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::closest_segment_point_and_dist"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::closest_segment_to_segment_points"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::safe_div"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::upper_tri_index"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::upper_trid_index"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::free_bias_vel_blocks"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::lu_factor_6x6"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::lu_solve_6x6"

[apis.description]
zh = """
计划迁移内部数学入口。此项不计99项公开符号。
"""
en = """
Planned internal math contract from frozen _src/math.py. Not part of the 99 public-export denominator.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::Vec3"

[apis.description]
zh = """
基础辅助接口已有源码。 定义宿主f32数组约定。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Host f32 array alias with frozen conventions; not a native/device ABI.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::Quat"

[apis.description]
zh = """
基础辅助接口已有源码。 定义宿主f32数组约定。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Host f32 array alias with frozen conventions; not a native/device ABI.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::Mat3"

[apis.description]
zh = """
基础辅助接口已有源码。 定义宿主f32数组约定。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Host f32 array alias with frozen conventions; not a native/device ABI.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::math::SpatialVector"

[apis.description]
zh = """
基础辅助接口已有源码。 定义宿主f32数组约定。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Host f32 array alias with frozen conventions; not a native/device ABI.
"""

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::math::mul_quat"
to_api = "rpc:mjwarp_rs::diagnostics::Error"

[deps.label]
zh = "沿用公共错误表示"
en = "Common error types"
+++

## 基础源码落地

2026-10-07：补齐基础行为。
八项公式使用宿主f32。
四元数采用wxyz顺序。
共轭不除以模长平方。
公式不隐式归一化。
解析样本采用固定容限。
其余20项数学契约仍待实现。
GPU数学与独立数值验收仍待补齐。
本轮不激活产品模块。

证据：C:/Rust/mjwarp-rs/docs/foundation-architecture.md。
