+++
uid = "a3e69712"
id = "mjwarp.model.parameters"
fingerprint = "74bc272d7d9d9a3641799be45d0ab68c6be8ab9bbe3681ab237a59a5cc5c8c1d"

[name]
zh = "逐字段批量参数子集"
en = "Per-field parameter batch subsets"

[description]
zh = """
保留十三项运动学独立参数；新增七项碰撞宿主独立周期。整数拓扑共享，不提供完整Model或碰撞上传。
"""
en = """
Thirteen existing kinematic parameters and seven collision host-only independent cycles; integer topology shared, not full Model or collision upload.
"""

[[source]]
path = "src/model/parameters.rs"

[[source]]
path = "src/model/collision_parameters.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::ParameterBatch"

[apis.description]
zh = """
字段批量辅助契约。
"""
en = """
Strict-subset parameter batching helper.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::KinematicsParameter"

[apis.description]
zh = """
字段批量辅助契约。
"""
en = """
Strict-subset parameter batching helper.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::KinematicsParameters"

[apis.description]
zh = """
字段批量辅助契约。
"""
en = """
Strict-subset parameter batching helper.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::PrimitiveGeometryParameter"

[apis.description]
zh = """
已实现碰撞七字段独立宿主周期与只读视图。复用ParameterBatch；拓扑共享。未提供上传或GPU物理。
"""
en = """
Implemented host-only independent cycles for seven collision fields and immutable views, using ParameterBatch with shared topology; no upload or GPU physics.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::PrimitiveGeometryParameters"

[apis.description]
zh = """
已实现碰撞七字段独立宿主周期与只读视图。复用ParameterBatch；拓扑共享。未提供上传或GPU物理。
"""
en = """
Implemented host-only independent cycles for seven collision fields and immutable views, using ParameterBatch with shared topology; no upload or GPU physics.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::PrimitiveGeometryHostBatch"

[apis.description]
zh = """
已实现碰撞七字段独立宿主周期与只读视图。复用ParameterBatch；拓扑共享。未提供上传或GPU物理。
"""
en = """
Implemented host-only independent cycles for seven collision fields and immutable views, using ParameterBatch with shared topology; no upload or GPU physics.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::PrimitiveGeometryTopology"

[apis.description]
zh = """
已实现碰撞七字段独立宿主周期与只读视图。复用ParameterBatch；拓扑共享。未提供上传或GPU物理。
"""
en = """
Implemented host-only independent cycles for seven collision fields and immutable views, using ParameterBatch with shared topology; no upload or GPU physics.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::model::PrimitiveGeometryWorld"

[apis.description]
zh = """
已实现碰撞七字段独立宿主周期与只读视图。复用ParameterBatch；拓扑共享。未提供上传或GPU物理。
"""
en = """
Implemented host-only independent cycles for seven collision fields and immutable views, using ParameterBatch with shared topology; no upload or GPU physics.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.attached"
to_api = "rpc:mjwarp_rs::model::AttachedModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::model::PrimitiveGeometryHostBatch"
to_api = "rpc:mjwarp_rs::model::PrimitiveGeometryInput"

[deps.label]
zh = "绑定已检查宿主几何"
en = "Bind checked host geometry"
+++

2026-10-14：既有ParameterBatch源码不变。新增七字段可独立取模，不要求周期整除或小于世界数；原active状态与UID保持。
